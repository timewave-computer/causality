//! Domain definitions for the Causality framework.
//!
//! This module defines domain boundaries within the Causality system:
//! - Domains represent logical boundaries for resources, effects, and constraints
//! - They povide isolation and governance while enabling controlled cross-domain interactions
//! - Extended with SMT integration for TEG-aware operations
//!
//! Domains are the foundation for resource governance and boundaries.

use crate::primitive::ids::DomainId;
use crate::primitive::string::Str;

//-----------------------------------------------------------------------------
// Domain Type Definition
//-----------------------------------------------------------------------------

/// A domain represents a logical boundary for resources and effects.
///
/// Domains act as a namespace and authority for managing related components.
/// They provide isolation and governance for resources while enabling
/// cross-domain interactions and computation. Extended with SMT integration
/// for TEG-aware operations and direct state management.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Domain {
    /// Unique identifier for the domain.
    pub id: DomainId,

    /// Human-readable name for the domain.
    pub name: String,

    /// Optional description of the domain's purpose.
    pub description: Option<String>,

    /// SMT state root for this domain's TEG data
    /// 
    /// This tracks the current state root of all TEG nodes, effects,
    /// resources, and constraints within this domain. Used for
    /// content-addressable storage and state verification.
    pub smt_state_root: Option<[u8; 32]>,

    /// Domain configuration flags for TEG operations
    pub config: DomainConfig,
}

//-----------------------------------------------------------------------------
// Domain Configuration
//-----------------------------------------------------------------------------

/// Configuration settings for domain behavior and TEG operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainConfig {
    /// Enable direct writes to SMT (bypasses validation)
    pub enable_direct_writes: bool,
    
    /// Maximum nodes per TEG transaction in this domain
    pub max_nodes_per_transaction: u32,
    
    /// Enable temporal constraint validation
    pub validate_temporal_constraints: bool,
    
    /// Enable cross-domain references from this domain
    pub allow_cross_domain_refs: bool,
    
    /// Maximum depth for cross-domain TEG traversal
    pub max_cross_domain_depth: u8,
    
    /// Enable content-addressable TEG node storage
    pub content_addressable_nodes: bool,
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            enable_direct_writes: true,
            max_nodes_per_transaction: 1000,
            validate_temporal_constraints: true,
            allow_cross_domain_refs: true,
            max_cross_domain_depth: 10,
            content_addressable_nodes: true,
        }
    }
}

//-----------------------------------------------------------------------------
// Domain Trait
//-----------------------------------------------------------------------------

/// Trait for types that can be represented as domains with SMT integration
pub trait AsDomain {
    /// Get the domain ID for this entity.
    fn domain_id(&self) -> Option<DomainId>;

    /// Get the trusted root for this domain
    fn get_trusted_root(&self) -> Str;
    
    /// Get the current SMT state root for this domain
    /// Returns None if the domain doesn't have SMT state or is not yet initialized
    fn get_smt_state_root(&self) -> Option<[u8; 32]> {
        None
    }
    
    /// Check if this domain supports direct SMT writes for TEG data
    fn supports_direct_writes(&self) -> bool {
        false
    }
    
    /// Check if this domain supports cross-domain TEG references
    fn supports_cross_domain_refs(&self) -> bool {
        true
    }
    
    /// Get the maximum number of nodes allowed per TEG transaction
    fn max_nodes_per_transaction(&self) -> u32 {
        1000
    }
    
    /// Check if temporal constraint validation is enabled
    fn validates_temporal_constraints(&self) -> bool {
        true
    }
}

/// Extended trait for domains that provide direct SMT integration
pub trait AsSmtDomain: AsDomain {
    /// Generate SMT key for TEG node storage in this domain
    fn generate_teg_node_key(&self, node_type: &str, node_id: &[u8]) -> String {
        format!("teg-{}-{}", node_type, hex::encode(node_id))
    }
    
    /// Generate SMT key for TEG effect storage in this domain
    fn generate_teg_effect_key(&self, effect_id: &[u8]) -> String {
        format!("teg-effect-{}", hex::encode(effect_id))
    }
    
    /// Generate SMT key for TEG resource storage in this domain
    fn generate_teg_resource_key(&self, resource_id: &[u8]) -> String {
        format!("teg-resource-{}", hex::encode(resource_id))
    }
    
    /// Generate SMT key for TEG intent storage in this domain
    fn generate_teg_intent_key(&self, intent_id: &[u8]) -> String {
        format!("teg-intent-{}", hex::encode(intent_id))
    }
    
    /// Generate SMT key for TEG constraint storage in this domain
    fn generate_teg_constraint_key(&self, constraint_id: &[u8]) -> String {
        format!("teg-constraint-{}", hex::encode(constraint_id))
    }
    
    /// Generate SMT key for cross-domain reference storage
    fn generate_cross_domain_ref_key(&self, target_domain: &DomainId, target_id: &[u8]) -> String {
        format!("cross-domain-{}-{}", hex::encode(target_domain.0), hex::encode(target_id))
    }
}

//-----------------------------------------------------------------------------
// Cross-Domain References
//-----------------------------------------------------------------------------

/// A domain-qualified reference to content in a specific domain
///
/// This type allows for precise cross-domain references by pairing
/// a domain ID with a content ID, enabling safe cross-domain operations
/// while maintaining appropriate boundaries.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct DomainQualifiedId<T> {
    /// Domain where the content exists
    pub domain: DomainId,

    /// Content-addressed ID
    pub id: T,
}

impl<T: Default> Default for DomainQualifiedId<T> {
    fn default() -> Self {
        Self {
            domain: DomainId::default(),
            id: T::default(),
        }
    }
}

//-----------------------------------------------------------------------------
// Domain Trait Implementations
//-----------------------------------------------------------------------------

impl AsDomain for Domain {
    fn domain_id(&self) -> Option<DomainId> {
        Some(self.id)
    }
    
    fn get_trusted_root(&self) -> Str {
        // For now, use the domain name as the trusted root
        // In a full implementation, this would be cryptographically derived
        Str::from_string(self.name.clone())
    }
    
    fn get_smt_state_root(&self) -> Option<[u8; 32]> {
        self.smt_state_root
    }
    
    fn supports_direct_writes(&self) -> bool {
        self.config.enable_direct_writes
    }
    
    fn supports_cross_domain_refs(&self) -> bool {
        self.config.allow_cross_domain_refs
    }
    
    fn max_nodes_per_transaction(&self) -> u32 {
        self.config.max_nodes_per_transaction
    }
    
    fn validates_temporal_constraints(&self) -> bool {
        self.config.validate_temporal_constraints
    }
}

impl AsSmtDomain for Domain {}

impl Domain {
    /// Create a new domain with SMT integration enabled
    pub fn new_with_smt(id: DomainId, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            smt_state_root: None,
            config: DomainConfig::default(),
        }
    }
    
    /// Create a new domain with custom configuration
    pub fn new_with_config(id: DomainId, name: String, config: DomainConfig) -> Self {
        Self {
            id,
            name,
            description: None,
            smt_state_root: None,
            config,
        }
    }
    
    /// Update the SMT state root for this domain
    pub fn update_smt_state_root(&mut self, new_root: [u8; 32]) {
        self.smt_state_root = Some(new_root);
    }
    
    /// Clear the SMT state root (useful for testing or reset scenarios)
    pub fn clear_smt_state_root(&mut self) {
        self.smt_state_root = None;
    }
    
    /// Check if this domain has been initialized with SMT state
    pub fn has_smt_state(&self) -> bool {
        self.smt_state_root.is_some()
    }
    
    /// Update domain configuration
    pub fn update_config(&mut self, config: DomainConfig) {
        self.config = config;
    }
    
    /// Enable or disable direct writes
    pub fn set_direct_writes(&mut self, enabled: bool) {
        self.config.enable_direct_writes = enabled;
    }
    
    /// Enable or disable cross-domain references
    pub fn set_cross_domain_refs(&mut self, enabled: bool) {
        self.config.allow_cross_domain_refs = enabled;
    }
    
    /// Set the maximum nodes per transaction
    pub fn set_max_nodes_per_transaction(&mut self, max_nodes: u32) {
        self.config.max_nodes_per_transaction = max_nodes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_creation_with_smt() {
        let domain_id = DomainId::default();
        let domain = Domain::new_with_smt(domain_id, "test-domain".to_string());
        
        assert_eq!(domain.id, domain_id);
        assert_eq!(domain.name, "test-domain");
        assert!(domain.supports_direct_writes());
        assert!(domain.supports_cross_domain_refs());
        assert_eq!(domain.max_nodes_per_transaction(), 1000);
        assert!(!domain.has_smt_state());
    }
    
    #[test]
    fn test_domain_smt_state_management() {
        let domain_id = DomainId::default();
        let mut domain = Domain::new_with_smt(domain_id, "test-domain".to_string());
        
        // Initially no SMT state
        assert!(!domain.has_smt_state());
        assert_eq!(domain.get_smt_state_root(), None);
        
        // Update SMT state root
        let test_root = [42u8; 32];
        domain.update_smt_state_root(test_root);
        
        assert!(domain.has_smt_state());
        assert_eq!(domain.get_smt_state_root(), Some(test_root));
        
        // Clear SMT state
        domain.clear_smt_state_root();
        assert!(!domain.has_smt_state());
    }
    
    #[test]
    fn test_domain_config_updates() {
        let domain_id = DomainId::default();
        let mut domain = Domain::new_with_smt(domain_id, "test-domain".to_string());
        
        // Test individual config updates
        domain.set_direct_writes(false);
        assert!(!domain.supports_direct_writes());
        
        domain.set_cross_domain_refs(false);
        assert!(!domain.supports_cross_domain_refs());
        
        domain.set_max_nodes_per_transaction(500);
        assert_eq!(domain.max_nodes_per_transaction(), 500);
    }
    
    #[test]
    fn test_smt_key_generation() {
        let domain_id = DomainId::default();
        let domain = Domain::new_with_smt(domain_id, "test-domain".to_string());
        
        let test_id = [1, 2, 3, 4];
        
        // Test different key types
        let node_key = domain.generate_teg_node_key("effect", &test_id);
        assert_eq!(node_key, "teg-effect-01020304");
        
        let effect_key = domain.generate_teg_effect_key(&test_id);
        assert_eq!(effect_key, "teg-effect-01020304");
        
        let resource_key = domain.generate_teg_resource_key(&test_id);
        assert_eq!(resource_key, "teg-resource-01020304");
        
        let intent_key = domain.generate_teg_intent_key(&test_id);
        assert_eq!(intent_key, "teg-intent-01020304");
        
        let constraint_key = domain.generate_teg_constraint_key(&test_id);
        assert_eq!(constraint_key, "teg-constraint-01020304");
    }
    
    #[test]
    fn test_cross_domain_ref_key_generation() {
        let domain_id = DomainId::default();
        let domain = Domain::new_with_smt(domain_id, "test-domain".to_string());
        
        let mut target_domain_bytes = [0u8; 32];
        target_domain_bytes[0] = 5;
        target_domain_bytes[1] = 6;
        target_domain_bytes[2] = 7;
        target_domain_bytes[3] = 8;
        let target_domain = DomainId(target_domain_bytes);
        let target_id = [1, 2, 3, 4];
        
        let cross_ref_key = domain.generate_cross_domain_ref_key(&target_domain, &target_id);
        assert!(cross_ref_key.starts_with("cross-domain-"));
        assert!(cross_ref_key.contains("05060708"));
        assert!(cross_ref_key.contains("01020304"));
    }
    
    #[test]
    fn test_domain_config_defaults() {
        let config = DomainConfig::default();
        
        assert!(config.enable_direct_writes);
        assert_eq!(config.max_nodes_per_transaction, 1000);
        assert!(config.validate_temporal_constraints);
        assert!(config.allow_cross_domain_refs);
        assert_eq!(config.max_cross_domain_depth, 10);
        assert!(config.content_addressable_nodes);
    }
}
