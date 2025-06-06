//! Mock implementations for TokenTransfer effect with all strategies

use super::transfer::{TokenTransfer, TransferReceipt, TransferError, TransferLog, TransferBalances};
use crate::{
    effects::{EffectResult, FailureMode},
    mocks::{MockStrategy, ChainParams, MockChainState, ChainConfig},
    mocks::blockchain::{ForkChoiceParams, NetworkTopology},
};
use std::time::Duration;
use std::collections::HashMap;

/// Mock handler for TokenTransfer effects
pub struct TokenTransferMockHandler {
    /// Mock strategy to use
    strategy: MockStrategy,
    
    /// Chain state for stateful mocking
    chain_state: Option<MockChainState>,
    
    /// Execution history for consistency
    execution_history: Vec<TransferExecution>,
    
    /// Token balances (simplified - address -> balance)
    balances: HashMap<String, u64>,
}

/// Record of transfer execution for consistency tracking
#[derive(Debug, Clone)]
struct TransferExecution {
    transfer: TokenTransfer,
    result: EffectResult<TransferReceipt, TransferError>,
    timestamp: u64,
    nonce: u64,
}

impl TokenTransferMockHandler {
    /// Create a new mock handler with strategy
    pub fn new(strategy: MockStrategy) -> Self {
        let mut balances = HashMap::new();
        // Initialize some mock balances
        balances.insert("0x1234".to_string(), 1_000_000);
        balances.insert("0x5678".to_string(), 500_000);
        
        Self {
            strategy,
            chain_state: None,
            execution_history: Vec::new(),
            balances,
        }
    }
    
    /// Create a mock handler with blockchain state
    pub fn with_chain_state(strategy: MockStrategy, chain_config: ChainConfig) -> Self {
        let chain_params = ChainParams {
            chain_config,
            finality_confirmations: 12,
            max_mempool_size: 1000,
            mev_enabled: false,
            fork_choice: ForkChoiceParams {
                min_confirmations: 12,
                reorg_probability: 0.01,
                max_reorg_depth: 6,
            },
            network_topology: NetworkTopology {
                node_count: 10,
                avg_latency: Duration::from_millis(100),
                partition_probability: 0.001,
                node_failure_rate: 0.01,
            },
        };
        
        let chain_state = MockChainState::new(&chain_params);
        
        let mut balances = HashMap::new();
        balances.insert("0x1234".to_string(), 1_000_000);
        balances.insert("0x5678".to_string(), 500_000);
        
        Self {
            strategy,
            chain_state: Some(chain_state),
            execution_history: Vec::new(),
            balances,
        }
    }
    
    /// Execute transfer effect with mock implementation
    pub async fn execute(&mut self, transfer: TokenTransfer) -> EffectResult<TransferReceipt, TransferError> {
        // Validate transfer first
        if let Err(error) = transfer.validate() {
            return EffectResult::Failure(error);
        }
        
        // Clone strategy to avoid borrow issues
        let strategy = self.strategy.clone();
        
        // Execute based on strategy
        let result = match &strategy {
            MockStrategy::AlwaysSucceed { success_value: _ } => {
                self.execute_always_succeed(&transfer).await
            }
            
            MockStrategy::AlwaysFail { failure_mode, error_message } => {
                self.execute_always_fail(&transfer, failure_mode, error_message).await
            }
            
            MockStrategy::Probabilistic { success_rate, failure_modes, success_value: _ } => {
                self.execute_probabilistic(&transfer, *success_rate, failure_modes).await
            }
            
            MockStrategy::Latency { base_strategy, min_latency, max_latency, timeout_rate } => {
                self.execute_with_latency(&transfer, base_strategy, *min_latency, *max_latency, *timeout_rate).await
            }
            
            MockStrategy::ResourceConstrained { base_strategy, available_gas, gas_per_operation: _, fail_on_exhaustion } => {
                self.execute_resource_constrained(&transfer, base_strategy, *available_gas, *fail_on_exhaustion).await
            }
            
            MockStrategy::Blockchain { chain_config, simulate_network, confirmations_required } => {
                self.execute_blockchain(&transfer, chain_config, *simulate_network, *confirmations_required).await
            }
        };
        
        // Record execution history
        let execution = TransferExecution {
            transfer: transfer.clone(),
            result: result.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            nonce: self.execution_history.len() as u64,
        };
        
        self.execution_history.push(execution);
        
        result
    }
    
    /// Always succeed strategy implementation
    async fn execute_always_succeed(&self, transfer: &TokenTransfer) -> EffectResult<TransferReceipt, TransferError> {
        let receipt = TransferReceipt {
            transaction_hash: format!("0x{:064x}", rand::random::<u64>()),
            block_number: 12345678,
            transaction_index: 0,
            gas_used: transfer.estimated_gas_cost(),
            gas_price: transfer.gas_price.unwrap_or(20_000_000_000),
            confirmations: transfer.confirmations.unwrap_or(12),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: "success".to_string(),
            logs: vec![
                TransferLog {
                    address: transfer.token.clone(),
                    topics: vec![
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
                        format!("0x{:040x}", rand::random::<u64>()),
                        format!("0x{:040x}", rand::random::<u64>()),
                    ],
                    data: format!("0x{:064x}", transfer.amount),
                    block_number: 12345678,
                    transaction_hash: format!("0x{:064x}", rand::random::<u64>()),
                    log_index: 0,
                }
            ],
            final_balances: TransferBalances {
                sender_balance: 1000000 - transfer.amount, // Mock balance
                receiver_balance: 500000 + transfer.amount, // Mock balance
                token: transfer.token.clone(),
            },
        };
        
        EffectResult::Success(receipt)
    }
    
    /// Always fail strategy implementation
    async fn execute_always_fail(&self, transfer: &TokenTransfer, failure_mode: &FailureMode, error_message: &Option<String>) -> EffectResult<TransferReceipt, TransferError> {
        let error = match failure_mode {
            FailureMode::InsufficientBalance => TransferError::InsufficientBalance {
                available: transfer.amount / 2, // Mock insufficient amount
                required: transfer.amount,
                token: transfer.token.clone(),
            },
            FailureMode::InvalidAddress => TransferError::InvalidAddress("Invalid address format".to_string()),
            FailureMode::NetworkError => TransferError::NetworkError {
                reason: error_message.clone().unwrap_or_else(|| "Mock network failure".to_string()),
                is_transient: true,
            },
            FailureMode::GasLimitExceeded => TransferError::GasLimitExceeded {
                gas_limit: transfer.gas_limit.unwrap_or(65000),
                gas_required: transfer.gas_limit.unwrap_or(65000) + 10000,
            },
            FailureMode::InsufficientAllowance => TransferError::InsufficientAllowance {
                allowance: transfer.amount / 2,
                required: transfer.amount,
            },
            FailureMode::Custom(msg) => TransferError::Custom(msg.clone()),
            _ => TransferError::Custom(
                error_message.clone().unwrap_or_else(|| "Mock failure".to_string())
            ),
        };
        
        EffectResult::Failure(error)
    }
    
    /// Probabilistic strategy implementation
    async fn execute_probabilistic(&self, transfer: &TokenTransfer, success_rate: f64, failure_modes: &[(FailureMode, f64)]) -> EffectResult<TransferReceipt, TransferError> {
        if rand::random::<f64>() < success_rate {
            self.execute_always_succeed(transfer).await
        } else {
            // Pick a random failure mode
            let failure_mode = if failure_modes.is_empty() {
                &FailureMode::NetworkError
            } else {
                &failure_modes[rand::random::<usize>() % failure_modes.len()].0
            };
            
            self.execute_always_fail(transfer, failure_mode, &None).await
        }
    }
    
    /// Latency strategy implementation
    async fn execute_with_latency(&self, transfer: &TokenTransfer, base_strategy: &Box<MockStrategy>, min_latency: Duration, max_latency: Duration, timeout_rate: f64) -> EffectResult<TransferReceipt, TransferError> {
        // Check for timeout
        if rand::random::<f64>() < timeout_rate {
            return EffectResult::Timeout;
        }
        
        // Simulate latency
        let latency_range = max_latency.as_millis().saturating_sub(min_latency.as_millis());
        let latency = min_latency + Duration::from_millis(
            if latency_range > 0 {
                rand::random::<u64>() % (latency_range as u64)
            } else {
                0
            }
        );
        
        tokio::time::sleep(latency).await;
        
        // Execute base strategy (simplified - recursion would be complex)
        match base_strategy.as_ref() {
            MockStrategy::AlwaysSucceed { .. } => self.execute_always_succeed(transfer).await,
            MockStrategy::AlwaysFail { failure_mode, error_message } => {
                self.execute_always_fail(transfer, failure_mode, error_message).await
            }
            _ => self.execute_always_succeed(transfer).await, // Fallback
        }
    }
    
    /// Resource constrained strategy implementation
    async fn execute_resource_constrained(&self, transfer: &TokenTransfer, base_strategy: &Box<MockStrategy>, available_gas: u64, fail_on_exhaustion: bool) -> EffectResult<TransferReceipt, TransferError> {
        let required_gas = transfer.estimated_gas_cost();
        
        if required_gas > available_gas {
            if fail_on_exhaustion {
                return EffectResult::Failure(TransferError::GasLimitExceeded {
                    gas_limit: available_gas,
                    gas_required: required_gas,
                });
            } else {
                return EffectResult::Timeout;
            }
        }
        
        // Execute base strategy (simplified)
        match base_strategy.as_ref() {
            MockStrategy::AlwaysSucceed { .. } => self.execute_always_succeed(transfer).await,
            MockStrategy::AlwaysFail { failure_mode, error_message } => {
                self.execute_always_fail(transfer, failure_mode, error_message).await
            }
            _ => self.execute_always_succeed(transfer).await, // Fallback
        }
    }
    
    /// Blockchain strategy implementation with realistic simulation
    async fn execute_blockchain(&mut self, transfer: &TokenTransfer, chain_config: &ChainConfig, simulate_network: bool, confirmations_required: u32) -> EffectResult<TransferReceipt, TransferError> {
        // Simulate network conditions if enabled
        if simulate_network {
            let network_delay = Duration::from_millis(
                (50.0 * chain_config.congestion_factor) as u64
            );
            tokio::time::sleep(network_delay).await;
            
            // Network failure based on congestion
            if rand::random::<f64>() < (chain_config.congestion_factor - 1.0).max(0.0) * 0.1 {
                return EffectResult::Failure(TransferError::NetworkError {
                    reason: "Network congestion".to_string(),
                    is_transient: true,
                });
            }
        }
        
        // Check gas price against minimum
        let min_gas_price = (chain_config.base_gas_price as f64 * chain_config.congestion_factor) as u64;
        if let Some(gas_price) = transfer.gas_price {
            if gas_price < min_gas_price {
                return EffectResult::Failure(TransferError::InsufficientGasPrice {
                    provided: gas_price,
                    minimum: min_gas_price,
                });
            }
        }
        
        // Check balance using our simplified balance tracking
        let sender_balance = self.balances.get(&transfer.from).cloned().unwrap_or(0);
        if sender_balance < transfer.amount {
            return EffectResult::Failure(TransferError::InsufficientBalance {
                available: sender_balance,
                required: transfer.amount,
                token: transfer.token.clone(),
            });
        }
        
        // Update balances
        self.balances.insert(transfer.from.clone(), sender_balance - transfer.amount);
        let receiver_balance = self.balances.get(&transfer.to).cloned().unwrap_or(0);
        self.balances.insert(transfer.to.clone(), receiver_balance + transfer.amount);
        
        // Simulate block confirmation delay
        let confirmation_delay = chain_config.block_time * confirmations_required;
        tokio::time::sleep(confirmation_delay).await;
        
        // Create realistic receipt
        let receipt = TransferReceipt {
            transaction_hash: format!("0x{:064x}", rand::random::<u64>()),
            block_number: 12345678 + rand::random::<u64>() % 1000,
            transaction_index: rand::random::<u32>() % 100,
            gas_used: transfer.estimated_gas_cost(),
            gas_price: transfer.gas_price.unwrap_or(min_gas_price),
            confirmations: confirmations_required,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: "success".to_string(),
            logs: self.generate_realistic_logs(transfer),
            final_balances: TransferBalances {
                sender_balance: self.balances.get(&transfer.from).cloned().unwrap_or(0),
                receiver_balance: self.balances.get(&transfer.to).cloned().unwrap_or(0),
                token: transfer.token.clone(),
            },
        };
        
        EffectResult::Success(receipt)
    }
    
    /// Generate realistic event logs for transfer
    fn generate_realistic_logs(&self, transfer: &TokenTransfer) -> Vec<TransferLog> {
        let mut logs = Vec::new();
        
        if transfer.token != "ETH" {
            // ERC-20 Transfer event
            logs.push(TransferLog {
                address: transfer.token.clone(),
                topics: vec![
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(), // Transfer(address,address,uint256)
                    format!("0x{:040x}", self.address_to_u64(&transfer.from)),
                    format!("0x{:040x}", self.address_to_u64(&transfer.to)),
                ],
                data: format!("0x{:064x}", transfer.amount),
                block_number: 12345678,
                transaction_hash: format!("0x{:064x}", rand::random::<u64>()),
                log_index: 0,
            });
        }
        
        logs
    }
    
    /// Convert address string to u64 for log generation (simplified)
    fn address_to_u64(&self, address: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get execution history for analysis
    pub fn get_execution_history(&self) -> &[TransferExecution] {
        &self.execution_history
    }
    
    /// Get chain state for inspection
    pub fn get_chain_state(&self) -> Option<&MockChainState> {
        self.chain_state.as_ref()
    }
    
    /// Reset handler state
    pub fn reset(&mut self) {
        self.execution_history.clear();
        // Reset balances to initial state
        self.balances.clear();
        self.balances.insert("0x1234".to_string(), 1_000_000);
        self.balances.insert("0x5678".to_string(), 500_000);
    }
}

/// Factory for creating TokenTransfer mock handlers
pub struct TokenTransferMockFactory;

impl TokenTransferMockFactory {
    /// Create a simple success mock
    pub fn always_succeed() -> TokenTransferMockHandler {
        TokenTransferMockHandler::new(MockStrategy::AlwaysSucceed { success_value: None })
    }
    
    /// Create a simple failure mock
    pub fn always_fail(reason: &str) -> TokenTransferMockHandler {
        let failure_mode = match reason {
            "insufficient_balance" => FailureMode::InsufficientBalance,
            "invalid_address" => FailureMode::InvalidAddress,
            "network_error" => FailureMode::NetworkError,
            "gas_limit_exceeded" => FailureMode::GasLimitExceeded,
            _ => FailureMode::Custom(reason.to_string()),
        };
        
        TokenTransferMockHandler::new(MockStrategy::AlwaysFail {
            failure_mode,
            error_message: Some(format!("Mock failure: {}", reason)),
        })
    }
    
    /// Create a probabilistic mock
    pub fn probabilistic(success_rate: f64) -> TokenTransferMockHandler {
        TokenTransferMockHandler::new(MockStrategy::Probabilistic {
            success_rate,
            failure_modes: vec![
                (FailureMode::InsufficientBalance, 0.4),
                (FailureMode::NetworkError, 0.3),
                (FailureMode::GasLimitExceeded, 0.3),
            ],
            success_value: None,
        })
    }
    
    /// Create a latency-aware mock
    pub fn with_latency(base_success_rate: f64, min_latency: Duration, max_latency: Duration) -> TokenTransferMockHandler {
        let base_strategy = Box::new(MockStrategy::Probabilistic {
            success_rate: base_success_rate,
            failure_modes: vec![(FailureMode::NetworkError, 1.0)],
            success_value: None,
        });
        
        TokenTransferMockHandler::new(MockStrategy::Latency {
            base_strategy,
            min_latency,
            max_latency,
            timeout_rate: 0.01, // 1% timeout rate
        })
    }
    
    /// Create a blockchain-realistic mock
    pub fn blockchain_realistic(chain_type: &str) -> TokenTransferMockHandler {
        let chain_config = match chain_type {
            "ethereum" => ChainConfig::ethereum(),
            "polygon" | "layer2" => ChainConfig::layer2(),
            "avalanche" | "testnet" => ChainConfig::testnet(),
            _ => ChainConfig::ethereum(),
        };
        
        TokenTransferMockHandler::with_chain_state(
            MockStrategy::Blockchain {
                chain_config: chain_config.clone(),
                simulate_network: true,
                confirmations_required: 12,
            },
            chain_config,
        )
    }
    
    /// Create a gas-constrained mock
    pub fn gas_constrained(available_gas: u64) -> TokenTransferMockHandler {
        let base_strategy = Box::new(MockStrategy::AlwaysSucceed { success_value: None });
        
        TokenTransferMockHandler::new(MockStrategy::ResourceConstrained {
            base_strategy,
            available_gas,
            gas_per_operation: 21000,
            fail_on_exhaustion: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_always_succeed_mock() {
        let mut handler = TokenTransferMockFactory::always_succeed();
        
        let transfer = TokenTransfer::eth_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        );
        
        let result = handler.execute(transfer).await;
        assert!(result.is_success());
        
        if let EffectResult::Success(receipt) = result {
            assert_eq!(receipt.status, "success");
            assert!(receipt.transaction_hash.starts_with("0x"));
            assert_eq!(receipt.gas_used, 21000);
        }
    }
    
    #[tokio::test]
    async fn test_always_fail_mock() {
        let mut handler = TokenTransferMockFactory::always_fail("insufficient_balance");
        
        let transfer = TokenTransfer::eth_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        );
        
        let result = handler.execute(transfer).await;
        assert!(result.is_failure());
        
        if let EffectResult::Failure(error) = result {
            assert!(matches!(error, TransferError::InsufficientBalance { .. }));
        }
    }
    
    #[tokio::test]
    async fn test_probabilistic_mock() {
        let mut handler = TokenTransferMockFactory::probabilistic(0.8);
        
        let transfer = TokenTransfer::eth_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        );
        
        // Run multiple times to test probabilistic behavior
        let mut successes = 0;
        let mut failures = 0;
        
        for _ in 0..20 {
            let result = handler.execute(transfer.clone()).await;
            if result.is_success() {
                successes += 1;
            } else {
                failures += 1;
            }
        }
        
        // With 80% success rate, we should have more successes than failures
        assert!(successes > failures);
        assert!(successes > 10); // Should have at least some successes
    }
    
    #[tokio::test]
    async fn test_blockchain_realistic_mock() {
        let mut handler = TokenTransferMockFactory::blockchain_realistic("ethereum");
        
        let transfer = TokenTransfer::eth_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        ).with_gas_price(25_000_000_000);
        
        let result = handler.execute(transfer).await;
        
        // Should succeed with proper gas price
        assert!(result.is_success());
        
        if let EffectResult::Success(receipt) = result {
            assert_eq!(receipt.confirmations, 12);
            assert!(receipt.gas_price >= 20_000_000_000); // Should be at least base price
        }
    }
    
    #[tokio::test]
    async fn test_gas_constrained_mock() {
        let mut handler = TokenTransferMockFactory::gas_constrained(50000);
        
        // This transfer needs 65000 gas but we only have 50000 available
        let transfer = TokenTransfer::erc20_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        );
        
        let result = handler.execute(transfer).await;
        assert!(result.is_failure());
        
        if let EffectResult::Failure(error) = result {
            assert!(matches!(error, TransferError::GasLimitExceeded { .. }));
        }
    }
    
    #[test]
    fn test_execution_history() {
        let handler = TokenTransferMockFactory::always_succeed();
        
        assert_eq!(handler.get_execution_history().len(), 0);
        // History would be tested in async context
    }
    
    #[test]
    fn test_chain_state_access() {
        let handler = TokenTransferMockFactory::blockchain_realistic("ethereum");
        
        assert!(handler.get_chain_state().is_some());
        
        let simple_handler = TokenTransferMockFactory::always_succeed();
        assert!(simple_handler.get_chain_state().is_none());
    }
} 