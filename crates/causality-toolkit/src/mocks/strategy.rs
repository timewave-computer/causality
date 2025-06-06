//! Mock strategy framework for automatic effect handler generation

use crate::effects::core::{EffectCategory, FailureMode};
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Mock strategy configuration for effect handler generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MockStrategy {
    /// Always return success with given value
    AlwaysSucceed {
        /// Pre-configured success value (JSON serialized)
        success_value: Option<String>,
    },
    
    /// Always return failure with given error
    AlwaysFail {
        /// Specific failure mode to use
        failure_mode: FailureMode,
        /// Custom error message
        error_message: Option<String>,
    },
    
    /// Probabilistic success/failure based on rates
    Probabilistic {
        /// Success rate (0.0 to 1.0)
        success_rate: f64,
        /// Possible failure modes with their probabilities
        failure_modes: Vec<(FailureMode, f64)>,
        /// Custom success value
        success_value: Option<String>,
    },
    
    /// Add realistic latency to responses
    Latency {
        /// Base strategy to wrap
        base_strategy: Box<MockStrategy>,
        /// Minimum response time
        min_latency: Duration,
        /// Maximum response time
        max_latency: Duration,
        /// Percentage of calls that timeout
        timeout_rate: f64,
    },
    
    /// Resource-constrained mock (gas, memory, etc.)
    ResourceConstrained {
        /// Base strategy to wrap
        base_strategy: Box<MockStrategy>,
        /// Available gas/compute units
        available_gas: u64,
        /// Gas cost per operation
        gas_per_operation: u64,
        /// Whether to fail when resources exhausted
        fail_on_exhaustion: bool,
    },
    
    /// Blockchain-specific mocking with realistic behavior
    Blockchain {
        /// Chain configuration
        chain_config: ChainConfig,
        /// Whether to simulate network conditions
        simulate_network: bool,
        /// Block confirmation requirements
        confirmations_required: u32,
    },
}

/// Blockchain chain configuration for realistic mocking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain identifier
    pub chain_id: u64,
    /// Average block time
    pub block_time: Duration,
    /// Base gas price
    pub base_gas_price: u64,
    /// Gas limit per block
    pub gas_limit: u64,
    /// Network congestion factor (1.0 = normal, >1.0 = congested)
    pub congestion_factor: f64,
    /// Finality requirements
    pub finality_blocks: u32,
}

/// Strategy configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Primary strategy to use
    pub strategy: MockStrategy,
    /// Whether to log mock calls for debugging
    pub log_calls: bool,
    /// Maximum execution time before timeout
    pub max_execution_time: Duration,
    /// Whether to validate parameters
    pub validate_parameters: bool,
}

impl MockStrategy {
    /// Create a simple always-succeed strategy
    pub fn always_succeed() -> Self {
        MockStrategy::AlwaysSucceed {
            success_value: None,
        }
    }
    
    /// Create a simple always-fail strategy
    pub fn always_fail(failure_mode: FailureMode) -> Self {
        MockStrategy::AlwaysFail {
            failure_mode,
            error_message: None,
        }
    }
    
    /// Create a probabilistic strategy with given success rate
    pub fn probabilistic(success_rate: f64) -> Self {
        MockStrategy::Probabilistic {
            success_rate,
            failure_modes: vec![(FailureMode::NetworkError, 1.0)],
            success_value: None,
        }
    }
    
    /// Create a latency-wrapped strategy
    pub fn with_latency(base: MockStrategy, min: Duration, max: Duration) -> Self {
        MockStrategy::Latency {
            base_strategy: Box::new(base),
            min_latency: min,
            max_latency: max,
            timeout_rate: 0.05, // 5% timeout rate
        }
    }
    
    /// Create a resource-constrained strategy
    pub fn resource_constrained(base: MockStrategy, available_gas: u64, gas_per_op: u64) -> Self {
        MockStrategy::ResourceConstrained {
            base_strategy: Box::new(base),
            available_gas,
            gas_per_operation: gas_per_op,
            fail_on_exhaustion: true,
        }
    }
    
    /// Create a blockchain strategy with given chain config
    pub fn blockchain(chain_config: ChainConfig) -> Self {
        MockStrategy::Blockchain {
            chain_config,
            simulate_network: true,
            confirmations_required: 12,
        }
    }
    
    /// Validate strategy parameters
    pub fn validate(&self) -> Result<(), StrategyError> {
        match self {
            MockStrategy::Probabilistic { success_rate, failure_modes, .. } => {
                if *success_rate < 0.0 || *success_rate > 1.0 {
                    return Err(StrategyError::InvalidParameter(
                        "success_rate must be between 0.0 and 1.0".to_string()
                    ));
                }
                
                let total_failure_prob: f64 = failure_modes.iter().map(|(_, prob)| prob).sum();
                if total_failure_prob < 0.0 || total_failure_prob > 1.0 {
                    return Err(StrategyError::InvalidParameter(
                        "total failure mode probability must be between 0.0 and 1.0".to_string()
                    ));
                }
            }
            
            MockStrategy::Latency { min_latency, max_latency, timeout_rate, base_strategy } => {
                if min_latency > max_latency {
                    return Err(StrategyError::InvalidParameter(
                        "min_latency cannot be greater than max_latency".to_string()
                    ));
                }
                
                if *timeout_rate < 0.0 || *timeout_rate > 1.0 {
                    return Err(StrategyError::InvalidParameter(
                        "timeout_rate must be between 0.0 and 1.0".to_string()
                    ));
                }
                
                base_strategy.validate()?;
            }
            
            MockStrategy::ResourceConstrained { gas_per_operation, available_gas, base_strategy, .. } => {
                if *gas_per_operation == 0 {
                    return Err(StrategyError::InvalidParameter(
                        "gas_per_operation must be greater than 0".to_string()
                    ));
                }
                
                if *available_gas == 0 {
                    return Err(StrategyError::InvalidParameter(
                        "available_gas must be greater than 0".to_string()
                    ));
                }
                
                base_strategy.validate()?;
            }
            
            MockStrategy::Blockchain { chain_config, .. } => {
                chain_config.validate()?;
            }
            
            _ => {} // AlwaysSucceed and AlwaysFail are always valid
        }
        
        Ok(())
    }
    
    /// Get expected categories that this strategy is suitable for
    pub fn suitable_categories(&self) -> Vec<EffectCategory> {
        match self {
            MockStrategy::Blockchain { .. } => vec![EffectCategory::Asset, EffectCategory::DeFi],
            MockStrategy::ResourceConstrained { .. } => vec![EffectCategory::Compute, EffectCategory::Storage],
            MockStrategy::Latency { .. } => vec![EffectCategory::Network, EffectCategory::Storage],
            _ => vec![
                EffectCategory::Asset,
                EffectCategory::DeFi,
                EffectCategory::Storage,
                EffectCategory::Compute,
                EffectCategory::Network,
            ],
        }
    }
}

impl ChainConfig {
    /// Create a default Ethereum-like chain config
    pub fn ethereum() -> Self {
        ChainConfig {
            chain_id: 1,
            block_time: Duration::from_secs(12),
            base_gas_price: 20_000_000_000, // 20 gwei
            gas_limit: 30_000_000,
            congestion_factor: 1.0,
            finality_blocks: 12,
        }
    }
    
    /// Create a fast testnet-like chain config
    pub fn testnet() -> Self {
        ChainConfig {
            chain_id: 31337,
            block_time: Duration::from_secs(2),
            base_gas_price: 1_000_000_000, // 1 gwei
            gas_limit: 30_000_000,
            congestion_factor: 0.5,
            finality_blocks: 3,
        }
    }
    
    /// Create a Layer 2 chain config
    pub fn layer2() -> Self {
        ChainConfig {
            chain_id: 137, // Polygon
            block_time: Duration::from_secs(2),
            base_gas_price: 30_000_000_000, // 30 gwei
            gas_limit: 20_000_000,
            congestion_factor: 1.2,
            finality_blocks: 256,
        }
    }
    
    /// Validate chain configuration
    pub fn validate(&self) -> Result<(), StrategyError> {
        if self.block_time.is_zero() {
            return Err(StrategyError::InvalidParameter(
                "block_time must be greater than zero".to_string()
            ));
        }
        
        if self.gas_limit == 0 {
            return Err(StrategyError::InvalidParameter(
                "gas_limit must be greater than zero".to_string()
            ));
        }
        
        if self.congestion_factor < 0.0 {
            return Err(StrategyError::InvalidParameter(
                "congestion_factor must be non-negative".to_string()
            ));
        }
        
        Ok(())
    }
}

impl StrategyConfig {
    /// Create a new strategy configuration
    pub fn new(strategy: MockStrategy) -> Self {
        StrategyConfig {
            strategy,
            log_calls: false,
            max_execution_time: Duration::from_secs(30),
            validate_parameters: true,
        }
    }
    
    /// Enable call logging
    pub fn with_logging(mut self) -> Self {
        self.log_calls = true;
        self
    }
    
    /// Set maximum execution time
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.max_execution_time = timeout;
        self
    }
    
    /// Disable parameter validation
    pub fn without_validation(mut self) -> Self {
        self.validate_parameters = false;
        self
    }
}

/// Errors that can occur during strategy operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum StrategyError {
    #[error("Invalid strategy parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Strategy not suitable for effect category: {0:?}")]
    UnsuitableCategory(EffectCategory),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Strategy execution failed: {0}")]
    ExecutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_always_succeed_strategy() {
        let strategy = MockStrategy::always_succeed();
        assert!(strategy.validate().is_ok());
        
        let categories = strategy.suitable_categories();
        assert!(categories.contains(&EffectCategory::Asset));
        assert!(categories.contains(&EffectCategory::DeFi));
    }
    
    #[test]
    fn test_always_fail_strategy() {
        let strategy = MockStrategy::always_fail(FailureMode::InsufficientBalance);
        assert!(strategy.validate().is_ok());
    }
    
    #[test]
    fn test_probabilistic_strategy_validation() {
        // Valid strategy
        let strategy = MockStrategy::probabilistic(0.8);
        assert!(strategy.validate().is_ok());
        
        // Invalid success rate
        let invalid_strategy = MockStrategy::Probabilistic {
            success_rate: 1.5, // Invalid: > 1.0
            failure_modes: vec![(FailureMode::NetworkError, 0.5)],
            success_value: None,
        };
        assert!(invalid_strategy.validate().is_err());
    }
    
    #[test]
    fn test_latency_strategy() {
        let base = MockStrategy::always_succeed();
        let strategy = MockStrategy::with_latency(
            base,
            Duration::from_millis(10),
            Duration::from_millis(100)
        );
        
        assert!(strategy.validate().is_ok());
    }
    
    #[test]
    fn test_resource_constrained_strategy() {
        let base = MockStrategy::always_succeed();
        let strategy = MockStrategy::resource_constrained(base, 1000, 10);
        assert!(strategy.validate().is_ok());
        
        let categories = strategy.suitable_categories();
        assert!(categories.contains(&EffectCategory::Compute));
        assert!(categories.contains(&EffectCategory::Storage));
    }
    
    #[test]
    fn test_blockchain_strategy() {
        let chain_config = ChainConfig::ethereum();
        let strategy = MockStrategy::blockchain(chain_config);
        assert!(strategy.validate().is_ok());
        
        let categories = strategy.suitable_categories();
        assert!(categories.contains(&EffectCategory::Asset));
        assert!(categories.contains(&EffectCategory::DeFi));
    }
    
    #[test]
    fn test_chain_config_validation() {
        let valid_config = ChainConfig::ethereum();
        assert!(valid_config.validate().is_ok());
        
        let invalid_config = ChainConfig {
            chain_id: 1,
            block_time: Duration::from_secs(0), // Invalid: zero block time
            base_gas_price: 1000,
            gas_limit: 1000,
            congestion_factor: 1.0,
            finality_blocks: 12,
        };
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_strategy_config() {
        let strategy = MockStrategy::always_succeed();
        let config = StrategyConfig::new(strategy)
            .with_logging()
            .with_timeout(Duration::from_secs(10))
            .without_validation();
            
        assert!(config.log_calls);
        assert_eq!(config.max_execution_time, Duration::from_secs(10));
        assert!(!config.validate_parameters);
    }
    
    #[test]
    fn test_strategy_serialization() {
        let strategy = MockStrategy::probabilistic(0.9);
        let serialized = serde_json::to_string(&strategy).unwrap();
        let deserialized: MockStrategy = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            MockStrategy::Probabilistic { success_rate, .. } => {
                assert_eq!(success_rate, 0.9);
            }
            _ => panic!("Deserialization failed"),
        }
    }
} 