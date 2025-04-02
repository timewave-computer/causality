// Time map implementation
// Local implementation of TimeMap to avoid external dependencies

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use causality_types::DomainId;

/// TimeMap for tracking causal time across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMap {
    /// Map of domain IDs to timestamps
    time_entries: HashMap<DomainId, u64>,
    /// Frozen status - if true, the map cannot be modified
    frozen: bool,
}

impl TimeMap {
    /// Create a new empty time map
    pub fn new() -> Self {
        TimeMap {
            time_entries: HashMap::new(),
            frozen: false,
        }
    }
    
    /// Create a new time map with the given entries
    pub fn with_entries(entries: HashMap<DomainId, u64>) -> Self {
        TimeMap {
            time_entries: entries,
            frozen: false,
        }
    }
    
    /// Get the timestamp for a domain
    pub fn get(&self, domain_id: &DomainId) -> Option<u64> {
        self.time_entries.get(domain_id).copied()
    }
    
    /// Set the timestamp for a domain
    pub fn set(&mut self, domain_id: DomainId, timestamp: u64) -> Result<(), &'static str> {
        if self.frozen {
            return Err("TimeMap is frozen and cannot be modified");
        }
        
        // Only update if the new timestamp is greater
        if let Some(current) = self.time_entries.get(&domain_id) {
            if timestamp <= *current {
                return Ok(());
            }
        }
        
        self.time_entries.insert(domain_id, timestamp);
        Ok(())
    }
    
    /// Merge with another time map, taking the maximum timestamps
    pub fn merge(&mut self, other: &TimeMap) -> Result<(), &'static str> {
        if self.frozen {
            return Err("TimeMap is frozen and cannot be modified");
        }
        
        for (domain_id, timestamp) in &other.time_entries {
            if let Some(current) = self.time_entries.get(domain_id) {
                if timestamp > current {
                    self.time_entries.insert(domain_id.clone(), *timestamp);
                }
            } else {
                self.time_entries.insert(domain_id.clone(), *timestamp);
            }
        }
        
        Ok(())
    }
    
    /// Freeze the time map to prevent further modifications
    pub fn freeze(&mut self) {
        self.frozen = true;
    }
    
    /// Check if this time map is frozen
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
    
    /// Get all entries in the time map
    pub fn entries(&self) -> &HashMap<DomainId, u64> {
        &self.time_entries
    }
    
    /// Create a mutable copy of this time map
    pub fn to_mutable(&self) -> TimeMap {
        TimeMap {
            time_entries: self.time_entries.clone(),
            frozen: false,
        }
    }
    
    /// Check if this time map is causally after another time map
    pub fn is_after(&self, other: &TimeMap) -> bool {
        // Check if all entries in other are present in self with greater or equal timestamps
        for (domain_id, timestamp) in &other.time_entries {
            match self.time_entries.get(domain_id) {
                Some(self_timestamp) if *self_timestamp >= *timestamp => continue,
                _ => return false,
            }
        }
        
        // Check if self has at least one entry with a greater timestamp
        for (domain_id, self_timestamp) in &self.time_entries {
            match other.time_entries.get(domain_id) {
                Some(other_timestamp) if *self_timestamp > *other_timestamp => return true,
                None => return true, // Self has an entry that other doesn't have
                _ => continue,
            }
        }
        
        false
    }
}

impl Default for TimeMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Tests for the TimeMap implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_map_basic() {
        let mut time_map = TimeMap::new();
        let domain_a = DomainId::new("domain-a");
        let domain_b = DomainId::new("domain-b");
        
        // Set and get timestamps
        assert!(time_map.set(domain_a.clone(), 100).is_ok());
        assert!(time_map.set(domain_b.clone(), 200).is_ok());
        
        assert_eq!(time_map.get(&domain_a), Some(100));
        assert_eq!(time_map.get(&domain_b), Some(200));
        assert_eq!(time_map.get(&DomainId::new("domain-c")), None);
    }
    
    #[test]
    fn test_time_map_merge() {
        let mut time_map1 = TimeMap::new();
        let mut time_map2 = TimeMap::new();
        
        let domain_a = DomainId::new("domain-a");
        let domain_b = DomainId::new("domain-b");
        let domain_c = DomainId::new("domain-c");
        
        // Set up the time maps
        time_map1.set(domain_a.clone(), 100).unwrap();
        time_map1.set(domain_b.clone(), 200).unwrap();
        
        time_map2.set(domain_a.clone(), 150).unwrap(); // Greater than in map1
        time_map2.set(domain_c.clone(), 300).unwrap(); // Not in map1
        
        // Merge map2 into map1
        time_map1.merge(&time_map2).unwrap();
        
        // Check the result
        assert_eq!(time_map1.get(&domain_a), Some(150)); // Updated
        assert_eq!(time_map1.get(&domain_b), Some(200)); // Unchanged
        assert_eq!(time_map1.get(&domain_c), Some(300)); // Added
    }
    
    #[test]
    fn test_time_map_frozen() {
        let mut time_map = TimeMap::new();
        let domain_a = DomainId::new("domain-a");
        
        // Set a timestamp
        time_map.set(domain_a.clone(), 100).unwrap();
        
        // Freeze the time map
        time_map.freeze();
        assert!(time_map.is_frozen());
        
        // Try to modify the frozen map
        assert!(time_map.set(domain_a.clone(), 200).is_err());
        
        // The timestamp should remain unchanged
        assert_eq!(time_map.get(&domain_a), Some(100));
        
        // Create a mutable copy
        let mut mutable_copy = time_map.to_mutable();
        assert!(!mutable_copy.is_frozen());
        
        // Modify the mutable copy
        assert!(mutable_copy.set(domain_a.clone(), 200).is_ok());
        assert_eq!(mutable_copy.get(&domain_a), Some(200));
    }
    
    #[test]
    fn test_time_map_causality() {
        let mut map1 = TimeMap::new();
        let mut map2 = TimeMap::new();
        let mut map3 = TimeMap::new();
        
        let domain_a = DomainId::new("domain-a");
        let domain_b = DomainId::new("domain-b");
        
        // Set up map1
        map1.set(domain_a.clone(), 100).unwrap();
        map1.set(domain_b.clone(), 200).unwrap();
        
        // Set up map2 (causally after map1)
        map2.set(domain_a.clone(), 150).unwrap();
        map2.set(domain_b.clone(), 200).unwrap();
        
        // Set up map3 (concurrent with map1)
        map3.set(domain_a.clone(), 100).unwrap();
        map3.set(domain_b.clone(), 150).unwrap();
        
        // Check causality
        assert!(map2.is_after(&map1));
        assert!(!map1.is_after(&map2));
        assert!(!map1.is_after(&map3));
        assert!(!map3.is_after(&map1));
    }
} 