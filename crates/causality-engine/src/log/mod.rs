// Log system for the Causality Engine
//
// This module provides functionality for logging and tracking facts,
// effects, and events in the system.

use async_trait::async_trait;

// Core modules
pub mod types;
pub mod fact;
pub mod effect_tracker;
pub mod memory_storage;

// Re-export core types
pub use types::{LogEntry, EntryType, EntryData, FactEntry, EffectEntry, SystemEventEntry, OperationEntry};
pub use event_entry::{EventEntry, EventSeverity};
pub use fact::{FactId, FactSnapshot, FactDependency, FactDependencyType};
pub use memory_storage::MemoryLogStorage;

/// Trait for log storage
#[async_trait]
pub trait LogStorage: Send + Sync {
    /// Get the number of entries in the storage (synchronous version)
    fn entry_count(&self) -> causality_error::Result<usize>;
    
    /// Read a batch of entries starting from the given offset (synchronous version)
    fn read(&self, offset: usize, limit: usize) -> causality_error::Result<Vec<LogEntry>>;
    
    /// Append an entry to the storage (synchronous version)
    fn append(&self, entry: LogEntry) -> causality_error::Result<()>;

    /// Append a batch of entries (synchronous version)
    fn append_batch(&self, entries: Vec<LogEntry>) -> causality_error::Result<()> {
        for entry in entries {
            self.append(entry)?;
        }
        Ok(())
    }

    /// Read entries in a time range (synchronous version) 
    fn read_time_range(&self, start_time: u64, end_time: u64) -> causality_error::Result<Vec<LogEntry>> {
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .filter(|e| {
                let ts = e.timestamp.to_millis();
                ts >= start_time && ts <= end_time
            })
            .collect())
    }

    /// Find entries by trace ID (synchronous version)
    fn find_entries_by_trace_id(&self, trace_id: &causality_types::TraceId) -> causality_error::Result<Vec<LogEntry>> {
        // Default implementation: filter all entries
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .filter(|e| {
                e.trace_id.as_ref().map_or(false, |t| t == trace_id)
            })
            .collect())
    }

    /// Get an entry by ID (synchronous version)
    fn get_entry_by_id(&self, id: &str) -> causality_error::Result<Option<LogEntry>> {
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .find(|e| e.id == id))
    }

    /// Get an entry by hash (synchronous version)
    fn get_entry_by_hash(&self, hash: &str) -> causality_error::Result<Option<LogEntry>> {
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .find(|e| e.entry_hash.as_ref().map_or(false, |h| h == hash)))
    }

    /// Get entries by trace ID (synchronous version)
    fn get_entries_by_trace(&self, trace_id: &str) -> causality_error::Result<Vec<LogEntry>> {
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .filter(|e| e.trace_id.as_ref().map_or(false, |t| t.as_str() == trace_id))
            .collect())
    }

    /// Find entries by type (synchronous version)
    fn find_entries_by_type(&self, entry_type: EntryType) -> causality_error::Result<Vec<LogEntry>> {
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .filter(|e| e.entry_type == entry_type)
            .collect())
    }

    /// Find entries in time range (synchronous version)
    fn find_entries_in_time_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> causality_error::Result<Vec<LogEntry>> {
        let entries = self.read(0, self.entry_count()?)?;
        Ok(entries.into_iter()
            .filter(|e| {
                let ts = crate::log::time_utils::timestamp_to_datetime(e.timestamp.clone());
                ts >= start && ts <= end
            })
            .collect())
    }

    /// Rotate the log (e.g., for log rotation)
    fn rotate(&self) -> causality_error::Result<()> {
        Ok(()) // Default no-op implementation
    }

    /// Compact the log (e.g., for removing duplicate entries)
    fn compact(&self) -> causality_error::Result<()> {
        Ok(()) // Default no-op implementation
    }

    /// Close the log
    fn close(&self) -> causality_error::Result<()> {
        Ok(()) // Default no-op implementation
    }

    // Async versions of the methods - for implementors that support async operations

    /// Get the number of entries in the storage (async version)
    async fn get_entry_count(&self) -> causality_error::Result<usize> {
        // Default implementation falls back to synchronous version
        self.entry_count()
    }

    /// Get all entries in the storage (async version)
    async fn get_all_entries(&self) -> causality_error::Result<Vec<LogEntry>> {
        // Default implementation falls back to synchronous version
        self.read(0, self.entry_count()?)
    }

    /// Get a range of entries from the storage (async version)
    async fn get_entries(&self, start: usize, end: usize) -> causality_error::Result<Vec<LogEntry>> {
        // Default implementation falls back to synchronous version
        self.read(start, end - start)
    }

    /// Append an entry to the storage (async version)
    async fn append_entry(&self, entry: LogEntry) -> causality_error::Result<()> {
        // Default implementation falls back to synchronous version
        self.append(entry)
    }

    /// Append a batch of entries (async version)
    async fn append_entries_batch(&self, entries: Vec<LogEntry>) -> causality_error::Result<()> {
        // Default implementation falls back to synchronous version
        for entry in entries {
            self.append(entry)?;
        }
        Ok(())
    }

    /// Find entries by type (async version)
    async fn find_entries_by_type_async(&self, entry_type: EntryType) -> causality_error::Result<Vec<LogEntry>> {
        // Default implementation falls back to synchronous version
        self.find_entries_by_type(entry_type)
    }

    /// Find entries in time range (async version)
    async fn find_entries_in_time_range_async(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> causality_error::Result<Vec<LogEntry>> {
        // Default implementation falls back to synchronous version
        self.find_entries_in_time_range(start, end)
    }

    /// Flush any pending operations to the storage (async version)
    async fn async_flush(&self) -> causality_error::Result<()> {
        // Default implementation - no-op, just return success
        Ok(())
    }

    /// Clear all entries (async version)
    async fn clear(&self) -> causality_error::Result<()> {
        // Default implementation - no-op
        Ok(())
    }
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
pub mod time_utils;

// Test utilities
#[cfg(test)]
pub mod test_utils;

// Test modules
#[cfg(test)]
pub mod tests;

// Specific entry types - consider migrating to types module
pub mod fact_entry;
pub mod effect_entry;
pub mod event_entry; 