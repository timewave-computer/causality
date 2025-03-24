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
use borsh::{BorshSerialize, BorshDeserialize};

use crate::types::{DomainId};
use crate::effect::{Effect, EffectOutcome};
use crate::resource::ResourceRegisterTrait;
use crate::verification::UnifiedProof;
use crate::zk::Proof;
use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};
use crate::tel::resource::operations::{ResourceOperation, ResourceOperationType};
use crate::tel::types::OperationId as TelOperationId;
use std::time::SystemTime;

/// Unique identifier for operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub String);

/// Content data for operation ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct OperationIdContent {
    /// Operation type
    op_type: String,
    /// Timestamp
    timestamp: u64,
    /// Random nonce for uniqueness
    nonce: [u8; 8],
}

impl ContentAddressed for OperationIdContent {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl OperationId {
    /// Create a new operation ID
    pub fn new() -> Self {
        // Create content for ID generation
        let content = OperationIdContent {
            op_type: "generic".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate content-derived ID
        let content_id = content.content_id();
        OperationId(format!("op:{}", content_id))
    }
    
    /// Create a new operation ID for a specific operation type
    pub fn for_operation_type(op_type: &str) -> Self {
        // Create content for ID generation
        let content = OperationIdContent {
            op_type: op_type.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate content-derived ID
        let content_id = content.content_id();
        OperationId(format!("op:{}:{}", op_type, content_id))
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

impl From<ContentId> for OperationId {
    fn from(content_id: ContentId) -> Self {
        Self(format!("op:{}", content_id))
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
    /// Resource ID (using content-based identification)
    pub resource_id: ContentId,
    
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
    /// Register identifier (content-based)
    pub register_id: ContentId,
    
    /// Register operation type
    pub operation: RegisterOperationType,
    
    /// Operation data
    pub data: HashMap<String, String>,
}

/// Type of register operation
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
    
    /// Unfreeze a register
    Unfreeze,
    
    /// Mark register as pending
    MarkPending,
    
    /// Consume a register (mark as used up)
    Consume,
    
    /// Archive a register
    Archive,
    
    /// Unarchive a register
    Unarchive,
    
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
    
    /// Convert the operation to a ResourceOperation for use with TEL verification
    pub fn convert_to_resource_operation(&self) -> ResourceOperation {
        // Create a ResourceOperation from this operation
        let operation_type = match &self.op_type {
            OperationType::Register(op_type) => match op_type {
                RegisterOperationType::Create => ResourceOperationType::Create,
                RegisterOperationType::Update => ResourceOperationType::Update,
                RegisterOperationType::Transfer => ResourceOperationType::Transfer,
                RegisterOperationType::Lock => ResourceOperationType::Lock,
                RegisterOperationType::Unlock => ResourceOperationType::Unlock,
                RegisterOperationType::Freeze => ResourceOperationType::Custom(1), // Using 1 for Freeze
                RegisterOperationType::Unfreeze => ResourceOperationType::Custom(2), // Using 2 for Unfreeze
                RegisterOperationType::Consume => ResourceOperationType::Delete,
                RegisterOperationType::Archive => ResourceOperationType::Custom(3), // Using 3 for Archive 
                RegisterOperationType::Unarchive => ResourceOperationType::Custom(4), // Using 4 for Unarchive
                _ => ResourceOperationType::Custom(0), // Default for other types
            },
            _ => ResourceOperationType::Custom(0), // Default for non-register operations
        };
        
        // Get target content ID from the first output resource (if available)
        let target = if let Some(resource_ref) = self.outputs.first() {
            resource_ref.resource_id.clone()
        } else {
            // Use operation ID hash as fallback
            ContentId::from_bytes(&self.id.0.as_bytes()[0..32].to_vec())
        };
        
        // Convert Proof format if available
        let proof = self.proof.as_ref().map(|p| {
            crate::tel::types::Proof {
                proof_type: "operation_proof".to_string(),
                data: p.data.clone(),
                verification_key: Some(p.verification_key.clone()),
            }
        });
        
        // Generate TEL operation ID from this operation's ID
        let operation_id = TelOperationId::from_content_id(&ContentId::from_bytes(&self.id.0.as_bytes()));
        
        // Extract a timestamp from metadata if available
        let timestamp = self.metadata.get("timestamp")
            .and_then(|ts| ts.parse::<u64>().ok())
            .unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
            });
        
        // Construct the resource operation
        ResourceOperation {
            operation_id,
            operation_type,
            target,
            inputs: Vec::new(), // Could populate from self.inputs if needed
            outputs: Vec::new(), // Could populate from self.outputs if needed
            proof,
            verification_key: self.proof.as_ref().map(|p| p.verification_key.clone()),
            domain: self.metadata.get("domain").cloned().unwrap_or_else(|| "unknown".to_string()),
            initiator: self.metadata.get("initiator").cloned().unwrap_or_else(|| "system".to_string()),
            parameters: HashMap::new(), // Map metadata to parameters if needed
            metadata: self.metadata.clone(),
            timestamp,
        }
    }
}

// Backward compatibility method
impl ResourceRef {
    /// Create resource reference from legacy ContentId
    #[deprecated(since = "0.2.0", note = "Use ContentId directly instead")]
    pub fn from_legacy_id(resource_id: crate::types::ContentId, domain_id: Option<DomainId>, ref_type: ResourceRefType) -> Self {
        let id_string = resource_id.to_string();
        let content_id = ContentId::new(&id_string);
        
        ResourceRef {
            resource_id: content_id,
            domain_id,
            ref_type,
            before_state: None,
            after_state: None,
        }
    }
}

// Backward compatibility for RegisterOperation
impl RegisterOperation {
    /// Create from legacy string ID
    #[deprecated(since = "0.2.0", note = "Use ContentId directly instead")]
    pub fn from_legacy_id(register_id: String, operation: RegisterOperationType, data: HashMap<String, String>) -> Self {
        let content_id = ContentId::new(&register_id);
        
        RegisterOperation {
            register_id: content_id,
            operation,
            data,
        }
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
