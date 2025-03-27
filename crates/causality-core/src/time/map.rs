// Time Map Implementation
//
// This file implements the TimeMap struct for tracking relative positions across domains.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Serialize, Deserialize};

use super::types::{DomainId, DomainPosition};

/// Map for tracking relative positions across domains
///
/// The TimeMap is used to track the relative positions of different domains
/// to establish a unified view of time across the Causality system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMap {
    /// Map of domain IDs to their positions
    positions: HashMap<DomainId, DomainPosition>,
    
    /// Domains with known comparable positions
    comparables: HashMap<DomainId, HashSet<DomainId>>,
}

/// A snapshot of a time map at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMapSnapshot {
    /// The positions at the time of the snapshot
    pub positions: HashMap<DomainId, DomainPosition>,
    
    /// Timestamp of the snapshot
    pub timestamp: u64,
    
    /// Hash of the snapshot data
    pub hash: String,
}

impl TimeMap {
    /// Create a new time map
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            comparables: HashMap::new(),
        }
    }
    
    /// Update the position of a domain
    pub fn update_position(&mut self, domain_id: &str, timestamp: u64) -> Option<DomainPosition> {
        let position = if let Some(existing) = self.positions.get(domain_id) {
            // Only update if timestamp is newer
            if timestamp > existing.timestamp {
                DomainPosition::with_timestamp(timestamp)
            } else {
                return None;
            }
        } else {
            DomainPosition::with_timestamp(timestamp)
        };
        
        self.positions.insert(domain_id.to_string(), position);
        Some(position)
    }
    
    /// Get the position of a domain
    pub fn get_position(&self, domain_id: &str) -> Option<DomainPosition> {
        self.positions.get(domain_id).copied()
    }
    
    /// Mark two domains as comparable
    pub fn mark_comparable(&mut self, domain_a: &str, domain_b: &str) {
        let domain_a = domain_a.to_string();
        let domain_b = domain_b.to_string();
        
        self.comparables
            .entry(domain_a.clone())
            .or_insert_with(HashSet::new)
            .insert(domain_b.clone());
            
        self.comparables
            .entry(domain_b)
            .or_insert_with(HashSet::new)
            .insert(domain_a);
    }
    
    /// Check if two domains are comparable
    pub fn are_comparable(&self, domain_a: &str, domain_b: &str) -> bool {
        if domain_a == domain_b {
            return true;
        }
        
        self.comparables
            .get(domain_a)
            .map(|set| set.contains(domain_b))
            .unwrap_or(false)
    }
    
    /// Merge the positions from another time map
    pub fn merge(&mut self, other: &TimeMap) {
        for (domain_id, position) in &other.positions {
            if let Some(existing) = self.positions.get(domain_id) {
                if position.timestamp > existing.timestamp {
                    self.positions.insert(domain_id.clone(), *position);
                }
            } else {
                self.positions.insert(domain_id.clone(), *position);
            }
        }
        
        for (domain_id, comparable_set) in &other.comparables {
            let entry = self.comparables
                .entry(domain_id.clone())
                .or_insert_with(HashSet::new);
                
            for comparable in comparable_set {
                entry.insert(comparable.clone());
            }
        }
    }
    
    /// Create a snapshot of the current time map
    pub fn snapshot(&self) -> TimeMapSnapshot {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Simple hash implementation - in a real system, this would be a proper hash
        let hash = format!("timemap-{}", timestamp);
        
        TimeMapSnapshot {
            positions: self.positions.clone(),
            timestamp,
            hash,
        }
    }
    
    /// Get all domain IDs in the map
    pub fn domain_ids(&self) -> Vec<DomainId> {
        self.positions.keys().cloned().collect()
    }
}

impl Default for TimeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TimeMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TimeMap {{")?;
        for (domain_id, position) in &self.positions {
            writeln!(f, "  {}: ts={}, idx={}", domain_id, position.timestamp, position.index)?;
        }
        writeln!(f, "}}")
    }
} 