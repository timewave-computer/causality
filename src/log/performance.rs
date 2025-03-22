// Log System Performance Optimizations
//
// This module provides performance enhancements for the log system including:
// - Batched writes for efficiency
// - Compression for log segments
// - Indexing for faster queries

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::log::{LogEntry, LogStorage, LogSegment, EntryType};
use crate::types::Timestamp;

/// Configuration for the batch writer
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        BatchWriter {
            storage,
            config,
            buffer: Mutex::new(Vec::with_capacity(config.max_batch_size)),
            last_flush: Mutex::new(Instant::now()),
        }
    }
    
    /// Add an entry to the batch
    pub fn add_entry(&self, entry: LogEntry) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.push(entry);
        
        // Check if we should flush due to batch size
        if buffer.len() >= self.config.max_batch_size {
            self.flush_internal(&mut buffer)?;
        }
        
        // Check if we should flush due to time
        let mut last_flush = self.last_flush.lock().unwrap();
        let elapsed = last_flush.elapsed();
        if elapsed >= Duration::from_millis(self.config.flush_interval_ms) {
            self.flush_internal(&mut buffer)?;
            *last_flush = Instant::now();
        }
        
        Ok(())
    }
    
    /// Flush all buffered entries to storage
    pub fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        if !buffer.is_empty() {
            self.flush_internal(&mut buffer)?;
            *self.last_flush.lock().unwrap() = Instant::now();
        }
        Ok(())
    }
    
    /// Internal flush implementation that takes a pre-locked buffer
    fn flush_internal(&self, buffer: &mut Vec<LogEntry>) -> Result<()> {
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
        
        // Write all entries in a single batch
        self.storage.append_entries(&entries_to_write)?;
        
        Ok(())
    }
    
    /// Compress a batch of entries
    fn compress_entries(&self, entries: &[LogEntry]) -> Result<Vec<LogEntry>> {
        // In a real implementation, this would compress entry data while preserving metadata
        // For this example, we'll just clone the entries (actual compression would be implemented here)
        Ok(entries.to_vec())
    }
    
    /// Start a background task that periodically flushes the buffer
    pub fn start_background_flush(&self) -> Result<BackgroundFlusher<S>> {
        BackgroundFlusher::new(self)
    }
}

/// Background flusher that periodically writes batched entries
pub struct BackgroundFlusher<S: LogStorage> {
    writer: Arc<BatchWriter<S>>,
    running: Arc<Mutex<bool>>,
}

impl<S: LogStorage> BackgroundFlusher<S> {
    /// Create a new background flusher for the given batch writer
    fn new(writer: &BatchWriter<S>) -> Result<Self> {
        let writer = Arc::new(writer.clone());
        let running = Arc::new(Mutex::new(true));
        let flusher = BackgroundFlusher {
            writer: writer.clone(),
            running: running.clone(),
        };
        
        // Spawn background thread
        let flush_interval = writer.config.flush_interval_ms;
        std::thread::spawn(move || {
            while *running.lock().unwrap() {
                std::thread::sleep(Duration::from_millis(flush_interval));
                if let Err(e) = writer.flush() {
                    eprintln!("Error flushing batch writer: {}", e);
                }
            }
        });
        
        Ok(flusher)
    }
    
    /// Stop the background flusher
    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        *running = false;
        self.writer.flush()?;
        Ok(())
    }
}

impl<S: LogStorage> Clone for BatchWriter<S> {
    fn clone(&self) -> Self {
        BatchWriter {
            storage: Arc::clone(&self.storage),
            config: self.config.clone(),
            buffer: Mutex::new(Vec::with_capacity(self.config.max_batch_size)),
            last_flush: Mutex::new(Instant::now()),
        }
    }
}

impl<S: LogStorage> Drop for BatchWriter<S> {
    fn drop(&mut self) {
        // Try to flush any remaining entries on drop
        if let Err(e) = self.flush() {
            eprintln!("Error flushing batch writer on drop: {}", e);
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
    pub fn compress_segment(segment: &LogSegment, level: u32) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(segment)
            .map_err(|e| Error::SerializationFailed(e.to_string()))?;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(&serialized)
            .map_err(|e| Error::IoError(e.to_string()))?;
        
        encoder.finish()
            .map_err(|e| Error::IoError(e.to_string()))
    }
    
    /// Decompress a log segment
    pub fn decompress_segment(compressed: &[u8]) -> Result<LogSegment> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| Error::IoError(e.to_string()))?;
        
        bincode::deserialize(&decompressed)
            .map_err(|e| Error::DeserializationFailed(e.to_string()))
    }
    
    /// Compress a single log entry
    pub fn compress_entry(entry: &LogEntry, level: u32) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(entry)
            .map_err(|e| Error::SerializationFailed(e.to_string()))?;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(&serialized)
            .map_err(|e| Error::IoError(e.to_string()))?;
        
        encoder.finish()
            .map_err(|e| Error::IoError(e.to_string()))
    }
    
    /// Decompress a single log entry
    pub fn decompress_entry(compressed: &[u8]) -> Result<LogEntry> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| Error::IoError(e.to_string()))?;
        
        bincode::deserialize(&decompressed)
            .map_err(|e| Error::DeserializationFailed(e.to_string()))
    }
}

/// Index for fast log entry retrieval based on various criteria
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
    pub fn add_entry(&self, entry: &LogEntry, position: usize) -> Result<()> {
        // Index by hash
        self.hash_index.lock().unwrap().insert(entry.hash.clone(), position);
        
        // Index by timestamp
        self.time_index.lock().unwrap()
            .entry(entry.timestamp)
            .or_insert_with(Vec::new)
            .push(position);
        
        // Index by type
        self.type_index.lock().unwrap()
            .entry(entry.entry_type)
            .or_insert_with(Vec::new)
            .push(position);
        
        // Index by domain if available
        if let Some(domain) = entry.domain.as_ref() {
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
}

/// Enhanced log storage that includes performance optimizations
pub struct OptimizedLogStorage<S: LogStorage> {
    /// Base storage implementation
    storage: Arc<S>,
    /// Batch writer for efficient writes
    batch_writer: BatchWriter<S>,
    /// Index for fast lookups
    index: LogIndex,
    /// Background flusher for periodic writes
    _background_flusher: Option<BackgroundFlusher<S>>,
}

impl<S: LogStorage> OptimizedLogStorage<S> {
    /// Create a new optimized log storage with the given base storage
    pub fn new(storage: S, config: Option<BatchConfig>) -> Result<Self> {
        let storage = Arc::new(storage);
        let config = config.unwrap_or_default();
        let batch_writer = BatchWriter::new(Arc::clone(&storage), config);
        let index = LogIndex::new();
        
        // Start background flusher
        let background_flusher = Some(batch_writer.start_background_flush()?);
        
        // Build the initial index by scanning all entries
        let entries = storage.get_all_entries()?;
        for (i, entry) in entries.iter().enumerate() {
            index.add_entry(entry, i)?;
        }
        
        Ok(OptimizedLogStorage {
            storage,
            batch_writer,
            index,
            _background_flusher: background_flusher,
        })
    }
    
    /// Get an entry by hash with index acceleration
    pub fn get_entry_by_hash(&self, hash: &str) -> Result<Option<LogEntry>> {
        if let Some(position) = self.index.find_by_hash(hash) {
            // Fast path: entry is in the index
            let entries = self.storage.get_all_entries()?;
            if position < entries.len() {
                return Ok(Some(entries[position].clone()));
            }
        }
        
        // Fallback to regular storage lookup
        self.storage.get_entry_by_hash(hash)
    }
    
    /// Find entries by type with index acceleration
    pub fn find_entries_by_type(&self, entry_type: EntryType) -> Result<Vec<LogEntry>> {
        let positions = self.index.find_by_type(entry_type);
        if !positions.is_empty() {
            // Fast path: entries are in the index
            let entries = self.storage.get_all_entries()?;
            let mut results = Vec::with_capacity(positions.len());
            
            for pos in positions {
                if pos < entries.len() {
                    results.push(entries[pos].clone());
                }
            }
            
            return Ok(results);
        }
        
        // Fallback to regular storage lookup
        self.storage.find_entries_by_type(entry_type)
    }
    
    /// Find entries in a time range with index acceleration
    pub fn find_entries_in_time_range(&self, start: Timestamp, end: Timestamp) -> Result<Vec<LogEntry>> {
        let positions = self.index.find_in_time_range(start, end);
        if !positions.is_empty() {
            // Fast path: entries are in the index
            let entries = self.storage.get_all_entries()?;
            let mut results = Vec::with_capacity(positions.len());
            
            for pos in positions {
                if pos < entries.len() {
                    results.push(entries[pos].clone());
                }
            }
            
            return Ok(results);
        }
        
        // Fallback to regular storage lookup
        self.storage.find_entries_in_time_range(start, end)
    }
}

/// Wrapper that implements LogStorage for OptimizedLogStorage
#[async_trait]
impl<S: LogStorage + Send + Sync> LogStorage for OptimizedLogStorage<S> {
    async fn append_entry(&self, entry: LogEntry) -> Result<()> {
        // Add to the batch writer instead of directly to storage
        self.batch_writer.add_entry(entry.clone())?;
        
        // Update the index with the new entry
        let position = self.storage.get_entry_count()?;
        self.index.add_entry(&entry, position)?;
        
        Ok(())
    }
    
    fn append_entries(&self, entries: &[LogEntry]) -> Result<()> {
        // Process entries in batch
        for entry in entries.iter() {
            self.batch_writer.add_entry(entry.clone())?;
            
            // Update the index with the new entry
            let position = self.storage.get_entry_count()?;
            self.index.add_entry(entry, position)?;
        }
        
        // Force flush to ensure all entries are written
        self.batch_writer.flush()?;
        
        Ok(())
    }
    
    fn get_entry_by_hash(&self, hash: &str) -> Result<Option<LogEntry>> {
        self.get_entry_by_hash(hash)
    }
    
    fn get_all_entries(&self) -> Result<Vec<LogEntry>> {
        // Ensure batch writer is flushed before reading
        self.batch_writer.flush()?;
        self.storage.get_all_entries()
    }
    
    fn find_entries_by_type(&self, entry_type: EntryType) -> Result<Vec<LogEntry>> {
        self.find_entries_by_type(entry_type)
    }
    
    fn find_entries_in_time_range(&self, start: Timestamp, end: Timestamp) -> Result<Vec<LogEntry>> {
        self.find_entries_in_time_range(start, end)
    }
    
    fn get_entry_count(&self) -> Result<usize> {
        // Ensure batch writer is flushed before counting
        self.batch_writer.flush()?;
        self.storage.get_entry_count()
    }
    
    fn clear(&self) -> Result<()> {
        self.batch_writer.flush()?;
        self.index.clear();
        self.storage.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::MemoryLogStorage;
    use crate::types::Timestamp;
    
    // Helper to create a test entry
    fn create_test_entry(entry_type: EntryType, timestamp: Timestamp) -> LogEntry {
        LogEntry {
            hash: format!("hash_{}", timestamp),
            entry_type,
            timestamp,
            data: Vec::new(),
            domain: Some("test_domain".to_string()),
            parents: Vec::new(),
        }
    }
    
    #[test]
    fn test_batch_writing() -> Result<()> {
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
    fn test_log_index() -> Result<()> {
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
    
    #[test]
    fn test_optimized_storage() -> Result<()> {
        let base_storage = MemoryLogStorage::new();
        let optimized = OptimizedLogStorage::new(base_storage, None)?;
        
        // Add test entries
        for i in 0..10 {
            let entry_type = if i % 2 == 0 { EntryType::Fact } else { EntryType::Effect };
            let entry = create_test_entry(entry_type, Timestamp::from_millis(i));
            optimized.append_entry(entry).block_on()?;
        }
        
        // Ensure all entries were written
        assert_eq!(optimized.get_entry_count()?, 10);
        
        // Test finding by hash using index
        let hash = "hash_5".to_string();
        let entry = optimized.get_entry_by_hash(&hash)?;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().timestamp, Timestamp::from_millis(5));
        
        // Test finding by type using index
        let facts = optimized.find_entries_by_type(EntryType::Fact)?;
        assert_eq!(facts.len(), 5);
        
        // Test finding by time range using index
        let time_range_entries = optimized.find_entries_in_time_range(
            Timestamp::from_millis(3),
            Timestamp::from_millis(7)
        )?;
        assert_eq!(time_range_entries.len(), 5);
        
        Ok(())
    }
    
    #[test]
    fn test_compression() -> Result<()> {
        // Create a test segment
        let mut segment = LogSegment::new("test_segment".to_string());
        for i in 0..100 {
            let entry = create_test_entry(EntryType::Fact, Timestamp::from_millis(i));
            segment.add_entry(entry);
        }
        
        // Compress the segment
        let compressed = compression::compress_segment(&segment, 6)?;
        
        // Verify compression ratio (should be significantly smaller)
        let original_size = bincode::serialize(&segment)
            .map_err(|e| Error::SerializationFailed(e.to_string()))?
            .len();
        println!("Original size: {}, Compressed size: {}", original_size, compressed.len());
        assert!(compressed.len() < original_size);
        
        // Decompress and verify
        let decompressed = compression::decompress_segment(&compressed)?;
        assert_eq!(decompressed.id, segment.id);
        assert_eq!(decompressed.entries.len(), segment.entries.len());
        
        Ok(())
    }
    
    // Helper to run async functions in tests
    trait BlockOn<T> {
        fn block_on(self) -> T;
    }
    
    impl<T> BlockOn<T> for std::result::Result<T, Error> {
        fn block_on(self) -> Self {
            self
        }
    }
} 