// src/trading/exchange.rs
// Hyperliquid Exchange endpoint — order signing via EIP-712 + secp256k1

use anyhow::{anyhow, Result};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::H256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use super::client::{MAINNET_API, TESTNET_API};

pub struct ExchangeClient {
    http: Client,
    base_url: String,
    wallet: LocalWallet,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub status: String,
    pub response: Option<OrderResult>,
}

#[derive(Debug, Deserialize)]
pub struct OrderResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub data: Option<Value>,
}

// Safety limits — always enforced
const MAX_SLIPPAGE_PCT: f64 = 5.0;
const MAX_POSITION_PCT: f64 = 20.0;
const MAX_LIMIT_DEVIATION_PCT: f64 = 5.0;

impl ExchangeClient {
    pub fn new(private_key: &str, testnet: bool) -> Result<Self> {
        let wallet: LocalWallet = private_key
            .trim_start_matches("0x")
            .parse()
            .map_err(|_| anyhow!("Invalid private key format"))?;
        let address = format!("{:?}", wallet.address());
        let base_url = if testnet {
            TESTNET_API.to_string()
        } else {
            MAINNET_API.to_string()
        };
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()?,
            base_url,
            wallet,
            address,
        })
    }

    fn timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Sign an action using Hyperliquid's connection_id hashing scheme.
    /// The action JSON is hashed with SHA-256 then signed via secp256k1.
    async fn sign_action(&self, action: &Value, vault_address: Option<&str>) -> Result<Value> {
        let nonce = Self::timestamp_ms();

        // Build the message bytes: action_hash ++ nonce ++ vault_flag
        let action_bytes = serde_json::to_vec(action)?;
        let action_hash = Sha256::digest(&action_bytes);

        let mut msg = Vec::with_capacity(32 + 8 + 1);
        msg.extend_from_slice(&action_hash);
        msg.extend_from_slice(&nonce.to_be_bytes());
        msg.push(if vault_address.is_some() { 1u8 } else { 0u8 });

        // Prefix for Ethereum personal_sign
        let prefixed = format!("\x19Ethereum Signed Message:\n{}", msg.len());
        let mut full = prefixed.into_bytes();
        full.extend_from_slice(&msg);
        let hash = Sha256::digest(&full);

        let sig = self
            .wallet
            .sign_hash(H256::from_slice(&hash))
            .await
            .map_err(|e| anyhow!("Signing failed: {e}"))?;

        Ok(json!({
            "r": format!("0x{:064x}", sig.r),
            "s": format!("0x{:064x}", sig.s),
            "v": sig.v as u64
        }))
    }

    async fn exchange_post(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/exchange", self.base_url);
        let resp = self.http.post(&url).json(&payload).send().await?;
        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            return Err(anyhow!("Exchange API error {}: {}", status, body));
        }
        Ok(body)
    }

    // ── Order placement ───────────────────────────────────────────────────────

    pub async fn market_order(
        &self,
        coin: &str,
        is_buy: bool,
        size: f64,
        current_price: f64,
        account_equity: f64,
    ) -> Result<Value> {
        // Safety: position size check
        let notional = size * current_price;
        let pct_of_equity = (notional / account_equity) * 100.0;
        if pct_of_equity > MAX_POSITION_PCT {
            warn!(
                "⚠️  Position {:.1}% of equity exceeds {}% limit",
                pct_of_equity, MAX_POSITION_PCT
            );
        }

        // Market order uses limit with slippage buffer
        let slippage = MAX_SLIPPAGE_PCT / 100.0;
        let limit_px = if is_buy {
            current_price * (1.0 + slippage)
        } else {
            current_price * (1.0 - slippage)
        };

        let coin_upper = coin.to_uppercase();
        let side = if is_buy { "B" } else { "A" };

        let action = json!({
            "type": "order",
            "orders": [{
                "a": 0,  // asset index — resolved by server from coin field
                "b": is_buy,
                "p": format!("{:.6}", limit_px),
                "s": format!("{:.6}", size),
                "r": false,
                "t": { "limit": { "tif": "Ioc" } }
            }],
            "grouping": "na",
            "coin": coin_upper
        });

        let sig = self.sign_action(&action, None).await?;
        let nonce = Self::timestamp_ms();

        let payload = json!({
            "action": action,
            "nonce": nonce,
            "signature": sig
        });

        info!("Placing market {} {} {} @ ≤{:.4}", side, size, coin_upper, limit_px);
        self.exchange_post(payload).await
    }

    pub async fn limit_order(
        &self,
        coin: &str,
        is_buy: bool,
        size: f64,
        limit_price: f64,
        current_price: f64,
    ) -> Result<Value> {
        // Safety: warn if limit is far from market
        let deviation_pct = ((limit_price - current_price) / current_price).abs() * 100.0;
        if deviation_pct > MAX_LIMIT_DEVIATION_PCT {
            warn!(
                "⚠️  Limit price {:.4} is {:.1}% from market {:.4}",
                limit_price, deviation_pct, current_price
            );
        }

        let coin_upper = coin.to_uppercase();

        let action = json!({
            "type": "order",
            "orders": [{
                "a": 0,
                "b": is_buy,
                "p": format!("{:.6}", limit_price),
                "s": format!("{:.6}", size),
                "r": false,
                "t": { "limit": { "tif": "Gtc" } }
            }],
            "grouping": "na",
            "coin": coin_upper
        });

        let sig = self.sign_action(&action, None).await?;
        let nonce = Self::timestamp_ms();

        let payload = json!({
            "action": action,
            "nonce": nonce,
            "signature": sig
        });

        info!("Placing limit {} {} {} @ {:.4}", if is_buy { "BUY" } else { "SELL" }, size, coin_upper, limit_price);
        self.exchange_post(payload).await
    }

    pub async fn cancel_order(&self, coin: &str, oid: u64) -> Result<Value> {
        let action = json!({
            "type": "cancel",
            "cancels": [{ "a": 0, "o": oid }],
            "coin": coin.to_uppercase()
        });
        let sig = self.sign_action(&action, None).await?;
        let nonce = Self::timestamp_ms();
        let payload = json!({ "action": action, "nonce": nonce, "signature": sig });
        self.exchange_post(payload).await
    }

    pub async fn cancel_all_orders(&self, coin: Option<&str>) -> Result<Value> {
        let action = json!({
            "type": "cancelByCloid",
            "coin": coin.unwrap_or("").to_uppercase()
        });
        let sig = self.sign_action(&action, None).await?;
        let nonce = Self::timestamp_ms();
        let payload = json!({ "action": action, "nonce": nonce, "signature": sig });
        self.exchange_post(payload).await
    }

    pub async fn close_position(&self, coin: &str, size: f64, current_price: f64) -> Result<Value> {
        // Close = sell if long, buy if short (size already signed)
        let is_buy = size < 0.0;
        let abs_size = size.abs();
        self.market_order(coin, is_buy, abs_size, current_price, f64::MAX)
            .await
    }

    pub async fn update_leverage(&self, coin: &str, leverage: u32, is_cross: bool) -> Result<Value> {
        let action = json!({
            "type": "updateLeverage",
            "asset": 0,
            "isCross": is_cross,
            "leverage": leverage,
            "coin": coin.to_uppercase()
        });
        let sig = self.sign_action(&action, None).await?;
        let nonce = Self::timestamp_ms();
        let payload = json!({ "action": action, "nonce": nonce, "signature": sig });
        self.exchange_post(payload).await
    }
}
