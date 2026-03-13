// src/mcp/server.rs
// OpenClaw MCP tool server — exposes Hyperliquid tools over JSON-RPC stdio

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{error, info};

use crate::analysis::signals::{MomentumEngine, print_signal};
use crate::trading::client::HyperliquidClient;
use crate::trading::exchange::ExchangeClient;

// ── MCP protocol types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

impl McpResponse {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(McpError { code, message: message.into() }),
        }
    }
}

// ── Tool definitions (sent to OpenClaw on initialize) ─────────────────────────

fn tool_definitions() -> Value {
    json!({
        "tools": [
            {
                "name": "hl_price",
                "description": "Get current mark price for a Hyperliquid perpetual",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string", "description": "Asset ticker, e.g. BTC" }
                    },
                    "required": ["coin"]
                }
            },
            {
                "name": "hl_meta",
                "description": "List all 228+ tradeable perpetuals on Hyperliquid",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "hl_market_scan",
                "description": "Scan all markets for momentum signals and trading opportunities",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "top_n": { "type": "integer", "description": "Return top N results (default 10)" }
                    }
                }
            },
            {
                "name": "hl_analyze",
                "description": "Deep momentum analysis for a specific asset",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string", "description": "Asset ticker" }
                    },
                    "required": ["coin"]
                }
            },
            {
                "name": "hl_balance",
                "description": "Get account equity, margin usage, and summary",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "hl_positions",
                "description": "List all open perpetual positions with P&L",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "hl_orders",
                "description": "List all open limit orders",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "hl_fills",
                "description": "Recent trade fills",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "description": "Number of fills to return (default 20)" }
                    }
                }
            },
            {
                "name": "hl_market_buy",
                "description": "Place a market buy order on Hyperliquid perpetuals",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string" },
                        "size": { "type": "number", "description": "Position size in coin units" }
                    },
                    "required": ["coin", "size"]
                }
            },
            {
                "name": "hl_market_sell",
                "description": "Place a market sell / short order on Hyperliquid perpetuals",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string" },
                        "size": { "type": "number" }
                    },
                    "required": ["coin", "size"]
                }
            },
            {
                "name": "hl_limit_buy",
                "description": "Place a GTC limit buy order",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string" },
                        "size": { "type": "number" },
                        "price": { "type": "number" }
                    },
                    "required": ["coin", "size", "price"]
                }
            },
            {
                "name": "hl_limit_sell",
                "description": "Place a GTC limit sell order",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string" },
                        "size": { "type": "number" },
                        "price": { "type": "number" }
                    },
                    "required": ["coin", "size", "price"]
                }
            },
            {
                "name": "hl_cancel_all",
                "description": "Cancel all open orders, optionally for a specific coin",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string", "description": "Optional: only cancel orders for this coin" }
                    }
                }
            },
            {
                "name": "hl_set_leverage",
                "description": "Set leverage for a coin (1–50x, cross or isolated)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "coin": { "type": "string" },
                        "leverage": { "type": "integer" },
                        "cross": { "type": "boolean", "description": "true = cross margin, false = isolated" }
                    },
                    "required": ["coin", "leverage"]
                }
            }
        ]
    })
}

// ── Main MCP server loop ──────────────────────────────────────────────────────

pub async fn run_stdio_server() -> Result<()> {
    let testnet = std::env::var("HYPERLIQUID_TESTNET").is_ok();
    let address = std::env::var("HYPERLIQUID_ADDRESS").ok();
    let private_key = std::env::var("HYPERLIQUID_PRIVATE_KEY").ok();

    let info_client = HyperliquidClient::new(testnet, address.clone());
    let exchange_client = private_key
        .as_deref()
        .map(|k| ExchangeClient::new(k, testnet));
    let engine = MomentumEngine::new();

    let stdin = io::stdin();
    let stdout = io::stdout();

    info!("HyperLiquid Claw MCP server ready (stdio)");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: McpRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = McpResponse::err(None, -32700, format!("Parse error: {e}"));
                writeln!(stdout.lock(), "{}", serde_json::to_string(&resp)?)?;
                continue;
            }
        };

        let id = req.id.clone();
        let resp = match req.method.as_str() {
            "initialize" => McpResponse::ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "hyperliquid-claw", "version": "3.0.0" }
                }),
            ),

            "tools/list" => McpResponse::ok(id, tool_definitions()),

            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let tool_name = params["name"].as_str().unwrap_or("").to_string();
                let args = params["arguments"].clone();

                match dispatch_tool(&tool_name, &args, &info_client, &exchange_client, &engine).await {
                    Ok(result) => McpResponse::ok(
                        id,
                        json!({ "content": [{ "type": "text", "text": result }] }),
                    ),
                    Err(e) => McpResponse::ok(
                        id,
                        json!({
                            "content": [{ "type": "text", "text": format!("❌ Error: {e}") }],
                            "isError": true
                        }),
                    ),
                }
            }

            _ => McpResponse::err(id, -32601, "Method not found"),
        };

        writeln!(stdout.lock(), "{}", serde_json::to_string(&resp)?)?;
        stdout.lock().flush()?;
    }

    Ok(())
}

async fn dispatch_tool(
    name: &str,
    args: &Value,
    client: &HyperliquidClient,
    exchange: &Option<Result<ExchangeClient>>,
    engine: &MomentumEngine,
) -> Result<String> {
    match name {
        "hl_price" => {
            let coin = args["coin"].as_str().unwrap_or("BTC");
            let price = client.get_price(coin).await?;
            Ok(format!("{} mark price: ${:.4}", coin.to_uppercase(), price))
        }

        "hl_meta" => {
            let assets = client.get_meta().await?;
            let list: Vec<_> = assets.iter().map(|a| a.name.clone()).collect();
            Ok(format!(
                "{} perpetuals available:\n{}",
                list.len(),
                list.join(", ")
            ))
        }

        "hl_market_scan" => {
            let top_n = args["top_n"].as_u64().unwrap_or(10) as usize;
            let signals = engine.top_opportunities(client, top_n).await?;
            if signals.is_empty() {
                return Ok("No strong signals detected right now. Market is neutral.".into());
            }
            let mut out = format!("📊 Top {} market signals:\n\n", signals.len());
            for s in &signals {
                out.push_str(&format!(
                    "• {:<6} ${:.4}  {}  {:+.2}%  Vol ${:.0}M  Confidence {:.0}%\n  → {}\n\n",
                    s.coin,
                    s.mark_px,
                    s.signal,
                    s.price_change_pct,
                    s.volume_24h / 1_000_000.0,
                    s.confidence * 100.0,
                    s.suggested_action
                ));
            }
            Ok(out)
        }

        "hl_analyze" => {
            let coin = args["coin"].as_str().unwrap_or("BTC");
            let summaries = client.get_all_mids().await?;
            let upper = coin.to_uppercase();
            match summaries.iter().find(|s| s.coin == upper) {
                Some(summary) => {
                    let sig = engine.analyze_summary(summary)
                        .ok_or_else(|| anyhow::anyhow!("Could not analyze {coin}"))?;
                    Ok(format!(
                        "Analysis for {coin}:\n\
                         Price: ${:.4}\n\
                         24h Change: {:+.2}%\n\
                         24h Volume: ${:.0}M\n\
                         Open Interest: ${:.0}M\n\
                         Funding Rate: {:.4}%\n\
                         Signal: {}\n\
                         Confidence: {:.0}%\n\
                         Action: {}",
                        sig.mark_px,
                        sig.price_change_pct,
                        sig.volume_24h / 1_000_000.0,
                        sig.open_interest / 1_000_000.0,
                        sig.funding_rate * 100.0,
                        sig.signal,
                        sig.confidence * 100.0,
                        sig.suggested_action
                    ))
                }
                None => Err(anyhow::anyhow!("Asset {} not found", coin)),
            }
        }

        "hl_balance" => {
            let state = client.get_account_state().await?;
            let m = &state.margin_summary;
            Ok(format!(
                "💰 Account Summary:\n\
                 Equity:         ${}\n\
                 Total Position: ${}\n\
                 Margin Used:    ${}\n\
                 Free Margin:    ${:.2}",
                m.account_value,
                m.total_ntl_pos,
                m.total_margin_used,
                m.account_value.parse::<f64>().unwrap_or(0.0)
                    - m.total_margin_used.parse::<f64>().unwrap_or(0.0)
            ))
        }

        "hl_positions" => {
            let state = client.get_account_state().await?;
            if state.positions.is_empty() {
                return Ok("No open positions.".into());
            }
            let mut out = "📈 Open Positions:\n\n".to_string();
            for ap in &state.positions {
                let p = &ap.position;
                out.push_str(&format!(
                    "• {:<6}  Size: {}  Entry: ${}  Value: ${}  P&L: ${}  ROE: {}%\n",
                    p.coin,
                    p.size,
                    p.entry_px.as_deref().unwrap_or("?"),
                    p.position_value,
                    p.unrealized_pnl,
                    p.roe
                ));
            }
            Ok(out)
        }

        "hl_orders" => {
            let orders = client.get_open_orders().await?;
            if orders.is_empty() {
                return Ok("No open orders.".into());
            }
            let mut out = "📋 Open Orders:\n\n".to_string();
            for o in &orders {
                out.push_str(&format!(
                    "• {:<6}  {}  {} @ ${}  (oid: {})\n",
                    o.coin, o.side, o.sz, o.limit_px, o.oid
                ));
            }
            Ok(out)
        }

        "hl_fills" => {
            let limit = args["limit"].as_u64().unwrap_or(20) as u32;
            let fills = client.get_user_fills(limit).await?;
            Ok(serde_json::to_string_pretty(&fills)?)
        }

        "hl_market_buy" | "hl_market_sell" => {
            let ex = require_exchange(exchange)?;
            let coin = args["coin"].as_str().ok_or_else(|| anyhow::anyhow!("coin required"))?;
            let size = args["size"].as_f64().ok_or_else(|| anyhow::anyhow!("size required"))?;
            let is_buy = name == "hl_market_buy";
            let price = client.get_price(coin).await?;
            let state = client.get_account_state().await?;
            let equity: f64 = state.margin_summary.account_value.parse().unwrap_or(10000.0);
            let result = ex.market_order(coin, is_buy, size, price, equity).await?;
            Ok(format!("Order result: {}", serde_json::to_string_pretty(&result)?))
        }

        "hl_limit_buy" | "hl_limit_sell" => {
            let ex = require_exchange(exchange)?;
            let coin = args["coin"].as_str().ok_or_else(|| anyhow::anyhow!("coin required"))?;
            let size = args["size"].as_f64().ok_or_else(|| anyhow::anyhow!("size required"))?;
            let limit_price = args["price"].as_f64().ok_or_else(|| anyhow::anyhow!("price required"))?;
            let is_buy = name == "hl_limit_buy";
            let current = client.get_price(coin).await?;
            let result = ex.limit_order(coin, is_buy, size, limit_price, current).await?;
            Ok(format!("Order result: {}", serde_json::to_string_pretty(&result)?))
        }

        "hl_cancel_all" => {
            let ex = require_exchange(exchange)?;
            let coin = args["coin"].as_str();
            let result = ex.cancel_all_orders(coin).await?;
            Ok(format!("Cancel result: {}", serde_json::to_string_pretty(&result)?))
        }

        "hl_set_leverage" => {
            let ex = require_exchange(exchange)?;
            let coin = args["coin"].as_str().ok_or_else(|| anyhow::anyhow!("coin required"))?;
            let leverage = args["leverage"].as_u64().ok_or_else(|| anyhow::anyhow!("leverage required"))? as u32;
            let cross = args["cross"].as_bool().unwrap_or(true);
            let result = ex.update_leverage(coin, leverage, cross).await?;
            Ok(format!("Leverage updated: {}", serde_json::to_string_pretty(&result)?))
        }

        other => Err(anyhow::anyhow!("Unknown tool: {other}")),
    }
}

fn require_exchange(ex: &Option<Result<ExchangeClient>>) -> Result<&ExchangeClient> {
    match ex {
        Some(Ok(e)) => Ok(e),
        Some(Err(e)) => Err(anyhow::anyhow!("Exchange client error: {e}")),
        None => Err(anyhow::anyhow!(
            "HYPERLIQUID_PRIVATE_KEY not set. Trading commands require a private key."
        )),
    }
}
