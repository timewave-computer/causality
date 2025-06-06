//! Token transfer effect example demonstrating complete AlgebraicEffect implementation

use crate::effects::{
    core::{AlgebraicEffect, EffectCategory, FailureMode},
    schema::{EffectSchema, ParameterDef, TypeDef, EffectMetadata},
};
use causality_core::system::content_addressing::{ContentAddressable, EntityId};
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Token transfer effect for moving tokens between addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    /// Source address (sender)
    pub from: String,
    /// Destination address (recipient)  
    pub to: String,
    /// Amount to transfer (in smallest unit, e.g., wei)
    pub amount: u64,
    /// Token contract address (for ERC-20 tokens)
    pub token_contract: Option<String>,
    /// Optional transaction memo
    pub memo: Option<String>,
    /// Maximum gas price willing to pay
    pub max_gas_price: Option<u64>,
}

/// Result of a successful token transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferReceipt {
    /// Transaction hash
    pub tx_hash: String,
    /// Block number where transaction was included
    pub block_number: u64,
    /// Gas used for the transaction
    pub gas_used: u64,
    /// Effective gas price paid
    pub gas_price: u64,
    /// Updated balance of sender after transfer
    pub sender_balance: u64,
    /// Updated balance of recipient after transfer
    pub recipient_balance: u64,
}

/// Errors that can occur during token transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferError {
    /// Insufficient balance to complete transfer
    InsufficientBalance { available: u64, requested: u64 },
    /// Invalid address format
    InvalidAddress(String),
    /// Token contract not found or invalid
    InvalidTokenContract(String),
    /// Gas limit exceeded
    GasLimitExceeded { limit: u64, required: u64 },
    /// Network error during submission
    NetworkError(String),
    /// Transaction failed on-chain
    TransactionFailed { reason: String, tx_hash: Option<String> },
    /// Transfer amount is zero or negative
    InvalidAmount,
    /// Sender and recipient are the same address
    SelfTransfer,
}

impl ContentAddressable for TokenTransfer {
    fn content_id(&self) -> EntityId {
        // Create content ID from transfer parameters
        let content = format!(
            "{}:{}:{}:{}:{}",
            self.from,
            self.to, 
            self.amount,
            self.token_contract.as_deref().unwrap_or("native"),
            self.memo.as_deref().unwrap_or("")
        );
        
        let mut bytes = [0u8; 32];
        let content_bytes = content.as_bytes();
        let copy_len = std::cmp::min(content_bytes.len(), 32);
        bytes[0..copy_len].copy_from_slice(&content_bytes[0..copy_len]);
        
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
        Duration::from_millis(2000) // ~2 seconds for blockchain confirmation
    }
    
    fn failure_modes() -> Vec<FailureMode> {
        vec![
            FailureMode::InsufficientBalance,
            FailureMode::InvalidAddress,
            FailureMode::GasLimitExceeded,
            FailureMode::NetworkError,
        ]
    }
    
    fn is_parallelizable() -> bool {
        false // Token transfers must be sequential to prevent double-spending
    }
    
    fn has_side_effects() -> bool {
        true // Transfers modify on-chain state
    }
    
    fn computational_cost() -> u32 {
        3 // Moderate cost due to cryptographic operations
    }
    
    fn gas_cost() -> u64 {
        21_000 // Standard Ethereum transfer gas cost
    }
}

impl TokenTransfer {
    /// Create a new token transfer
    pub fn new(from: String, to: String, amount: u64) -> Self {
        TokenTransfer {
            from,
            to,
            amount,
            token_contract: None,
            memo: None,
            max_gas_price: None,
        }
    }
    
    /// Create an ERC-20 token transfer
    pub fn erc20(from: String, to: String, amount: u64, token_contract: String) -> Self {
        TokenTransfer {
            from,
            to,
            amount,
            token_contract: Some(token_contract),
            memo: None,
            max_gas_price: None,
        }
    }
    
    /// Add a memo to the transfer
    pub fn with_memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }
    
    /// Set maximum gas price
    pub fn with_max_gas_price(mut self, max_gas_price: u64) -> Self {
        self.max_gas_price = Some(max_gas_price);
        self
    }
    
    /// Generate a detailed schema for this effect
    pub fn detailed_schema() -> EffectSchema {
        let metadata = EffectMetadata {
            category: Self::effect_category(),
            expected_duration: Self::expected_duration(),
            failure_modes: Self::failure_modes(),
            parallelizable: Self::is_parallelizable(),
            has_side_effects: Self::has_side_effects(),
            computational_cost: Self::computational_cost(),
            gas_cost: Self::gas_cost(),
        };
        
        EffectSchema::new(
            Self::effect_name().to_string(),
            vec![
                ParameterDef::new("from".to_string(), TypeDef::Address)
                    .with_description("Source address (sender)".to_string()),
                ParameterDef::new("to".to_string(), TypeDef::Address)
                    .with_description("Destination address (recipient)".to_string()),
                ParameterDef::new("amount".to_string(), TypeDef::UInt(64))
                    .with_description("Amount to transfer in smallest unit".to_string()),
                ParameterDef::new("token_contract".to_string(), TypeDef::Option(Box::new(TypeDef::Address)))
                    .optional()
                    .with_description("Token contract address for ERC-20 transfers".to_string()),
                ParameterDef::new("memo".to_string(), TypeDef::Option(Box::new(TypeDef::String)))
                    .optional()
                    .with_description("Optional transaction memo".to_string()),
                ParameterDef::new("max_gas_price".to_string(), TypeDef::Option(Box::new(TypeDef::UInt(64))))
                    .optional()
                    .with_description("Maximum gas price willing to pay".to_string()),
            ],
            TypeDef::Custom("TransferReceipt".to_string()),
            TypeDef::Custom("TransferError".to_string()),
            metadata,
        )
    }
    
    /// Validate transfer parameters
    pub fn validate(&self) -> Result<(), TransferError> {
        // Check for self-transfer
        if self.from == self.to {
            return Err(TransferError::SelfTransfer);
        }
        
        // Check for zero amount
        if self.amount == 0 {
            return Err(TransferError::InvalidAmount);
        }
        
        // Basic address validation (simplified)
        if self.from.is_empty() || self.to.is_empty() {
            return Err(TransferError::InvalidAddress("Empty address".to_string()));
        }
        
        // Validate token contract if provided
        if let Some(ref contract) = self.token_contract {
            if contract.is_empty() {
                return Err(TransferError::InvalidTokenContract("Empty contract address".to_string()));
            }
        }
        
        Ok(())
    }
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::InsufficientBalance { available, requested } => {
                write!(f, "Insufficient balance: available {}, requested {}", available, requested)
            }
            TransferError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            TransferError::InvalidTokenContract(contract) => write!(f, "Invalid token contract: {}", contract),
            TransferError::GasLimitExceeded { limit, required } => {
                write!(f, "Gas limit exceeded: limit {}, required {}", limit, required)
            }
            TransferError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TransferError::TransactionFailed { reason, tx_hash } => {
                if let Some(hash) = tx_hash {
                    write!(f, "Transaction failed: {} (tx: {})", reason, hash)
                } else {
                    write!(f, "Transaction failed: {}", reason)
                }
            }
            TransferError::InvalidAmount => write!(f, "Invalid transfer amount"),
            TransferError::SelfTransfer => write!(f, "Cannot transfer to same address"),
        }
    }
}

impl std::error::Error for TransferError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_transfer_creation() {
        let transfer = TokenTransfer::new(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000
        );
        
        assert_eq!(transfer.from, "0x1234");
        assert_eq!(transfer.to, "0x5678");
        assert_eq!(transfer.amount, 1000);
        assert!(transfer.token_contract.is_none());
        assert!(transfer.memo.is_none());
    }
    
    #[test]
    fn test_erc20_transfer() {
        let transfer = TokenTransfer::erc20(
            "0x1234".to_string(),
            "0x5678".to_string(),
            1000,
            "0xabcd".to_string()
        );
        
        assert_eq!(transfer.token_contract, Some("0xabcd".to_string()));
    }
    
    #[test]
    fn test_with_memo() {
        let transfer = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 1000)
            .with_memo("Payment for services".to_string());
        
        assert_eq!(transfer.memo, Some("Payment for services".to_string()));
    }
    
    #[test]
    fn test_validation() {
        // Valid transfer
        let valid_transfer = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 1000);
        assert!(valid_transfer.validate().is_ok());
        
        // Self-transfer
        let self_transfer = TokenTransfer::new("0x1234".to_string(), "0x1234".to_string(), 1000);
        assert!(matches!(self_transfer.validate(), Err(TransferError::SelfTransfer)));
        
        // Zero amount
        let zero_transfer = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 0);
        assert!(matches!(zero_transfer.validate(), Err(TransferError::InvalidAmount)));
        
        // Empty address
        let empty_addr = TokenTransfer::new("".to_string(), "0x5678".to_string(), 1000);
        assert!(matches!(empty_addr.validate(), Err(TransferError::InvalidAddress(_))));
    }
    
    #[test]
    fn test_algebraic_effect_trait() {
        assert_eq!(TokenTransfer::effect_name(), "token_transfer");
        assert_eq!(TokenTransfer::effect_category(), EffectCategory::Asset);
        assert_eq!(TokenTransfer::expected_duration(), Duration::from_millis(2000));
        assert!(!TokenTransfer::is_parallelizable());
        assert!(TokenTransfer::has_side_effects());
        assert_eq!(TokenTransfer::computational_cost(), 3);
        assert_eq!(TokenTransfer::gas_cost(), 21_000);
    }
    
    #[test]
    fn test_content_addressing() {
        let transfer1 = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 1000);
        let transfer2 = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 1000);
        let transfer3 = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 2000);
        
        // Same transfers should have same content ID
        assert_eq!(transfer1.content_id(), transfer2.content_id());
        
        // Different transfers should have different content ID
        assert_ne!(transfer1.content_id(), transfer3.content_id());
    }
    
    #[test]
    fn test_detailed_schema() {
        let schema = TokenTransfer::detailed_schema();
        
        assert_eq!(schema.name, "token_transfer");
        assert_eq!(schema.parameters.len(), 6);
        assert!(schema.has_parameter("from"));
        assert!(schema.has_parameter("to"));
        assert!(schema.has_parameter("amount"));
        assert!(schema.has_parameter("token_contract"));
        assert!(schema.has_parameter("memo"));
        assert!(schema.has_parameter("max_gas_price"));
        
        // Check required vs optional parameters
        assert_eq!(schema.required_parameters().len(), 3); // from, to, amount
        assert_eq!(schema.optional_parameters().len(), 3); // token_contract, memo, max_gas_price
    }
    
    #[test]
    fn test_error_display() {
        let error = TransferError::InsufficientBalance { available: 500, requested: 1000 };
        assert_eq!(error.to_string(), "Insufficient balance: available 500, requested 1000");
        
        let error = TransferError::InvalidAddress("0xinvalid".to_string());
        assert_eq!(error.to_string(), "Invalid address: 0xinvalid");
        
        let error = TransferError::SelfTransfer;
        assert_eq!(error.to_string(), "Cannot transfer to same address");
    }
    
    #[test]
    fn test_serialization() {
        let transfer = TokenTransfer::new("0x1234".to_string(), "0x5678".to_string(), 1000)
            .with_memo("Test transfer".to_string());
            
        let serialized = serde_json::to_string(&transfer).unwrap();
        let deserialized: TokenTransfer = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(transfer.from, deserialized.from);
        assert_eq!(transfer.to, deserialized.to);
        assert_eq!(transfer.amount, deserialized.amount);
        assert_eq!(transfer.memo, deserialized.memo);
    }
} 