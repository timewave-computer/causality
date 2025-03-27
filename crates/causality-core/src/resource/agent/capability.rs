// capability.rs - Capability bundle system for agent resources
//
// This module defines capability bundles, which are predefined sets of capabilities
// that can be assigned to agents based on roles or needed access patterns.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::agent::types::{AgentId, AgentError};
use crate::resource::capabilities::{Capability, CapabilityId};
use crate::crypto::ContentHash;

use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Errors that can occur when working with capability bundles
#[derive(Error, Debug)]
pub enum CapabilityBundleError {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Bundle validation error
    #[error("Bundle validation error: {0}")]
    ValidationError(String),
    
    /// Capability error
    #[error("Capability error: {0}")]
    CapabilityError(String),
    
    /// Delegation error
    #[error("Delegation error: {0}")]
    DelegationError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// A result type for capability bundle operations
pub type CapabilityBundleResult<T> = Result<T, CapabilityBundleError>;

/// Unique identifier for a capability bundle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityBundleId(pub String);

impl CapabilityBundleId {
    /// Create a new capability bundle ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Generate a capability bundle ID from content hash
    pub fn from_content_hash(hash: ContentHash) -> Self {
        Self(format!("bundle:{}", hash))
    }
    
    /// Get the string representation of the bundle ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityBundleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CapabilityBundleId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CapabilityBundleId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Scope of a capability bundle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityBundleScope {
    /// Bundle applies to a specific resource
    Resource(ResourceId),
    
    /// Bundle applies to all resources of a specific type
    ResourceType(ResourceType),
    
    /// Bundle applies globally
    Global,
    
    /// Bundle applies to resources matching a pattern
    Pattern(String),
}

/// Delegation rules for a capability bundle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationRules {
    /// Whether delegation is allowed
    pub allow_delegation: bool,
    
    /// Maximum depth of delegation chain
    pub max_delegation_depth: Option<u32>,
    
    /// Allowed delegatee agent IDs (if empty, any agent can be a delegatee)
    pub allowed_delegatees: HashSet<AgentId>,
    
    /// Whether sub-delegation is allowed (delegatee can further delegate)
    pub allow_sub_delegation: bool,
    
    /// Time limit for delegation, in seconds (if None, no time limit)
    pub time_limit: Option<u64>,
}

impl Default for DelegationRules {
    fn default() -> Self {
        Self {
            allow_delegation: false,
            max_delegation_depth: None,
            allowed_delegatees: HashSet::new(),
            allow_sub_delegation: false,
            time_limit: None,
        }
    }
}

/// A capability bundle is a named collection of capabilities
/// that can be assigned to agents as a unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityBundle {
    /// Unique identifier for the bundle
    pub id: CapabilityBundleId,
    
    /// Human-readable name for the bundle
    pub name: String,
    
    /// Description of the bundle's purpose and contents
    pub description: String,
    
    /// The capabilities included in this bundle
    pub capabilities: Vec<Capability>,
    
    /// Scope of the bundle (what resources it applies to)
    pub scope: CapabilityBundleScope,
    
    /// Delegation rules for this bundle
    pub delegation_rules: DelegationRules,
    
    /// Bundle metadata
    pub metadata: HashMap<String, String>,
}

impl CapabilityBundle {
    /// Create a new capability bundle
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        scope: CapabilityBundleScope,
    ) -> Self {
        let name = name.into();
        let description = description.into();
        
        // Generate ID from name and scope
        let id_str = format!("{}:{:?}", name, scope);
        let bundle_id = CapabilityBundleId::from_content_hash(ContentHash::calculate(id_str.as_bytes()));
        
        Self {
            id: bundle_id,
            name,
            description,
            capabilities: Vec::new(),
            scope,
            delegation_rules: DelegationRules::default(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a capability to the bundle
    pub fn add_capability(&mut self, capability: Capability) -> &mut Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Set delegation rules for the bundle
    pub fn with_delegation_rules(mut self, rules: DelegationRules) -> Self {
        self.delegation_rules = rules;
        self
    }
    
    /// Add metadata to the bundle
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Get the bundle ID
    pub fn id(&self) -> &CapabilityBundleId {
        &self.id
    }
    
    /// Get the bundle name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the bundle description
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// Get the capabilities in the bundle
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    
    /// Get the bundle scope
    pub fn scope(&self) -> &CapabilityBundleScope {
        &self.scope
    }
    
    /// Get the delegation rules
    pub fn delegation_rules(&self) -> &DelegationRules {
        &self.delegation_rules
    }
    
    /// Get the bundle metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Check if a resource is in scope for this bundle
    pub fn is_in_scope(&self, resource_id: &ResourceId, resource_type: &ResourceType) -> bool {
        match &self.scope {
            CapabilityBundleScope::Resource(scope_resource_id) => {
                scope_resource_id == resource_id
            },
            CapabilityBundleScope::ResourceType(scope_resource_type) => {
                scope_resource_type == resource_type
            },
            CapabilityBundleScope::Global => true,
            CapabilityBundleScope::Pattern(pattern) => {
                // Check if resource ID matches the pattern
                resource_id.to_string().contains(pattern)
            }
        }
    }
    
    /// Get capabilities that apply to a specific resource
    pub fn get_capabilities_for_resource(
        &self,
        resource_id: &ResourceId,
        resource_type: &ResourceType,
    ) -> Vec<&Capability> {
        if !self.is_in_scope(resource_id, resource_type) {
            return Vec::new();
        }
        
        self.capabilities.iter().collect()
    }
}

/// Builder for creating capability bundles
pub struct CapabilityBundleBuilder {
    /// Bundle name
    name: String,
    
    /// Bundle description
    description: String,
    
    /// Bundle scope
    scope: CapabilityBundleScope,
    
    /// Capabilities to include
    capabilities: Vec<Capability>,
    
    /// Delegation rules
    delegation_rules: DelegationRules,
    
    /// Bundle metadata
    metadata: HashMap<String, String>,
}

impl CapabilityBundleBuilder {
    /// Create a new capability bundle builder
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scope: CapabilityBundleScope::Global,
            capabilities: Vec::new(),
            delegation_rules: DelegationRules::default(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the bundle scope
    pub fn scope(mut self, scope: CapabilityBundleScope) -> Self {
        self.scope = scope;
        self
    }
    
    /// Add a capability to the bundle
    pub fn add_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Set delegation rules
    pub fn delegation_rules(mut self, rules: DelegationRules) -> Self {
        self.delegation_rules = rules;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Build the capability bundle
    pub fn build(self) -> CapabilityBundle {
        let mut bundle = CapabilityBundle::new(
            self.name,
            self.description,
            self.scope,
        );
        
        // Add capabilities
        bundle.capabilities = self.capabilities;
        
        // Add delegation rules
        bundle.delegation_rules = self.delegation_rules;
        
        // Add metadata
        bundle.metadata = self.metadata;
        
        bundle
    }
}

/// A capability bundle delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDelegation {
    /// The bundle being delegated
    pub bundle_id: CapabilityBundleId,
    
    /// The agent delegating the bundle
    pub delegator: AgentId,
    
    /// The agent receiving the delegation
    pub delegatee: AgentId,
    
    /// When the delegation was created
    pub created_at: u64,
    
    /// When the delegation expires (if applicable)
    pub expires_at: Option<u64>,
    
    /// Whether the delegation can be further delegated
    pub allows_sub_delegation: bool,
    
    /// Delegation metadata
    pub metadata: HashMap<String, String>,
}

impl CapabilityDelegation {
    /// Create a new capability delegation
    pub fn new(
        bundle_id: CapabilityBundleId,
        delegator: AgentId,
        delegatee: AgentId,
        created_at: u64,
    ) -> Self {
        Self {
            bundle_id,
            delegator,
            delegatee,
            created_at,
            expires_at: None,
            allows_sub_delegation: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Set an expiration time
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Allow sub-delegation
    pub fn with_sub_delegation(mut self, allows_sub_delegation: bool) -> Self {
        self.allows_sub_delegation = allows_sub_delegation;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Check if the delegation is expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_time >= expires_at
        } else {
            false
        }
    }
}

/// Manager for capability bundles
pub struct CapabilityBundleManager {
    /// Available bundles
    bundles: HashMap<CapabilityBundleId, CapabilityBundle>,
    
    /// Active delegations
    delegations: HashMap<(AgentId, CapabilityBundleId), Vec<CapabilityDelegation>>,
}

impl CapabilityBundleManager {
    /// Create a new capability bundle manager
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
            delegations: HashMap::new(),
        }
    }
    
    /// Register a capability bundle
    pub fn register_bundle(&mut self, bundle: CapabilityBundle) -> CapabilityBundleResult<()> {
        if self.bundles.contains_key(&bundle.id) {
            return Err(CapabilityBundleError::ValidationError(
                format!("Bundle with ID {} already exists", bundle.id)
            ));
        }
        
        self.bundles.insert(bundle.id.clone(), bundle);
        Ok(())
    }
    
    /// Create and register a standard bundle
    pub fn register_standard_bundle(
        &mut self,
        bundle_type: StandardBundleType,
        resource_type: Option<ResourceType>,
    ) -> CapabilityBundleResult<CapabilityBundleId> {
        let bundle = Self::create_standard_bundle(bundle_type, resource_type)?;
        let bundle_id = bundle.id.clone();
        self.register_bundle(bundle)?;
        Ok(bundle_id)
    }
    
    /// Get a registered bundle by ID
    pub fn get_bundle(&self, bundle_id: &CapabilityBundleId) -> Option<&CapabilityBundle> {
        self.bundles.get(bundle_id)
    }
    
    /// Get all registered bundles
    pub fn get_all_bundles(&self) -> Vec<&CapabilityBundle> {
        self.bundles.values().collect()
    }
    
    /// Remove a bundle
    pub fn remove_bundle(&mut self, bundle_id: &CapabilityBundleId) -> CapabilityBundleResult<()> {
        if !self.bundles.contains_key(bundle_id) {
            return Err(CapabilityBundleError::ValidationError(
                format!("Bundle with ID {} does not exist", bundle_id)
            ));
        }
        
        self.bundles.remove(bundle_id);
        
        // Remove any delegations for this bundle
        let keys_to_remove: Vec<(AgentId, CapabilityBundleId)> = self.delegations
            .keys()
            .filter(|(_, bid)| bid == bundle_id)
            .cloned()
            .collect();
        
        for key in keys_to_remove {
            self.delegations.remove(&key);
        }
        
        Ok(())
    }
    
    /// Delegate a bundle to an agent
    pub fn delegate_bundle(
        &mut self,
        bundle_id: &CapabilityBundleId,
        delegator: &AgentId,
        delegatee: &AgentId,
        current_time: u64,
    ) -> CapabilityBundleResult<CapabilityDelegation> {
        // Verify the bundle exists
        let bundle = self.bundles.get(bundle_id)
            .ok_or_else(|| CapabilityBundleError::ValidationError(
                format!("Bundle with ID {} does not exist", bundle_id)
            ))?;
            
        // Check delegation rules
        if !bundle.delegation_rules.allow_delegation {
            return Err(CapabilityBundleError::DelegationError(
                format!("Bundle {} does not allow delegation", bundle_id)
            ));
        }
        
        // Check if delegatee is allowed
        if !bundle.delegation_rules.allowed_delegatees.is_empty() && 
           !bundle.delegation_rules.allowed_delegatees.contains(delegatee) {
            return Err(CapabilityBundleError::DelegationError(
                format!("Agent {} is not allowed as a delegatee for bundle {}", delegatee, bundle_id)
            ));
        }
        
        // Create the delegation
        let mut delegation = CapabilityDelegation::new(
            bundle_id.clone(),
            delegator.clone(),
            delegatee.clone(),
            current_time,
        );
        
        // Set expiration if specified in rules
        if let Some(time_limit) = bundle.delegation_rules.time_limit {
            delegation.expires_at = Some(current_time + time_limit);
        }
        
        // Set sub-delegation permission
        delegation.allows_sub_delegation = bundle.delegation_rules.allow_sub_delegation;
        
        // Store the delegation
        let key = (delegatee.clone(), bundle_id.clone());
        let delegations = self.delegations.entry(key).or_insert_with(Vec::new);
        delegations.push(delegation.clone());
        
        Ok(delegation)
    }
    
    /// Revoke a delegation
    pub fn revoke_delegation(
        &mut self,
        bundle_id: &CapabilityBundleId,
        delegator: &AgentId,
        delegatee: &AgentId,
    ) -> CapabilityBundleResult<()> {
        let key = (delegatee.clone(), bundle_id.clone());
        
        if let Some(delegations) = self.delegations.get_mut(&key) {
            // Find delegations from this delegator
            let original_len = delegations.len();
            delegations.retain(|d| &d.delegator != delegator);
            
            if delegations.len() == original_len {
                return Err(CapabilityBundleError::DelegationError(
                    format!("No delegation found from {} to {} for bundle {}", 
                            delegator, delegatee, bundle_id)
                ));
            }
            
            // If no delegations left, remove the entry
            if delegations.is_empty() {
                self.delegations.remove(&key);
            }
            
            Ok(())
        } else {
            Err(CapabilityBundleError::DelegationError(
                format!("No delegations found for agent {} and bundle {}", 
                        delegatee, bundle_id)
            ))
        }
    }
    
    /// Check if an agent has a bundle
    pub fn has_bundle(
        &self,
        agent_id: &AgentId,
        bundle_id: &CapabilityBundleId,
        current_time: u64,
    ) -> bool {
        let key = (agent_id.clone(), bundle_id.clone());
        
        if let Some(delegations) = self.delegations.get(&key) {
            // Check if there's at least one non-expired delegation
            delegations.iter().any(|d| !d.is_expired(current_time))
        } else {
            false
        }
    }
    
    /// Get all bundles for an agent
    pub fn get_agent_bundles(
        &self,
        agent_id: &AgentId,
        current_time: u64,
    ) -> Vec<CapabilityBundleId> {
        self.delegations.iter()
            .filter_map(|((aid, bid), delegations)| {
                if aid == agent_id && 
                   delegations.iter().any(|d| !d.is_expired(current_time)) {
                    Some(bid.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all capabilities for a resource that an agent has via bundles
    pub fn get_agent_capabilities_for_resource(
        &self,
        agent_id: &AgentId,
        resource_id: &ResourceId,
        resource_type: &ResourceType,
        current_time: u64,
    ) -> Vec<Capability> {
        let mut capabilities = Vec::new();
        
        // Get all bundles the agent has
        let bundle_ids = self.get_agent_bundles(agent_id, current_time);
        
        // Get capabilities from each bundle that apply to the resource
        for bundle_id in bundle_ids {
            if let Some(bundle) = self.bundles.get(&bundle_id) {
                if bundle.is_in_scope(resource_id, resource_type) {
                    capabilities.extend(bundle.capabilities.clone());
                }
            }
        }
        
        // Remove duplicates (same capability ID and permissions)
        let mut unique_capabilities: HashMap<String, Capability> = HashMap::new();
        for cap in capabilities {
            let key = format!("{}:{}", cap.id(), cap.permissions().join(","));
            unique_capabilities.insert(key, cap);
        }
        
        unique_capabilities.into_values().collect()
    }
    
    /// Create a standard capability bundle
    pub fn create_standard_bundle(
        bundle_type: StandardBundleType,
        resource_type: Option<ResourceType>,
    ) -> CapabilityBundleResult<CapabilityBundle> {
        match bundle_type {
            StandardBundleType::ReadOnly => {
                let scope = match resource_type {
                    Some(rt) => CapabilityBundleScope::ResourceType(rt),
                    None => CapabilityBundleScope::Global,
                };
                
                let mut builder = CapabilityBundleBuilder::new(
                    "Read Only",
                    "Provides read-only access to resources",
                )
                .scope(scope)
                .add_capability(Capability::new("read", vec!["read"]))
                .add_capability(Capability::new("list", vec!["list"]))
                .add_capability(Capability::new("describe", vec!["describe"]))
                .with_metadata("standard_bundle", "true");
                
                Ok(builder.build())
            },
            StandardBundleType::ReadWrite => {
                let scope = match resource_type {
                    Some(rt) => CapabilityBundleScope::ResourceType(rt),
                    None => CapabilityBundleScope::Global,
                };
                
                let mut builder = CapabilityBundleBuilder::new(
                    "Read Write",
                    "Provides read and write access to resources",
                )
                .scope(scope)
                .add_capability(Capability::new("read", vec!["read"]))
                .add_capability(Capability::new("write", vec!["write"]))
                .add_capability(Capability::new("list", vec!["list"]))
                .add_capability(Capability::new("describe", vec!["describe"]))
                .with_metadata("standard_bundle", "true");
                
                Ok(builder.build())
            },
            StandardBundleType::Admin => {
                let scope = match resource_type {
                    Some(rt) => CapabilityBundleScope::ResourceType(rt),
                    None => CapabilityBundleScope::Global,
                };
                
                let mut builder = CapabilityBundleBuilder::new(
                    "Admin",
                    "Provides administrative access to resources",
                )
                .scope(scope)
                .add_capability(Capability::new("read", vec!["read"]))
                .add_capability(Capability::new("write", vec!["write"]))
                .add_capability(Capability::new("delete", vec!["delete"]))
                .add_capability(Capability::new("create", vec!["create"]))
                .add_capability(Capability::new("list", vec!["list"]))
                .add_capability(Capability::new("describe", vec!["describe"]))
                .add_capability(Capability::new("admin", vec!["admin"]))
                .with_metadata("standard_bundle", "true");
                
                // Allow delegation for admin bundles
                let delegation_rules = DelegationRules {
                    allow_delegation: true,
                    allow_sub_delegation: false,
                    ..Default::default()
                };
                
                builder = builder.delegation_rules(delegation_rules);
                
                Ok(builder.build())
            },
            StandardBundleType::UserBasic => {
                let builder = CapabilityBundleBuilder::new(
                    "User Basic",
                    "Basic capabilities for user agents",
                )
                .scope(CapabilityBundleScope::Global)
                .add_capability(Capability::new("user.profile.read", vec!["read"]))
                .add_capability(Capability::new("user.profile.update", vec!["write"]))
                .add_capability(Capability::new("resource.list", vec!["list"]))
                .add_capability(Capability::new("resource.describe", vec!["describe"]))
                .with_metadata("standard_bundle", "true")
                .with_metadata("agent_type", "user");
                
                Ok(builder.build())
            },
            StandardBundleType::CommitteeBasic => {
                let builder = CapabilityBundleBuilder::new(
                    "Committee Basic",
                    "Basic capabilities for committee agents",
                )
                .scope(CapabilityBundleScope::Global)
                .add_capability(Capability::new("committee.validate", vec!["validate"]))
                .add_capability(Capability::new("committee.sign", vec!["sign"]))
                .add_capability(Capability::new("committee.observe", vec!["observe"]))
                .add_capability(Capability::new("resource.read", vec!["read"]))
                .with_metadata("standard_bundle", "true")
                .with_metadata("agent_type", "committee");
                
                Ok(builder.build())
            },
            StandardBundleType::OperatorBasic => {
                let builder = CapabilityBundleBuilder::new(
                    "Operator Basic",
                    "Basic capabilities for operator agents",
                )
                .scope(CapabilityBundleScope::Global)
                .add_capability(Capability::new("operator.monitor", vec!["monitor"]))
                .add_capability(Capability::new("operator.maintenance", vec!["maintenance"]))
                .add_capability(Capability::new("resource.read", vec!["read"]))
                .add_capability(Capability::new("resource.list", vec!["list"]))
                .with_metadata("standard_bundle", "true")
                .with_metadata("agent_type", "operator");
                
                Ok(builder.build())
            },
            StandardBundleType::Custom(name, capabilities) => {
                let scope = match resource_type {
                    Some(rt) => CapabilityBundleScope::ResourceType(rt),
                    None => CapabilityBundleScope::Global,
                };
                
                let mut builder = CapabilityBundleBuilder::new(
                    name,
                    format!("Custom capability bundle: {}", name),
                )
                .scope(scope)
                .with_metadata("standard_bundle", "false")
                .with_metadata("custom_bundle", "true");
                
                for (id, permissions) in capabilities {
                    builder = builder.add_capability(Capability::new(id, permissions));
                }
                
                Ok(builder.build())
            },
        }
    }
}

impl Default for CapabilityBundleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard capability bundle types
#[derive(Debug, Clone)]
pub enum StandardBundleType {
    /// Read-only access
    ReadOnly,
    
    /// Read and write access
    ReadWrite,
    
    /// Administrative access
    Admin,
    
    /// Basic capabilities for user agents
    UserBasic,
    
    /// Basic capabilities for committee agents
    CommitteeBasic,
    
    /// Basic capabilities for operator agents
    OperatorBasic,
    
    /// Custom bundle with specified capabilities
    Custom(String, Vec<(String, Vec<String>)>),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Create a test agent ID
    fn create_test_agent_id(name: &str) -> AgentId {
        AgentId::from_content_hash(ContentHash::calculate(name.as_bytes()).as_bytes(), AgentType::User)
    }
    
    /// Create a test resource ID
    fn create_test_resource_id(name: &str) -> ResourceId {
        ResourceId::new(ContentHash::calculate(name.as_bytes()))
    }
    
    #[test]
    fn test_capability_bundle_creation() {
        let bundle = CapabilityBundle::new(
            "Test Bundle",
            "A bundle for testing",
            CapabilityBundleScope::Global,
        );
        
        assert_eq!(bundle.name(), "Test Bundle");
        assert_eq!(bundle.description(), "A bundle for testing");
        assert!(matches!(bundle.scope(), CapabilityBundleScope::Global));
        assert_eq!(bundle.capabilities().len(), 0);
    }
    
    #[test]
    fn test_capability_bundle_builder() {
        let bundle = CapabilityBundleBuilder::new(
            "Builder Test",
            "Testing the builder pattern",
        )
        .scope(CapabilityBundleScope::Global)
        .add_capability(Capability::new("read", vec!["read"]))
        .add_capability(Capability::new("write", vec!["write"]))
        .with_metadata("test", "value")
        .build();
        
        assert_eq!(bundle.name(), "Builder Test");
        assert_eq!(bundle.description(), "Testing the builder pattern");
        assert!(matches!(bundle.scope(), CapabilityBundleScope::Global));
        assert_eq!(bundle.capabilities().len(), 2);
        assert_eq!(bundle.metadata().get("test"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_standard_bundles() {
        // Read-only bundle
        let read_only = CapabilityBundleManager::create_standard_bundle(
            StandardBundleType::ReadOnly,
            Some(ResourceType::new("document")),
        ).unwrap();
        
        assert_eq!(read_only.name(), "Read Only");
        assert!(matches!(read_only.scope(), 
                         CapabilityBundleScope::ResourceType(rt) if rt.as_str() == "document"));
        assert_eq!(read_only.capabilities().len(), 3);
        assert!(read_only.capabilities().iter().any(|c| c.id().as_str() == "read"));
        
        // Admin bundle
        let admin = CapabilityBundleManager::create_standard_bundle(
            StandardBundleType::Admin,
            None,
        ).unwrap();
        
        assert_eq!(admin.name(), "Admin");
        assert!(matches!(admin.scope(), CapabilityBundleScope::Global));
        assert_eq!(admin.capabilities().len(), 7);
        assert!(admin.capabilities().iter().any(|c| c.id().as_str() == "admin"));
        assert!(admin.delegation_rules().allow_delegation);
    }
    
    #[test]
    fn test_bundle_manager() {
        let mut manager = CapabilityBundleManager::new();
        
        // Register a standard bundle
        let bundle_id = manager.register_standard_bundle(
            StandardBundleType::ReadWrite,
            None,
        ).unwrap();
        
        // Verify it was registered
        assert!(manager.get_bundle(&bundle_id).is_some());
        
        // Create agents
        let alice = create_test_agent_id("alice");
        let bob = create_test_agent_id("bob");
        
        // Delegate the bundle
        let time = 100;
        let delegation = manager.delegate_bundle(&bundle_id, &alice, &bob, time).unwrap();
        
        assert_eq!(delegation.delegator, alice);
        assert_eq!(delegation.delegatee, bob);
        
        // Check if bob has the bundle
        assert!(manager.has_bundle(&bob, &bundle_id, time));
        
        // Bob should have capabilities from the bundle
        let resource_id = create_test_resource_id("document");
        let resource_type = ResourceType::new("document");
        
        let capabilities = manager.get_agent_capabilities_for_resource(
            &bob,
            &resource_id,
            &resource_type,
            time,
        );
        
        assert!(!capabilities.is_empty());
        assert!(capabilities.iter().any(|c| c.id().as_str() == "read"));
        assert!(capabilities.iter().any(|c| c.id().as_str() == "write"));
        
        // Revoke the delegation
        manager.revoke_delegation(&bundle_id, &alice, &bob).unwrap();
        
        // Bob should no longer have the bundle
        assert!(!manager.has_bundle(&bob, &bundle_id, time));
    }
    
    #[test]
    fn test_bundle_expiration() {
        let mut manager = CapabilityBundleManager::new();
        
        // Create a bundle with an Admin role that has delegation with time limits
        let mut bundle = CapabilityBundleBuilder::new(
            "Expiring Bundle",
            "Bundle that expires",
        )
        .scope(CapabilityBundleScope::Global)
        .add_capability(Capability::new("temp.read", vec!["read"]))
        .build();
        
        // Set delegation rules with time limit
        bundle.delegation_rules = DelegationRules {
            allow_delegation: true,
            time_limit: Some(100), // Expires after 100 time units
            ..Default::default()
        };
        
        manager.register_bundle(bundle).unwrap();
        let bundle_id = CapabilityBundleId::new("Expiring Bundle");
        
        // Create agents
        let admin = create_test_agent_id("admin");
        let user = create_test_agent_id("user");
        
        // Delegate at time 1000
        let time = 1000;
        manager.delegate_bundle(&bundle_id, &admin, &user, time).unwrap();
        
        // At time 1000, user should have the bundle
        assert!(manager.has_bundle(&user, &bundle_id, time));
        
        // At time 1099, user should have the bundle
        assert!(manager.has_bundle(&user, &bundle_id, time + 99));
        
        // At time 1100, user should not have the bundle
        assert!(!manager.has_bundle(&user, &bundle_id, time + 100));
    }
} 