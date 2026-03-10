# Changelog

All notable changes to Hyperliquid Claw are documented here.

## [2.0.0] — 2026-01-27

### Added
- 🎉 Official Hyperliquid SDK integration
- 🐍 Python momentum engine (`analyze_market.py`)
- 🐍 Python signal classification module (`signals.py`)
- 🐍 Shared Python utilities (`utils.py`)
- 📊 CoinGecko chart + volume analysis (JS + Python, no API key needed)
- 🎯 Automated bull/bear momentum signal detection
- 📈 Real-time P&L position monitor (`check-positions.mjs`)
- 🔧 Full CLI overhaul with JSON output
- 🛡️ 5% slippage protection on all market orders
- 📝 Complete strategy documentation
- 🧪 Testnet support
- `install.sh` one-command installer

### Fixed
- All trading operations now use correct SDK method signatures
- Market orders properly use IOC limit orders with slippage buffer

## [1.0.0] — 2026-01-27

### Added
- 🚀 Initial release
- Basic portfolio monitoring (balance, positions, orders, fills)
- Market order execution
- Price queries
