// Domain capability management
//
// This module defines the capability management for domains, providing
// a way to register and query domain capabilities.

use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::RwLock;
use serde::{Serialize, Deserialize};

use crate::selection::DomainId;

/// Domain capability represents a specific functionality a domain can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DomainCapability {
    /// Unique name of the capability
    pub name: String,
    /// Description of what this capability provides
    pub description: String,
    /// Version of this capability
    pub version: String,
    /// Required permissions to use this capability
    pub required_permissions: Vec<String>,
    /// Configuration options available for this capability
    pub config_options: BTreeMap<String, String>,
}

impl DomainCapability {
    /// Create a new capability with the given name
    pub fn new(name: &str, description: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            required_permissions: Vec::new(),
            config_options: BTreeMap::new(),
        }
    }
    
    /// Add a required permission to this capability
    pub fn with_permission(mut self, permission: &str) -> Self {
        self.required_permissions.push(permission.to_string());
        self
    }
    
    /// Add a configuration option to this capability
    pub fn with_config_option(mut self, key: &str, value: &str) -> Self {
        self.config_options.insert(key.to_string(), value.to_string());
        self
    }
}

/// Domain capability manager - keeps track of capabilities for each domain
pub struct DomainCapabilityManager {
    /// Capabilities by domain
    capabilities: RwLock<HashMap<DomainId, HashSet<DomainCapability>>>,
}

impl DomainCapabilityManager {
    /// Create a new domain capability manager
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a capability for a domain
    pub fn register_capability(&self, domain_id: &DomainId, capability: DomainCapability) {
        let mut capabilities = self.capabilities.write().unwrap();
        let domain_capabilities = capabilities
            .entry(domain_id.clone())
            .or_insert_with(HashSet::new);
        domain_capabilities.insert(capability);
    }
    
    /// Check if a domain has a specific capability
    pub fn has_capability(&self, domain_id: &DomainId, capability_name: &str) -> bool {
        let capabilities = self.capabilities.read().unwrap();
        if let Some(domain_capabilities) = capabilities.get(domain_id) {
            domain_capabilities.iter().any(|cap| cap.name == capability_name)
        } else {
            false
        }
    }
    
    /// Get all capabilities for a domain
    pub fn get_capabilities(&self, domain_id: &DomainId) -> HashSet<DomainCapability> {
        let capabilities = self.capabilities.read().unwrap();
        capabilities.get(domain_id)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }
    
    /// Get domains that have a specific capability
    pub fn get_domains_with_capability(&self, capability_name: &str) -> Vec<DomainId> {
        let capabilities = self.capabilities.read().unwrap();
        capabilities.iter()
            .filter(|(_, caps)| {
                caps.iter().any(|cap| cap.name == capability_name)
            })
            .map(|(domain_id, _)| domain_id.clone())
            .collect()
    }
    
    /// Remove a capability from a domain
    pub fn remove_capability(&self, domain_id: &DomainId, capability_name: &str) {
        let mut capabilities = self.capabilities.write().unwrap();
        if let Some(domain_capabilities) = capabilities.get_mut(domain_id) {
            domain_capabilities.retain(|cap| cap.name != capability_name);
        }
    }
    
    /// Remove all capabilities for a domain
    pub fn remove_domain(&self, domain_id: &DomainId) {
        let mut capabilities = self.capabilities.write().unwrap();
        capabilities.remove(domain_id);
    }
}

impl Default for DomainCapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard capabilities that domains might support
pub mod standard_capabilities {
    /// Transaction capability - domain can process transactions
    pub const TRANSACTION: &str = "transaction";
    /// Storage capability - domain can store data
    pub const STORAGE: &str = "storage";
    /// Query capability - domain can execute queries
    pub const QUERY: &str = "query";
    /// Smart contract capability - domain can execute smart contracts
    pub const SMART_CONTRACT: &str = "smart_contract";
    /// Token capability - domain has token support
    pub const TOKEN: &str = "token";
    /// NFT capability - domain has NFT support
    pub const NFT: &str = "nft";
    /// Verification capability - domain can verify facts
    pub const VERIFICATION: &str = "verification";
    /// Time synchronization capability - domain supports time sync
    pub const TIME_SYNC: &str = "time_sync";
}

/// Extension trait for capabilities
pub trait CapabilityExtension {
    /// Get all capabilities this entity has
    fn capabilities(&self) -> Vec<String>;
    
    /// Check if entity has a specific capability
    fn has_capability(&self, name: &str) -> bool {
        self.capabilities().iter().any(|cap| cap == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_manager() {
        let manager = DomainCapabilityManager::new();
        
        // Create some capabilities
        let tx_capability = DomainCapability::new(
            "transaction", 
            "Ability to process transactions", 
            "1.0"
        );
        
        let storage_capability = DomainCapability::new(
            "storage", 
            "Ability to store data", 
            "1.0"
        );
        
        // Register capabilities for domains
        manager.register_capability(&"domain1".to_string(), tx_capability.clone());
        manager.register_capability(&"domain1".to_string(), storage_capability.clone());
        manager.register_capability(&"domain2".to_string(), tx_capability.clone());
        
        // Check capabilities
        assert!(manager.has_capability(&"domain1".to_string(), "transaction"));
        assert!(manager.has_capability(&"domain1".to_string(), "storage"));
        assert!(manager.has_capability(&"domain2".to_string(), "transaction"));
        assert!(!manager.has_capability(&"domain2".to_string(), "storage"));
        
        // Get all capabilities for a domain
        let domain1_caps = manager.get_capabilities(&"domain1".to_string());
        assert_eq!(domain1_caps.len(), 2);
        
        // Get domains with a capability
        let tx_domains = manager.get_domains_with_capability("transaction");
        assert_eq!(tx_domains.len(), 2);
        assert!(tx_domains.contains(&"domain1".to_string()));
        assert!(tx_domains.contains(&"domain2".to_string()));
        
        // Remove a capability
        manager.remove_capability(&"domain1".to_string(), "transaction");
        assert!(!manager.has_capability(&"domain1".to_string(), "transaction"));
        assert!(manager.has_capability(&"domain1".to_string(), "storage"));
        
        // Remove a domain
        manager.remove_domain(&"domain2".to_string());
        assert!(manager.get_capabilities(&"domain2".to_string()).is_empty());
    }
} 
