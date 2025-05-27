//! Registry Provider Interface
//!
//! Defines a unified Registry trait for registering and retrieving
//! definitions by identifier. This provides a consistent interface
//! for various registries throughout the system.

//-----------------------------------------------------------------------------
// Registry Provider Trait
//-----------------------------------------------------------------------------
use crate::anyhow::Result;
use async_trait::async_trait;
use std::hash::Hash;

/// Unified generic trait for registering and retrieving definitions by ID.
/// Combines sync and async operations with bulk operations support.
#[async_trait]
pub trait AsRegistry<Id, Definition>: Send + Sync
where
    Id: Hash + Eq + Send + Sync, // Identifier must be hashable, equatable, sendable, and syncable
    Definition: Send + Sync + Clone,
{
    /// Registers a definition with a given ID.
    /// Returns an error if registration fails (e.g., ID already exists and updates aren't allowed).
    async fn register(&mut self, id: Id, definition: Definition) -> Result<()>;

    /// Retrieves a clone of the definition by its ID.
    /// Returns `Ok(Some(Definition))` if found, `Ok(None)` if not found, or an error.
    async fn get(&self, id: &Id) -> Result<Option<Definition>>;

    /// Unregisters a definition by its identifier.
    /// Returns `Ok(())` whether the ID existed or not, or an error on failure.
    async fn unregister(&mut self, id: &Id) -> Result<()>;

    /// Checks if an identifier is registered.
    async fn is_registered(&self, id: &Id) -> Result<bool> {
        Ok(self.get(id).await?.is_some())
    }

    /// Synchronous version of get for non-async contexts
    fn get_sync(&self, id: &Id) -> Result<Option<Definition>>;

    /// Synchronous version of register for non-async contexts  
    fn register_sync(&mut self, id: Id, definition: Definition) -> Result<()>;

    /// Remove a definition by its ID (alias for unregister)
    async fn remove(&mut self, id: &Id) -> Result<Option<Definition>> {
        let existing = self.get(id).await?;
        self.unregister(id).await?;
        Ok(existing)
    }

    /// Check if an ID exists in the registry (alias for is_registered)
    async fn contains(&self, id: &Id) -> Result<bool> {
        self.is_registered(id).await
    }

    /// Count of entries in the registry
    async fn count(&self) -> Result<usize>;

    /// Clear all entries from the registry
    async fn clear(&mut self) -> Result<()>;
}

/// Extended registry trait with additional bulk operations capabilities.
/// Adds bulk operations and iteration to the base registry trait.
#[async_trait]
pub trait AsExtendedRegistry<Id, Definition>: AsRegistry<Id, Definition>
where
    Id: Hash + Eq + Send + Sync,
    Definition: Send + Sync + Clone,
{
    /// Insert multiple key-value pairs. Returns error on first conflict.
    async fn register_many(&mut self, entries: impl Iterator<Item = (Id, Definition)> + Send) -> Result<()>;

    /// Get multiple values by their keys
    async fn get_many(&self, ids: impl Iterator<Item = Id> + Send) -> Result<Vec<Option<Definition>>>;

    /// Get all registered IDs
    async fn get_all_ids(&self) -> Result<Vec<Id>>;

    /// Get all definitions
    async fn get_all_definitions(&self) -> Result<Vec<Definition>>;
}
