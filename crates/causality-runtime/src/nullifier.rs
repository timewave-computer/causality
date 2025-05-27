//! Nullifier Registry
//!
//! This module provides a registry for tracking resource consumption and nullifiers
//! to ensure resources can only be used once in the causality system.

//-----------------------------------------------------------------------------
// Nullifier Registry
//-----------------------------------------------------------------------------

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use causality_types::serialization::{SimpleSerialize, Encode, Decode, DecodeError};
use parking_lot::Mutex;

use causality_types::{
    core::id::{ResourceId, TransactionId, DomainId, EntityId, AsId},
    resource::Nullifier,
};

use causality_core::smt::{TegMultiDomainSmt, MemoryBackend};

//-----------------------------------------------------------------------------
// Storage Backend
//-----------------------------------------------------------------------------

/// Storage backend for NullifierRegistry
#[derive(Debug)]
pub enum NullifierStorageBackend {
    /// Traditional BTreeMap storage
    BTreeMap {
        consumed_resources: BTreeMap<ResourceId, Nullifier>,
        nullified_resources: BTreeSet<ResourceId>,
    },
    /// SMT-backed storage with domain awareness
    Smt {
        smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
        domain_id: DomainId,
    },
}

impl Default for NullifierStorageBackend {
    fn default() -> Self {
        Self::BTreeMap {
            consumed_resources: BTreeMap::new(),
            nullified_resources: BTreeSet::new(),
        }
    }
}

impl Clone for NullifierStorageBackend {
    fn clone(&self) -> Self {
        match self {
            Self::BTreeMap { consumed_resources, nullified_resources } => Self::BTreeMap {
                consumed_resources: consumed_resources.clone(),
                nullified_resources: nullified_resources.clone(),
            },
            Self::Smt { smt, domain_id } => Self::Smt {
                smt: Arc::clone(smt),
                domain_id: *domain_id,
            },
        }
    }
}

//-----------------------------------------------------------------------------
// Nullifier Registry Implementation
//-----------------------------------------------------------------------------

/// Registry for tracking resource consumption and nullifiers.
#[derive(Debug, Clone, Default)]
pub struct NullifierRegistry {
    storage: NullifierStorageBackend,
}

impl Encode for NullifierRegistry {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match &self.storage {
            NullifierStorageBackend::BTreeMap { consumed_resources, nullified_resources } => {
                // Simple implementation - serialize the two maps separately
                let mut bytes = Vec::new();
                
                // Serialize consumed_resources map
                bytes.extend((consumed_resources.len() as u32).to_le_bytes());
                for (id, nullifier) in consumed_resources {
                    bytes.extend(id.as_ssz_bytes());
                    bytes.extend(nullifier.as_ssz_bytes());
                }
                
                // Serialize nullified_resources set
                bytes.extend((nullified_resources.len() as u32).to_le_bytes());
                for id in nullified_resources {
                    bytes.extend(id.as_ssz_bytes());
                }
                
                bytes
            }
            NullifierStorageBackend::Smt { smt: _, domain_id } => {
                // Serialize SMT backend metadata
                let mut bytes = Vec::new();
                bytes.extend(domain_id.as_ssz_bytes());
                // For now, just serialize the domain_id as the SMT state is complex
                // In a full implementation, we'd serialize the SMT tree structure
                bytes
            }
        }
    }
}

impl Decode for NullifierRegistry {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Deserialize consumed_resources map
        if offset + 4 > bytes.len() {
            return Err(DecodeError { message: "Insufficient data for consumed_resources length".to_string() });
        }
        let consumed_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        let mut consumed_resources = BTreeMap::new();
        for _ in 0..consumed_len {
            let id = ResourceId::from_ssz_bytes(&bytes[offset..])?;
            offset += id.as_ssz_bytes().len();
            
            let nullifier = Nullifier::from_ssz_bytes(&bytes[offset..])?;
            offset += nullifier.as_ssz_bytes().len();
            
            consumed_resources.insert(id, nullifier);
        }
        
        // Deserialize nullified_resources set
        if offset + 4 > bytes.len() {
            return Err(DecodeError { message: "Insufficient data for nullified_resources length".to_string() });
        }
        let nullified_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
        offset += 4;
        
        let mut nullified_resources = BTreeSet::new();
        for _ in 0..nullified_len {
            let id = ResourceId::from_ssz_bytes(&bytes[offset..])?;
            offset += id.as_ssz_bytes().len();
            nullified_resources.insert(id);
        }
        
        Ok(NullifierRegistry {
            storage: NullifierStorageBackend::BTreeMap {
                consumed_resources,
                nullified_resources,
            },
        })
    }
}

impl SimpleSerialize for NullifierRegistry {}

impl NullifierRegistry {
    /// Create a new empty nullifier registry with BTreeMap storage.
    pub fn new() -> Self {
        Self {
            storage: NullifierStorageBackend::default(),
        }
    }

    /// Create a new nullifier registry with SMT storage.
    pub fn new_with_smt(domain_id: DomainId) -> Self {
        let backend = MemoryBackend::new();
        let smt = TegMultiDomainSmt::new(backend);
        Self {
            storage: NullifierStorageBackend::Smt {
                smt: Arc::new(Mutex::new(smt)),
                domain_id,
            },
        }
    }

    /// Register a resource nullifier.
    pub fn register(
        &mut self,
        resource_id: &ResourceId,
        nullifier: Nullifier,
    ) -> Result<()> {
        match &mut self.storage {
            NullifierStorageBackend::BTreeMap { consumed_resources, nullified_resources } => {
                // Check if resource already has a nullifier
                if consumed_resources.contains_key(resource_id) {
                    return Err(anyhow!("Resource already nullified: {:?}", resource_id));
                }

                // Store nullifier for this resource
                consumed_resources.insert(*resource_id, nullifier);

                // Add resource ID to the set of nullified resources
                nullified_resources.insert(*resource_id);
            }
            NullifierStorageBackend::Smt { smt, domain_id } => {
                let nullifier_key = format!("{}-nullifier-{}", domain_id.namespace_prefix(), resource_id);
                
                // Check if resource already has a nullifier
                if smt.lock().has_data(&nullifier_key) {
                    return Err(anyhow!("Resource already nullified: {:?}", resource_id));
                }
                
                // Store nullifier data in SMT
                let nullifier_data = nullifier.as_ssz_bytes();
                smt.lock().store_data(&nullifier_key, &nullifier_data)
                    .map_err(|e| anyhow!("Failed to store nullifier: {}", e))?;
            }
        }

        Ok(())
    }

    /// Check if a resource has been nullified.
    pub fn is_nullified(&self, resource_id: &ResourceId) -> Result<bool> {
        match &self.storage {
            NullifierStorageBackend::BTreeMap { consumed_resources, .. } => {
                Ok(consumed_resources.contains_key(resource_id))
            }
            NullifierStorageBackend::Smt { smt, domain_id } => {
                let nullifier_key = format!("{}-nullifier-{}", domain_id.namespace_prefix(), resource_id);
                Ok(smt.lock().has_data(&nullifier_key))
            }
        }
    }

    /// Check if a resource has been consumed (alias for is_nullified).
    pub fn is_consumed(&self, resource_id: &ResourceId) -> bool {
        self.is_nullified(resource_id).unwrap_or(false)
    }

    /// Consume a resource by creating and registering a nullifier for it.
    pub fn consume_resource(
        &mut self,
        resource_id: &ResourceId,
        _tx_id: &TransactionId,
    ) -> Result<()> {
        // Create a nullifier for this resource
        let nullifier = Nullifier::new(EntityId::new((*resource_id).inner()));

        // Register the nullifier
        self.register(resource_id, nullifier)
    }

    /// Get the nullifier for a specific resource, if it exists.
    pub fn get_nullifier(&self, resource_id: &ResourceId) -> Option<Nullifier> {
        match &self.storage {
            NullifierStorageBackend::BTreeMap { consumed_resources, .. } => {
                consumed_resources.get(resource_id).cloned()
            }
            NullifierStorageBackend::Smt { smt, domain_id } => {
                let nullifier_key = format!("{}-nullifier-{}", domain_id.namespace_prefix(), resource_id);
                if let Ok(Some(nullifier_data)) = smt.lock().get_data(&nullifier_key) {
                    // Deserialize nullifier from SSZ bytes
                    Nullifier::from_ssz_bytes(&nullifier_data).ok()
                } else {
                    None
                }
            }
        }
    }

    /// Get a copy of all consumed resource IDs
    pub fn get_consumed_resources(&self) -> Vec<ResourceId> {
        match &self.storage {
            NullifierStorageBackend::BTreeMap { consumed_resources, .. } => {
                // Return all keys from the consumed_resources map
                consumed_resources.keys().cloned().collect()
            }
            NullifierStorageBackend::Smt { smt: _, domain_id } => {
                // Implement SMT iteration for consumed resources
                let consumed_resources = Vec::new();
                let _prefix = format!("{}-nullifier-", domain_id.namespace_prefix());
                
                // In a real implementation, we'd iterate through SMT keys with the prefix
                // For now, return empty vector as SMT doesn't expose key iteration
                consumed_resources
            }
        }
    }
}
