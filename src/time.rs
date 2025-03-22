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
} 