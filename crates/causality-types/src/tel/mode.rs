//! Defines execution modes for TEL.

use crate::serialization::{Decode, Encode, SimpleSerialize, DecodeError};

/// The mode in which a TEL effect should be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ExecutionMode {
    /// The effect should be executed eagerly, as soon as possible.
    #[default]
    Eager = 0,
    /// The effect should be executed only once all dependencies are satisfied.
    Strict = 1,
    /// The effect should be executed lazily, only when needed.
    Lazy = 2,
}


// Manually implement Encode for ExecutionMode
impl Encode for ExecutionMode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let value = match self {
            ExecutionMode::Eager => 0u8,
            ExecutionMode::Strict => 1u8,
            ExecutionMode::Lazy => 2u8,
        };
        vec![value]
    }
}

// Manually implement Decode for ExecutionMode
impl Decode for ExecutionMode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Expected at least 1 byte for ExecutionMode".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(ExecutionMode::Eager),
            1 => Ok(ExecutionMode::Strict),
            2 => Ok(ExecutionMode::Lazy),
            _ => Err(DecodeError {
                message: format!("Invalid ExecutionMode value: {}", bytes[0]),
            }),
        }
    }
}

// Implement SimpleSerialize for ExecutionMode
impl SimpleSerialize for ExecutionMode {}

/// The mode in which the TEL interpreter should operate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum InterpreterMode {
    /// The interpreter should execute effects sequentially.
    #[default]
    Sequential = 0,
    /// The interpreter should execute effects in parallel when possible.
    Parallel = 1,
    /// The interpreter should optimize for ZK proof generation.
    ZkOptimized = 2,
    /// The interpreter should evaluate effects (compute results).
    Evaluate = 3,
    /// The interpreter should simulate effects (dry run).
    Simulate = 4,
}


// Manually implement Encode for InterpreterMode
impl Encode for InterpreterMode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let value = match self {
            InterpreterMode::Sequential => 0u8,
            InterpreterMode::Parallel => 1u8,
            InterpreterMode::ZkOptimized => 2u8,
            InterpreterMode::Evaluate => 3u8,
            InterpreterMode::Simulate => 4u8,
        };
        vec![value]
    }
}

// Manually implement Decode for InterpreterMode
impl Decode for InterpreterMode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Expected at least 1 byte for InterpreterMode".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(InterpreterMode::Sequential),
            1 => Ok(InterpreterMode::Parallel),
            2 => Ok(InterpreterMode::ZkOptimized),
            3 => Ok(InterpreterMode::Evaluate),
            4 => Ok(InterpreterMode::Simulate),
            _ => Err(DecodeError {
                message: format!("Invalid InterpreterMode value: {}", bytes[0]),
            }),
        }
    }
}

// Implement SimpleSerialize for InterpreterMode
impl SimpleSerialize for InterpreterMode {}
