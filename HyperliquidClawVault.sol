// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title HyperliquidClawVault
 * @notice On-chain vault contract for the HyperLiquid Claw trading system.
 *
 * Responsibilities:
 *  1. Hold user USDC deposits used as perpetual trading collateral.
 *  2. Allow the designated operator (hl-claw bot) to mark trades and
 *     distribute P&L back to depositors.
 *  3. Enforce per-trade position limits and total exposure caps.
 *  4. Emit events consumed by off-chain Rust indexer for real-time P&L.
 *
 * Note: Actual order execution happens off-chain via the Hyperliquid API.
 *       This contract acts as the capital pool and accounting layer.
 */

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

contract HyperliquidClawVault is Ownable, ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;

    // ── State ──────────────────────────────────────────────────────────────────

    IERC20 public immutable usdc;

    /// Authorised trading operator (the hl-claw bot wallet)
    address public operator;

    /// Total USDC deposited across all users (6 decimals)
    uint256 public totalDeposited;

    /// Total realised P&L credited to depositors (may be negative)
    int256 public totalRealizedPnl;

    /// Per-user deposit ledger
    mapping(address => uint256) public deposits;

    /// Per-user realised P&L
    mapping(address => int256) public userPnl;

    // Safety limits (in USDC, 6 decimals)
    uint256 public maxPositionUsd;   // max single-trade notional
    uint256 public maxTotalExposure; // max aggregate open notional

    /// Current open notional tracked by operator
    uint256 public currentExposure;

    // ── Events ─────────────────────────────────────────────────────────────────

    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event TradeOpened(
        bytes32 indexed tradeId,
        string coin,
        bool isLong,
        uint256 sizeUsd,
        uint256 entryPrice
    );
    event TradeClosed(
        bytes32 indexed tradeId,
        string coin,
        int256 pnlUsd,
        uint256 exitPrice
    );
    event PnlDistributed(uint256 totalPnl, uint256 recipientCount);
    event OperatorUpdated(address indexed newOperator);
    event LimitsUpdated(uint256 maxPosition, uint256 maxExposure);

    // ── Modifiers ──────────────────────────────────────────────────────────────

    modifier onlyOperator() {
        require(msg.sender == operator, "HLC: caller is not operator");
        _;
    }

    // ── Constructor ────────────────────────────────────────────────────────────

    constructor(
        address _usdc,
        address _operator,
        uint256 _maxPositionUsd,
        uint256 _maxTotalExposure
    ) Ownable(msg.sender) {
        require(_usdc != address(0), "HLC: zero USDC address");
        require(_operator != address(0), "HLC: zero operator address");
        usdc = IERC20(_usdc);
        operator = _operator;
        maxPositionUsd = _maxPositionUsd;
        maxTotalExposure = _maxTotalExposure;
    }

    // ── User functions ─────────────────────────────────────────────────────────

    /**
     * @notice Deposit USDC into the trading vault.
     * @param amount Amount of USDC (6 decimals) to deposit.
     */
    function deposit(uint256 amount) external nonReentrant whenNotPaused {
        require(amount > 0, "HLC: zero deposit");
        usdc.safeTransferFrom(msg.sender, address(this), amount);
        deposits[msg.sender] += amount;
        totalDeposited += amount;
        emit Deposited(msg.sender, amount);
    }

    /**
     * @notice Withdraw USDC from the vault.
     *         Users can only withdraw their deposit minus any unrealised loss.
     * @param amount Amount to withdraw.
     */
    function withdraw(uint256 amount) external nonReentrant {
        require(amount > 0, "HLC: zero withdrawal");
        uint256 available = withdrawableBalance(msg.sender);
        require(available >= amount, "HLC: insufficient withdrawable balance");

        deposits[msg.sender] -= amount;
        totalDeposited -= amount;
        usdc.safeTransfer(msg.sender, amount);
        emit Withdrawn(msg.sender, amount);
    }

    /**
     * @notice Compute withdrawable balance for a user.
     *         Deducts their share of any negative P&L from their deposit.
     */
    function withdrawableBalance(address user) public view returns (uint256) {
        uint256 dep = deposits[user];
        int256 pnl = userPnl[user];
        if (pnl < 0) {
            uint256 loss = uint256(-pnl);
            return dep > loss ? dep - loss : 0;
        }
        return dep + uint256(pnl);
    }

    // ── Operator functions (called by hl-claw Rust bot) ───────────────────────

    /**
     * @notice Record that a trade has been opened on Hyperliquid.
     * @param tradeId Unique identifier (keccak256 of coin+timestamp).
     * @param coin    Asset ticker, e.g. "BTC".
     * @param isLong  True = long, false = short.
     * @param sizeUsd Notional value in USDC (6 decimals).
     * @param entryPrice Mark price at open (scaled ×10^6).
     */
    function recordTradeOpen(
        bytes32 tradeId,
        string calldata coin,
        bool isLong,
        uint256 sizeUsd,
        uint256 entryPrice
    ) external onlyOperator {
        require(sizeUsd <= maxPositionUsd, "HLC: position exceeds max size");
        require(
            currentExposure + sizeUsd <= maxTotalExposure,
            "HLC: total exposure limit exceeded"
        );
        currentExposure += sizeUsd;
        emit TradeOpened(tradeId, coin, isLong, sizeUsd, entryPrice);
    }

    /**
     * @notice Record trade close and distribute P&L proportionally.
     * @param tradeId     Must match a previously opened trade.
     * @param coin        Asset ticker.
     * @param pnlUsd      Signed P&L in USDC (6 decimals). Negative = loss.
     * @param sizeUsd     Notional (to reduce currentExposure).
     * @param exitPrice   Mark price at close (scaled ×10^6).
     * @param recipients  Depositor addresses to credit/debit P&L.
     */
    function recordTradeClose(
        bytes32 tradeId,
        string calldata coin,
        int256 pnlUsd,
        uint256 sizeUsd,
        uint256 exitPrice,
        address[] calldata recipients
    ) external onlyOperator nonReentrant {
        currentExposure = currentExposure > sizeUsd
            ? currentExposure - sizeUsd
            : 0;
        totalRealizedPnl += pnlUsd;

        // Distribute P&L proportionally by deposit share
        if (recipients.length > 0 && totalDeposited > 0) {
            for (uint256 i = 0; i < recipients.length; i++) {
                address r = recipients[i];
                uint256 share = deposits[r];
                if (share == 0) continue;
                int256 userShare = (pnlUsd * int256(share)) / int256(totalDeposited);
                userPnl[r] += userShare;
            }
        }

        // If there's positive P&L, transfer it into the vault
        if (pnlUsd > 0) {
            usdc.safeTransferFrom(msg.sender, address(this), uint256(pnlUsd));
        }

        emit TradeClosed(tradeId, coin, pnlUsd, exitPrice);
        emit PnlDistributed(
            pnlUsd > 0 ? uint256(pnlUsd) : 0,
            recipients.length
        );
    }

    // ── Admin functions ────────────────────────────────────────────────────────

    function setOperator(address _operator) external onlyOwner {
        require(_operator != address(0), "HLC: zero address");
        operator = _operator;
        emit OperatorUpdated(_operator);
    }

    function setLimits(
        uint256 _maxPositionUsd,
        uint256 _maxTotalExposure
    ) external onlyOwner {
        maxPositionUsd = _maxPositionUsd;
        maxTotalExposure = _maxTotalExposure;
        emit LimitsUpdated(_maxPositionUsd, _maxTotalExposure);
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    /// Emergency drain — only callable by owner, only when paused
    function emergencyWithdraw(address token, uint256 amount) external onlyOwner whenPaused {
        IERC20(token).safeTransfer(owner(), amount);
    }

    // ── View helpers ───────────────────────────────────────────────────────────

    function vaultBalance() external view returns (uint256) {
        return usdc.balanceOf(address(this));
    }

    function userSummary(address user)
        external
        view
        returns (
            uint256 deposited,
            int256 pnl,
            uint256 withdrawable
        )
    {
        return (deposits[user], userPnl[user], withdrawableBalance(user));
    }
}
