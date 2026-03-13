// src/trading/client.rs
// Hyperliquid REST API client — Info + Exchange endpoints

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

pub const MAINNET_API: &str = "https://api.hyperliquid.xyz";
pub const TESTNET_API: &str = "https://api.hyperliquid-testnet.xyz";

#[derive(Clone)]
pub struct HyperliquidClient {
    http: Client,
    base_url: String,
    pub address: Option<String>,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetMeta {
    pub name: String,
    #[serde(rename = "szDecimals")]
    pub sz_decimals: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MarketSummary {
    pub coin: String,
    #[serde(rename = "markPx")]
    pub mark_px: String,
    #[serde(rename = "midPx")]
    pub mid_px: Option<String>,
    #[serde(rename = "openInterest")]
    pub open_interest: String,
    #[serde(rename = "dayNtlVlm")]
    pub day_volume: String,
    pub funding: String,
    #[serde(rename = "prevDayPx")]
    pub prev_day_px: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    pub coin: String,
    #[serde(rename = "szi")]
    pub size: String,
    #[serde(rename = "entryPx")]
    pub entry_px: Option<String>,
    #[serde(rename = "positionValue")]
    pub position_value: String,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
    #[serde(rename = "returnOnEquity")]
    pub roe: String,
    pub leverage: LeverageInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeverageInfo {
    #[serde(rename = "type")]
    pub leverage_type: String,
    pub value: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountState {
    #[serde(rename = "marginSummary")]
    pub margin_summary: MarginSummary,
    #[serde(rename = "assetPositions")]
    pub positions: Vec<AssetPosition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarginSummary {
    #[serde(rename = "accountValue")]
    pub account_value: String,
    #[serde(rename = "totalNtlPos")]
    pub total_ntl_pos: String,
    #[serde(rename = "totalMarginUsed")]
    pub total_margin_used: String,
    #[serde(rename = "totalRawUsd")]
    pub total_raw_usd: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssetPosition {
    pub position: Position,
    #[serde(rename = "type")]
    pub pos_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenOrder {
    pub coin: String,
    pub side: String,
    #[serde(rename = "limitPx")]
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub timestamp: u64,
}

// ── Client implementation ─────────────────────────────────────────────────────

impl HyperliquidClient {
    pub fn new(testnet: bool, address: Option<String>) -> Self {
        let base_url = if testnet {
            TESTNET_API.to_string()
        } else {
            MAINNET_API.to_string()
        };
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("Failed to build HTTP client"),
            base_url,
            address,
        }
    }

    async fn info_post(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/info", self.base_url);
        debug!("POST /info {:?}", payload);
        let resp = self.http.post(&url).json(&payload).send().await?;
        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            return Err(anyhow!("API error {}: {}", status, body));
        }
        Ok(body)
    }

    // ── Info endpoint helpers ─────────────────────────────────────────────────

    pub async fn get_meta(&self) -> Result<Vec<AssetMeta>> {
        let body = self.info_post(json!({"type": "meta"})).await?;
        let assets = body["universe"]
            .as_array()
            .ok_or_else(|| anyhow!("unexpected meta shape"))?
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        Ok(assets)
    }

    pub async fn get_all_mids(&self) -> Result<Vec<MarketSummary>> {
        let body = self.info_post(json!({"type": "metaAndAssetCtxs"})).await?;
        let arr = body.as_array().ok_or_else(|| anyhow!("expected array"))?;
        if arr.len() < 2 {
            return Err(anyhow!("unexpected metaAndAssetCtxs shape"));
        }
        let universe = arr[0]["universe"]
            .as_array()
            .ok_or_else(|| anyhow!("no universe"))?;
        let ctxs = arr[1]
            .as_array()
            .ok_or_else(|| anyhow!("no asset contexts"))?;

        let summaries = universe
            .iter()
            .zip(ctxs.iter())
            .filter_map(|(meta, ctx)| {
                let coin = meta["name"].as_str()?.to_string();
                Some(MarketSummary {
                    coin,
                    mark_px: ctx["markPx"].as_str().unwrap_or("0").to_string(),
                    mid_px: ctx["midPx"].as_str().map(String::from),
                    open_interest: ctx["openInterest"].as_str().unwrap_or("0").to_string(),
                    day_volume: ctx["dayNtlVlm"].as_str().unwrap_or("0").to_string(),
                    funding: ctx["funding"].as_str().unwrap_or("0").to_string(),
                    prev_day_px: ctx["prevDayPx"].as_str().unwrap_or("0").to_string(),
                })
            })
            .collect();
        Ok(summaries)
    }

    pub async fn get_price(&self, coin: &str) -> Result<f64> {
        let summaries = self.get_all_mids().await?;
        let upper = coin.to_uppercase();
        summaries
            .iter()
            .find(|s| s.coin == upper)
            .and_then(|s| s.mark_px.parse().ok())
            .ok_or_else(|| anyhow!("Asset {} not found", coin))
    }

    pub async fn get_account_state(&self) -> Result<AccountState> {
        let addr = self
            .address
            .as_ref()
            .ok_or_else(|| anyhow!("HYPERLIQUID_ADDRESS not set"))?;
        let body = self
            .info_post(json!({"type": "clearinghouseState", "user": addr}))
            .await?;
        Ok(serde_json::from_value(body)?)
    }

    pub async fn get_open_orders(&self) -> Result<Vec<OpenOrder>> {
        let addr = self
            .address
            .as_ref()
            .ok_or_else(|| anyhow!("HYPERLIQUID_ADDRESS not set"))?;
        let body = self
            .info_post(json!({"type": "openOrders", "user": addr}))
            .await?;
        Ok(serde_json::from_value(body)?)
    }

    pub async fn get_user_fills(&self, limit: u32) -> Result<Value> {
        let addr = self
            .address
            .as_ref()
            .ok_or_else(|| anyhow!("HYPERLIQUID_ADDRESS not set"))?;
        let body = self
            .info_post(json!({"type": "userFills", "user": addr}))
            .await?;
        // Return last N fills
        if let Some(arr) = body.as_array() {
            let start = arr.len().saturating_sub(limit as usize);
            return Ok(json!(arr[start..]));
        }
        Ok(body)
    }

    pub async fn get_l2_book(&self, coin: &str) -> Result<Value> {
        let body = self
            .info_post(json!({"type": "l2Book", "coin": coin.to_uppercase()}))
            .await?;
        Ok(body)
    }

    pub async fn get_candles(
        &self,
        coin: &str,
        interval: &str,
        start_ms: u64,
        end_ms: u64,
    ) -> Result<Value> {
        let body = self
            .info_post(json!({
                "type": "candleSnapshot",
                "req": {
                    "coin": coin.to_uppercase(),
                    "interval": interval,
                    "startTime": start_ms,
                    "endTime": end_ms
                }
            }))
            .await?;
        Ok(body)
    }
}
