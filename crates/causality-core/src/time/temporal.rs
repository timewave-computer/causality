// Temporal time abstractions
//
// This file contains abstractions for handling logical time, temporal ordering,
// and causal relationships between events.

use causality_error::{Error, Result};
use std::cmp::Ordering;
use std::fmt;

/// A logical timestamp that represents a point in logical time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicalTimestamp {
    /// Counter value for this timestamp
    counter: u64,
    /// Node ID that created this timestamp
    node_id: u32,
}

impl LogicalTimestamp {
    /// Create a new logical timestamp
    pub fn new(counter: u64, node_id: u32) -> Self {
        Self { counter, node_id }
    }
    
    /// Increment the counter value, creating a new timestamp
    pub fn increment(&self) -> Self {
        Self {
            counter: self.counter + 1,
            node_id: self.node_id,
        }
    }
    
    /// Get the counter value
    pub fn counter(&self) -> u64 {
        self.counter
    }
    
    /// Get the node ID
    pub fn node_id(&self) -> u32 {
        self.node_id
    }
    
    /// Merge this timestamp with another, taking the maximum counter value
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            counter: std::cmp::max(self.counter, other.counter),
            node_id: self.node_id,
        }
    }
}

impl PartialOrd for LogicalTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogicalTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.counter.cmp(&other.counter) {
            Ordering::Equal => self.node_id.cmp(&other.node_id),
            ordering => ordering,
        }
    }
}

impl fmt::Display for LogicalTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}@N{}", self.counter, self.node_id)
    }
}

/// A vector clock for tracking causality between distributed events
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    /// Map of node IDs to counter values
    counters: std::collections::HashMap<u32, u64>,
}

impl VectorClock {
    /// Create a new, empty vector clock
    pub fn new() -> Self {
        Self {
            counters: std::collections::HashMap::new(),
        }
    }
    
    /// Increment the counter for a specific node
    pub fn increment(&mut self, node_id: u32) -> Result<()> {
        let counter = self.counters.entry(node_id).or_insert(0);
        *counter += 1;
        Ok(())
    }
    
    /// Get the counter value for a specific node
    pub fn get(&self, node_id: u32) -> u64 {
        *self.counters.get(&node_id).unwrap_or(&0)
    }
    
    /// Merge this vector clock with another
    pub fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        
        for (node_id, counter) in &other.counters {
            let entry = result.counters.entry(*node_id).or_insert(0);
            *entry = std::cmp::max(*entry, *counter);
        }
        
        result
    }
    
    /// Check if this vector clock happens before another
    pub fn happens_before(&self, other: &Self) -> bool {
        let mut found_less = false;
        
        for (node_id, counter) in &self.counters {
            let other_counter = other.get(*node_id);
            
            if *counter > other_counter {
                return false;
            }
            
            if *counter < other_counter {
                found_less = true;
            }
        }
        
        // Check if other has nodes we don't have
        for (node_id, counter) in &other.counters {
            if *counter > 0 && !self.counters.contains_key(node_id) {
                found_less = true;
            }
        }
        
        found_less
    }
    
    /// Check if this vector clock is concurrent with another
    pub fn concurrent_with(&self, other: &Self) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

/// Trait for types that provide time services
pub trait TimeProvider {
    /// Get the current logical timestamp
    fn current_time(&self) -> Result<LogicalTimestamp>;
    
    /// Get the vector clock
    fn vector_clock(&self) -> Result<VectorClock>;
    
    /// Update the vector clock based on an observed event
    fn observe_event(&mut self, timestamp: LogicalTimestamp) -> Result<()>;
} 