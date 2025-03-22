// File storage implementation for Causality Unified Log System
//
// This module provides a file-based implementation of the LogStorage trait.

use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::log::entry::LogEntry;
use crate::log::storage::{LogStorage, StorageConfig, StorageFormat};
use crate::log::segment::{LogSegment, SegmentInfo, generate_segment_id};

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

impl FileLogStorage {
    /// Create a new file storage with the given base path and configuration
    pub fn new<P: AsRef<Path>>(base_path: P, config: StorageConfig) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        // Create the base directory if it doesn't exist
        fs::create_dir_all(&base_path)?;
        
        // Load existing segments or create a new one
        let mut segments = Self::load_segments(&base_path)?;
        
        let active_segment = if segments.is_empty() {
            // Create a new segment
            Self::create_new_segment()?
        } else {
            // Load the latest segment
            let latest_segment = segments.last().unwrap();
            Self::load_segment_from_file(&latest_segment.path, latest_segment.clone())?
        };
        
        // Calculate total entry count
        let entry_count = segments.iter().map(|seg| seg.entry_count).sum();
        
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
        config.storage_dir = Some(base_path.as_ref().to_path_buf());
        Self::new(base_path, config)
    }
    
    /// Load existing segments from the base path
    fn load_segments(base_path: &Path) -> Result<Vec<SegmentInfo>> {
        let meta_file = base_path.join("segments.json");
        
        if !meta_file.exists() {
            return Ok(Vec::new());
        }
        
        let file = File::open(meta_file)?;
        let segments: Vec<SegmentInfo> = serde_json::from_reader(file)?;
        
        Ok(segments)
    }
    
    /// Save segment index to disk
    fn save_segment_index(base_path: &Path, segments: &[SegmentInfo]) -> Result<()> {
        let meta_file = base_path.join("segments.json");
        let file = File::create(meta_file)?;
        serde_json::to_writer(file, segments)?;
        
        Ok(())
    }
    
    /// Create a new segment
    fn create_new_segment() -> Result<LogSegment> {
        let segment_id = generate_segment_id();
        Ok(LogSegment::new(segment_id))
    }
    
    /// Load a segment from a file
    fn load_segment_from_file(path: &Path, info: SegmentInfo) -> Result<LogSegment> {
        if !path.exists() {
            return Err(Error::Storage(format!("Segment file not found: {:?}", path)));
        }
        
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        let format = if path.extension().map_or(false, |ext| ext == "json") {
            StorageFormat::Json
        } else {
            StorageFormat::Binary
        };
        
        match format {
            StorageFormat::Json => {
                let mut segment = LogSegment::new(info.id.clone());
                
                // Read line by line
                let content = String::from_utf8(buffer)?;
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let entry: LogEntry = serde_json::from_str(line)?;
                    segment.append(entry)?;
                }
                
                Ok(segment)
            }
            StorageFormat::Binary => {
                // Decode using bincode
                let entries: Vec<LogEntry> = bincode::deserialize(&buffer)?;
                
                let mut segment = LogSegment::new(info.id.clone());
                for entry in entries {
                    segment.append(entry)?;
                }
                
                Ok(segment)
            }
            StorageFormat::Cbor => {
                // Decode using ciborium
                let entries: Vec<LogEntry> = ciborium::de::from_reader(&buffer[..])?;
                
                let mut segment = LogSegment::new(info.id.clone());
                for entry in entries {
                    segment.append(entry)?;
                }
                
                Ok(segment)
            }
        }
    }
    
    /// Save a segment to disk
    fn save_segment(&self, segment: &LogSegment) -> Result<()> {
        let segment_info = segment.info()?;
        let segment_path = self.base_path.join(format!("{}.{}", 
            segment_info.id, 
            match self.config.format {
                StorageFormat::Json => "json",
                StorageFormat::Binary => "bin",
                StorageFormat::Cbor => "cbor",
            }
        ));
        
        let mut file = File::create(&segment_path)?;
        
        match self.config.format {
            StorageFormat::Json => {
                // Write entries as JSON, one per line
                for entry in segment.entries()? {
                    let json = serde_json::to_string(&entry)?;
                    file.write_all(json.as_bytes())?;
                    file.write_all(b"\n")?;
                }
            }
            StorageFormat::Binary => {
                // Write as a single bincode blob
                let entries = segment.entries()?;
                let data = bincode::serialize(&entries)?;
                file.write_all(&data)?;
            }
            StorageFormat::Cbor => {
                // Write as CBOR using ciborium
                let entries = segment.entries()?;
                ciborium::ser::into_writer(&entries, &mut file)?;
            }
        }
        
        if self.config.sync_on_write {
            file.sync_all()?;
        }
        
        // Update segment info
        let mut segment_info = segment_info;
        segment_info.path = segment_path;
        
        // Update segments list
        {
            let mut segments = self.segments.write().map_err(|_| {
                Error::Storage("Failed to acquire write lock on segments".to_string())
            })?;
            
            // Replace or add the segment info
            let existing_idx = segments.iter().position(|s| s.id == segment_info.id);
            if let Some(idx) = existing_idx {
                segments[idx] = segment_info;
            } else {
                segments.push(segment_info);
            }
            
            // Save the segment index
            Self::save_segment_index(&self.base_path, &segments)?;
        }
        
        Ok(())
    }
    
    /// Rotate the active segment
    fn rotate_segment(&self) -> Result<()> {
        // Save the current active segment
        {
            let active_segment = self.active_segment.lock().map_err(|_| {
                Error::Storage("Failed to acquire lock on active segment".to_string())
            })?;
            
            self.save_segment(&active_segment)?;
        }
        
        // Create a new active segment
        {
            let mut active_segment = self.active_segment.lock().map_err(|_| {
                Error::Storage("Failed to acquire lock on active segment".to_string())
            })?;
            
            *active_segment = Self::create_new_segment()?;
        }
        
        Ok(())
    }
    
    /// Check if segment rotation is needed
    fn check_rotate(&self) -> Result<bool> {
        let active_segment = self.active_segment.lock().map_err(|_| {
            Error::Storage("Failed to acquire lock on active segment".to_string())
        })?;
        
        let info = active_segment.info()?;
        
        // Check if the segment is too large
        if self.config.max_segment_size > 0 && info.size_bytes >= self.config.max_segment_size {
            return Ok(true);
        }
        
        // Check if the segment has too many entries
        if self.config.max_segment_entries > 0 && info.entry_count >= self.config.max_segment_entries {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Get a segment by ID
    fn get_segment(&self, segment_id: &str) -> Result<Arc<Mutex<LogSegment>>> {
        // First check the cache
        {
            let cache = self.segment_cache.read().map_err(|_| {
                Error::Storage("Failed to acquire read lock on segment cache".to_string())
            })?;
            
            if let Some(segment) = cache.get(segment_id) {
                // Update segment queue for LRU behavior
                {
                    let mut queue = self.segment_queue.lock().map_err(|_| {
                        Error::Storage("Failed to acquire lock on segment queue".to_string())
                    })?;
                    
                    // Remove if already in queue
                    if let Some(idx) = queue.iter().position(|id| id == segment_id) {
                        queue.remove(idx);
                    }
                    
                    // Add to the end
                    queue.push_back(segment_id.to_string());
                }
                
                return Ok(segment.clone());
            }
        }
        
        // Not in cache, need to load from disk
        let segments = self.segments.read().map_err(|_| {
            Error::Storage("Failed to acquire read lock on segments".to_string())
        })?;
        
        let segment_info = segments.iter()
            .find(|seg| seg.id == segment_id)
            .ok_or_else(|| Error::Storage(format!("Segment not found: {}", segment_id)))?;
        
        let segment = Self::load_segment_from_file(&segment_info.path, segment_info.clone())?;
        let segment = Arc::new(Mutex::new(segment));
        
        // Add to cache
        {
            let mut cache = self.segment_cache.write().map_err(|_| {
                Error::Storage("Failed to acquire write lock on segment cache".to_string())
            })?;
            
            let mut queue = self.segment_queue.lock().map_err(|_| {
                Error::Storage("Failed to acquire lock on segment queue".to_string())
            })?;
            
            // Check if cache is full
            if cache.len() >= self.config.in_memory_segments && !queue.is_empty() {
                // Remove least recently used segment
                if let Some(id) = queue.pop_front() {
                    cache.remove(&id);
                }
            }
            
            // Add to cache
            cache.insert(segment_id.to_string(), segment.clone());
            queue.push_back(segment_id.to_string());
        }
        
        Ok(segment)
    }
}

impl LogStorage for FileLogStorage {
    fn append(&self, mut entry: LogEntry) -> Result<()> {
        // Get config for hash verification
        let config = &self.config;
        
        // Ensure the entry has a valid hash if required
        if config.enforce_hash_verification {
            self.ensure_valid_hash(&mut entry)?;
        }
        
        // Verify the hash before storing
        self.verify_entry_hash(&entry, config)?;
        
        // Get a lock on the active segment
        let mut active_segment = self.active_segment.lock().map_err(|e| {
            Error::LogError(format!("Failed to lock active segment: {}", e))
        })?;
        
        // Check if we need to rotate
        if active_segment.is_full(&self.config) {
            drop(active_segment); // Release the lock before rotating
            self.rotate_segment()?;
            active_segment = self.active_segment.lock().map_err(|e| {
                Error::LogError(format!("Failed to lock active segment after rotation: {}", e))
            })?;
        }
        
        // Append the entry to the active segment
        active_segment.append(entry)?;
        
        // Update entry count
        let mut count = self.entry_count.write().map_err(|e| {
            Error::LogError(format!("Failed to lock entry count: {}", e))
        })?;
        *count += 1;
        
        // Check if we need to sync to disk
        if self.config.sync_on_write {
            active_segment.flush()?;
        }
        
        Ok(())
    }
    
    fn read(&self, start: usize, count: usize) -> Result<Vec<LogEntry>> {
        if count == 0 {
            return Ok(Vec::new());
        }
        
        let total_entries = self.entry_count()?;
        
        if start >= total_entries {
            return Ok(Vec::new());
        }
        
        let end = std::cmp::min(start + count, total_entries);
        let entries_to_read = end - start;
        
        // Find which segments contain the requested entries
        let segments = self.segments.read().map_err(|_| {
            Error::Storage("Failed to acquire read lock on segments".to_string())
        })?;
        
        // Include active segment
        let active_segment_info = {
            let active_segment = self.active_segment.lock().map_err(|_| {
                Error::Storage("Failed to acquire lock on active segment".to_string())
            })?;
            
            active_segment.info()?
        };
        
        // Combine segment info
        let mut all_segments = segments.clone();
        all_segments.push(active_segment_info);
        
        // Sort by create time
        all_segments.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        let mut result = Vec::with_capacity(entries_to_read);
        let mut current_pos = 0;
        
        for segment_info in all_segments {
            let segment_start = current_pos;
            let segment_end = segment_start + segment_info.entry_count;
            
            // Skip if the segment is completely before the start
            if segment_end <= start {
                current_pos = segment_end;
                continue;
            }
            
            // Stop if the segment is completely after the end
            if segment_start >= end {
                break;
            }
            
            // Get the segment
            let segment = if segment_info.id == active_segment_info.id {
                // Active segment
                let active_segment = self.active_segment.lock().map_err(|_| {
                    Error::Storage("Failed to acquire lock on active segment".to_string())
                })?;
                
                active_segment.entries()?
            } else {
                // Get from cache/disk
                let segment = self.get_segment(&segment_info.id)?;
                let segment = segment.lock().map_err(|_| {
                    Error::Storage("Failed to acquire lock on segment".to_string())
                })?;
                
                segment.entries()?
            };
            
            // Calculate the range of entries to read from this segment
            let seg_read_start = if start > segment_start {
                start - segment_start
            } else {
                0
            };
            
            let seg_read_end = std::cmp::min(segment_info.entry_count, seg_read_start + (end - std::cmp::max(start, segment_start)));
            
            // Add the entries
            if seg_read_start < seg_read_end && seg_read_start < segment.len() {
                let seg_read_end = std::cmp::min(seg_read_end, segment.len());
                result.extend_from_slice(&segment[seg_read_start..seg_read_end]);
            }
            
            current_pos = segment_end;
            
            if result.len() >= entries_to_read {
                break;
            }
        }
        
        Ok(result)
    }
    
    fn entry_count(&self) -> Result<usize> {
        let count = self.entry_count.read().map_err(|_| {
            Error::Storage("Failed to acquire read lock on entry count".to_string())
        })?;
        
        Ok(*count)
    }
    
    fn flush(&self) -> Result<()> {
        // Save the active segment
        {
            let active_segment = self.active_segment.lock().map_err(|_| {
                Error::Storage("Failed to acquire lock on active segment".to_string())
            })?;
            
            self.save_segment(&active_segment)?;
        }
        
        Ok(())
    }
    
    fn close(&self) -> Result<()> {
        // Flush to ensure everything is saved
        self.flush()?;
        
        // Clear the cache
        {
            let mut cache = self.segment_cache.write().map_err(|_| {
                Error::Storage("Failed to acquire write lock on segment cache".to_string())
            })?;
            
            let mut queue = self.segment_queue.lock().map_err(|_| {
                Error::Storage("Failed to acquire lock on segment queue".to_string())
            })?;
            
            cache.clear();
            queue.clear();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::collections::HashMap;
    use crate::log::entry::{EntryType, EntryData, EventEntry, EventSeverity};
    
    #[test]
    fn test_file_storage_operations() -> Result<()> {
        // Create a temporary directory
        let dir = tempdir()?;
        let path = dir.path();
        
        // Create a storage config
        let config = StorageConfig::default()
            .with_max_segment_entries(5)
            .with_format(StorageFormat::Json);
        
        // Create storage
        let storage = FileLogStorage::new(path, config)?;
        
        // Initial state
        assert_eq!(storage.entry_count()?, 0);
        assert_eq!(storage.read(0, 10)?.len(), 0);
        
        // Create test entries
        let entries = (0..10).map(|i| {
            let event_entry = EventEntry {
                event_name: format!("test_event_{}", i),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"index": i}),
                resources: None,
                domains: None,
            };
            
            LogEntry {
                id: format!("entry_{}", i),
                timestamp: Utc::now(),
                entry_type: EntryType::Event,
                data: EntryData::Event(event_entry),
                trace_id: Some("test_trace".to_string()),
                parent_id: None,
                metadata: HashMap::new(),
                entry_hash: None,
            }
        }).collect::<Vec<_>>();
        
        // Add entries
        for entry in entries.clone() {
            storage.append(entry)?;
        }
        
        // Check count
        assert_eq!(storage.entry_count()?, 10);
        
        // Flush to ensure everything is saved
        storage.flush()?;
        
        // Read entries
        let read_entries = storage.read(0, 10)?;
        assert_eq!(read_entries.len(), 10);
        
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
        
        // Close the storage
        storage.close()?;
        
        // Reopen and verify data is still there
        let storage2 = FileLogStorage::new(path, config)?;
        assert_eq!(storage2.entry_count()?, 10);
        
        let read_entries2 = storage2.read(0, 10)?;
        assert_eq!(read_entries2.len(), 10);
        
        Ok(())
    }
} 