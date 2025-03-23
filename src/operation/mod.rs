// Unified Operation Model
//
// This module implements a unified operation model that consolidates different
// operational concepts (Effects, Operations, ResourceRegister operations) into
// a single abstraction with different execution contexts.

mod context;
mod transformation;
mod execution;
mod zk;
mod api;
// External tests module - not part of public API
#[cfg(test)]
mod test_fixtures;

pub use context::*;
pub use transformation::*;
pub use execution::*;
pub use zk::*;
pub use api::*;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::types::{ResourceId, DomainId};
use crate::effect::{Effect, EffectOutcome};
use crate::resource::ResourceRegisterTrait;
use crate::verification::UnifiedProof;
use crate::zk::Proof;

/// Unique identifier for operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub String);

impl OperationId {
    /// Create a new operation ID
    pub fn new() -> Self {
        OperationId(Uuid::new_v4().to_string())
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of operations supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Transfer ownership of a resource
    Transfer,
    
    /// Deposit a resource into the system
    Deposit,
    
    /// Withdraw a resource from the system
    Withdrawal,
    
    /// Create a new resource
    Create,
    
    /// Update an existing resource
    Update,
    
    /// Delete a resource
    Delete,
    
    /// Merge multiple resources
    Merge,
    
    /// Split a resource into multiple resources
    Split,
    
    /// Execute a custom operation
    Custom(String),
}

/// Reference to a resource involved in an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    /// Resource ID
    pub resource_id: ResourceId,
    
    /// Domain ID (if cross-domain)
    pub domain_id: Option<DomainId>,
    
    /// Reference type (input or output)
    pub ref_type: ResourceRefType,
    
    /// State before operation (for inputs)
    pub before_state: Option<String>,
    
    /// State after operation (for outputs)
    pub after_state: Option<String>,
}

/// Type of resource reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceRefType {
    /// Input resource (read)
    Input,
    
    /// Output resource (write)
    Output,
    
    /// Both input and output (read-write)
    ReadWrite,
}

/// Authorization for an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorization {
    /// Authorization type
    pub auth_type: AuthorizationType,
    
    /// Authorization data (signature, capability proof, etc.)
    pub data: Vec<u8>,
    
    /// Authorizing entity
    pub authorizer: String,
}

/// Types of authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationType {
    /// Signature-based authorization
    Signature,
    
    /// Capability-based authorization
    Capability,
    
    /// Role-based authorization
    Role,
    
    /// Zero-knowledge proof authorization
    ZkProof,
    
    /// Multi-signature authorization
    MultiSig,
    
    /// No authorization (system operation)
    None,
}

/// Resource conservation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConservation {
    /// Whether conservation laws are enforced for this operation
    pub enforce: bool,
    
    /// Conservation domain (what properties must be conserved)
    pub domain: ConservationDomain,
    
    /// Input totals for conserved properties
    pub input_totals: HashMap<String, u64>,
    
    /// Output totals for conserved properties
    pub output_totals: HashMap<String, u64>,
}

/// Domain of conservation laws
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConservationDomain {
    /// Quantity conservation (e.g., token amounts)
    Quantity,
    
    /// Value conservation (e.g., financial value)
    Value,
    
    /// Information conservation
    Information,
    
    /// Custom conservation domain
    Custom(String),
    
    /// No conservation
    None,
}

/// Unified Operation model that spans abstraction levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation<C: ExecutionContext> {
    /// Unique identifier for this operation
    pub id: OperationId,
    
    /// The operation type
    pub op_type: OperationType,
    
    /// Abstract representation (what the operation logically does)
    pub abstract_representation: Box<dyn Effect>,
    
    /// Concrete implementation (how it's implemented on registers)
    pub concrete_implementation: Option<RegisterOperation>,
    
    /// Physical execution details (actual on-chain state changes)
    pub physical_execution: Option<PhysicalOperation>,
    
    /// Execution context (where and how it executes)
    pub context: C,
    
    /// Input resources/registers this operation reads from
    pub inputs: Vec<ResourceRef>,
    
    /// Output resources/registers this operation writes to
    pub outputs: Vec<ResourceRef>,
    
    /// Authorization for this operation
    pub authorization: Authorization,
    
    /// Proof for this operation (if applicable)
    pub proof: Option<UnifiedProof>,
    
    /// ZK proof for on-chain verification
    pub zk_proof: Option<Proof>,
    
    /// Resource conservation details
    pub conservation: ResourceConservation,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Register-level operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterOperation {
    /// Register identifier
    pub register_id: String,
    
    /// Register operation type
    pub operation: RegisterOperationType,
    
    /// Operation data
    pub data: HashMap<String, String>,
}

/// Types of register operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterOperationType {
    /// Create a new register
    Create,
    
    /// Update a register
    Update,
    
    /// Transfer ownership
    Transfer,
    
    /// Lock a register
    Lock,
    
    /// Unlock a register
    Unlock,
    
    /// Freeze a register
    Freeze,
    
    /// Archive a register
    Archive,
    
    /// Custom register operation
    Custom(String),
}

/// Physical on-chain operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalOperation {
    /// Domain ID
    pub domain_id: DomainId,
    
    /// Transaction hash (if available)
    pub tx_hash: Option<String>,
    
    /// Block height (if available)
    pub block_height: Option<u64>,
    
    /// On-chain operation data
    pub data: Vec<u8>,
}

impl<C: ExecutionContext> Operation<C> {
    /// Create a new operation
    pub fn new(op_type: OperationType, effect: Box<dyn Effect>, context: C) -> Self {
        Self {
            id: OperationId::new(),
            op_type,
            abstract_representation: effect,
            concrete_implementation: None,
            physical_execution: None,
            context,
            inputs: Vec::new(),
            outputs: Vec::new(),
            authorization: Authorization {
                auth_type: AuthorizationType::None,
                data: Vec::new(),
                authorizer: String::new(),
            },
            proof: None,
            zk_proof: None,
            conservation: ResourceConservation {
                enforce: false,
                domain: ConservationDomain::None,
                input_totals: HashMap::new(),
                output_totals: HashMap::new(),
            },
            metadata: HashMap::new(),
        }
    }
    
    /// Add an input resource
    pub fn with_input(mut self, resource_ref: ResourceRef) -> Self {
        self.inputs.push(resource_ref);
        self
    }
    
    /// Add an output resource
    pub fn with_output(mut self, resource_ref: ResourceRef) -> Self {
        self.outputs.push(resource_ref);
        self
    }
    
    /// Set authorization
    pub fn with_authorization(mut self, authorization: Authorization) -> Self {
        self.authorization = authorization;
        self
    }
    
    /// Set proof
    pub fn with_proof(mut self, proof: UnifiedProof) -> Self {
        self.proof = Some(proof);
        self
    }
    
    /// Set ZK proof
    pub fn with_zk_proof(mut self, proof: Proof) -> Self {
        self.zk_proof = Some(proof);
        self
    }
    
    /// Set conservation rules
    pub fn with_conservation(mut self, conservation: ResourceConservation) -> Self {
        self.conservation = conservation;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set concrete implementation
    pub fn with_concrete_implementation(mut self, implementation: RegisterOperation) -> Self {
        self.concrete_implementation = Some(implementation);
        self
    }
    
    /// Set physical execution
    pub fn with_physical_execution(mut self, execution: PhysicalOperation) -> Self {
        self.physical_execution = Some(execution);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Basic tests for OperationId
    #[test]
    fn test_operation_id() {
        let id = OperationId::new();
        assert!(!id.as_str().is_empty());
        
        let formatted = format!("{}", id);
        assert_eq!(formatted, id.0);
    }
} 