// Register-based resource model for Causality
//
// This module implements the register-based resource model as defined in ADR-006.
// It provides the core data structures for register representation, operations,
// and authorization with a focus on the ZK-based verification approach.
//
// Registers in Causality are designed to be one-time use to prevent replay attacks
// and ensure the integrity of operations across domains. Once a register is used
// in an operation (update, transfer, deletion), it is marked as consumed and cannot
// be reused for another operation. A new register must be created to store the
// resulting state.

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use rand; // For simulating hashes and nullifiers
use uuid::Uuid;

use crate::types::ResourceId;
use crate::tel::{Address, Domain};
use crate::tel::types::Metadata;
use crate::error::{Error, Result};
use crate::ast::AstContext;

// Define mock types for ZK proof integration
// In a real implementation, these would be imported from the zkp module
mod mock_zkp {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::sync::Arc;
    
    /// Mock proof system type 
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProofSystem {
        Groth16,
        Plonk,
        Stark,
    }
    
    /// Mock proof type
    #[derive(Debug, Clone)]
    pub struct Proof {
        pub proof_data: Vec<u8>,
        pub proof_system: ProofSystem,
    }
    
    /// Mock public inputs type
    #[derive(Debug, Clone, Default)]
    pub struct PublicInputs {
        data: HashMap<String, String>,
    }
    
    impl PublicInputs {
        pub fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
        
        pub fn insert(&mut self, key: String, value: String) {
            self.data.insert(key, value);
        }
        
        pub fn get(&self, key: &str) -> Option<&String> {
            self.data.get(key)
        }
    }
    
    /// Mock circuit trait
    pub trait Circuit: Debug + Send + Sync {
        fn verify(&self, proof: &Proof, public_inputs: &PublicInputs, verification_key: &[u8]) -> bool;
    }
    
    // Mock circuit implementations for different register operations
    
    /// Mock create circuit
    #[derive(Debug)]
    pub struct RegisterCreateCircuit {}
    
    impl Circuit for RegisterCreateCircuit {
        fn verify(&self, _proof: &Proof, _public_inputs: &PublicInputs, _verification_key: &[u8]) -> bool {
            // In a real implementation, this would verify the proof
            true
        }
    }
    
    /// Mock update circuit
    #[derive(Debug)]
    pub struct RegisterUpdateCircuit {}
    
    impl Circuit for RegisterUpdateCircuit {
        fn verify(&self, _proof: &Proof, _public_inputs: &PublicInputs, _verification_key: &[u8]) -> bool {
            // In a real implementation, this would verify the proof
            true
        }
    }
    
    /// Mock transfer circuit
    #[derive(Debug)]
    pub struct RegisterTransferCircuit {}
    
    impl Circuit for RegisterTransferCircuit {
        fn verify(&self, _proof: &Proof, _public_inputs: &PublicInputs, _verification_key: &[u8]) -> bool {
            // In a real implementation, this would verify the proof
            true
        }
    }
    
    /// Mock delete circuit
    #[derive(Debug)]
    pub struct RegisterDeleteCircuit {}
    
    impl Circuit for RegisterDeleteCircuit {
        fn verify(&self, _proof: &Proof, _public_inputs: &PublicInputs, _verification_key: &[u8]) -> bool {
            // In a real implementation, this would verify the proof
            true
        }
    }
    
    /// Mock merge circuit
    #[derive(Debug)]
    pub struct RegisterMergeCircuit {}
    
    impl Circuit for RegisterMergeCircuit {
        fn verify(&self, _proof: &Proof, _public_inputs: &PublicInputs, _verification_key: &[u8]) -> bool {
            // In a real implementation, this would verify the proof
            true
        }
    }
    
    /// Mock split circuit
    #[derive(Debug)]
    pub struct RegisterSplitCircuit {}
    
    impl Circuit for RegisterSplitCircuit {
        fn verify(&self, _proof: &Proof, _public_inputs: &PublicInputs, _verification_key: &[u8]) -> bool {
            // In a real implementation, this would verify the proof
            true
        }
    }
    
    // Mock circuit creation functions
    pub mod register {
        use super::*;
        
        pub mod create {
            use super::*;
            
            pub fn create_register_create_circuit() -> Arc<dyn Circuit> {
                Arc::new(RegisterCreateCircuit {})
            }
        }
        
        pub mod update {
            use super::*;
            
            pub fn create_register_update_circuit() -> Arc<dyn Circuit> {
                Arc::new(RegisterUpdateCircuit {})
            }
        }
        
        pub mod transfer {
            use super::*;
            
            pub fn create_register_transfer_circuit() -> Arc<dyn Circuit> {
                Arc::new(RegisterTransferCircuit {})
            }
        }
        
        pub mod delete {
            use super::*;
            
            pub fn create_register_delete_circuit() -> Arc<dyn Circuit> {
                Arc::new(RegisterDeleteCircuit {})
            }
        }
        
        pub mod merge {
            use super::*;
            
            pub fn create_register_merge_circuit() -> Arc<dyn Circuit> {
                Arc::new(RegisterMergeCircuit {})
            }
        }
        
        pub mod split {
            use super::*;
            
            pub fn create_register_split_circuit() -> Arc<dyn Circuit> {
                Arc::new(RegisterSplitCircuit {})
            }
        }
    }
}

// Use our mock types
use mock_zkp::{Circuit, Proof, ProofSystem, PublicInputs};

/// A globally unique identifier for a register
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegisterId(pub Uuid);

impl RegisterId {
    /// Create a new random register ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create a register ID from a string
    pub fn from_str(s: &str) -> Result<Self> {
        let uuid = Uuid::parse_str(s).map_err(|e| 
            Error::ParseError(format!("Invalid register ID: {}", e)))?;
        Ok(Self(uuid))
    }
    
    /// Create a register ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    /// Create a deterministic register ID based on a type and name
    pub fn deterministic(register_type: &str, name: &str) -> Self {
        // Create a namespace UUID for registers (using a fixed UUID)
        let namespace = Uuid::parse_str("f2d0ce98-2c3a-4d3b-8a3c-d84fe8e3c29a")
            .expect("Invalid namespace UUID");
            
        // Create a deterministic UUID based on the type and name
        let combined = format!("{}:{}", register_type, name);
        let uuid = Uuid::new_v5(&namespace, combined.as_bytes());
        
        Self(uuid)
    }
    
    /// Get the underlying UUID
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RegisterId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RegisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Block height type
pub type BlockHeight = u64;

/// State of a register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterState {
    /// Register is active and can be used
    Active,
    /// Register is locked and cannot be modified
    Locked,
    /// Register is frozen and cannot be used
    Frozen,
    /// Register has been consumed and cannot be used again (for one-time registers)
    Consumed,
    /// Register is archived (after garbage collection)
    Archived,
    /// Register is a summary of other registers
    Summary,
    /// Register is scheduled for deletion
    PendingDeletion,
    /// Register has been deleted but is kept as a tombstone
    Tombstone,
}

impl RegisterState {
    /// Check if the register is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if the register is locked
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Locked)
    }
    
    /// Check if the register is frozen
    pub fn is_frozen(&self) -> bool {
        matches!(self, Self::Frozen)
    }
    
    /// Check if the register has been consumed
    pub fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed)
    }
    
    /// Check if the register is archived
    pub fn is_archived(&self) -> bool {
        matches!(self, Self::Archived)
    }
    
    /// Check if the register is a summary
    pub fn is_summary(&self) -> bool {
        matches!(self, Self::Summary)
    }
    
    /// Check if the register is pending deletion
    pub fn is_pending_deletion(&self) -> bool {
        matches!(self, Self::PendingDeletion)
    }
    
    /// Check if the register is a tombstone
    pub fn is_tombstone(&self) -> bool {
        matches!(self, Self::Tombstone)
    }
}

/// Contents of a register
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegisterContents {
    /// Binary data
    Binary(Vec<u8>),
    
    /// String data
    String(String),
    
    /// JSON data
    Json(serde_json::Value),
    
    /// Token balance
    TokenBalance {
        token_type: String,
        address: Address,
        amount: u64,
    },
    
    /// NFT content
    NFTContent {
        collection_address: String,
        token_id: String,
    },
    
    /// State commitment
    StateCommitment {
        commitment_type: String,
        data: Vec<u8>,
    },
    
    /// Time map commitment
    TimeMapCommitment {
        block_height: BlockHeight,
        data: Vec<u8>,
    },
    
    /// Generic data object
    DataObject {
        data_format: String,
        data: Vec<u8>,
    },
    
    /// Effect DAG
    EffectDAG {
        effect_id: String,
        data: Vec<u8>,
    },
    
    /// Resource nullifier
    ResourceNullifier {
        nullifier_key: String,
        data: Vec<u8>,
    },
    
    /// Resource commitment
    ResourceCommitment {
        commitment_key: String,
        data: Vec<u8>,
    },
    
    /// Composite contents (contains multiple sub-contents)
    CompositeContents(Vec<RegisterContents>),
    
    /// Empty register
    Empty,
}

impl RegisterContents {
    /// Create new binary contents
    pub fn binary(data: Vec<u8>) -> Self {
        Self::Binary(data)
    }
    
    /// Create new string contents
    pub fn string(data: String) -> Self {
        Self::String(data)
    }
    
    /// Create new JSON contents
    pub fn json(data: serde_json::Value) -> Self {
        Self::Json(data)
    }
    
    /// Create token balance contents
    pub fn token_balance(token_type: String, address: Address, amount: u64) -> Self {
        Self::TokenBalance {
            token_type,
            address,
            amount,
        }
    }
    
    /// Create NFT contents
    pub fn nft_content(collection_address: String, token_id: String) -> Self {
        Self::NFTContent {
            collection_address,
            token_id,
        }
    }
    
    /// Create state commitment contents
    pub fn state_commitment(commitment_type: String, data: Vec<u8>) -> Self {
        Self::StateCommitment {
            commitment_type,
            data,
        }
    }
    
    /// Create time map commitment contents
    pub fn time_map_commitment(block_height: BlockHeight, data: Vec<u8>) -> Self {
        Self::TimeMapCommitment {
            block_height,
            data,
        }
    }
    
    /// Create data object contents
    pub fn data_object(data_format: String, data: Vec<u8>) -> Self {
        Self::DataObject {
            data_format,
            data,
        }
    }
    
    /// Create effect DAG contents
    pub fn effect_dag(effect_id: String, data: Vec<u8>) -> Self {
        Self::EffectDAG {
            effect_id,
            data,
        }
    }
    
    /// Create resource nullifier contents
    pub fn resource_nullifier(nullifier_key: String, data: Vec<u8>) -> Self {
        Self::ResourceNullifier {
            nullifier_key,
            data,
        }
    }
    
    /// Create resource commitment contents
    pub fn resource_commitment(commitment_key: String, data: Vec<u8>) -> Self {
        Self::ResourceCommitment {
            commitment_key,
            data,
        }
    }
    
    /// Create composite contents
    pub fn composite(contents: Vec<RegisterContents>) -> Self {
        Self::CompositeContents(contents)
    }
    
    /// Create empty contents
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// Get size of contents in bytes (approximate)
    pub fn size(&self) -> usize {
        match self {
            Self::Binary(data) => data.len(),
            Self::String(data) => data.len(),
            Self::Json(data) => serde_json::to_string(data).unwrap_or_default().len(),
            Self::TokenBalance { .. } => 64, // Approximate size
            Self::NFTContent { .. } => 128, // Approximate size
            Self::StateCommitment { data, .. } => data.len() + 32,
            Self::TimeMapCommitment { data, .. } => data.len() + 16,
            Self::DataObject { data, .. } => data.len() + 32,
            Self::EffectDAG { data, .. } => data.len() + 32,
            Self::ResourceNullifier { data, .. } => data.len() + 32,
            Self::ResourceCommitment { data, .. } => data.len() + 32,
            Self::CompositeContents(contents) => contents.iter().map(|c| c.size()).sum(),
            Self::Empty => 0,
        }
    }
}

/// Time range for register validity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time of the range (milliseconds since epoch)
    pub start: u64,
    /// End time of the range (milliseconds since epoch, None means indefinite)
    pub end: Option<u64>,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: u64, end: Option<u64>) -> Self {
        Self { start, end }
    }
    
    /// Create a time range from now with the given duration
    pub fn from_now(duration_ms: Option<u64>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self::new(now, duration_ms.map(|d| now + d))
    }
    
    /// Check if the range includes the given time
    pub fn includes(&self, time: u64) -> bool {
        time >= self.start && self.end.map_or(true, |end| time <= end)
    }
    
    /// Check if the range is currently valid
    pub fn is_valid_now(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        self.includes(now)
    }
}

/// Operation verification result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the verification was successful
    pub success: bool,
    /// Detailed status message
    pub message: String,
    /// Additional details about the verification
    pub details: HashMap<String, serde_json::Value>,
}

impl VerificationResult {
    /// Create a new successful verification result
    pub fn success() -> Self {
        Self {
            success: true,
            message: "Verification successful".to_string(),
            details: HashMap::new(),
        }
    }
    
    /// Create a new failed verification result
    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            details: HashMap::new(),
        }
    }
    
    /// Add a detail to the verification result
    pub fn with_detail(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }
}

/// Methods for authorizing register operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationMethod {
    /// Owner-signed authorization with digital signature
    OwnerSigned(Address),
    
    /// Authorization with a ZK proof
    ZKProofAuthorization {
        /// Verification key for the proof
        verification_key: String,
        /// The ZK proof
        proof: Vec<u8>,
    },
    
    /// Token-based authorization
    TokenOwnershipAuthorization {
        /// Token address
        token_address: String,
        /// Token amount required
        amount: u64,
        /// Token owner address
        owner: Address,
    },
    
    /// Multi-signature authorization
    MultiSigAuthorization {
        /// Addresses required for authorization
        addresses: Vec<Address>,
        /// Signatures from these addresses
        signatures: Vec<Vec<u8>>,
        /// Minimum number of signatures required
        threshold: usize,
    },
    
    /// Time-locked authorization (cannot be used until timestamp)
    TimeLockedAuthorization {
        /// Underlying authorization method
        method: Box<AuthorizationMethod>,
        /// Earliest time this authorization can be used
        unlock_time: u64,
    },
    
    /// DAO governance authorization
    DAOAuthorization {
        /// Address of the DAO
        dao_address: String,
        /// ID of the proposal that authorized this
        proposal_id: String,
        /// Proof of proposal execution
        execution_proof: Vec<u8>,
    },
    
    /// Identity-based authorization
    IdentityAuthorization {
        /// Identity provider
        provider: String,
        /// Identity proof
        proof: Vec<u8>,
        /// Identity claim fields
        claims: HashMap<String, String>,
    },
    
    /// Verifiable credential authorization
    CredentialAuthorization {
        /// Credential type
        credential_type: String,
        /// Credential proof
        proof: Vec<u8>,
        /// Issuer of the credential
        issuer: String,
    },
    
    /// Delegation-based authorization
    DelegatedAuthorization {
        /// Original owner
        delegator: Address,
        /// Delegated address
        delegate: Address,
        /// Delegation proof
        delegation_proof: Vec<u8>,
        /// Methods that are delegated
        methods: Vec<String>,
        /// Expiry time of delegation
        expiry: Option<u64>,
    },
    
    /// Any of the contained methods is sufficient
    Any(Vec<AuthorizationMethod>),
    
    /// At least threshold number of methods must pass
    Threshold(usize, Vec<AuthorizationMethod>),
}

/// Type of operation to perform on registers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Create a new register
    CreateRegister,
    /// Update an existing register
    UpdateRegister,
    /// Delete a register
    DeleteRegister,
    /// Transfer ownership of a register
    TransferOwnership(Address),
    /// Merge multiple registers
    MergeRegisters,
    /// Split a register into multiple registers
    SplitRegister,
    /// Composite operation (multiple operations combined)
    CompositeOperation(Vec<OperationType>),
}

/// A register operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterOperation {
    /// Type of operation
    pub op_type: OperationType,
    /// IDs of registers involved in the operation
    pub registers: Vec<RegisterId>,
    /// New contents for the register (for create/update operations)
    pub new_contents: Option<RegisterContents>,
    /// Authorization for the operation
    pub authorization: AuthorizationMethod,
    /// ZK proof verifying the operation's correctness
    pub proof: Option<Vec<u8>>,
    /// Resource delta for the operation
    pub resource_delta: String,
    /// Associated AST context
    pub ast_context: Option<AstContext>,
}

/// Nullifier for a register to prevent double-spending
/// 
/// As described in ADR-006, registers are one-time use and a nullifier
/// is used to mark a register as consumed to prevent replay attacks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterNullifier {
    /// ID of the register
    pub register_id: RegisterId,
    /// The nullifier value
    pub nullifier: Vec<u8>,
    /// Block height when the nullifier was created
    pub block_height: BlockHeight,
    /// Transaction ID that created the nullifier
    pub transaction_id: String,
}

impl RegisterNullifier {
    /// Create a new nullifier for a register
    pub fn new(register_id: RegisterId, transaction_id: String, block_height: BlockHeight) -> Self {
        // In a real implementation, this would use cryptographic functions to generate
        // a secure nullifier value. For now, we'll create a simple hash representation.
        let nullifier = {
            let mut data = Vec::new();
            data.extend_from_slice(register_id.to_string().as_bytes());
            data.extend_from_slice(transaction_id.as_bytes());
            data.extend_from_slice(&block_height.to_be_bytes());
            data
        };
        
        Self {
            register_id,
            nullifier,
            block_height,
            transaction_id,
        }
    }
    
    /// Get the nullifier value as a hex string
    pub fn as_hex(&self) -> String {
        // Convert the nullifier to a hex string
        self.nullifier.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

/// The register data structure
/// 
/// As defined in ADR-006, registers are one-time use atomic storage units
/// that can hold various types of content and are used to model state
/// transitions in the Causality system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Register {
    /// Unique identifier for the register
    pub register_id: RegisterId,
    /// Owner of the register
    pub owner: Address,
    /// Domain where the register exists
    pub domain: Domain,
    /// Contents of the register
    pub contents: RegisterContents,
    /// Current state of the register
    pub state: RegisterState,
    /// Time when the register was created
    pub created_at: u64,
    /// Time when the register was last updated
    pub last_updated: u64,
    /// Block height when the register was last updated
    pub last_updated_height: BlockHeight,
    /// Valid time range for the register
    pub validity: TimeRange,
    /// Epoch the register belongs to (for garbage collection)
    pub epoch: u64,
    /// Transaction that created this register
    pub created_by_tx: String,
    /// Transaction that consumed this register (if consumed)
    pub consumed_by_tx: Option<String>,
    /// IDs of registers created when this one was consumed
    pub successors: Vec<RegisterId>,
    /// If this register is a summary, the registers it summarizes
    pub summarizes: Option<Vec<RegisterId>>,
    /// If this is an archived register, reference to the archive
    pub archive_reference: Option<String>,
    /// Additional metadata
    pub metadata: Metadata,
    /// History of operations on the register
    pub history: Vec<String>,
}

impl Register {
    /// Create a new register
    /// 
    /// Creates a new active register with the specified owner, domain, and contents.
    pub fn new(
        register_id: RegisterId,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        metadata: Option<Metadata>,
        created_at: Option<u64>,
        last_updated: Option<u64>,
    ) -> Self {
        let now = created_at.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        
        let metadata = metadata.unwrap_or_else(|| {
            let mut m = HashMap::new();
            m.insert("created".to_string(), now.to_string());
            m
        });
        
        Self {
            register_id,
            owner,
            domain,
            contents,
            state: RegisterState::Active,
            created_at: now,
            last_updated: last_updated.unwrap_or(now),
            last_updated_height: 0, // Will be updated when committed
            validity: TimeRange::from_now(None),
            epoch: 0, // Will be set when added to the registry
            created_by_tx: "unknown".to_string(),
            consumed_by_tx: None,
            successors: Vec::new(),
            summarizes: None,
            archive_reference: None,
            metadata,
            history: Vec::new(),
        }
    }
    
    /// Create a new register with a random ID
    pub fn new_with_random_id(
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        metadata: Option<Metadata>,
    ) -> Self {
        Self::new(
            RegisterId::new(),
            owner,
            domain,
            contents,
            metadata,
            None,
            None,
        )
    }
    
    /// Create a new register with a deterministic ID based on name
    pub fn new_with_deterministic_id(
        name: &str,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        metadata: Option<Metadata>,
    ) -> Self {
        Self::new(
            RegisterId::deterministic("register", name),
            owner,
            domain,
            contents,
            metadata,
            None,
            None,
        )
    }
    
    /// Check if the register is active
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
    
    /// Check if the register is locked
    pub fn is_locked(&self) -> bool {
        self.state.is_locked()
    }
    
    /// Check if the register is frozen
    pub fn is_frozen(&self) -> bool {
        self.state.is_frozen()
    }
    
    /// Check if the register has been consumed
    pub fn is_consumed(&self) -> bool {
        self.state.is_consumed()
    }
    
    /// Check if the register is archived
    pub fn is_archived(&self) -> bool {
        self.state.is_archived()
    }
    
    /// Check if the register is a summary
    pub fn is_summary(&self) -> bool {
        self.state.is_summary()
    }
    
    /// Check if the register is pending deletion
    pub fn is_pending_deletion(&self) -> bool {
        self.state.is_pending_deletion()
    }
    
    /// Check if the register is a tombstone
    pub fn is_tombstone(&self) -> bool {
        self.state.is_tombstone()
    }
    
    /// Lock the register
    pub fn lock(&mut self) -> Result<()> {
        match self.state {
            RegisterState::Active => {
                self.state = RegisterState::Locked;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot lock register in state {:?}", self.state
            ))),
        }
    }
    
    /// Unlock the register
    pub fn unlock(&mut self) -> Result<()> {
        match self.state {
            RegisterState::Locked => {
                self.state = RegisterState::Active;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot unlock register in state {:?}", self.state
            ))),
        }
    }
    
    /// Freeze the register
    pub fn freeze(&mut self) -> Result<()> {
        match self.state {
            RegisterState::Active | RegisterState::Locked => {
                self.state = RegisterState::Frozen;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot freeze register in state {:?}", self.state
            ))),
        }
    }
    
    /// Unfreeze the register
    pub fn unfreeze(&mut self) -> Result<()> {
        match self.state {
            RegisterState::Frozen => {
                self.state = RegisterState::Active;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot unfreeze register in state {:?}", self.state
            ))),
        }
    }
    
    /// Consume the register (mark as one-time use)
    pub fn consume(&mut self, tx_id: &str, block_height: BlockHeight) -> Result<RegisterNullifier> {
        match self.state {
            RegisterState::Active => {
                self.state = RegisterState::Consumed;
                self.consumed_by_tx = Some(tx_id.to_string());
                self.last_updated_height = block_height;
                self.update_timestamp();
                
                // Create a nullifier
                let nullifier = RegisterNullifier::new(
                    self.register_id.clone(),
                    tx_id.to_string(),
                    block_height,
                );
                
                Ok(nullifier)
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot consume register in state {:?}", self.state
            ))),
        }
    }
    
    /// Mark the register for deletion
    pub fn mark_for_deletion(&mut self) -> Result<()> {
        match self.state {
            RegisterState::Active | RegisterState::Locked | RegisterState::Frozen => {
                self.state = RegisterState::PendingDeletion;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot mark register for deletion in state {:?}", self.state
            ))),
        }
    }
    
    /// Convert the register to a tombstone
    pub fn convert_to_tombstone(&mut self) -> Result<()> {
        match self.state {
            RegisterState::PendingDeletion => {
                self.state = RegisterState::Tombstone;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot convert register to tombstone in state {:?}", self.state
            ))),
        }
    }
    
    /// Archive the register
    pub fn archive(&mut self, archive_reference: &str) -> Result<()> {
        match self.state {
            RegisterState::Active | RegisterState::Consumed | RegisterState::Locked => {
                self.state = RegisterState::Archived;
                self.archive_reference = Some(archive_reference.to_string());
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot archive register in state {:?}", self.state
            ))),
        }
    }
    
    /// Mark this register as a summary of other registers
    pub fn mark_as_summary(&mut self, summarized_registers: Vec<RegisterId>) -> Result<()> {
        if summarized_registers.is_empty() {
            return Err(Error::InvalidArgument(
                "Cannot create a summary of zero registers".to_string()
            ));
        }
        
        match self.state {
            RegisterState::Active => {
                self.state = RegisterState::Summary;
                self.summarizes = Some(summarized_registers);
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot mark register as summary in state {:?}", self.state
            ))),
        }
    }
    
    /// Update the register contents
    pub fn update_contents(&mut self, contents: RegisterContents) -> Result<()> {
        match self.state {
            RegisterState::Active => {
                self.contents = contents;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot update register contents in state {:?}", self.state
            ))),
        }
    }
    
    /// Update register owner
    pub fn update_owner(&mut self, new_owner: Address) -> Result<()> {
        match self.state {
            RegisterState::Active => {
                self.owner = new_owner;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(Error::InvalidState(format!(
                "Cannot update register owner in state {:?}", self.state
            ))),
        }
    }
    
    /// Add a successor register ID
    pub fn add_successor(&mut self, successor_id: RegisterId) {
        self.successors.push(successor_id);
    }
    
    /// Add an operation to the history
    pub fn add_history_entry(&mut self, operation: &str) {
        self.history.push(operation.to_string());
    }
    
    /// Set the register's epoch
    pub fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }
    
    /// Set the created by transaction ID
    pub fn set_created_by_tx(&mut self, tx_id: &str) {
        self.created_by_tx = tx_id.to_string();
    }
    
    /// Set the last updated block height
    pub fn set_last_updated_height(&mut self, height: BlockHeight) {
        self.last_updated_height = height;
    }
    
    /// Add metadata to the register
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get metadata from the register
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Set the register's validity period
    pub fn set_validity(&mut self, start: u64, end: Option<u64>) {
        self.validity = TimeRange::new(start, end);
    }
    
    /// Check if the register is valid at a specific time
    pub fn is_valid_at(&self, time: u64) -> bool {
        self.validity.contains(time)
    }
    
    /// Generate a nullifier for this register
    pub fn generate_nullifier(&self, tx_id: &str, block_height: BlockHeight) -> RegisterNullifier {
        RegisterNullifier::new(self.register_id.clone(), tx_id.to_string(), block_height)
    }
    
    /// Update the last_updated timestamp to now
    fn update_timestamp(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
}

/// Service for creating and managing registers
pub trait RegisterService: Send + Sync {
    /// Create a new register
    async fn create_register(
        &self,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<RegisterId>;
    
    /// Get a register by ID
    async fn get_register(&self, register_id: &RegisterId) -> Result<Register>;
    
    /// Update a register
    async fn update_register(
        &self,
        register_id: &RegisterId,
        new_contents: RegisterContents,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<()>;
    
    /// Transfer register ownership
    async fn transfer_register(
        &self,
        register_id: &RegisterId,
        new_owner: Address,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<()>;
    
    /// Delete a register
    async fn delete_register(
        &self,
        register_id: &RegisterId,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<()>;
    
    /// Apply a register operation
    async fn apply_operation(
        &self,
        operation: RegisterOperation,
    ) -> Result<Vec<RegisterId>>;
    
    /// Verify a register operation
    async fn verify_operation(
        &self,
        operation: &RegisterOperation,
    ) -> Result<bool>;
    
    /// Get registers associated with an AST context
    async fn get_registers_by_ast_context(
        &self,
        ast_context: &AstContext,
    ) -> Result<Vec<Register>>;
    
    /// Get all registers owned by an address
    async fn get_registers_by_owner(
        &self,
        owner: &Address,
    ) -> Result<Vec<Register>>;
    
    /// Consume a register (mark it as one-time use)
    async fn consume_register(
        &self,
        register_id: &RegisterId,
        transaction_id: &str,
        successors: &[RegisterId],
        authorization: AuthorizationMethod,
    ) -> Result<()>;
    
    /// Lock a register
    async fn lock_register(
        &self,
        register_id: &RegisterId,
        reason: &str,
        authorization: AuthorizationMethod,
    ) -> Result<()>;
    
    /// Unlock a register
    async fn unlock_register(
        &self,
        register_id: &RegisterId,
        reason: &str,
        authorization: AuthorizationMethod,
    ) -> Result<()>;
}

/// Implementation of register operations aligned with ADR-006
pub mod operations {
    use super::*;
    use super::mock_zkp::register::{
        create::create_register_create_circuit,
        update::create_register_update_circuit,
        transfer::create_register_transfer_circuit,
        delete::create_register_delete_circuit,
        merge::create_register_merge_circuit,
        split::create_register_split_circuit,
    };
    
    /// Process a register operation with ZK verification
    /// 
    /// This function validates and applies a register operation,
    /// checking its proofs and generating appropriate new registers
    /// as needed. It follows the model defined in ADR-006.
    pub fn process_operation(
        operation: &RegisterOperation,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Verify operation validity
        let verification = verify_operation(operation, registers, current_block_height)?;
        if !verification.success {
            return Err(Error::InvalidOperation(verification.message));
        }
        
        // Process the operation based on its type
        match &operation.op_type {
            OperationType::CreateRegister => {
                create_register_operation(operation, registers, current_block_height, tx_id)
            }
            OperationType::UpdateRegister => {
                update_register_operation(operation, registers, current_block_height, tx_id)
            }
            OperationType::DeleteRegister => {
                delete_register_operation(operation, registers, current_block_height, tx_id)
            }
            OperationType::TransferOwnership(new_owner) => {
                transfer_ownership_operation(operation, new_owner, registers, current_block_height, tx_id)
            }
            OperationType::MergeRegisters => {
                merge_registers_operation(operation, registers, current_block_height, tx_id)
            }
            OperationType::SplitRegister => {
                split_register_operation(operation, registers, current_block_height, tx_id)
            }
            OperationType::CompositeOperation(ops) => {
                // For composite operations, process each sub-operation
                let mut result_ids = Vec::new();
                
                // In a real implementation, this would be atomic - either all succeed or all fail
                for op_type in ops {
                    let sub_op = RegisterOperation {
                        op_type: op_type.clone(),
                        registers: operation.registers.clone(),
                        new_contents: operation.new_contents.clone(),
                        authorization: operation.authorization.clone(),
                        proof: operation.proof.clone(),
                        resource_delta: operation.resource_delta.clone(),
                        ast_context: operation.ast_context.clone(),
                    };
                    
                    let ids = process_operation(&sub_op, registers, current_block_height, tx_id)?;
                    result_ids.extend(ids);
                }
                
                Ok(result_ids)
            }
        }
    }
    
    /// Verify a register operation
    /// 
    /// This function checks the validity of a register operation,
    /// including authorization, ZK proofs, and resource conservation.
    pub fn verify_operation(
        operation: &RegisterOperation,
        registers: &HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
    ) -> Result<VerificationResult> {
        // Check operation basics
        if let Err(e) = validate_operation_basics(operation) {
            return Ok(VerificationResult::failure(&format!("Invalid operation: {}", e)));
        }
        
        // Verify authorization
        if let Err(e) = verify_authorization(operation, registers) {
            return Ok(VerificationResult::failure(&format!("Authorization failed: {}", e)));
        }
        
        // Verify ZK proof if present
        if let Some(proof) = &operation.proof {
            if let Err(e) = verify_zk_proof(operation, proof, registers) {
                return Ok(VerificationResult::failure(&format!("Proof verification failed: {}", e)));
            }
        }
        
        // Verify resource conservation (if applicable)
        if let Err(e) = verify_resource_conservation(operation, registers) {
            return Ok(VerificationResult::failure(&format!("Resource conservation error: {}", e)));
        }
        
        // All verifications passed
        Ok(VerificationResult::success())
    }
    
    /// Validate basic operation structure and requirements
    fn validate_operation_basics(operation: &RegisterOperation) -> Result<()> {
        match &operation.op_type {
            OperationType::CreateRegister => {
                // CreateRegister requires new_contents
                if operation.new_contents.is_none() {
                    return Err(Error::InvalidArgument(
                        "CreateRegister operation requires new_contents".to_string()
                    ));
                }
            }
            OperationType::UpdateRegister => {
                // UpdateRegister requires exactly one register ID and new_contents
                if operation.registers.len() != 1 {
                    return Err(Error::InvalidArgument(
                        "UpdateRegister operation requires exactly one register ID".to_string()
                    ));
                }
                if operation.new_contents.is_none() {
                    return Err(Error::InvalidArgument(
                        "UpdateRegister operation requires new_contents".to_string()
                    ));
                }
            }
            OperationType::DeleteRegister => {
                // DeleteRegister requires exactly one register ID
                if operation.registers.len() != 1 {
                    return Err(Error::InvalidArgument(
                        "DeleteRegister operation requires exactly one register ID".to_string()
                    ));
                }
            }
            OperationType::TransferOwnership(_) => {
                // TransferOwnership requires exactly one register ID
                if operation.registers.len() != 1 {
                    return Err(Error::InvalidArgument(
                        "TransferOwnership operation requires exactly one register ID".to_string()
                    ));
                }
            }
            OperationType::MergeRegisters => {
                // MergeRegisters requires at least two register IDs and new_contents
                if operation.registers.len() < 2 {
                    return Err(Error::InvalidArgument(
                        "MergeRegisters operation requires at least two register IDs".to_string()
                    ));
                }
                if operation.new_contents.is_none() {
                    return Err(Error::InvalidArgument(
                        "MergeRegisters operation requires new_contents".to_string()
                    ));
                }
            }
            OperationType::SplitRegister => {
                // SplitRegister requires exactly one register ID and new_contents
                if operation.registers.len() != 1 {
                    return Err(Error::InvalidArgument(
                        "SplitRegister operation requires exactly one register ID".to_string()
                    ));
                }
                if operation.new_contents.is_none() {
                    return Err(Error::InvalidArgument(
                        "SplitRegister operation requires new_contents (for the resulting registers)".to_string()
                    ));
                }
            }
            OperationType::CompositeOperation(ops) => {
                // CompositeOperation requires at least one sub-operation
                if ops.is_empty() {
                    return Err(Error::InvalidArgument(
                        "CompositeOperation requires at least one sub-operation".to_string()
                    ));
                }
                // Each sub-operation should be valid
                for op in ops {
                    let sub_op = RegisterOperation {
                        op_type: op.clone(),
                        registers: operation.registers.clone(),
                        new_contents: operation.new_contents.clone(),
                        authorization: operation.authorization.clone(),
                        proof: operation.proof.clone(),
                        resource_delta: operation.resource_delta.clone(),
                        ast_context: operation.ast_context.clone(),
                    };
                    validate_operation_basics(&sub_op)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Verify authorization for an operation
    fn verify_authorization(
        operation: &RegisterOperation,
        registers: &HashMap<RegisterId, Register>,
    ) -> Result<()> {
        // In a complete implementation, this would check signatures, permissions, etc.
        // based on the authorization method in the operation.
        
        // For now, we'll implement a basic authorization check
        if !operation.registers.is_empty() {
            // For operations on existing registers, check that all affected registers 
            // are owned by the same address specified in the authorization
            for reg_id in &operation.registers {
                if let Some(register) = registers.get(reg_id) {
                    // Skip checks for CreateRegister operations
                    if matches!(operation.op_type, OperationType::CreateRegister) {
                        continue;
                    }
                    
                    // Check if register is active
                    if !register.state.is_active() {
                        return Err(Error::InvalidOperation(format!(
                            "Register {} is not active", reg_id
                        )));
                    }
                    
                    // Check if authorization matches based on the method
                    match &operation.authorization {
                        AuthorizationMethod::OwnerSigned(owner) => {
                            if *owner != register.owner {
                                return Err(Error::Unauthorized(format!(
                                    "Owner mismatch for register {}", reg_id
                                )));
                            }
                        }
                        AuthorizationMethod::ZKProofAuthorization { .. } => {
                            // ZK Proof verification would be handled separately by the verify_zk_proof function
                            // For now, we'll assume it's valid if the proof is present
                        }
                        AuthorizationMethod::TokenOwnershipAuthorization { owner, .. } => {
                            // Check if the token owner is the register owner
                            if *owner != register.owner {
                                return Err(Error::Unauthorized(format!(
                                    "Token owner doesn't match register owner for register {}", reg_id
                                )));
                            }
                            // Additional checks for token balance would happen in a real implementation
                        }
                        AuthorizationMethod::MultiSigAuthorization { addresses, threshold, .. } => {
                            // Check if the register owner is in the list of authorized addresses
                            if !addresses.contains(&register.owner) {
                                return Err(Error::Unauthorized(format!(
                                    "Register owner not in multi-sig list for register {}", reg_id
                                )));
                            }
                            // In a real implementation, we would verify the signatures
                        }
                        AuthorizationMethod::TimeLockedAuthorization { method, unlock_time } => {
                            // Check if the timelock has expired
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64;
                                
                            if now < *unlock_time {
                                return Err(Error::Unauthorized(format!(
                                    "Time-locked authorization has not yet expired for register {}", reg_id
                                )));
                            }
                            
                            // Recursively verify the underlying method
                            // This is a simplified version; in a real implementation, we'd create 
                            // a new operation with the inner method for verification
                        }
                        AuthorizationMethod::DAOAuthorization { .. } => {
                            // DAO authorization would require complex verification
                            // For now, we'll assume it's valid if present
                        }
                        AuthorizationMethod::Any(methods) => {
                            // With Any combinator, we would check if any method passes
                            // For now, we'll assume it's valid if at least one method is provided
                            if methods.is_empty() {
                                return Err(Error::Unauthorized(
                                    "Any authorization requires at least one method".to_string()
                                ));
                            }
                        }
                        AuthorizationMethod::Threshold(threshold, methods) => {
                            // With Threshold combinator, we would check if enough methods pass
                            // For now, we'll just check if there are enough methods
                            if methods.len() < *threshold {
                                return Err(Error::Unauthorized(format!(
                                    "Threshold authorization requires at least {} methods, but only {} provided",
                                    threshold, methods.len()
                                )));
                            }
                        }
                        // Other authorization methods would have their own checks
                        _ => {
                            // For methods we haven't implemented detailed checks for,
                            // we'll assume they're valid for now
                        }
                    }
                } else {
                    return Err(Error::NotFound(format!(
                        "Register {} not found", reg_id
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// Verify ZK proof for an operation
    fn verify_zk_proof(
        operation: &RegisterOperation,
        proof: &[u8],
        registers: &HashMap<RegisterId, Register>,
    ) -> Result<()> {
        // Create public inputs based on the operation type
        let public_inputs = create_public_inputs(operation, registers)?;
        
        // Get the appropriate circuit based on operation type
        let circuit = get_circuit_for_operation(&operation.op_type)?;
        
        // Create a proof object
        let proof_obj = Proof {
            proof_data: proof.to_vec(),
            proof_system: ProofSystem::Groth16, // Use Groth16 as default system
        };
        
        // Get the verification key (in a real implementation, this would be stored/retrieved)
        let verification_key = vec![]; // Placeholder for actual verification key
        
        // Verify the proof
        // In a real implementation, this would be:
        // let verification_result = circuit.verify(&proof_obj, &public_inputs, &verification_key).await?;
        // if !verification_result {
        //     return Err(Error::InvalidProof("ZK proof verification failed".to_string()));
        // }
        
        // For now, we'll simulate verification by checking for empty proof
        if proof.is_empty() {
            return Err(Error::InvalidOperation("Empty ZK proof".to_string()));
        }
        
        Ok(())
    }
    
    /// Verify resource conservation for an operation
    fn verify_resource_conservation(
        operation: &RegisterOperation,
        registers: &HashMap<RegisterId, Register>,
    ) -> Result<()> {
        // This checks that resources are properly conserved in the operation
        // For example, token balances should sum to the same before and after
        
        // In a real implementation, this would analyze the register contents
        // to ensure that resources are conserved across the operation
        
        Ok(())
    }
    
    /// Create public inputs for ZK proof verification
    fn create_public_inputs(
        operation: &RegisterOperation,
        registers: &HashMap<RegisterId, Register>,
    ) -> Result<PublicInputs> {
        let mut public_inputs = PublicInputs::new();
        
        match &operation.op_type {
            OperationType::CreateRegister => {
                // Get owner from authorization
                let owner = match &operation.authorization {
                    AuthorizationMethod::OwnerSigned(addr) => addr.clone(),
                    _ => return Err(Error::InvalidArgument(
                        "CreateRegister operation requires OwnerSigned authorization".to_string()
                    )),
                };
                
                // Get contents
                let contents = operation.new_contents.as_ref().ok_or_else(|| 
                    Error::InvalidArgument("CreateRegister requires new_contents".to_string())
                )?;
                
                // Simulate register hash calculation
                let register_hash = format!("hash-{}", rand::random::<u64>());
                
                // Add public inputs
                public_inputs.insert("register_hash".to_string(), register_hash);
                public_inputs.insert("owner_address".to_string(), owner);
                public_inputs.insert("domain".to_string(), "default".to_string()); // Default domain
            }
            OperationType::UpdateRegister => {
                // Get register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("UpdateRegister requires a register ID".to_string())
                )?;
                
                // Get register
                let register = registers.get(register_id).ok_or_else(|| 
                    Error::NotFound(format!("Register {} not found", register_id))
                )?;
                
                // Get new contents
                let new_contents = operation.new_contents.as_ref().ok_or_else(|| 
                    Error::InvalidArgument("UpdateRegister operation requires new_contents".to_string())
                )?;
                
                // Simulate register hash calculations
                let old_register_hash = register.content_hash();
                let new_register_hash = format!("hash-{}", rand::random::<u64>());
                
                // Add public inputs
                public_inputs.insert("old_register_hash".to_string(), old_register_hash);
                public_inputs.insert("new_register_hash".to_string(), new_register_hash);
                public_inputs.insert("old_register_id".to_string(), register_id.to_string());
                public_inputs.insert("new_register_id".to_string(), register_id.to_string());
                public_inputs.insert("owner_address".to_string(), register.owner.clone());
                public_inputs.insert("domain".to_string(), register.domain.clone());
            }
            OperationType::DeleteRegister => {
                // Get register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("DeleteRegister requires a register ID".to_string())
                )?;
                
                // Get register
                let register = registers.get(register_id).ok_or_else(|| 
                    Error::NotFound(format!("Register {} not found", register_id))
                )?;
                
                // Simulate register hash and nullifier
                let register_hash = register.content_hash();
                let nullifier = format!("nullifier-{}", rand::random::<u64>());
                
                // Add public inputs
                public_inputs.insert("register_hash".to_string(), register_hash);
                public_inputs.insert("register_id".to_string(), register_id.to_string());
                public_inputs.insert("owner_address".to_string(), register.owner.clone());
                public_inputs.insert("domain".to_string(), register.domain.clone());
                public_inputs.insert("nullifier".to_string(), nullifier);
            }
            OperationType::TransferOwnership(new_owner) => {
                // Get register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("TransferOwnership requires a register ID".to_string())
                )?;
                
                // Get register
                let register = registers.get(register_id).ok_or_else(|| 
                    Error::NotFound(format!("Register {} not found", register_id))
                )?;
                
                // Simulate register hash calculations
                let old_register_hash = register.content_hash();
                let new_register_hash = format!("hash-{}", rand::random::<u64>());
                
                // Add public inputs
                public_inputs.insert("old_register_hash".to_string(), old_register_hash);
                public_inputs.insert("new_register_hash".to_string(), new_register_hash);
                public_inputs.insert("old_register_id".to_string(), register_id.to_string());
                public_inputs.insert("new_register_id".to_string(), register_id.to_string());
                public_inputs.insert("old_owner_address".to_string(), register.owner.clone());
                public_inputs.insert("new_owner_address".to_string(), new_owner.clone());
                public_inputs.insert("domain".to_string(), register.domain.clone());
            }
            OperationType::MergeRegisters => {
                // Vector for input register hashes
                let mut input_register_hashes = Vec::new();
                let mut nullifiers = Vec::new();
                
                // Get owner and domain from first register
                let first_register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("MergeRegisters requires at least one register ID".to_string())
                )?;
                
                let first_register = registers.get(first_register_id).ok_or_else(|| 
                    Error::NotFound(format!("Register {} not found", first_register_id))
                )?;
                
                let owner = first_register.owner.clone();
                let domain = first_register.domain.clone();
                
                // Process each input register
                for reg_id in &operation.registers {
                    let register = registers.get(reg_id).ok_or_else(|| 
                        Error::NotFound(format!("Register {} not found", reg_id))
                    )?;
                    
                    // Calculate hash and nullifier
                    let register_hash = register.content_hash();
                    let nullifier = format!("nullifier-{}", rand::random::<u64>());
                    
                    input_register_hashes.push(register_hash);
                    nullifiers.push(nullifier);
                }
                
                // Simulate output register hash
                let output_register_hash = format!("hash-{}", rand::random::<u64>());
                
                // Add public inputs
                public_inputs.insert("input_register_hashes".to_string(), 
                                   serde_json::to_string(&input_register_hashes).unwrap());
                public_inputs.insert("output_register_hash".to_string(), output_register_hash);
                public_inputs.insert("nullifiers".to_string(),
                                   serde_json::to_string(&nullifiers).unwrap());
                public_inputs.insert("owner_address".to_string(), owner);
                public_inputs.insert("domain".to_string(), domain);
            }
            OperationType::SplitRegister => {
                // Get register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("SplitRegister requires a register ID".to_string())
                )?;
                
                // Get register
                let register = registers.get(register_id).ok_or_else(|| 
                    Error::NotFound(format!("Register {} not found", register_id))
                )?;
                
                // Get split contents
                let split_contents = match operation.new_contents.as_ref() {
                    Some(RegisterContents::CompositeContents(contents)) => contents,
                    _ => return Err(Error::InvalidArgument(
                        "SplitRegister operation requires CompositeContents".to_string()
                    )),
                };
                
                // Simulate register hash calculations
                let input_register_hash = register.content_hash();
                let nullifier = format!("nullifier-{}", rand::random::<u64>());
                
                // Simulate output register hashes
                let mut output_register_hashes = Vec::new();
                for _ in split_contents {
                    output_register_hashes.push(format!("hash-{}", rand::random::<u64>()));
                }
                
                // Add public inputs
                public_inputs.insert("input_register_hash".to_string(), input_register_hash);
                public_inputs.insert("output_register_hashes".to_string(),
                                   serde_json::to_string(&output_register_hashes).unwrap());
                public_inputs.insert("nullifier".to_string(), nullifier);
                public_inputs.insert("owner_address".to_string(), register.owner.clone());
                public_inputs.insert("domain".to_string(), register.domain.clone());
            }
            OperationType::CompositeOperation(_) => {
                // For composite operations, we'd need to create inputs for each sub-operation
                // This is a simplified version that just adds a flag
                public_inputs.insert("is_composite".to_string(), "true".to_string());
            }
        }
        
        Ok(public_inputs)
    }
    
    /// Get the appropriate ZK circuit for a register operation
    fn get_circuit_for_operation(op_type: &OperationType) -> Result<Arc<dyn Circuit>> {
        match op_type {
            OperationType::CreateRegister => Ok(create_register_create_circuit()),
            OperationType::UpdateRegister => Ok(create_register_update_circuit()),
            OperationType::DeleteRegister => Ok(create_register_delete_circuit()),
            OperationType::TransferOwnership(_) => Ok(create_register_transfer_circuit()),
            OperationType::MergeRegisters => Ok(create_register_merge_circuit()),
            OperationType::SplitRegister => Ok(create_register_split_circuit()),
            OperationType::CompositeOperation(_) => {
                // For composite operations, we'd need a more complex handling
                // For now, just return an error
                Err(Error::NotImplemented("Composite operation circuits not supported yet".to_string()))
            }
        }
    }
    
    /// Process a create register operation
    fn create_register_operation(
        operation: &RegisterOperation,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Get the owner from the authorization
        let owner = match &operation.authorization {
            AuthorizationMethod::OwnerSigned(addr) => addr.clone(),
            _ => return Err(Error::InvalidArgument(
                "CreateRegister operation requires OwnerSigned authorization".to_string()
            )),
        };
        
        // Get the domain (for now we'll use a default)
        let domain = "default".to_string();
        
        // Get the new contents
        let contents = operation.new_contents.as_ref().ok_or_else(|| 
            Error::InvalidArgument("CreateRegister operation requires new_contents".to_string())
        )?;
        
        // Create a new register with a random ID
        let register = Register::new_with_random_id(
            owner,
            domain,
            contents.clone(),
            None,
        );
        
        // Store the register
        let register_id = register.register_id.clone();
        registers.insert(register_id.clone(), register);
        
        Ok(vec![register_id])
    }
    
    /// Process an update register operation
    fn update_register_operation(
        operation: &RegisterOperation,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Get the register ID
        let register_id = operation.registers.first().ok_or_else(|| 
            Error::InvalidArgument("UpdateRegister operation requires a register ID".to_string())
        )?;
        
        // Get the new contents
        let new_contents = operation.new_contents.as_ref().ok_or_else(|| 
            Error::InvalidArgument("UpdateRegister operation requires new_contents".to_string())
        )?;
        
        // Get the register
        let register = registers.get_mut(register_id).ok_or_else(|| 
            Error::NotFound(format!("Register {} not found", register_id))
        )?;
        
        // Following the one-time use pattern from ADR-006:
        // 1. Consume the current register
        let nullifier = register.consume(tx_id, current_block_height)?;
        
        // 2. Create a new register with the updated contents
        let new_register = Register::new_with_random_id(
            register.owner.clone(),
            register.domain.clone(),
            new_contents.clone(),
            None,
        );
        
        // Add the new register
        let new_register_id = new_register.register_id.clone();
        registers.insert(new_register_id.clone(), new_register);
        
        // Record the successor in the consumed register
        register.successors.push(new_register_id.clone());
        
        Ok(vec![new_register_id])
    }
    
    /// Process a delete register operation
    fn delete_register_operation(
        operation: &RegisterOperation,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Get the register ID
        let register_id = operation.registers.first().ok_or_else(|| 
            Error::InvalidArgument("DeleteRegister operation requires a register ID".to_string())
        )?;
        
        // Get the register
        let register = registers.get_mut(register_id).ok_or_else(|| 
            Error::NotFound(format!("Register {} not found", register_id))
        )?;
        
        // Consume the register (marking it as consumed)
        register.consume(tx_id, current_block_height)?;
        
        // No new registers created during deletion
        Ok(vec![])
    }
    
    /// Process a transfer ownership operation
    fn transfer_ownership_operation(
        operation: &RegisterOperation,
        new_owner: &Address,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Get the register ID
        let register_id = operation.registers.first().ok_or_else(|| 
            Error::InvalidArgument("TransferOwnership operation requires a register ID".to_string())
        )?;
        
        // Get the register
        let register = registers.get_mut(register_id).ok_or_else(|| 
            Error::NotFound(format!("Register {} not found", register_id))
        )?;
        
        // Following the one-time use pattern from ADR-006:
        // 1. Consume the current register
        let nullifier = register.consume(tx_id, current_block_height)?;
        
        // 2. Create a new register with the new owner
        let new_register = Register::new_with_random_id(
            new_owner.clone(),
            register.domain.clone(),
            register.contents.clone(),
            None,
        );
        
        // Add the new register
        let new_register_id = new_register.register_id.clone();
        registers.insert(new_register_id.clone(), new_register);
        
        // Record the successor in the consumed register
        register.successors.push(new_register_id.clone());
        
        Ok(vec![new_register_id])
    }
    
    /// Process a merge registers operation
    fn merge_registers_operation(
        operation: &RegisterOperation,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Check if we have at least two registers
        if operation.registers.len() < 2 {
            return Err(Error::InvalidArgument(
                "MergeRegisters operation requires at least two register IDs".to_string()
            ));
        }
        
        // Get the new contents
        let new_contents = operation.new_contents.as_ref().ok_or_else(|| 
            Error::InvalidArgument("MergeRegisters operation requires new_contents".to_string())
        )?;
        
        // Assume all registers have the same owner and domain
        // (a real implementation would verify this more carefully)
        let first_register_id = &operation.registers[0];
        let first_register = registers.get(first_register_id).ok_or_else(|| 
            Error::NotFound(format!("Register {} not found", first_register_id))
        )?;
        
        let owner = first_register.owner.clone();
        let domain = first_register.domain.clone();
        let epoch = first_register.epoch;
        
        // Create a new register with the merged contents
        let mut new_register = Register::new_with_random_id(
            owner,
            domain,
            new_contents.clone(),
            None,
        );
        
        // Set epoch and transaction ID
        new_register.set_epoch(epoch);
        new_register.set_created_by_tx(tx_id);
        
        // Add the new register
        let new_register_id = new_register.register_id.clone();
        registers.insert(new_register_id.clone(), new_register);
        
        // Consume all the source registers
        for reg_id in &operation.registers {
            if let Some(register) = registers.get_mut(reg_id) {
                register.consume(tx_id, current_block_height)?;
            } else {
                return Err(Error::NotFound(format!("Register {} not found", reg_id)));
            }
        }
        
        Ok(vec![new_register_id])
    }
    
    /// Process a split register operation
    fn split_register_operation(
        operation: &RegisterOperation,
        registers: &mut HashMap<RegisterId, Register>,
        current_block_height: BlockHeight,
        tx_id: &str,
    ) -> Result<Vec<RegisterId>> {
        // Get the register ID
        let register_id = operation.registers.first().ok_or_else(|| 
            Error::InvalidArgument("SplitRegister operation requires a register ID".to_string())
        )?;
        
        // Get the register
        let register = registers.get_mut(register_id).ok_or_else(|| 
            Error::NotFound(format!("Register {} not found", register_id))
        )?;
        
        // Get the new contents (for split registers, this should be a composite)
        let split_contents = match operation.new_contents.as_ref() {
            Some(RegisterContents::CompositeContents(contents)) => contents,
            _ => return Err(Error::InvalidArgument(
                "SplitRegister operation requires CompositeContents".to_string()
            )),
        };
        
        // Create new registers for each of the split contents
        let mut new_register_ids = Vec::new();
        
        for content in split_contents {
            let new_register = Register::new_with_random_id(
                register.owner.clone(),
                register.domain.clone(),
                content.clone(),
                None,
            );
            
            let new_register_id = new_register.register_id.clone();
            registers.insert(new_register_id.clone(), new_register);
            new_register_ids.push(new_register_id);
        }
        
        // Consume the original register
        register.consume(tx_id, current_block_height)?;
        
        Ok(new_register_ids)
    }
} 