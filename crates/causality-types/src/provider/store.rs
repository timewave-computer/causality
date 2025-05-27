//! Key-Value Store Provider Interface
//!
//! Defines generic traits for key-value store access, providing a consistent
//! interface for both read-only and mutable storage operations across the system.

//-----------------------------------------------------------------------------
// Store Provider Traits
//-----------------------------------------------------------------------------

use crate::anyhow::Result;
use async_trait::async_trait;
use std::hash::Hash;

/// Generic trait for read-only key-value store access.
#[async_trait]
pub trait AsKeyValueStore<K, V>: Send + Sync
where
    K: Hash + Eq + Send + Sync, // Key must be hashable, equatable, sendable, and syncable
    V: Send + Sync + Clone,
{
    /// Get a value by its key.
    async fn get(&self, key: &K) -> Result<Option<V>>;

    /// Checks if a key exists in the store.
    async fn contains_key(&self, key: &K) -> Result<bool> {
        Ok(self.get(key).await?.is_some())
    }
}

/// Generic trait for a mutable key-value store, extending the read-only store.
#[async_trait]
pub trait AsMutableKeyValueStore<K, V>: AsKeyValueStore<K, V>
// New trait for mutable operation
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync + Clone,
{
    /// Set a value for a given key.
    async fn set(&mut self, key: K, value: V) -> Result<()>;

    /// Delete a value by its key.
    /// Returns the value if it existed, or Ok(None) if not (or Ok(()) depending on desired semantics).
    async fn delete(&mut self, key: &K) -> Result<Option<V>>; // Or Result<()>
}
