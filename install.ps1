# install.ps1 — HyperLiquid Claw v3 Windows Installer
# Run from PowerShell: .\install.ps1
# Requires: PowerShell 5+ or PowerShell Core 7+

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  🦀 HyperLiquid Claw v3 — Windows     ║" -ForegroundColor Cyan
Write-Host "║     Rust Edition · OpenClaw Skill      ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── 1. Check / install Rust ───────────────────────────────────────────────────
Write-Host "[INFO] Checking Rust toolchain..." -ForegroundColor Yellow
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "[INFO] Rust not found. Downloading rustup-init.exe..." -ForegroundColor Yellow
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupExe = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe
    Start-Process -FilePath $rustupExe -ArgumentList "-y", "--quiet" -Wait
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
    Write-Host "[OK] Rust installed" -ForegroundColor Green
} else {
    $rustVer = (& rustc --version)
    Write-Host "[OK] $rustVer" -ForegroundColor Green
}

# ── 2. Build ──────────────────────────────────────────────────────────────────
Write-Host "[INFO] Building release binaries (~60s first run)..." -ForegroundColor Yellow
& cargo build --release --bins
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

$cargobin = "$env:USERPROFILE\.cargo\bin"
Copy-Item "target\release\hl-claw.exe" "$cargobin\hl-claw.exe" -Force
Copy-Item "target\release\hl-mcp.exe"  "$cargobin\hl-mcp.exe"  -Force
Write-Host "[OK] Installed -> $cargobin\hl-claw.exe" -ForegroundColor Green
Write-Host "[OK] Installed -> $cargobin\hl-mcp.exe"  -ForegroundColor Green

# ── 3. OpenClaw skill ─────────────────────────────────────────────────────────
$skillDir = "$env:USERPROFILE\.openclaw\skills\hyperliquid"
New-Item -ItemType Directory -Force -Path $skillDir | Out-Null
Copy-Item "SKILL.md" "$skillDir\SKILL.md" -Force

$manifest = @{
    name      = "hyperliquid-claw"
    version   = "3.0.0"
    command   = "hl-mcp"
    transport = "stdio"
} | ConvertTo-Json

Set-Content "$skillDir\manifest.json" $manifest
Write-Host "[OK] OpenClaw skill installed" -ForegroundColor Green

# ── 4. .env template ──────────────────────────────────────────────────────────
$envPath = "$skillDir\.env"
if (-not (Test-Path $envPath)) {
    @"
# HyperLiquid Claw — Environment Configuration
# Set these in your PowerShell profile or run them before using hl-claw:
#
# `$env:HYPERLIQUID_ADDRESS = "0xYourWalletAddress"
# `$env:HYPERLIQUID_PRIVATE_KEY = "0xYourPrivateKey"
# `$env:HYPERLIQUID_TESTNET = "1"   # uncomment for testnet
"@ | Set-Content $envPath
    Write-Host "[OK] Created $envPath" -ForegroundColor Green
}

# ── 5. Verify ─────────────────────────────────────────────────────────────────
try {
    $ver = & hl-claw --version
    Write-Host "[OK] $ver" -ForegroundColor Green
} catch {
    Write-Host "[WARN] hl-claw not in PATH. Restart your terminal." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✅  HyperLiquid Claw v3 installed!" -ForegroundColor Green
Write-Host ""
Write-Host "  Quick start:" -ForegroundColor White
Write-Host '  $env:HYPERLIQUID_ADDRESS = "0xYourAddress"'
Write-Host "  hl-claw price BTC"
Write-Host "  hl-claw scan"
Write-Host ""
Write-Host "  Restart OpenClaw — the skill will auto-load." -ForegroundColor Cyan
Write-Host ""
