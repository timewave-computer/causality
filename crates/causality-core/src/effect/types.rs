// Effect Types
//
// This module defines core types for the effect system, including
// identifiers, type information, and content addressing integration.

use std::fmt::{self, Debug, Display};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use causality_types::ContentId;

use crate::serialization::{to_bytes, from_bytes};

/// Unique identifier for an effect instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectId(ContentId);

impl EffectId {
    /// Create a new unique effect ID
    pub fn new_unique() -> Self {
        Self(ContentId::generate())
    }

    /// Create an effect ID from a content ID
    pub fn from_content_id(id: ContentId) -> Self {
        Self(id)
    }

    /// Get the inner content ID
    pub fn as_content_id(&self) -> &ContentId {
        &self.0
    }
    
    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        format!("Effect-{}", self.0)
    }
}

impl Display for EffectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Effect-{}", self.0)
    }
}

/// Type ID for categorizing effects, with content addressing support
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectTypeId {
    /// Name of the effect type
    name: String,
    
    /// Optional namespace for organization
    namespace: Option<String>,
    
    /// Version of the effect type (for evolution)
    version: Option<String>,
    
    /// Content hash for addressing and verification
    content_hash: Option<ContentId>,
}

impl EffectTypeId {
    /// Create a new effect type ID with just a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: None,
            version: None,
            content_hash: None,
        }
    }
    
    /// Create a new effect type ID with namespace and name
    pub fn with_namespace(namespace: &str, name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: Some(namespace.to_string()),
            version: None,
            content_hash: None,
        }
    }
    
    /// Create a new effect type ID with namespace, name, and version
    pub fn with_version(namespace: &str, name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: Some(namespace.to_string()),
            version: Some(version.to_string()),
            content_hash: None,
        }
    }
    
    /// Set the content hash for this effect type
    pub fn with_content_hash(mut self, hash: ContentId) -> Self {
        self.content_hash = Some(hash);
        self
    }
    
    /// Compute the content hash for this effect type
    pub fn compute_content_hash(&mut self) -> Result<ContentId, anyhow::Error> {
        let bytes = to_bytes(self)?;
        let content_id = ContentId::from_bytes(&bytes)?;
        self.content_hash = Some(content_id.clone());
        Ok(content_id)
    }
    
    /// Get the name of this effect type
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the namespace of this effect type, if any
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
    
    /// Get the version of this effect type, if any
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    
    /// Get the content hash of this effect type, if set
    pub fn content_hash(&self) -> Option<&ContentId> {
        self.content_hash.as_ref()
    }
    
    /// Get the fully qualified name of this effect type
    pub fn qualified_name(&self) -> String {
        match (&self.namespace, &self.version) {
            (Some(ns), Some(ver)) => format!("{}:{}:{}", ns, self.name, ver),
            (Some(ns), None) => format!("{}:{}", ns, self.name),
            (None, Some(ver)) => format!("{}:{}", self.name, ver),
            (None, None) => self.name.clone(),
        }
    }
    
    /// Check if this effect type is compatible with another type
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // If content hashes are present and equal, they're definitely compatible
        if let (Some(h1), Some(h2)) = (self.content_hash(), other.content_hash()) {
            if h1 == h2 {
                return true;
            }
        }
        
        // Otherwise check by name, namespace, and version
        self.name == other.name && 
        self.namespace == other.namespace &&
        match (&self.version, &other.version) {
            // If both have versions, they must match
            (Some(v1), Some(v2)) => v1 == v2,
            // If one has no version, consider compatible
            _ => true,
        }
    }
}

impl Display for EffectTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.qualified_name())
    }
}

impl From<&str> for EffectTypeId {
    fn from(s: &str) -> Self {
        // Parse qualified names like "namespace:name:version"
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            1 => Self::new(parts[0]),
            2 => Self::with_namespace(parts[0], parts[1]),
            _ => Self::with_version(parts[0], parts[1], parts[2]),
        }
    }
}

/// The target boundary for effect execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionBoundary {
    /// Execute inside the system boundary (within the local system)
    Inside,
    /// Execute outside the system boundary (on external systems)
    Outside,
    /// Execute at the system boundary (interface between local and external)
    Boundary,
    /// Execution boundary doesn't matter
    Any,
} 