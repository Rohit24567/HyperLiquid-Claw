#!/usr/bin/env node
/**
 * hyperliquid.mjs — Core trading client for Hyperliquid Claw
 * Uses the official Hyperliquid SDK
 */

import { Hyperliquid } from "hyperliquid";
import { createRequire } from "module";
import { readFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load .env if present
const envPath = join(__dirname, "..", ".env");
if (existsSync(envPath)) {
  const lines = readFileSync(envPath, "utf8").split("\n");
  for (const line of lines) {
    const [key, ...rest] = line.split("=");
    if (key && !key.startsWith("#") && rest.length) {
      process.env[key.trim()] = rest.join("=").trim();
    }
  }
}

const PRIVATE_KEY = process.env.HYPERLIQUID_PRIVATE_KEY;
const ADDRESS     = process.env.HYPERLIQUID_ADDRESS;
const TESTNET     = process.env.HYPERLIQUID_TESTNET === "1";

const sdk = new Hyperliquid(PRIVATE_KEY || null, TESTNET);

const [,, cmd, ...args] = process.argv;

async function requireAddress() {
  if (PRIVATE_KEY) return null; // SDK derives address from key
  if (!ADDRESS) {
    console.error(JSON.stringify({ error: "Address required. Set HYPERLIQUID_ADDRESS or HYPERLIQUID_PRIVATE_KEY." }));
    process.exit(1);
  }
  return ADDRESS;
}

async function requireKey() {
  if (!PRIVATE_KEY) {
    console.error(JSON.stringify({ error: "Private key required. Set HYPERLIQUID_PRIVATE_KEY to trade." }));
    process.exit(1);
  }
}

const commands = {
  async balance() {
    const addr = await requireAddress();
    const data = await sdk.info.perpetuals.getClearinghouseState(addr || undefined);
    console.log(JSON.stringify(data, null, 2));
  },

  async positions() {
    const addr = args[0] || await requireAddress();
    const data = await sdk.info.perpetuals.getClearinghouseState(addr || undefined);
    const positions = (data.assetPositions || []).filter(p => parseFloat(p.position?.szi || 0) !== 0);
    const equity    = parseFloat(data.marginSummary?.accountValue || 0);
    const available = parseFloat(data.withdrawable || 0);
    console.log(JSON.stringify({ equity, available, positions }, null, 2));
  },

  async orders() {
    const addr = await requireAddress();
    const data = await sdk.info.getOpenOrders(addr || undefined);
    console.log(JSON.stringify(data, null, 2));
  },

  async fills() {
    const addr = await requireAddress();
    const data = await sdk.info.getUserFills(addr || undefined);
    console.log(JSON.stringify(data.slice(0, 20), null, 2));
  },

  async price() {
    const coin = args[0];
    if (!coin) { console.error(JSON.stringify({ error: "Usage: price <COIN>" })); process.exit(1); }
    const mids = await sdk.info.getAllMids();
    const key  = coin.toUpperCase();
    const price = mids[key] || mids[`${key}-PERP`];
    if (!price) { console.error(JSON.stringify({ error: `Unknown coin: ${coin}` })); process.exit(1); }
    console.log(JSON.stringify({ coin: key, price: parseFloat(price) }, null, 2));
  },

  async meta() {
    const data = await sdk.info.perpetuals.getMeta();
    const coins = (data.universe || []).map(a => a.name);
    console.log(JSON.stringify({ count: coins.length, coins }, null, 2));
  },

  async ["market-buy"]() {
    await requireKey();
    const [coin, size] = args;
    if (!coin || !size) { console.error(JSON.stringify({ error: "Usage: market-buy <COIN> <SIZE>" })); process.exit(1); }
    const mids  = await sdk.info.getAllMids();
    const mark  = parseFloat(mids[coin.toUpperCase()] || mids[`${coin.toUpperCase()}-PERP`]);
    const limit = (mark * 1.05).toFixed(2);
    const result = await sdk.exchange.placeOrder({
      coin: coin.toUpperCase(),
      is_buy: true,
      sz: parseFloat(size),
      limit_px: parseFloat(limit),
      order_type: { limit: { tif: "Ioc" } },
      reduce_only: false,
    });
    console.log(JSON.stringify(result, null, 2));
  },

  async ["market-sell"]() {
    await requireKey();
    const [coin, size] = args;
    if (!coin || !size) { console.error(JSON.stringify({ error: "Usage: market-sell <COIN> <SIZE>" })); process.exit(1); }
    const mids  = await sdk.info.getAllMids();
    const mark  = parseFloat(mids[coin.toUpperCase()] || mids[`${coin.toUpperCase()}-PERP`]);
    const limit = (mark * 0.95).toFixed(2);
    const result = await sdk.exchange.placeOrder({
      coin: coin.toUpperCase(),
      is_buy: false,
      sz: parseFloat(size),
      limit_px: parseFloat(limit),
      order_type: { limit: { tif: "Ioc" } },
      reduce_only: false,
    });
    console.log(JSON.stringify(result, null, 2));
  },

  async ["limit-buy"]() {
    await requireKey();
    const [coin, size, price] = args;
    if (!coin || !size || !price) { console.error(JSON.stringify({ error: "Usage: limit-buy <COIN> <SIZE> <PRICE>" })); process.exit(1); }
    const result = await sdk.exchange.placeOrder({
      coin: coin.toUpperCase(),
      is_buy: true,
      sz: parseFloat(size),
      limit_px: parseFloat(price),
      order_type: { limit: { tif: "Gtc" } },
      reduce_only: false,
    });
    console.log(JSON.stringify(result, null, 2));
  },

  async ["limit-sell"]() {
    await requireKey();
    const [coin, size, price] = args;
    if (!coin || !size || !price) { console.error(JSON.stringify({ error: "Usage: limit-sell <COIN> <SIZE> <PRICE>" })); process.exit(1); }
    const result = await sdk.exchange.placeOrder({
      coin: coin.toUpperCase(),
      is_buy: false,
      sz: parseFloat(size),
      limit_px: parseFloat(price),
      order_type: { limit: { tif: "Gtc" } },
      reduce_only: false,
    });
    console.log(JSON.stringify(result, null, 2));
  },

  async ["cancel-all"]() {
    await requireKey();
    const coin   = args[0];
    const addr   = undefined; // derived from private key
    const orders = await sdk.info.getOpenOrders(addr);
    const targets = coin ? orders.filter(o => o.coin === coin.toUpperCase()) : orders;
    if (!targets.length) { console.log(JSON.stringify({ cancelled: 0, message: "No open orders." })); return; }
    const cancels = targets.map(o => ({ coin: o.coin, oid: o.oid }));
    const result  = await sdk.exchange.cancelOrder(cancels);
    console.log(JSON.stringify({ cancelled: cancels.length, result }, null, 2));
  },
};

if (!cmd || !commands[cmd]) {
  console.error(JSON.stringify({
    error: `Unknown command: ${cmd || "(none)"}`,
    available: Object.keys(commands),
  }));
  process.exit(1);
}

commands[cmd]().catch(e => {
  console.error(JSON.stringify({ error: e.message || String(e) }));
  process.exit(1);
});
