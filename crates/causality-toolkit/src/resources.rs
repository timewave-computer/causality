
//! Resource management utilities for the Causality toolkit.

use std::collections::BTreeMap;
use causality_core::{EntityId, Value};
use sha2::{Sha256, Digest};

/// Resource manager for handling system resources
#[derive(Debug, Clone)]
pub struct ResourceManager {
    resources: BTreeMap<EntityId, Value>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }
    
    /// Add a resource to the manager
    pub fn add_resource(&mut self, id: EntityId, value: Value) {
        self.resources.insert(id, value);
    }
    
    /// Get a resource by ID
    pub fn get_resource(&self, id: &EntityId) -> Option<&Value> {
        self.resources.get(id)
    }
    
    /// Remove a resource by ID
    pub fn remove_resource(&mut self, id: &EntityId) -> Option<Value> {
        self.resources.remove(id)
    }
    
    /// List all resource IDs
    pub fn list_resources(&self) -> Vec<EntityId> {
        self.resources.keys().cloned().collect()
    }
    
    /// Clear all resources
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    /// Get the number of resources
    pub fn count(&self) -> usize {
        self.resources.len()
    }
    
    /// Create a new resource with a name and initial balance using content-addressing
    pub fn create_resource(&mut self, name: &str, initial_balance: u64) -> EntityId {
        // Create content-addressed ID using SHA-256 for deterministic addressing
        let mut hasher = Sha256::new();
        hasher.update(b"resource:");
        hasher.update(name.as_bytes());
        hasher.update(b":");
        hasher.update(initial_balance.to_le_bytes());
        
        let hash = hasher.finalize();
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&hash);
        
        let id = EntityId::new(id_bytes);
        let value = Value::Int(initial_balance as u32);
        
        // Store the resource
        self.resources.insert(id, value);
        
        id
    }
    
    /// Get the balance of a resource (assumes it's stored as an Int value)
    pub fn get_resource_balance(&self, id: &EntityId) -> Option<u64> {
        match self.resources.get(id) {
            Some(Value::Int(balance)) => Some(*balance as u64),
            _ => None,
        }
    }
    
    /// Transfer resources between two resource IDs
    pub fn transfer_resource(&mut self, from_id: &EntityId, to_id: &EntityId, amount: u64) -> bool {
        // Get current balances
        let from_balance = match self.get_resource_balance(from_id) {
            Some(balance) => balance,
            None => return false,
        };
        
        let to_balance = self.get_resource_balance(to_id).unwrap_or(0);
        
        // Check if transfer is possible
        if from_balance < amount {
            return false;
        }
        
        // Perform transfer
        let new_from_balance = from_balance - amount;
        let new_to_balance = to_balance + amount;
        
        self.resources.insert(*from_id, Value::Int(new_from_balance as u32));
        self.resources.insert(*to_id, Value::Int(new_to_balance as u32));
        
        true
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_addressed_resources() {
        let mut manager = ResourceManager::new();
        
        // Same content should produce same IDs
        let id1 = manager.create_resource("token", 100);
        let id2 = manager.create_resource("token", 100);
        assert_eq!(id1, id2, "Same content should produce same EntityId");
        
        // Different content should produce different IDs
        let id3 = manager.create_resource("token", 200);
        let id4 = manager.create_resource("other_token", 100);
        assert_ne!(id1, id3, "Different balance should produce different EntityId");
        assert_ne!(id1, id4, "Different name should produce different EntityId");
        
        // Verify resources are accessible by their content-addressed IDs
        assert_eq!(manager.get_resource_balance(&id1), Some(100));
        assert_eq!(manager.get_resource_balance(&id3), Some(200));
        assert_eq!(manager.get_resource_balance(&id4), Some(100));
    }
} 