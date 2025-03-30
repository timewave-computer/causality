// Domain capabilities
//
// This module provides domain-specific capability types and functionality
// that integrate with the core capability system.

use std::any::Any;
use std::collections::{HashSet, HashMap};
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

use super::{
    ResourceId, IdentityId, Capability, CapabilityGrants, 
    ResourceGuard, ResourceRegistry, CapabilityError,
    ContentHash, ContentRef, ContentAddressed
};

/// Domain-specific capability that can be supported by domain adapters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainCapabilityType {
    // Transaction capabilities
    SendTransaction,
    SignTransaction,
    BatchTransactions,
    
    // Smart contract capabilities
    DeployContract,
    ExecuteContract,
    QueryContract,
    
    // State capabilities
    ReadState,
    WriteState,
    
    // Cryptographic capabilities
    VerifySignature,
    GenerateProof,
    VerifyProof,
    
    // ZK capabilities
    ZkProve,
    ZkVerify,
    
    // Consensus capabilities
    Stake,
    Validate,
    Vote,
    
    // Governance capabilities
    ProposeUpgrade,
    VoteOnProposal,
    
    // Cross-domain capabilities
    BridgeAssets,
    VerifyBridgeTransaction,
    
    // Custom capability (with name)
    Custom(String)
}

impl DomainCapabilityType {
    /// Convert a domain capability to a string
    pub fn to_string(&self) -> String {
        match self {
            DomainCapabilityType::SendTransaction => "send_transaction".to_string(),
            DomainCapabilityType::SignTransaction => "sign_transaction".to_string(),
            DomainCapabilityType::BatchTransactions => "batch_transactions".to_string(),
            DomainCapabilityType::DeployContract => "deploy_contract".to_string(),
            DomainCapabilityType::ExecuteContract => "execute_contract".to_string(),
            DomainCapabilityType::QueryContract => "query_contract".to_string(),
            DomainCapabilityType::ReadState => "read_state".to_string(),
            DomainCapabilityType::WriteState => "write_state".to_string(),
            DomainCapabilityType::VerifySignature => "verify_signature".to_string(),
            DomainCapabilityType::GenerateProof => "generate_proof".to_string(),
            DomainCapabilityType::VerifyProof => "verify_proof".to_string(),
            DomainCapabilityType::ZkProve => "zk_prove".to_string(),
            DomainCapabilityType::ZkVerify => "zk_verify".to_string(),
            DomainCapabilityType::Stake => "stake".to_string(),
            DomainCapabilityType::Validate => "validate".to_string(),
            DomainCapabilityType::Vote => "vote".to_string(),
            DomainCapabilityType::ProposeUpgrade => "propose_upgrade".to_string(),
            DomainCapabilityType::VoteOnProposal => "vote_on_proposal".to_string(),
            DomainCapabilityType::BridgeAssets => "bridge_assets".to_string(),
            DomainCapabilityType::VerifyBridgeTransaction => "verify_bridge_transaction".to_string(),
            DomainCapabilityType::Custom(name) => format!("custom_{}", name),
        }
    }
    
    /// Create a capability from a domain capability type
    pub fn create_capability(&self, grants: CapabilityGrants, owner: IdentityId) -> DomainCapability {
        let id = self.create_resource_id();
        
        DomainCapability {
            capability_type: self.clone(),
            grants,
            id,
            origin: Some(owner),
            content_hash: None,
        }
    }
    
    /// Create a resource ID for a domain capability
    fn create_resource_id(&self) -> ResourceId {
        let capability_str = self.to_string();
        let id_str = format!("domain_{}", capability_str);
        ResourceId::new(content_addressing::hash_string(&id_str))
    }
}

/// A domain-specific capability
#[derive(Debug, Clone)]
pub struct DomainCapability {
    /// The domain capability type
    pub capability_type: DomainCapabilityType,
    
    /// The capability grants
    pub grants: CapabilityGrants,
    
    /// The identifier for the capability
    pub id: ResourceId,
    
    /// The origin identity that created the capability
    pub origin: Option<IdentityId>,
    
    /// The content hash if content-addressed
    pub content_hash: Option<ContentHash>,
}

impl DomainCapability {
    /// Convert to a standard capability
    pub fn to_capability<T: Send + Sync + 'static>(&self) -> Capability<T> {
        Capability {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Create a new domain capability
    pub fn new(
        capability_type: DomainCapabilityType,
        grants: CapabilityGrants,
        owner: IdentityId
    ) -> Self {
        capability_type.create_capability(grants, owner)
    }
    
    /// Create a content-addressed version of this capability
    pub fn to_content_addressed(&self, content_hash: ContentHash) -> Self {
        let mut result = self.clone();
        result.content_hash = Some(content_hash);
        result
    }
    
    /// Get the content hash if this is content-addressed
    pub fn content_hash(&self) -> Option<&ContentHash> {
        self.content_hash.as_ref()
    }
    
    /// Check if this capability is content-addressed
    pub fn is_content_addressed(&self) -> bool {
        self.content_hash.is_some()
    }
    
    /// Check if this domain capability applies to the given content reference
    pub fn applies_to<T>(
        &self,
        content_ref: &ContentRef<T>,
    ) -> bool {
        // Implementation of applies_to method
        false
    }
}

/// Error type for domain capability operations
#[derive(Error, Debug)]
pub enum DomainCapabilityError {
    #[error("Invalid capability type: {0}")]
    InvalidCapabilityType(String),
    
    #[error("Missing required grants")]
    MissingGrants,
    
    #[error("Underlying capability error: {0}")]
    CapabilityError(#[from] CapabilityError),
    
    #[error("Content addressing error: {0}")]
    ContentAddressingError(String),
}

/// Domain registry with enhanced capability-based domain management
pub struct DomainCapabilityRegistry {
    /// The underlying resource registry
    registry: ResourceRegistry,
    
    /// Default domain capabilities by domain type
    domain_types: HashMap<String, HashSet<DomainCapabilityType>>,
}

impl DomainCapabilityRegistry {
    /// Create a new domain capability registry
    pub fn new() -> Self {
        Self {
            registry: ResourceRegistry::new(),
            domain_types: HashMap::new(),
        }
    }
    
    /// Register domain type with default capabilities
    pub fn register_domain_type(&mut self, domain_type: &str, capabilities: HashSet<DomainCapabilityType>) {
        self.domain_types.insert(domain_type.to_string(), capabilities);
    }
    
    /// Get the default capabilities for a domain type
    pub fn get_domain_capabilities(&self, domain_type: &str) -> Option<&HashSet<DomainCapabilityType>> {
        self.domain_types.get(&domain_type.to_string())
    }
    
    /// Register a resource and get a domain capability
    pub fn register<T: Send + Sync + 'static>(
        &self,
        resource: T,
        owner: IdentityId,
        capability_type: DomainCapabilityType,
    ) -> Result<DomainCapability, CapabilityError> {
        // Register in the core registry with full rights
        let capability = self.registry.register(resource, owner.clone())?;
        
        // Create a domain capability with the specified type
        let domain_capability = DomainCapability {
            capability_type,
            grants: capability.grants,
            id: capability.id,
            origin: capability.origin,
            content_hash: None,
        };
        
        Ok(domain_capability)
    }
    
    /// Access a resource using a domain capability
    pub fn access<T: Send + Sync + 'static>(
        &self,
        capability: &DomainCapability,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        // Create a standard capability
        let std_capability = capability.to_capability::<T>();
        
        // Access with the standard capability
        self.registry.access(&std_capability)
    }
    
    /// Access a resource by content reference
    pub fn access_by_content<T: Send + Sync + 'static>(
        &self,
        content_ref: &ContentRef<T>,
    ) -> Result<ResourceGuard<T>, CapabilityError> {
        self.registry.access_by_content(content_ref)
    }
    
    /// Transfer a capability to another identity
    pub fn transfer_capability(
        &self,
        capability: &DomainCapability,
        from: &IdentityId,
        to: &IdentityId,
    ) -> Result<(), CapabilityError> {
        let std_capability = Capability {
            id: capability.id.clone(),
            grants: capability.grants.clone(),
            origin: capability.origin.clone(),
            _phantom: std::marker::PhantomData::<dyn Any + Send + Sync>,
        };
        
        self.registry.transfer_capability(&std_capability, from, to)
    }
}

/// Helper functions for working with domain capabilities
pub mod helpers {
    use super::*;
    use std::collections::HashMap;
    
    /// Create a new domain registry
    pub fn create_domain_registry() -> DomainCapabilityRegistry {
        DomainCapabilityRegistry::new()
    }
    
    /// Create a domain registry with common domain types
    pub fn create_domain_registry_with_defaults() -> DomainCapabilityRegistry {
        let mut registry = DomainCapabilityRegistry::new();
        
        // EVM domain type
        let mut evm_capabilities = HashSet::new();
        evm_capabilities.insert(DomainCapabilityType::SendTransaction);
        evm_capabilities.insert(DomainCapabilityType::SignTransaction);
        evm_capabilities.insert(DomainCapabilityType::DeployContract);
        evm_capabilities.insert(DomainCapabilityType::ExecuteContract);
        evm_capabilities.insert(DomainCapabilityType::ReadState);
        registry.register_domain_type("evm", evm_capabilities);
        
        // CosmWasm domain type
        let mut cosmwasm_capabilities = HashSet::new();
        cosmwasm_capabilities.insert(DomainCapabilityType::SendTransaction);
        cosmwasm_capabilities.insert(DomainCapabilityType::DeployContract);
        cosmwasm_capabilities.insert(DomainCapabilityType::ExecuteContract);
        cosmwasm_capabilities.insert(DomainCapabilityType::QueryContract);
        cosmwasm_capabilities.insert(DomainCapabilityType::ReadState);
        cosmwasm_capabilities.insert(DomainCapabilityType::Stake);
        cosmwasm_capabilities.insert(DomainCapabilityType::Vote);
        registry.register_domain_type("cosmwasm", cosmwasm_capabilities);
        
        // TEL domain type
        let mut tel_capabilities = HashSet::new();
        tel_capabilities.insert(DomainCapabilityType::ExecuteContract);
        tel_capabilities.insert(DomainCapabilityType::QueryContract);
        tel_capabilities.insert(DomainCapabilityType::ZkProve);
        tel_capabilities.insert(DomainCapabilityType::ZkVerify);
        registry.register_domain_type("tel", tel_capabilities);
        
        registry
    }
    
    /// Create common domain capabilities
    pub fn create_read_state_capability(owner: IdentityId) -> DomainCapability {
        DomainCapability::new(
            DomainCapabilityType::ReadState,
            CapabilityGrants::read_only(),
            owner,
        )
    }
    
    /// Create transaction capability
    pub fn create_transaction_capability(owner: IdentityId) -> DomainCapability {
        DomainCapability::new(
            DomainCapabilityType::SendTransaction,
            CapabilityGrants::full(),  // Transactions need full rights
            owner,
        )
    }
    
    /// Create contract execution capability
    pub fn create_contract_execution_capability(owner: IdentityId) -> DomainCapability {
        DomainCapability::new(
            DomainCapabilityType::ExecuteContract,
            CapabilityGrants::new(true, true, false),  // Read and write, but not delegate
            owner,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_capability_types() {
        let send_tx = DomainCapabilityType::SendTransaction;
        let execute = DomainCapabilityType::ExecuteContract;
        let custom = DomainCapabilityType::Custom("test".to_string());
        
        // Test to_string
        assert_eq!(send_tx.to_string(), "send_transaction");
        assert_eq!(execute.to_string(), "execute_contract");
        assert_eq!(custom.to_string(), "custom_test");
    }
    
    #[test]
    fn test_domain_capability_registry() {
        // Create a registry
        let registry = DomainCapabilityRegistry::new();
        
        // Create an identity
        let alice = IdentityId::new();
        
        // Create a test resource
        let test_data = "Domain test data".to_string();
        
        // Register the resource
        let capability = registry.register(
            test_data,
            alice.clone(),
            DomainCapabilityType::ReadState,
        ).unwrap();
        
        // Verify capability type
        assert_eq!(
            capability.capability_type,
            DomainCapabilityType::ReadState
        );
        
        // Access the resource
        let guard = registry.access::<String>(&capability).unwrap();
        let data = guard.read().unwrap();
        assert_eq!(*data, "Domain test data".to_string());
    }
    
    #[test]
    fn test_domain_capability_helpers() {
        // Create an identity
        let alice = IdentityId::new();
        
        // Test read state capability
        let read_cap = helpers::create_read_state_capability(alice.clone());
        assert_eq!(read_cap.capability_type, DomainCapabilityType::ReadState);
        assert_eq!(read_cap.grants, CapabilityGrants::read_only());
        
        // Test transaction capability
        let tx_cap = helpers::create_transaction_capability(alice.clone());
        assert_eq!(tx_cap.capability_type, DomainCapabilityType::SendTransaction);
        assert_eq!(tx_cap.grants, CapabilityGrants::full());
        
        // Test registry with defaults
        let registry = helpers::create_domain_registry_with_defaults();
        let evm_caps = registry.get_domain_capabilities("evm").unwrap();
        assert!(evm_caps.contains(&DomainCapabilityType::SendTransaction));
        assert!(evm_caps.contains(&DomainCapabilityType::DeployContract));
        
        let tel_caps = registry.get_domain_capabilities("tel").unwrap();
        assert!(tel_caps.contains(&DomainCapabilityType::ZkProve));
        assert!(tel_caps.contains(&DomainCapabilityType::ZkVerify));
    }
} 