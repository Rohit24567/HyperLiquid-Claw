// src/bin/main.rs — hl-claw CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod trading {
    pub mod client;
    pub mod exchange;
}
mod analysis {
    pub mod signals;
}

use analysis::signals::{MomentumEngine, print_signal};
use trading::client::HyperliquidClient;
use trading::exchange::ExchangeClient;

#[derive(Parser)]
#[command(
    name = "hl-claw",
    version = "3.0.0",
    about = "🦀 Hyperliquid Claw — Rust-powered trading CLI for Hyperliquid perpetuals",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the current mark price for a coin
    Price { coin: String },
    /// List all tradeable perpetuals
    Meta,
    /// Show account balance and margin summary
    Balance,
    /// Show open positions with P&L
    Positions,
    /// Show open orders
    Orders,
    /// Show recent fills
    Fills {
        #[arg(default_value = "20")]
        limit: u32,
    },
    /// Scan all markets for momentum signals
    Scan {
        #[arg(short, long, default_value = "10")]
        top: usize,
    },
    /// Analyze a specific asset
    Analyze { coin: String },
    /// Place a market buy
    MarketBuy { coin: String, size: f64 },
    /// Place a market sell / short
    MarketSell { coin: String, size: f64 },
    /// Place a GTC limit buy
    LimitBuy { coin: String, size: f64, price: f64 },
    /// Place a GTC limit sell
    LimitSell { coin: String, size: f64, price: f64 },
    /// Cancel all orders (optionally for one coin)
    CancelAll { coin: Option<String> },
    /// Set leverage (1–50)
    SetLeverage {
        coin: String,
        leverage: u32,
        #[arg(long, default_value = "true")]
        cross: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    let testnet = std::env::var("HYPERLIQUID_TESTNET").is_ok();
    let address = std::env::var("HYPERLIQUID_ADDRESS").ok();
    let private_key = std::env::var("HYPERLIQUID_PRIVATE_KEY").ok();

    let client = HyperliquidClient::new(testnet, address.clone());
    let engine = MomentumEngine::new();

    match cli.command {
        Commands::Price { coin } => {
            let price = client.get_price(&coin).await?;
            println!("{} mark price: ${:.4}", coin.to_uppercase(), price);
        }

        Commands::Meta => {
            let assets = client.get_meta().await?;
            println!("{} perpetuals on Hyperliquid:", assets.len());
            for (i, a) in assets.iter().enumerate() {
                print!("{:<8}", a.name);
                if (i + 1) % 8 == 0 {
                    println!();
                }
            }
            println!();
        }

        Commands::Balance => {
            let state = client.get_account_state().await?;
            let m = &state.margin_summary;
            println!(
                "💰 Account Balance\n\
                 ─────────────────────────────\n\
                 Equity:         ${}\n\
                 Total Position: ${}\n\
                 Margin Used:    ${}\n\
                 Free Margin:    ${:.2}",
                m.account_value,
                m.total_ntl_pos,
                m.total_margin_used,
                m.account_value.parse::<f64>().unwrap_or(0.0)
                    - m.total_margin_used.parse::<f64>().unwrap_or(0.0)
            );
        }

        Commands::Positions => {
            let state = client.get_account_state().await?;
            if state.positions.is_empty() {
                println!("No open positions.");
                return Ok(());
            }
            println!("📈 Open Positions:");
            for ap in &state.positions {
                let p = &ap.position;
                println!(
                    "  {:<6}  Size: {:>10}  Entry: ${:<12}  Value: ${:<12}  P&L: ${:<10}  ROE: {}%",
                    p.coin,
                    p.size,
                    p.entry_px.as_deref().unwrap_or("?"),
                    p.position_value,
                    p.unrealized_pnl,
                    p.roe
                );
            }
        }

        Commands::Orders => {
            let orders = client.get_open_orders().await?;
            if orders.is_empty() {
                println!("No open orders.");
                return Ok(());
            }
            println!("📋 Open Orders:");
            for o in &orders {
                println!(
                    "  {:<6}  {}  {} @ ${:<12}  oid: {}",
                    o.coin, o.side, o.sz, o.limit_px, o.oid
                );
            }
        }

        Commands::Fills { limit } => {
            let fills = client.get_user_fills(limit).await?;
            println!("{}", serde_json::to_string_pretty(&fills)?);
        }

        Commands::Scan { top } => {
            println!("🔍 Scanning {} markets for signals...\n", top);
            let signals = engine.top_opportunities(&client, top).await?;
            if signals.is_empty() {
                println!("No strong signals detected. Market is quiet.");
            } else {
                for s in &signals {
                    print_signal(s);
                }
            }
        }

        Commands::Analyze { coin } => {
            let summaries = client.get_all_mids().await?;
            let upper = coin.to_uppercase();
            match summaries.iter().find(|s| s.coin == upper) {
                Some(summary) => {
                    if let Some(sig) = engine.analyze_summary(summary) {
                        print_signal(&sig);
                    }
                }
                None => println!("Asset {} not found.", coin),
            }
        }

        Commands::MarketBuy { coin, size } => {
            let ex = require_key(&private_key, testnet)?;
            let price = client.get_price(&coin).await?;
            let state = client.get_account_state().await?;
            let equity: f64 = state.margin_summary.account_value.parse().unwrap_or(10000.0);
            let result = ex.market_order(&coin, true, size, price, equity).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::MarketSell { coin, size } => {
            let ex = require_key(&private_key, testnet)?;
            let price = client.get_price(&coin).await?;
            let state = client.get_account_state().await?;
            let equity: f64 = state.margin_summary.account_value.parse().unwrap_or(10000.0);
            let result = ex.market_order(&coin, false, size, price, equity).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::LimitBuy { coin, size, price } => {
            let ex = require_key(&private_key, testnet)?;
            let current = client.get_price(&coin).await?;
            let result = ex.limit_order(&coin, true, size, price, current).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::LimitSell { coin, size, price } => {
            let ex = require_key(&private_key, testnet)?;
            let current = client.get_price(&coin).await?;
            let result = ex.limit_order(&coin, false, size, price, current).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::CancelAll { coin } => {
            let ex = require_key(&private_key, testnet)?;
            let result = ex.cancel_all_orders(coin.as_deref()).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::SetLeverage { coin, leverage, cross } => {
            let ex = require_key(&private_key, testnet)?;
            let result = ex.update_leverage(&coin, leverage, cross).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

fn require_key(key: &Option<String>, testnet: bool) -> Result<ExchangeClient> {
    let k = key
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("HYPERLIQUID_PRIVATE_KEY not set"))?;
    ExchangeClient::new(k, testnet)
}
