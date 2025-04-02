// Log system for the Causality Engine
//
// This module provides functionality for logging and tracking facts,
// effects, and events in the system.

// Core modules
pub mod types;
pub mod fact;
pub mod effect_tracker;
pub mod memory_storage;

// Re-export core types
pub use types::{LogEntry, EntryType, EntryData, FactEntry, EffectEntry, SystemEventEntry, OperationEntry};
pub use fact::{FactId, FactSnapshot, FactDependency, FactDependencyType};
pub use memory_storage::MemoryLogStorage;

/// Trait for log storage
pub trait LogStorage: Send + Sync {
    /// Get the number of entries in the storage
    fn entry_count(&self) -> causality_error::Result<usize>;
    
    /// Read a batch of entries starting from the given offset
    fn read(&self, offset: usize, limit: usize) -> causality_error::Result<Vec<LogEntry>>;
    
    /// Append an entry to the storage
    fn append(&self, entry: LogEntry) -> causality_error::Result<()>;
}

// Legacy modules - these may be deprecated in future versions
// TODO: Consider migrating to new module structure
pub mod entry {
    pub use super::types::{LogEntry, EntryType, EntryData};
}
pub mod storage;
pub mod fact_snapshot;
pub mod file_storage;
pub mod filter;
pub mod fact_types;
pub mod event;
pub mod replay;
pub mod fact_replay;
pub mod performance;
pub mod visualization;
pub mod segment;
pub mod segment_manager;
pub mod sync;

// Test utilities
#[cfg(test)]
pub mod test_utils;

// Specific entry types - consider migrating to types module
pub mod fact_entry;
pub mod effect_entry;
pub mod event_entry; 