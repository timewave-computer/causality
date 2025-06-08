//! Error types for the Causality simulation framework

use thiserror::Error;

/// Main error type for simulation operations
#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("Simulation configuration error: {0}")]
    Configuration(String),
    
    #[error("Engine state error: {0}")]
    EngineState(String),
    
    #[error("Effect execution error: {0}")]
    EffectExecutionError(String),
    
    #[error("Network simulation error: {0}")]
    NetworkError(String),
    
    #[error("Cross-chain simulation error: {0}")]
    CrossChainError(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Snapshot error: {0}")]
    SnapshotError(String),
    
    #[error("Resource unavailable: {resource_id}")]
    ResourceUnavailable { resource_id: String },
    
    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },
    
    #[error("Snapshot operation failed: {0}")]
    SnapshotOperationFailed(String),
    
    #[error("Fault injection error: {0}")]
    FaultInjectionError(String),
    
    #[error("TEG execution error: {0}")]
    TegExecutionError(String),
    
    #[error("Intent processing error: {0}")]
    IntentProcessingError(String),
    
    #[error("Visualization error: {0}")]
    VisualizationError(String),
    
    #[error("Engine operation error: {0}")]
    EngineError(String),
    
    #[error("Core causality error: {0}")]
    MachineError(#[from] causality_core::system::error::MachineError),
    
    /// Branch not found
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    
    #[error("Optimization error: {0}")]
    OptimizationError(String),
    
    #[error("Register error: {message} (register: {register_id})")]
    RegisterError { 
        message: String, 
        register_id: String 
    },
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Compilation error: {0}")]
    CompilationError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for simulation operations
pub type SimulationResult<T> = Result<T, SimulationError>;

/// Error type for fault injection operations
#[derive(Error, Debug)]
pub enum FaultError {
    #[error("Invalid fault target: {0}")]
    InvalidTarget(String),
    
    #[error("Fault injection failed: {0}")]
    InjectionFailed(String),
    
    #[error("Fault configuration error: {0}")]
    ConfigurationError(String),
}

/// Error type for snapshot operations
#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Snapshot not found: {id}")]
    NotFound { id: String },
    
    #[error("Snapshot creation failed: {0}")]
    CreationFailed(String),
    
    #[error("Snapshot restoration failed: {0}")]
    RestorationFailed(String),
    
    #[error("Invalid snapshot state: {0}")]
    InvalidState(String),
    
    /// Branch not found
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Compilation error
    #[error("Compilation error: {0}")]
    CompilationError(String),
} 