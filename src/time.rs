// Time module for Causality
//
// This module provides functionality for time tracking, synchronization,
// and causal ordering across distributed components using Lamport logical clocks.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};

/// A Lamport logical clock implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportTime(u64);

impl LamportTime {
    /// Create a new Lamport clock with the specified starting value
    pub fn new(initial: u64) -> Self {
        Self(initial)
    }
    
    /// Create a new Lamport clock starting from 0
    pub fn zero() -> Self {
        Self(0)
    }
    
    /// Get the current clock value
    pub fn value(&self) -> u64 {
        self.0
    }
    
    /// Increment the clock and return the new value
    pub fn increment(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
    
    /// Update the clock based on a received timestamp
    /// This updates the clock to max(local + 1, received + 1)
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
        Self::zero()
    }
}

impl fmt::Display for LamportTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}", self.0)
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

/// Thread-safe shared Lamport clock
pub struct SharedLamportClock {
    time: AtomicU64,
}

impl SharedLamportClock {
    /// Create a new shared clock with the specified initial value
    pub fn new(initial: u64) -> Self {
        Self {
            time: AtomicU64::new(initial),
        }
    }
    
    /// Get the current clock value
    pub fn get(&self) -> LamportTime {
        LamportTime(self.time.load(Ordering::SeqCst))
    }
    
    /// Increment the clock and return the new value
    pub fn increment(&self) -> LamportTime {
        let new_value = self.time.fetch_add(1, Ordering::SeqCst) + 1;
        LamportTime(new_value)
    }
    
    /// Update the clock based on a received timestamp
    pub fn update(&self, received: LamportTime) {
        let mut current = self.time.load(Ordering::SeqCst);
        loop {
            let new_value = std::cmp::max(current, received.value()) + 1;
            match self.time.compare_exchange(current, new_value, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

impl Default for SharedLamportClock {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lamport_time_basic() {
        let mut time = LamportTime::zero();
        assert_eq!(time.value(), 0);
        
        time.increment();
        assert_eq!(time.value(), 1);
        
        time.update(LamportTime::new(5));
        assert_eq!(time.value(), 6);
        
        time.update(LamportTime::new(3));
        assert_eq!(time.value(), 7);
    }
    
    #[test]
    fn test_shared_lamport_clock() {
        let clock = SharedLamportClock::default();
        assert_eq!(clock.get().value(), 0);
        
        let t1 = clock.increment();
        assert_eq!(t1.value(), 1);
        
        clock.update(LamportTime::new(5));
        assert_eq!(clock.get().value(), 6);
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