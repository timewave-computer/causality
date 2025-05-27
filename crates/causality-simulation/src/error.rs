//! Error types for Simulation
//!
//! This module defines the error types used throughout the simulation crate,
//! providing categorized error handling and proper context propagation.

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

use causality_runtime::tel::traits::HostCallError as TelExecutionError;
use thiserror::Error;

/// Main error type for the simulation crate.
#[derive(Error, Debug)]
pub enum SimulationError {
    /// Represents an error during simulation setup or configuration.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Represents an error during simulation runtime.
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Represents an error related to an invalid operation or state.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Represents an error related to snapshotting or history management.
    #[error("History/Snapshot error: {0}")]
    History(String),

    /// An IO error occurred (no longer direct std::io::Error to avoid From conflict).
    #[error("I/O error: {0}")]
    Io(String),

    /// A ssz (de)serialization error occurred.
    #[error("SSZ error: {0}")]
    Ssz(#[from] std::io::Error),

    /// An error passed up from another Causality crate.
    #[error(transparent)]
    Upstream(#[from] anyhow::Error),

    /// Represents a file I/O error.
    #[error("File I/O error: {0}")]
    FileIo(String),

    /// Represents a serialization/deserialization error.
    #[error("Serialization/Deserialization error: {0}")]
    Serialization(String),

    /// Represents an invalid state for an operation.
    #[error("Invalid state for operation: {0}")]
    State(String),

    /// Represents a mocking error.
    #[error("Mocking error: {0}")]
    Mocking(String),

    /// Represents a handler registration error.
    #[error("Effect handler registration error: {0}")]
    HandlerRegistration(String),

    /// Represents a TEL execution error.
    #[error("TEL execution error: {0}")]
    TelExecution(#[from] TelExecutionError),

    /// Represents a checkpoint/restoration error.
    #[error("Checkpoint error: {0}")]
    CheckpointError(String),

    /// Represents an evaluation error.
    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    /// Represents a configuration error.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Represents an unknown error.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for simulation operations.
pub type SimulationResult<T> = Result<T, SimulationError>;
