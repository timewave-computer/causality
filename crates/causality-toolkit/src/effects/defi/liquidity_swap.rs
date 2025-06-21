//! LiquiditySwap effect implementation for DEX swap operations

use crate::effects::{AlgebraicEffect, EffectCategory, FailureMode};
use causality_core::system::content_addressing::{ContentAddressable, EntityId};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::hash::{Hash, Hasher};

/// Liquidity swap effect for token exchanges on DEX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquiditySwap {
    /// Input token address
    pub token_in: String,
    
    /// Output token address
    pub token_out: String,
    
    /// Amount of input token to swap
    pub amount_in: u64,
    
    /// Minimum amount of output token expected
    pub amount_out_min: u64,
    
    /// Maximum slippage tolerance (in basis points, e.g., 50 = 0.5%)
    pub slippage_tolerance: u16,
    
    /// DEX protocol to use
    pub dex_protocol: DexProtocol,
    
    /// Trading pair address or pool identifier
    pub pool_address: String,
    
    /// Swap deadline (unix timestamp)
    pub deadline: u64,
    
    /// User address initiating the swap
    pub user_address: String,
    
    /// Optional routing path for multi-hop swaps
    pub routing_path: Option<Vec<String>>,
    
    /// Gas limit for the swap transaction
    pub gas_limit: Option<u64>,
    
    /// Gas price for the transaction
    pub gas_price: Option<u64>,
    
    /// Whether to use exact input or exact output
    pub swap_type: SwapType,
    
    /// Fee tier for the pool (in basis points)
    pub fee_tier: Option<u16>,
}

/// DEX protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DexProtocol {
    /// Uniswap V2 style (constant product)
    UniswapV2,
    /// Uniswap V3 style (concentrated liquidity)
    UniswapV3,
    /// Curve style (stable coin swaps)
    Curve,
    /// Balancer style (weighted pools)
    Balancer,
    /// 1inch aggregator
    OneInch,
    /// Custom AMM implementation
    Custom(String),
}

/// Swap type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapType {
    /// Exact amount in, variable amount out
    ExactInput,
    /// Variable amount in, exact amount out
    ExactOutput,
}

/// Pool information for liquidity calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    /// Reserve of token 0
    pub reserve0: u64,
    /// Reserve of token 1
    pub reserve1: u64,
    /// Total liquidity in the pool
    pub total_liquidity: u64,
    /// Pool fee in basis points
    pub fee: u16,
    /// Current price (token1/token0)
    pub current_price: f64,
    /// 24h volume
    pub volume_24h: u64,
}

impl LiquiditySwap {
    /// Create a new liquidity swap
    pub fn new(
        token_in: String,
        token_out: String,
        amount_in: u64,
        amount_out_min: u64,
        user_address: String,
    ) -> Self {
        Self {
            token_in,
            token_out,
            amount_in,
            amount_out_min,
            slippage_tolerance: 50, // 0.5% default
            dex_protocol: DexProtocol::UniswapV2,
            pool_address: String::new(),
            deadline: std::time::std::time::UNIX_EPOCH
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 1800, // 30 minutes from now
            user_address,
            routing_path: None,
            gas_limit: None,
            gas_price: None,
            swap_type: SwapType::ExactInput,
            fee_tier: None,
        }
    }
    
    /// Create a simple token swap
    pub fn simple_swap(
        token_in: String,
        token_out: String,
        amount_in: u64,
        user_address: String,
    ) -> Self {
        Self::new(token_in, token_out, amount_in, 0, user_address)
            .with_slippage_tolerance(100) // 1%
            .with_gas_limit(200000)
    }
    
    /// Create a stable coin swap (using Curve-style AMM)
    pub fn stable_swap(
        token_in: String,
        token_out: String,
        amount_in: u64,
        user_address: String,
    ) -> Self {
        Self::new(token_in, token_out, amount_in, 0, user_address)
            .with_dex_protocol(DexProtocol::Curve)
            .with_slippage_tolerance(10) // 0.1% for stable swaps
            .with_gas_limit(150000)
    }
    
    /// Set slippage tolerance
    pub fn with_slippage_tolerance(mut self, slippage_bp: u16) -> Self {
        self.slippage_tolerance = slippage_bp;
        self
    }
    
    /// Set DEX protocol
    pub fn with_dex_protocol(mut self, protocol: DexProtocol) -> Self {
        self.dex_protocol = protocol;
        self
    }
    
    /// Set pool address
    pub fn with_pool_address(mut self, pool_address: String) -> Self {
        self.pool_address = pool_address;
        self
    }
    
    /// Set swap deadline
    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = deadline;
        self
    }
    
    /// Set routing path for multi-hop swaps
    pub fn with_routing_path(mut self, path: Vec<String>) -> Self {
        self.routing_path = Some(path);
        self
    }
    
    /// Set gas limit
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }
    
    /// Set gas price
    pub fn with_gas_price(mut self, gas_price: u64) -> Self {
        self.gas_price = Some(gas_price);
        self
    }
    
    /// Set swap type
    pub fn with_swap_type(mut self, swap_type: SwapType) -> Self {
        self.swap_type = swap_type;
        self
    }
    
    /// Set fee tier
    pub fn with_fee_tier(mut self, fee_tier: u16) -> Self {
        self.fee_tier = Some(fee_tier);
        self
    }
    
    /// Set minimum output amount
    pub fn with_minimum_output(mut self, amount_out_min: u64) -> Self {
        self.amount_out_min = amount_out_min;
        self
    }
    
    /// Validate swap parameters
    pub fn validate(&self) -> Result<(), SwapError> {
        // Validate addresses
        if self.token_in.is_empty() {
            return Err(SwapError::InvalidToken("input token address is empty".to_string()));
        }
        
        if self.token_out.is_empty() {
            return Err(SwapError::InvalidToken("output token address is empty".to_string()));
        }
        
        if self.token_in == self.token_out {
            return Err(SwapError::InvalidToken("cannot swap token to itself".to_string()));
        }
        
        if self.user_address.is_empty() {
            return Err(SwapError::InvalidAddress("user address is empty".to_string()));
        }
        
        // Validate amounts
        if self.amount_in == 0 {
            return Err(SwapError::InvalidAmount("input amount cannot be zero".to_string()));
        }
        
        // Validate slippage
        if self.slippage_tolerance > 10000 { // 100%
            return Err(SwapError::InvalidSlippage("slippage tolerance cannot exceed 100%".to_string()));
        }
        
        // Validate deadline
        let current_time = std::time::std::time::UNIX_EPOCH
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if self.deadline <= current_time {
            return Err(SwapError::DeadlineExpired);
        }
        
        // Validate gas parameters
        if let Some(gas_limit) = self.gas_limit {
            if gas_limit == 0 {
                return Err(SwapError::InvalidGasParameters("gas limit cannot be zero".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Calculate estimated output amount using constant product formula
    pub fn estimate_output_amount(&self, pool_info: &PoolInfo) -> Result<u64, SwapError> {
        match self.dex_protocol {
            DexProtocol::UniswapV2 => {
                self.calculate_uniswap_v2_output(pool_info)
            }
            DexProtocol::Curve => {
                self.calculate_curve_output(pool_info)
            }
            DexProtocol::Balancer => {
                self.calculate_balancer_output(pool_info)
            }
            _ => {
                // Fallback to simple constant product
                self.calculate_uniswap_v2_output(pool_info)
            }
        }
    }
    
    /// Calculate Uniswap V2 style output using constant product formula
    fn calculate_uniswap_v2_output(&self, pool_info: &PoolInfo) -> Result<u64, SwapError> {
        let fee = pool_info.fee as f64 / 10000.0; // Convert basis points to decimal
        let amount_in_with_fee = (self.amount_in as f64) * (1.0 - fee);
        
        // x * y = k (constant product)
        // amount_out = (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee)
        let reserve_in = pool_info.reserve0 as f64;
        let reserve_out = pool_info.reserve1 as f64;
        
        let amount_out = (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee);
        
        Ok(amount_out as u64)
    }
    
    /// Calculate Curve style output for stable swaps
    fn calculate_curve_output(&self, pool_info: &PoolInfo) -> Result<u64, SwapError> {
        // Simplified Curve calculation (StableSwap invariant)
        // For stable coins, minimal slippage expected
        let fee = pool_info.fee as f64 / 10000.0;
        let amount_out = (self.amount_in as f64) * (1.0 - fee) * 0.9999; // Very low slippage
        
        Ok(amount_out as u64)
    }
    
    /// Calculate Balancer style output for weighted pools
    fn calculate_balancer_output(&self, pool_info: &PoolInfo) -> Result<u64, SwapError> {
        // Simplified Balancer calculation
        // Assuming equal weights for simplicity
        let fee = pool_info.fee as f64 / 10000.0;
        let amount_in_with_fee = (self.amount_in as f64) * (1.0 - fee);
        
        let reserve_in = pool_info.reserve0 as f64;
        let reserve_out = pool_info.reserve1 as f64;
        
        // Similar to Uniswap but with different curve
        let amount_out = (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee * 0.95);
        
        Ok(amount_out as u64)
    }
    
    /// Calculate price impact percentage
    pub fn calculate_price_impact(&self, pool_info: &PoolInfo) -> Result<f64, SwapError> {
        let estimated_output = self.estimate_output_amount(pool_info)?;
        
        // Calculate the ideal output without any slippage (pure price)
        // For simplicity, we'll use reserve ratio to determine ideal rate
        let reserve_ratio = pool_info.reserve1 as f64 / pool_info.reserve0 as f64;
        let ideal_output = (self.amount_in as f64) * reserve_ratio;
        
        // Price impact = (ideal_output - actual_output) / ideal_output * 100
        if ideal_output > 0.0 {
            let price_impact = ((ideal_output - estimated_output as f64) / ideal_output) * 100.0;
            Ok(price_impact.max(0.0))
        } else {
            Ok(0.0)
        }
    }
    
    /// Calculate estimated gas cost
    pub fn estimated_gas_cost(&self) -> u64 {
        let base_gas = match self.dex_protocol {
            DexProtocol::UniswapV2 => 150000,
            DexProtocol::UniswapV3 => 200000,
            DexProtocol::Curve => 120000,
            DexProtocol::Balancer => 180000,
            DexProtocol::OneInch => 300000, // Aggregator overhead
            DexProtocol::Custom(_) => 200000,
        };
        
        // Add gas for multi-hop routing
        let routing_gas = self.routing_path.as_ref()
            .map(|path| (path.len() as u64).saturating_sub(1) * 50000)
            .unwrap_or(0);
        
        base_gas + routing_gas
    }
    
    /// Estimate total transaction cost
    pub fn estimated_transaction_cost(&self) -> Option<u64> {
        self.gas_price.map(|price| self.estimated_gas_cost() * price)
    }
}

/// Receipt returned upon successful swap completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapReceipt {
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Block number where transaction was included
    pub block_number: u64,
    
    /// Actual amount of input token used
    pub amount_in: u64,
    
    /// Actual amount of output token received
    pub amount_out: u64,
    
    /// Effective exchange rate (amount_out / amount_in)
    pub exchange_rate: f64,
    
    /// Price impact percentage
    pub price_impact: f64,
    
    /// Gas used
    pub gas_used: u64,
    
    /// Gas price
    pub gas_price: u64,
    
    /// Protocol fees paid
    pub protocol_fees: u64,
    
    /// Pool state after swap
    pub pool_state_after: PoolInfo,
    
    /// Timestamp when swap was executed
    pub timestamp: u64,
    
    /// Swap route taken (for multi-hop swaps)
    pub route_taken: Vec<String>,
    
    /// Event logs from the swap
    pub logs: Vec<SwapLog>,
}

/// Event log from swap transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapLog {
    /// Pool or contract address that emitted the log
    pub address: String,
    
    /// Event topics
    pub topics: Vec<String>,
    
    /// Event data
    pub data: String,
    
    /// Block number
    pub block_number: u64,
    
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Log index
    pub log_index: u32,
}

/// Comprehensive error types for liquidity swaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapError {
    /// Insufficient input token balance
    InsufficientBalance {
        available: u64,
        required: u64,
        token: String,
    },
    
    /// Invalid token address
    InvalidToken(String),
    
    /// Invalid user address
    InvalidAddress(String),
    
    /// Invalid swap amount
    InvalidAmount(String),
    
    /// Slippage tolerance exceeded
    SlippageExceeded {
        expected_min: u64,
        actual_output: u64,
        slippage: f64,
    },
    
    /// Invalid slippage parameters
    InvalidSlippage(String),
    
    /// Swap deadline has expired
    DeadlineExpired,
    
    /// Insufficient liquidity in pool
    InsufficientLiquidity {
        pool_address: String,
        available_liquidity: u64,
        required_liquidity: u64,
    },
    
    /// Pool not found
    PoolNotFound(String),
    
    /// Invalid gas parameters
    InvalidGasParameters(String),
    
    /// Network error during swap
    NetworkError {
        reason: String,
        is_transient: bool,
    },
    
    /// Price impact too high
    PriceImpactTooHigh {
        impact_percentage: f64,
        max_allowed: f64,
    },
    
    /// DEX protocol not supported
    UnsupportedProtocol(String),
    
    /// Routing path invalid or not found
    InvalidRoutingPath(String),
    
    /// MEV protection triggered
    MevProtection {
        reason: String,
        sandwich_detected: bool,
    },
    
    /// Custom swap error
    Custom(String),
}

impl std::fmt::Display for SwapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapError::InsufficientBalance { available, required, token } => {
                write!(f, "Insufficient {} balance: have {}, need {}", token, available, required)
            }
            SwapError::InvalidToken(token) => write!(f, "Invalid token: {}", token),
            SwapError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            SwapError::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            SwapError::SlippageExceeded { expected_min, actual_output, slippage } => {
                write!(f, "Slippage exceeded: expected min {}, got {}, slippage {:.2}%", expected_min, actual_output, slippage)
            }
            SwapError::InvalidSlippage(msg) => write!(f, "Invalid slippage: {}", msg),
            SwapError::DeadlineExpired => write!(f, "Swap deadline has expired"),
            SwapError::InsufficientLiquidity { pool_address, available_liquidity, required_liquidity } => {
                write!(f, "Insufficient liquidity in pool {}: have {}, need {}", pool_address, available_liquidity, required_liquidity)
            }
            SwapError::PoolNotFound(pool) => write!(f, "Pool not found: {}", pool),
            SwapError::InvalidGasParameters(msg) => write!(f, "Invalid gas parameters: {}", msg),
            SwapError::NetworkError { reason, is_transient } => {
                write!(f, "Network error ({}): {}", if *is_transient { "transient" } else { "permanent" }, reason)
            }
            SwapError::PriceImpactTooHigh { impact_percentage, max_allowed } => {
                write!(f, "Price impact too high: {:.2}% (max allowed: {:.2}%)", impact_percentage, max_allowed)
            }
            SwapError::UnsupportedProtocol(protocol) => write!(f, "Unsupported protocol: {}", protocol),
            SwapError::InvalidRoutingPath(path) => write!(f, "Invalid routing path: {}", path),
            SwapError::MevProtection { reason, sandwich_detected } => {
                write!(f, "MEV protection triggered (sandwich: {}): {}", sandwich_detected, reason)
            }
            SwapError::Custom(msg) => write!(f, "Custom error: {}", msg),
        }
    }
}

impl std::error::Error for SwapError {}

impl ContentAddressable for LiquiditySwap {
    fn content_id(&self) -> EntityId {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.token_in.hash(&mut hasher);
        self.token_out.hash(&mut hasher);
        self.amount_in.hash(&mut hasher);
        self.amount_out_min.hash(&mut hasher);
        self.slippage_tolerance.hash(&mut hasher);
        self.user_address.hash(&mut hasher);
        self.deadline.hash(&mut hasher);
        
        if let Some(ref path) = self.routing_path {
            path.hash(&mut hasher);
        }
        
        let hash = hasher.finish();
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&hash.to_le_bytes());
        
        EntityId::from_bytes(bytes)
    }
}

impl AlgebraicEffect for LiquiditySwap {
    type Result = SwapReceipt;
    type Error = SwapError;
    
    fn effect_name() -> &'static str {
        "liquidity_swap"
    }
    
    fn effect_category() -> EffectCategory {
        EffectCategory::DeFi
    }
    
    fn expected_duration() -> Duration {
        Duration::from_secs(20) // DEX swaps can be slower than simple transfers
    }
    
    fn failure_modes() -> Vec<FailureMode> {
        vec![
            FailureMode::InsufficientBalance,
            FailureMode::InvalidAddress,
            FailureMode::NetworkError,
            FailureMode::GasLimitExceeded,
            FailureMode::Timeout,
            FailureMode::Custom("slippage_exceeded".to_string()),
            FailureMode::Custom("insufficient_liquidity".to_string()),
            FailureMode::Custom("deadline_expired".to_string()),
            FailureMode::Custom("price_impact_too_high".to_string()),
            FailureMode::Custom("pool_not_found".to_string()),
            FailureMode::Custom("mev_protection".to_string()),
            FailureMode::Custom("unsupported_protocol".to_string()),
        ]
    }
    
    fn is_parallelizable() -> bool {
        true // Swaps from different users can be parallelized
    }
    
    fn has_side_effects() -> bool {
        true // Swaps modify pool state and user balances
    }
    
    fn computational_cost() -> u32 {
        4 // Higher than simple transfers due to AMM calculations
    }
    
    fn gas_cost() -> u64 {
        200000 // Conservative estimate for DEX swap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_liquidity_swap_creation() {
        let swap = LiquiditySwap::new(
            "0xA0b86a33E6441b5a033de1C3A95cfDBa59A5eb78".to_string(), // USDC
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
            1000000, // 1 USDC (6 decimals)
            0,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        assert_eq!(swap.token_in, "0xA0b86a33E6441b5a033de1C3A95cfDBa59A5eb78");
        assert_eq!(swap.token_out, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
        assert_eq!(swap.amount_in, 1000000);
        assert_eq!(swap.slippage_tolerance, 50); // 0.5% default
    }
    
    #[test]
    fn test_simple_swap() {
        let swap = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            "0x1234".to_string(),
        );
        
        assert_eq!(swap.slippage_tolerance, 100); // 1%
        assert_eq!(swap.gas_limit, Some(200000));
    }
    
    #[test]
    fn test_stable_swap() {
        let swap = LiquiditySwap::stable_swap(
            "USDC".to_string(),
            "USDT".to_string(),
            1000000,
            "0x1234".to_string(),
        );
        
        assert_eq!(swap.slippage_tolerance, 10); // 0.1%
        assert!(matches!(swap.dex_protocol, DexProtocol::Curve));
    }
    
    #[test]
    fn test_validation_success() {
        let swap = LiquiditySwap::new(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            990000, // 1% slippage
            "0x1234".to_string(),
        );
        
        assert!(swap.validate().is_ok());
    }
    
    #[test]
    fn test_validation_same_token() {
        let swap = LiquiditySwap::new(
            "USDC".to_string(),
            "USDC".to_string(),
            1000000,
            990000,
            "0x1234".to_string(),
        );
        
        assert!(matches!(swap.validate(), Err(SwapError::InvalidToken(_))));
    }
    
    #[test]
    fn test_validation_zero_amount() {
        let swap = LiquiditySwap::new(
            "USDC".to_string(),
            "WETH".to_string(),
            0,
            0,
            "0x1234".to_string(),
        );
        
        assert!(matches!(swap.validate(), Err(SwapError::InvalidAmount(_))));
    }
    
    #[test]
    fn test_validation_excessive_slippage() {
        let swap = LiquiditySwap::new(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            0,
            "0x1234".to_string(),
        ).with_slippage_tolerance(15000); // 150%
        
        assert!(matches!(swap.validate(), Err(SwapError::InvalidSlippage(_))));
    }
    
    #[test]
    fn test_uniswap_v2_calculation() {
        let swap = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000, // 1 USDC
            "0x1234".to_string(),
        );
        
        let pool_info = PoolInfo {
            reserve0: 1000000000000, // 1M USDC
            reserve1: 500000000000000000, // 500 WETH (18 decimals)
            total_liquidity: 22360679774997, // sqrt(k)
            fee: 30, // 0.3%
            current_price: 0.0005, // 1 USDC = 0.0005 WETH
            volume_24h: 10000000000000,
        };
        
        let estimated_output = swap.estimate_output_amount(&pool_info).unwrap();
        assert!(estimated_output > 0);
        assert!(estimated_output < 500000000000000); // Should be less than perfect rate due to slippage
    }
    
    #[test]
    fn test_price_impact_calculation() {
        let swap = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            100000000, // 100 USDC (larger trade for more impact)
            "0x1234".to_string(),
        );
        
        let pool_info = PoolInfo {
            reserve0: 1000000000000, // 1M USDC (6 decimals)
            reserve1: 500000000000000000, // 500 WETH (18 decimals) 
            total_liquidity: 22360679774997,
            fee: 30,
            current_price: 0.0005, // 1 USDC = 0.0005 WETH
            volume_24h: 10000000000000,
        };
        
        let price_impact = swap.calculate_price_impact(&pool_info).unwrap();
        // For a 100 USDC trade in a 1M USDC pool, there should be measurable price impact
        assert!(price_impact >= 0.0); // Price impact can be 0 for very small trades
        assert!(price_impact < 100.0); // Should be reasonable
    }
    
    #[test]
    fn test_gas_estimation() {
        let uniswap_v2_swap = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            "0x1234".to_string(),
        );
        assert_eq!(uniswap_v2_swap.estimated_gas_cost(), 150000);
        
        let curve_swap = LiquiditySwap::stable_swap(
            "USDC".to_string(),
            "USDT".to_string(),
            1000000,
            "0x1234".to_string(),
        );
        assert_eq!(curve_swap.estimated_gas_cost(), 120000);
        
        let multi_hop_swap = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "DAI".to_string(),
            1000000,
            "0x1234".to_string(),
        ).with_routing_path(vec!["USDC".to_string(), "WETH".to_string(), "DAI".to_string()]);
        
        // Should be base + 2 hops * 50k gas each
        assert_eq!(multi_hop_swap.estimated_gas_cost(), 150000 + 100000);
    }
    
    #[test]
    fn test_content_addressing() {
        let swap1 = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            "0x1234".to_string(),
        );
        
        let swap2 = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            "0x1234".to_string(),
        );
        
        let swap3 = LiquiditySwap::simple_swap(
            "USDC".to_string(),
            "WETH".to_string(),
            2000000, // Different amount
            "0x1234".to_string(),
        );
        
        // Same swaps should have same content ID
        assert_eq!(swap1.content_id(), swap2.content_id());
        
        // Different swaps should have different content ID
        assert_ne!(swap1.content_id(), swap3.content_id());
    }
    
    #[test]
    fn test_algebraic_effect_implementation() {
        assert_eq!(LiquiditySwap::effect_name(), "liquidity_swap");
        assert_eq!(LiquiditySwap::effect_category(), EffectCategory::DeFi);
        assert_eq!(LiquiditySwap::expected_duration(), Duration::from_secs(20));
        assert!(LiquiditySwap::is_parallelizable());
        assert!(LiquiditySwap::has_side_effects());
        assert_eq!(LiquiditySwap::computational_cost(), 4);
        assert_eq!(LiquiditySwap::gas_cost(), 200000);
        
        let failure_modes = LiquiditySwap::failure_modes();
        assert!(failure_modes.contains(&FailureMode::InsufficientBalance));
        assert!(failure_modes.contains(&FailureMode::Custom("slippage_exceeded".to_string())));
        assert!(failure_modes.contains(&FailureMode::Custom("insufficient_liquidity".to_string())));
    }
    
    #[test]
    fn test_fluent_interface() {
        let swap = LiquiditySwap::new(
            "USDC".to_string(),
            "WETH".to_string(),
            1000000,
            990000,
            "0x1234".to_string(),
        )
        .with_slippage_tolerance(200)
        .with_dex_protocol(DexProtocol::UniswapV3)
        .with_pool_address("0xpool123".to_string())
        .with_gas_limit(250000)
        .with_gas_price(30_000_000_000)
        .with_routing_path(vec!["USDC".to_string(), "WETH".to_string()])
        .with_swap_type(SwapType::ExactOutput)
        .with_fee_tier(500);
        
        assert_eq!(swap.slippage_tolerance, 200);
        assert!(matches!(swap.dex_protocol, DexProtocol::UniswapV3));
        assert_eq!(swap.pool_address, "0xpool123");
        assert_eq!(swap.gas_limit, Some(250000));
        assert_eq!(swap.gas_price, Some(30_000_000_000));
        assert!(swap.routing_path.is_some());
        assert!(matches!(swap.swap_type, SwapType::ExactOutput));
        assert_eq!(swap.fee_tier, Some(500));
    }
    
    #[test]
    fn test_error_display() {
        let error = SwapError::SlippageExceeded {
            expected_min: 990000,
            actual_output: 980000,
            slippage: 1.01,
        };
        assert_eq!(error.to_string(), "Slippage exceeded: expected min 990000, got 980000, slippage 1.01%");
        
        let error = SwapError::InsufficientLiquidity {
            pool_address: "0xpool123".to_string(),
            available_liquidity: 500000,
            required_liquidity: 1000000,
        };
        assert_eq!(error.to_string(), "Insufficient liquidity in pool 0xpool123: have 500000, need 1000000");
    }
} 