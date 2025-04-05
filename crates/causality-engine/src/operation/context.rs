// Operation context for state management
// Original file: src/operation/context.rs

// Operation Context Module
//
// This module defines the execution contexts for operations,
// allowing operations to be parameterized by their execution environment.

use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use causality_error::{EngineResult, EngineError};
use causality_types::DomainId;
use causality_core::effect::context::Capability;

use crate::operation::types::Context;

/// Trait for execution contexts
pub trait ExecutionContext: Debug + Send + Sync + 'static {
    /// Get the environment for this context
    fn environment(&self) -> ExecutionEnvironment;
    
    /// Get the domain for this context, if any
    fn domain(&self) -> Option<DomainId>;
    
    /// Get the execution phase
    fn phase(&self) -> ExecutionPhase;
    
    /// Check if proof is required for this context
    fn proof_required(&self) -> bool;
    
    /// Get capabilities required for this context
    fn required_capabilities(&self) -> Vec<String>;
    
    /// Create a context from a previous context
    fn from_previous_context(previous: &Context) -> EngineResult<Self> where Self: Sized;
    
    /// Clone the context into a boxed trait object
    fn clone_context(&self) -> Box<dyn ExecutionContext>;
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
    
    /// Create a new abstract context from a previous context
    pub fn from_previous_context(previous: &Context) -> EngineResult<Self> {
        match previous {
            Context::Abstract(ctx) => Ok(ctx.clone()),
            Context::Register(ctx) => Ok(AbstractContext {
                phase: ctx.phase(),
                proof_required: ctx.proof_required(),
                required_capabilities: ctx.required_capabilities().iter().map(|s| s.parse().unwrap()).collect(),
            }),
            Context::Physical(ctx) => Ok(AbstractContext {
                phase: ctx.phase(),
                proof_required: ctx.proof_required(),
                required_capabilities: ctx.required_capabilities().iter().map(|s| s.parse().unwrap()).collect(),
            }),
            Context::Zk(ctx) => Ok(AbstractContext {
                phase: ctx.phase(),
                proof_required: true,
                required_capabilities: ctx.required_capabilities().iter().map(|s| s.parse().unwrap()).collect(),
            }),
        }
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
    
    fn proof_required(&self) -> bool {
        self.proof_required
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        self.required_capabilities.iter().map(|c| c.to_string()).collect()
    }
    
    fn from_previous_context(previous: &Context) -> EngineResult<Self> {
        Self::from_previous_context(previous)
    }
    
    fn clone_context(&self) -> Box<dyn ExecutionContext> {
        Box::new(self.clone())
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
    
    fn proof_required(&self) -> bool {
        self.proof_required
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        self.required_capabilities.iter().map(|c| c.to_string()).collect()
    }
    
    fn from_previous_context(previous: &Context) -> EngineResult<Self> {
        match previous {
            Context::Register(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected RegisterContext".to_string())),
        }
    }
    
    fn clone_context(&self) -> Box<dyn ExecutionContext> {
        Box::new(self.clone())
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
    
    fn proof_required(&self) -> bool {
        self.proof_required
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        self.required_capabilities.iter().map(|c| c.to_string()).collect()
    }
    
    fn from_previous_context(previous: &Context) -> EngineResult<Self> {
        match previous {
            Context::Physical(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected PhysicalContext".to_string())),
        }
    }
    
    fn clone_context(&self) -> Box<dyn ExecutionContext> {
        Box::new(self.clone())
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
    
    fn proof_required(&self) -> bool {
        true // ZK context always requires a proof
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        self.required_capabilities.iter().map(|c| c.to_string()).collect()
    }
    
    fn from_previous_context(previous: &Context) -> EngineResult<Self> {
        match previous {
            Context::Zk(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected ZkContext".to_string())),
        }
    }
    
    fn clone_context(&self) -> Box<dyn ExecutionContext> {
        Box::new(self.clone())
    }
}

/// Extension trait for context conversion
pub trait ContextConversion: ExecutionContext {
    /// Convert this context to a Context enum
    fn to_context(&self) -> Context;
    
    /// Create this context from a Context enum
    fn from_context(context: &Context) -> EngineResult<Self> where Self: Sized;
}

// Implement ContextConversion for AbstractContext
impl ContextConversion for AbstractContext {
    fn to_context(&self) -> Context {
        Context::Abstract(self.clone())
    }
    
    fn from_context(context: &Context) -> EngineResult<Self> {
        match context {
            Context::Abstract(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected AbstractContext".to_string())),
        }
    }
}

// Implement ContextConversion for RegisterContext
impl ContextConversion for RegisterContext {
    fn to_context(&self) -> Context {
        Context::Register(self.clone())
    }
    
    fn from_context(context: &Context) -> EngineResult<Self> {
        match context {
            Context::Register(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected RegisterContext".to_string())),
        }
    }
}

// Implement ContextConversion for PhysicalContext
impl ContextConversion for PhysicalContext {
    fn to_context(&self) -> Context {
        Context::Physical(self.clone())
    }
    
    fn from_context(context: &Context) -> EngineResult<Self> {
        match context {
            Context::Physical(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected PhysicalContext".to_string())),
        }
    }
}

// Implement ContextConversion for ZkContext
impl ContextConversion for ZkContext {
    fn to_context(&self) -> Context {
        Context::Zk(self.clone())
    }
    
    fn from_context(context: &Context) -> EngineResult<Self> {
        match context {
            Context::Zk(ctx) => Ok(ctx.clone()),
            _ => Err(EngineError::InvalidArgument("Expected ZkContext".to_string())),
        }
    }
}

pub fn context_to_enum(ctx: &Context) -> Context {
    ctx.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::effect::types::Right;
    use causality_types::ContentId;
    
    #[test]
    fn test_abstract_context() {
        // Create a ContentId for the resource
        let resource_id = ContentId::random();
        
        // Create a capability with the resource_id and Right
        let capability = Capability::new(resource_id, Right::Read);
        
        let context = AbstractContext::new(ExecutionPhase::Planning)
            .with_proof_required(true)
            .with_capability(capability.clone());
        
        assert_eq!(context.environment(), ExecutionEnvironment::Abstract);
        assert_eq!(context.phase(), ExecutionPhase::Planning);
        assert_eq!(context.domain(), None);
        assert!(context.proof_required());
        assert_eq!(context.required_capabilities().len(), 1);
    }
    
    #[test]
    fn test_register_context() {
        // Create a ContentId for the resource
        let resource_id = ContentId::random();
        
        // Create a capability with the resource_id and Right
        let capability = Capability::new(resource_id, Right::Read);
        
        let context = RegisterContext::new(ExecutionPhase::Execution, "test_namespace")
            .with_proof_required(true)
            .with_capability(capability.clone());
        
        assert_eq!(context.environment(), ExecutionEnvironment::Register);
        assert_eq!(context.phase(), ExecutionPhase::Execution);
        assert_eq!(context.domain(), None);
        assert!(context.proof_required());
        assert_eq!(context.required_capabilities().len(), 1);
    }
    
    #[test]
    fn test_physical_context() {
        let domain_id = DomainId::new("test_domain");
        
        // Create a ContentId for the resource
        let resource_id = ContentId::random();
        
        // Create a capability with the resource_id and Right
        let capability = Capability::new(resource_id, Right::Read);
        
        let context = PhysicalContext::new(ExecutionPhase::Execution, domain_id.clone())
            .with_proof_required(true)
            .with_capability(capability.clone());
        
        assert_eq!(context.environment(), ExecutionEnvironment::OnChain(domain_id.clone()));
        assert_eq!(context.phase(), ExecutionPhase::Execution);
        assert_eq!(context.domain(), Some(domain_id));
        assert!(context.proof_required());
        assert_eq!(context.required_capabilities().len(), 1);
    }
} 