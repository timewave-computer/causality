// Error types for the Causality system

use std::fmt;
use thiserror::Error;

use crate::types::{ResourceId, DomainId, Asset, Amount};
use crate::resource::RegisterId;

/// The main error type for the Causality system
#[derive(Error, Debug, Clone)]
pub enum Error {
    // Effect system errors
    #[error("Invalid effect: {0}")]
    InvalidEffect(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Handler error: {0}")]
    HandlerError(String),

    #[error("Continuation error: {0}")]
    ContinuationError(String),

    #[error("Unhandled effect: {0}")]
    UnhandledEffect(String),

    #[error("Not initialized: {0}")]
    NotInitialized(String),

    #[error("Missing amount")]
    MissingAmount,

    #[error("Missing timestamp")]
    MissingTimestamp,

    #[error("Missing inner effects")]
    MissingInnerEffects,

    // Resource errors
    #[error("Resource not found: {0}")]
    ResourceNotFound(ResourceId),

    #[error("Resource already locked: {0}")]
    ResourceAlreadyLocked(ResourceId),

    #[error("Resource deadlock detected")]
    ResourceDeadlock,

    #[error("Resource timeout: {0}")]
    ResourceTimeout(ResourceId),

    // AST and Resource Graph errors
    #[error("AST node not found: {0}")]
    AstNodeNotFound(String),

    #[error("Resource imbalance: {0}")]
    ResourceImbalance(String),

    #[error("Resource attribution failed: {0}")]
    ResourceAttributionFailed(String),

    #[error("Controller transition error: {0}")]
    ControllerTransitionError(String),

    #[error("Divergence analysis error: {0}")]
    DivergenceAnalysisError(String),

    // Domain errors
    #[error("Domain not found: {0}")]
    DomainNotFound(DomainId),

    #[error("Domain connection error: {0}")]
    DomainConnectionError(String),

    #[error("Sync manager error: {0}")]
    SyncManagerError(String),

    #[error("Domain API error: {0}")]
    DomainApiError(String),

    #[error("Domain data error: {0}")]
    DomainDataError(String),

    #[error("Unsupported fact type: {0}")]
    UnsupportedFactType(String),

    #[error("Unsupported transaction type: {0}")]
    UnsupportedTransactionType(String),

    #[error("Insufficient balance: {domain} {asset} {required} (available: {available})")]
    InsufficientBalance {
        domain: DomainId,
        asset: String,
        required: String,
        available: String,
    },

    #[error("Transaction error: {0}")]
    TransactionError(String),

    // RISC-V compilation errors
    #[error("RISC-V compilation error: {0}")]
    RiscVCompilationError(String),

    #[error("RISC-V execution error: {0}")]
    RiscVExecutionError(String),

    #[error("ZK proof generation error: {0}")]
    ZkProofGenerationError(String),

    #[error("ZK proof verification error: {0}")]
    ZkProofVerificationError(String),

    // RISC-V VM errors
    #[error("Invalid register: {0}")]
    InvalidRegister(usize),

    #[error("Invalid memory access: {0}")]
    InvalidMemoryAccess(String),

    #[error("Memory map error: {0}")]
    MemoryMapError(String),

    #[error("Out of memory: {0}")]
    OutOfMemory(String),
    
    #[error("No program loaded")]
    NoProgramLoaded,

    // Content-addressed code errors
    #[error("Code repository error: {0}")]
    CodeRepositoryError(String),

    #[error("Code not found: {0}")]
    CodeNotFound(String),

    #[error("Code incompatible: {0}")]
    CodeIncompatible(String),
    
    // Note: These errors are available in all builds for consistency,
    // even when the code-repo feature is disabled

    // Log errors
    #[error("Log error: {0}")]
    LogError(String),

    #[error("Replay error: {0}")]
    ReplayError(String),

    // Register errors
    #[error("Register not found: {0}")]
    RegisterNotFound(RegisterId),

    #[error("Register access denied: {0}")]
    RegisterAccessDenied(RegisterId),

    #[error("Register operation failed: {0}")]
    RegisterOperationFailed(String),

    // Time map errors
    #[error("Time map error: {0}")]
    TimeMapError(String),

    #[error("Time map validation failed: {0}")]
    TimeMapValidationFailed(String),

    // Concurrency errors
    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    // IO and serialization errors
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    // Miscellaneous errors
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    // Feature flag errors
    #[error("Feature disabled: {0}")]
    FeatureDisabled(String),

    #[error("JSON error: {0}")]
    JsonError(String),

    #[error("Bincode error: {0}")]
    BincodeError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IO(String),
    
    /// Invalid JSON
    #[error("Invalid JSON: {0}")]
    Json(String),
    
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Resource already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    /// Invalid hash for content-addressed entry
    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    /// Invalid register operation
    #[error("Invalid register operation: {0}")]
    InvalidOperation(String),
    
    /// Register authorization failed
    #[error("Register authorization failed: {0}")]
    Unauthorized(String),
    
    // ZK-VM specific errors
    /// ZK-VM backend error
    #[error("ZK-VM backend error: {0}")]
    ZkVmBackendError(String),
    
    /// ZK-VM compilation error
    #[error("ZK-VM compilation error: {0}")]
    ZkVmCompilationError(String),
    
    /// ZK-VM execution error
    #[error("ZK-VM execution error: {0}")]
    ZkVmExecutionError(String),
    
    /// ZK-VM proof validation error
    #[error("ZK-VM validation error: {0}")]
    ValidationError(String),
    
    /// ZK-VM configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// ZK-VM service unavailable
    #[error("ZK-VM service unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// ZK-VM network error
    #[error("Network error: {0}")]
    NetworkError(String),
}

// Implement From traits for various error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::BincodeError(err.to_string())
    }
}

/// Result type for all operations
pub type Result<T> = std::result::Result<T, Error>; 