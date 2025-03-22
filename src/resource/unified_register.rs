// Unified Register Implementation
//
// This module provides a unified register implementation that combines
// the functionality from the TEL resource model and the ZK register system.
// It serves as the foundation for the unified register API.

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::{Address, Domain, BlockHeight};

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

/// A valid time range for a register
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (milliseconds since epoch)
    pub start: u64,
    /// Optional end time (milliseconds since epoch)
    pub end: Option<u64>,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: u64, end: Option<u64>) -> Self {
        Self { start, end }
    }
    
    /// Create a time range starting from now
    pub fn from_now(duration_ms: Option<u64>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self::new(now, duration_ms.map(|d| now + d))
    }
    
    /// Check if a time is within this range
    pub fn contains(&self, time: u64) -> bool {
        time >= self.start && self.end.map_or(true, |end| time <= end)
    }
    
    /// Check if this range is valid
    pub fn is_valid(&self) -> bool {
        self.end.map_or(true, |end| end >= self.start)
    }
    
    /// Get the duration of this range in milliseconds
    pub fn duration(&self) -> Option<u64> {
        self.end.map(|end| end.saturating_sub(self.start))
    }
}

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
    Json(String),
    /// Empty register
    Empty,
}

impl RegisterContents {
    /// Create empty register contents
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// Create register contents from binary data
    pub fn with_binary(data: Vec<u8>) -> Self {
        Self::Binary(data)
    }
    
    /// Create register contents from a string
    pub fn with_string(data: &str) -> Self {
        Self::String(data.to_string())
    }
    
    /// Create register contents from JSON
    pub fn with_json(json: &str) -> Self {
        Self::Json(json.to_string())
    }
    
    /// Get the binary data if this contains binary
    pub fn as_binary(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Binary(data) => Some(data),
            _ => None,
        }
    }
    
    /// Get the string data if this contains a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(data) => Some(data),
            _ => None,
        }
    }
    
    /// Get the JSON data if this contains JSON
    pub fn as_json(&self) -> Option<&str> {
        match self {
            Self::Json(data) => Some(data),
            _ => None,
        }
    }
    
    /// Get a string representation of the contents
    pub fn to_string(&self) -> String {
        match self {
            Self::Binary(data) => format!("<binary data: {} bytes>", data.len()),
            Self::String(data) => data.clone(),
            Self::Json(data) => data.clone(),
            Self::Empty => "<empty>".to_string(),
        }
    }
}

impl Default for RegisterContents {
    fn default() -> Self {
        Self::Empty
    }
}

/// A register nullifier for one-time use registers
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
    pub fn new(register_id: RegisterId, transaction_id: &str, block_height: BlockHeight) -> Self {
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
            transaction_id: transaction_id.to_string(),
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

/// Type alias for register metadata
pub type Metadata = HashMap<String, String>;

/// The unified register data structure
/// 
/// This combines functionality from both the TEL register and the ZK register systems.
/// As defined in ADR-006, registers are one-time use atomic storage units
/// that can hold various types of content and are used to model state
/// transitions in the system.
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
                    tx_id,
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
        RegisterNullifier::new(self.register_id.clone(), tx_id, block_height)
    }
    
    /// Update the last_updated timestamp to now
    fn update_timestamp(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_creation() {
        let owner = "owner".to_string();
        let domain = "domain".to_string();
        let contents = RegisterContents::with_string("Hello, world!");
        
        let register = Register::new_with_random_id(
            owner.clone(),
            domain.clone(),
            contents.clone(),
            None,
        );
        
        assert_eq!(register.owner, owner);
        assert_eq!(register.domain, domain);
        assert_eq!(register.contents, contents);
        assert_eq!(register.state, RegisterState::Active);
        assert!(register.is_active());
    }
    
    #[test]
    fn test_register_state_transitions() {
        let mut register = Register::new_with_random_id(
            "owner".to_string(),
            "domain".to_string(),
            RegisterContents::with_string("Hello, world!"),
            None,
        );
        
        // Test locking
        register.lock().unwrap();
        assert!(register.is_locked());
        
        // Test unlocking
        register.unlock().unwrap();
        assert!(register.is_active());
        
        // Test freezing
        register.freeze().unwrap();
        assert!(register.is_frozen());
        
        // Test unfreezing
        register.unfreeze().unwrap();
        assert!(register.is_active());
        
        // Test consuming
        let nullifier = register.consume("tx123", 42).unwrap();
        assert!(register.is_consumed());
        assert_eq!(register.consumed_by_tx, Some("tx123".to_string()));
        assert_eq!(register.last_updated_height, 42);
        assert_eq!(nullifier.register_id, register.register_id);
        assert_eq!(nullifier.transaction_id, "tx123");
        assert_eq!(nullifier.block_height, 42);
    }
    
    #[test]
    fn test_deterministic_register_id() {
        let id1 = RegisterId::deterministic("register", "test1");
        let id2 = RegisterId::deterministic("register", "test1");
        let id3 = RegisterId::deterministic("register", "test2");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_register_contents() {
        let binary = RegisterContents::with_binary(vec![1, 2, 3]);
        let string = RegisterContents::with_string("Hello");
        let json = RegisterContents::with_json(r#"{"key": "value"}"#);
        let empty = RegisterContents::empty();
        
        assert_eq!(binary.as_binary(), Some(&vec![1, 2, 3]));
        assert_eq!(string.as_string(), Some("Hello"));
        assert_eq!(json.as_json(), Some(r#"{"key": "value"}"#));
        
        assert!(binary.as_string().is_none());
        assert!(string.as_binary().is_none());
        assert!(empty.as_binary().is_none());
    }
    
    #[test]
    fn test_time_range() {
        let range = TimeRange::new(100, Some(200));
        
        assert!(range.contains(100));
        assert!(range.contains(150));
        assert!(range.contains(200));
        assert!(!range.contains(99));
        assert!(!range.contains(201));
        
        assert!(range.is_valid());
        assert_eq!(range.duration(), Some(100));
        
        let infinite = TimeRange::new(100, None);
        assert!(infinite.contains(100));
        assert!(infinite.contains(u64::MAX));
        assert_eq!(infinite.duration(), None);
    }
} 