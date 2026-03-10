"""
utils.py — Shared utilities for Hyperliquid Claw Python scripts
"""

import os
import json
from pathlib import Path
from typing import Any


# ── Environment ────────────────────────────────────────────────────────────────

def load_env() -> None:
    """Load .env from skill root if present."""
    env_path = Path(__file__).parent.parent / ".env"
    if not env_path.exists():
        return
    for line in env_path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, val = line.partition("=")
        os.environ.setdefault(key.strip(), val.strip())


def get_address() -> str | None:
    return os.environ.get("HYPERLIQUID_ADDRESS")


def get_private_key() -> str | None:
    return os.environ.get("HYPERLIQUID_PRIVATE_KEY")


def is_testnet() -> bool:
    return os.environ.get("HYPERLIQUID_TESTNET") == "1"


# ── Formatting ─────────────────────────────────────────────────────────────────

def fmt_price(price: float) -> str:
    if price >= 1000:
        return f"${price:,.2f}"
    if price >= 1:
        return f"${price:.4f}"
    return f"${price:.6f}"


def fmt_pct(value: float) -> str:
    sign = "+" if value >= 0 else ""
    return f"{sign}{value:.2f}%"


def fmt_volume(volume: float) -> str:
    if volume >= 1e9:
        return f"${volume / 1e9:.2f}B"
    if volume >= 1e6:
        return f"${volume / 1e6:.1f}M"
    return f"${volume:,.0f}"


def sparkline(values: list[float], width: int = 10) -> str:
    blocks = "▁▂▃▄▅▆▇█"
    subset = values[-width:]
    lo, hi = min(subset), max(subset)
    span = hi - lo or 1
    return "".join(blocks[round(((v - lo) / span) * 7)] for v in subset)


# ── Output ─────────────────────────────────────────────────────────────────────

def print_json(data: Any) -> None:
    print("--- JSON ---")
    print(json.dumps(data, indent=2))


def error_exit(msg: str) -> None:
    import sys
    print(json.dumps({"error": msg}))
    sys.exit(1)
