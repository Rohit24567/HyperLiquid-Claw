# 🦀 Hyperliquid Claw — OpenClaw Skill Definition
# Version: 3.0.0 (Rust Edition)
# Compatible with: OpenClaw / clawd.bot

name: hyperliquid-claw
version: "3.0.0"
description: >
  AI-driven trading skill for Hyperliquid perpetual futures.
  Built in Rust for maximum performance and safety.
  Gives your OpenClaw assistant full read and trade access to all 228+
  perpetuals on the Hyperliquid DEX via a native MCP stdio server.

author: Hyperliquid Claw Contributors
license: MIT
homepage: https://github.com/YourUsername/HyperLiquid-Claw

# ── Runtime ──────────────────────────────────────────────────────────────────

runtime:
  type: binary
  command: hl-mcp          # compiled Rust binary; runs as MCP stdio server
  install_hint: >
    cargo install --path . --bin hl-mcp

environment:
  required: []
  optional:
    - name: HYPERLIQUID_ADDRESS
      description: Your wallet address (read-only mode — no private key needed)
      example: "0xABCDEF..."
    - name: HYPERLIQUID_PRIVATE_KEY
      description: Private key for executing trades (required for trading commands)
      example: "0xYourPrivateKey"
    - name: HYPERLIQUID_TESTNET
      description: Set to any value to use testnet instead of mainnet
      example: "1"

# ── Tools exposed to OpenClaw ─────────────────────────────────────────────────

tools:
  - name: hl_price
    description: Get the current mark price for any Hyperliquid perpetual
    parameters:
      coin: { type: string, required: true, description: "Asset ticker, e.g. BTC, ETH, SOL" }
    examples:
      - "What's the BTC price?"
      - "Show me the ETH mark price"

  - name: hl_meta
    description: List all 228+ tradeable perpetuals on Hyperliquid
    parameters: {}
    examples:
      - "What assets can I trade on Hyperliquid?"
      - "List all perpetuals"

  - name: hl_market_scan
    description: >
      Scan all markets for momentum signals using the Rust momentum engine.
      Detects strong bull/bear moves with volume confirmation.
    parameters:
      top_n: { type: integer, required: false, default: 10, description: "Number of top results" }
    examples:
      - "Analyze the crypto market on Hyperliquid"
      - "What's moving right now? Show me the top opportunities"
      - "Scan for trading signals"

  - name: hl_analyze
    description: >
      Deep momentum analysis for a single asset: price change, volume, OI,
      funding rate, signal classification and confidence score.
    parameters:
      coin: { type: string, required: true }
    examples:
      - "Analyze BTC momentum"
      - "What's the SOL signal right now?"
      - "Give me a full analysis of ETH"

  - name: hl_balance
    description: Show account equity, margin usage, and free margin
    parameters: {}
    examples:
      - "Check my portfolio"
      - "What's my account balance?"
      - "How much margin do I have left?"

  - name: hl_positions
    description: List all open perpetual positions with entry price, size, and unrealised P&L
    parameters: {}
    examples:
      - "Show my positions"
      - "What trades am I in?"
      - "Check my P&L"

  - name: hl_orders
    description: List all open limit orders
    parameters: {}
    examples:
      - "Show my open orders"
      - "What limit orders do I have?"

  - name: hl_fills
    description: Recent trade fills / execution history
    parameters:
      limit: { type: integer, required: false, default: 20 }
    examples:
      - "Show my recent trades"
      - "What fills did I get today?"

  - name: hl_market_buy
    description: >
      Place a market buy (long) order. Uses IOC limit with 5% slippage buffer.
      Warns if position exceeds 20% of equity.
    parameters:
      coin: { type: string, required: true }
      size: { type: number, required: true, description: "Size in coin units" }
    examples:
      - "Buy 0.1 BTC"
      - "Enter a SOL long with 0.5 SOL"
      - "Go long on ETH, 1 ETH"

  - name: hl_market_sell
    description: >
      Place a market sell / short order. Same safety features as market buy.
    parameters:
      coin: { type: string, required: true }
      size: { type: number, required: true }
    examples:
      - "Short 0.5 ETH"
      - "Sell 0.1 BTC"
      - "Close my BTC position"

  - name: hl_limit_buy
    description: Place a GTC limit buy order
    parameters:
      coin: { type: string, required: true }
      size: { type: number, required: true }
      price: { type: number, required: true }
    examples:
      - "Buy 0.001 BTC at 88000"
      - "Limit buy ETH at 3100 for 1 ETH"

  - name: hl_limit_sell
    description: Place a GTC limit sell order
    parameters:
      coin: { type: string, required: true }
      size: { type: number, required: true }
      price: { type: number, required: true }
    examples:
      - "Sell 1 ETH at 3500"
      - "Set a limit sell for 0.001 BTC at 95000"

  - name: hl_cancel_all
    description: Cancel all open orders, or all orders for a specific coin
    parameters:
      coin: { type: string, required: false }
    examples:
      - "Cancel all my orders"
      - "Cancel all BTC orders"

  - name: hl_set_leverage
    description: Set leverage for a coin (1–50x, cross or isolated margin)
    parameters:
      coin: { type: string, required: true }
      leverage: { type: integer, required: true, description: "1 to 50" }
      cross: { type: boolean, required: false, default: true, description: "true=cross, false=isolated" }
    examples:
      - "Set BTC to 10x cross leverage"
      - "Set ETH to 5x isolated"

# ── Momentum Strategy (built into the skill) ──────────────────────────────────

strategy:
  name: Momentum Scalp
  description: >
    Automated bull/bear detection via price change + volume confirmation.
    Implemented in the Rust MomentumEngine (src/analysis/signals.rs).

  entry_conditions:
    - price_change_pct: "> 0.5% or < -0.5% (24h)"
    - volume_vs_oi: "> 1.5× baseline"
    - funding_rate: "contrarian filter applied"

  risk_parameters:
    position_size_pct: 10        # % of account equity per trade
    max_loss_pct: 1              # stop loss
    take_profit_pct: 2           # profit target
    max_concurrent_positions: 1
    max_hold_hours: 4

  safety_limits:
    slippage_cap_pct: 5          # enforced in Rust ExchangeClient
    position_warning_pct: 20     # warn if trade > 20% equity
    limit_deviation_warning_pct: 5

# ── System prompt injection for OpenClaw ──────────────────────────────────────

system_prompt: |
  You have full access to the Hyperliquid perpetual futures DEX through
  the Hyperliquid Claw skill. You can:

  READ (no key needed):
  • Check prices, list markets, scan for signals
  • View account balance, open positions, order history

  TRADE (requires HYPERLIQUID_PRIVATE_KEY):
  • Place market and limit orders (buy/sell/long/short)
  • Cancel orders, set leverage
  • Close positions

  SAFETY RULES — always enforce these:
  1. Never risk more than 10% of equity on a single trade
  2. Always confirm with the user before placing orders
  3. Warn loudly if position would exceed 20% of equity
  4. Recommend stop loss at -1% and take profit at +2%
  5. Never auto-retry a failed trade
  6. On testnet: all actions are safe to test freely

  SIGNAL GUIDE:
  🟢 STRONG BULLISH → Consider long entry (confirm volume)
  🟡 BULLISH        → Wait for confirmation before entering
  ⚪ NEUTRAL        → No edge, stay flat
  🟠 BEARISH        → Wait for confirmation
  🔴 STRONG BEARISH → Consider short entry (confirm volume)
