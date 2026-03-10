#!/usr/bin/env node
/**
 * check-positions.mjs — Real-time P&L position monitor
 */

import { Hyperliquid } from "hyperliquid";
import { readFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const envPath   = join(__dirname, "..", ".env");
if (existsSync(envPath)) {
  for (const line of readFileSync(envPath, "utf8").split("\n")) {
    const [k, ...v] = line.split("=");
    if (k && !k.startsWith("#") && v.length) process.env[k.trim()] = v.join("=").trim();
  }
}

const PRIVATE_KEY = process.env.HYPERLIQUID_PRIVATE_KEY;
const ADDRESS     = process.env.HYPERLIQUID_ADDRESS;
const TESTNET     = process.env.HYPERLIQUID_TESTNET === "1";

if (!PRIVATE_KEY && !ADDRESS) {
  console.error("Set HYPERLIQUID_ADDRESS or HYPERLIQUID_PRIVATE_KEY");
  process.exit(1);
}

const sdk = new Hyperliquid(PRIVATE_KEY || null, TESTNET);

async function main() {
  const state     = await sdk.info.perpetuals.getClearinghouseState(ADDRESS || undefined);
  const equity    = parseFloat(state.marginSummary?.accountValue || 0);
  const available = parseFloat(state.withdrawable || 0);
  const positions = (state.assetPositions || []).filter(p => parseFloat(p.position?.szi || 0) !== 0);

  console.log(`\n🦀 Hyperliquid Claw — Position Monitor`);
  console.log(`   Equity:    $${equity.toFixed(2)}`);
  console.log(`   Available: $${available.toFixed(2)}\n`);

  if (!positions.length) {
    console.log("   No open positions.");
  } else {
    for (const ap of positions) {
      const p      = ap.position;
      const coin   = p.coin;
      const size   = parseFloat(p.szi);
      const entry  = parseFloat(p.entryPx);
      const upnl   = parseFloat(p.unrealizedPnl);
      const upnlPct = equity > 0 ? (upnl / equity) * 100 : 0;
      const dir    = size > 0 ? "LONG  📈" : "SHORT 📉";

      console.log(`   ${coin} — ${dir}`);
      console.log(`     Size:       ${Math.abs(size)} ${coin}`);
      console.log(`     Entry:      $${entry.toLocaleString()}`);
      console.log(`     Unrealized: $${upnl.toFixed(2)} (${upnlPct >= 0 ? "+" : ""}${upnlPct.toFixed(2)}%)`);

      if (upnlPct >= 2.0)  console.log(`     ✅ PROFIT TARGET HIT (+2%) — consider closing`);
      if (upnlPct <= -1.0) console.log(`     🛑 STOP LOSS HIT (-1%)    — consider closing`);
      console.log("");
    }
  }

  const mids = await sdk.info.getAllMids();
  const prices = ["BTC","ETH","SOL"].map(c => `${c}: $${parseFloat(mids[c] || 0).toLocaleString()}`).join(" · ");
  console.log(`   ${prices}\n`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
