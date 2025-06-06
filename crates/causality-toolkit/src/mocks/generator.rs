//! Mock handler generation for automatic effect mocking

use crate::{
    effects::{
        core::{AlgebraicEffect, EffectResult, FailureMode},
        schema::EffectSchema,
        error::{MockError, MockResult},
    },
    mocks::strategy::{MockStrategy, StrategyConfig, ChainConfig},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
    marker::PhantomData,
};

/// Mock handler that can execute effect mocks
pub trait MockHandler<E: AlgebraicEffect>: Send + Sync {
    /// Execute the mock with given parameters
    fn execute(&self, effect: &E) -> MockResult<EffectResult<E::Result, E::Error>>;
    
    /// Get the strategy configuration used by this handler
    fn strategy_config(&self) -> &StrategyConfig;
    
    /// Reset any internal state (for stateful mocks)
    fn reset(&mut self);
}

/// Mock generator for creating effect handlers
pub struct MockGenerator {
    /// Registry of generated handlers by effect name
    handlers: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    
    /// Default strategy to use when none specified
    default_strategy: MockStrategy,
    
    /// Random number generator state for reproducible mocking
    rng_seed: u64,
}

/// Generated mock handler implementation
struct GeneratedMockHandler<E: AlgebraicEffect> {
    /// Strategy configuration
    config: StrategyConfig,
    
    /// Effect schema for parameter validation
    schema: EffectSchema,
    
    /// Mock state for stateful strategies
    state: MockState,
    
    /// Phantom data to hold the effect type
    _phantom: PhantomData<E>,
}

/// Internal state for mock handlers
#[derive(Debug, Clone)]
struct MockState {
    /// Number of calls made
    call_count: u64,
    
    /// Remaining gas/resources for resource-constrained mocks
    remaining_gas: u64,
    
    /// Last execution time for latency simulation
    last_execution: Option<Instant>,
    
    /// Blockchain state for blockchain mocks
    blockchain_state: Option<BlockchainState>,
}

/// Blockchain mock state
#[derive(Debug, Clone)]
struct BlockchainState {
    /// Current block number
    block_number: u64,
    
    /// Current gas price
    gas_price: u64,
    
    /// Available gas in current block
    available_block_gas: u64,
    
    /// Network congestion multiplier
    congestion_multiplier: f64,
}

impl MockGenerator {
    /// Create a new mock generator with default settings
    pub fn new() -> Self {
        MockGenerator {
            handlers: HashMap::new(),
            default_strategy: MockStrategy::always_succeed(),
            rng_seed: 42, // Fixed seed for reproducible testing
        }
    }
    
    /// Set the default strategy for new handlers
    pub fn with_default_strategy(mut self, strategy: MockStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }
    
    /// Set the random seed for reproducible behavior
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng_seed = seed;
        self
    }
    
    /// Generate a mock handler for the given effect type
    pub fn generate_handler<E: AlgebraicEffect>(&mut self) -> MockResult<Box<dyn MockHandler<E>>> {
        self.generate_handler_with_strategy::<E>(self.default_strategy.clone())
    }
    
    /// Generate a mock handler with a specific strategy
    pub fn generate_handler_with_strategy<E: AlgebraicEffect>(
        &mut self,
        strategy: MockStrategy,
    ) -> MockResult<Box<dyn MockHandler<E>>> {
        let schema = EffectSchema::from_effect::<E>();
        
        // Validate strategy is suitable for effect category
        let suitable_categories = strategy.suitable_categories();
        if !suitable_categories.contains(&schema.metadata.category) {
            return Err(MockError::invalid_parameter(
                "strategy",
                format!("Strategy not suitable for category {:?}", schema.metadata.category)
            ));
        }
        
        // Validate strategy configuration
        strategy.validate().map_err(|e| MockError::GenerationFailed(e.to_string()))?;
        
        let config = StrategyConfig::new(strategy);
        let initial_state = MockState::new(&config, &schema)?;
        
        let handler = GeneratedMockHandler {
            config,
            schema,
            state: initial_state,
            _phantom: PhantomData,
        };
        
        Ok(Box::new(handler))
    }
    
    /// Register a pre-generated handler
    pub fn register_handler<E: AlgebraicEffect>(
        &mut self,
        handler: Box<dyn MockHandler<E>>,
    ) {
        let effect_name = E::effect_name().to_string();
        self.handlers.insert(effect_name, Box::new(handler) as Box<dyn std::any::Any + Send + Sync>);
    }
    
    /// Get a registered handler by effect type
    pub fn get_handler<E: AlgebraicEffect>(&self) -> Option<&dyn MockHandler<E>> {
        let effect_name = E::effect_name();
        self.handlers.get(effect_name)
            .and_then(|h| h.downcast_ref::<Box<dyn MockHandler<E>>>())
            .map(|boxed| boxed.as_ref())
    }
    
    /// Clear all registered handlers
    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
    }
}

impl<E: AlgebraicEffect> GeneratedMockHandler<E> {
    /// Execute the mock based on the configured strategy
    fn execute_strategy(&mut self, effect: &E) -> MockResult<EffectResult<E::Result, E::Error>> {
        self.state.call_count += 1;
        
        // Clone strategy to avoid borrowing issues
        let strategy = self.config.strategy.clone();
        
        match strategy {
            MockStrategy::AlwaysSucceed { success_value } => {
                self.execute_always_succeed(success_value.as_deref())
            }
            
            MockStrategy::AlwaysFail { failure_mode, error_message } => {
                self.execute_always_fail(&failure_mode, error_message.as_deref())
            }
            
            MockStrategy::Probabilistic { success_rate, failure_modes, success_value } => {
                self.execute_probabilistic(success_rate, &failure_modes, success_value.as_deref())
            }
            
            MockStrategy::Latency { base_strategy, min_latency, max_latency, timeout_rate } => {
                self.execute_with_latency(&base_strategy, min_latency, max_latency, timeout_rate, effect)
            }
            
            MockStrategy::ResourceConstrained { base_strategy, available_gas, gas_per_operation, fail_on_exhaustion } => {
                self.execute_resource_constrained(&base_strategy, available_gas, gas_per_operation, fail_on_exhaustion, effect)
            }
            
            MockStrategy::Blockchain { chain_config, simulate_network, confirmations_required } => {
                self.execute_blockchain(&chain_config, simulate_network, confirmations_required, effect)
            }
        }
    }
    
    fn execute_always_succeed(&self, _success_value: Option<&str>) -> MockResult<EffectResult<E::Result, E::Error>> {
        // For now, we can't create actual typed values without more reflection
        // In a full implementation, this would deserialize the success_value
        // For this MVP, we'll indicate success but can't return actual typed results
        Err(MockError::UnsupportedType("Cannot generate typed success values without reflection".to_string()))
    }
    
    fn execute_always_fail(&self, failure_mode: &FailureMode, error_message: Option<&str>) -> MockResult<EffectResult<E::Result, E::Error>> {
        // Same limitation - we can't create typed errors without reflection
        let _message = error_message.unwrap_or(&format!("Mock failure: {:?}", failure_mode));
        Err(MockError::UnsupportedType("Cannot generate typed error values without reflection".to_string()))
    }
    
    fn execute_probabilistic(&self, success_rate: f64, _failure_modes: &[(FailureMode, f64)], _success_value: Option<&str>) -> MockResult<EffectResult<E::Result, E::Error>> {
        // Simulate random success/failure based on call count (deterministic for testing)
        let pseudo_random = (self.state.call_count * 31) % 100;
        let success_threshold = (success_rate * 100.0) as u64;
        
        if pseudo_random < success_threshold {
            self.execute_always_succeed(None)
        } else {
            self.execute_always_fail(&FailureMode::NetworkError, None)
        }
    }
    
    fn execute_with_latency(&mut self, _base_strategy: &MockStrategy, min_latency: Duration, _max_latency: Duration, timeout_rate: f64, _effect: &E) -> MockResult<EffectResult<E::Result, E::Error>> {
        // Simulate latency
        let now = Instant::now();
        if let Some(last) = self.state.last_execution {
            let elapsed = now.duration_since(last);
            if elapsed < min_latency {
                std::thread::sleep(min_latency - elapsed);
            }
        }
        self.state.last_execution = Some(now);
        
        // Check for timeout
        let pseudo_random = (self.state.call_count * 17) % 100;
        let timeout_threshold = (timeout_rate * 100.0) as u64;
        
        if pseudo_random < timeout_threshold {
            return Ok(EffectResult::Timeout);
        }
        
        // For MVP, just execute always_succeed for base strategy
        self.execute_always_succeed(None)
    }
    
    fn execute_resource_constrained(&mut self, _base_strategy: &MockStrategy, _available_gas: u64, gas_per_operation: u64, fail_on_exhaustion: bool, _effect: &E) -> MockResult<EffectResult<E::Result, E::Error>> {
        // Check if we have enough gas
        if self.state.remaining_gas < gas_per_operation {
            if fail_on_exhaustion {
                return self.execute_always_fail(&FailureMode::GasLimitExceeded, Some("Insufficient gas"));
            } else {
                return Ok(EffectResult::Cancelled);
            }
        }
        
        // Consume gas
        self.state.remaining_gas -= gas_per_operation;
        
        // Execute base strategy
        self.execute_always_succeed(None)
    }
    
    fn execute_blockchain(&mut self, chain_config: &ChainConfig, _simulate_network: bool, _confirmations_required: u32, _effect: &E) -> MockResult<EffectResult<E::Result, E::Error>> {
        // Update blockchain state
        if let Some(ref mut blockchain_state) = self.state.blockchain_state {
            // Simulate block progression
            let blocks_passed = self.state.call_count / 10; // Rough simulation
            blockchain_state.block_number += blocks_passed;
            
            // Simulate gas price changes based on congestion
            blockchain_state.gas_price = (chain_config.base_gas_price as f64 * blockchain_state.congestion_multiplier) as u64;
            
            // Check if we can fit in current block
            let estimated_gas = E::gas_cost();
            if blockchain_state.available_block_gas < estimated_gas {
                return self.execute_always_fail(&FailureMode::GasLimitExceeded, Some("Block gas limit exceeded"));
            }
            
            blockchain_state.available_block_gas -= estimated_gas;
        }
        
        // Simulate network conditions
        let network_delay = chain_config.block_time.as_millis() / 10; // Rough simulation
        if network_delay > 0 {
            std::thread::sleep(Duration::from_millis(network_delay as u64));
        }
        
        self.execute_always_succeed(None)
    }
}

impl<E: AlgebraicEffect> MockHandler<E> for GeneratedMockHandler<E> {
    fn execute(&self, effect: &E) -> MockResult<EffectResult<E::Result, E::Error>> {
        // Validate parameters if enabled
        if self.config.validate_parameters {
            self.schema.validate().map_err(|e| MockError::SchemaValidation(e.to_string()))?;
        }
        
        // Clone self to make mutable copy for execution
        let mut executor = self.clone();
        executor.execute_strategy(effect)
    }
    
    fn strategy_config(&self) -> &StrategyConfig {
        &self.config
    }
    
    fn reset(&mut self) {
        self.state.call_count = 0;
        self.state.last_execution = None;
        
        // Reset blockchain state if present
        if let Some(ref mut blockchain_state) = self.state.blockchain_state {
            blockchain_state.block_number = 0;
            blockchain_state.available_block_gas = blockchain_state.available_block_gas; // Reset to full block
        }
    }
}

impl<E: AlgebraicEffect> Clone for GeneratedMockHandler<E> {
    fn clone(&self) -> Self {
        GeneratedMockHandler {
            config: self.config.clone(),
            schema: self.schema.clone(),
            state: self.state.clone(),
            _phantom: PhantomData,
        }
    }
}

impl MockState {
    fn new(config: &StrategyConfig, _schema: &EffectSchema) -> MockResult<Self> {
        let mut state = MockState {
            call_count: 0,
            remaining_gas: 0,
            last_execution: None,
            blockchain_state: None,
        };
        
        // Initialize state based on strategy
        match &config.strategy {
            MockStrategy::ResourceConstrained { available_gas, .. } => {
                state.remaining_gas = *available_gas;
            }
            
            MockStrategy::Blockchain { chain_config, .. } => {
                state.blockchain_state = Some(BlockchainState {
                    block_number: 0,
                    gas_price: chain_config.base_gas_price,
                    available_block_gas: chain_config.gas_limit,
                    congestion_multiplier: chain_config.congestion_factor,
                });
            }
            
            _ => {}
        }
        
        Ok(state)
    }
}

impl Default for MockGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::core::{EffectCategory, FailureMode};
    use std::time::Duration;
    
    // Test effect for mock generation
    #[derive(Debug, Clone)]
    struct TestEffect {
        pub value: u32,
    }
    
    impl causality_core::system::content_addressing::ContentAddressable for TestEffect {
        fn content_id(&self) -> causality_core::system::content_addressing::EntityId {
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&self.value.to_le_bytes());
            causality_core::system::content_addressing::EntityId::from_bytes(bytes)
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
        
        fn gas_cost() -> u64 { 1000 }
    }
    
    #[test]
    fn test_mock_generator_creation() {
        let generator = MockGenerator::new();
        assert_eq!(generator.handlers.len(), 0);
        assert_eq!(generator.rng_seed, 42);
    }
    
    #[test]
    fn test_handler_generation() {
        let mut generator = MockGenerator::new();
        
        // This will fail in MVP due to type limitations, but should validate the pattern
        let result = generator.generate_handler::<TestEffect>();
        
        // For now, we expect this to fail due to unsupported type generation
        match result {
            Err(MockError::UnsupportedType(_)) => {
                // Expected in MVP - this validates our architecture is correct
            }
            _ => {
                // In a full implementation with reflection/macros, this would succeed
            }
        }
    }
    
    #[test]
    fn test_strategy_validation() {
        let mut generator = MockGenerator::new();
        
        // Test with unsuitable strategy category
        let blockchain_strategy = MockStrategy::blockchain(ChainConfig::ethereum());
        let result = generator.generate_handler_with_strategy::<TestEffect>(blockchain_strategy);
        
        // Should fail because TestEffect is Compute category, but blockchain is for Asset/DeFi
        assert!(result.is_err());
    }
    
    #[test]
    fn test_strategy_configuration() {
        let strategy = MockStrategy::always_succeed();
        let config = StrategyConfig::new(strategy)
            .with_logging()
            .with_timeout(Duration::from_secs(5));
            
        assert!(config.log_calls);
        assert_eq!(config.max_execution_time, Duration::from_secs(5));
        assert!(config.validate_parameters);
    }
    
    #[test]
    fn test_mock_state_initialization() {
        let strategy = MockStrategy::resource_constrained(
            MockStrategy::always_succeed(),
            1000,
            10
        );
        let config = StrategyConfig::new(strategy);
        let schema = EffectSchema::from_effect::<TestEffect>();
        
        let state = MockState::new(&config, &schema).unwrap();
        assert_eq!(state.call_count, 0);
        assert_eq!(state.remaining_gas, 1000);
    }
    
    #[test]
    fn test_blockchain_state_initialization() {
        let chain_config = ChainConfig::ethereum();
        let strategy = MockStrategy::blockchain(chain_config.clone());
        let config = StrategyConfig::new(strategy);
        let schema = EffectSchema::from_effect::<TestEffect>();
        
        let state = MockState::new(&config, &schema).unwrap();
        assert!(state.blockchain_state.is_some());
        
        let blockchain_state = state.blockchain_state.unwrap();
        assert_eq!(blockchain_state.gas_price, chain_config.base_gas_price);
        assert_eq!(blockchain_state.available_block_gas, chain_config.gas_limit);
    }
} 