// src/analysis/signals.rs
// Momentum engine — replicates Python signals.py in pure Rust

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::trading::client::{HyperliquidClient, MarketSummary};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Signal {
    StrongBullish,
    Bullish,
    Neutral,
    Bearish,
    StrongBearish,
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::StrongBullish => write!(f, "🟢 STRONG BULLISH"),
            Signal::Bullish => write!(f, "🟡 BULLISH"),
            Signal::Neutral => write!(f, "⚪ NEUTRAL"),
            Signal::Bearish => write!(f, "🟠 BEARISH"),
            Signal::StrongBearish => write!(f, "🔴 STRONG BEARISH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSignal {
    pub coin: String,
    pub mark_px: f64,
    pub price_change_pct: f64,
    pub volume_24h: f64,
    pub open_interest: f64,
    pub funding_rate: f64,
    pub signal: Signal,
    pub confidence: f64,
    pub suggested_action: String,
    pub timestamp: u64,
}

pub struct MomentumEngine {
    // Thresholds
    strong_move_pct: f64,
    weak_move_pct: f64,
    volume_multiplier: f64,
}

impl Default for MomentumEngine {
    fn default() -> Self {
        Self {
            strong_move_pct: 0.5,
            weak_move_pct: 0.2,
            volume_multiplier: 1.5,
        }
    }
}

impl MomentumEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Compute momentum signal for a single market summary
    pub fn analyze_summary(&self, s: &MarketSummary) -> Option<MarketSignal> {
        let mark_px: f64 = s.mark_px.parse().ok()?;
        let prev_px: f64 = s.prev_day_px.parse().ok()?;
        let volume: f64 = s.day_volume.parse().ok()?;
        let oi: f64 = s.open_interest.parse().ok()?;
        let funding: f64 = s.funding.parse().ok()?;

        if prev_px == 0.0 || mark_px == 0.0 {
            return None;
        }

        let change_pct = ((mark_px - prev_px) / prev_px) * 100.0;

        // Volume score (0..1) — normalise by OI as a rough proxy
        let vol_score = if oi > 0.0 {
            (volume / oi).min(3.0) / 3.0
        } else {
            0.5
        };

        let (signal, confidence) = self.classify(change_pct, vol_score, funding);

        let suggested_action = match &signal {
            Signal::StrongBullish => format!(
                "Consider LONG {coin} — strong upside momentum. Target +2%, Stop -1%",
                coin = s.coin
            ),
            Signal::Bullish => format!("Watch {coin} for long entry confirmation", coin = s.coin),
            Signal::StrongBearish => format!(
                "Consider SHORT {coin} — strong downside momentum. Target +2%, Stop -1%",
                coin = s.coin
            ),
            Signal::Bearish => format!("Watch {coin} for short entry confirmation", coin = s.coin),
            Signal::Neutral => format!("{} — no clear edge, stay flat", s.coin),
        };

        Some(MarketSignal {
            coin: s.coin.clone(),
            mark_px,
            price_change_pct: change_pct,
            volume_24h: volume,
            open_interest: oi,
            funding_rate: funding,
            signal,
            confidence,
            suggested_action,
            timestamp: Self::now_ms(),
        })
    }

    fn classify(&self, change_pct: f64, vol_score: f64, funding: f64) -> (Signal, f64) {
        let abs_change = change_pct.abs();
        let is_up = change_pct > 0.0;

        // Funding contrarian signal (extreme positive = potential short pressure)
        let funding_bias: f64 = -funding.signum() * 0.05;

        let raw_confidence = (abs_change / self.strong_move_pct).min(1.0) * 0.6
            + vol_score * 0.3
            + funding_bias * 0.1;

        let confidence = raw_confidence.clamp(0.0, 1.0);

        let signal = if abs_change >= self.strong_move_pct && vol_score >= (1.0 / self.volume_multiplier) {
            if is_up {
                Signal::StrongBullish
            } else {
                Signal::StrongBearish
            }
        } else if abs_change >= self.weak_move_pct {
            if is_up {
                Signal::Bullish
            } else {
                Signal::Bearish
            }
        } else {
            Signal::Neutral
        };

        (signal, confidence)
    }

    /// Scan all markets and return top signals
    pub async fn scan_all(&self, client: &HyperliquidClient) -> Result<Vec<MarketSignal>> {
        let summaries = client.get_all_mids().await?;
        let mut signals: Vec<MarketSignal> = summaries
            .iter()
            .filter_map(|s| self.analyze_summary(s))
            .collect();

        // Sort by confidence descending
        signals.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        Ok(signals)
    }

    /// Scan and filter only actionable signals
    pub async fn top_opportunities(
        &self,
        client: &HyperliquidClient,
        n: usize,
    ) -> Result<Vec<MarketSignal>> {
        let all = self.scan_all(client).await?;
        let actionable: Vec<_> = all
            .into_iter()
            .filter(|s| s.signal != Signal::Neutral)
            .take(n)
            .collect();
        Ok(actionable)
    }
}

/// Pretty-print a signal to the terminal
pub fn print_signal(s: &MarketSignal) {
    let change_arrow = if s.price_change_pct >= 0.0 { "▲" } else { "▼" };
    println!(
        "\n{:<6}  ${:.4}  {}  {:+.2}%  Vol ${:.0}M  OI ${:.0}M  Funding {:.4}%\n  → {}  (confidence: {:.0}%)",
        s.coin,
        s.mark_px,
        s.signal,
        s.price_change_pct,
        s.volume_24h / 1_000_000.0,
        s.open_interest / 1_000_000.0,
        s.funding_rate * 100.0,
        s.suggested_action,
        s.confidence * 100.0,
    );
}
