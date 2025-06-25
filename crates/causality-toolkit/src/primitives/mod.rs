
//! Causality DeFi Primitives
//!
//! This module provides a comprehensive set of DeFi primitives that can be used
//! to build complex financial applications on the Causality platform. All primitives
//! support content addressing, ZK proofs, and cross-chain operations.

pub mod fungible_token;
pub mod non_fungible_token;
pub mod vault;
pub mod lending_market;
pub mod dex;

// Re-export all primitive types for convenience
pub use fungible_token::*;
pub use non_fungible_token::*;
pub use vault::*;
pub use lending_market::*;
pub use dex::*;

use causality_core::{Value, EntityId};
use causality_core::system::content_addressing::ContentAddressable;
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use anyhow::Result;
use thiserror::Error;

/// Common trait for all DeFi primitives
pub trait DeFiPrimitive: ContentAddressable + Serialize + for<'de> Deserialize<'de> + Clone {
    /// The type of asset this primitive operates on
    type Asset;
    
    /// The type of operation this primitive supports
    type Operation;
    
    /// The type of state this primitive maintains
    type State;
    
    /// Get the unique identifier for this primitive instance
    fn id(&self) -> EntityId;
    
    /// Get the current state of the primitive
    fn state(&self) -> &Self::State;
    
    /// Apply an operation to the primitive and return the new state
    fn apply_operation(&self, operation: Self::Operation) -> Result<Self::State>;
    
    /// Validate that an operation is allowed in the current state
    fn validate_operation(&self, operation: &Self::Operation) -> Result<()>;
    
    /// Get the primitive type name
    fn primitive_type(&self) -> &'static str;
    
    /// Get metadata about this primitive instance
    fn metadata(&self) -> BTreeMap<String, Value>;
}

/// Common asset types used across primitives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    /// Fungible token with specific denomination
    Fungible { denom: String, decimals: u8 },
    
    /// Non-fungible token with collection and token ID
    NonFungible { collection: String, token_id: String },
    
    /// Wrapped representation of external assets
    Wrapped { chain: String, contract: String, token_type: Box<AssetType> },
    
    /// LP token for DEX liquidity
    LiquidityToken { pool_id: String, token_a: Box<AssetType>, token_b: Box<AssetType> },
    
    /// Collateral representation for lending
    Collateral { underlying: Box<AssetType>, collateral_factor: u64 },
    
    /// Debt representation for lending
    Debt { underlying: Box<AssetType>, interest_rate: u64 },
}

/// Asset amount with type information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    /// Type of the asset
    pub asset_type: AssetType,
    
    /// Amount (in smallest unit for fungible, 1 for NFT)
    pub amount: u128,
    
    /// Optional metadata
    pub metadata: BTreeMap<String, Value>,
}

/// Common error types for DeFi operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum DeFiError {
    /// Insufficient balance for operation
    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: u128, required: u128 },
    
    /// Asset not found
    #[error("Asset not found: {asset_id}")]
    AssetNotFound { asset_id: String },
    
    /// Unauthorized operation
    #[error("Unauthorized operation '{operation}' by user '{user}'")]
    Unauthorized { user: String, operation: String },
    
    /// Invalid operation parameters
    #[error("Invalid operation: {reason}")]
    InvalidOperation { reason: String },
    
    /// Slippage exceeded
    #[error("Slippage exceeded: expected {expected}, actual {actual}")]
    SlippageExceeded { expected: u128, actual: u128 },
    
    /// Liquidation threshold breached
    #[error("Liquidation threshold breached: collateral ratio {collateral_ratio}, threshold {threshold}")]
    LiquidationThreshold { collateral_ratio: u64, threshold: u64 },
    
    /// Pool liquidity insufficient
    #[error("Insufficient liquidity in pool: {pool_id}")]
    InsufficientLiquidity { pool_id: String },
    
    /// Price oracle error
    #[error("Oracle error: {reason}")]
    OracleError { reason: String },
    
    /// Custom error with message
    #[error("Custom error: {message}")]
    Custom { message: String },
}

/// Result type for DeFi operations
pub type DeFiResult<T> = Result<T, DeFiError>;

/// Common configuration for all primitives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveConfig {
    /// Administrator addresses
    pub admins: Vec<String>,
    
    /// Fee configuration
    pub fees: FeeConfig,
    
    /// Access control settings
    pub access_control: AccessControlConfig,
    
    /// Emergency settings
    pub emergency: EmergencyConfig,
}

/// Fee configuration for primitives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeeConfig {
    /// Trading fees (basis points)
    pub trading_fee_bps: u64,
    
    /// Protocol fees (basis points)
    pub protocol_fee_bps: u64,
    
    /// Fee recipient address
    pub fee_recipient: String,
    
    /// Minimum fee amount
    pub minimum_fee: u128,
}

/// Configuration for access control
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Whether to enable access control
    pub enabled: bool,
    /// Whether the primitive is public or permissioned
    pub is_public: bool,
    /// Whitelisted users (if not public)  
    pub whitelist: Vec<String>,
    /// Blacklisted users
    pub blacklist: Vec<String>,
    /// Allowed principals
    pub allowed_principals: Vec<String>,
    /// Required permissions
    pub required_permissions: Vec<String>,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            is_public: true,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
            allowed_principals: Vec::new(),
            required_permissions: Vec::new(),
        }
    }
}

/// Emergency configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmergencyConfig {
    /// Whether emergency mode is enabled
    pub emergency_enabled: bool,
    
    /// Emergency admin addresses
    pub emergency_admins: Vec<String>,
    
    /// Circuit breaker thresholds
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Maximum price change percentage before trigger
    pub max_price_change_bps: u64,
    
    /// Maximum volume per time window
    pub max_volume_per_window: u128,
    
    /// Time window for volume measurement (seconds)
    pub volume_window_seconds: u64,
    
    /// Cool down period after trigger (seconds)
    pub cooldown_seconds: u64,
}

impl Default for PrimitiveConfig {
    fn default() -> Self {
        Self {
            admins: Vec::new(),
            fees: FeeConfig {
                trading_fee_bps: 30, // 0.3%
                protocol_fee_bps: 5,  // 0.05%
                fee_recipient: "fee_collector".to_string(),
                minimum_fee: 1,
            },
            access_control: AccessControlConfig {
                enabled: false,
                is_public: true,
                whitelist: Vec::new(),
                blacklist: Vec::new(),
                allowed_principals: Vec::new(),
                required_permissions: Vec::new(),
            },
            emergency: EmergencyConfig {
                emergency_enabled: false,
                emergency_admins: Vec::new(),
                circuit_breaker: CircuitBreakerConfig {
                    max_price_change_bps: 1000, // 10%
                    max_volume_per_window: u128::MAX,
                    volume_window_seconds: 3600, // 1 hour
                    cooldown_seconds: 1800, // 30 minutes
                },
            },
        }
    }
}

/// Helper function to calculate fees
pub fn calculate_fee(amount: u128, fee_bps: u64) -> u128 {
    (amount * fee_bps as u128) / 10000
}

/// Helper function to apply slippage protection
pub fn check_slippage(expected: u128, actual: u128, max_slippage_bps: u64) -> DeFiResult<()> {
    let max_deviation = (expected * max_slippage_bps as u128) / 10000;
    
    if actual < expected.saturating_sub(max_deviation) || actual > expected + max_deviation {
        return Err(DeFiError::SlippageExceeded { expected, actual });
    }
    
    Ok(())
}

/// Helper function to validate access control
pub fn check_access_control(user: &str, config: &AccessControlConfig) -> DeFiResult<()> {
    // Check blacklist first
    if config.blacklist.contains(&user.to_string()) {
        return Err(DeFiError::Unauthorized { 
            user: user.to_string(), 
            operation: "access_denied_blacklisted".to_string() 
        });
    }
    
    // Check whitelist if not public
    if !config.is_public && !config.whitelist.contains(&user.to_string()) {
        return Err(DeFiError::Unauthorized { 
            user: user.to_string(), 
            operation: "access_denied_not_whitelisted".to_string() 
        });
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let asset = Asset {
            asset_type: AssetType::Fungible { 
                denom: "USDC".to_string(), 
                decimals: 6 
            },
            amount: 1_000_000, // 1 USDC
            metadata: BTreeMap::new(),
        };
        
        assert_eq!(asset.amount, 1_000_000);
        match asset.asset_type {
            AssetType::Fungible { denom, decimals } => {
                assert_eq!(denom, "USDC");
                assert_eq!(decimals, 6);
            }
            _ => panic!("Expected fungible token"),
        }
    }
    
    #[test]
    fn test_fee_calculation() {
        // Test 0.3% fee on 1000 units
        let fee = calculate_fee(1000, 30);
        assert_eq!(fee, 3);
        
        // Test 1% fee on 10000 units
        let fee = calculate_fee(10000, 100);
        assert_eq!(fee, 100);
        
        // Test zero fee
        let fee = calculate_fee(1000, 0);
        assert_eq!(fee, 0);
    }
    
    #[test]
    fn test_slippage_check() {
        // Within tolerance
        assert!(check_slippage(1000, 1005, 100).is_ok()); // 0.5% change, 1% tolerance
        
        // Exceeds tolerance
        assert!(check_slippage(1000, 1020, 100).is_err()); // 2% change, 1% tolerance
        
        // Negative change within tolerance
        assert!(check_slippage(1000, 995, 100).is_ok()); // -0.5% change, 1% tolerance
        
        // Negative change exceeds tolerance
        assert!(check_slippage(1000, 980, 100).is_err()); // -2% change, 1% tolerance
    }
    
    #[test]
    fn test_access_control() {
        let mut config = AccessControlConfig::default();
        
        // Public access - should allow anyone not blacklisted
        assert!(check_access_control("user1", &config).is_ok());
        
        // Add to blacklist
        config.blacklist.push("user1".to_string());
        assert!(check_access_control("user1", &config).is_err());
        
        // Make private
        config.is_public = false;
        config.blacklist.clear();
        config.whitelist.push("user2".to_string());
        
        assert!(check_access_control("user1", &config).is_err()); // Not whitelisted
        assert!(check_access_control("user2", &config).is_ok()); // Whitelisted
    }
} 