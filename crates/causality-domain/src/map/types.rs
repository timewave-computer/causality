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

use causality_types::{BlockHash, BlockHeight, Timestamp};
use crate::domain::DomainId;

/// A time point represents a specific observed moment in a domain's timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    /// Block height associated with this time point
    pub height: BlockHeight,
    /// Block hash associated with this time point
    pub hash: BlockHash,
    /// Timestamp in seconds
    pub timestamp: Timestamp,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Whether this time point has been verified
    pub verified: bool,
    /// Source of this time point (e.g., "rpc", "peers", "consensus")
    pub source: String,
}

impl TimePoint {
    /// Create a new time point
    pub fn new(
        height: BlockHeight,
        hash: BlockHash,
        timestamp: Timestamp,
    ) -> Self {
        TimePoint {
            height,
            hash,
            timestamp,
            confidence: 1.0,
            verified: false,
            source: "default".to_string(),
        }
    }
    
    /// Set the confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.max(0.0).min(1.0);
        self
    }
    
    /// Set the verification status
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    /// Set the source of this time point
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }
    
    /// Create a time point from a generic source
    pub fn from_source<T: ToString>(
        height: BlockHeight,
        hash: BlockHash,
        timestamp: Timestamp,
        source: T,
        confidence: f64,
    ) -> Self {
        TimePoint {
            height,
            hash,
            timestamp,
            confidence: confidence.max(0.0).min(1.0),
            verified: false,
            source: source.to_string(),
        }
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "TimePoint(height={}, ts={}, conf={:.2}, src={})",
            self.height, self.timestamp, self.confidence, self.source
        )
    }
}

/// A time range between two time points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Starting time in seconds
    pub start: Timestamp,
    /// Ending time in seconds
    pub end: Timestamp,
    /// Whether this range is inclusive at the start
    pub start_inclusive: bool,
    /// Whether this range is inclusive at the end
    pub end_inclusive: bool,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        TimeRange {
            start,
            end,
            start_inclusive: true,
            end_inclusive: true,
        }
    }
    
    /// Create an exclusive time range
    pub fn exclusive(start: Timestamp, end: Timestamp) -> Self {
        TimeRange {
            start,
            end,
            start_inclusive: false,
            end_inclusive: false,
        }
    }
    
    /// Create a half-open range [start, end)
    pub fn half_open(start: Timestamp, end: Timestamp) -> Self {
        TimeRange {
            start,
            end,
            start_inclusive: true,
            end_inclusive: false,
        }
    }
    
    /// Check if a timestamp is within this range
    pub fn contains(&self, timestamp: Timestamp) -> bool {
        let start_check = if self.start_inclusive {
            timestamp >= self.start
        } else {
            timestamp > self.start
        };
        
        let end_check = if self.end_inclusive {
            timestamp <= self.end
        } else {
            timestamp < self.end
        };
        
        start_check && end_check
    }
    
    /// Get the duration of this range in seconds
    pub fn duration(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
    
    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        // This range starts before other ends
        let this_before_other_ends = if other.end_inclusive {
            self.start <= other.end
        } else {
            self.start < other.end
        };
        
        // This range ends after other starts
        let this_ends_after_other_starts = if self.end_inclusive {
            other.start <= self.end
        } else {
            other.start < self.end
        };
        
        this_before_other_ends && this_ends_after_other_starts
    }
    
    /// Create the intersection of this range with another
    pub fn intersection(&self, other: &TimeRange) -> Option<TimeRange> {
        if !self.overlaps(other) {
            return None;
        }
        
        let (start, start_inclusive) = if self.start < other.start {
            (other.start, other.start_inclusive)
        } else if self.start > other.start {
            (self.start, self.start_inclusive)
        } else {
            // Equal starts, inclusive if both are inclusive
            (self.start, self.start_inclusive && other.start_inclusive)
        };
        
        let (end, end_inclusive) = if self.end < other.end {
            (self.end, self.end_inclusive)
        } else if self.end > other.end {
            (other.end, other.end_inclusive)
        } else {
            // Equal ends, inclusive if both are inclusive
            (self.end, self.end_inclusive && other.end_inclusive)
        };
        
        Some(TimeRange {
            start,
            end,
            start_inclusive,
            end_inclusive,
        })
    }
    
    /// Create a new range that spans both this range and another
    pub fn union(&self, other: &TimeRange) -> TimeRange {
        let (start, start_inclusive) = if self.start < other.start {
            (self.start, self.start_inclusive)
        } else if self.start > other.start {
            (other.start, other.start_inclusive)
        } else {
            // Equal starts, inclusive if either is inclusive
            (self.start, self.start_inclusive || other.start_inclusive)
        };
        
        let (end, end_inclusive) = if self.end > other.end {
            (self.end, self.end_inclusive)
        } else if self.end < other.end {
            (other.end, other.end_inclusive)
        } else {
            // Equal ends, inclusive if either is inclusive
            (self.end, self.end_inclusive || other.end_inclusive)
        };
        
        TimeRange {
            start,
            end,
            start_inclusive,
            end_inclusive,
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_bracket = if self.start_inclusive { '[' } else { '(' };
        let end_bracket = if self.end_inclusive { ']' } else { ')' };
        
        write!(f, "{}{}, {}{}", start_bracket, self.start, self.end, end_bracket)
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
        self.range.contains(timestamp)
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
        self.range.overlaps(&other.range)
    }
    
    /// Get the intersection of this window with another
    pub fn intersection(&self, other: &TimeWindow) -> Option<TimeRange> {
        self.range.intersection(&other.range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_point() {
        let point = TimePoint::new(100, "block123".into(), 1000);
        assert_eq!(point.height, 100);
        assert_eq!(point.timestamp, 1000);
        assert_eq!(point.confidence, 1.0);
        assert_eq!(point.verified, false);
        
        let point = point
            .with_confidence(0.8)
            .with_verification(true)
            .with_source("rpc");
        
        assert_eq!(point.confidence, 0.8);
        assert_eq!(point.verified, true);
        assert_eq!(point.source, "rpc");
        
        // Test confidence bounds
        let point = TimePoint::new(100, "block123".into(), 1000)
            .with_confidence(1.5);
        assert_eq!(point.confidence, 1.0);
        
        let point = TimePoint::new(100, "block123".into(), 1000)
            .with_confidence(-0.5);
        assert_eq!(point.confidence, 0.0);
    }
    
    #[test]
    fn test_time_range() {
        // Inclusive range
        let range = TimeRange::new(1000, 2000);
        assert!(range.contains(1000));
        assert!(range.contains(1500));
        assert!(range.contains(2000));
        assert!(!range.contains(999));
        assert!(!range.contains(2001));
        
        // Exclusive range
        let range = TimeRange::exclusive(1000, 2000);
        assert!(!range.contains(1000));
        assert!(range.contains(1500));
        assert!(!range.contains(2000));
        
        // Half-open range
        let range = TimeRange::half_open(1000, 2000);
        assert!(range.contains(1000));
        assert!(range.contains(1500));
        assert!(!range.contains(2000));
        
        // Duration
        assert_eq!(range.duration(), 1000);
    }
    
    #[test]
    fn test_time_range_overlaps() {
        let range1 = TimeRange::new(1000, 2000);
        let range2 = TimeRange::new(1500, 2500);
        let range3 = TimeRange::new(2500, 3000);
        
        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
        assert!(!range1.overlaps(&range3));
        assert!(!range3.overlaps(&range1));
        assert!(range2.overlaps(&range3));
        
        // Edge cases
        let range4 = TimeRange::new(2000, 3000);
        assert!(range1.overlaps(&range4)); // They touch at 2000
        
        let range5 = TimeRange::exclusive(2000, 3000);
        assert!(!range1.overlaps(&range5)); // They don't overlap because range5 excludes 2000
    }
    
    #[test]
    fn test_time_range_intersection() {
        let range1 = TimeRange::new(1000, 2000);
        let range2 = TimeRange::new(1500, 2500);
        
        let intersection = range1.intersection(&range2).unwrap();
        assert_eq!(intersection.start, 1500);
        assert_eq!(intersection.end, 2000);
        assert!(intersection.start_inclusive);
        assert!(intersection.end_inclusive);
        
        // No intersection
        let range3 = TimeRange::new(3000, 4000);
        assert!(range1.intersection(&range3).is_none());
        
        // Inclusive/exclusive combinations
        let range4 = TimeRange::new(1000, 2000);
        let range5 = TimeRange::exclusive(1000, 2000);
        
        let intersection = range4.intersection(&range5).unwrap();
        assert_eq!(intersection.start, 1000);
        assert_eq!(intersection.end, 2000);
        assert!(!intersection.start_inclusive); // False because range5 is exclusive at start
        assert!(!intersection.end_inclusive); // False because range5 is exclusive at end
    }
    
    #[test]
    fn test_time_range_union() {
        let range1 = TimeRange::new(1000, 2000);
        let range2 = TimeRange::new(1500, 2500);
        
        let union = range1.union(&range2);
        assert_eq!(union.start, 1000);
        assert_eq!(union.end, 2500);
        assert!(union.start_inclusive);
        assert!(union.end_inclusive);
        
        // Non-overlapping ranges
        let range3 = TimeRange::new(3000, 4000);
        let union = range1.union(&range3);
        assert_eq!(union.start, 1000);
        assert_eq!(union.end, 4000);
        
        // Inclusive/exclusive combinations
        let range4 = TimeRange::exclusive(1000, 2000);
        let range5 = TimeRange::exclusive(1500, 2500);
        
        let union = range4.union(&range5);
        assert_eq!(union.start, 1000);
        assert_eq!(union.end, 2500);
        assert!(!union.start_inclusive); // False because range4 is exclusive at start
        assert!(!union.end_inclusive); // False because range5 is exclusive at end
    }
    
    #[test]
    fn test_time_window() {
        let domain_id: DomainId = "test_domain".into();
        let range = TimeRange::new(1000, 2000);
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
        let range1 = TimeRange::new(1000, 2000);
        let range2 = TimeRange::new(1500, 2500);
        
        let window1 = TimeWindow::new(domain_id.clone(), range1, 100, "block100".into());
        let window2 = TimeWindow::new(domain_id.clone(), range2, 150, "block150".into());
        
        assert!(window1.overlaps(&window2));
        
        let intersection = window1.intersection(&window2).unwrap();
        assert_eq!(intersection.start, 1500);
        assert_eq!(intersection.end, 2000);
    }
} 