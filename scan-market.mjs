#!/usr/bin/env node
/**
 * scan-market.mjs — Quick price scanner for major Hyperliquid perpetuals
 */

import { Hyperliquid } from "hyperliquid";

const TESTNET = process.env.HYPERLIQUID_TESTNET === "1";
const sdk     = new Hyperliquid(null, TESTNET);

const WATCHLIST = ["BTC","ETH","SOL","AVAX","ARB","DOGE","MATIC","LINK","OP","APT"];

async function main() {
  const mids = await sdk.info.getAllMids();

  console.log(`\n🦀 Hyperliquid Claw — Market Scan`);
  console.log(`   ${new Date().toLocaleTimeString()}\n`);

  for (const coin of WATCHLIST) {
    const price = parseFloat(mids[coin] || mids[`${coin}-PERP`] || 0);
    if (!price) continue;
    const formatted = price >= 1000
      ? `$${price.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
      : `$${price.toFixed(4)}`;
    console.log(`   ${coin.padEnd(6)} ${formatted}`);
  }

  // Also show first 20 available perps
  const meta  = await sdk.info.perpetuals.getMeta();
  const all   = (meta.universe || []).map(a => a.name).slice(0, 20);
  const extra = all.filter(c => !WATCHLIST.includes(c));

  if (extra.length) {
    console.log(`\n   Other perpetuals (first 20):`);
    console.log(`   ${extra.join(", ")}`);
  }
  console.log("");
}

main().catch(e => { console.error(e.message); process.exit(1); });
