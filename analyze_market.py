#!/usr/bin/env python3
"""
analyze_market.py — Python momentum engine for Hyperliquid Claw
Uses CoinGecko API (no key required). Outputs signal + JSON for OpenClaw.
"""

import sys
import json
import requests
from datetime import datetime

COINS = {
    "BTC":   "bitcoin",
    "ETH":   "ethereum",
    "SOL":   "solana",
    "AVAX":  "avalanche-2",
    "ARB":   "arbitrum",
    "DOGE":  "dogecoin",
    "MATIC": "matic-network",
    "LINK":  "chainlink",
}

coin_arg  = sys.argv[1].upper() if len(sys.argv) > 1 else "BTC"
coin_id   = COINS.get(coin_arg, coin_arg.lower())
coin_label = coin_arg


def fetch_market(coin_id: str) -> dict:
    url = (
        "https://api.coingecko.com/api/v3/coins/markets"
        f"?vs_currency=usd&ids={coin_id}&price_change_percentage=1h,6h,24h"
    )
    r = requests.get(url, timeout=10)
    r.raise_for_status()
    data = r.json()
    if not data:
        raise ValueError(f"Unknown coin: {coin_id}")
    return data[0]


def fetch_ohlcv(coin_id: str) -> list[list]:
    url = f"https://api.coingecko.com/api/v3/coins/{coin_id}/ohlc?vs_currency=usd&days=1"
    r = requests.get(url, timeout=10)
    r.raise_for_status()
    return r.json()  # [[ts, o, h, l, c], ...]


def compute_signal(change_1h: float, change_6h: float, volume_ratio: float) -> tuple[str, str, int]:
    score = 0
    score += 2 if change_1h > 0.5 else (1 if change_1h > 0.2 else (-2 if change_1h < -0.5 else (-1 if change_1h < -0.2 else 0)))
    score += 2 if change_6h > 1.0 else (1 if change_6h > 0.3 else (-2 if change_6h < -1.0 else (-1 if change_6h < -0.3 else 0)))
    score += 1 if volume_ratio > 1.5 else (-1 if volume_ratio < 0.7 else 0)

    if score >= 4:  return "STRONG BULLISH 🚀", "HIGH-PROBABILITY LONG — enter with full size", score
    if score >= 2:  return "BULLISH 📈",         "Consider long — wait for volume confirmation", score
    if score <= -4: return "STRONG BEARISH 🔻", "HIGH-PROBABILITY SHORT — enter with full size", score
    if score <= -2: return "BEARISH 📉",         "Consider short — wait for volume confirmation", score
    return               "NEUTRAL ⚖️",          "Wait for a clearer setup — no trade", score


def sparkline(values: list[float], width: int = 10) -> str:
    blocks = "▁▂▃▄▅▆▇█"
    subset = values[-width:]
    lo, hi = min(subset), max(subset)
    span = hi - lo or 1
    return "".join(blocks[round(((v - lo) / span) * 7)] for v in subset)


def main():
    print(f"\n🦀 Hyperliquid Claw — Python Momentum Engine")
    print(f"   Coin: {coin_label}\n")

    market = fetch_market(coin_id)
    ohlcv  = fetch_ohlcv(coin_id)

    price      = market["current_price"]
    volume_24h = market["total_volume"]
    change_1h  = market.get("price_change_percentage_1h_in_currency") or 0.0
    change_6h  = market.get("price_change_percentage_6h_in_currency") or 0.0
    change_24h = market.get("price_change_percentage_24h") or 0.0

    closes        = [c[4] for c in ohlcv]
    volume_ratio  = 1.0   # placeholder; real ratio needs historical avg

    signal, action, score = compute_signal(change_1h, change_6h, volume_ratio)

    print(f"💰 Price:       ${price:,.2f}")
    print(f"📊 Chart (10h): {sparkline(closes)}")
    print()
    print(f"📈 1h change:   {change_1h:+.2f}%")
    print(f"📈 6h change:   {change_6h:+.2f}%")
    print(f"📈 24h change:  {change_24h:+.2f}%")
    print()
    print(f"📦 Volume 24h:  ${volume_24h / 1e6:.1f}M")
    print(f"📦 Vol ratio:   {volume_ratio:.2f}x average")
    print()
    print(f"🎯 Signal:      {signal}  (score: {score:+d})")
    print(f"💡 Action:      {action}")
    print()

    result = {
        "coin":        coin_label,
        "price":       price,
        "change_1h":   change_1h,
        "change_6h":   change_6h,
        "change_24h":  change_24h,
        "volume_24h":  volume_24h,
        "volume_ratio": volume_ratio,
        "signal":      signal,
        "action":      action,
        "score":       score,
        "timestamp":   datetime.utcnow().isoformat() + "Z",
    }

    print("--- JSON ---")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)
