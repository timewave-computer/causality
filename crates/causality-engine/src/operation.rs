// Operation system for atomic state changes
// Original file: src/operation/mod.rs

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
mod verification;
// External tests module - not part of public API
#[cfg(test)]
mod test_fixtures;

pub use context::*;
pub use transformation::*;
pub use execution::*;
pub use zk::*;
pub use api::*;
pub use verification::*;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use std::fmt;
use std::time::SystemTime;

use causality_error::{EngineResult as Result, EngineError};
use causality_types::ContentId;
use causality_core::Effect;
use causality_core::EffectOutcome;
use causality_types::{DomainId, Timestamp};
use causality_core::effect::{ResourceOperation, ResourceOperationType};
use causality_core::domain::{Address, Domain};
use causality_core::metadata::Metadata;
use causality_core::params::Parameters;
use causality_core::content::ContentAddressed;

/// Content-addressed functionality
pub trait ContentAddressed {
    /// Get the content hash
    fn content_hash(&self) -> Vec<u8>;
    
    /// Verify the content hash
    fn verify(&self) -> bool;
    
    /// Convert to bytes
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Convert from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized;
    
    /// Get content ID
    fn content_id(&self) -> ContentId {
        let hash = self.content_hash();
        ContentId::from_bytes(&hash)
    }
}

/// HashError for content addressing
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Invalid hash format
    #[error("Invalid hash format: {0}")]
    InvalidFormat(String),
    
    /// Hash computation error
    #[error("Hash computation error: {0}")]
    ComputationError(String),
}

/// Unified proof abstraction - combines different proof types
pub enum UnifiedProof {
    /// A cryptographic proof (e.g., signature)
    Cryptographic(Vec<u8>),
    /// A ZK proof
    ZeroKnowledge(Vec<u8>),
    /// A capability-based proof
    Capability(String),
    /// Multiple proofs combined
    Composite(Vec<UnifiedProof>),
}

/// Proof type from the core library
pub struct Proof {
    /// Proof data
    pub data: Vec<u8>,
    /// Proof type
    pub proof_type: String,
    /// Verification key
    pub verification_key: Option<Vec<u8>>,
}

/// ResourceOperation type for operations
pub struct ResourceOperation {
    /// Operation ID
    pub id: String,
    /// Operation type
    pub operation_type: ResourceOperationType,
    /// Operation parameters
    pub parameters: HashMap<String, String>,
}

/// ResourceOperationType for operations
pub enum ResourceOperationType {
    /// Create a resource
    Create,
    /// Read a resource
    Read,
    /// Update a resource
    Update,
    /// Delete a resource
    Delete,
    /// Transfer a resource
    Transfer,
    /// Custom operation
    Custom(String),
}

/// Unique identifier for operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub String);

/// Content data for operation ID generation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationIdContent {
    /// Operation type
    op_type: String,
    /// Timestamp
    timestamp: u64,
    /// Random nonce for uniqueness
    nonce: [u8; 8],
}

impl ContentAddressed for OperationIdContent {
    fn content_hash(&self) -> Vec<u8> {
        // Use simple serialization and SHA-256 hash
        let serialized = self.to_bytes();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&serialized);
        hasher.finalize().to_vec()
    }
    
    fn verify(&self) -> bool {
        // Simple verification - just rehash and compare
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let mut hasher = sha2::Sha256::new();
        hasher.update(&serialized);
        let computed_hash = hasher.finalize().to_vec();
        
        hash == computed_hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // Use serde_json for serialization
        serde_json::to_vec(self)
            .unwrap_or_else(|_| Vec::new())
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| causality_error::Error::SerializationError(e.to_string()))
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

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::Transfer => write!(f, "transfer"),
            OperationType::Deposit => write!(f, "deposit"),
            OperationType::Withdrawal => write!(f, "withdrawal"),
            OperationType::Create => write!(f, "create"),
            OperationType::Update => write!(f, "update"),
            OperationType::Delete => write!(f, "delete"),
            OperationType::Merge => write!(f, "merge"),
            OperationType::Split => write!(f, "split"),
            OperationType::Custom(s) => write!(f, "custom:{}", s),
        }
    }
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
    
    /// Convert an operation into a TEL resource operation
    pub fn convert_to_resource_operation(&self) -> Option<ResourceOperation> {
        if self.inputs.is_empty() && self.outputs.is_empty() {
            return None;
        }

        // Determine operation type
        let operation_type = match self.op_type {
            OperationType::Create => ResourceOperationType::Create,
            OperationType::Update => ResourceOperationType::Update,
            OperationType::Transfer => ResourceOperationType::Transfer,
            OperationType::Delete => ResourceOperationType::Delete,
            _ => ResourceOperationType::Custom(0),
        };

        // Get content IDs for targets
        let target_content_ids: Vec<ContentId> = self.inputs.iter()
            .chain(self.outputs.iter())
            .map(|ref_| ref_.resource_id.clone())
            .collect();

        if target_content_ids.is_empty() {
            return None;
        }

        // Get the first content ID as target
        let target = target_content_ids[0].clone();

        // Current time as timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Convert proof format if available
        let (proof, verification_key) = if let Some(p) = &self.proof {
            let tel_proof = TelProof {
                proof_type: "operation_proof".to_string(),
                data: p.data.clone(),
            };
            (Some(tel_proof), Some(p.verification_key.clone()))
        } else {
            (None, None)
        };

        // Create the resource operation
        Some(ResourceOperation {
            operation_type,
            target,
            inputs: Vec::new(),      // Empty for now
            outputs: Vec::new(),     // Empty for now
            proof,
            verification_key,
            domain: self.metadata.get("domain").cloned().unwrap_or_else(|| "default".to_string()),
            initiator: self.metadata.get("initiator").cloned().unwrap_or_else(|| "system".to_string()),
            parameters: Parameters::new(),
            metadata: self.metadata.clone(),
            timestamp,
        })
    }
}

// Backward compatibility method
impl ResourceRef {
    /// Create resource reference from legacy ContentId
    #[deprecated(since = "0.2.0", note = "Use ContentId directly instead")]
    pub fn from_legacy_id(resource_id: causality_types::ContentId, domain_id: Option<DomainId>, ref_type: ResourceRefType) -> Self {
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
