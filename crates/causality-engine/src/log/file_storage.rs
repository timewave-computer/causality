// File-based log storage
// Original file: src/log/storage/file_storage.rs

// File storage implementation for Causality Unified Log System
//
// This module provides a file-based implementation of the LogStorage trait.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write}; // Added BufReader, BufWriter
use std::sync::{Arc, Mutex, PoisonError, RwLock}; // Added PoisonError and guards
use std::collections::{HashMap, VecDeque};
use std::fmt;
use futures::executor::block_on; // Ensure this import is present
  // Add std::env for temp_dir

use causality_error::{BoxError, CausalityError, EngineError, Result}; // Use alias and BoxError
use crate::log::{types::{LogEntry, EntryType}, storage::{StorageConfig, StorageFormat}, LogStorage};
use crate::log::segment::{LogSegment, SegmentInfo, generate_segment_id};
// use crate::error_conversions::{convert_boxed_error, BoxedCausalityError}; // Likely unused now
 // Keep only types that are known to exist
 // Assuming this is needed
use tracing::warn; // Assuming these are needed

// Add dependency on ciborium for CBOR serialization if not already in Cargo.toml
// extern crate ciborium; // Usually not needed with Rust 2018+

/// File-based storage for log entries
///
/// This implementation stores log entries in files on disk.
pub struct FileLogStorage {
    /// The storage configuration
    config: StorageConfig,
    /// The base path for storage
    base_path: PathBuf,
    /// The segments in this storage
    segments: RwLock<Vec<SegmentInfo>>,
    /// The active segment
    active_segment: Mutex<LogSegment>,
    /// A cache of recently accessed segments
    segment_cache: RwLock<HashMap<String, Arc<Mutex<LogSegment>>>>,
    /// The segment access queue for cache management
    segment_queue: Mutex<VecDeque<String>>,
    /// The total number of entries
    entry_count: RwLock<usize>,
}

// Helper to convert PoisonError to EngineError -> BoxError
fn map_poison_err<Guard>(err: PoisonError<Guard>) -> BoxError {
    Box::new(EngineError::SyncError(format!("Mutex/RwLock poisoned: {}", err)))
}


impl FileLogStorage {
    /// Create a new file storage with the given base path and configuration
    pub fn new<P: AsRef<Path>>(base_path: P, config: StorageConfig) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create the base directory if it doesn't exist
        fs::create_dir_all(&base_path)
            .map_err(|e| EngineError::StorageError(format!("Failed to create base directory: {}", e)))?;

        // Load existing segments or create a new one
        let segments = Self::load_segments(&base_path)?;

        let active_segment = if segments.is_empty() {
            // Create a new segment
            Self::create_new_segment()?
        } else {
            // Load the latest segment
            let latest_segment = segments.last().unwrap(); // Safe due to is_empty check
            // Create a fallback path if path is None
            let segment_path = match &latest_segment.path {
                Some(path) => path.clone(),
                // Use .log extension consistent with saving
                None => base_path.join(format!("{}.log", latest_segment.id))
            };
            Self::load_segment_from_file(&segment_path, latest_segment.clone())?
        };

        // Calculate total entry count (could be inaccurate if segments weren't loaded properly before)
        let entry_count = segments.iter().map(|seg| seg.entry_count).sum::<usize>() + active_segment.info().entry_count;

        Ok(Self {
            config,
            base_path,
            segments: RwLock::new(segments),
            active_segment: Mutex::new(active_segment),
            segment_cache: RwLock::new(HashMap::new()),
            segment_queue: Mutex::new(VecDeque::new()),
            entry_count: RwLock::new(entry_count),
        })
    }

    /// Create a new file storage with the given base path and default configuration
    pub fn with_default_config<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let mut config = StorageConfig::default();
        config.base_dir = base_path.as_ref().to_path_buf(); // Ensure base_dir is set
        Self::new(base_path, config)
    }

    /// Load existing segments from the base path
    fn load_segments(base_path: &Path) -> Result<Vec<SegmentInfo>> {
        let meta_file = base_path.join("segments.json");

        if !meta_file.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(meta_file)
            .map_err(|e| EngineError::StorageError(format!("Failed to open segments.json: {}", e)))?;
        let reader = BufReader::new(file);
        let segments: Vec<SegmentInfo> = serde_json::from_reader(reader)
            .map_err(|e| EngineError::StorageError(format!("Failed to parse segments.json: {}", e)))?;

        Ok(segments)
    }

    /// Save segment index to disk
    fn save_segment_index(base_path: &Path, segments: &[SegmentInfo]) -> Result<()> {
        let meta_file = base_path.join("segments.json");

        let file = File::create(meta_file)
            .map_err(|e| EngineError::StorageError(format!("Failed to create segments.json: {}", e)))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, segments) // Use pretty print for readability
            .map_err(|e| EngineError::StorageError(format!("Failed to write segments to JSON: {}", e)))?;

        Ok(())
    }

    /// Create a new segment
    fn create_new_segment() -> Result<LogSegment> {
        let segment_id = generate_segment_id();
        // Pass config format to new segment if needed, or use default
        Ok(LogSegment::new(segment_id /*, start_time, start_index? */))
    }

    /// Load a segment from a file
    fn load_segment_from_file(path: &Path, info: SegmentInfo) -> Result<LogSegment> {
        if !path.exists() {
            return Err(EngineError::StorageError(format!("Segment file not found: {:?}", path)).into_boxed());
        }

        let file = File::open(path)
            .map_err(|e| EngineError::StorageError(format!("Failed to open segment file {:?}: {}", path, e)))?;
        let mut reader = BufReader::new(file);

        // Default to JSON format if not specified
        let format = StorageFormat::Json;

        let entries: Vec<LogEntry> = match format {
            StorageFormat::Json => {
                let mut entries = Vec::new();
                // Need BufRead trait
                use std::io::BufRead;
                for line in reader.lines() {
                     let line = line.map_err(|e| EngineError::StorageError(format!("Failed to read line: {}", e)))?;
                     if !line.trim().is_empty() {
                         let entry: LogEntry = serde_json::from_str(&line)
                             .map_err(|e| EngineError::StorageError(format!("Failed to parse entry JSON: '{}', error: {}", line, e)))?;
                         entries.push(entry);
                     }
                 }
                entries
            }
            StorageFormat::Binary => {
                // Need to read whole file for bincode::deserialize_from?
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)
                    .map_err(|e| EngineError::StorageError(format!("Failed to read segment file for bincode: {}", e)))?;
                bincode::deserialize(&buffer)
                    .map_err(|e| EngineError::StorageError(format!("Failed to deserialize bincode: {}", e)))?
            }
            StorageFormat::Cbor => {
                 // from_reader might consume the reader
                 ciborium::de::from_reader(reader)
                    .map_err(|e| EngineError::StorageError(format!("Failed to deserialize CBOR: {}", e)))?
            }
        };

        // Reconstruct LogSegment
        let mut segment = LogSegment::new(info.id.clone());
        
        // Update the segment with the loaded entries
        for entry in entries {
            segment.append(entry)?;
        }
        
        // Set other needed properties
        segment.set_path(path);
        
        Ok(segment)
    }

    /// Save a segment to disk
    fn save_segment(&self, segment: &LogSegment) -> Result<()> {
        let segment_info = segment.info(); // Assuming this returns &SegmentInfo

        let segment_path = self.base_path.join(format!("{}.{}",
            segment_info.id,
            match self.config.format {
                StorageFormat::Json => "log", // Use consistent .log extension maybe?
                StorageFormat::Binary => "bin",
                StorageFormat::Cbor => "cbor",
            }
        ));

        let file = File::create(&segment_path)
            .map_err(|e| EngineError::StorageError(format!("Failed to create segment file: {}", e)))?;
        let mut file_writer = BufWriter::new(file); // Use BufWriter

        match self.config.format {
            StorageFormat::Json => {
                let entries = segment.entries(); // Assuming returns Vec<LogEntry> or slice
                for entry in entries {
                    // Handle potential serialization error for each entry
                    let json = serde_json::to_string(&entry)
                         .map_err(|e| EngineError::StorageError(format!("Failed to serialize entry to JSON: {}", e)))?;
                    file_writer.write_all(json.as_bytes())
                         .map_err(|e| EngineError::StorageError(format!("Failed to write to file: {}", e)))?;
                    file_writer.write_all(b"\n")
                         .map_err(|e| EngineError::StorageError(format!("Failed to write newline to file: {}", e)))?;
                }
            }
            StorageFormat::Binary => {
                let entries = segment.entries(); // Assuming returns Vec<LogEntry> or slice
                // serialize_into is often more efficient for writers
                bincode::serialize_into(&mut file_writer, &entries)
                    .map_err(|e| EngineError::StorageError(format!("Failed to serialize entries to bincode: {}", e)))?;
            }
            StorageFormat::Cbor => {
                let entries = segment.entries(); // Assuming returns Vec<LogEntry> or slice
                ciborium::ser::into_writer(&entries, &mut file_writer)
                    .map_err(|e| EngineError::StorageError(format!("Failed to serialize to CBOR: {}", e)))?;
            }
        }
        // Flush the writer to ensure data is written to the OS buffer
        file_writer.flush()
            .map_err(|e| EngineError::StorageError(format!("Failed to flush file writer: {}", e)))?;

        // Get the underlying file to sync if needed, requires BufWriter::into_inner()
        // file_writer.into_inner().map_err(|_| EngineError::StorageError("Failed to get inner file".to_string()))?.sync_all().map_err(|e| ...)?;
        // Or just rely on OS flush for now.

        let mut segment_info_mut = segment_info.clone(); // Clone to modify path
        segment_info_mut.path = Some(segment_path);

        {
            let mut segments = self.segments.write().map_err(map_poison_err)?;
            let existing_idx = segments.iter().position(|s| s.id == segment_info_mut.id);
            if let Some(idx) = existing_idx {
                segments[idx] = segment_info_mut;
            } else {
                // Only push if it's genuinely new? Save logic might be called on existing too.
                // Let's assume adding is fine, might duplicate if called incorrectly.
                segments.push(segment_info_mut);
            }
            // Sort segments? Maybe by start_index or start_time?
            // segments.sort_by_key(|s| s.start_index);
            Self::save_segment_index(&self.base_path, &segments)?;
        }
        Ok(())
    }

    /// Rotate the active segment
    fn rotate_segment(&self) -> Result<()> {
        let new_segment = Self::create_new_segment()?;
        let old_segment_guard = { // Scope for the first lock
             let mut active_segment_guard = self.active_segment.lock().map_err(map_poison_err)?;
             // Swap the segment out, save the old one after releasing the lock
             std::mem::replace(&mut *active_segment_guard, new_segment)
        }; // Lock released here

        // Now save the old segment without holding the lock
        self.save_segment(&old_segment_guard)?;

        Ok(())
    }

    /// Check if we need to rotate the active segment
    fn check_rotate(&self) -> Result<bool> {
        let active_segment_guard = self.active_segment.lock().map_err(map_poison_err)?;
        let info = active_segment_guard.info();

        let size_limit_reached = self.config.max_segment_size > 0
            && info.size_bytes >= (self.config.max_segment_size as u64);

        let entry_limit_reached = self.config.max_entries_per_segment > 0
            && info.entry_count >= self.config.max_entries_per_segment;

        Ok(size_limit_reached || entry_limit_reached)
    }

    /// Get a segment by ID, loading from disk if necessary and managing cache
    fn get_segment(&self, segment_id: &str) -> Result<Arc<Mutex<LogSegment>>> {
        // Check cache first
        {
            let cache = self.segment_cache.read().map_err(map_poison_err)?;
            if let Some(segment_arc) = cache.get(segment_id) {
                 // Update LRU queue
                 {
                     let mut queue = self.segment_queue.lock().map_err(map_poison_err)?;
                     queue.retain(|id| id != segment_id); // Remove if exists
                     queue.push_back(segment_id.to_string()); // Add to end
                 }
                 return Ok(segment_arc.clone());
             }
        } // Read lock released

        // Not in cache, load from disk
        let segment_info = { // Scope for segments lock
            let segments = self.segments.read().map_err(map_poison_err)?;
            segments.iter()
                .find(|seg| seg.id == segment_id)
                .cloned() // Clone the info needed
                .ok_or_else(|| EngineError::StorageError(format!("Segment info not found for ID: {}", segment_id)))?
        }; // Read lock released

        // Get the path from segment_info, clone it to avoid borrowing issues
        let path_clone = segment_info.path.clone();
        let segment_path = path_clone.as_ref()
            .ok_or_else(|| EngineError::StorageError(format!("Segment info missing path for ID: {}", segment_info.id)))?;

        let segment = Self::load_segment_from_file(segment_path, segment_info)?;
        let segment_arc = Arc::new(Mutex::new(segment));

        // Add to cache
        {
            let mut cache = self.segment_cache.write().map_err(map_poison_err)?;
            let mut queue = self.segment_queue.lock().map_err(map_poison_err)?;

            // Use a reasonable default cache size if not defined in config
            let cache_size = 10; // Default cache size
            
            while cache.len() >= cache_size {
                if let Some(lru_id) = queue.pop_front() {
                    cache.remove(&lru_id);
                } else {
                    break; // Queue empty, should not happen if cache is full
                }
            }

            // Add new segment
            if cache.insert(segment_id.to_string(), segment_arc.clone()).is_none() {
                 // Only add to queue if it was a new insertion
                 queue.push_back(segment_id.to_string());
             } else {
                // If it was already inserted concurrently, ensure it's at the end of queue
                queue.retain(|id| id != segment_id); // Remove if exists
                queue.push_back(segment_id.to_string()); // Add to end
             }
        }

        Ok(segment_arc)
    }


    /// Ensure an entry has a valid hash (Placeholder)
    fn ensure_valid_hash(&self, _entry: &mut LogEntry) -> Result<()> {
        // TODO: Implement actual hashing and verification logic on LogEntry
        // if self.config.auto_hash && entry.entry_hash.is_none() {
        //     // entry.calculate_hash()?; // Call the (to be implemented) method
        // }
        // if self.config.enforce_hash_verification {
        //     // entry.verify_hash()?; // Call the (to be implemented) method
        // }
        Ok(()) // No-op for now
    }

    /// Helper method to flush all segments to disk
     fn flush_storage(&self) -> Result<()> {
        // Save active segment
        { // Scope for lock
            let active_segment_guard = self.active_segment.lock().map_err(map_poison_err)?;
            self.save_segment(&active_segment_guard)?;
        } // Lock released

        // Save cached segments (optional, might already be saved)
        // This could be expensive if cache is large
        // {
        //     let cache = self.segment_cache.read().map_err(map_poison_err)?;
        //     for segment_mutex in cache.values() {
        //         let segment_guard = segment_mutex.lock().map_err(map_poison_err)?;
        //         self.save_segment(&segment_guard)?;
        //     }
        // }

         // Ensure segment index is saved
         { // Scope for lock
             let segments_guard = self.segments.read().map_err(map_poison_err)?;
             Self::save_segment_index(&self.base_path, &segments_guard)?;
         } // Lock released
         Ok(())
     }

    /// Append a log entry
    fn append_log(&self, entry: LogEntry) -> Result<()> {
        // Check if we need to rotate first
        let should_rotate = self.check_rotate()?;
        
        if should_rotate {
            self.rotate_segment()?;
        }
        
        // Now append to the active segment
        {
            let mut active_segment_guard = self.active_segment.lock().map_err(map_poison_err)?;
            active_segment_guard.append(entry)?; // Using append method instead
            
            // Check if we need to flush - auto_flush field may not exist
            // Safely flush to ensure data is written
            active_segment_guard.flush()?;
        }
        
        Ok(())
    }
}

impl fmt::Debug for FileLogStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileLogStorage")
            .field("config", &self.config)
            .field("base_path", &self.base_path)
            // Avoid printing locks directly
            .field("segment_count", &self.segments.read().map_err(|_| fmt::Error)?.len())
            .field("cache_size", &self.segment_cache.read().map_err(|_| fmt::Error)?.len())
            .field("entry_count", &self.entry_count.read().map_err(|_| fmt::Error)?)
            .finish()
    }
}

#[async_trait::async_trait]
impl LogStorage for FileLogStorage {
    async fn append_entry(&self, entry: LogEntry) -> Result<()> {
        // Delegate to sync version - this is a compatibility layer
        self.append_log(entry)
    }

    async fn get_all_entries(&self) -> Result<Vec<LogEntry>> {
        self.flush_storage()?; // Ensure all segments are saved before reading all
        let mut all_entries = Vec::new();
        let segments_infos = self.segments.read().map_err(map_poison_err)?;

        for info in segments_infos.iter() {
             // If segment info doesn't contain path, construct it
             let segment_path = info.path.as_ref()
                .cloned() // Clone the PathBuf if it exists
                .unwrap_or_else(|| self.base_path.join(format!("{}.log", info.id))); // Construct if None

             // Load segment - avoid using get_segment to prevent cache pollution/LRU updates
             match Self::load_segment_from_file(&segment_path, info.clone()) {
                 Ok(segment) => {
                     all_entries.extend(segment.entries().iter().cloned()); // Use cloned()
                 },
                 Err(e) => {
                     // Log error and continue? Or fail? For get_all, maybe try best effort.
                     warn!("Failed to load segment {}: {}", info.id, e);
                 }
             }
        }

        // Include active segment entries - these might duplicate entries if flush didn't happen exactly before read.
        // For a consistent view, locking everything might be needed, or rely on flush.
        // Let's assume flush makes the saved segments authoritative.
        // {
        //     let active_guard = self.active_segment.lock().map_err(map_poison_err)?;
        //     all_entries.extend(active_guard.entries().iter().cloned());
        // }
        // Deduplication needed if active is included after flush.

        Ok(all_entries)
    }

    async fn get_entries(&self, start: usize, end: usize) -> Result<Vec<LogEntry>> {
        // Inefficient: load all and slice. Requires indexing/metadata for efficiency.
        let all_entries = self.get_all_entries().await?;
        if start >= all_entries.len() { return Ok(Vec::new()); }
        let effective_end = end.min(all_entries.len());
        Ok(all_entries[start..effective_end].to_vec())
    }

    async fn get_entry_count(&self) -> Result<usize> {
        Ok(*self.entry_count.read().map_err(map_poison_err)?)
    }

    async fn clear(&self) -> Result<()> {
        // Ensure data is flushed before deleting
        self.flush_storage()?;

        let paths_to_delete = { // Scope for segments lock
             let mut segments = self.segments.write().map_err(map_poison_err)?;
             let paths: Vec<PathBuf> = segments.iter().filter_map(|info| info.path.clone()).collect();
             segments.clear();
             // Save the now empty segment index
             Self::save_segment_index(&self.base_path, &segments)?;
             paths
        }; // Write lock released

        // Delete segment files
        for path in paths_to_delete {
             fs::remove_file(&path)
                .map_err(|e| EngineError::StorageError(format!("Failed to remove segment file {:?}: {}", path, e)))?;
        }
        // Also remove the index file itself
         fs::remove_file(self.base_path.join("segments.json"))
             .map_err(|e| EngineError::StorageError(format!("Failed to remove segments.json: {}", e)))?;


        // Clear active segment by replacing it with a new one
        { // Scope for lock
            let mut active_segment_guard = self.active_segment.lock().map_err(map_poison_err)?;
            *active_segment_guard = Self::create_new_segment()?; // Create a new empty active segment
        } // Lock released

        // Clear cache and queue
        { // Scope for lock
            self.segment_cache.write().map_err(map_poison_err)?.clear();
        } // Lock released
        { // Scope for lock
            self.segment_queue.lock().map_err(map_poison_err)?.clear();
        } // Lock released
        { // Scope for lock
            let mut count = self.entry_count.write().map_err(map_poison_err)?;
            *count = 0; // Reset entry count
        } // Lock released

        Ok(())
    }


    // --- Synchronous Wrapper Methods ---

    fn append(&self, entry: LogEntry) -> Result<()> {
        block_on(self.append_entry(entry))
    }

    fn append_batch(&self, entries: Vec<LogEntry>) -> Result<()> {
        // block_on requires Future + Send, the async block is Send
        block_on(async {
            for entry in entries {
                self.append_entry(entry).await?;
            }
            Ok(()) // Explicit type annotation needed if Result alias is ambiguous
        })
    }

    fn read(&self, start: usize, count: usize) -> Result<Vec<LogEntry>> {
        block_on(self.get_entries(start, start + count))
    }

    fn read_time_range(&self, start_time: u64, end_time: u64) -> Result<Vec<LogEntry>> {
         // Inefficient fallback: load all and filter
         let all_entries = block_on(self.get_all_entries())?;
         Ok(all_entries.into_iter()
            .filter(|e| {
                let ts = e.timestamp.to_millis(); // Assuming Timestamp has to_millis()
                ts >= start_time && ts <= end_time
            })
            .collect())
    }

    fn entry_count(&self) -> Result<usize> {
        block_on(self.get_entry_count())
    }

    fn get_entry_by_id(&self, id: &str) -> Result<Option<LogEntry>> {
        let all_entries = block_on(self.get_all_entries())?;
        Ok(all_entries.into_iter().find(|e| e.id == id))
    }

    fn get_entries_by_trace(&self, trace_id: &str) -> Result<Vec<LogEntry>> {
         let all_entries = block_on(self.get_all_entries())?;
         Ok(all_entries.into_iter()
             .filter(|e| e.trace_id.as_ref().map_or(false, |t| t.0 == trace_id)) // Compare t.0 (String) with trace_id (&str)
             .collect())
     }


    fn get_entry_by_hash(&self, hash: &str) -> Result<Option<LogEntry>> {
        let all_entries = block_on(self.get_all_entries())?;
        Ok(all_entries.into_iter().find(|e| e.entry_hash.as_ref().map_or(false, |h| h == hash)))
    }

    fn find_entries_by_type(&self, entry_type: EntryType) -> Result<Vec<LogEntry>> {
        let all_entries = block_on(self.get_all_entries())?;
        Ok(all_entries.into_iter().filter(|e| e.entry_type == entry_type).collect())
    }

    // fn find_entries_in_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<LogEntry>> {
    //     // Requires time_utils or similar for conversion
    //     // let all_entries = block_on(self.get_all_entries())?;
    //     // Ok(all_entries.into_iter()
    //     //     .filter(|e| {
    //     //         // Assuming time_utils::timestamp_to_datetime exists
    //     //         // let ts = time_utils::timestamp_to_datetime(e.timestamp.clone());
    //     //         // ts >= start && ts <= end
    //     //         false // Placeholder
    //     //     })
    //     //     .collect())
    //     unimplemented!("find_entries_in_time_range requires time conversion utility");
    // }


    fn rotate(&self) -> Result<()> {
        self.rotate_segment()
    }

    fn compact(&self) -> Result<()> {
        // Compaction logic (complex, likely involves reading/writing segments)
        warn!("Compaction is not yet implemented for FileLogStorage.");
        Ok(()) // No-op for now
    }

    fn close(&self) -> Result<()> {
        self.flush_storage()
    }

    // This duplicates get_entries_by_trace - remove one? Keep the one accepting &TraceId.
    fn find_entries_by_trace_id(&self, trace_id: &causality_types::TraceId) -> Result<Vec<LogEntry>> {
         let all_entries = block_on(self.get_all_entries())?;
         Ok(all_entries.into_iter()
             .filter(|e| e.trace_id.as_ref() == Some(trace_id)) // Direct comparison should work if TraceId impl PartialEq
             .collect())
     }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env; // Use std::env for temp_dir
    use tempfile::tempdir;
    use std::collections::HashMap;
    use causality_types::{Timestamp, TraceId}; // Import TraceId
    use crate::log::event_entry::{EventEntry, EventSeverity}; // Import EventEntry/Severity

    // Helper function to create a test entry
    fn create_test_entry(id: &str) -> LogEntry {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        let trace_id_str = format!("test_trace_{}", id);
        let trace_id = TraceId::from_str(&trace_id_str); // Use from_str instead of new

        LogEntry {
            id: id.to_string(),
            timestamp: Timestamp(now), // Assuming Timestamp is a tuple struct
            entry_type: EntryType::Event, // Assuming Event is a variant
            data: EntryData::Event(EventEntry { // Assuming Event is a variant
                event_name: format!("test_event_{}", id),
                severity: EventSeverity::Info, // Assuming Info is a variant
                component: "test".to_string(),
                details: serde_json::json!({"id": id}),
                resources: None,
                domains: None,
            }),
            trace_id: Some(trace_id),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        }
    }

    fn setup_test_dir(name: &str) -> PathBuf {
        let path = env::temp_dir().join("causality_tests").join(name);
        // Clean up potential leftovers from previous runs
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("Failed to create test directory");
        path
    }

    fn cleanup_test_dir(path: &Path) {
         let _ = fs::remove_dir_all(path);
     }


    #[test]
    fn test_file_storage_operations() -> Result<()> {
        let path = setup_test_dir("file_storage_ops");

        // Create a storage config
        let mut config = StorageConfig::default();
        config.max_entries_per_segment = 5; // Set low for rotation testing
        config.format = StorageFormat::Json;
        config.cache_size = 2; // Test caching

        // Create storage
        let storage = FileLogStorage::new(&path, config)?;

        // Initial state
        assert_eq!(storage.entry_count()?, 0);

        // Create test entries
        let entries = (0..10).map(|i| {
            create_test_entry(&format!("entry_{}", i))
        }).collect::<Vec<_>>();

        // Add entries using sync wrapper
        storage.append_batch(entries.clone())?;

        // Check count
        assert_eq!(storage.entry_count()?, 10);

        // Flush to ensure everything is saved
        storage.flush_storage()?;

        // Verify segment files exist (expecting at least 2 due to rotation)
        let segment_files: Vec<_> = fs::read_dir(&path)?
            .filter_map(std::result::Result::ok) // Use std::result::Result::ok
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "log"))
            .collect();
         // With max_entries=5, 10 entries should create seg0 (5 entries) + active (5 entries)
         // Flush saves active, so we expect 2 .log files.
        assert!(segment_files.len() >= 2, "Expected at least 2 segment files, found {}", segment_files.len());

        // Read entries
        let read_entries = storage.read(0, 10)?;
        assert_eq!(read_entries.len(), 10);
        assert_eq!(read_entries[0].id, "entry_0");
        assert_eq!(read_entries[9].id, "entry_9");


        // Read with pagination
        let first_page = storage.read(0, 3)?;
        assert_eq!(first_page.len(), 3);
        assert_eq!(first_page[0].id, "entry_0");

        let second_page = storage.read(3, 3)?;
        assert_eq!(second_page.len(), 3);
        assert_eq!(second_page[0].id, "entry_3");

        // Read beyond end
        let beyond_end = storage.read(10, 2)?;
        assert_eq!(beyond_end.len(), 0);

        // Test get_entry_by_id
        let entry5 = storage.get_entry_by_id("entry_5")?;
        assert!(entry5.is_some());
        assert_eq!(entry5.unwrap().id, "entry_5");

        // Test find_entries_by_trace_id (using the wrapper)
        let trace_id_3 = TraceId::from_str("test_trace_entry_3");
        let trace_entries = storage.find_entries_by_trace_id(&trace_id_3)?;
        assert_eq!(trace_entries.len(), 1);
        assert_eq!(trace_entries[0].id, "entry_3");


        // Close the storage (which also flushes)
        storage.close()?;

        // Re-open storage and verify data persists
        let config2 = StorageConfig::default(); // Use default config for re-opening
        let storage2 = FileLogStorage::new(&path, config2)?;
        assert_eq!(storage2.entry_count()?, 10);
        let re_read_entries = storage2.read(0, 10)?;
        assert_eq!(re_read_entries.len(), 10);
        assert_eq!(re_read_entries[5].id, "entry_5");

        // Test clear
        block_on(storage2.clear())?; // Use block_on for async clear
        assert_eq!(storage2.entry_count()?, 0);
        let empty_entries = storage2.read(0, 1)?;
        assert!(empty_entries.is_empty());

        // Verify segment files are gone after clear
         let segment_files_after_clear: Vec<_> = fs::read_dir(&path)?
             .filter_map(std::result::Result::ok) // Use std::result::Result::ok
             .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "log" || ext == "bin" || ext == "cbor" || ext == "json"))
             .collect();
        assert!(segment_files_after_clear.is_empty(), "Expected no segment files after clear, found {}", segment_files_after_clear.len());


        cleanup_test_dir(&path);
        Ok(())
    }

    #[test]
    fn test_segment_rotation() -> Result<()> {
         let path = setup_test_dir("segment_rotation");

        // Create storage with small segment size and entry count
        let mut config = StorageConfig::default();
        // config.max_segment_size = 10 * 1024; // Test rotation by size later if needed
        config.max_entries_per_segment = 3; // Rotate every 3 entries
        config.format = StorageFormat::Json; // Use Json for easier debugging if needed

        let storage = FileLogStorage::new(&path, config)?;

        // Add 7 entries to trigger rotation multiple times
        for i in 0..7 {
            let entry = create_test_entry(&format!("rot-entry-{}", i));
            storage.append(entry)?; // Use sync append
        }

        // Check count
        assert_eq!(storage.entry_count()?, 7);

        // Flush to ensure active segment is saved
        storage.flush_storage()?;

        // Check number of segment files created (0-2, 3-5, active=6) -> Expect 3 files
         let segment_files: Vec<_> = fs::read_dir(&path)?
             .filter_map(std::result::Result::ok) // Use std::result::Result::ok
             .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "log")) // Assuming .log extension
             .collect();
        assert!(segment_files.len() >= 3, "Expected at least 3 segment files due to rotation, found {}", segment_files.len());


        // Verify content by reading all
        let all_read = storage.read(0, 10)?; // Read more than expected
        assert_eq!(all_read.len(), 7);
        assert_eq!(all_read[0].id, "rot-entry-0");
        assert_eq!(all_read[6].id, "rot-entry-6");

        cleanup_test_dir(&path);
        Ok(())
    }
} 