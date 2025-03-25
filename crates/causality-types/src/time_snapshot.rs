// Time-based snapshot data structures
// Original file: src/time/time_map_snapshot.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// A snapshot of a time map at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeMapSnapshot {
    /// Timestamp of this snapshot
    pub timestamp: u64,
    
    /// Domain timestamps at this point in time
    pub domain_timestamps: HashMap<String, u64>,
    
    /// Causality edges known at this point in time
    pub causality_edges: Vec<(String, String)>,
    
    /// Hash of the time map at this point
    pub hash: String,
}

impl TimeMapSnapshot {
    /// Create a new time map snapshot
    pub fn new(
        timestamp: u64,
        domain_timestamps: HashMap<String, u64>,
        causality_edges: Vec<(String, String)>,
    ) -> Self {
        // In a real implementation, we'd compute a proper hash here
        let hash = format!("snapshot_{}", timestamp);
        
        Self {
            timestamp,
            domain_timestamps,
            causality_edges,
            hash,
        }
    }
    
    /// Create an empty snapshot with just a timestamp
    pub fn with_timestamp(timestamp: u64) -> Self {
        Self {
            timestamp,
            domain_timestamps: HashMap::new(),
            causality_edges: Vec::new(),
            hash: format!("snapshot_{}", timestamp),
        }
    }
    
    /// Get the timestamp of this snapshot
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// Get the hash of this snapshot
    pub fn hash(&self) -> &str {
        &self.hash
    }
} 