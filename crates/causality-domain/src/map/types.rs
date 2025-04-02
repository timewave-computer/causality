// Domain map type definitions
// Original file: src/domain/map/types.rs

// Time-related types
//
// This module defines the core types used in time synchronization
// and management across domains.

use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::cmp::{PartialEq, PartialOrd, Ordering};

use causality_types::{BlockHash, BlockHeight, Timestamp};
use crate::domain::DomainId;

/// A time point represents a specific observed moment in a domain's timeline
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimePoint {
    /// Timestamp value (e.g., Unix timestamp)
    pub timestamp: u64,
    /// Block height (if applicable)
    pub block_height: Option<u64>,
    /// Block identifier (if applicable)
    pub block_id: Option<String>,
    /// Sequence number (if applicable)
    pub sequence: Option<u64>,
}

impl TimePoint {
    /// Create a new time point with just a timestamp
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            block_height: None,
            block_id: None,
            sequence: None,
        }
    }
    
    /// Create a new time point with a timestamp and block height
    pub fn with_block(timestamp: u64, block_height: u64) -> Self {
        Self {
            timestamp,
            block_height: Some(block_height),
            block_id: None,
            sequence: None,
        }
    }
    
    /// Create a new time point with timestamp, block height, and block ID
    pub fn with_block_id(timestamp: u64, block_height: u64, block_id: impl Into<String>) -> Self {
        Self {
            timestamp,
            block_height: Some(block_height),
            block_id: Some(block_id.into()),
            sequence: None,
        }
    }
    
    /// Create a new time point with a sequence number
    pub fn with_sequence(timestamp: u64, sequence: u64) -> Self {
        Self {
            timestamp,
            block_height: None,
            block_id: None,
            sequence: Some(sequence),
        }
    }
}

impl PartialOrd for TimePoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // First compare by timestamp
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => {
                // If timestamps are equal, try comparing by block height
                if let (Some(self_height), Some(other_height)) = (self.block_height, other.block_height) {
                    match self_height.cmp(&other_height) {
                        Ordering::Equal => {
                            // If block heights are equal, try comparing by sequence
                            if let (Some(self_seq), Some(other_seq)) = (self.sequence, other.sequence) {
                                Some(self_seq.cmp(&other_seq))
                            } else {
                                Some(Ordering::Equal)
                            }
                        }
                        ordering => Some(ordering),
                    }
                } else if let (Some(self_seq), Some(other_seq)) = (self.sequence, other.sequence) {
                    // Compare by sequence if block heights aren't available
                    Some(self_seq.cmp(&other_seq))
                } else {
                    Some(Ordering::Equal)
                }
            }
            ordering => Some(ordering),
        }
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "TimePoint(ts={}, bh={:?}, bid={:?}, seq={:?})",
            self.timestamp, self.block_height, self.block_id, self.sequence
        )
    }
}

/// A time range between two time points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time point (inclusive)
    pub start: TimePoint,
    /// End time point (inclusive)
    pub end: TimePoint,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: TimePoint, end: TimePoint) -> Self {
        Self { start, end }
    }
    
    /// Create a time range from timestamps
    pub fn from_timestamps(start: u64, end: u64) -> Self {
        Self {
            start: TimePoint::new(start),
            end: TimePoint::new(end),
        }
    }
    
    /// Check if a time point is contained in this range
    pub fn contains(&self, point: &TimePoint) -> bool {
        (point >= &self.start) && (point <= &self.end)
    }
    
    /// Get the duration of this range in seconds
    pub fn duration(&self) -> u64 {
        if self.end.timestamp >= self.start.timestamp {
            self.end.timestamp - self.start.timestamp
        } else {
            0
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TimeRange(start={}, end={})", self.start, self.end)
    }
}

/// A time window represents a snapshot of a domain at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Range of time this window covers
    pub range: TimeRange,
    /// Block height at the start of the window
    pub start_height: BlockHeight,
    /// Block height at the end of the window
    pub end_height: Option<BlockHeight>,
    /// Block hash at the start of the window
    pub start_hash: BlockHash,
    /// Block hash at the end of the window
    pub end_hash: Option<BlockHash>,
    /// When this window was created
    pub created_at: DateTime<Utc>,
    /// Additional metadata about this window
    pub metadata: HashMap<String, String>,
}

impl TimeWindow {
    /// Create a new time window
    pub fn new(
        domain_id: DomainId,
        range: TimeRange,
        start_height: BlockHeight,
        start_hash: BlockHash,
    ) -> Self {
        TimeWindow {
            domain_id,
            range,
            start_height,
            end_height: None,
            start_hash,
            end_hash: None,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the end height and hash
    pub fn with_end(mut self, end_height: BlockHeight, end_hash: BlockHash) -> Self {
        self.end_height = Some(end_height);
        self.end_hash = Some(end_hash);
        self
    }
    
    /// Add metadata to this window
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if a timestamp is within this window
    pub fn contains(&self, timestamp: Timestamp) -> bool {
        self.range.contains(&TimePoint {
            timestamp: timestamp.timestamp,
            block_height: Some(timestamp.block_height),
            block_id: Some(timestamp.block_id.clone()),
            sequence: Some(timestamp.sequence),
        })
    }
    
    /// Check if this window is complete (has both start and end)
    pub fn is_complete(&self) -> bool {
        self.end_height.is_some() && self.end_hash.is_some()
    }
    
    /// Get the duration of this window in seconds
    pub fn duration(&self) -> u64 {
        self.range.duration()
    }
    
    /// Check if this window overlaps with another
    pub fn overlaps(&self, other: &TimeWindow) -> bool {
        self.range.contains(&other.range.start) || self.range.contains(&other.range.end)
    }
    
    /// Get the intersection of this window with another
    pub fn intersection(&self, other: &TimeWindow) -> Option<TimeRange> {
        let mut intersection_points = Vec::new();
        
        if self.range.contains(&other.range.start) {
            intersection_points.push(other.range.start);
        }
        if self.range.contains(&other.range.end) {
            intersection_points.push(other.range.end);
        }
        
        if intersection_points.is_empty() {
            None
        } else {
            let start = intersection_points.iter().min().unwrap().clone();
            let end = intersection_points.iter().max().unwrap().clone();
            Some(TimeRange::new(start, end))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_point() {
        let point = TimePoint::new(1000);
        assert_eq!(point.timestamp, 1000);
        assert_eq!(point.block_height, None);
        assert_eq!(point.block_id, None);
        assert_eq!(point.sequence, None);
        
        let point = TimePoint::with_block(1000, 100);
        assert_eq!(point.timestamp, 1000);
        assert_eq!(point.block_height, Some(100));
        assert_eq!(point.block_id, None);
        assert_eq!(point.sequence, None);
        
        let point = TimePoint::with_block_id(1000, 100, "block100");
        assert_eq!(point.timestamp, 1000);
        assert_eq!(point.block_height, Some(100));
        assert_eq!(point.block_id, Some("block100"));
        assert_eq!(point.sequence, None);
        
        let point = TimePoint::with_sequence(1000, 1);
        assert_eq!(point.timestamp, 1000);
        assert_eq!(point.block_height, None);
        assert_eq!(point.block_id, None);
        assert_eq!(point.sequence, Some(1));
    }
    
    #[test]
    fn test_time_range() {
        let range = TimeRange::from_timestamps(1000, 2000);
        assert!(range.contains(&TimePoint::new(1000)));
        assert!(range.contains(&TimePoint::new(1500)));
        assert!(range.contains(&TimePoint::new(2000)));
        assert!(!range.contains(&TimePoint::new(999)));
        assert!(!range.contains(&TimePoint::new(2001)));
        
        assert_eq!(range.duration(), 1000);
    }
    
    #[test]
    fn test_time_window() {
        let domain_id: DomainId = "test_domain".into();
        let range = TimeRange::from_timestamps(1000, 2000);
        let window = TimeWindow::new(domain_id.clone(), range, 100, "block100".into());
        
        assert_eq!(window.domain_id, domain_id);
        assert_eq!(window.start_height, 100);
        assert!(window.end_height.is_none());
        assert!(!window.is_complete());
        
        let window = window.with_end(200, "block200".into());
        assert!(window.is_complete());
        assert_eq!(window.end_height, Some(200));
        
        assert!(window.contains(1500));
        assert!(!window.contains(2500));
        
        assert_eq!(window.duration(), 1000);
    }
    
    #[test]
    fn test_time_window_overlaps() {
        let domain_id: DomainId = "test_domain".into();
        let range1 = TimeRange::from_timestamps(1000, 2000);
        let range2 = TimeRange::from_timestamps(1500, 2500);
        
        let window1 = TimeWindow::new(domain_id.clone(), range1, 100, "block100".into());
        let window2 = TimeWindow::new(domain_id.clone(), range2, 150, "block150".into());
        
        assert!(window1.overlaps(&window2));
        
        let intersection = window1.intersection(&window2).unwrap();
        assert!(intersection.contains(&TimePoint::new(1500)));
        assert!(intersection.contains(&TimePoint::new(2000)));
    }
} 