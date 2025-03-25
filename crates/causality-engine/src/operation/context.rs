// Operation context for state management
// Original file: src/operation/context.rs

// Operation Context Module
//
// This module defines the execution contexts for operations,
// allowing operations to be parameterized by their execution environment.

use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use causality_types::DomainId;
use causality_patterns::Capability;

/// Trait for operation execution contexts
pub trait ExecutionContext: Clone + Debug + Serialize + Deserialize + Send + Sync + 'static {
    /// The environment this context operates in
    fn environment(&self) -> ExecutionEnvironment;
    
    /// The domain this context is associated with (if any)
    fn domain(&self) -> Option<DomainId>;
    
    /// The execution phase this context represents
    fn phase(&self) -> ExecutionPhase;
    
    /// Whether this context requires a ZK proof
    fn requires_proof(&self) -> bool;
    
    /// Get capability requirements for this context
    fn capability_requirements(&self) -> Vec<Capability>;
    
    /// Get the default environment for this context type
    fn default_environment() -> ExecutionEnvironment where Self: Sized;
    
    /// Create this context from a previous context
    fn from_previous_context(previous: &dyn ExecutionContext) -> Result<Self> where Self: Sized;
}

/// Execution phases for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionPhase {
    /// Planning phase (intent formation)
    Planning,
    
    /// Validation phase (checking preconditions)
    Validation,
    
    /// Authorization phase (verifying permissions)
    Authorization,
    
    /// Execution phase (applying changes)
    Execution,
    
    /// Verification phase (confirming effects)
    Verification,
    
    /// Finalization phase (recording outcomes)
    Finalization,
}

/// Execution environments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    /// Abstract environment (logical operations)
    Abstract,
    
    /// Program execution environment
    Program,
    
    /// Register-based environment
    Register,
    
    /// Physical on-chain environment
    OnChain(DomainId),
    
    /// ZK verification environment
    ZkVm,
}

/// Abstract execution context for logical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractContext {
    /// The current execution phase
    pub phase: ExecutionPhase,
    
    /// Whether a proof is required for this operation
    pub proof_required: bool,
    
    /// Required capabilities
    pub required_capabilities: Vec<Capability>,
}

impl AbstractContext {
    /// Create a new abstract context
    pub fn new(phase: ExecutionPhase) -> Self {
        Self {
            phase,
            proof_required: false,
            required_capabilities: Vec::new(),
        }
    }
    
    /// Set proof requirement
    pub fn with_proof_required(mut self, required: bool) -> Self {
        self.proof_required = required;
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

impl ExecutionContext for AbstractContext {
    fn environment(&self) -> ExecutionEnvironment {
        ExecutionEnvironment::Abstract
    }
    
    fn domain(&self) -> Option<DomainId> {
        None
    }
    
    fn phase(&self) -> ExecutionPhase {
        self.phase.clone()
    }
    
    fn requires_proof(&self) -> bool {
        self.proof_required
    }
    
    fn capability_requirements(&self) -> Vec<Capability> {
        self.required_capabilities.clone()
    }
    
    fn default_environment() -> ExecutionEnvironment {
        ExecutionEnvironment::Abstract
    }
    
    fn from_previous_context(previous: &dyn ExecutionContext) -> Result<Self> {
        Ok(AbstractContext {
            phase: previous.phase(),
            proof_required: previous.requires_proof(),
            required_capabilities: previous.capability_requirements(),
        })
    }
}

/// Register-based execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterContext {
    /// The current execution phase
    pub phase: ExecutionPhase,
    
    /// Whether a proof is required for this operation
    pub proof_required: bool,
    
    /// Required capabilities
    pub required_capabilities: Vec<Capability>,
    
    /// Register namespace
    pub namespace: String,
}

impl RegisterContext {
    /// Create a new register context
    pub fn new(phase: ExecutionPhase, namespace: &str) -> Self {
        Self {
            phase,
            proof_required: false,
            required_capabilities: Vec::new(),
            namespace: namespace.to_string(),
        }
    }
    
    /// Set proof requirement
    pub fn with_proof_required(mut self, required: bool) -> Self {
        self.proof_required = required;
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

impl ExecutionContext for RegisterContext {
    fn environment(&self) -> ExecutionEnvironment {
        ExecutionEnvironment::Register
    }
    
    fn domain(&self) -> Option<DomainId> {
        None
    }
    
    fn phase(&self) -> ExecutionPhase {
        self.phase.clone()
    }
    
    fn requires_proof(&self) -> bool {
        self.proof_required
    }
    
    fn capability_requirements(&self) -> Vec<Capability> {
        self.required_capabilities.clone()
    }
    
    fn default_environment() -> ExecutionEnvironment {
        ExecutionEnvironment::Register
    }
    
    fn from_previous_context(previous: &dyn ExecutionContext) -> Result<Self> {
        Ok(RegisterContext {
            phase: previous.phase(),
            proof_required: previous.requires_proof(),
            required_capabilities: previous.capability_requirements(),
            namespace: "default".to_string(), // Default namespace
        })
    }
}

/// Physical on-chain execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalContext {
    /// The current execution phase
    pub phase: ExecutionPhase,
    
    /// The domain ID
    pub domain_id: DomainId,
    
    /// Whether a proof is required for this operation
    pub proof_required: bool,
    
    /// Required capabilities
    pub required_capabilities: Vec<Capability>,
}

impl PhysicalContext {
    /// Create a new physical context
    pub fn new(phase: ExecutionPhase, domain_id: DomainId) -> Self {
        Self {
            phase,
            domain_id,
            proof_required: false,
            required_capabilities: Vec::new(),
        }
    }
    
    /// Set proof requirement
    pub fn with_proof_required(mut self, required: bool) -> Self {
        self.proof_required = required;
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

impl ExecutionContext for PhysicalContext {
    fn environment(&self) -> ExecutionEnvironment {
        ExecutionEnvironment::OnChain(self.domain_id.clone())
    }
    
    fn domain(&self) -> Option<DomainId> {
        Some(self.domain_id.clone())
    }
    
    fn phase(&self) -> ExecutionPhase {
        self.phase.clone()
    }
    
    fn requires_proof(&self) -> bool {
        self.proof_required
    }
    
    fn capability_requirements(&self) -> Vec<Capability> {
        self.required_capabilities.clone()
    }
    
    fn default_environment() -> ExecutionEnvironment {
        // We need a placeholder domain ID for the default environment
        ExecutionEnvironment::OnChain(DomainId::from("default"))
    }
    
    fn from_previous_context(previous: &dyn ExecutionContext) -> Result<Self> {
        // For physical context, we need a domain ID
        let domain_id = if let Some(domain) = previous.domain() {
            domain
        } else {
            return Err(Error::InvalidArgument("Domain ID required for physical context".to_string()));
        };
        
        Ok(PhysicalContext {
            phase: previous.phase(),
            domain_id,
            proof_required: previous.requires_proof(),
            required_capabilities: previous.capability_requirements(),
        })
    }
}

/// ZK verification context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkContext {
    /// The current execution phase
    pub phase: ExecutionPhase,
    
    /// The domain ID (if any)
    pub domain_id: Option<DomainId>,
    
    /// Required capabilities
    pub required_capabilities: Vec<Capability>,
    
    /// Circuit identifier
    pub circuit_id: String,
}

impl ZkContext {
    /// Create a new ZK context
    pub fn new(phase: ExecutionPhase, circuit_id: &str) -> Self {
        Self {
            phase,
            domain_id: None,
            required_capabilities: Vec::new(),
            circuit_id: circuit_id.to_string(),
        }
    }
    
    /// Set domain ID
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

impl ExecutionContext for ZkContext {
    fn environment(&self) -> ExecutionEnvironment {
        ExecutionEnvironment::ZkVm
    }
    
    fn domain(&self) -> Option<DomainId> {
        self.domain_id.clone()
    }
    
    fn phase(&self) -> ExecutionPhase {
        self.phase.clone()
    }
    
    fn requires_proof(&self) -> bool {
        true // ZK context always requires a proof
    }
    
    fn capability_requirements(&self) -> Vec<Capability> {
        self.required_capabilities.clone()
    }
    
    fn default_environment() -> ExecutionEnvironment {
        ExecutionEnvironment::ZkVm
    }
    
    fn from_previous_context(previous: &dyn ExecutionContext) -> Result<Self> {
        Ok(ZkContext {
            phase: previous.phase(),
            domain_id: previous.domain(),
            required_capabilities: previous.capability_requirements(),
            circuit_id: "default".to_string(), // Default circuit ID
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_patterns::{Right, CapabilityType};
    
    #[test]
    fn test_abstract_context() {
        let capability = Capability::new(CapabilityType::Resource, "resource_id", Right::Read);
        
        let context = AbstractContext::new(ExecutionPhase::Planning)
            .with_proof_required(true)
            .with_capability(capability.clone());
        
        assert_eq!(context.environment(), ExecutionEnvironment::Abstract);
        assert_eq!(context.phase(), ExecutionPhase::Planning);
        assert_eq!(context.domain(), None);
        assert!(context.requires_proof());
        assert_eq!(context.capability_requirements().len(), 1);
    }
    
    #[test]
    fn test_register_context() {
        let capability = Capability::new(CapabilityType::Resource, "resource_id", Right::Read);
        
        let context = RegisterContext::new(ExecutionPhase::Execution, "test_namespace")
            .with_proof_required(true)
            .with_capability(capability.clone());
        
        assert_eq!(context.environment(), ExecutionEnvironment::Register);
        assert_eq!(context.phase(), ExecutionPhase::Execution);
        assert_eq!(context.domain(), None);
        assert!(context.requires_proof());
        assert_eq!(context.capability_requirements().len(), 1);
    }
    
    #[test]
    fn test_physical_context() {
        let domain_id = DomainId::new("test_domain");
        let capability = Capability::new(CapabilityType::Resource, "resource_id", Right::Read);
        
        let context = PhysicalContext::new(ExecutionPhase::Execution, domain_id.clone())
            .with_proof_required(true)
            .with_capability(capability.clone());
        
        assert_eq!(context.environment(), ExecutionEnvironment::OnChain(domain_id.clone()));
        assert_eq!(context.phase(), ExecutionPhase::Execution);
        assert_eq!(context.domain(), Some(domain_id));
        assert!(context.requires_proof());
        assert_eq!(context.capability_requirements().len(), 1);
    }
} 