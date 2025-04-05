// Domain-specific error types
// These errors are specifically for the domain crates (causality-domain-*)

use thiserror::Error;
use crate::CausalityError;
use std::any::Any;

/// Domain-specific error codes
pub mod codes {
    use crate::ErrorCode;
    
    // Domain error codes start with 7000
    pub const ADAPTER_ERROR: ErrorCode = ErrorCode(7001);
    pub const CONTRACT_ERROR: ErrorCode = ErrorCode(7002);
    pub const TRANSACTION_ERROR: ErrorCode = ErrorCode(7003);
    pub const CHAIN_ERROR: ErrorCode = ErrorCode(7004);
    pub const PROTOCOL_ERROR: ErrorCode = ErrorCode(7005);
    pub const VERIFICATION_ERROR: ErrorCode = ErrorCode(7006);
    pub const BRIDGE_ERROR: ErrorCode = ErrorCode(7007);
}

/// Domain-specific error types
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    /// Domain adapter error
    #[error("Adapter error: {0}")]
    AdapterError(String),
    
    /// Smart contract error
    #[error("Contract error: {0}")]
    ContractError(String),
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// Blockchain error
    #[error("Chain error: {0}")]
    ChainError(String),
    
    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    /// Verification error
    #[error("Verification error: {0}")]
    VerificationError(String),
    
    /// Bridge error
    #[error("Bridge error: {0}")]
    BridgeError(String),
}

impl CausalityError for DomainError {
    fn error_code(&self) -> &'static str {
        match self {
            DomainError::AdapterError(_) => "DOMAIN_ADAPTER_ERROR",
            DomainError::ContractError(_) => "DOMAIN_CONTRACT_ERROR",
            DomainError::TransactionError(_) => "DOMAIN_TRANSACTION_ERROR",
            DomainError::ChainError(_) => "DOMAIN_CHAIN_ERROR",
            DomainError::ProtocolError(_) => "DOMAIN_PROTOCOL_ERROR",
            DomainError::VerificationError(_) => "DOMAIN_VERIFICATION_ERROR",
            DomainError::BridgeError(_) => "DOMAIN_BRIDGE_ERROR",
            // Add other variants if they exist, or a default arm
            // _ => "DOMAIN_UNKNOWN_ERROR",
        }
    }

    fn as_any(&self) -> &dyn Any { self }
}

/// Convenient Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Convert from domain error to boxed error
impl From<DomainError> for Box<dyn CausalityError> {
    fn from(err: DomainError) -> Self {
        Box::new(err)
    }
}

// Helper methods for creating domain errors
impl DomainError {
    /// Create a new adapter error
    pub fn adapter_error(message: impl Into<String>) -> Self {
        DomainError::AdapterError(message.into())
    }
    
    /// Create a new contract error
    pub fn contract_error(message: impl Into<String>) -> Self {
        DomainError::ContractError(message.into())
    }
    
    /// Create a new transaction error
    pub fn transaction_error(message: impl Into<String>) -> Self {
        DomainError::TransactionError(message.into())
    }
    
    /// Create a new chain error
    pub fn chain_error(message: impl Into<String>) -> Self {
        DomainError::ChainError(message.into())
    }
} 