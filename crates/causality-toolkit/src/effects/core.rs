//! Core algebraic effect traits and types for automatic mock and test generation

use causality_core::system::content_addressing::ContentAddressable;
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Category of algebraic effect for automatic processing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectCategory {
    /// Asset transfer and token operations
    Asset,
    /// DeFi protocol interactions (swaps, lending, staking)
    DeFi,
    /// Data storage and retrieval operations
    Storage,
    /// Computational operations and processing
    Compute,
    /// Network communication and messaging
    Network,
}

/// Common failure modes for effects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureMode {
    // Asset/Token failures
    InsufficientBalance,
    InvalidAddress,
    TokenNotFound,
    
    // Network failures
    NetworkError,
    Timeout,
    ConnectionFailed,
    
    // Gas/Resource failures
    GasLimitExceeded,
    ResourceExhaustion,
    
    // DeFi-specific failures
    SlippageExceeded,
    InsufficientLiquidity,
    VaultCapacityExceeded,
    VaultPaused,
    InsufficientAllowance,
    
    // Storage failures
    StorageUnavailable,
    DataCorrupted,
    AccessDenied,
    
    // Compute failures
    ComputationFailed,
    InvalidInput,
    
    // Custom failure mode with description
    Custom(String),
}

/// Core trait for algebraic effects that enables automatic mock and test generation
/// 
/// Effects are pure data structures that describe what should happen, separating
/// interface from implementation. This trait provides metadata needed for automatic
/// mock generation, test case creation, and schema generation.
///
/// # Example
/// ```rust
/// use causality_toolkit::effects::core::{AlgebraicEffect, EffectCategory, FailureMode};
/// use causality_core::system::content_addressing::{ContentAddressable, EntityId};
/// use serde::{Serialize, Deserialize};
/// use std::time::Duration;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct TokenTransfer {
///     pub from: String,  // Simplified for example
///     pub to: String,
///     pub amount: u64,
/// }
///
/// impl ContentAddressable for TokenTransfer {
///     fn content_id(&self) -> EntityId {
///         // Simple implementation for example
///         let mut bytes = [0u8; 32];
///         bytes[0..8].copy_from_slice(&self.amount.to_le_bytes());
///         EntityId::from_bytes(bytes)
///     }
/// }
///
/// impl AlgebraicEffect for TokenTransfer {
///     type Result = String; // Simplified for example
///     type Error = String;
///     
///     fn effect_name() -> &'static str { "token_transfer" }
///     fn effect_category() -> EffectCategory { EffectCategory::Asset }
///     fn expected_duration() -> Duration { Duration::from_millis(200) }
///     fn failure_modes() -> Vec<FailureMode> {
///         vec![
///             FailureMode::InsufficientBalance,
///             FailureMode::InvalidAddress,
///             FailureMode::NetworkError,
///         ]
///     }
/// }
/// ```
pub trait AlgebraicEffect: ContentAddressable + Clone + Send + Sync + 'static {
    /// The success result type when the effect executes successfully
    type Result: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    
    /// The error type when the effect fails to execute
    type Error: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    
    /// Human-readable name for this effect type
    fn effect_name() -> &'static str;
    
    /// Category of this effect for automatic processing
    fn effect_category() -> EffectCategory;
    
    /// Expected duration for this effect under normal conditions
    fn expected_duration() -> Duration;
    
    /// Common failure modes that this effect can encounter
    fn failure_modes() -> Vec<FailureMode>;
    
    /// Whether this effect can be executed in parallel with others
    /// Default: true (most effects are parallelizable)
    fn is_parallelizable() -> bool {
        true
    }
    
    /// Whether this effect modifies external state
    /// Default: true (most effects have side effects)
    fn has_side_effects() -> bool {
        true
    }
    
    /// Estimated computational cost (arbitrary units for comparison)
    /// Default: 1 (minimal cost)
    fn computational_cost() -> u32 {
        1
    }
    
    /// Estimated gas cost for blockchain operations
    /// Default: 0 (no gas cost for non-blockchain effects)
    fn gas_cost() -> u64 {
        0
    }
}

/// Result of effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectResult<T, E> {
    /// Effect executed successfully
    Success(T),
    /// Effect failed with error
    Failure(E),
    /// Effect timed out during execution
    Timeout,
    /// Effect was cancelled before completion
    Cancelled,
}

impl<T, E> EffectResult<T, E> {
    /// Returns true if the effect was successful
    pub fn is_success(&self) -> bool {
        matches!(self, EffectResult::Success(_))
    }
    
    /// Returns true if the effect failed
    pub fn is_failure(&self) -> bool {
        matches!(self, EffectResult::Failure(_))
    }
    
    /// Returns true if the effect timed out
    pub fn is_timeout(&self) -> bool {
        matches!(self, EffectResult::Timeout)
    }
    
    /// Returns true if the effect was cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self, EffectResult::Cancelled)
    }
    
    /// Unwrap the success value, panicking if not successful
    pub fn unwrap(self) -> T {
        match self {
            EffectResult::Success(value) => value,
            _ => panic!("Called unwrap on non-success EffectResult"),
        }
    }
    
    /// Convert to a Result, treating timeout and cancellation as errors
    pub fn into_result(self) -> Result<T, EffectError<E>> {
        match self {
            EffectResult::Success(value) => Ok(value),
            EffectResult::Failure(error) => Err(EffectError::ExecutionFailed(error)),
            EffectResult::Timeout => Err(EffectError::Timeout),
            EffectResult::Cancelled => Err(EffectError::Cancelled),
        }
    }
}

/// Error types for effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectError<E> {
    /// Effect-specific execution error
    ExecutionFailed(E),
    /// Effect execution timed out
    Timeout,
    /// Effect was cancelled
    Cancelled,
    /// Invalid effect parameters
    InvalidParameters(String),
    /// Handler not found for effect type
    HandlerNotFound(String),
    /// Resource constraints prevented execution
    ResourceConstrained(String),
}

impl<E: std::fmt::Display> std::fmt::Display for EffectError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::ExecutionFailed(e) => write!(f, "Effect execution failed: {}", e),
            EffectError::Timeout => write!(f, "Effect execution timed out"),
            EffectError::Cancelled => write!(f, "Effect execution was cancelled"),
            EffectError::InvalidParameters(msg) => write!(f, "Invalid effect parameters: {}", msg),
            EffectError::HandlerNotFound(effect_type) => write!(f, "No handler found for effect type: {}", effect_type),
            EffectError::ResourceConstrained(msg) => write!(f, "Resource constraints: {}", msg),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for EffectError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EffectError::ExecutionFailed(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::system::content_addressing::EntityId;
    
    // Test effect for examples
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEffect {
        pub value: u32,
    }
    
    impl ContentAddressable for TestEffect {
        fn content_id(&self) -> EntityId {
            // For testing, create a simple EntityId from the value
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&self.value.to_le_bytes());
            EntityId::from_bytes(bytes)
        }
    }
    
    impl AlgebraicEffect for TestEffect {
        type Result = u32;
        type Error = String;
        
        fn effect_name() -> &'static str { "test_effect" }
        fn effect_category() -> EffectCategory { EffectCategory::Compute }
        fn expected_duration() -> Duration { Duration::from_millis(10) }
        fn failure_modes() -> Vec<FailureMode> {
            vec![FailureMode::ComputationFailed]
        }
    }
    
    #[test]
    fn test_effect_category_serialization() {
        let category = EffectCategory::DeFi;
        let serialized = serde_json::to_string(&category).unwrap();
        let deserialized: EffectCategory = serde_json::from_str(&serialized).unwrap();
        assert_eq!(category, deserialized);
    }
    
    #[test]
    fn test_failure_mode_serialization() {
        let failure = FailureMode::SlippageExceeded;
        let serialized = serde_json::to_string(&failure).unwrap();
        let deserialized: FailureMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(failure, deserialized);
    }
    
    #[test]
    fn test_custom_failure_mode() {
        let custom_failure = FailureMode::Custom("Custom error message".to_string());
        assert_eq!(
            format!("{:?}", custom_failure),
            "Custom(\"Custom error message\")"
        );
    }
    
    #[test]
    fn test_algebraic_effect_trait() {
        assert_eq!(TestEffect::effect_name(), "test_effect");
        assert_eq!(TestEffect::effect_category(), EffectCategory::Compute);
        assert_eq!(TestEffect::expected_duration(), Duration::from_millis(10));
        assert_eq!(TestEffect::failure_modes(), vec![FailureMode::ComputationFailed]);
        assert!(TestEffect::is_parallelizable());
        assert!(TestEffect::has_side_effects());
        assert_eq!(TestEffect::computational_cost(), 1);
        assert_eq!(TestEffect::gas_cost(), 0);
    }
    
    #[test]
    fn test_effect_result() {
        let success: EffectResult<u32, String> = EffectResult::Success(42);
        assert!(success.is_success());
        assert!(!success.is_failure());
        assert_eq!(success.unwrap(), 42);
        
        let failure: EffectResult<u32, String> = EffectResult::Failure("error".to_string());
        assert!(!failure.is_success());
        assert!(failure.is_failure());
        
        let timeout: EffectResult<u32, String> = EffectResult::Timeout;
        assert!(timeout.is_timeout());
        
        let cancelled: EffectResult<u32, String> = EffectResult::Cancelled;
        assert!(cancelled.is_cancelled());
    }
    
    #[test]
    fn test_effect_result_into_result() {
        let success: EffectResult<u32, String> = EffectResult::Success(42);
        assert_eq!(success.into_result().unwrap(), 42);
        
        let failure: EffectResult<u32, String> = EffectResult::Failure("error".to_string());
        assert!(failure.into_result().is_err());
        
        let timeout: EffectResult<u32, String> = EffectResult::Timeout;
        assert!(matches!(timeout.into_result(), Err(EffectError::Timeout)));
    }
    
    #[test]
    fn test_content_addressing() {
        let effect1 = TestEffect { value: 42 };
        let effect2 = TestEffect { value: 42 };
        let effect3 = TestEffect { value: 43 };
        
        // Same content should have same ID
        assert_eq!(effect1.content_id(), effect2.content_id());
        
        // Different content should have different ID
        assert_ne!(effect1.content_id(), effect3.content_id());
    }
}

/// Library for managing and organizing effects
#[derive(Debug)]
pub struct EffectLibrary {
    effects: std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl EffectLibrary {
    /// Create a new effect library
    pub fn new() -> Self {
        Self {
            effects: std::collections::HashMap::new(),
        }
    }
    
    /// Add an effect to the library
    pub fn add_effect<T: AlgebraicEffect>(&mut self, name: String, effect: T) {
        self.effects.insert(name, Box::new(effect));
    }
    
    /// Check if an effect exists in the library
    pub fn has_effect(&self, name: &str) -> bool {
        self.effects.contains_key(name)
    }
    
    /// Get the number of effects in the library
    pub fn count(&self) -> usize {
        self.effects.len()
    }
    
    /// List all effect names
    pub fn list_effects(&self) -> Vec<String> {
        self.effects.keys().cloned().collect()
    }
    
    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
    
    /// Execute a mathematical operation
    pub fn execute_math_operation(&self, operation: &str, args: Vec<i32>) -> Option<i32> {
        match operation {
            "add" => {
                if args.len() >= 2 {
                    Some(args[0] + args[1])
                } else {
                    None
                }
            }
            "multiply" => {
                if args.len() >= 2 {
                    Some(args[0] * args[1])
                } else {
                    None
                }
            }
            "subtract" => {
                if args.len() >= 2 {
                    Some(args[0] - args[1])
                } else {
                    None
                }
            }
            "divide" => {
                if args.len() >= 2 && args[1] != 0 {
                    Some(args[0] / args[1])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Execute a string operation
    pub fn execute_string_operation(&self, operation: &str, args: Vec<&str>) -> Option<String> {
        match operation {
            "concat" => {
                Some(args.join(""))
            }
            "uppercase" => {
                if !args.is_empty() {
                    Some(args[0].to_uppercase())
                } else {
                    None
                }
            }
            "lowercase" => {
                if !args.is_empty() {
                    Some(args[0].to_lowercase())
                } else {
                    None
                }
            }
            "reverse" => {
                if !args.is_empty() {
                    Some(args[0].chars().rev().collect())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for EffectLibrary {
    fn default() -> Self {
        Self::new()
    }
} 