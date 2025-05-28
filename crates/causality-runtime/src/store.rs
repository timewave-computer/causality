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
use causality_core::smt::{TegMultiDomainSmt, MemoryBackend};
use causality_types::{
    expression::value::ValueExpr,
    primitive::ids::{DomainId, ValueExprId},
    system::{
        serialization::Encode,
        provider::AsRegistry,
    },
};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
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

/// Simple registry implementation for ValueExpr that owns its data directly
#[derive(Debug, Default)]
pub struct SimpleValueRegistry {
    values: HashMap<String, ValueExpr>,
}

impl SimpleValueRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl AsRegistry<ValueExpr> for SimpleValueRegistry {
    async fn register(&mut self, key: String, service: ValueExpr) -> Result<()> {
        self.values.insert(key, service);
        Ok(())
    }

    async fn unregister(&mut self, key: &str) -> Result<Option<ValueExpr>> {
        Ok(self.values.remove(key))
    }

    async fn lookup(&self, key: &str) -> Result<Option<&ValueExpr>> {
        Ok(self.values.get(key))
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.values.keys().cloned().collect())
    }

    async fn clear(&mut self) -> Result<()> {
        self.values.clear();
        Ok(())
    }
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
            ValueStorageBackend::Smt { smt, domain_id: _ } => {
                let value_key = format!("{}-value-{}", "", id);
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
            ValueStorageBackend::Smt { smt: _, domain_id: _ } => {
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
            ValueStorageBackend::Smt { smt, domain_id: _ } => {
                let value_key = format!("{}-value-{}", "", id);
                smt.lock().store_data(&value_key, &buf)
                    .map_err(|e| anyhow!("Failed to store value: {}", e))?;
            }
        }
        
        Ok(id)
    }

    /// Lookup a value by key. Returns an owned copy of the value if found.
    pub async fn lookup(&self, key: &str) -> Result<Option<ValueExpr>> {
        match &self.storage {
            ValueStorageBackend::HashMap(values) => {
                let mut id_bytes = [0u8; 32];
                hex::decode_to_slice(key, &mut id_bytes)
                    .map_err(|e| anyhow!("Invalid key format for ValueExprId (hex decode failed): {}", e))?;
                let id = ValueExprId(id_bytes);
                let guard = values.read();
                Ok(guard.get(&id).cloned())
            }
            ValueStorageBackend::Smt { smt: _, domain_id: _ } => {
                // SMT lookup to return an owned ValueExpr would require deserialization.
                Err(anyhow!("Lookup returning an owned ValueExpr is not yet fully implemented for SMT backend."))
            }
        }
    }

    /// Unregister (remove) a value by key. Returns the removed value if found.
    pub async fn unregister(&mut self, key: &str) -> Result<Option<ValueExpr>> {
        match &mut self.storage {
            ValueStorageBackend::HashMap(values) => {
                let mut id_bytes = [0u8; 32];
                hex::decode_to_slice(key, &mut id_bytes)
                    .map_err(|e| anyhow!("Invalid key format for ValueExprId (hex decode failed): {}", e))?;
                let id = ValueExprId(id_bytes);
                let mut guard = values.write();
                Ok(guard.remove(&id))
            }
            ValueStorageBackend::Smt { smt: _, domain_id: _ } => {
                // SMT unregister would require deleting the value.
                Err(anyhow!("Unregister is not yet fully implemented for SMT backend."))
            }
        }
    }
}

// Note: RuntimeValueStore does not implement AsRegistry due to lifetime issues
// with returning references from Arc<RwLock<HashMap>>. Use SimpleValueRegistry instead
// for AsRegistry functionality.
