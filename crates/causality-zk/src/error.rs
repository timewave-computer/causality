//! Error types for ZK operations

use thiserror::Error;

/// Main ZK error type encompassing all ZK-related errors
#[derive(Error, Debug)]
pub enum ZkError {
    #[error("Circuit error: {0}")]
    Circuit(#[from] CircuitError),
    
    #[error("Proof error: {0}")]
    Proof(#[from] ProofError),
    
    #[error("Verification error: {0}")]
    Verification(#[from] VerificationError),
    
    #[error("Witness error: {0}")]
    Witness(#[from] WitnessError),
    
    #[error("Backend error: {0}")]
    Backend(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Circuit compilation and validation errors
#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("Invalid instruction sequence: {0}")]
    InvalidInstructions(String),
    
    #[error("Constraint generation failed: {0}")]
    ConstraintGeneration(String),
    
    #[error("Unsupported instruction: {0}")]
    UnsupportedInstruction(String),
    
    #[error("Circuit optimization failed: {0}")]
    OptimizationFailed(String),
    
    #[error("Circuit validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Missing public input: {0}")]
    MissingPublicInput(String),
    
    #[error("Invalid witness schema: {0}")]
    InvalidWitnessSchema(String),
}

/// Proof generation errors
#[derive(Error, Debug)]
pub enum ProofError {
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
    
    #[error("Proof generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
    
    #[error("Invalid witness: {0}")]
    InvalidWitness(String),
    
    #[error("Circuit mismatch: expected {expected}, got {actual}")]
    CircuitMismatch { expected: String, actual: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Timeout during proof generation")]
    Timeout,
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
}

/// Proof verification errors
#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Invalid proof format: {0}")]
    InvalidProofFormat(String),
    
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    
    #[error("Public input mismatch: {0}")]
    PublicInputMismatch(String),
    
    #[error("Proof is malformed: {0}")]
    MalformedProof(String),
    
    #[error("Backend verification error: {0}")]
    BackendError(String),
}

/// Witness validation errors
#[derive(Error, Debug)]
pub enum WitnessError {
    #[error("Witness validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Invalid witness format: {0}")]
    InvalidFormat(String),
    
    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Result type for ZK operations
pub type ZkResult<T> = Result<T, ZkError>;

/// Result type for circuit operations
pub type CircuitResult<T> = Result<T, CircuitError>;

/// Result type for proof operations
pub type ProofResult<T> = Result<T, ProofError>;

/// Result type for verification operations
pub type VerificationResult<T> = Result<T, VerificationError>;

/// Result type for witness operations
pub type WitnessResult<T> = Result<T, WitnessError>; 