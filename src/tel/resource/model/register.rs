// Register-based resource model for TEL
//
// This module implements the register-based resource model as defined in ADR 003.
// It provides the core data structures for resource representation and state tracking.

use std::fmt;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::tel::error::{TelError, TelResult};
use crate::tel::types::{ResourceId, Address, Domain, Metadata, OperationId};

/// A globally unique identifier for a register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegisterId(pub Uuid);

impl RegisterId {
    /// Create a new random register ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create a register ID from a string
    pub fn from_str(s: &str) -> TelResult<Self> {
        let uuid = Uuid::parse_str(s).map_err(|e| 
            TelError::ParseError(format!("Invalid register ID: {}", e)))?;
        Ok(Self(uuid))
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

/// State of a register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterState {
    /// Register is active and can be used
    Active,
    /// Register is locked and cannot be modified
    Locked,
    /// Register is frozen and cannot be used
    Frozen,
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
    /// Resource data
    Resource(Box<Resource>),
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
    
    /// Create new resource contents
    pub fn resource(resource: Resource) -> Self {
        Self::Resource(Box::new(resource))
    }
    
    /// Create empty contents
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// Get size of contents in bytes
    pub fn size(&self) -> usize {
        match self {
            Self::Binary(data) => data.len(),
            Self::String(data) => data.len(),
            Self::Json(data) => serde_json::to_string(data).unwrap_or_default().len(),
            Self::Resource(_) => 0, // Size is calculated separately
            Self::Empty => 0,
        }
    }
}

/// A label for tracking resources across domains and time
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControllerLabel {
    /// Domain of the controller
    pub domain: Domain,
    /// ID of the controller
    pub controller_id: String,
    /// Timestamp when the label was created
    pub created_at: u64,
    /// Optional execution context
    pub context: Option<String>,
}

impl ControllerLabel {
    /// Create a new controller label
    pub fn new(domain: Domain, controller_id: String, context: Option<String>) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            domain,
            controller_id,
            created_at,
            context,
        }
    }
}

/// Time range for resource validity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (inclusive)
    pub start: u64,
    /// End time (exclusive)
    pub end: Option<u64>,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: u64, end: Option<u64>) -> Self {
        Self { start, end }
    }
    
    /// Create a new time range from now
    pub fn from_now(duration_ms: Option<u64>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        let end = duration_ms.map(|d| now + d);
        
        Self { start: now, end }
    }
    
    /// Check if a time is within this range
    pub fn contains(&self, time: u64) -> bool {
        time >= self.start && self.end.map_or(true, |end| time < end)
    }
    
    /// Check if this range is expired
    pub fn is_expired(&self) -> bool {
        if let Some(end) = self.end {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
                
            end <= now
        } else {
            false
        }
    }
}

/// Time-related data for a resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceTimeData {
    /// Time range for validity
    pub validity: TimeRange,
    /// Time when the resource was created
    pub created_at: u64,
    /// Time when the resource was last updated
    pub updated_at: u64,
    /// Epoch when the resource was created
    pub epoch: u64,
}

impl ResourceTimeData {
    /// Create new time data at the current time
    pub fn now(epoch: u64, validity_duration: Option<u64>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            validity: TimeRange::new(now, validity_duration.map(|d| now + d)),
            created_at: now,
            updated_at: now,
            epoch,
        }
    }
    
    /// Update the time data
    pub fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        self.updated_at = now;
    }
    
    /// Check if the resource is valid
    pub fn is_valid(&self) -> bool {
        !self.validity.is_expired()
    }
}

/// Type of resource logic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceLogicType {
    /// Normal resource with no special logic
    Normal,
    /// Fungible resource
    Fungible,
    /// Non-fungible resource
    NonFungible,
    /// Executable resource
    Executable,
    /// Custom resource logic
    Custom(u16),
}

/// Resource logic interface
pub trait ResourceLogic: Send + Sync + fmt::Debug {
    /// Get the type of resource logic
    fn logic_type(&self) -> ResourceLogicType;
    
    /// Validate a mutation on the resource
    fn validate_mutation(&self, resource: &Resource, new_contents: &RegisterContents) -> TelResult<()>;
    
    /// Process a transfer of the resource
    fn process_transfer(&self, resource: &Resource, from: &Address, to: &Address) -> TelResult<RegisterContents>;
}

/// A high-level resource abstraction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for the resource
    pub id: ResourceId,
    /// Type of the resource
    pub resource_type: String,
    /// Version of the resource
    pub version: String,
    /// Data contained in the resource
    pub data: serde_json::Value,
    /// Type of resource logic
    pub logic_type: ResourceLogicType,
    /// Time-related data
    pub time_data: ResourceTimeData,
    /// Controller label for cross-domain tracking
    pub controller: Option<ControllerLabel>,
    /// Additional metadata
    pub metadata: Metadata,
}

impl Resource {
    /// Create a new resource
    pub fn new(
        id: ResourceId,
        resource_type: String,
        version: String,
        data: serde_json::Value,
        logic_type: ResourceLogicType,
        epoch: u64,
        controller: Option<ControllerLabel>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            id,
            resource_type,
            version,
            data,
            logic_type,
            time_data: ResourceTimeData::now(epoch, None),
            controller,
            metadata: metadata.unwrap_or_else(|| {
                let mut m = HashMap::new();
                m.insert("created".to_string(), serde_json::Value::String(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                        .to_string()
                ));
                m
            }),
        }
    }
    
    /// Check if the resource is valid
    pub fn is_valid(&self) -> bool {
        self.time_data.is_valid()
    }
    
    /// Set the validity period for the resource
    pub fn set_validity(&mut self, duration_ms: Option<u64>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        self.time_data.validity = TimeRange::new(now, duration_ms.map(|d| now + d));
    }
    
    /// Update the resource data
    pub fn update_data(&mut self, data: serde_json::Value) {
        self.data = data;
        self.time_data.update();
    }
    
    /// Add metadata to the resource
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
}

/// A register in the TEL resource system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Register {
    /// Unique identifier for the register
    pub id: RegisterId,
    /// State of the register
    pub state: RegisterState,
    /// Owner of the register
    pub owner: Address,
    /// Domain of the register
    pub domain: Domain,
    /// Contents of the register
    pub contents: RegisterContents,
    /// Time when the register was created
    pub created_at: u64,
    /// Time when the register was last updated
    pub updated_at: u64,
    /// Epoch when the register was created
    pub epoch: u64,
    /// History of operations on the register
    pub history: Vec<OperationId>,
    /// Additional metadata
    pub metadata: Metadata,
}

impl Register {
    /// Create a new register
    pub fn new(
        id: RegisterId,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        epoch: u64,
        metadata: Option<Metadata>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            id,
            state: RegisterState::Active,
            owner,
            domain,
            contents,
            created_at: now,
            updated_at: now,
            epoch,
            history: Vec::new(),
            metadata: metadata.unwrap_or_else(|| {
                let mut m = HashMap::new();
                m.insert("created".to_string(), serde_json::Value::String(now.to_string()));
                m
            }),
        }
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
    
    /// Check if the register is pending deletion
    pub fn is_pending_deletion(&self) -> bool {
        self.state.is_pending_deletion()
    }
    
    /// Check if the register is a tombstone
    pub fn is_tombstone(&self) -> bool {
        self.state.is_tombstone()
    }
    
    /// Lock the register
    pub fn lock(&mut self) -> TelResult<()> {
        match self.state {
            RegisterState::Active => {
                self.state = RegisterState::Locked;
                self.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                Ok(())
            }
            _ => Err(TelError::ResourceError(format!(
                "Cannot lock register in state {:?}", self.state
            ))),
        }
    }
    
    /// Unlock the register
    pub fn unlock(&mut self) -> TelResult<()> {
        match self.state {
            RegisterState::Locked => {
                self.state = RegisterState::Active;
                self.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                Ok(())
            }
            _ => Err(TelError::ResourceError(format!(
                "Cannot unlock register in state {:?}", self.state
            ))),
        }
    }
    
    /// Mark the register for deletion
    pub fn mark_for_deletion(&mut self) -> TelResult<()> {
        match self.state {
            RegisterState::Active | RegisterState::Locked | RegisterState::Frozen => {
                self.state = RegisterState::PendingDeletion;
                self.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                Ok(())
            }
            _ => Err(TelError::ResourceError(format!(
                "Cannot mark register for deletion in state {:?}", self.state
            ))),
        }
    }
    
    /// Convert the register to a tombstone
    pub fn to_tombstone(&mut self) -> TelResult<()> {
        match self.state {
            RegisterState::PendingDeletion => {
                self.state = RegisterState::Tombstone;
                self.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                // Clear contents to save space
                self.contents = RegisterContents::Empty;
                Ok(())
            }
            _ => Err(TelError::ResourceError(format!(
                "Cannot convert register to tombstone in state {:?}", self.state
            ))),
        }
    }
    
    /// Add an operation to the history
    pub fn add_operation(&mut self, operation_id: OperationId) {
        self.history.push(operation_id);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Update the register contents
    pub fn update_contents(&mut self, contents: RegisterContents) -> TelResult<()> {
        if !self.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot update register in state {:?}", self.state
            )));
        }
        
        self.contents = contents;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
    
    /// Add metadata to the register
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
    
    /// Get the age of the register in milliseconds
    pub fn age(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        now - self.created_at
    }
    
    /// Transfer ownership of the register
    pub fn transfer(&mut self, new_owner: Address) -> TelResult<()> {
        if !self.is_active() {
            return Err(TelError::ResourceError(format!(
                "Cannot transfer register in state {:?}", self.state
            )));
        }
        
        self.owner = new_owner;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Ok(())
    }
} 