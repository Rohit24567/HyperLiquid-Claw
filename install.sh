#!/usr/bin/env bash
# install.sh — HyperLiquid Claw v3 (Rust Edition) installer
# Supports: macOS, Linux (Ubuntu/Debian/Arch), Windows (Git Bash / WSL)
# Usage: bash install.sh

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
die()   { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

echo -e "${BOLD}"
echo "╔════════════════════════════════════════╗"
echo "║  🦀 HyperLiquid Claw v3 — Installer   ║"
echo "║     Rust Edition · OpenClaw Skill      ║"
echo "╚════════════════════════════════════════╝"
echo -e "${NC}"

# ── 1. Rust toolchain ─────────────────────────────────────────────────────────
info "Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
    warn "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
    source "$HOME/.cargo/env"
fi
RUST_VER=$(rustc --version | awk '{print $2}')
ok "Rust $RUST_VER"

# ── 2. Build ──────────────────────────────────────────────────────────────────
info "Building release binaries (this takes ~60s on first run)..."
cargo build --release --bins 2>&1 | tail -5
ok "Build complete"

BIN_DIR="$HOME/.cargo/bin"
cp target/release/hl-claw  "$BIN_DIR/hl-claw"
cp target/release/hl-mcp   "$BIN_DIR/hl-mcp"
ok "Installed → $BIN_DIR/hl-claw"
ok "Installed → $BIN_DIR/hl-mcp"

# ── 3. OpenClaw skill directory ───────────────────────────────────────────────
SKILL_DIR="$HOME/.openclaw/skills/hyperliquid"
info "Installing OpenClaw skill to $SKILL_DIR..."
mkdir -p "$SKILL_DIR"
cp SKILL.md "$SKILL_DIR/SKILL.md"

# Write the skill manifest
cat > "$SKILL_DIR/manifest.json" <<EOF
{
  "name": "hyperliquid-claw",
  "version": "3.0.0",
  "command": "hl-mcp",
  "transport": "stdio"
}
EOF
ok "Skill manifest installed"

# ── 4. Environment template ───────────────────────────────────────────────────
if [ ! -f "$SKILL_DIR/.env" ]; then
    cat > "$SKILL_DIR/.env" <<'EOF'
# HyperLiquid Claw — Environment Configuration
# Copy this file, fill in your values, and source it before running:
#   source ~/.openclaw/skills/hyperliquid/.env

# Read-only mode: just set your address
export HYPERLIQUID_ADDRESS=0xYourWalletAddress

# Trading mode: set your private key (keep this secret!)
# export HYPERLIQUID_PRIVATE_KEY=0xYourPrivateKey

# Uncomment to use testnet (safe for testing)
# export HYPERLIQUID_TESTNET=1
EOF
    ok "Created $SKILL_DIR/.env — edit it to add your wallet address"
else
    info ".env already exists, skipping"
fi

# ── 5. Verify ─────────────────────────────────────────────────────────────────
info "Verifying installation..."
if hl-claw --version &>/dev/null; then
    ok "hl-claw $(hl-claw --version)"
else
    warn "hl-claw not found in PATH. Add $BIN_DIR to your PATH."
fi

echo ""
echo -e "${BOLD}${GREEN}✅  HyperLiquid Claw v3 installed successfully!${NC}"
echo ""
echo -e "  ${BOLD}Quick start:${NC}"
echo "  1. Edit ~/.openclaw/skills/hyperliquid/.env"
echo "  2. source ~/.openclaw/skills/hyperliquid/.env"
echo "  3. hl-claw price BTC"
echo "  4. hl-claw scan"
echo ""
echo -e "  ${BOLD}OpenClaw:${NC}"
echo "  Restart OpenClaw — the 'hyperliquid-claw' skill will auto-load."
echo ""
