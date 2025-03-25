//! ID types for the effect system
//!
//! This module defines ID types used throughout the effect system.

use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Identifier for an effect type
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EffectTypeId(Arc<str>);

impl EffectTypeId {
    /// Create a new effect type ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Arc::from(id.into()))
    }
    
    /// Get the string representation of this ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for EffectTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for EffectTypeId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for EffectTypeId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Identifier for an effect context
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ContextId(Arc<str>);

impl ContextId {
    /// Create a new context ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Arc::from(id.into()))
    }
    
    /// Generate a random context ID
    pub fn generate() -> Self {
        use uuid::Uuid;
        Self(Arc::from(Uuid::new_v4().to_string()))
    }
    
    /// Get the string representation of this ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a capability
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CapabilityId(Arc<str>);

impl CapabilityId {
    /// Create a new capability ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Arc::from(id.into()))
    }
    
    /// Get the string representation of this ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for CapabilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for CapabilityId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CapabilityId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Identifier for a domain
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DomainId(Arc<str>);

impl DomainId {
    /// Create a new domain ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Arc::from(id.into()))
    }
    
    /// Get the string representation of this ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for DomainId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for DomainId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Identifier for a resource
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId(Arc<str>);

impl ResourceId {
    /// Create a new resource ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Arc::from(id.into()))
    }
    
    /// Get the string representation of this ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ResourceId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ResourceId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_type_id() {
        let id = EffectTypeId::new("test.effect");
        assert_eq!(id.as_str(), "test.effect");
        
        // Test display
        assert_eq!(format!("{}", id), "test.effect");
        
        // Test from impls
        let id2: EffectTypeId = "test.effect".into();
        assert_eq!(id, id2);
        
        let id3: EffectTypeId = "test.effect".to_string().into();
        assert_eq!(id, id3);
    }
    
    #[test]
    fn test_context_id() {
        let id = ContextId::new("test.context");
        assert_eq!(id.as_str(), "test.context");
        
        // Test display
        assert_eq!(format!("{}", id), "test.context");
        
        // Test generate
        let generated = ContextId::generate();
        assert_ne!(generated.as_str(), "");
        assert_ne!(generated, ContextId::generate());
    }
    
    #[test]
    fn test_capability_id() {
        let id = CapabilityId::new("test.capability");
        assert_eq!(id.as_str(), "test.capability");
        
        // Test display
        assert_eq!(format!("{}", id), "test.capability");
        
        // Test from impls
        let id2: CapabilityId = "test.capability".into();
        assert_eq!(id, id2);
        
        let id3: CapabilityId = "test.capability".to_string().into();
        assert_eq!(id, id3);
    }
    
    #[test]
    fn test_domain_id() {
        let id = DomainId::new("test.domain");
        assert_eq!(id.as_str(), "test.domain");
        
        // Test display
        assert_eq!(format!("{}", id), "test.domain");
        
        // Test from impls
        let id2: DomainId = "test.domain".into();
        assert_eq!(id, id2);
        
        let id3: DomainId = "test.domain".to_string().into();
        assert_eq!(id, id3);
    }
    
    #[test]
    fn test_resource_id() {
        let id = ResourceId::new("test.resource");
        assert_eq!(id.as_str(), "test.resource");
        
        // Test display
        assert_eq!(format!("{}", id), "test.resource");
        
        // Test from impls
        let id2: ResourceId = "test.resource".into();
        assert_eq!(id, id2);
        
        let id3: ResourceId = "test.resource".to_string().into();
        assert_eq!(id, id3);
    }
} 