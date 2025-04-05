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
    /// Append an entry to the log
    async fn append_entry(&self, entry: LogEntry) -> CausalityResult<()>;
    
    /// Get all entries in the log
    async fn get_all_entries(&self) -> CausalityResult<Vec<LogEntry>>;
    
    /// Get entries in a specific range
    async fn get_entries(&self, start: usize, end: usize) -> CausalityResult<Vec<LogEntry>>;
    
    /// Get the total number of entries in the log
    async fn get_entry_count(&self) -> CausalityResult<usize>;
    
    /// Clear all entries from the log
    async fn clear(&self) -> CausalityResult<()>;
    
    /// Append a new entry to the log
    fn append(&self, entry: LogEntry) -> EngineResult<()>;
    
    /// Append multiple entries to the log
    fn append_batch(&self, entries: Vec<LogEntry>) -> EngineResult<()>;
    
    /// Read entries from the log
    fn read(&self, start: usize, count: usize) -> EngineResult<Vec<LogEntry>>;
    
    /// Read entries within a time range
    fn read_time_range(&self, start_time: u64, end_time: u64) -> EngineResult<Vec<LogEntry>>;
    
    /// Get an entry by ID
    fn get_entry_by_id(&self, id: &str) -> EngineResult<Option<LogEntry>>;
    
    /// Get entries by trace ID
    fn get_entries_by_trace(&self, trace_id: &str) -> EngineResult<Vec<LogEntry>>;
    
    /// Get an entry by hash
    fn get_entry_by_hash(&self, hash: &str) -> EngineResult<Option<LogEntry>>;
    
    /// Find entries by type
    fn find_entries_by_type(&self, entry_type: EntryType) -> EngineResult<Vec<LogEntry>>;
    
    /// Find entries within a time range
    fn find_entries_in_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> EngineResult<Vec<LogEntry>>;
    
    /// Rotate the log
    fn rotate(&self) -> EngineResult<()>;
    
    /// Compact the log
    fn compact(&self) -> EngineResult<()>;
    
    /// Flush any pending writes
    fn flush(&self) -> EngineResult<()>;
    
    /// Close the storage
    fn close(&self) -> EngineResult<()>;
} 