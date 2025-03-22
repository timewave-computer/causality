// Time module for Causality
//
// This module provides functionality for time tracking, synchronization,
// and causal ordering across distributed components using Lamport logical clocks.

use std::fmt;
use serde::{Serialize, Deserialize};

/// Lamport logical clock time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LamportTime(pub u64);

impl LamportTime {
    /// Create a new logical clock with initial value 0
    pub fn new() -> Self {
        LamportTime(0)
    }
    
    /// Get the current value
    pub fn value(&self) -> u64 {
        self.0
    }
    
    /// Increment the logical clock
    pub fn increment(&mut self) {
        self.0 += 1;
    }
    
    /// Update the clock based on a received message timestamp.
    /// Sets the clock to max(local, received) + 1
    pub fn update(&mut self, received: LamportTime) {
        self.0 = std::cmp::max(self.0, received.0) + 1;
    }
    
    /// Check if this time happened before another time
    pub fn happened_before(&self, other: &LamportTime) -> bool {
        self.0 < other.0
    }
    
    /// Check if this time happened after another time
    pub fn happened_after(&self, other: &LamportTime) -> bool {
        self.0 > other.0
    }
}

impl Default for LamportTime {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LamportTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// Snapshot of time-related information for time-indexed data structures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeMapSnapshot {
    /// Timestamp (in milliseconds since epoch)
    pub timestamp: u64,
    
    /// Block height at time of snapshot
    pub block_height: u64,
    
    /// Transaction ID associated with the snapshot
    pub transaction_id: String,
}

impl TimeMapSnapshot {
    /// Create a new time snapshot with the given parameters
    pub fn new(timestamp: u64, block_height: u64, transaction_id: String) -> Self {
        Self {
            timestamp,
            block_height,
            transaction_id,
        }
    }
    
    /// Create a default time snapshot (useful for testing)
    pub fn default() -> Self {
        Self {
            timestamp: 0,
            block_height: 0,
            transaction_id: "default".to_string(),
        }
    }
}

impl fmt::Display for TimeMapSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TimeMapSnapshot(t={}, h={}, tx={})",
            self.timestamp, self.block_height, self.transaction_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lamport_clock() {
        let mut t1 = LamportTime::new();
        assert_eq!(t1.value(), 0);
        
        t1.increment();
        assert_eq!(t1.value(), 1);
        
        let t2 = LamportTime(3);
        t1.update(t2);
        assert_eq!(t1.value(), 4);
        
        assert!(t1.happened_after(&t2));
        assert!(t2.happened_before(&t1));
    }
    
    #[test]
    fn test_time_snapshot_creation() {
        let snapshot = TimeMapSnapshot::new(1234, 5678, "test-tx".to_string());
        
        assert_eq!(snapshot.timestamp, 1234);
        assert_eq!(snapshot.block_height, 5678);
        assert_eq!(snapshot.transaction_id, "test-tx");
    }
    
    #[test]
    fn test_time_snapshot_default() {
        let snapshot = TimeMapSnapshot::default();
        
        assert_eq!(snapshot.timestamp, 0);
        assert_eq!(snapshot.block_height, 0);
        assert_eq!(snapshot.transaction_id, "default");
    }
    
    #[test]
    fn test_time_snapshot_display() {
        let snapshot = TimeMapSnapshot::new(1234, 5678, "test-tx".to_string());
        let display = format!("{}", snapshot);
        
        assert_eq!(display, "TimeMapSnapshot(t=1234, h=5678, tx=test-tx)");
    }
} 