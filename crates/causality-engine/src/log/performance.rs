// Performance optimizations for the log system
// Original file: src/log/performance.rs

// Log System Performance Optimizations
//
// This module provides performance enhancements for the log system including:
// - Batched writes for efficiency
// - Compression for log segments
// - Indexing for faster queries

use std::{ 
    collections::HashMap,
    fmt::{self, Debug},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::executor::block_on;
use serde::{Serialize, Deserialize};
use tokio::runtime::Runtime;
use tokio::time::sleep;

use causality_error::{EngineError, EngineResult, Result as CausalityResult, CausalityError};
use causality_types::Timestamp;
use crate::log::types::{LogEntry, EntryType};
use crate::log::LogStorage;
use crate::log::segment::LogSegment;

/// Configuration for log flushing
pub struct FlushConfig {
    /// Maximum number of entries to buffer before writing
    pub max_entries: usize,
    /// Maximum time to wait before flushing (in milliseconds)
    pub flush_interval_ms: u64,
    /// Whether to flush on every write
    pub flush_on_write: bool,
}

impl Default for FlushConfig {
    fn default() -> Self {
        FlushConfig {
            max_entries: 100,
            flush_interval_ms: 1000,
            flush_on_write: false,
        }
    }
}

/// Configuration for batch operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of entries to buffer before writing
    pub max_batch_size: usize,
    /// Maximum time to wait before flushing the buffer (in milliseconds)
    pub flush_interval_ms: u64,
    /// Whether to compress batches when writing to storage
    pub compress_batches: bool,
    /// Compression level (0-9, where 0 is no compression and 9 is maximum)
    pub compression_level: u32,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            max_batch_size: 1000,
            flush_interval_ms: 500,
            compress_batches: true,
            compression_level: 6,
        }
    }
}

/// Configuration for optimized storage
#[derive(Debug, Clone, Default)]
pub struct OptimizedStorageConfig {
    pub batch_config: BatchConfig,
    // Add other optimization fields here if needed in the future
    // pub enable_background_flush: bool,
    // pub flush_interval: Duration,
}

/// Batch writer for efficiently writing entries to log storage
pub struct BatchWriter<S: LogStorage> {
    /// The underlying storage
    storage: Arc<S>,
    /// Configuration for batch operations
    config: BatchConfig,
    /// Buffered entries waiting to be written
    buffer: Mutex<Vec<LogEntry>>,
    /// Last time the buffer was flushed
    last_flush: Mutex<Instant>,
}

impl<S: LogStorage> BatchWriter<S> {
    /// Create a new batch writer with the given storage and configuration
    pub fn new(storage: Arc<S>, config: BatchConfig) -> Self {
        let max_batch_size = config.max_batch_size;
        BatchWriter {
            storage,
            config,
            buffer: Mutex::new(Vec::with_capacity(max_batch_size)),
            last_flush: Mutex::new(Instant::now()),
        }
    }
    
    /// Add an entry to the batch
    pub fn add_entry(&self, entry: LogEntry) -> Result<(), EngineError> {
        let mut buffer = self.buffer.lock().map_err(|e| EngineError::SyncError(e.to_string()))?;
        buffer.push(entry);
        
        let should_flush_size = buffer.len() >= self.config.max_batch_size;
        drop(buffer);

        if should_flush_size {
            let mut buffer_lock = self.buffer.lock().map_err(|e| EngineError::SyncError(e.to_string()))?;
            self.flush_internal(&mut buffer_lock)?;
            return Ok(());
        }
        
        let mut last_flush_lock = self.last_flush.lock().map_err(|e| EngineError::SyncError(e.to_string()))?;
        if last_flush_lock.elapsed() >= Duration::from_millis(self.config.flush_interval_ms) {
             let mut buffer_lock = self.buffer.lock().map_err(|e| EngineError::SyncError(e.to_string()))?;
             if !buffer_lock.is_empty() {
                 self.flush_internal(&mut buffer_lock)?;
                 *last_flush_lock = Instant::now();
             }
        }
        
        Ok(())
    }
    
    /// Flush all buffered entries to storage
    pub fn flush(&self) -> Result<(), EngineError> {
        let mut buffer = self.buffer.lock().map_err(|e| EngineError::SyncError(e.to_string()))?;
        if !buffer.is_empty() {
            self.flush_internal(&mut buffer)?;
            *self.last_flush.lock().map_err(|e| EngineError::SyncError(e.to_string()))? = Instant::now();
        }
        Ok(())
    }
    
    /// Internal flush implementation that takes a pre-locked buffer
    fn flush_internal(&self, buffer: &mut Vec<LogEntry>) -> Result<(), EngineError> {
        if buffer.is_empty() {
            return Ok(());
        }
        
        let entries = std::mem::take(buffer);
        
        // Apply compression if enabled
        let entries_to_write = if self.config.compress_batches {
            self.compress_entries(&entries)?
        } else {
            entries
        };
        
        // Use sync append_batch from the trait
        self.storage.append_batch(entries_to_write).map_err(|e| EngineError::LogError(e.to_string()))?; 
        
        Ok(())
    }
    
    /// Compress a batch of entries
    fn compress_entries(&self, entries: &[LogEntry]) -> Result<Vec<LogEntry>, EngineError> {
        // In a real implementation, this would compress entry data while preserving metadata
        // For this example, we'll just clone the entries (actual compression would be implemented here)
        Ok(entries.to_vec())
    }
}

impl<S: LogStorage> std::fmt::Debug for BatchWriter<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchWriter")
            .field("config", &self.config)
            .field("buffer_size", &self.buffer.lock().map(|b| b.len()).unwrap_or(0))
            .finish()
    }
}

/// Configuration for the background flusher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundFlusherConfig {
    /// Interval for flushing
    pub flush_interval: Duration,
    /// Whether the flusher is enabled
    pub enabled: bool,
}

impl Default for BackgroundFlusherConfig {
    fn default() -> Self {
        Self {
            flush_interval: Duration::from_secs(5),
            enabled: true,
        }
    }
}

/// Background task for periodically flushing log entries
struct BackgroundFlusher<S: LogStorage + 'static> {
    /// Shared writer instance
    writer: Arc<BatchWriter<S>>,
    /// Configuration for the flusher
    config: BackgroundFlusherConfig,
    /// Signal to stop the flusher
    stop_signal: Arc<AtomicBool>,
    /// Handle for the background thread
    handle: Option<thread::JoinHandle<()>>,
    /// Tokio runtime for async operations
    runtime: Arc<Runtime>,
}

impl<S: LogStorage + 'static> Debug for BackgroundFlusher<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackgroundFlusher")
            .field("config", &self.config)
            .field("running", &self.handle.is_some())
            .finish()
    }
}

impl<S: LogStorage + 'static> BackgroundFlusher<S> {
    /// Create and start a new background flusher
    fn start(
        writer: Arc<BatchWriter<S>>,
        config: BackgroundFlusherConfig,
        runtime: Arc<Runtime>,
    ) -> Self {
        let stop_signal = Arc::new(AtomicBool::new(false));
        let writer_for_thread = writer.clone();
        let flusher_config = config.clone();
        let flusher_stop_signal = stop_signal.clone();
        let flusher_runtime = runtime.clone();

        let handle = if config.enabled {
            // Clone the Arc for the final flush outside the async block
            let final_flush_writer = writer_for_thread.clone(); 
            Some(thread::spawn(move || {
                let _guard = flusher_runtime.enter();
                flusher_runtime.block_on(async move {
                    loop {
                        sleep(flusher_config.flush_interval).await;
                        if flusher_stop_signal.load(Ordering::Relaxed) {
                            break;
                        }
                        if let Err(e) = crate::log::LogStorage::async_flush(&*writer_for_thread.storage).await {
                             eprintln!("BackgroundFlusher: Error during flush: {}", e);
                        }
                    }
                });
                // Use the newly cloned Arc here
                if let Err(e) = futures::executor::block_on(crate::log::LogStorage::async_flush(&*final_flush_writer.storage)) {
                    eprintln!("BackgroundFlusher: Error during final flush: {}", e);
                }
            }))
        } else {
            None
        };
        Self { writer, config, stop_signal, handle, runtime }
    }

    /// Stop the background flusher
    fn stop(mut self) {
        if let Some(handle) = self.handle.take() {
            self.stop_signal.store(true, Ordering::Relaxed);
            if let Err(e) = handle.join() {
                 eprintln!("BackgroundFlusher: Error joining background thread: {:?}", e);
            } 
            if let Err(e) = futures::executor::block_on(crate::log::LogStorage::async_flush(&*self.writer.storage)) {
                eprintln!("BackgroundFlusher: Error during final flush in stop(): {}", e);
            }
        }
    }
}

impl<S: LogStorage + 'static> Drop for BackgroundFlusher<S> {
    fn drop(&mut self) {
        if self.handle.is_some() {
            self.stop_signal.store(true, Ordering::Relaxed);
            if let Some(handle) = self.handle.take() {
                 if let Err(e) = handle.join() {
                      eprintln!("BackgroundFlusher: Error joining background thread on drop: {:?}", e);
                 }
                 if let Err(e) = futures::executor::block_on(crate::log::LogStorage::async_flush(&*self.writer.storage)) {
                     eprintln!("BackgroundFlusher: Error during final flush in drop(): {}", e);
                 }
            }
        }
    }
}

/// Compression utilities for log segments
pub mod compression {
    use super::*;
    use std::io::{Read, Write};
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use flate2::read::GzDecoder;
    
    /// Compress a log segment
    pub fn compress_segment(segment: &LogSegment, level: u32) -> Result<Vec<u8>, EngineError> {
        let serialized = bincode::serialize(segment)
            .map_err(|e| EngineError::SerializationFailed(e.to_string()))?;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(&serialized)
            .map_err(|e| EngineError::IoError(e.to_string()))?;
        
        encoder.finish()
            .map_err(|e| EngineError::IoError(e.to_string()))
    }
    
    /// Decompress a log segment
    pub fn decompress_segment(compressed: &[u8]) -> Result<LogSegment, EngineError> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| EngineError::IoError(e.to_string()))?;
        
        bincode::deserialize(&decompressed)
            .map_err(|e| EngineError::DeserializationFailed(e.to_string()))
    }
    
    /// Compress a single log entry
    pub fn compress_entry(entry: &LogEntry, level: u32) -> Result<Vec<u8>, EngineError> {
        let serialized = bincode::serialize(entry)
            .map_err(|e| EngineError::SerializationFailed(e.to_string()))?;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(&serialized)
            .map_err(|e| EngineError::IoError(e.to_string()))?;
        
        encoder.finish()
            .map_err(|e| EngineError::IoError(e.to_string()))
    }
    
    /// Decompress a single log entry
    pub fn decompress_entry(compressed: &[u8]) -> Result<LogEntry, EngineError> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| EngineError::IoError(e.to_string()))?;
        
        bincode::deserialize(&decompressed)
            .map_err(|e| EngineError::DeserializationFailed(e.to_string()))
    }
}

/// Index for fast log entry retrieval based on various criteria
#[derive(Debug)]
pub struct LogIndex {
    /// Index entries by hash
    hash_index: Mutex<HashMap<String, usize>>,
    /// Index entries by timestamp
    time_index: Mutex<HashMap<Timestamp, Vec<usize>>>,
    /// Index entries by type
    type_index: Mutex<HashMap<EntryType, Vec<usize>>>,
    /// Index entries by domain
    domain_index: Mutex<HashMap<String, Vec<usize>>>,
}

impl LogIndex {
    /// Create a new log index
    pub fn new() -> Self {
        LogIndex {
            hash_index: Mutex::new(HashMap::new()),
            time_index: Mutex::new(HashMap::new()),
            type_index: Mutex::new(HashMap::new()),
            domain_index: Mutex::new(HashMap::new()),
        }
    }
    
    /// Add an entry to the index
    pub fn add_entry(&self, entry: &LogEntry, position: usize) -> Result<(), EngineError> {
        // Index by entry_hash if available
        if let Some(hash) = &entry.entry_hash {
            self.hash_index.lock().unwrap().insert(hash.clone(), position);
        }
        
        // Index by timestamp
        self.time_index.lock().unwrap()
            .entry(entry.timestamp.clone())
            .or_insert_with(Vec::new)
            .push(position);
        
        // Index by entry type
        self.type_index.lock().unwrap()
            .entry(entry.entry_type.clone())
            .or_insert_with(Vec::new)
            .push(position);
        
        // Index by domain if available - look in metadata
        if let Some(domain) = entry.metadata.get("domain") {
            self.domain_index.lock().unwrap()
                .entry(domain.clone())
                .or_insert_with(Vec::new)
                .push(position);
        }
        
        Ok(())
    }
    
    /// Find an entry by hash
    pub fn find_by_hash(&self, hash: &str) -> Option<usize> {
        self.hash_index.lock().unwrap().get(hash).copied()
    }
    
    /// Find entries by timestamp
    pub fn find_by_timestamp(&self, timestamp: Timestamp) -> Vec<usize> {
        self.time_index.lock().unwrap()
            .get(&timestamp)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Find entries by type
    pub fn find_by_type(&self, entry_type: EntryType) -> Vec<usize> {
        self.type_index.lock().unwrap()
            .get(&entry_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Find entries by domain
    pub fn find_by_domain(&self, domain: &str) -> Vec<usize> {
        self.domain_index.lock().unwrap()
            .get(domain)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Find entries in a time range
    pub fn find_in_time_range(&self, start: Timestamp, end: Timestamp) -> Vec<usize> {
        let time_index = self.time_index.lock().unwrap();
        let mut results = Vec::new();
        
        for (ts, positions) in time_index.iter() {
            if *ts >= start && *ts <= end {
                results.extend(positions);
            }
        }
        
        results
    }
    
    /// Clear the index
    pub fn clear(&self) {
        self.hash_index.lock().unwrap().clear();
        self.time_index.lock().unwrap().clear();
        self.type_index.lock().unwrap().clear();
        self.domain_index.lock().unwrap().clear();
    }

    // Helper methods for adding to indexes
    fn add_to_hash_index(&self, hash: &str, entry_id: usize) -> Result<(), EngineError> {
        let mut hash_idx = self.hash_index.lock().map_err(|e| EngineError::LogError(format!("Failed to lock hash index: {}", e)))?;
        hash_idx.insert(hash.to_string(), entry_id);
        Ok(())
    }
    
    fn add_to_timestamp_index(&self, timestamp: Timestamp, entry_id: usize) -> Result<(), EngineError> {
        let mut timestamp_idx = self.time_index.lock().map_err(|e| EngineError::LogError(format!("Failed to lock timestamp index: {}", e)))?;
        timestamp_idx.entry(timestamp).or_insert_with(Vec::new).push(entry_id);
        Ok(())
    }
    
    fn add_to_type_index(&self, entry_type: &EntryType, entry_id: usize) -> Result<(), EngineError> {
        let mut type_idx = self.type_index.lock().map_err(|e| EngineError::LogError(format!("Failed to lock type index: {}", e)))?;
        type_idx.entry(entry_type.clone()).or_insert_with(Vec::new).push(entry_id);
        Ok(())
    }
    
    fn add_to_domain_index(&self, domain_id: &str, entry_id: usize) -> Result<(), EngineError> {
        let mut domain_idx = self.domain_index.lock().map_err(|e| EngineError::LogError(format!("Failed to lock domain index: {}", e)))?;
        domain_idx.entry(domain_id.to_string()).or_insert_with(Vec::new).push(entry_id);
        Ok(())
    }
}

/// Storage with optimizations for read/write performance
#[derive(Debug)]
pub struct OptimizedLogStorage<S: LogStorage + Send + Sync + 'static> {
    /// Base storage implementation
    storage: Arc<S>,
    /// Batch writer for efficient writes
    batch_writer: BatchWriter<S>,
    /// Index for fast lookups
    index: LogIndex,
    /// Background flusher for periodic writes
    _background_flusher: Option<BackgroundFlusher<S>>,
}

impl<S: LogStorage + Send + Sync + 'static> OptimizedLogStorage<S> {
    /// Create a new optimized log storage with the given base storage
    pub async fn new(
        storage: S,
        batch_config: BatchConfig,
        flusher_config: BackgroundFlusherConfig,
        runtime: Arc<Runtime>,
    ) -> causality_error::Result<Self> {
        let storage_arc = Arc::new(storage);
        let batch_writer = BatchWriter::new(storage_arc.clone(), batch_config);
        let index = LogIndex::new();

        let batch_writer_arc = Arc::new(batch_writer);

        // Start the background flusher
        let flusher = BackgroundFlusher::start(batch_writer_arc.clone(), flusher_config, runtime);

        // Use try_unwrap carefully
        let batch_writer_instance = Arc::try_unwrap(batch_writer_arc)
             .map_err(|_| Box::new(EngineError::SyncError("Failed to unwrap Arc for batch_writer".to_string())) as Box<dyn CausalityError>)?; // Proper error handling

        let optimized_storage = Self {
            storage: storage_arc.clone(),
            batch_writer: batch_writer_instance, // Assign unwrapped instance
            index,
            _background_flusher: Some(flusher), // Initialize the field
        };
        
        // Build index (moved from original position)
        let entries = storage_arc.get_all_entries().await?; // Await the future
        for (i, entry) in entries.iter().enumerate() {
            optimized_storage.index.add_entry(entry, i).map_err(|e| {
                 Box::new(EngineError::LogError(format!("Error adding entry to index: {}", e))) as Box<dyn CausalityError>
            })?;
        }
        
        Ok(optimized_storage)
    }
    
    /// Get an entry by hash with index acceleration
    pub async fn get_entry_by_hash(&self, hash: &str) -> causality_error::Result<Option<LogEntry>> {
        if let Some(position) = self.index.find_by_hash(hash) {
            // Fast path: entry is in the index
            let entries = match self.storage.get_all_entries().await {
                Ok(entries) => entries,
                Err(e) => return Err(Box::new(EngineError::LogError(format!("Error getting all entries: {}", e)))),
            };
            
            if position < entries.len() {
                return Ok(Some(entries[position].clone()));
            }
        }
        
        // Fall back to direct lookup - Await inside the match
        match self.storage.get_entry_by_hash(hash) {
            Ok(entry) => Ok(entry),
            Err(e) => Err(Box::new(EngineError::LogError(format!("Error getting entry by hash: {}", e)))),
        }
    }
    
    /// Find entries by type
    pub async fn find_entries_by_type(&self, entry_type: EntryType) -> EngineResult<Vec<LogEntry>> {
        // Check if we have an index first
        let type_positions = self.index.find_by_type(entry_type.clone());
        
        if !type_positions.is_empty() {
            // We have positions in our index, convert to entries
            let mut entries = Vec::with_capacity(type_positions.len());
            
            // Use a simple synchronous implementation to get all entries
            let entry_count_result = self.storage.get_entry_count().await; // Added .await
            let count = match entry_count_result {
                 Ok(c) => c,
                 Err(e) => return Err(EngineError::LogError(format!("Failed to get entry count: {}", e))),
            };
            let all_entries = match self.storage.read(0, count) {
                Ok(entries) => entries,
                Err(e) => return Err(EngineError::LogError(format!("Failed to read entries: {}", e))),
            };
            
            for pos in type_positions {
                if pos < all_entries.len() {
                    entries.push(all_entries[pos].clone());
                }
            }
            
            return Ok(entries);
        }
        
        // Fall back to storage lookup using the synchronous method
        match self.storage.find_entries_by_type(entry_type) {
            Ok(entries) => Ok(entries),
            Err(e) => Err(EngineError::LogError(format!("Failed to find entries by type: {}", e))),
        }
    }
    
    /// Find entries in time range
    pub async fn find_entries_in_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> EngineResult<Vec<LogEntry>> {
        // Convert DateTime to our timestamp format
        let start_ts = Timestamp::from_millis(start.timestamp_millis() as u64);
        let end_ts = Timestamp::from_millis(end.timestamp_millis() as u64);
        
        // Check if we have an index first
        let time_positions = self.index.find_in_time_range(start_ts, end_ts);
        
        if !time_positions.is_empty() {
            // We have positions in our index, convert to entries
            let mut entries = Vec::with_capacity(time_positions.len());
            
            // Use a simple synchronous implementation to get all entries
            let entry_count_result = self.storage.get_entry_count().await; // Added .await
            let count = match entry_count_result {
                 Ok(c) => c,
                 Err(e) => return Err(EngineError::LogError(format!("Failed to get entry count: {}", e))),
            };
            let all_entries = match self.storage.read(0, count) {
                 Ok(entries) => entries,
                 Err(e) => return Err(EngineError::LogError(format!("Failed to read entries: {}", e))),
            };
            
            for pos in time_positions {
                if pos < all_entries.len() {
                    entries.push(all_entries[pos].clone());
                }
            }
            
            return Ok(entries);
        }
        
        // Fall back to storage lookup using the synchronous method
        self.storage.find_entries_in_time_range(start, end).map_err(|e| {
            EngineError::LogError(format!("Failed to find entries in time range: {}", e))
        })
    }

    // Rename this method to async_flush to avoid confusion
    async fn internal_async_flush(&self) -> CausalityResult<()> {
        // Get entries to flush
        let entries_to_flush = {
            let mut buffer = self.batch_writer.buffer.lock()
                 .map_err(|_| Box::new(EngineError::SyncError("Mutex poisoned".to_string())) as Box<dyn CausalityError>)?; 
            if buffer.is_empty() {
                return Ok(());
            }
            let entries = std::mem::take(&mut *buffer);
            // Update last_flush time using Mutex lock
            *self.batch_writer.last_flush.lock().map_err(|_| Box::new(EngineError::SyncError("Mutex poisoned".to_string())) as Box<dyn CausalityError>)? = Instant::now(); 
            entries
        };
        
        if !entries_to_flush.is_empty() {
             let entries = if self.batch_writer.config.compress_batches {
                 self.compress_entries(&entries_to_flush).map_err(|e| Box::new(e) as Box<dyn CausalityError>)?
             } else {
                 entries_to_flush
             };
             crate::log::LogStorage::append_entries_batch(&*self.storage, entries).await?; 
        }
        Ok(())
    }

    /// Compress a set of log entries for storage efficiency
    pub fn compress_entries(&self, entries: &[LogEntry]) -> EngineResult<Vec<LogEntry>> {
        // If compression is not enabled in the batch writer, just return the entries
        if !self.batch_writer.config.compress_batches {
            return Ok(entries.to_vec());
        }
        
        // Use the compression module to handle the compression
        let mut compressed_entries = Vec::with_capacity(entries.len());
        
        for entry in entries {
            // Serialize entry to bytes
            let serialized = serde_json::to_vec(entry)
                .map_err(|e| EngineError::LogError(format!("Failed to serialize entry: {}", e)))?;
            
            // Compress the bytes
            let level = self.batch_writer.config.compression_level;
            let compressed_bytes = compression::compress_entry(entry, level)?;
            
            // Create a new entry with the compressed data
            let mut compressed_entry = entry.clone();
            compressed_entry.metadata.insert("compressed".to_string(), 
                true.to_string());
            
            compressed_entries.push(compressed_entry);
        }
        
        Ok(compressed_entries)
    }
}

/// Wrapper that implements LogStorage for OptimizedLogStorage
#[async_trait]
impl<S: LogStorage + Send + Sync + 'static> LogStorage for OptimizedLogStorage<S> {
    async fn append_entry(&self, entry: LogEntry) -> CausalityResult<()> {
        if self.batch_writer.config.max_batch_size == 0 {
            return crate::log::LogStorage::append_entry(&*self.storage, entry).await;
        }
        let should_flush;
        {
            let mut batch = self.batch_writer.buffer.lock()
                .map_err(|_| Box::new(EngineError::SyncError("Mutex poisoned".to_string())) as Box<dyn CausalityError>)?; 
            batch.push(entry.clone());
            should_flush = batch.len() >= self.batch_writer.config.max_batch_size;
        }
        if should_flush {
            self.internal_async_flush().await?;
        }
        Ok(())
    }
    
    async fn get_all_entries(&self) -> CausalityResult<Vec<LogEntry>> {
        self.internal_async_flush().await?;
        crate::log::LogStorage::get_all_entries(&*self.storage).await
    }

    async fn get_entries(&self, start: usize, end: usize) -> CausalityResult<Vec<LogEntry>> {
        self.internal_async_flush().await?;
        crate::log::LogStorage::get_entries(&*self.storage, start, end).await
    }

    async fn get_entry_count(&self) -> CausalityResult<usize> {
        let batch_len = {
            let batch = self.batch_writer.buffer.lock()
                .map_err(|_| Box::new(EngineError::SyncError("Mutex poisoned".to_string())) as Box<dyn CausalityError>)?; 
            batch.len()
        };
        let storage_count = crate::log::LogStorage::get_entry_count(&*self.storage).await?;
        Ok(storage_count + batch_len)
    }
    
    async fn clear(&self) -> CausalityResult<()> {
        self.internal_async_flush().await?;
        {
            let mut buffer = self.batch_writer.buffer.lock()
                .map_err(|_| Box::new(EngineError::SyncError("Mutex poisoned".to_string())) as Box<dyn CausalityError>)?; 
            buffer.clear();
        }
        self.index.clear();
        crate::log::LogStorage::clear(&*self.storage).await
    }

    /// Flush any pending operations to the storage (async version)
    async fn async_flush(&self) -> CausalityResult<()> {
        // Flush the batch writer's buffer first using the internal method
        self.internal_async_flush().await?;
        
        // Then explicitly call the underlying storage's async_flush
        crate::log::LogStorage::async_flush(&*self.storage).await
    }

    fn append(&self, entry: LogEntry) -> CausalityResult<()> {
        // Add to the batch writer 
        self.batch_writer.add_entry(entry.clone())
            .map_err(|e| Box::new(e) as Box<dyn CausalityError>)
    }
    
    fn append_batch(&self, entries: Vec<LogEntry>) -> CausalityResult<()> {
        for entry in entries {
            self.batch_writer.add_entry(entry)
                .map_err(|e| Box::new(e) as Box<dyn CausalityError>)?;
        }
        Ok(())
    }
    
    fn read(&self, start: usize, count: usize) -> CausalityResult<Vec<LogEntry>> {
        block_on(<Self as LogStorage>::async_flush(self))?; // Flush before reading
        self.storage.read(start, count).map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn entry_count(&self) -> CausalityResult<usize> {
         block_on(<Self as LogStorage>::async_flush(self))?; // Flush before reading count
         self.storage.entry_count().map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn get_entry_by_hash(&self, hash: &str) -> CausalityResult<Option<LogEntry>> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.get_entry_by_hash(hash).map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn get_entries_by_trace(&self, trace_id: &str) -> CausalityResult<Vec<LogEntry>> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.get_entries_by_trace(trace_id).map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn find_entries_by_type(&self, entry_type: EntryType) -> CausalityResult<Vec<LogEntry>> {
         block_on(<Self as LogStorage>::async_flush(self))?;
         self.storage.find_entries_by_type(entry_type).map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn find_entries_in_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> CausalityResult<Vec<LogEntry>> {
         block_on(<Self as LogStorage>::async_flush(self))?;
         self.storage.find_entries_in_time_range(start, end).map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn rotate(&self) -> CausalityResult<()> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.rotate().map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn compact(&self) -> CausalityResult<()> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.compact().map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn close(&self) -> CausalityResult<()> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.close().map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn get_entry_by_id(&self, id: &str) -> CausalityResult<Option<LogEntry>> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.get_entry_by_id(id).map_err(|e| e) // Propagate CausalityResult error
    }
    
    fn find_entries_by_trace_id(&self, trace_id: &causality_types::TraceId) -> CausalityResult<Vec<LogEntry>> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.find_entries_by_trace_id(trace_id).map_err(|e| e) // Propagate CausalityResult error
    }

    fn read_time_range(&self, start_time: u64, end_time: u64) -> CausalityResult<Vec<LogEntry>> {
        block_on(<Self as LogStorage>::async_flush(self))?;
        self.storage.read_time_range(start_time, end_time).map_err(|e| e) // Propagate CausalityResult error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::MemoryLogStorage;
    use causality_types::Timestamp;
    use tokio;

    // Helper to create a test entry
    fn create_test_entry(entry_type: EntryType, timestamp: Timestamp) -> LogEntry {
        LogEntry {
            id: format!("test-entry-{}", rand::random::<u32>()),
            timestamp,
            entry_type,
            data: EntryData::Custom(serde_json::json!({
                "test": "data"
            })),
            trace_id: None,
            parent_id: None,
            metadata: std::collections::HashMap::new(),
            entry_hash: Some(format!("hash-{}", rand::random::<u32>())),
        }
    }
    
    #[test]
    fn test_batch_writing() -> Result<(), EngineError> {
        let storage = MemoryLogStorage::new();
        let config = BatchConfig {
            max_batch_size: 5,
            flush_interval_ms: 100,
            compress_batches: false,
            compression_level: 0,
        };
        let batch_writer = BatchWriter::new(Arc::new(storage), config);
        
        // Add entries below batch size threshold
        for i in 0..3 {
            let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
            batch_writer.add_entry(entry)?;
        }
        
        // Entries should still be in buffer
        assert_eq!(batch_writer.storage.get_entry_count()?, 0);
        
        // Add more entries to exceed batch size
        for i in 3..8 {
            let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
            batch_writer.add_entry(entry)?;
        }
        
        // Batch should have been flushed
        assert_eq!(batch_writer.storage.get_entry_count()?, 5);
        
        // Explicitly flush remaining entries
        batch_writer.flush()?;
        assert_eq!(batch_writer.storage.get_entry_count()?, 8);
        
        Ok(())
    }
    
    #[test]
    fn test_log_index() -> Result<(), EngineError> {
        let index = LogIndex::new();
        
        // Add test entries to the index
        for i in 0..10 {
            let entry_type = if i % 2 == 0 { EntryType::Fact } else { EntryType::Effect };
            let entry = create_test_entry(entry_type, Timestamp::from_millis(i));
            index.add_entry(&entry, i)?;
        }
        
        // Test finding by hash
        let hash = "hash_5".to_string();
        assert_eq!(index.find_by_hash(&hash), Some(5));
        
        // Test finding by type
        let fact_positions = index.find_by_type(EntryType::Fact);
        assert_eq!(fact_positions.len(), 5); // Entries 0, 2, 4, 6, 8
        assert!(fact_positions.contains(&0));
        assert!(fact_positions.contains(&2));
        
        // Test finding by time range
        let time_range_positions = index.find_in_time_range(
            Timestamp::from_millis(3),
            Timestamp::from_millis(7)
        );
        assert_eq!(time_range_positions.len(), 5); // Entries 3, 4, 5, 6, 7
        
        // Test finding by domain
        let domain_positions = index.find_by_domain("test_domain");
        assert_eq!(domain_positions.len(), 10); // All entries
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_optimized_storage() -> Result<(), Box<dyn std::error::Error>> {
        let base_storage = MemoryLogStorage::new();
        let batch_config = BatchConfig { max_batch_size: 5, ..Default::default() };
        let flusher_config = BackgroundFlusherConfig { enabled: false, ..Default::default() };
        let runtime = Arc::new(Runtime::new().unwrap());

        let optimized = OptimizedLogStorage::new(
            base_storage,
            batch_config,
            flusher_config,
            runtime
        ).await?;

        for i in 0..10 {
            let entry_type = if i % 2 == 0 { EntryType::Fact } else { EntryType::Effect };
            let entry = create_test_entry(entry_type, Timestamp::from_millis(i as u64));
            optimized.append_entry(entry).await?;
        }

        optimized.async_flush().await?;
        assert_eq!(optimized.get_entry_count().await?, 10);

        let entries_read = optimized.read(5, 1)?;
        assert_eq!(entries_read.len(), 1);
        let hash_to_find = entries_read[0].entry_hash.clone().unwrap();
        let entry = optimized.get_entry_by_hash(&hash_to_find)?;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().timestamp, Timestamp::from_millis(5));

        let facts = optimized.find_entries_by_type(EntryType::Fact)?;
        assert_eq!(facts.len(), 5);

        let time_range_entries_sync = optimized.read_time_range(3, 7)?;
        assert_eq!(time_range_entries_sync.len(), 5); 

        Ok(())
    }
    
    #[test]
    fn test_compression() -> Result<(), EngineError> {
        // Create a test segment
        let mut segment = LogSegment::new("test_segment".to_string());
        let mut entries = Vec::new();
        
        // Add entries to the segment and keep a copy for comparison
        for i in 0..100 {
            let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
            segment.add_entry(entry.clone())?;
            entries.push(entry);
        }
        
        // Compress the segment
        let compressed = compression::compress_segment(&segment, 6)?;
        
        // Verify compression ratio (should be significantly smaller)
        let original_size = bincode::serialize(&segment)
            .map_err(|e| EngineError::SerializationFailed(e.to_string()))?
            .len();
        println!("Original size: {}, Compressed size: {}", original_size, compressed.len());
        assert!(compressed.len() < original_size);
        
        // Decompress and verify
        let decompressed = compression::decompress_segment(&compressed)?;
        assert_eq!(decompressed.info().id, segment.info().id);
        assert_eq!(decompressed.entry_count(), segment.entry_count());
        
        Ok(())
    }
}

/// Writer for log entries with performance optimizations
pub struct LogWriter<S: LogStorage> {
    /// The storage implementation
    pub storage: Arc<S>,
    /// The queue of log entries
    pub queue: Arc<RwLock<Vec<LogEntry>>>,
    /// The configuration for flushing
    pub config: FlushConfig,
    /// Timestamp of the last flush operation
    pub last_flush: Arc<RwLock<Instant>>,
}

impl<S: LogStorage> LogWriter<S> {
    /// Create a new log writer
    pub fn new(storage: Arc<S>, config: FlushConfig) -> Self {
        Self {
            storage,
            queue: Arc::new(RwLock::new(Vec::new())),
            config,
            last_flush: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Add a log entry to the queue
    pub async fn add_entry(&self, entry: LogEntry) -> CausalityResult<()> {
        let mut queue = self.queue.write().unwrap();
        queue.push(entry);
        let should_flush = queue.len() >= self.config.max_entries;
        drop(queue); 
        if should_flush { 
            self.flush_queue().await?; 
        }
        Ok(())
    }
    
    /// Flush the queue to storage
    pub async fn flush_queue(&self) -> CausalityResult<()> {
        let entries = {
            let mut queue = self.queue.write().unwrap();
            if queue.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *queue)
        };
        self.storage.append_entries_batch(entries).await?;
        *self.last_flush.write().unwrap() = Instant::now();
        Ok(())
    }
    
    /// Flush both the queue and the underlying storage
    pub async fn async_flush(&self) -> CausalityResult<()> {
        self.flush_queue().await?;
        self.storage.async_flush().await?;
        Ok(())
    }
    
    /// Check if a flush is needed based on time or queue size
    pub fn needs_flush(&self) -> bool {
        let queue = self.queue.read().unwrap();
        !queue.is_empty()
    }
} 