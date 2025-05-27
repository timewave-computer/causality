// ZK Models
//
// This module defines the core data structures used in ZK circuits and proofs.

extern crate alloc;

use alloc::{vec::Vec, string::String};
use causality_types::serialization::{SimpleSerialize, Encode, Decode, DecodeError};
use sha2::{Digest, Sha256};

use crate::core::Error;

//-----------------------------------------------------------------------------
// Content Addressable Trait
//-----------------------------------------------------------------------------

/// Trait for types that can generate content-based hashes
pub trait ContentAddressable: Encode {
    /// Generate a SHA-256 hash of the serialized content
    fn content_hash(&self) -> Result<[u8; 32], Error> {
        let serialized = self.as_ssz_bytes();

        let mut hasher = Sha256::new();
        hasher.update(&serialized);

        Ok(hasher.finalize().into())
    }

    /// Verify that the content hash matches the expected hash
    fn verify_hash(&self, expected_hash: &[u8; 32]) -> Result<bool, Error> {
        let actual_hash = self.content_hash()?;
        Ok(&actual_hash == expected_hash)
    }
}

/// Blanket implementation for all types that implement Encode
impl<T: Encode> ContentAddressable for T {}

//-----------------------------------------------------------------------------
// Conversion Functions
//-----------------------------------------------------------------------------

/// Create an ID from a serializable type
pub fn id_from_serializable<T: Encode>(
    value: &T,
) -> Result<[u8; 32], Error> {
    let serialized = value.as_ssz_bytes();

    let mut hasher = Sha256::new();
    hasher.update(&serialized);

    Ok(hasher.finalize().into())
}

/// Verify that the content of a value corresponds to its ID
pub fn verify_id<T: Encode>(
    value: &T,
    id: &[u8; 32],
) -> Result<bool, Error> {
    let computed_id = id_from_serializable(value)?;
    Ok(&computed_id == id)
}

//-----------------------------------------------------------------------------
// Circuit Types
//-----------------------------------------------------------------------------

/// Circuit data structure for ZK proofs
#[derive(Debug, Clone)]
pub struct CircuitData {
    /// Circuit identifier
    pub id: [u8; 32],
    /// Circuit bytecode
    pub bytecode: Vec<u8>,
    /// Circuit metadata
    pub metadata: CircuitMetadata,
}

impl SimpleSerialize for CircuitData {}

impl Encode for CircuitData {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(&(self.bytecode.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.bytecode);
        bytes.extend_from_slice(&self.metadata.as_ssz_bytes());
        bytes
    }
}

impl Decode for CircuitData {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 36 {
            return Err(DecodeError {
                message: "Invalid CircuitData length".to_string(),
            });
        }
        
        let mut id = [0u8; 32];
        id.copy_from_slice(&bytes[0..32]);
        
        let bytecode_len = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]) as usize;
        if bytes.len() < 36 + bytecode_len {
            return Err(DecodeError {
                message: "Invalid bytecode length".to_string(),
            });
        }
        
        let bytecode = bytes[36..36 + bytecode_len].to_vec();
        
        Ok(CircuitData {
            id,
            bytecode,
            metadata: CircuitMetadata {
                name: "default".to_string(),
                version: "1.0".to_string(),
                timing: CircuitTiming {
                    expected_ms: 0,
                    max_ms: 0,
                },
                circuit_type: CircuitType::Wasm,
            },
        })
    }
}

/// Circuit metadata
#[derive(Debug, Clone)]
pub struct CircuitMetadata {
    /// Circuit name
    pub name: String,
    /// Circuit version
    pub version: String,
    /// Circuit timing information
    pub timing: CircuitTiming,
    /// Circuit type
    pub circuit_type: CircuitType,
}

impl SimpleSerialize for CircuitMetadata {}

impl Encode for CircuitMetadata {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.version.as_ssz_bytes());
        bytes.extend_from_slice(&self.timing.as_ssz_bytes());
        bytes.extend_from_slice(&self.circuit_type.as_ssz_bytes());
        bytes
    }
}

/// Circuit timing information
#[derive(Debug, Clone)]
pub struct CircuitTiming {
    /// Expected execution time in milliseconds
    pub expected_ms: u64,
    /// Maximum execution time in milliseconds
    pub max_ms: u64,
}

impl SimpleSerialize for CircuitTiming {}

impl Encode for CircuitTiming {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.expected_ms.to_le_bytes());
        bytes.extend_from_slice(&self.max_ms.to_le_bytes());
        bytes
    }
}

/// Circuit type enumeration
#[derive(Debug, Clone)]
pub enum CircuitType {
    /// WASM circuit
    Wasm,
    /// RISC-V circuit
    RiscV,
    /// Native circuit
    Native,
}

impl SimpleSerialize for CircuitType {}

impl Encode for CircuitType {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            CircuitType::Wasm => vec![0],
            CircuitType::RiscV => vec![1],
            CircuitType::Native => vec![2],
        }
    }
} 