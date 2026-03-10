# Hyperliquid API Reference

> Official docs: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api

## Base URLs

| Network | URL |
|---------|-----|
| Mainnet | `https://api.hyperliquid.xyz` |
| Testnet | `https://api.hyperliquid-testnet.xyz` |

## Info Endpoints (Read-Only)

All info requests are `POST /info` with a JSON body.

### Clearinghouse State (Balance + Positions)
```json
{ "type": "clearinghouseState", "user": "0x..." }
```

### Open Orders
```json
{ "type": "openOrders", "user": "0x..." }
```

### User Fills (Trade History)
```json
{ "type": "userFills", "user": "0x..." }
```

### All Mid Prices
```json
{ "type": "allMids" }
```

### Perpetuals Metadata (228+ assets)
```json
{ "type": "meta" }
```

## Exchange Endpoints (Trading)

All trading requests are `POST /exchange` and require ECDSA signature.

### Place Order
```json
{
  "action": {
    "type": "order",
    "orders": [{
      "a": 0,           // asset index (from meta)
      "b": true,        // is_buy
      "p": "45000",     // limit price (string)
      "s": "0.1",       // size (string)
      "r": false,       // reduce_only
      "t": { "limit": { "tif": "Gtc" } }  // or "Ioc" for market
    }],
    "grouping": "na"
  },
  "nonce": 1234567890,
  "signature": "0x..."
}
```

### Cancel Order
```json
{
  "action": {
    "type": "cancel",
    "cancels": [{ "a": 0, "o": 12345 }]   // asset index, order id
  },
  "nonce": 1234567890,
  "signature": "0x..."
}
```

## Order Types

| TIF | Meaning |
|-----|---------|
| `Gtc` | Good Till Cancelled (standard limit) |
| `Ioc` | Immediate or Cancel (market simulation) |
| `Alo` | Add Liquidity Only (post-only) |

## Notes

- Sizes are in base currency (BTC, ETH, etc.), not USD
- Prices are in USD
- Asset indices come from `/info` → `meta` → `universe[].name`
- All values are strings in the API; parse to float after receiving
- The official `hyperliquid` npm package handles signing automatically
