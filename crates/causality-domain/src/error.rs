// Error types for domain system
//
// This module defines error types specifically for the domain system
// and implements the CausalityError trait for these types.

use thiserror::Error;
use causality_error::{CausalityError, ErrorCode, ErrorDomain, BoxError};

/// Error codes for domain-related errors
pub const DOMAIN_NOT_FOUND: ErrorCode = ErrorCode(7001);
pub const UNSUPPORTED_OPERATION: ErrorCode = ErrorCode(7002);
pub const CONNECTION_ERROR: ErrorCode = ErrorCode(7003);
pub const INVALID_ARGUMENT: ErrorCode = ErrorCode(7004);
pub const TRANSACTION_ERROR: ErrorCode = ErrorCode(7005);
pub const TIME_MAP_ERROR: ErrorCode = ErrorCode(7006);
pub const SYNC_MANAGER_ERROR: ErrorCode = ErrorCode(7007);
pub const LOCK_ERROR: ErrorCode = ErrorCode(7008);
pub const ADAPTER_ERROR: ErrorCode = ErrorCode(7009);
pub const FACT_ERROR: ErrorCode = ErrorCode(7010);
pub const OBSERVER_ERROR: ErrorCode = ErrorCode(7011);
pub const VERIFICATION_ERROR: ErrorCode = ErrorCode(7012);
pub const SYSTEM_ERROR: ErrorCode = ErrorCode(7013);
pub const TIMEOUT_ERROR: ErrorCode = ErrorCode(7014);
pub const INVALID_FACT: ErrorCode = ErrorCode(7015);
pub const FACT_VERIFICATION: ErrorCode = ErrorCode(7016);
pub const ACCESS_DENIED: ErrorCode = ErrorCode(7017);
pub const DOMAIN_DATA_ERROR: ErrorCode = ErrorCode(7018);
pub const DOMAIN_ADAPTER_NOT_FOUND: ErrorCode = ErrorCode(7019);
pub const UNSUPPORTED_DOMAIN_TYPE: ErrorCode = ErrorCode(7020);

/// Result type alias using the causality error system
pub type Result<T> = causality_error::Result<T>;

/// Error type that encapsulates all domain-related errors
#[derive(Error, Debug)]
pub enum Error {
    /// Domain not found error
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    /// Unsupported operation error
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Invalid argument error
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Time map error
    #[error("Time map error: {0}")]
    TimeMapError(String),

    /// Sync manager error
    #[error("Sync manager error: {0}")]
    SyncManagerError(String),

    /// Lock acquisition error
    #[error("Lock error: {0}")]
    LockError(String),

    /// Domain adapter error
    #[error("Domain adapter error: {0}")]
    DomainAdapter(String),

    /// Fact error
    #[error("Fact error: {0}")]
    FactError(String),

    /// Invalid fact
    #[error("Invalid fact: {0}")]
    InvalidFact(String),

    /// Fact verification error
    #[error("Fact verification error: {0}")]
    FactVerification(String),

    /// Observer error
    #[error("Observer error: {0}")]
    ObserverError(String),

    /// Verification error
    #[error("Verification error: {0}")]
    VerificationError(String),

    /// System error
    #[error("System error: {0}")]
    SystemError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    /// Access denied error
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    /// Domain data error
    #[error("Domain data error: {0}")]
    DomainDataError(String),
    
    /// Domain adapter not found
    #[error("Domain adapter not found: {0}")]
    DomainAdapterNotFound(String),
    
    /// Unsupported domain type
    #[error("Unsupported domain type: {0}")]
    UnsupportedDomainType(String),
    
    /// Domain-specific error
    #[error("Domain error: {0}")]
    DomainError(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

impl CausalityError for Error {
    fn code(&self) -> ErrorCode {
        match self {
            Error::DomainNotFound(_) => DOMAIN_NOT_FOUND,
            Error::UnsupportedOperation(_) => UNSUPPORTED_OPERATION,
            Error::ConnectionError(_) => CONNECTION_ERROR,
            Error::InvalidArgument(_) => INVALID_ARGUMENT,
            Error::TransactionError(_) => TRANSACTION_ERROR,
            Error::TimeMapError(_) => TIME_MAP_ERROR,
            Error::SyncManagerError(_) => SYNC_MANAGER_ERROR,
            Error::LockError(_) => LOCK_ERROR,
            Error::DomainAdapter(_) => ADAPTER_ERROR,
            Error::FactError(_) => FACT_ERROR,
            Error::InvalidFact(_) => INVALID_FACT,
            Error::FactVerification(_) => FACT_VERIFICATION,
            Error::ObserverError(_) => OBSERVER_ERROR,
            Error::VerificationError(_) => VERIFICATION_ERROR,
            Error::SystemError(_) => SYSTEM_ERROR,
            Error::TimeoutError(_) => TIMEOUT_ERROR,
            Error::AccessDenied(_) => ACCESS_DENIED,
            Error::DomainDataError(_) => DOMAIN_DATA_ERROR,
            Error::DomainAdapterNotFound(_) => DOMAIN_ADAPTER_NOT_FOUND,
            Error::UnsupportedDomainType(_) => UNSUPPORTED_DOMAIN_TYPE,
            Error::DomainError(_) => DOMAIN_NOT_FOUND, // Using existing code
            Error::Other(_) => ErrorCode(7999),
        }
    }

    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Domain
    }
}

/// From implementation to convert DomainAdapterError to Error
impl From<crate::adapter::DomainAdapterError> for Error {
    fn from(error: crate::adapter::DomainAdapterError) -> Self {
        match error {
            crate::adapter::DomainAdapterError::DomainNotFound(msg) => Error::DomainNotFound(msg),
            crate::adapter::DomainAdapterError::UnsupportedOperation(msg) => Error::UnsupportedOperation(msg),
            crate::adapter::DomainAdapterError::ConnectionError(msg) => Error::ConnectionError(msg),
            crate::adapter::DomainAdapterError::InvalidArgument(msg) => Error::InvalidArgument(msg),
            crate::adapter::DomainAdapterError::TransactionError(msg) => Error::TransactionError(msg),
            crate::adapter::DomainAdapterError::Other(msg) => Error::DomainAdapter(msg),
        }
    }
}

/// Domain-specific error types
#[derive(Error, Debug)]
pub enum DomainError {
    /// Domain not found error
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    /// Unsupported operation error
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Invalid argument error
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Time map error
    #[error("Time map error: {0}")]
    TimeMapError(String),

    /// Sync manager error
    #[error("Sync manager error: {0}")]
    SyncManagerError(String),

    /// Lock acquisition error
    #[error("Lock error: {0}")]
    LockError(String),

    /// Adapter error
    #[error("Domain adapter error: {0}")]
    AdapterError(String),

    /// Fact observation error
    #[error("Fact error: {0}")]
    FactError(String),

    /// Observer error
    #[error("Observer error: {0}")]
    ObserverError(String),

    /// Verification error
    #[error("Verification error: {0}")]
    VerificationError(String),

    /// System error
    #[error("System error: {0}")]
    SystemError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Error from another domain
    #[error("Error from domain {domain}: {message}")]
    DomainSpecificError {
        domain: String,
        message: String,
        code: u32,
    },

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

impl CausalityError for DomainError {
    fn code(&self) -> ErrorCode {
        match self {
            DomainError::DomainNotFound(_) => DOMAIN_NOT_FOUND,
            DomainError::UnsupportedOperation(_) => UNSUPPORTED_OPERATION,
            DomainError::ConnectionError(_) => CONNECTION_ERROR,
            DomainError::InvalidArgument(_) => INVALID_ARGUMENT,
            DomainError::TransactionError(_) => TRANSACTION_ERROR,
            DomainError::TimeMapError(_) => TIME_MAP_ERROR,
            DomainError::SyncManagerError(_) => SYNC_MANAGER_ERROR,
            DomainError::LockError(_) => LOCK_ERROR,
            DomainError::AdapterError(_) => ADAPTER_ERROR,
            DomainError::FactError(_) => FACT_ERROR,
            DomainError::ObserverError(_) => OBSERVER_ERROR,
            DomainError::VerificationError(_) => VERIFICATION_ERROR,
            DomainError::SystemError(_) => SYSTEM_ERROR,
            DomainError::TimeoutError(_) => TIMEOUT_ERROR,
            DomainError::DomainSpecificError { code, .. } => ErrorCode(*code),
            DomainError::Other(_) => ErrorCode(7999),
        }
    }

    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Domain
    }
}

/// Adapter error that implements the CausalityError trait
#[derive(Error, Debug)]
pub enum AdapterError {
    /// The requested domain was not found
    #[error("Domain not found: {0}")]
    DomainNotFound(String),
    
    /// The requested operation is not supported
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// An error occurred while connecting to the domain
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Invalid argument provided
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    /// A transaction error occurred
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// Any other error
    #[error("Error: {0}")]
    Other(String),
}

impl CausalityError for AdapterError {
    fn code(&self) -> ErrorCode {
        match self {
            AdapterError::DomainNotFound(_) => DOMAIN_NOT_FOUND,
            AdapterError::UnsupportedOperation(_) => UNSUPPORTED_OPERATION,
            AdapterError::ConnectionError(_) => CONNECTION_ERROR,
            AdapterError::InvalidArgument(_) => INVALID_ARGUMENT,
            AdapterError::TransactionError(_) => TRANSACTION_ERROR,
            AdapterError::Other(_) => ADAPTER_ERROR,
        }
    }

    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Domain
    }
}

/// Convert the legacy domain adapter error to the new adapter error
impl From<crate::adapter::DomainAdapterError> for AdapterError {
    fn from(error: crate::adapter::DomainAdapterError) -> Self {
        match error {
            crate::adapter::DomainAdapterError::DomainNotFound(msg) => AdapterError::DomainNotFound(msg),
            crate::adapter::DomainAdapterError::UnsupportedOperation(msg) => AdapterError::UnsupportedOperation(msg),
            crate::adapter::DomainAdapterError::ConnectionError(msg) => AdapterError::ConnectionError(msg),
            crate::adapter::DomainAdapterError::InvalidArgument(msg) => AdapterError::InvalidArgument(msg),
            crate::adapter::DomainAdapterError::TransactionError(msg) => AdapterError::TransactionError(msg),
            crate::adapter::DomainAdapterError::Other(msg) => AdapterError::Other(msg),
        }
    }
}

/// Factory function to create a domain not found error
pub fn domain_not_found(domain_id: impl ToString) -> BoxError {
    Box::new(DomainError::DomainNotFound(domain_id.to_string()))
}

/// Factory function to create an unsupported operation error
pub fn unsupported_operation(msg: impl ToString) -> BoxError {
    Box::new(DomainError::UnsupportedOperation(msg.to_string()))
}

/// Factory function to create a connection error
pub fn connection_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::ConnectionError(msg.to_string()))
}

/// Factory function to create an invalid argument error
pub fn invalid_argument(msg: impl ToString) -> BoxError {
    Box::new(DomainError::InvalidArgument(msg.to_string()))
}

/// Factory function to create a transaction error
pub fn transaction_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::TransactionError(msg.to_string()))
}

/// Factory function to create a time map error
pub fn time_map_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::TimeMapError(msg.to_string()))
}

/// Factory function to create a sync manager error
pub fn sync_manager_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::SyncManagerError(msg.to_string()))
}

/// Factory function to create a lock error
pub fn lock_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::LockError(msg.to_string()))
}

/// Factory function to create an adapter error
pub fn adapter_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::AdapterError(msg.to_string()))
}

/// Factory function to create a system error
pub fn system_error(msg: impl ToString) -> BoxError {
    Box::new(DomainError::SystemError(msg.to_string()))
}

// Implementation to convert Box<Error> to Box<dyn CausalityError>
impl From<Box<Error>> for Box<dyn causality_error::CausalityError> {
    fn from(error: Box<Error>) -> Self {
        error as Box<dyn causality_error::CausalityError>
    }
}

// Implementation to convert Error to Box<dyn CausalityError>
impl From<Error> for Box<dyn causality_error::CausalityError> {
    fn from(error: Error) -> Self {
        Box::new(error) as Box<dyn causality_error::CausalityError>
    }
}