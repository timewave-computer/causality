// Log storage system
// Original file: src/log/storage.rs

// Storage module for Causality Unified Log System
//
// This module provides functionality for storing and retrieving log entries
// from various storage backends (file, memory, etc.).

// These modules are directly in the log directory, not in storage subdir
// pub mod memory_storage;
// pub mod file_storage;

// pub use memory_storage::MemoryLogStorage;
// pub use file_storage::FileLogStorage;

use std::fmt;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fmt::Debug;
use async_trait::async_trait;

use causality_error::{EngineResult, Result as CausalityResult};
use causality_types::DomainId;
use chrono::{DateTime, Utc};
use crate::log::LogEntry;
use crate::log::entry::EntryType;

/// The format of log storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageFormat {
    /// JSON storage (one entry per line)
    Json,
    /// Binary storage (using bincode)
    Binary,
    /// CBOR storage (more compact than JSON)
    Cbor,
}

impl fmt::Display for StorageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageFormat::Json => write!(f, "json"),
            StorageFormat::Binary => write!(f, "binary"),
            StorageFormat::Cbor => write!(f, "cbor"),
        }
    }
}

/// Compression algorithm used for log segments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// LZ4 compression
    Lz4,
    /// Zstd compression
    Zstd,
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionAlgorithm::None => write!(f, "none"),
            CompressionAlgorithm::Gzip => write!(f, "gzip"),
            CompressionAlgorithm::Lz4 => write!(f, "lz4"),
            CompressionAlgorithm::Zstd => write!(f, "zstd"),
        }
    }
}

/// Configuration for log storage
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// The base directory for log storage
    pub base_dir: PathBuf,
    /// The domain ID for this log
    pub domain_id: DomainId,
    /// Format to use for storage
    pub format: StorageFormat,
    /// Whether to enforce hash verification
    pub enforce_hash_verification: bool,
    /// Whether to automatically calculate hashes
    pub auto_hash: bool,
    /// The maximum number of entries per segment
    pub max_entries_per_segment: usize,
    /// The maximum segment size in bytes
    pub max_segment_size: usize,
    /// The path to store segments
    pub segment_path: PathBuf,
    /// The compression level
    pub compression_level: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            format: StorageFormat::Binary,
            enforce_hash_verification: false,
            auto_hash: true,
            max_entries_per_segment: 1000,
            max_segment_size: 1024 * 1024 * 1024, // 1GB
            segment_path: PathBuf::from("logs"),
            compression_level: 6,
            base_dir: PathBuf::new(),
            domain_id: DomainId::new("default"),
        }
    }
}

/// Interface for log storage implementations
#[async_trait]
pub trait LogStorage: Send + Sync + Debug {
    /// Get the total number of entries in the log
    fn entry_count(&self) -> CausalityResult<usize>;
    
    /// Read entries from the log
    fn read(&self, offset: usize, limit: usize) -> CausalityResult<Vec<LogEntry>>;
    
    /// Append a new entry to the log
    fn append(&self, entry: LogEntry) -> CausalityResult<()>;
    
    /// Get an entry by ID
    fn get_entry_by_id(&self, id: &str) -> CausalityResult<Option<LogEntry>>;
    
    /// Get entries by trace ID
    fn get_entries_by_trace(&self, trace_id: &str) -> CausalityResult<Vec<LogEntry>>;

    /// Find entries by type
    fn find_entries_by_type(&self, entry_type: EntryType) -> CausalityResult<Vec<LogEntry>> {
        let all_entries = self.read(0, self.entry_count()?)?;
        Ok(all_entries.into_iter()
            .filter(|entry| entry.entry_type == entry_type)
            .collect())
    }
    
    /// Read entries within a time range
    fn read_time_range(&self, start_time: u64, end_time: u64) -> CausalityResult<Vec<LogEntry>> {
        let all_entries = self.read(0, self.entry_count()?)?;
        Ok(all_entries.into_iter()
            .filter(|entry| {
                let ts = entry.timestamp.to_millis();
                ts >= start_time && ts <= end_time
            })
            .collect())
    }
    
    /// Append multiple entries to the log
    fn append_batch(&self, entries: Vec<LogEntry>) -> CausalityResult<()> {
        for entry in entries {
            self.append(entry)?;
        }
        Ok(())
    }
    
    /// Rotate the log
    fn rotate(&self) -> CausalityResult<()> {
        Ok(()) // Default no-op implementation
    }
    
    /// Compact the log
    fn compact(&self) -> CausalityResult<()> {
        Ok(()) // Default no-op implementation
    }
    
    /// Flush any pending writes
    fn flush(&self) -> CausalityResult<()> {
        Ok(()) // Default no-op implementation
    }
    
    /// Close the storage
    fn close(&self) -> CausalityResult<()> {
        Ok(()) // Default no-op implementation
    }
    
    // Async methods
    
    /// Asynchronously flush any pending writes
    async fn async_flush(&self) -> CausalityResult<()> {
        Ok(self.flush()?)
    }
    
    /// Append an entry to the log asynchronously
    async fn append_entry(&self, entry: LogEntry) -> CausalityResult<()> {
        Ok(self.append(entry)?)
    }
    
    /// Get all entries in the log asynchronously
    async fn get_all_entries(&self) -> CausalityResult<Vec<LogEntry>> {
        Ok(self.read(0, self.entry_count()?)?)
    }
    
    /// Get entries in a specific range asynchronously
    async fn get_entries(&self, start: usize, end: usize) -> CausalityResult<Vec<LogEntry>> {
        let limit = if end > start { end - start } else { 0 };
        Ok(self.read(start, limit)?)
    }
    
    /// Get the total number of entries in the log asynchronously
    async fn get_entry_count(&self) -> CausalityResult<usize> {
        Ok(self.entry_count()?)
    }
    
    /// Clear all entries from the log asynchronously
    async fn clear(&self) -> CausalityResult<()> {
        // Default implementation - can be overridden by implementations
        Ok(())
    }
} 