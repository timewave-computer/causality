// Resource reference implementations
//
// This file contains types and functions for working with resource references.

use std::fmt::{self, Display};
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use super::types::{ResourceId, ResourceType};

/// Resource reference
///
/// A reference to a resource in the system, which can be resolved
/// to access the actual resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceRef {
    /// Resource ID being referenced
    pub resource_id: ResourceId,
    
    /// Expected resource type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_type: Option<ResourceType>,
    
    /// Reference hint (e.g., for locating the resource)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ResourceRef {
    /// Create a new resource reference
    pub fn new(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            expected_type: None,
            hint: None,
        }
    }
    
    /// Create a new resource reference with expected type
    pub fn new_with_type(resource_id: ResourceId, expected_type: ResourceType) -> Self {
        Self {
            resource_id,
            expected_type: Some(expected_type),
            hint: None,
        }
    }
    
    /// Create a new resource reference with hint
    pub fn new_with_hint(resource_id: ResourceId, hint: impl Into<String>) -> Self {
        Self {
            resource_id,
            expected_type: None,
            hint: Some(hint.into()),
        }
    }
    
    /// Create a new resource reference with type and hint
    pub fn new_with_type_and_hint(
        resource_id: ResourceId, 
        expected_type: ResourceType,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            resource_id,
            expected_type: Some(expected_type),
            hint: Some(hint.into()),
        }
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    /// Get the expected resource type, if specified
    pub fn expected_type(&self) -> Option<&ResourceType> {
        self.expected_type.as_ref()
    }
    
    /// Get the reference hint, if specified
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
    
    /// Set the expected type for this reference
    pub fn with_expected_type(mut self, expected_type: ResourceType) -> Self {
        self.expected_type = Some(expected_type);
        self
    }
    
    /// Set the hint for this reference
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ref:{}", self.resource_id)?;
        
        if let Some(expected_type) = &self.expected_type {
            write!(f, ":{}", expected_type)?;
        }
        
        if let Some(hint) = &self.hint {
            write!(f, ":hint={}", hint)?;
        }
        
        Ok(())
    }
}

impl FromStr for ResourceRef {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format: ref:<resource_id>[:<expected_type>][:<hint=value>]
        if !s.starts_with("ref:") {
            return Err(format!("Resource reference must start with 'ref:': {}", s));
        }
        
        // Split the string into parts
        let parts: Vec<&str> = s[4..].split(':').collect();
        
        if parts.is_empty() {
            return Err("Resource reference must contain a resource ID".to_string());
        }
        
        // Parse the resource ID
        let resource_id = ResourceId::from_str(parts[0])
            .map_err(|e| format!("Invalid resource ID in reference: {}", e))?;
        
        let mut reference = ResourceRef::new(resource_id);
        
        // Parse additional parts
        for i in 1..parts.len() {
            let part = parts[i];
            
            if part.starts_with("hint=") {
                reference.hint = Some(part[5..].to_string());
            } else {
                // Assume it's a resource type
                match ResourceType::from_str(part) {
                    Ok(resource_type) => {
                        reference.expected_type = Some(resource_type);
                    }
                    Err(e) => {
                        return Err(format!("Invalid resource type in reference: {}", e));
                    }
                }
            }
        }
        
        Ok(reference)
    }
}

/// Resource reference collection
///
/// A collection of resource references.
#[derive(Debug, Clone, Default, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceRefCollection {
    /// List of resource references
    pub references: Vec<ResourceRef>,
}

impl ResourceRefCollection {
    /// Create a new empty resource reference collection
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
        }
    }
    
    /// Create a new resource reference collection with initial references
    pub fn with_references(references: Vec<ResourceRef>) -> Self {
        Self { references }
    }
    
    /// Add a reference to the collection
    pub fn add(&mut self, reference: ResourceRef) {
        self.references.push(reference);
    }
    
    /// Remove a reference from the collection
    pub fn remove(&mut self, resource_id: &ResourceId) -> bool {
        if let Some(pos) = self.references.iter().position(|r| &r.resource_id == resource_id) {
            self.references.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Check if the collection contains a reference to a resource
    pub fn contains(&self, resource_id: &ResourceId) -> bool {
        self.references.iter().any(|r| &r.resource_id == resource_id)
    }
    
    /// Find references of a specific type
    pub fn find_by_type(&self, resource_type: &ResourceType) -> Vec<&ResourceRef> {
        self.references
            .iter()
            .filter(|r| {
                if let Some(expected_type) = &r.expected_type {
                    expected_type.is_compatible_with(resource_type)
                } else {
                    false
                }
            })
            .collect()
    }
    
    /// Get all references in the collection
    pub fn all(&self) -> &[ResourceRef] {
        &self.references
    }
    
    /// Get the number of references in the collection
    pub fn len(&self) -> usize {
        self.references.len()
    }
    
    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }
}

/// Resource dependency
///
/// Represents a dependency between resources.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceDependency {
    /// Source resource
    pub source: ResourceRef,
    
    /// Target resource
    pub target: ResourceRef,
    
    /// Dependency type
    pub dependency_type: String,
    
    /// Whether this is a required dependency
    pub required: bool,
    
    /// Additional attributes
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
}

impl ResourceDependency {
    /// Create a new resource dependency
    pub fn new(
        source: ResourceRef,
        target: ResourceRef,
        dependency_type: impl Into<String>,
        required: bool,
    ) -> Self {
        Self {
            source,
            target,
            dependency_type: dependency_type.into(),
            required,
            attributes: std::collections::HashMap::new(),
        }
    }
    
    /// Set an attribute for this dependency
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
} 