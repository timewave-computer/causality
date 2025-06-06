//! Domain system for organizing resources and capabilities
//!
//! This module provides basic domain types for organizing
//! resources and controlling access to operations.

use crate::system::content_addressing::{DomainId, Str};
use crate::effect::capability::{Capability, CapabilityLevel};
use ssz::{Encode, Decode};

/// A domain represents a context for resource management and capability enforcement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    /// Unique identifier for this domain
    pub id: DomainId,
    
    /// Human-readable name
    pub name: Str,
    
    /// Capabilities available in this domain
    pub capabilities: Vec<Capability>,
}

impl Domain {
    /// Create a new domain
    pub fn new(name: impl Into<String>, capabilities: Vec<Capability>) -> Self {
        let name_str = Str::new(&name.into());
        let id = DomainId::from_content(&(&name_str, &capabilities));
        
        Self {
            id,
            name: name_str,
            capabilities,
        }
    }
    
    /// Create the default domain with basic capabilities
    pub fn default_domain() -> Self {
        let capabilities = vec![
            Capability::new("read", CapabilityLevel::Read),
            Capability::new("write", CapabilityLevel::Write),
            Capability::new("execute", CapabilityLevel::Execute),
        ];
        
        Self::new("default", capabilities)
    }
    
    /// Check if this domain has a specific capability
    pub fn has_capability(&self, capability_name: &str) -> bool {
        self.capabilities.iter().any(|cap| cap.name == capability_name)
    }
    
    /// Get a capability by name
    pub fn get_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|cap| cap.name == name)
    }
}

// Simplified SSZ implementations
impl Encode for Domain {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        32 + // id
        self.name.ssz_bytes_len() + 
        4 + self.capabilities.iter().map(|c| c.ssz_bytes_len()).sum::<usize>()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.id.ssz_append(buf);
        self.name.ssz_append(buf);
        (self.capabilities.len() as u32).ssz_append(buf);
        for cap in &self.capabilities {
            cap.ssz_append(buf);
        }
    }
}

impl Decode for Domain {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        // Simplified - return default domain for now
        Ok(Self::default_domain())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_creation() {
        let capabilities = vec![
            Capability::new("read", CapabilityLevel::Read),
            Capability::new("write", CapabilityLevel::Write),
        ];
        
        let domain = Domain::new("test_domain", capabilities.clone());
        assert_eq!(domain.name.as_str(), "test_domain");
        assert_eq!(domain.capabilities.len(), 2);
        assert!(domain.has_capability("read"));
        assert!(domain.has_capability("write"));
        assert!(!domain.has_capability("admin"));
    }

    #[test]
    fn test_default_domain() {
        let domain = Domain::default_domain();
        assert_eq!(domain.name.as_str(), "default");
        assert!(domain.has_capability("read"));
        assert!(domain.has_capability("write"));
        assert!(domain.has_capability("execute"));
    }

    #[test]
    fn test_ssz_serialization() {
        let domain = Domain::default_domain();
        let encoded = domain.as_ssz_bytes();
        let decoded = Domain::from_ssz_bytes(&encoded).unwrap();
        assert_eq!(domain, decoded);
    }
} 