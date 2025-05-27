//! Nullifier types for resource consumption proofs

use crate::primitive::ids::{EntityId, AsId};
use crate::system::serialization::{Encode, Decode, DecodeError, SimpleSerialize};
use sha2::{Sha256, Digest};

/// Nullifier represents a proof that a resource has been consumed
#[derive(Debug, Clone, PartialEq)]
pub struct Nullifier {
    pub resource_id: EntityId,
    pub nullifier_hash: [u8; 32],
}

impl Nullifier {
    pub fn new(resource_id: EntityId) -> Self {
        // Simple implementation - in practice would use cryptographic nullifier
        let mut hasher = Sha256::new();
        hasher.update(resource_id.inner());
        hasher.update(b"nullifier");
        let nullifier_hash: [u8; 32] = hasher.finalize().into();
        
        Self {
            resource_id,
            nullifier_hash,
        }
    }
}

impl Encode for Nullifier {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.resource_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.nullifier_hash);
        bytes
    }
}

impl Decode for Nullifier {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 64 {
            return Err(DecodeError {
                message: format!("Nullifier requires at least 64 bytes, got {}", bytes.len()),
            });
        }
        
        let resource_id = EntityId::from_ssz_bytes(&bytes[0..32])?;
        let mut nullifier_hash = [0u8; 32];
        nullifier_hash.copy_from_slice(&bytes[32..64]);
        
        Ok(Self {
            resource_id,
            nullifier_hash,
        })
    }
}

impl SimpleSerialize for Nullifier {} 