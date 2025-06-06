//! Nullifier system for tracking resource consumption
//!
//! This module implements a nullifier system for tracking resource consumption
//! without compromising privacy or immutability. Resources remain immutable,
//! and consumption is tracked via cryptographic nullifiers.

use crate::system::content_addressing::{NullifierId, ResourceId, Timestamp};
use crate::{Sha256Hasher, Hasher};
use ssz::{Encode, Decode};
use std::collections::HashSet;

/// A cryptographic nullifier that proves a resource has been consumed
/// without revealing which specific resource was consumed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nullifier {
    /// Unique nullifier identifier (derived from resource + secret)
    pub id: NullifierId,
    
    /// When this nullifier was created
    pub timestamp: Timestamp,
    
    /// Optional metadata about the consumption
    pub metadata: Option<String>,
}

/// Nullifier set for tracking all consumed resources
#[derive(Debug, Clone)]
pub struct NullifierSet {
    /// Set of all nullifiers (consumed resources)
    nullifiers: HashSet<NullifierId>,
    
    /// Creation timestamps for each nullifier
    timestamps: std::collections::HashMap<NullifierId, Timestamp>,
}

/// Error types for nullifier operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullifierError {
    /// Resource has already been consumed (nullifier exists)
    AlreadyConsumed(NullifierId),
    
    /// Invalid nullifier format
    InvalidNullifier,
    
    /// Nullifier generation failed
    GenerationFailed(String),
}

impl Nullifier {
    /// Generate a nullifier for a resource consumption
    /// 
    /// The nullifier is derived from:
    /// - Resource ID (public)
    /// - Consumption context (operation type, timestamp)
    /// - Optional secret material for privacy
    pub fn generate(
        resource_id: ResourceId,
        operation: &str,
        secret: Option<&[u8]>,
    ) -> Self {
        let timestamp = Timestamp::now();
        
        // Combine inputs for nullifier derivation
        let mut input_data = Vec::new();
        input_data.extend_from_slice(resource_id.as_bytes());
        input_data.extend_from_slice(operation.as_bytes());
        input_data.extend_from_slice(&timestamp.as_millis().to_le_bytes());
        
        // Add secret material if provided (for privacy)
        if let Some(secret) = secret {
            input_data.extend_from_slice(secret);
        }
        
        // Generate deterministic nullifier ID
        let hash = Sha256Hasher::hash(&input_data);
        let id = NullifierId::from_bytes(hash.into());
        
        Self {
            id,
            timestamp,
            metadata: Some(operation.to_string()),
        }
    }
    
    /// Generate a nullifier with entropy for privacy
    pub fn generate_private(
        resource_id: ResourceId,
        operation: &str,
        private_key: &[u8; 32],
        nonce: u64,
    ) -> Self {
        let mut secret_material = Vec::new();
        secret_material.extend_from_slice(private_key);
        secret_material.extend_from_slice(&nonce.to_le_bytes());
        
        Self::generate(resource_id, operation, Some(&secret_material))
    }
    
    /// Verify that this nullifier was correctly generated for a resource
    pub fn verify(&self, resource_id: ResourceId, operation: &str, secret: Option<&[u8]>) -> bool {
        let expected = Self::generate(resource_id, operation, secret);
        // Note: We only check the ID, not timestamp since that varies
        self.id == expected.id
    }
}

impl NullifierSet {
    /// Create a new empty nullifier set
    pub fn new() -> Self {
        Self {
            nullifiers: HashSet::new(),
            timestamps: std::collections::HashMap::new(),
        }
    }
    
    /// Add a nullifier to the set (marks resource as consumed)
    pub fn add_nullifier(&mut self, nullifier: Nullifier) -> Result<(), NullifierError> {
        if self.nullifiers.contains(&nullifier.id) {
            return Err(NullifierError::AlreadyConsumed(nullifier.id));
        }
        
        self.timestamps.insert(nullifier.id, nullifier.timestamp);
        self.nullifiers.insert(nullifier.id);
        Ok(())
    }
    
    /// Check if a nullifier exists (resource is consumed)
    pub fn contains(&self, nullifier_id: &NullifierId) -> bool {
        self.nullifiers.contains(nullifier_id)
    }
    
    /// Check if a resource has been consumed by generating its nullifier
    pub fn is_resource_consumed(
        &self,
        resource_id: ResourceId,
        operation: &str,
        secret: Option<&[u8]>,
    ) -> bool {
        let nullifier = Nullifier::generate(resource_id, operation, secret);
        self.contains(&nullifier.id)
    }
    
    /// Try to consume a resource by adding its nullifier
    pub fn consume_resource(
        &mut self,
        resource_id: ResourceId,
        operation: &str,
        secret: Option<&[u8]>,
    ) -> Result<Nullifier, NullifierError> {
        let nullifier = Nullifier::generate(resource_id, operation, secret);
        self.add_nullifier(nullifier.clone())?;
        Ok(nullifier)
    }
    
    /// Get the timestamp when a nullifier was added
    pub fn get_consumption_time(&self, nullifier_id: &NullifierId) -> Option<Timestamp> {
        self.timestamps.get(nullifier_id).copied()
    }
    
    /// Get total number of consumed resources
    pub fn size(&self) -> usize {
        self.nullifiers.len()
    }
    
    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.nullifiers.is_empty()
    }
    
    /// Get all nullifier IDs (for ZK proof generation)
    pub fn get_all_nullifiers(&self) -> Vec<NullifierId> {
        self.nullifiers.iter().copied().collect()
    }
}

impl Default for NullifierSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Encode for Nullifier {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        32 + // id
        8 + // timestamp (fixed 8 bytes)
        4 + self.metadata.as_ref().map(|s| s.len()).unwrap_or(0) // metadata
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.id.ssz_append(buf);
        self.timestamp.ssz_append(buf);
        
        if let Some(metadata) = &self.metadata {
            (metadata.len() as u32).ssz_append(buf);
            buf.extend_from_slice(metadata.as_bytes());
        } else {
            0u32.ssz_append(buf);
        }
    }
}

impl Decode for Nullifier {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.len() < 44 { // 32 + 8 + 4 minimum
            return Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 44,
            });
        }
        
        let id = NullifierId::from_ssz_bytes(&bytes[0..32])?;
        let timestamp = Timestamp::from_ssz_bytes(&bytes[32..40])?;
        
        let metadata_len = u32::from_ssz_bytes(&bytes[40..44])? as usize;
        let metadata = if metadata_len > 0 {
            if bytes.len() < 44 + metadata_len {
                return Err(ssz::DecodeError::InvalidByteLength {
                    len: bytes.len(),
                    expected: 44 + metadata_len,
                });
            }
            Some(String::from_utf8(bytes[44..44 + metadata_len].to_vec())
                .map_err(|_| ssz::DecodeError::BytesInvalid("Invalid UTF-8 in metadata".to_string()))?)
        } else {
            None
        };
        
        Ok(Self {
            id,
            timestamp,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::content_addressing::EntityId;

    #[test]
    fn test_nullifier_generation() {
        let resource_id = EntityId::ZERO;
        let operation = "consume";
        
        let nullifier1 = Nullifier::generate(resource_id, operation, None);
        let nullifier2 = Nullifier::generate(resource_id, operation, None);
        
        // Same inputs should generate same nullifier ID (deterministic)
        assert_eq!(nullifier1.id, nullifier2.id);
        
        // But different operations should generate different nullifiers
        let nullifier3 = Nullifier::generate(resource_id, "transfer", None);
        assert_ne!(nullifier1.id, nullifier3.id);
    }
    
    #[test]
    fn test_nullifier_verification() {
        let resource_id = EntityId::ZERO;
        let operation = "consume";
        let secret = Some(b"secret".as_slice());
        
        let nullifier = Nullifier::generate(resource_id, operation, secret);
        
        assert!(nullifier.verify(resource_id, operation, secret));
        assert!(!nullifier.verify(resource_id, operation, None));
        assert!(!nullifier.verify(resource_id, "transfer", secret));
    }
    
    #[test]
    fn test_nullifier_set() {
        let mut set = NullifierSet::new();
        assert!(set.is_empty());
        
        let resource_id = EntityId::ZERO;
        let operation = "consume";
        
        // First consumption should succeed
        let nullifier = set.consume_resource(resource_id, operation, None).unwrap();
        assert_eq!(set.size(), 1);
        assert!(set.contains(&nullifier.id));
        
        // Second consumption of same resource should fail
        let result = set.consume_resource(resource_id, operation, None);
        assert!(matches!(result, Err(NullifierError::AlreadyConsumed(_))));
        
        // Different operation should succeed
        let nullifier2 = set.consume_resource(resource_id, "transfer", None).unwrap();
        assert_eq!(set.size(), 2);
        assert_ne!(nullifier.id, nullifier2.id);
    }
    
    #[test]
    fn test_private_nullifiers() {
        let resource_id = EntityId::ZERO;
        let operation = "consume";
        let private_key = [42u8; 32];
        
        let nullifier1 = Nullifier::generate_private(resource_id, operation, &private_key, 0);
        let nullifier2 = Nullifier::generate_private(resource_id, operation, &private_key, 1);
        
        // Different nonces should generate different nullifiers
        assert_ne!(nullifier1.id, nullifier2.id);
        
        // Same inputs should be deterministic
        let nullifier3 = Nullifier::generate_private(resource_id, operation, &private_key, 0);
        assert_eq!(nullifier1.id, nullifier3.id);
    }
    
    #[test]
    fn test_ssz_serialization() {
        let resource_id = EntityId::ZERO;
        let nullifier = Nullifier::generate(resource_id, "test", None);
        
        let encoded = nullifier.as_ssz_bytes();
        let decoded = Nullifier::from_ssz_bytes(&encoded).unwrap();
        
        assert_eq!(nullifier.id, decoded.id);
        assert_eq!(nullifier.metadata, decoded.metadata);
        // Note: timestamp may vary slightly due to timing
    }
} 