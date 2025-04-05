// Log segmentation utilities
// Original file: src/log/segment.rs

// Log segment management for Causality Unified Log System
//
// This module provides functionality for managing log segments,
// which are chunks of log entries organized by time or other criteria.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use causality_error::{Result, EngineError};
use causality_types::Timestamp;
use crate::log::storage::StorageConfig;
use crate::log::types::LogEntry;

/// Status of a segment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentStatus {
    /// Active and writable
    Active,
    
    /// Closed and read-only
    Closed,
    
    /// Archived for long-term storage
    Archived,
    
    /// Being compacted
    Compacting,
}

/// Information about a log segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// The unique ID of the segment
    pub id: String,
    /// The creation timestamp
    pub created_at: Timestamp,
    /// The start timestamp for entries in this segment
    pub start_time: Timestamp,
    /// The end timestamp for entries in this segment
    pub end_time: Option<Timestamp>,
    /// The number of entries in this segment
    pub entry_count: usize,
    /// The size of the segment in bytes
    pub size_bytes: u64,
    /// Whether the segment is read-only
    pub read_only: bool,
    /// The path to the segment file, if stored on disk
    #[serde(skip)]
    pub path: Option<PathBuf>,
    /// Additional metadata for the segment
    pub metadata: HashMap<String, String>,
    /// The status of the segment
    pub status: SegmentStatus,
}

impl SegmentInfo {
    /// Create a new segment info
    pub fn new(
        id: String,
        start_time: Timestamp,
    ) -> Self {
        SegmentInfo {
            id,
            created_at: Timestamp::now(),
            start_time,
            end_time: None,
            entry_count: 0,
            size_bytes: 0,
            read_only: false,
            path: None,
            metadata: HashMap::new(),
            status: SegmentStatus::Active,
        }
    }
    
    /// Create a new segment info from DateTime
    pub fn from_datetime(
        id: String,
        start_time: DateTime<Utc>,
    ) -> Self {
        Self::new(id, Timestamp::from_datetime(&start_time))
    }
    
    /// Set the path for this segment
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Mark this segment as read-only
    pub fn mark_readonly(&mut self) {
        self.read_only = true;
    }
    
    /// Update the entry count
    pub fn update_entry_count(&mut self, count: usize) {
        self.entry_count = count;
    }
    
    /// Update the size in bytes
    pub fn update_size(&mut self, size_bytes: u64) {
        self.size_bytes = size_bytes;
    }
    
    /// Set the end time
    pub fn set_end_time(&mut self, end_time: Timestamp) {
        self.end_time = Some(end_time);
    }
    
    /// Mark this segment as closed
    pub fn mark_closed(&mut self, end_time: Timestamp, entry_count: usize, size_bytes: usize) {
        self.end_time = Some(end_time);
        self.entry_count = entry_count;
        self.size_bytes = size_bytes as u64;
        self.status = SegmentStatus::Closed;
    }
}

/// A segment of log entries
#[derive(Serialize, Deserialize)]
pub struct LogSegment {
    /// The segment info
    info: SegmentInfo,
    /// The entries in this segment
    entries: Vec<LogEntry>,
    /// Whether this segment has been modified since loading
    modified: bool,
    /// Additional metadata for the segment
    metadata: HashMap<String, String>,
}

impl LogSegment {
    /// Create a new log segment with the given ID
    pub fn new(id: String) -> Self {
        let now = Timestamp::now();
        Self {
            info: SegmentInfo::new(id, now),
            entries: Vec::new(),
            modified: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a segment from existing entries
    pub fn from_entries(id: String, entries: Vec<LogEntry>) -> Result<Self> {
        if entries.is_empty() {
            return Err(Box::new(EngineError::InvalidArgument("Cannot create segment with no entries".to_string())));
        }
        
        let first_entry_time = entries.first().unwrap().timestamp;
        let last_entry_time = entries.last().unwrap().timestamp;
        
        let info = SegmentInfo {
            id,
            created_at: Timestamp::now(),
            start_time: first_entry_time,
            end_time: Some(last_entry_time),
            entry_count: entries.len(),
            size_bytes: 0, // Size will be calculated during serialization
            read_only: true,
            path: None,
            metadata: HashMap::new(),
            status: SegmentStatus::Closed,
        };
        
        Ok(Self {
            info,
            entries,
            modified: false,
            metadata: HashMap::new(),
        })
    }
    
    /// Get the segment info
    pub fn info(&self) -> &SegmentInfo {
        &self.info
    }
    
    /// Get a mutable reference to the segment info
    pub fn info_mut(&mut self) -> &mut SegmentInfo {
        &mut self.info
    }
    
    /// Get the entries in this segment
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }
    
    /// Get a mutable reference to the entries
    pub fn entries_mut(&mut self) -> &mut Vec<LogEntry> {
        self.modified = true;
        &mut self.entries
    }
    
    /// Add an entry to this segment
    pub fn add_entry(&mut self, entry: LogEntry) -> causality_error::Result<()> {
        // Check if the segment is read-only
        if self.info.read_only {
            return Err(EngineError::ExecutionFailed("Segment is read-only".to_string()).into());
        }
        
        // Update segment info
        self.modified = true;
        self.info.update_entry_count(self.entries.len() + 1);
        
        let entry_time = entry.timestamp.clone();
        if let Some(end_time) = self.info.end_time {
            if entry_time > end_time {
                self.info.set_end_time(entry_time);
            }
        } else {
            self.info.set_end_time(entry_time);
        }
        
        // Add the entry
        self.entries.push(entry);
        
        Ok(())
    }
    
    /// Check if this segment has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    
    /// Mark this segment as read-only
    pub fn mark_readonly(&mut self) {
        self.info.mark_readonly();
    }
    
    /// Get the number of entries in this segment
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if this segment is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Set the path for this segment
    pub fn set_path<P: AsRef<Path>>(&mut self, path: P) {
        self.info.path = Some(path.as_ref().to_path_buf());
    }
    
    /// Clear the modified flag
    pub fn clear_modified(&mut self) {
        self.modified = false;
    }
    
    /// Check if this segment is full according to configured limits
    pub fn is_full(&self, config: &StorageConfig) -> bool {
        // Check if the entry count exceeds the maximum
        if config.max_entries_per_segment > 0 {
            if self.entries.len() >= config.max_entries_per_segment {
                return true;
            }
        }
        
        // Check if the time span exceeds the maximum
        if config.max_segment_size > 0 {
            if let Some(end_time) = self.info.end_time {
                // Calculate time difference in hours
                let hours_diff = end_time.difference(&self.info.start_time) / 3600; // Convert seconds to hours
                
                if hours_diff >= (config.max_segment_size / (1024 * 1024)) as u64 {  // Convert to MB and then to hours
                    return true;
                }
            }
        }
        
        // Check if the size exceeds the maximum
        if self.info.size_bytes >= config.max_segment_size as u64 {
            return true;
        }
        
        false
    }
    
    /// Append an entry to this segment
    pub fn append(&mut self, entry: LogEntry) -> causality_error::Result<()> {
        if self.info.read_only {
            return Err(EngineError::ExecutionFailed("Segment is read-only".to_string()).into());
        }
        
        // Update segment info
        let info = &mut self.info;
        info.entry_count += 1;
        
        // Update end time if the new entry has a newer timestamp
        if let Some(end_time) = info.end_time {
            if entry.timestamp > end_time {
                info.end_time = Some(entry.timestamp);
            }
        } else {
            info.end_time = Some(entry.timestamp);
        }
        
        // Add the entry
        self.entries.push(entry);
        self.modified = true;
        
        Ok(())
    }
    
    /// Flush the segment to ensure changes are persisted
    pub fn flush(&mut self) -> Result<()> {
        // No action needed for in-memory segments
        // For file-backed segments, this would be overridden
        self.modified = false;
        Ok(())
    }
}

/// Generate a new segment ID
pub fn generate_segment_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    format!("segment_{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_engine::{EntryType, EntryData, EventEntry, EventSeverity};
    
    #[test]
    fn test_create_segment() {
        let now = Utc::now();
        let segment = LogSegment::new("test_segment".to_string());
        
        assert_eq!(segment.info().id, "test_segment");
        assert_eq!(segment.info().start_time, now);
        assert!(segment.info().end_time.is_none());
        assert_eq!(segment.info().entry_count, 0);
        assert!(!segment.info().read_only);
        assert!(segment.info().path.is_none());
        assert!(segment.is_empty());
        assert!(!segment.is_modified());
    }
    
    #[test]
    fn test_add_entry() -> Result<()> {
        let now = Utc::now();
        let mut segment = LogSegment::new("test_segment".to_string());
        
        // Create an event entry
        let entry = LogEntry {
            id: "entry1".to_string(),
            timestamp: now,
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
            metadata: std::collections::HashMap::new(),
            entry_hash: None,
        };
        
        segment.add_entry(entry)?;
        
        assert_eq!(segment.entry_count(), 1);
        assert!(segment.is_modified());
        assert_eq!(segment.info().entry_count, 1);
        assert_eq!(segment.info().end_time, Some(now));
        
        Ok(())
    }
    
    #[test]
    fn test_from_entries() -> Result<()> {
        let now = Utc::now();
        
        // Create several entries
        let entries = vec![
            LogEntry {
                id: "entry1".to_string(),
                timestamp: now,
                entry_type: EntryType::Event,
                data: EntryData::Event(EventEntry {
                    event_name: "test_event1".to_string(),
                    severity: EventSeverity::Info,
                    component: "test".to_string(),
                    details: serde_json::json!({"test": "value1"}),
                    resources: None,
                    domains: None,
                }),
                trace_id: None,
                parent_id: None,
                metadata: std::collections::HashMap::new(),
                entry_hash: None,
            },
            LogEntry {
                id: "entry2".to_string(),
                timestamp: now,
                entry_type: EntryType::Event,
                data: EntryData::Event(EventEntry {
                    event_name: "test_event2".to_string(),
                    severity: EventSeverity::Info,
                    component: "test".to_string(),
                    details: serde_json::json!({"test": "value2"}),
                    resources: None,
                    domains: None,
                }),
                trace_id: None,
                parent_id: None,
                metadata: std::collections::HashMap::new(),
                entry_hash: None,
            },
        ];
        
        let segment = LogSegment::from_entries("test_segment".to_string(), entries)?;
        
        assert_eq!(segment.info().id, "test_segment");
        assert_eq!(segment.info().start_time, now);
        assert_eq!(segment.info().end_time, Some(now));
        assert_eq!(segment.info().entry_count, 2);
        assert!(!segment.info().read_only);
        assert!(segment.info().path.is_none());
        assert!(!segment.is_empty());
        assert!(!segment.is_modified());
        
        Ok(())
    }
    
    #[test]
    fn test_readonly_segment() -> Result<()> {
        let now = Utc::now();
        let mut segment = LogSegment::new("test_segment".to_string());
        
        // Create an event entry
        let entry = LogEntry {
            id: "entry1".to_string(),
            timestamp: now,
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
            metadata: std::collections::HashMap::new(),
            entry_hash: None,
        };
        
        segment.add_entry(entry)?;
        
        // Mark as read-only
        segment.mark_readonly();
        assert!(segment.info().read_only);
        
        // Try to add another entry
        let entry2 = LogEntry {
            id: "entry2".to_string(),
            timestamp: now,
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: "test_event2".to_string(),
                severity: EventSeverity::Info,
                component: "test".to_string(),
                details: serde_json::json!({"test": "value2"}),
                resources: None,
                domains: None,
            }),
            trace_id: None,
            parent_id: None,
            metadata: std::collections::HashMap::new(),
            entry_hash: None,
        };
        
        let result = segment.add_entry(entry2);
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_generate_segment_id() {
        let id1 = generate_segment_id();
        let id2 = generate_segment_id();
        
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
    }
} 