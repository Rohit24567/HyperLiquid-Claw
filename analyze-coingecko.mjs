#!/usr/bin/env node
/**
 * analyze-coingecko.mjs — Chart + volume + momentum analysis via CoinGecko
 * No API key required. Covers BTC, ETH, SOL, AVAX, ARB, DOGE and more.
 */

import fetch from "node-fetch";

const COINS = {
  BTC:  "bitcoin",
  ETH:  "ethereum",
  SOL:  "solana",
  AVAX: "avalanche-2",
  ARB:  "arbitrum",
  DOGE: "dogecoin",
  MATIC:"matic-network",
  LINK: "chainlink",
};

const COIN_ID = process.argv[2]
  ? (COINS[process.argv[2].toUpperCase()] || process.argv[2].toLowerCase())
  : "bitcoin";

const COIN_LABEL = Object.entries(COINS).find(([,v]) => v === COIN_ID)?.[0] || COIN_ID.toUpperCase();

async function fetchOHLCV(coinId) {
  const url = `https://api.coingecko.com/api/v3/coins/${coinId}/ohlc?vs_currency=usd&days=1`;
  const res  = await fetch(url);
  if (!res.ok) throw new Error(`CoinGecko OHLC error: ${res.status}`);
  return res.json(); // [[ts, o, h, l, c], ...]
}

async function fetchMarketData(coinId) {
  const url = `https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids=${coinId}&price_change_percentage=1h,6h,24h`;
  const res  = await fetch(url);
  if (!res.ok) throw new Error(`CoinGecko market error: ${res.status}`);
  const data = await res.json();
  return data[0];
}

function computeSignal(priceChange1h, priceChange6h, volumeRatio) {
  const score =
    (priceChange1h > 0.5 ? 2 : priceChange1h > 0.2 ? 1 : priceChange1h < -0.5 ? -2 : priceChange1h < -0.2 ? -1 : 0) +
    (priceChange6h > 1.0 ? 2 : priceChange6h > 0.3 ? 1 : priceChange6h < -1.0 ? -2 : priceChange6h < -0.3 ? -1 : 0) +
    (volumeRatio > 1.5 ? 1 : volumeRatio < 0.7 ? -1 : 0);

  if (score >= 4)  return { signal: "STRONG BULLISH 🚀", action: "HIGH-PROBABILITY LONG", color: "green" };
  if (score >= 2)  return { signal: "BULLISH 📈",        action: "Consider long with confirmation", color: "green" };
  if (score <= -4) return { signal: "STRONG BEARISH 🔻", action: "HIGH-PROBABILITY SHORT", color: "red" };
  if (score <= -2) return { signal: "BEARISH 📉",        action: "Consider short with confirmation", color: "red" };
  return              { signal: "NEUTRAL ⚖️",           action: "Wait for clearer setup", color: "gray" };
}

function renderBar(values, width = 40) {
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  return values.slice(-10).map(v => {
    const h = Math.round(((v - min) / range) * 7);
    return ["▁","▂","▃","▄","▅","▆","▇","█"][h];
  }).join("");
}

async function main() {
  console.log(`\n🦀 Hyperliquid Claw — Market Analysis`);
  console.log(`   Coin: ${COIN_LABEL} (${COIN_ID})\n`);

  const [ohlcv, market] = await Promise.all([fetchOHLCV(COIN_ID), fetchMarketData(COIN_ID)]);

  const closes     = ohlcv.map(c => c[4]);
  const currentPrice = market.current_price;
  const volume24h   = market.total_volume;
  const avgVolume   = volume24h; // CoinGecko gives 24h, we approximate avg as same
  const volumeRatio = 1.0;      // Would need historical avg for real ratio

  const change1h  = market.price_change_percentage_1h_in_currency || 0;
  const change6h  = market.price_change_percentage_6h_in_currency || 0;
  const change24h = market.price_change_percentage_24h || 0;

  const { signal, action } = computeSignal(change1h, change6h, volumeRatio);

  console.log(`💰 Price:       $${currentPrice.toLocaleString()}`);
  console.log(`📊 Chart (10h): ${renderBar(closes)}`);
  console.log(``);
  console.log(`📈 1h change:   ${change1h >= 0 ? "+" : ""}${change1h.toFixed(2)}%`);
  console.log(`📈 6h change:   ${change6h >= 0 ? "+" : ""}${change6h.toFixed(2)}%`);
  console.log(`📈 24h change:  ${change24h >= 0 ? "+" : ""}${change24h.toFixed(2)}%`);
  console.log(``);
  console.log(`📦 Volume 24h:  $${(volume24h / 1e6).toFixed(1)}M`);
  console.log(`📦 Vol ratio:   ${volumeRatio.toFixed(2)}x average`);
  console.log(``);
  console.log(`🎯 Signal:      ${signal}`);
  console.log(`💡 Action:      ${action}`);
  console.log(``);

  // Machine-readable JSON for OpenClaw parsing
  const result = {
    coin: COIN_LABEL,
    price: currentPrice,
    change_1h: change1h,
    change_6h: change6h,
    change_24h: change24h,
    volume_24h: volume24h,
    volume_ratio: volumeRatio,
    signal,
    action,
    timestamp: new Date().toISOString(),
  };

  console.log("--- JSON ---");
  console.log(JSON.stringify(result, null, 2));
}

main().catch(e => {
  console.error(`Error: ${e.message}`);
  process.exit(1);
});
