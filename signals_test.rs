// tests/signals_test.rs

use hyperliquid_claw::analysis::signals::{MomentumEngine, Signal};
use hyperliquid_claw::trading::client::MarketSummary;

fn make_summary(coin: &str, mark: f64, prev: f64, vol: f64, oi: f64) -> MarketSummary {
    MarketSummary {
        coin: coin.to_string(),
        mark_px: mark.to_string(),
        mid_px: Some(mark.to_string()),
        open_interest: oi.to_string(),
        day_volume: vol.to_string(),
        funding: "0.0001".to_string(),
        prev_day_px: prev.to_string(),
    }
}

#[test]
fn test_strong_bullish_signal() {
    let engine = MomentumEngine::new();
    // 1% up move, high volume
    let s = make_summary("BTC", 101_000.0, 100_000.0, 2_000_000_000.0, 1_000_000_000.0);
    let sig = engine.analyze_summary(&s).unwrap();
    assert_eq!(sig.signal, Signal::StrongBullish);
    assert!(sig.price_change_pct > 0.0);
    assert!(sig.confidence > 0.5);
}

#[test]
fn test_strong_bearish_signal() {
    let engine = MomentumEngine::new();
    // 1% down move
    let s = make_summary("ETH", 3_000.0, 3_030.3, 2_000_000_000.0, 1_000_000_000.0);
    let sig = engine.analyze_summary(&s).unwrap();
    assert_eq!(sig.signal, Signal::StrongBearish);
    assert!(sig.price_change_pct < 0.0);
}

#[test]
fn test_neutral_signal() {
    let engine = MomentumEngine::new();
    // 0.01% move — negligible
    let s = make_summary("SOL", 100.01, 100.0, 10_000.0, 1_000_000.0);
    let sig = engine.analyze_summary(&s).unwrap();
    assert_eq!(sig.signal, Signal::Neutral);
}

#[test]
fn test_zero_price_returns_none() {
    let engine = MomentumEngine::new();
    let s = make_summary("ZERO", 0.0, 0.0, 0.0, 0.0);
    assert!(engine.analyze_summary(&s).is_none());
}
