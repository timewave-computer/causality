//! Storage commitment types for blockchain state verification
//!
//! This module provides types for tracking and verifying blockchain storage state
//! using content addressing and cryptographic commitments.

use crate::system::{EntityId, Str, ContentAddressable, Error, Result};
use crate::system::serialization::ToBytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};

/// A commitment to a specific piece of blockchain storage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCommitment {
    /// Unique identifier for this storage commitment
    pub id: EntityId,
    
    /// The blockchain domain this storage belongs to
    pub domain: Str,
    
    /// Contract address (for EVM chains) or equivalent identifier
    pub contract_address: Str,
    
    /// Storage key or path
    pub storage_key: Str,
    
    /// Expected storage value hash
    pub value_hash: [u8; 32],
    
    /// Block number or height when this commitment was made
    pub block_number: u64,
    
    /// Optional metadata for additional context
    pub metadata: BTreeMap<Str, Str>,
}

impl StorageCommitment {
    /// Create a new storage commitment
    pub fn new(
        domain: impl Into<Str>,
        contract_address: impl Into<Str>,
        storage_key: impl Into<Str>,
        value_hash: [u8; 32],
        block_number: u64,
    ) -> Self {
        let domain = domain.into();
        let contract_address = contract_address.into();
        let storage_key = storage_key.into();
        
        // Generate deterministic ID from commitment data
        let content_str = Str::from(format!(
            "storage_commitment:{}:{}:{}:{}",
            domain, contract_address, storage_key, block_number
        ));
        let id = EntityId::from_content(&content_str);
        
        Self {
            id,
            domain,
            contract_address,
            storage_key,
            value_hash,
            block_number,
            metadata: BTreeMap::new(),
        }
    }
    
    /// Add metadata to the commitment
    pub fn with_metadata(mut self, key: impl Into<Str>, value: impl Into<Str>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Get the storage path as a string
    pub fn storage_path(&self) -> String {
        format!("{}:{}", self.contract_address, self.storage_key)
    }
    
    /// Verify that a given value matches this commitment
    pub fn verify_value(&self, value: &[u8]) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(value);
        let computed_hash = hasher.finalize();
        computed_hash.as_slice() == &self.value_hash
    }
}

impl ContentAddressable for StorageCommitment {
    fn content_id(&self) -> EntityId {
        self.id
    }
}

/// A storage key derivation utility for complex storage layouts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageKeyDerivation {
    /// Base storage slot
    pub base_slot: u64,
    
    /// Key components for mappings and arrays
    pub key_components: Vec<StorageKeyComponent>,
    
    /// The final derived storage key
    pub derived_key: Str,
}

/// Components used in storage key derivation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageKeyComponent {
    /// A fixed value
    Fixed(Vec<u8>),
    
    /// An address (20 bytes for Ethereum)
    Address([u8; 20]),
    
    /// A 256-bit integer
    Uint256([u8; 32]),
    
    /// A string value
    String(Str),
    
    /// An array index
    ArrayIndex(u64),
}

impl StorageKeyDerivation {
    /// Create a new storage key derivation
    pub fn new(base_slot: u64) -> Self {
        Self {
            base_slot,
            key_components: Vec::new(),
            derived_key: Str::from(format!("0x{:064x}", base_slot)),
        }
    }
    
    /// Add a key component for mapping or array access
    pub fn with_component(mut self, component: StorageKeyComponent) -> Self {
        self.key_components.push(component);
        self.recompute_key();
        self
    }
    
    /// Recompute the derived storage key based on components
    fn recompute_key(&mut self) {
        if self.key_components.is_empty() {
            self.derived_key = Str::from(format!("0x{:064x}", self.base_slot));
            return;
        }
        
        let mut hasher = Sha256::new();
        
        for component in &self.key_components {
            match component {
                StorageKeyComponent::Fixed(bytes) => hasher.update(bytes),
                StorageKeyComponent::Address(addr) => {
                    let mut padded = [0u8; 32];
                    padded[12..].copy_from_slice(addr);
                    hasher.update(&padded);
                }
                StorageKeyComponent::Uint256(value) => hasher.update(value),
                StorageKeyComponent::String(s) => hasher.update(s.value.as_bytes()),
                StorageKeyComponent::ArrayIndex(idx) => {
                    let mut bytes = [0u8; 32];
                    bytes[24..].copy_from_slice(&idx.to_be_bytes());
                    hasher.update(&bytes);
                }
            }
        }
        
        let mut base_bytes = [0u8; 32];
        base_bytes[24..].copy_from_slice(&self.base_slot.to_be_bytes());
        hasher.update(&base_bytes);
        
        let result = hasher.finalize();
        self.derived_key = Str::from(format!("0x{}", hex::encode(result)));
    }
    
    /// Get the final derived storage key
    pub fn key(&self) -> &Str {
        &self.derived_key
    }
}

/// Extension trait for content addressing with storage proofs
pub trait StorageAddressable: ContentAddressable {
    /// Get the storage commitment for this item
    fn storage_commitment(&self) -> Option<StorageCommitment>;
    
    /// Verify storage consistency
    fn verify_storage(&self, commitment: &StorageCommitment) -> Result<bool> {
        Ok(self.content_id() == commitment.id)
    }
}

/// A collection of storage commitments for batch verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCommitmentBatch {
    /// Batch identifier
    pub id: EntityId,
    
    /// Individual storage commitments
    pub commitments: Vec<StorageCommitment>,
    
    /// Block range for this batch
    pub block_range: (u64, u64),
    
    /// Merkle root of all commitments
    pub merkle_root: [u8; 32],
}

impl StorageCommitmentBatch {
    /// Create a new storage commitment batch
    pub fn new(commitments: Vec<StorageCommitment>) -> Result<Self> {
        if commitments.is_empty() {
            return Err(Error::serialization("Cannot create empty storage commitment batch"));
        }
        
        let block_range = (
            commitments.iter().map(|c| c.block_number).min().unwrap(),
            commitments.iter().map(|c| c.block_number).max().unwrap(),
        );
        
        let merkle_root = Self::compute_merkle_root(&commitments)?;
        
        // Generate batch ID
        let content_str = Str::from(format!(
            "storage_batch:{}:{}:{}",
            commitments.len(),
            block_range.0,
            block_range.1
        ));
        let id = EntityId::from_content(&content_str);
        
        Ok(Self {
            id,
            commitments,
            block_range,
            merkle_root,
        })
    }
    
    /// Compute merkle root of commitments
    fn compute_merkle_root(commitments: &[StorageCommitment]) -> Result<[u8; 32]> {
        if commitments.is_empty() {
            return Err(Error::serialization("Cannot compute merkle root of empty commitments"));
        }
        
        let mut hashes: Vec<[u8; 32]> = commitments
            .iter()
            .map(|c| {
                let mut hasher = Sha256::new();
                hasher.update(&c.id.to_bytes());
                hasher.update(&c.value_hash);
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                hash
            })
            .collect();
        
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                let result = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&result);
                next_level.push(hash);
            }
            
            hashes = next_level;
        }
        
        Ok(hashes[0])
    }
    
    /// Verify the batch merkle root
    pub fn verify_merkle_root(&self) -> Result<bool> {
        let computed_root = Self::compute_merkle_root(&self.commitments)?;
        Ok(computed_root == self.merkle_root)
    }
}

impl ContentAddressable for StorageCommitmentBatch {
    fn content_id(&self) -> EntityId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_commitment_creation() {
        let commitment = StorageCommitment::new(
            "ethereum",
            "0x1234567890123456789012345678901234567890",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            [1u8; 32],
            12345,
        );
        
        assert_eq!(commitment.domain.as_str(), "ethereum");
        assert_eq!(commitment.contract_address.as_str(), "0x1234567890123456789012345678901234567890");
        assert_eq!(commitment.block_number, 12345);
    }
    
    #[test]
    fn test_storage_key_derivation() {
        let mut derivation = StorageKeyDerivation::new(0);
        derivation = derivation.with_component(StorageKeyComponent::Address([1u8; 20]));
        
        assert!(derivation.key().as_str().starts_with("0x"));
        assert_eq!(derivation.key().as_str().len(), 66);
    }
    
    #[test]
    fn test_storage_commitment_batch() {
        let commitment1 = StorageCommitment::new("ethereum", "0x1234", "0x0000", [1u8; 32], 100);
        let commitment2 = StorageCommitment::new("ethereum", "0x5678", "0x0001", [2u8; 32], 101);
        
        let batch = StorageCommitmentBatch::new(vec![commitment1, commitment2]).unwrap();
        
        assert_eq!(batch.commitments.len(), 2);
        assert_eq!(batch.block_range, (100, 101));
        assert!(batch.verify_merkle_root().unwrap());
    }
} 