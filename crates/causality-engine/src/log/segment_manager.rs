// Log segment management
// Original file: src/log/segment_manager.rs

// Log Segment Manager for Causality Unified Log System
//
// This module provides functionality for managing multiple log segments,
// including rotation, indexing, and retrieval.

use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, Mutex};
use std::fs;
use chrono::{DateTime, Utc, Duration};

use causality_error::{EngineResult as Result, EngineError as Error};
use crate::log::LogEntry;
use crate::log::segment::{LogSegment, SegmentInfo, generate_segment_id};
use crate::log::storage::StorageConfig;
use causality_types::Timestamp;
use crate::error_conversions::convert_boxed_error;

/// Criteria for rotating log segments
#[derive(Clone)]
pub enum RotationCriteria {
    /// Rotate after a certain number of entries
    EntryCount(usize),
    /// Rotate after reaching a certain size (in bytes)
    Size(usize),
    /// Rotate after a certain time interval
    TimeInterval(Duration),
    /// Custom rotation function
    Custom(Arc<dyn Fn(&LogSegment) -> bool + Send + Sync>),
}

/// Options for segment manager
#[derive(Clone)]
pub struct SegmentManagerOptions {
    /// Base directory for storing segments
    pub base_dir: PathBuf,
    /// Maximum number of active segments in memory
    pub max_active_segments: usize,
    /// Whether to compress inactive segments
    pub compress_inactive: bool,
    /// Rotation criteria
    pub rotation_criteria: Vec<RotationCriteria>,
    /// Segment naming pattern
    pub segment_name_pattern: String,
    /// Whether to auto-flush on rotation
    pub auto_flush: bool,
    /// Segment index directory
    pub index_dir: Option<PathBuf>,
}

impl Default for SegmentManagerOptions {
    fn default() -> Self {
        SegmentManagerOptions {
            base_dir: PathBuf::from("logs"),
            max_active_segments: 2,
            compress_inactive: true,
            rotation_criteria: vec![
                RotationCriteria::EntryCount(10000),
                RotationCriteria::Size(10 * 1024 * 1024), // 10MB
                RotationCriteria::TimeInterval(Duration::days(1)),
            ],
            segment_name_pattern: "segment_{timestamp}".to_string(),
            auto_flush: true,
            index_dir: None,
        }
    }
}

/// Index entry for a log segment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SegmentIndexEntry {
    /// The segment info
    pub info: SegmentInfo,
    /// First entry timestamp
    pub first_entry_time: Timestamp,
    /// Last entry timestamp
    pub last_entry_time: Timestamp,
    /// Number of entries
    pub entry_count: usize,
    /// Path to segment file
    pub path: PathBuf,
}

/// Manager for log segments
pub struct LogSegmentManager {
    /// Options for the segment manager
    options: SegmentManagerOptions,
    /// Currently active segment for writing
    active_segment: Arc<Mutex<LogSegment>>,
    /// Cached segments (recently used)
    cached_segments: Arc<RwLock<HashMap<String, Arc<Mutex<LogSegment>>>>>,
    /// Index of all segments
    segment_index: Arc<RwLock<BTreeMap<Timestamp, SegmentIndexEntry>>>,
    /// Storage configuration
    storage_config: StorageConfig,
    /// Last rotation timestamp
    last_rotation: Arc<Mutex<DateTime<Utc>>>,
}

impl LogSegmentManager {
    /// Create a new segment manager with the given options
    pub fn new(options: SegmentManagerOptions, storage_config: StorageConfig) -> Result<Self> {
        // Create base directory if it doesn't exist
        if !options.base_dir.exists() {
            fs::create_dir_all(&options.base_dir)?;
        }
        
        // Create index directory if specified
        if let Some(index_dir) = &options.index_dir {
            if !index_dir.exists() {
                fs::create_dir_all(index_dir)?;
            }
        }
        
        // Create initial active segment
        let segment_id = generate_segment_id();
        let segment_path = options.base_dir.join(format!("{}.log", segment_id));
        let mut segment = LogSegment::new(segment_id);
        segment.set_path(&segment_path);
        
        let active_segment = Arc::new(Mutex::new(segment));
        
        // Initialize segment index
        let segment_index = Arc::new(RwLock::new(BTreeMap::new()));
        
        // Initialize cached segments
        let cached_segments = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(LogSegmentManager {
            options,
            active_segment,
            cached_segments,
            segment_index,
            storage_config,
            last_rotation: Arc::new(Mutex::new(Utc::now())),
        })
    }
    
    /// Append a log entry to the active segment
    pub fn append(&self, entry: LogEntry) -> Result<()> {
        // Check if we need to rotate the segment first
        self.check_rotation()?;
        
        // Append the entry to the active segment
        let mut active = self.active_segment.lock().map_err(|_| {
            Error::Other("Failed to acquire lock on active segment".to_string())
        })?;
        
        active.append(entry).map_err(convert_boxed_error)?;
        
        // Auto-flush if configured
        if self.options.auto_flush {
            active.flush().map_err(convert_boxed_error)?;
        }
        
        Ok(())
    }
    
    /// Flush all active segments to disk
    pub fn flush(&self) -> Result<()> {
        // Flush the active segment
        let mut active = self.active_segment.lock().map_err(|_| {
            Error::Other("Failed to acquire lock on active segment".to_string())
        })?;
        
        active.flush().map_err(convert_boxed_error)?;
        
        // Flush all cached segments
        let cached = self.cached_segments.read().map_err(|_| {
            Error::Other("Failed to acquire read lock on cached segments".to_string())
        })?;
        
        for (_, segment) in cached.iter() {
            let mut segment = segment.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on cached segment".to_string())
            })?;
            
            segment.flush().map_err(convert_boxed_error)?;
        }
        
        Ok(())
    }
    
    /// Check if the active segment needs rotation
    fn check_rotation(&self) -> Result<()> {
        let should_rotate = {
            let active = self.active_segment.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on active segment".to_string())
            })?;
            
            // Check rotation criteria
            for criteria in &self.options.rotation_criteria {
                match criteria {
                    RotationCriteria::EntryCount(max_entries) => {
                        if active.entry_count() >= *max_entries {
                            self.rotate_segment()?;
                            return Ok(());
                        }
                    },
                    RotationCriteria::Size(_) => {
                        if active.is_full(&self.storage_config) {
                            self.rotate_segment()?;
                            return Ok(());
                        }
                    },
                    RotationCriteria::TimeInterval(duration) => {
                        let last_rotation = self.last_rotation.lock().map_err(|_| {
                            Error::Other("Failed to acquire lock on last rotation".to_string())
                        })?;
                        
                        let now = Utc::now();
                        if now.signed_duration_since(*last_rotation) >= *duration {
                            self.rotate_segment()?;
                            return Ok(());
                        }
                    },
                    RotationCriteria::Custom(func) => {
                        if func(&active) {
                            self.rotate_segment()?;
                            return Ok(());
                        }
                    },
                }
            }
            
            Ok(())
        };
        
        should_rotate
    }
    
    /// Rotate the active segment
    fn rotate_segment(&self) -> Result<()> {
        // Create a new segment
        let segment_id = generate_segment_id();
        let segment_path = self.options.base_dir.join(format!("{}.log", segment_id));
        let mut new_segment = LogSegment::new(segment_id);
        new_segment.set_path(&segment_path);
        
        // Swap the active segment
        let old_segment = {
            let mut active_lock = self.active_segment.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on active segment".to_string())
            })?;
            
            // Mark the old segment as read-only
            active_lock.mark_readonly();
            
            // Flush the old segment
            active_lock.flush().map_err(convert_boxed_error)?;
            
            // Activate a new segment
            let _old_info = active_lock.info().clone();
            
            // Extract the segment before updating
            let old_segment = std::mem::replace(&mut *active_lock, new_segment);
            
            // Update the last rotation timestamp
            let mut last_rotation = self.last_rotation.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on last rotation".to_string())
            })?;
            *last_rotation = Utc::now();
            
            // Return the old segment
            old_segment
        };
        
        // Add the old segment to the cache
        self.add_to_cache(old_segment)?;
        
        // Manage cache size
        self.manage_cache()?;
        
        Ok(())
    }
    
    /// Add a segment to the cache
    fn add_to_cache(&self, segment: LogSegment) -> Result<()> {
        let segment_id = segment.info().id.clone();
        let segment_arc = Arc::new(Mutex::new(segment));
        
        let mut cached = self.cached_segments.write().map_err(|_| {
            Error::Other("Failed to acquire write lock on cached segments".to_string())
        })?;
        
        cached.insert(segment_id, segment_arc);
        
        Ok(())
    }
    
    /// Manage the cache size
    fn manage_cache(&self) -> Result<()> {
        let mut cached = self.cached_segments.write().map_err(|_| {
            Error::Other("Failed to acquire write lock on cached segments".to_string())
        })?;
        
        // If the cache is not over limit, do nothing
        if cached.len() <= self.options.max_active_segments {
            return Ok(());
        }
        
        // Get segments sorted by last access time
        let mut segments: Vec<(String, Arc<Mutex<LogSegment>>)> = cached.drain().collect();
        
        // Sort by access time (we would need to track this in LogSegment)
        // For now just remove the oldest segments by ID
        segments.sort_by(|(id_a, _), (id_b, _)| id_a.cmp(id_b));
        
        // Keep the most recently used segments
        let to_keep = segments.split_off(segments.len() - self.options.max_active_segments);
        
        // Re-insert the segments to keep
        for (id, segment) in to_keep {
            cached.insert(id, segment);
        }
        
        // The removed segments are automatically dropped when segments goes out of scope
        
        Ok(())
    }
    
    /// Get a segment by ID
    pub fn get_segment(&self, segment_id: &str) -> Result<Option<Arc<Mutex<LogSegment>>>> {
        // Check if it's the active segment
        let active = self.active_segment.lock().map_err(|_| {
            Error::Other("Failed to acquire lock on active segment".to_string())
        })?;
        
        if active.info().id == segment_id {
            return Ok(Some(self.active_segment.clone()));
        }
        
        // Check the cache
        let cached = self.cached_segments.read().map_err(|_| {
            Error::Other("Failed to acquire read lock on cached segments".to_string())
        })?;
        
        if let Some(segment) = cached.get(segment_id) {
            return Ok(Some(segment.clone()));
        }
        
        // If not found in cache, load from disk
        self.load_segment(segment_id)
    }
    
    /// Load a segment from disk
    fn load_segment(&self, segment_id: &str) -> Result<Option<Arc<Mutex<LogSegment>>>> {
        // Look up the segment in the index
        let segment_path = self.options.base_dir.join(format!("{}.log", segment_id));
        
        if !segment_path.exists() {
            return Ok(None);
        }
        
        // TODO: Implement actual loading from disk
        // For now, just create an empty segment with the ID
        let mut segment = LogSegment::new(segment_id.to_string());
        segment.set_path(&segment_path);
        segment.mark_readonly();
        
        let segment_arc = Arc::new(Mutex::new(segment));
        
        // Add to cache
        let mut cached = self.cached_segments.write().map_err(|_| {
            Error::Other("Failed to acquire write lock on cached segments".to_string())
        })?;
        
        cached.insert(segment_id.to_string(), segment_arc.clone());
        
        // Manage cache size
        self.manage_cache()?;
        
        Ok(Some(segment_arc))
    }
    
    /// Get the active segment
    pub fn active_segment(&self) -> Arc<Mutex<LogSegment>> {
        self.active_segment.clone()
    }
    
    /// List all segments
    pub fn list_segments(&self) -> Result<Vec<SegmentInfo>> {
        let mut segments = Vec::new();
        
        // Add the active segment
        let active = self.active_segment.lock().map_err(|_| {
            Error::Other("Failed to acquire lock on active segment".to_string())
        })?;
        
        segments.push(active.info().clone());
        
        // Add cached segments
        let cached = self.cached_segments.read().map_err(|_| {
            Error::Other("Failed to acquire read lock on cached segments".to_string())
        })?;
        
        for (_, segment) in cached.iter() {
            let segment = segment.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on segment".to_string())
            })?;
            
            segments.push(segment.info().clone());
        }
        
        // Get segments from the index that aren't in memory
        let index = self.segment_index.read().map_err(|_| {
            Error::Other("Failed to acquire read lock on segment index".to_string())
        })?;
        
        for (_, entry) in index.iter() {
            let info = entry.info.clone();
            
            // Skip if already in the list
            if segments.iter().any(|s| s.id == info.id) {
                continue;
            }
            
            segments.push(info);
        }
        
        // Sort by creation time
        segments.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        Ok(segments)
    }
    
    /// Get entries within a time range
    pub fn get_entries_in_range(
        &self,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();
        
        // Get the relevant segments
        let segments = self.get_segments_in_range(start_time, end_time)?;
        
        for segment_arc in segments {
            let segment = segment_arc.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on segment".to_string())
            })?;
            
            // Filter entries by time range
            for entry in segment.entries() {
                if entry.timestamp >= start_time && entry.timestamp <= end_time {
                    entries.push(entry.clone());
                }
            }
        }
        
        // Sort by timestamp
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        Ok(entries)
    }
    
    /// Get segments that might contain entries in the given time range
    fn get_segments_in_range(
        &self,
        start_time: Timestamp,
        end_time: Timestamp
    ) -> Result<Vec<Arc<Mutex<LogSegment>>>> {
        let mut segments = Vec::new();
        
        // Check the active segment
        let active = self.active_segment.lock().map_err(|_| {
            Error::Other("Failed to acquire lock on active segment".to_string())
        })?;
        
        let active_start = active.entries().first().map(|e| e.timestamp).unwrap_or(causality_types::Timestamp(0));
        let active_end = active.entries().last().map(|e| e.timestamp).unwrap_or(causality_types::Timestamp(0));
        
        if active_start <= end_time && active_end >= start_time {
            segments.push(self.active_segment.clone());
        }
        
        // Check cached segments
        let cached = self.cached_segments.read().map_err(|_| {
            Error::Other("Failed to acquire read lock on cached segments".to_string())
        })?;
        
        for (_, segment_arc) in cached.iter() {
            let segment = segment_arc.lock().map_err(|_| {
                Error::Other("Failed to acquire lock on segment".to_string())
            })?;
            
            let segment_start = segment.entries().first().map(|e| e.timestamp).unwrap_or(causality_types::Timestamp(0));
            let segment_end = segment.entries().last().map(|e| e.timestamp).unwrap_or(causality_types::Timestamp(0));
            
            if segment_start <= end_time && segment_end >= start_time {
                segments.push(segment_arc.clone());
            }
        }
        
        // Check the index for segments not in memory
        let index = self.segment_index.read().map_err(|_| {
            Error::Other("Failed to acquire read lock on segment index".to_string())
        })?;
        
        for (_, entry) in index.range(start_time..=end_time) {
            // Skip if we already have this segment
            let segment_id = &entry.info.id;
            if segments.iter().any(|s| {
                let segment = s.lock().unwrap();
                segment.info().id == *segment_id
            }) {
                continue;
            }
            
            // Load the segment
            if let Some(segment) = self.load_segment(segment_id)? {
                segments.push(segment);
            }
        }
        
        Ok(segments)
    }
    
    /// Merge segments
    pub fn merge_segments(&self, segment_ids: &[String]) -> Result<String> {
        if segment_ids.len() < 2 {
            return Err(Error::Other("Need at least 2 segments to merge".to_string()));
        }
        
        // Load all segments to merge
        let mut segments = Vec::new();
        let mut entries = Vec::new();
        
        for id in segment_ids {
            if let Some(segment_arc) = self.get_segment(id)? {
                let segment = segment_arc.lock().map_err(|_| {
                    Error::Other("Failed to acquire lock on segment".to_string())
                })?;
                
                if !segment.info().read_only {
                    return Err(Error::Other(format!(
                        "Cannot merge active segment {}", id
                    )));
                }
                
                // Collect entries
                entries.extend(segment.entries().iter().cloned());
                segments.push(segment_arc.clone());
            } else {
                return Err(Error::Other(format!("Segment {} not found", id)));
            }
        }
        
        // Sort entries by timestamp
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Create the merged segment
        let new_id = generate_segment_id();
        let new_path = self.options.base_dir.join(format!("{}.log", new_id));
        
        let mut merged_segment = LogSegment::new(new_id.clone());
        merged_segment.set_path(&new_path);
        
        // Add entries to the merged segment
        for entry in entries {
            merged_segment.append(entry).map_err(convert_boxed_error)?;
        }
        
        // Mark as read-only and flush
        merged_segment.mark_readonly();
        merged_segment.flush().map_err(convert_boxed_error)?;
        
        // Add to cache
        self.add_to_cache(merged_segment)?;
        
        // Update index
        // TODO: Update index and remove old segments
        
        // Manage cache size
        self.manage_cache()?;
        
        Ok(new_id)
    }
    
    /// Close the segment manager
    pub fn close(&self) -> Result<()> {
        // Flush everything
        self.flush()?;
        
        // Clear the cache
        let mut cached = self.cached_segments.write().map_err(|_| {
            Error::Other("Failed to acquire write lock on cached segments".to_string())
        })?;
        
        cached.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::types::{EntryType, EntryData};
    use crate::log::{EventEntry, EventSeverity};
    use std::collections::HashMap;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_segment_manager() -> Result<()> {
        let temp_dir = tempdir()?;
        let options = SegmentManagerOptions {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let manager = LogSegmentManager::new(options, StorageConfig::default())?;
        
        // Should have one active segment
        let segments = manager.list_segments()?;
        assert_eq!(segments.len(), 1);
        assert!(!segments[0].read_only);
        
        Ok(())
    }
    
    #[test]
    fn test_append_entry() -> Result<()> {
        let temp_dir = tempdir()?;
        let options = SegmentManagerOptions {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let manager = LogSegmentManager::new(options, StorageConfig::default())?;
        
        // Create an entry
        let entry = LogEntry {
            id: "entry1".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: "test_event".to_string(),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"test": "value"}),
                resources: None,
                domains: None,
            }),
            trace_id: None,
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        // Append the entry
        manager.append(entry)?;
        
        // Check that the entry was added
        let active = manager.active_segment();
        let active = active.lock().map_err(|_| {
            Error::Other("Failed to acquire lock on active segment".to_string())
        })?;
        
        assert_eq!(active.entry_count(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_segment_rotation() -> Result<()> {
        let temp_dir = tempdir()?;
        let options = SegmentManagerOptions {
            base_dir: temp_dir.path().to_path_buf(),
            rotation_criteria: vec![RotationCriteria::EntryCount(2)],
            ..Default::default()
        };
        
        let manager = LogSegmentManager::new(options, StorageConfig::default())?;
        
        // Create entries
        for i in 1..=5 {
            let entry = LogEntry {
                id: format!("entry{}", i),
                timestamp: Utc::now(),
                entry_type: EntryType::Event,
                data: EntryData::Event(EventEntry {
                    event_name: "test_event".to_string(),
                    severity: EventSeverity::Info,
                    component: "test".to_string(),
                    details: serde_json::json!({"test": i}),
                    resources: None,
                    domains: None,
                }),
                trace_id: None,
                parent_id: None,
                metadata: HashMap::new(),
                entry_hash: None,
            };
            
            // Append the entry
            manager.append(entry)?;
        }
        
        // Should have rotated twice (3 segments total)
        let segments = manager.list_segments()?;
        assert_eq!(segments.len(), 3);
        
        // Only the active segment should be writable
        assert!(!segments[2].read_only); // Active segment
        assert!(segments[0].read_only);  // First segment (rotated)
        assert!(segments[1].read_only);  // Second segment (rotated)
        
        Ok(())
    }
} 