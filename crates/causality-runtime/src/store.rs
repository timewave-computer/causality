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
use causality_types::{AsRegistry, ValueExprId, core::id::DomainId, expr::value::ValueExpr, serialization::{Decode, Encode}};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use causality_core::smt::{TegMultiDomainSmt, MemoryBackend};
use hex;

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
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                let value_key = format!("{}-value-{}", "", id);
                if let Ok(Some(value_data)) =  TegMultiDomainSmt::<MemoryBackend>::get_data(&value_key) {
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
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                // Count values by iterating through keys with the domain prefix
                let _prefix = format!("{}-value-", "");
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
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                let value_key = format!("{}-value-{}", "", id);
                TegMultiDomainSmt::<MemoryBackend>::store_data(&value_key, &buf)
                    .map_err(|e| anyhow!("Failed to store value: {}", e))?;
            }
        }
        
        Ok(id)
    }
}

#[async_trait::async_trait]
impl AsRegistry<ValueExpr> for RuntimeValueStore {
    async fn register(
        &mut self,
        key: String, 
        service: ValueExpr, 
    ) -> Result<()> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                let mut id_bytes = [0u8; 32];
                hex::decode_to_slice(&key, &mut id_bytes)
                    .map_err(|e| anyhow!("Invalid key format for ValueExprId (hex decode failed): {}", e))?;
                let id = ValueExprId(id_bytes);
                values.write().insert(id, service);
            }
            ValueStorageBackend::Smt { _smt, _domain_id } => { 
                let value_data = service.as_ssz_bytes();
                TegMultiDomainSmt::<MemoryBackend>::store_data(&key, &value_data)
                    .map_err(|e| anyhow!("Failed to store value in SMT: {}", e))?;
            }
        }
        Ok(())
    }

    async fn unregister(&mut self, key: &str) -> Result<Option<ValueExpr>> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                let mut id_bytes = [0u8; 32];
                hex::decode_to_slice(key, &mut id_bytes)
                    .map_err(|e| anyhow!("Invalid key format for ValueExprId (hex decode failed): {}", e))?;
                let id = ValueExprId(id_bytes);
                Ok(values.write().remove(&id))
            }
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                let existing_data = TegMultiDomainSmt::<MemoryBackend>::get_data(key)
                    .map_err(|e| anyhow!("Failed to access SMT for unregister: {}", e))?;
                
                TegMultiDomainSmt::<MemoryBackend>::store_data(key, &[])
                    .map_err(|e| anyhow!("Failed to remove value from SMT: {}", e))?;

                if let Some(value_data) = existing_data {
                    if value_data.is_empty() { 
                        Ok(None)
                    } else {
                        ValueExpr::from_ssz_bytes(&value_data)
                            .map(Some)
                            .map_err(|e| anyhow!("Failed to deserialize SMT data for unregister: {}", e))
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn lookup(&self, key: &str) -> Result<Option<&ValueExpr>> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                let mut id_bytes = [0u8; 32];
                hex::decode_to_slice(key, &mut id_bytes)
                    .map_err(|e| anyhow!("Invalid key format for ValueExprId (hex decode failed): {}", e))?;
                let id = ValueExprId(id_bytes);
                let guard = values.read();
                if guard.contains_key(&id) {
                    return Err(anyhow!("HashMap lookup returning a stable reference through RwLock is complex and not safely implemented here."));
                } else {
                    Ok(None)
                }
            }
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                Err(anyhow!("Lookup returning a direct reference is not supported for SMT backend."))
            }
        }
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                let keys = values.read().keys().map(|id| hex::encode(id.0)).collect();
                Ok(keys)
            }
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                Ok(Vec::new()) 
            }
        }
    }

    async fn clear(&mut self) -> Result<()> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                values.write().clear();
            }
            ValueStorageBackend::Smt { _smt, _domain_id } => {
                return Err(anyhow!("Clear operation not fully implemented for SMT backend."));
            }
        }
        Ok(())
    }
}
