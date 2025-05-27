//! Runtime Store Implementation
//!
//! This module provides minimalistic registry-based stores for values with a
//! registry interface for efficient lookup and persistence.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use causality_types::serialization::Encode;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};

use causality_types::{
    core::id::{ValueExprId, DomainId},
    expr::value::ValueExpr,
    provider::registry::AsRegistry,
    serialization::Decode,
};
use causality_core::smt::{TegMultiDomainSmt, MemoryBackend};
// Use direct ID creation instead of generate_id

//-----------------------------------------------------------------------------
// Storage Backend
//-----------------------------------------------------------------------------

/// Storage backend for RuntimeValueStore
#[derive(Debug)]
pub enum ValueStorageBackend {
    /// Traditional HashMap storage
    HashMap(Arc<RwLock<HashMap<ValueExprId, ValueExpr>>>),
    /// SMT-backed storage with domain awareness
    Smt {
        smt: Arc<parking_lot::Mutex<TegMultiDomainSmt<MemoryBackend>>>,
        domain_id: DomainId,
    },
}

impl Default for ValueStorageBackend {
    fn default() -> Self {
        Self::HashMap(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl Clone for ValueStorageBackend {
    fn clone(&self) -> Self {
        match self {
            Self::HashMap(map) => Self::HashMap(Arc::clone(map)),
            Self::Smt { smt, domain_id } => Self::Smt {
                smt: Arc::clone(smt),
                domain_id: *domain_id,
            },
        }
    }
}

//-----------------------------------------------------------------------------
// Runtime Value Store
//-----------------------------------------------------------------------------

/// Thread-safe registry-based value expression store.
#[derive(Debug, Default, Clone)]
pub struct RuntimeValueStore {
    storage: ValueStorageBackend,
}

impl RuntimeValueStore {
    /// Create a new empty store with HashMap storage.
    pub fn new() -> Self {
        Self {
            storage: ValueStorageBackend::default(),
        }
    }

    /// Create a new value store with SMT storage
    pub fn new_with_smt(domain_id: DomainId) -> Self {
        let backend = MemoryBackend::new();
        let smt = TegMultiDomainSmt::new(backend);
        Self {
            storage: ValueStorageBackend::Smt {
                smt: Arc::new(parking_lot::Mutex::new(smt)),
                domain_id,
            },
        }
    }

    /// Get the raw bytes of a value expression.
    pub fn get_raw_bytes(&self, id: &ValueExprId) -> Result<Vec<u8>> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                // Acquire the read lock - parking_lot RwLock doesn't return Result
                let values = values.read();

                // Get the expression and handle the case where it doesn't exist
                let expr = values
                    .get(id)
                    .ok_or_else(|| anyhow!("Value expression not found: {:?}", id))?;

                // Serialize the value expression to bytes using SSZ
                let buf = expr.as_ssz_bytes();

                Ok(buf)
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                if let Ok(Some(value_data)) = smt.lock().get_data(&value_key) {
                    Ok(value_data)
                } else {
                    Err(anyhow!("Value expression not found in SMT: {:?}", id))
                }
            }
        }
    }

    /// Get the number of values in the store.
    pub fn len(&self) -> usize {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => values.read().len(),
            ValueStorageBackend::Smt { smt: _, domain_id } => {
                // Count values by iterating through keys with the domain prefix
                let _prefix = format!("{}-value-", domain_id.namespace_prefix());
                // For now, return 0 as SMT doesn't expose key iteration
                // In a full implementation, we'd maintain a separate counter
                0
            }
        }
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add a value and return its ID.
    pub fn add_value(&mut self, value: ValueExpr) -> Result<ValueExprId> {
        // Serialize the value to bytes using SSZ
        let buf = value.as_ssz_bytes();
        
        // Create a hash of the data for the ID
        let mut hasher = Sha256::new();
        hasher.update(&buf);
        let hash_result = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_result);
        
        let id = ValueExprId::new(hash_array);
        
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                values.write().insert(id, value);
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                smt.lock().store_data(&value_key, &buf)
                    .map_err(|e| anyhow!("Failed to store value: {}", e))?;
            }
        }
        
        Ok(id)
    }
}

#[async_trait::async_trait]
impl AsRegistry<ValueExprId, ValueExpr> for RuntimeValueStore {
    async fn register(
        &mut self,
        id: ValueExprId,
        definition: ValueExpr,
    ) -> Result<()> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                values.write().insert(id, definition);
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                let value_data = definition.as_ssz_bytes();
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                smt.lock().store_data(&value_key, &value_data)
                    .map_err(|e| anyhow!("Failed to store value: {}", e))?;
            }
        }
        Ok(())
    }

    async fn get(&self, id: &ValueExprId) -> Result<Option<ValueExpr>> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                Ok(values.read().get(id).cloned())
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                if let Ok(Some(value_data)) = smt.lock().get_data(&value_key) {
                    // Deserialize from SSZ bytes back to ValueExpr
                    match ValueExpr::from_ssz_bytes(&value_data) {
                        Ok(value_expr) => Ok(Some(value_expr)),
                        Err(_) => Ok(None), // Deserialization failed
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn unregister(&mut self, id: &ValueExprId) -> Result<()> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                values.write().remove(id);
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                // Implement SMT value removal
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                // For now, we'll just mark it as removed by storing empty data
                // In a full implementation, we'd have proper deletion
                smt.lock().store_data(&value_key, &[])
                    .map_err(|e| anyhow!("Failed to remove value: {}", e))?;
            }
        }
        Ok(())
    }

    // Implement the missing sync methods
    fn get_sync(&self, id: &ValueExprId) -> Result<Option<ValueExpr>> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                Ok(values.read().get(id).cloned())
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                if let Ok(Some(value_data)) = smt.lock().get_data(&value_key) {
                    // Deserialize from SSZ bytes back to ValueExpr
                    match ValueExpr::from_ssz_bytes(&value_data) {
                        Ok(value_expr) => Ok(Some(value_expr)),
                        Err(_) => Ok(None), // Deserialization failed
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn register_sync(&mut self, id: ValueExprId, definition: ValueExpr) -> Result<()> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                values.write().insert(id, definition);
            }
            ValueStorageBackend::Smt { smt, domain_id } => {
                let value_data = definition.as_ssz_bytes();
                let value_key = format!("{}-value-{}", domain_id.namespace_prefix(), id);
                smt.lock().store_data(&value_key, &value_data)
                    .map_err(|e| anyhow!("Failed to store value: {}", e))?;
            }
        }
        Ok(())
    }

    async fn count(&self) -> Result<usize> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                Ok(values.read().len())
            }
            ValueStorageBackend::Smt { smt: _, domain_id } => {
                // Count values by iterating through keys with the domain prefix
                let _prefix = format!("{}-value-", domain_id.namespace_prefix());
                // For now, return 0 as SMT doesn't expose key iteration
                // In a full implementation, we'd maintain a separate counter
                Ok(0)
            }
        }
    }

    async fn clear(&mut self) -> Result<()> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                values.write().clear();
            }
            ValueStorageBackend::Smt { smt: _, domain_id: _ } => {
                // Implement SMT clear by clearing all values with the domain prefix
                // For now, we'll just return success as SMT doesn't expose bulk operations
                // In a full implementation, we'd iterate through all keys and remove them
                ()
            }
        }
        Ok(())
    }

    // Use the default implementation for is_registered
}
