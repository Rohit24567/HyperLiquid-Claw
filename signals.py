"""
signals.py — Signal classification logic for Hyperliquid Claw
Shared between analyze_market.py and other Python scripts.
"""

from dataclasses import dataclass
from typing import Literal


SignalType = Literal[
    "STRONG BULLISH",
    "BULLISH",
    "NEUTRAL",
    "BEARISH",
    "STRONG BEARISH",
]


@dataclass
class Signal:
    type: SignalType
    emoji: str
    score: int
    action: str
    trade_direction: Literal["long", "short", "none"]
    confidence: Literal["high", "medium", "low"]

    def __str__(self) -> str:
        return f"{self.type} {self.emoji} (score: {self.score:+d})"


def classify(
    change_1h: float,
    change_6h: float,
    volume_ratio: float,
    rsi: float | None = None,
) -> Signal:
    """
    Score-based signal classification.

    Parameters
    ----------
    change_1h     : 1-hour price change in %
    change_6h     : 6-hour price change in %
    volume_ratio  : current volume / average volume
    rsi           : RSI(14) if available (optional)

    Returns
    -------
    Signal dataclass
    """
    score = 0

    # 1h momentum
    if   change_1h >  0.5: score += 2
    elif change_1h >  0.2: score += 1
    elif change_1h < -0.5: score -= 2
    elif change_1h < -0.2: score -= 1

    # 6h trend
    if   change_6h >  1.0: score += 2
    elif change_6h >  0.3: score += 1
    elif change_6h < -1.0: score -= 2
    elif change_6h < -0.3: score -= 1

    # Volume confirmation
    if   volume_ratio > 1.5: score += 1
    elif volume_ratio < 0.7: score -= 1

    # RSI filter (optional)
    if rsi is not None:
        if   rsi > 70: score -= 1   # overbought — dampens bullish
        elif rsi < 30: score += 1   # oversold   — dampens bearish

    if score >= 4:
        return Signal("STRONG BULLISH", "🚀", score, "HIGH-PROBABILITY LONG — full position size", "long", "high")
    if score >= 2:
        return Signal("BULLISH", "📈", score, "Consider long — confirm with volume", "long", "medium")
    if score <= -4:
        return Signal("STRONG BEARISH", "🔻", score, "HIGH-PROBABILITY SHORT — full position size", "short", "high")
    if score <= -2:
        return Signal("BEARISH", "📉", score, "Consider short — confirm with volume", "short", "medium")
    return Signal("NEUTRAL", "⚖️", score, "No clear edge — wait for setup", "none", "low")


def entry_criteria(signal: Signal, volume_ratio: float, min_change: float = 0.5) -> dict:
    """Return structured entry criteria for a given signal."""
    return {
        "trade":           signal.trade_direction != "none",
        "direction":       signal.trade_direction,
        "confidence":      signal.confidence,
        "volume_ok":       volume_ratio >= 1.5,
        "momentum_ok":     signal.score >= 4 or signal.score <= -4,
        "position_size_pct": 10 if signal.confidence == "high" else 5,
        "stop_loss_pct":   1.0,
        "take_profit_pct": 2.0,
    }
