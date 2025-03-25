// Log storage system
// Original file: src/log/storage.rs

// Storage module for Causality Unified Log System
//
// This module provides functionality for storing and retrieving log entries
// from various storage backends (file, memory, etc.).

pub mod memory_storage;
pub mod file_storage;

pub use memory_storage::MemoryLogStorage;
pub use file_storage::FileLogStorage;

use std::fmt;
use std::path::Path;
use serde::{Serialize, Deserialize};

use causality_types::{Result, Error};
use causality_engine::LogEntry;

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

/// Log storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// The maximum size of a segment in bytes
    pub max_segment_size: u64,
    /// The maximum number of entries in a segment
    pub max_segment_entries: usize,
    /// Whether to compress segments
    pub compress: bool,
    /// The compression algorithm to use
    pub compression_algorithm: CompressionAlgorithm,
    /// The storage format
    pub format: StorageFormat,
    /// Whether to sync to disk after each write
    pub sync_on_write: bool,
    /// The maximum number of segments to keep in memory
    pub in_memory_segments: usize,
    /// The directory to store segments
    #[serde(skip)]
    pub storage_dir: Option<std::path::PathBuf>,
    /// Whether to verify hashes when reading entries
    pub verify_hashes: bool,
    /// Whether to enforce hash verification when writing
    pub enforce_hash_verification: bool,
    /// Whether to automatically rotate segments
    pub auto_rotate: bool,
    /// Whether to compact segments on close
    pub compact_on_close: bool,
    /// Maximum segment compaction ratio (0.0-1.0)
    /// Segments with more than this ratio of deleted entries will be compacted
    pub compaction_threshold: f32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_segment_size: 1024 * 1024 * 10, // 10MB
            max_segment_entries: 10000,
            compress: false,
            compression_algorithm: CompressionAlgorithm::None,
            format: StorageFormat::Binary,
            sync_on_write: true,
            in_memory_segments: 2,
            storage_dir: None,
            verify_hashes: true,
            enforce_hash_verification: true,
            auto_rotate: true,
            compact_on_close: false,
            compaction_threshold: 0.3, // Compact if 30% or more entries are deleted
        }
    }
}

impl StorageConfig {
    /// Create a new storage configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the maximum segment size
    pub fn with_max_segment_size(mut self, size: u64) -> Self {
        self.max_segment_size = size;
        self
    }
    
    /// Set the maximum number of entries in a segment
    pub fn with_max_segment_entries(mut self, entries: usize) -> Self {
        self.max_segment_entries = entries;
        self
    }
    
    /// Set compression
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
    
    /// Set the compression algorithm
    pub fn with_compression_algorithm(mut self, algorithm: CompressionAlgorithm) -> Self {
        self.compression_algorithm = algorithm;
        self
    }
    
    /// Set the storage format
    pub fn with_format(mut self, format: StorageFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set sync on write
    pub fn with_sync_on_write(mut self, sync: bool) -> Self {
        self.sync_on_write = sync;
        self
    }
    
    /// Set the maximum number of segments to keep in memory
    pub fn with_in_memory_segments(mut self, segments: usize) -> Self {
        self.in_memory_segments = segments;
        self
    }
    
    /// Set the storage directory
    pub fn with_storage_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.storage_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    
    /// Set whether to verify hashes when reading entries
    pub fn with_verify_hashes(mut self, verify: bool) -> Self {
        self.verify_hashes = verify;
        self
    }
    
    /// Set whether to enforce hash verification when writing
    pub fn with_enforce_hash_verification(mut self, enforce: bool) -> Self {
        self.enforce_hash_verification = enforce;
        self
    }
    
    /// Set whether to automatically rotate segments
    pub fn with_auto_rotate(mut self, auto_rotate: bool) -> Self {
        self.auto_rotate = auto_rotate;
        self
    }
    
    /// Set whether to compact segments on close
    pub fn with_compact_on_close(mut self, compact: bool) -> Self {
        self.compact_on_close = compact;
        self
    }
    
    /// Set the compaction threshold
    pub fn with_compaction_threshold(mut self, threshold: f32) -> Self {
        self.compaction_threshold = threshold.clamp(0.0, 1.0);
        self
    }
}

/// Interface for log storage implementations
pub trait LogStorage: Send + Sync {
    /// Append an entry to the log
    fn append(&self, entry: LogEntry) -> Result<()>;
    
    /// Append multiple entries to the log in a batch
    fn append_batch(&self, entries: Vec<LogEntry>) -> Result<()> {
        for entry in entries {
            self.append(entry)?;
        }
        Ok(())
    }
    
    /// Alias for append, for backward compatibility
    fn append_entry(&self, entry: &LogEntry) -> Result<()> {
        self.append(entry.clone())
    }
    
    /// Read entries from the log
    fn read(&self, start: usize, count: usize) -> Result<Vec<LogEntry>>;
    
    /// Read entries within a specific time range
    fn read_time_range(&self, start_time: u64, end_time: u64) -> Result<Vec<LogEntry>> {
        // Default implementation - inefficient but works
        let total = self.entry_count()?;
        let all_entries = self.read(0, total)?;
        
        Ok(all_entries
            .into_iter()
            .filter(|e| e.timestamp >= start_time && e.timestamp <= end_time)
            .collect())
    }
    
    /// Alias for read, for backward compatibility
    fn read_entries(&self, start: usize, count: usize) -> Result<Vec<LogEntry>> {
        self.read(start, count)
    }
    
    /// Get the number of entries in the log
    fn entry_count(&self) -> Result<usize>;
    
    /// Get an entry by its ID
    fn get_entry_by_id(&self, id: &str) -> Result<Option<LogEntry>> {
        // Default implementation - inefficient but works
        // Storage implementations should override this for better performance
        let total = self.entry_count()?;
        let batch_size = 100;
        let mut offset = 0;
        
        while offset < total {
            let entries = self.read(offset, batch_size)?;
            if entries.is_empty() {
                break;
            }
            
            for entry in &entries {
                if entry.id == id {
                    return Ok(Some(entry.clone()));
                }
            }
            
            offset += entries.len();
        }
        
        Ok(None)
    }
    
    /// Get entries by their trace ID
    fn get_entries_by_trace(&self, trace_id: &str) -> Result<Vec<LogEntry>> {
        // Default implementation - inefficient but works
        // Storage implementations should override this for better performance
        let total = self.entry_count()?;
        let batch_size = 100;
        let mut offset = 0;
        let mut result = Vec::new();
        
        while offset < total {
            let entries = self.read(offset, batch_size)?;
            if entries.is_empty() {
                break;
            }
            
            for entry in entries {
                if let Some(entry_trace_id) = &entry.trace_id {
                    if entry_trace_id == trace_id {
                        result.push(entry);
                    }
                }
            }
            
            offset += batch_size;
        }
        
        Ok(result)
    }
    
    /// Rotate log segments
    fn rotate(&self) -> Result<()> {
        // Default implementation does nothing
        // File-based storage should override this
        Ok(())
    }
    
    /// Compact the log storage by removing deleted entries
    fn compact(&self) -> Result<()> {
        // Default implementation does nothing
        // File-based storage should override this
        Ok(())
    }
    
    /// Flush any pending writes to storage
    fn flush(&self) -> Result<()>;
    
    /// Close the storage
    fn close(&self) -> Result<()>;
    
    /// Verify that an entry's hash is valid before storing it
    fn verify_entry_hash(&self, entry: &LogEntry, config: &StorageConfig) -> Result<()> {
        // If hash verification is disabled, skip the check
        if !config.enforce_hash_verification {
            return Ok(());
        }
        
        // If the entry has a hash, verify it
        if let Some(hash) = &entry.entry_hash {
            if !entry.verify_hash() {
                return Err(Error::InvalidHash(format!(
                    "Invalid hash for log entry {}. Hash verification failed.", 
                    entry.id
                )));
            }
        } else {
            // If the entry doesn't have a hash and verification is enforced, reject it
            return Err(Error::InvalidHash(format!(
                "Missing hash for log entry {}. Hash required when verification is enforced.", 
                entry.id
            )));
        }
        
        Ok(())
    }
    
    /// Ensure an entry has a valid hash, generating one if needed
    fn ensure_valid_hash(&self, entry: &mut LogEntry) -> Result<()> {
        if entry.entry_hash.is_none() || !entry.verify_hash() {
            entry.generate_hash();
        }
        Ok(())
    }
} 