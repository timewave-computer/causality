//! Core shared components for ZK proving
//!
//! This module contains minimal, no_std compatible code shared between
//! WASM and RISC-V targets.

use causality_types::serialization::{Encode, Decode, SimpleSerialize};

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
// Import Digest trait for Sha256::new()
use sha2::Digest;
use sha2::Sha256;

use causality_types::primitive::ids::ResourceId;

//-----------------------------------------------------------------------------
// Type Definition
//-----------------------------------------------------------------------------

/// Hash-based identifier for a witness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WitnessId(pub [u8; 32]);

impl SimpleSerialize for WitnessId {}

impl Encode for WitnessId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for WitnessId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() != 32 {
            return Err(causality_types::serialization::DecodeError::new("Invalid WitnessId length"));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(WitnessId(array))
    }
}

/// Hash-based identifier for a proof
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProofId(pub [u8; 32]);

impl SimpleSerialize for ProofId {}

impl Encode for ProofId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for ProofId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() != 32 {
            return Err(causality_types::serialization::DecodeError::new("Invalid ProofId length"));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(ProofId(array))
    }
}

/// Unique identifier for a ZK circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CircuitId(pub [u8; 32]);

impl SimpleSerialize for CircuitId {}

impl Encode for CircuitId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for CircuitId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() != 32 {
            return Err(causality_types::serialization::DecodeError::new("Invalid CircuitId length"));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(CircuitId(array))
    }
}

impl CircuitId {
    /// Create a new CircuitId by hashing data
    pub fn new(data: &[u8]) -> Self {
        {
            let mut hasher = Sha256::new();
            hasher.update(data);
            CircuitId(hasher.finalize().into())
        }
    }
}

impl std::fmt::Display for CircuitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl ProofId {
    /// Create a new ProofId by hashing data (e.g., proof bytes)
    pub fn new(data: &[u8]) -> Self {
        {
            let mut hasher = Sha256::new();
            hasher.update(data);
            ProofId(hasher.finalize().into())
        }
    }
}

//-----------------------------------------------------------------------------
// Error Types - Minimal set
//-----------------------------------------------------------------------------

/// Error type for ZK operations

#[derive(Debug, Clone)]
pub enum Error {
    /// Invalid input for proof generation
    InvalidInput(String),

    /// Circuit compilation error
    CircuitCompilation(String),

    /// Proof generation error
    ProofGeneration(String),

    /// Proof verification error
    ProofVerification(String),

    /// Serialization error
    Serialization(String),

    /// Deserialization error
    DeserializationError(String),

    /// Step limit exceeded in dynamic expression evaluation
    StepLimitExceeded(u32),

    /// Type mismatch in dynamic expression evaluation
    TypeMismatch { expected: String, actual: String },

    /// Invalid function arity in combinator application
    InvalidArity { expected: usize, actual: usize },

    /// Arithmetic overflow in integer operation
    ArithmeticOverflow,

    /// Division by zero
    DivisionByZero,

    /// Invalid operation (e.g., unsupported combinator)
    InvalidOperation(String),

    /// Expression evaluation error
    ExprEvaluation(String),

    /// Environment error (e.g., VM or runtime issue)
    EnvironmentError(String),

    /// Error related to coprocessor interaction
    Coprocessor(String),

    /// Generic error for compatibility
    GenericError(String),

    /// Not implemented
    NotImplemented(String),

    /// Field not found
    FieldNotFound(String, ResourceId),

    /// Resource not found
    ResourceNotFound(ResourceId),

    /// Witness deserialization error
    WitnessDeserialization(String),

    /// Invalid argument error
    InvalidArgument(String),

    /// Circuit compilation error
    CircuitCompilationError(String),

    /// Witness generation error
    WitnessGeneration(String),
}

#[cfg(feature = "host")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Error::CircuitCompilation(msg) => {
                write!(f, "Circuit compilation error: {}", msg)
            }
            Error::ProofGeneration(msg) => {
                write!(f, "Proof generation error: {}", msg)
            }
            Error::ProofVerification(msg) => {
                write!(f, "Proof verification error: {}", msg)
            }
            Error::StepLimitExceeded(limit) => {
                write!(f, "Step limit exceeded: {}", limit)
            }
            Error::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            Error::InvalidArity { expected, actual } => write!(
                f,
                "Invalid arity: expected {} arguments, got {}",
                expected, actual
            ),
            Error::ArithmeticOverflow => write!(f, "Arithmetic overflow"),
            Error::DivisionByZero => write!(f, "Division by zero"),
            Error::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {}", msg)
            }
            Error::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            Error::ExprEvaluation(msg) => {
                write!(f, "Expression evaluation error: {}", msg)
            }
            Error::EnvironmentError(msg) => {
                write!(f, "Environment error: {}", msg)
            }
            Error::Coprocessor(msg) => write!(f, "Coprocessor error: {}", msg),
            Error::GenericError(msg) => write!(f, "Generic error: {}", msg),
            Error::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            Error::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Error::FieldNotFound(field, id) => {
                write!(f, "Field '{}' not found for resource {}", field, id)
            }
            Error::ResourceNotFound(id) => write!(f, "Resource not found: {}", id),
            Error::WitnessDeserialization(msg) => {
                write!(f, "Witness deserialization error: {}", msg)
            }
            Error::InvalidArgument(msg) => write!(f, "Invalid argument error: {}", msg),
            Error::CircuitCompilationError(msg) => {
                write!(f, "Circuit compilation error: {}", msg)
            }
            Error::WitnessGeneration(msg) => write!(f, "Witness generation error: {}", msg),
        }
    }
}

#[cfg(not(feature = "host"))]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Error::CircuitCompilation(msg) => {
                write!(f, "Circuit compilation error: {}", msg)
            }
            Error::ProofGeneration(msg) => {
                write!(f, "Proof generation error: {}", msg)
            }
            Error::ProofVerification(msg) => {
                write!(f, "Proof verification error: {}", msg)
            }
            Error::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            Error::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            Error::StepLimitExceeded(limit) => {
                write!(f, "Step limit exceeded: {}", limit)
            }
            Error::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            Error::InvalidArity { expected, actual } => write!(
                f,
                "Invalid arity: expected {} arguments, got {}",
                expected, actual
            ),
            Error::ArithmeticOverflow => write!(f, "Arithmetic overflow"),
            Error::DivisionByZero => write!(f, "Division by zero"),
            Error::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {}", msg)
            }
            Error::ExprEvaluation(msg) => {
                write!(f, "Expression error: {}", msg)
            }
            Error::EnvironmentError(msg) => {
                write!(f, "Environment error: {}", msg)
            }
            Error::Coprocessor(msg) => write!(f, "Coprocessor error: {}", msg),
            Error::GenericError(msg) => write!(f, "Generic error: {}", msg),
            Error::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Error::FieldNotFound(field, id) => {
                write!(f, "Field '{}' not found for resource {}", field, id)
            }
            Error::ResourceNotFound(id) => write!(f, "Resource not found: {}", id),
            Error::WitnessDeserialization(msg) => {
                write!(f, "Witness deserialization error: {}", msg)
            }
            Error::InvalidArgument(msg) => write!(f, "Invalid argument error: {}", msg),
            Error::CircuitCompilationError(msg) => {
                write!(f, "Circuit compilation error: {}", msg)
            }
            Error::WitnessGeneration(msg) => write!(f, "Witness generation error: {}", msg),
        }
    }
}

#[cfg(feature = "host")]
impl std::error::Error for Error {}

//-----------------------------------------------------------------------------
// Deterministic Serialization Helper
//-----------------------------------------------------------------------------

/// Serialize a value using ssz and return the serialized bytes
pub fn serialize<T: Encode>(value: &T) -> Result<Vec<u8>, Error> {
    Ok(value.as_ssz_bytes())
}

/// Deserialize a value using ssz
pub fn deserialize<T: Decode>(data: &[u8]) -> Result<T, Error> {
    T::from_ssz_bytes(data)
        .map_err(|e| Error::DeserializationError(format!("Failed to deserialize: {}", e)))
}

/// Generate a deterministic hash from a serializable value
pub fn hash_from_serializable<T: Encode>(
    value: &T,
) -> Result<[u8; 32], Error> {
    let serialized = value.as_ssz_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    Ok(bytes)
}

/// Verify that the hash of a serializable value matches the expected hash
pub fn verify_hash<T: Encode>(
    value: &T,
    expected: &[u8; 32],
) -> Result<bool, Error> {
    let computed = hash_from_serializable(value)?;
    Ok(computed == *expected)
}

//-----------------------------------------------------------------------------
// Core Circuit Execution Logic
//-----------------------------------------------------------------------------

/// Run the circuit with the provided witnesses
///
/// # Arguments
/// * `circuit_data` - Serialized circuit information
/// * `witness_data` - Witness data for the circuit
///
/// # Returns
/// Result containing the circuit output or an error
pub fn run_circuit<T: Decode + Encode>(
    input_data: &[u8],
) -> Result<Vec<u8>, Error> {
    // Deserialize the input
    let witness_data = deserialize_witness_data::<T>(input_data)?;
    
    // Process the witness (placeholder implementation)
    let witnesses = vec![witness_data];
    process_witnesses(witnesses)
}

/// Process the witness data and produce a result
fn process_witnesses<T: Decode + Encode>(
    witnesses: Vec<T>,
) -> Result<Vec<u8>, Error> {
    // For now, just serialize the first witness as output
    if let Some(first_witness) = witnesses.first() {
        serialize(first_witness)
    } else {
        Err(Error::InvalidInput("No witnesses provided".to_string()))
    }
}

fn deserialize_witness_data<T: Decode>(
    data: &[u8],
) -> Result<T, Error> {
    T::from_ssz_bytes(data)
        .map_err(|e| Error::WitnessDeserialization(format!("Failed to deserialize witness: {}", e)))
}

/// ZK Combinator Interpreter for circuit execution
#[derive(Debug, Clone)]
pub struct ZkCombinatorInterpreter {
    /// Step counter for execution limits
    pub step_count: u32,
    /// Maximum allowed steps
    pub max_steps: u32,
}

impl Default for ZkCombinatorInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkCombinatorInterpreter {
    /// Create a new interpreter with default step limit
    pub fn new() -> Self {
        Self {
            step_count: 0,
            max_steps: 10000,
        }
    }

    /// Create a new interpreter with custom step limit
    pub fn with_step_limit(max_steps: u32) -> Self {
        Self {
            step_count: 0,
            max_steps,
        }
    }

    /// Reset the step counter
    pub fn reset(&mut self) {
        self.step_count = 0;
    }

    /// Increment step counter and check limits
    pub fn increment_step(&mut self) -> Result<(), Error> {
        self.step_count += 1;
        if self.step_count > self.max_steps {
            Err(Error::StepLimitExceeded(self.step_count))
        } else {
            Ok(())
        }
    }
}
