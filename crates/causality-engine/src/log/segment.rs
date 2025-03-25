// Log segmentation utilities
// Original file: src/log/segment.rs

// Log segment management for Causality Unified Log System
//
// This module provides functionality for managing log segments,
// which are chunks of log entries organized by time or other criteria.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use causality_types::{Error, Result};
use crate::log::LogEntry;

/// Information about a log segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// The unique ID of the segment
    pub id: String,
    /// The creation timestamp
    pub created_at: DateTime<Utc>,
    /// The start timestamp for entries in this segment
    pub start_time: DateTime<Utc>,
    /// The end timestamp for entries in this segment
    pub end_time: Option<DateTime<Utc>>,
    /// The number of entries in this segment
    pub entry_count: usize,
    /// The size of the segment in bytes
    pub size_bytes: u64,
    /// Whether the segment is read-only
    pub read_only: bool,
    /// The path to the segment file, if stored on disk
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl SegmentInfo {
    /// Create a new segment info
    pub fn new(
        id: String,
        start_time: DateTime<Utc>,
    ) -> Self {
        SegmentInfo {
            id,
            created_at: Utc::now(),
            start_time,
            end_time: None,
            entry_count: 0,
            size_bytes: 0,
            read_only: false,
            path: None,
        }
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
    pub fn set_end_time(&mut self, end_time: DateTime<Utc>) {
        self.end_time = Some(end_time);
    }
}

/// A segment of log entries
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
        let now = Utc::now();
        Self {
            info: SegmentInfo::new(id, now),
            entries: Vec::new(),
            modified: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new log segment from existing entries
    pub fn from_entries(id: String, entries: Vec<LogEntry>) -> Result<Self> {
        if entries.is_empty() {
            return Err(Error::InvalidArgument("Cannot create segment with no entries".to_string()));
        }
        
        let start_time = entries.first().unwrap().timestamp().clone();
        let end_time = entries.last().unwrap().timestamp().clone();
        
        let mut info = SegmentInfo::new(id, start_time);
        info.set_end_time(end_time);
        info.update_entry_count(entries.len());
        
        Ok(LogSegment {
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
    pub fn add_entry(&mut self, entry: LogEntry) -> Result<()> {
        // Check if the segment is read-only
        if self.info.read_only {
            return Err(Error::OperationFailed("Segment is read-only".to_string()));
        }
        
        // Update the end time if needed
        let entry_time = entry.timestamp().clone();
        if let Some(end_time) = self.info.end_time {
            if entry_time > end_time {
                self.info.set_end_time(entry_time);
            }
        } else {
            self.info.set_end_time(entry_time);
        }
        
        // Add the entry
        self.entries.push(entry);
        self.info.update_entry_count(self.entries.len());
        self.modified = true;
        
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
    
    /// Check if the segment is full according to the storage config
    pub fn is_full(&self, config: &causality_engine::StorageConfig) -> bool {
        // Check if the segment has too many entries
        if config.max_segment_entries > 0 && self.entries.len() >= config.max_segment_entries {
            return true;
        }
        
        // Check if the segment is too large in bytes
        if config.max_segment_size > 0 {
            // Estimate size - this is approximate but good enough for limit checks
            let estimated_size = self.entries.len() * 256; // Assume average 256 bytes per entry
            if estimated_size >= config.max_segment_size as usize {
                return true;
            }
        }
        
        false
    }
    
    /// Append an entry to this segment
    pub fn append(&mut self, entry: LogEntry) -> Result<()> {
        self.add_entry(entry)
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