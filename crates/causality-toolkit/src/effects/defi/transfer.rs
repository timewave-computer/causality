//! TokenTransfer effect implementation for asset transfer operations

use crate::effects::{AlgebraicEffect, EffectCategory, FailureMode};
use causality_core::system::content_addressing::{ContentAddressable, EntityId};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::hash::{Hash, Hasher};

/// Token transfer effect for moving assets between addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    /// Source address for the transfer
    pub from: String,
    
    /// Destination address for the transfer
    pub to: String,
    
    /// Amount to transfer (in smallest unit)
    pub amount: u64,
    
    /// Token address/identifier
    pub token: String,
    
    /// Optional gas limit for the transaction
    pub gas_limit: Option<u64>,
    
    /// Optional gas price (in wei or equivalent)
    pub gas_price: Option<u64>,
    
    /// Optional memo/data field
    pub memo: Option<String>,
    
    /// Required confirmations for finality
    pub confirmations: Option<u32>,
    
    /// Timeout for the transfer operation
    pub timeout: Option<Duration>,
}

impl TokenTransfer {
    /// Create a new token transfer
    pub fn new(from: String, to: String, amount: u64, token: String) -> Self {
        Self {
            from,
            to,
            amount,
            token,
            gas_limit: None,
            gas_price: None,
            memo: None,
            confirmations: None,
            timeout: None,
        }
    }
    
    /// Create a simple ETH transfer
    pub fn eth_transfer(from: String, to: String, amount: u64) -> Self {
        Self::new(from, to, amount, "ETH".to_string())
            .with_gas_limit(21000)
            .with_confirmations(12)
            .with_timeout(Duration::from_secs(300))
    }
    
    /// Create an ERC-20 token transfer
    pub fn erc20_transfer(from: String, to: String, amount: u64, token: String) -> Self {
        Self::new(from, to, amount, token)
            .with_gas_limit(65000)
            .with_confirmations(12)
            .with_timeout(Duration::from_secs(300))
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
    
    /// Add memo to transfer
    pub fn with_memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }
    
    /// Set required confirmations
    pub fn with_confirmations(mut self, confirmations: u32) -> Self {
        self.confirmations = Some(confirmations);
        self
    }
    
    /// Set operation timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Validate transfer parameters
    pub fn validate(&self) -> Result<(), TransferError> {
        // Validate addresses
        if self.from.is_empty() {
            return Err(TransferError::InvalidAddress("from address is empty".to_string()));
        }
        
        if self.to.is_empty() {
            return Err(TransferError::InvalidAddress("to address is empty".to_string()));
        }
        
        if self.from == self.to {
            return Err(TransferError::InvalidAddress("cannot transfer to self".to_string()));
        }
        
        // Validate amount
        if self.amount == 0 {
            return Err(TransferError::InvalidAmount("amount cannot be zero".to_string()));
        }
        
        // Validate token
        if self.token.is_empty() {
            return Err(TransferError::InvalidToken("token identifier is empty".to_string()));
        }
        
        // Validate gas parameters
        if let Some(gas_limit) = self.gas_limit {
            if gas_limit == 0 {
                return Err(TransferError::InvalidGasParameters("gas limit cannot be zero".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Calculate estimated gas cost
    pub fn estimated_gas_cost(&self) -> u64 {
        let base_gas = if self.token == "ETH" { 21000 } else { 65000 };
        
        // Add gas for memo if present
        let memo_gas = self.memo.as_ref()
            .map(|m| m.len() as u64 * 68) // Rough estimate: 68 gas per byte
            .unwrap_or(0);
        
        base_gas + memo_gas
    }
    
    /// Estimate total transaction cost (gas * price)
    pub fn estimated_transaction_cost(&self) -> Option<u64> {
        self.gas_price.map(|price| self.estimated_gas_cost() * price)
    }
}

/// Receipt returned upon successful transfer completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferReceipt {
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Block number where transaction was included
    pub block_number: u64,
    
    /// Transaction index within the block
    pub transaction_index: u32,
    
    /// Gas actually used
    pub gas_used: u64,
    
    /// Effective gas price
    pub gas_price: u64,
    
    /// Number of confirmations received
    pub confirmations: u32,
    
    /// Timestamp when transaction was mined
    pub timestamp: u64,
    
    /// Status: "success" or "failed"
    pub status: String,
    
    /// Optional event logs from the transaction
    pub logs: Vec<TransferLog>,
    
    /// Final balances after transfer
    pub final_balances: TransferBalances,
}

/// Event log from transfer transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferLog {
    /// Address that emitted the log
    pub address: String,
    
    /// Log topics (indexed event parameters)
    pub topics: Vec<String>,
    
    /// Log data (non-indexed event parameters)
    pub data: String,
    
    /// Block number
    pub block_number: u64,
    
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Log index within transaction
    pub log_index: u32,
}

/// Balance information after transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferBalances {
    /// Sender's balance after transfer
    pub sender_balance: u64,
    
    /// Receiver's balance after transfer
    pub receiver_balance: u64,
    
    /// Token used for the transfer
    pub token: String,
}

/// Comprehensive error types for token transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferError {
    /// Insufficient balance in source account
    InsufficientBalance {
        available: u64,
        required: u64,
        token: String,
    },
    
    /// Invalid address format or address not found
    InvalidAddress(String),
    
    /// Invalid transfer amount
    InvalidAmount(String),
    
    /// Token not found or invalid token identifier
    InvalidToken(String),
    
    /// Network-related errors
    NetworkError {
        reason: String,
        is_transient: bool,
    },
    
    /// Gas limit exceeded during execution
    GasLimitExceeded {
        gas_limit: u64,
        gas_required: u64,
    },
    
    /// Insufficient gas price
    InsufficientGasPrice {
        provided: u64,
        minimum: u64,
    },
    
    /// Invalid gas parameters
    InvalidGasParameters(String),
    
    /// Insufficient allowance for ERC-20 transfer
    InsufficientAllowance {
        allowance: u64,
        required: u64,
    },
    
    /// Transaction timeout
    Timeout {
        timeout_duration: Duration,
    },
    
    /// Transaction was cancelled
    Cancelled,
    
    /// Transaction failed in execution
    TransactionFailed {
        reason: String,
        transaction_hash: Option<String>,
    },
    
    /// Blockchain/consensus related errors
    BlockchainError {
        error_type: BlockchainErrorType,
        details: String,
    },
    
    /// Custom error with description
    Custom(String),
}

/// Types of blockchain-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockchainErrorType {
    /// Node is not synced
    NodeNotSynced,
    /// Mempool is full
    MempoolFull,
    /// Fork detected
    ForkDetected,
    /// Block reorganization occurred
    Reorganization,
    /// Consensus failure
    ConsensusFailure,
    /// RPC error
    RpcError,
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::InsufficientBalance { available, required, token } => {
                write!(f, "Insufficient {} balance: have {}, need {}", token, available, required)
            }
            TransferError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            TransferError::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            TransferError::InvalidToken(token) => write!(f, "Invalid token: {}", token),
            TransferError::NetworkError { reason, is_transient } => {
                write!(f, "Network error ({}): {}", if *is_transient { "transient" } else { "permanent" }, reason)
            }
            TransferError::GasLimitExceeded { gas_limit, gas_required } => {
                write!(f, "Gas limit exceeded: limit {}, required {}", gas_limit, gas_required)
            }
            TransferError::InsufficientGasPrice { provided, minimum } => {
                write!(f, "Insufficient gas price: provided {}, minimum {}", provided, minimum)
            }
            TransferError::InvalidGasParameters(msg) => write!(f, "Invalid gas parameters: {}", msg),
            TransferError::InsufficientAllowance { allowance, required } => {
                write!(f, "Insufficient allowance: have {}, need {}", allowance, required)
            }
            TransferError::Timeout { timeout_duration } => {
                write!(f, "Transfer timeout after {:?}", timeout_duration)
            }
            TransferError::Cancelled => write!(f, "Transfer was cancelled"),
            TransferError::TransactionFailed { reason, transaction_hash } => {
                if let Some(hash) = transaction_hash {
                    write!(f, "Transaction {} failed: {}", hash, reason)
                } else {
                    write!(f, "Transaction failed: {}", reason)
                }
            }
            TransferError::BlockchainError { error_type, details } => {
                write!(f, "Blockchain error ({:?}): {}", error_type, details)
            }
            TransferError::Custom(msg) => write!(f, "Custom error: {}", msg),
        }
    }
}

impl std::error::Error for TransferError {}

impl ContentAddressable for TokenTransfer {
    fn content_id(&self) -> EntityId {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.from.hash(&mut hasher);
        self.to.hash(&mut hasher);
        self.amount.hash(&mut hasher);
        self.token.hash(&mut hasher);
        
        if let Some(gas_limit) = self.gas_limit {
            gas_limit.hash(&mut hasher);
        }
        
        if let Some(gas_price) = self.gas_price {
            gas_price.hash(&mut hasher);
        }
        
        if let Some(ref memo) = self.memo {
            memo.hash(&mut hasher);
        }
        
        let hash = hasher.finish();
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&hash.to_le_bytes());
        
        EntityId::from_bytes(bytes)
    }
}

impl AlgebraicEffect for TokenTransfer {
    type Result = TransferReceipt;
    type Error = TransferError;
    
    fn effect_name() -> &'static str {
        "token_transfer"
    }
    
    fn effect_category() -> EffectCategory {
        EffectCategory::Asset
    }
    
    fn expected_duration() -> Duration {
        Duration::from_secs(15) // Average block time for most chains
    }
    
    fn failure_modes() -> Vec<FailureMode> {
        vec![
            FailureMode::InsufficientBalance,
            FailureMode::InvalidAddress,
            FailureMode::NetworkError,
            FailureMode::GasLimitExceeded,
            FailureMode::Timeout,
            FailureMode::InsufficientAllowance,
            FailureMode::Custom("transaction_failed".to_string()),
            FailureMode::Custom("mempool_full".to_string()),
            FailureMode::Custom("nonce_too_low".to_string()),
            FailureMode::Custom("replacement_underpriced".to_string()),
        ]
    }
    
    fn is_parallelizable() -> bool {
        false // Token transfers from same account must be sequential due to nonce
    }
    
    fn has_side_effects() -> bool {
        true // Transfers modify blockchain state
    }
    
    fn computational_cost() -> u32 {
        3 // Medium computational cost (signature verification, state updates)
    }
    
    fn gas_cost() -> u64 {
        65000 // Conservative estimate for ERC-20 transfer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_transfer_creation() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        );
        
        assert_eq!(transfer.from, "0x1234");
        assert_eq!(transfer.to, "0x5678");
        assert_eq!(transfer.amount, 1000);
        assert_eq!(transfer.token, "USDC");
    }
    
    #[test]
    fn test_eth_transfer() {
        let transfer = TokenTransfer::eth_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        );
        
        assert_eq!(transfer.token, "ETH");
        assert_eq!(transfer.gas_limit, Some(21000));
        assert_eq!(transfer.confirmations, Some(12));
        assert!(transfer.timeout.is_some());
    }
    
    #[test]
    fn test_erc20_transfer() {
        let transfer = TokenTransfer::erc20_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "0xA0b86a33E6441b5a033de1C3A95cfDBa59A5eb78".to_string()
        );
        
        assert_eq!(transfer.gas_limit, Some(65000));
        assert_eq!(transfer.confirmations, Some(12));
    }
    
    #[test]
    fn test_validation_success() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        );
        
        assert!(transfer.validate().is_ok());
    }
    
    #[test]
    fn test_validation_empty_from() {
        let transfer = TokenTransfer::new(
            "".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        );
        
        assert!(matches!(transfer.validate(), Err(TransferError::InvalidAddress(_))));
    }
    
    #[test]
    fn test_validation_empty_to() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "".to_string(),
            1000,
            "USDC".to_string()
        );
        
        assert!(matches!(transfer.validate(), Err(TransferError::InvalidAddress(_))));
    }
    
    #[test]
    fn test_validation_self_transfer() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x1234".to_string(),
            1000,
            "USDC".to_string()
        );
        
        assert!(matches!(transfer.validate(), Err(TransferError::InvalidAddress(_))));
    }
    
    #[test]
    fn test_validation_zero_amount() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            0,
            "USDC".to_string()
        );
        
        assert!(matches!(transfer.validate(), Err(TransferError::InvalidAmount(_))));
    }
    
    #[test]
    fn test_gas_cost_estimation() {
        let eth_transfer = TokenTransfer::eth_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        );
        
        assert_eq!(eth_transfer.estimated_gas_cost(), 21000);
        
        let erc20_transfer = TokenTransfer::erc20_transfer(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "0xA0b86a33E6441b5a033de1C3A95cfDBa59A5eb78".to_string()
        );
        
        assert_eq!(erc20_transfer.estimated_gas_cost(), 65000);
    }
    
    #[test]
    fn test_gas_cost_with_memo() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        ).with_memo("test memo".to_string());
        
        let expected_gas = 65000 + ("test memo".len() as u64 * 68);
        assert_eq!(transfer.estimated_gas_cost(), expected_gas);
    }
    
    #[test]
    fn test_transaction_cost_estimation() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        ).with_gas_price(20_000_000_000); // 20 gwei
        
        let expected_cost = transfer.estimated_gas_cost() * 20_000_000_000;
        assert_eq!(transfer.estimated_transaction_cost(), Some(expected_cost));
    }
    
    #[test]
    fn test_content_addressing() {
        let transfer1 = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        );
        
        let transfer2 = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        );
        
        let transfer3 = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            2000, // Different amount
            "USDC".to_string()
        );
        
        // Same transfers should have same content ID
        assert_eq!(transfer1.content_id(), transfer2.content_id());
        
        // Different transfers should have different content ID
        assert_ne!(transfer1.content_id(), transfer3.content_id());
    }
    
    #[test]
    fn test_algebraic_effect_implementation() {
        assert_eq!(TokenTransfer::effect_name(), "token_transfer");
        assert_eq!(TokenTransfer::effect_category(), EffectCategory::Asset);
        assert_eq!(TokenTransfer::expected_duration(), Duration::from_secs(15));
        assert!(!TokenTransfer::is_parallelizable());
        assert!(TokenTransfer::has_side_effects());
        assert_eq!(TokenTransfer::computational_cost(), 3);
        assert_eq!(TokenTransfer::gas_cost(), 65000);
        
        let failure_modes = TokenTransfer::failure_modes();
        assert!(failure_modes.contains(&FailureMode::InsufficientBalance));
        assert!(failure_modes.contains(&FailureMode::InvalidAddress));
        assert!(failure_modes.contains(&FailureMode::NetworkError));
        assert!(failure_modes.contains(&FailureMode::GasLimitExceeded));
    }
    
    #[test]
    fn test_transfer_error_display() {
        let error = TransferError::InsufficientBalance {
            available: 500,
            required: 1000,
            token: "ETH".to_string(),
        };
        
        assert_eq!(error.to_string(), "Insufficient ETH balance: have 500, need 1000");
        
        let error = TransferError::InvalidAddress("invalid format".to_string());
        assert_eq!(error.to_string(), "Invalid address: invalid format");
        
        let error = TransferError::NetworkError {
            reason: "connection failed".to_string(),
            is_transient: true,
        };
        assert_eq!(error.to_string(), "Network error (transient): connection failed");
    }
    
    #[test]
    fn test_fluent_interface() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "USDC".to_string()
        )
        .with_gas_limit(100000)
        .with_gas_price(25_000_000_000)
        .with_memo("Payment for services".to_string())
        .with_confirmations(6)
        .with_timeout(Duration::from_secs(600));
        
        assert_eq!(transfer.gas_limit, Some(100000));
        assert_eq!(transfer.gas_price, Some(25_000_000_000));
        assert_eq!(transfer.memo, Some("Payment for services".to_string()));
        assert_eq!(transfer.confirmations, Some(6));
        assert_eq!(transfer.timeout, Some(Duration::from_secs(600)));
    }
} 