// Register-based resource model for TEL
//
// This module implements the register-based resource model as defined in ADR 003.
// It provides the core data structures for resource representation and state tracking.
//
// MIGRATION NOTE: This file is being migrated to use ResourceRegister instead of
// separate Resource and Register abstractions, as part of the unification process.

use std::{
    collections::HashMap,
    fmt,
};

use crypto::{ContentId, Hasher};
use resource::{
    ResourceRegister, StateVisibility, UnifiedRegisterState, UnifiedResourceLogic,
    migrate_helpers,
};
use serde::{Serialize, Deserialize};
use serde_json;
use tel::common::{Domain, Address, TelResult, TelError, Metadata};
use tel::crypto::OperationId;
use borsh::{BorshSerialize, BorshDeserialize};
use crate::crypto::{
    hash::{HashError, HashFactory, HashOutput},
    ContentAddressed,
};
use crate::tel::types::{ResourceId};

/// A globally unique identifier for a register
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RegisterId(pub ContentId);

impl RegisterId {
    /// Create a new random register ID
    pub fn new() -> Self {
        // Generate a unique string based on the current time to hash
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
            
        let register_data = format!("register-{}", now);
        
        // Generate a content ID
        let hasher = HashFactory::default().create_hasher().unwrap();
        let hash = hasher.hash(register_data.as_bytes());
        let content_id = ContentId::from(hash);
        
        // Create register ID from the content_id
        Self(content_id)
    }
    
    /// Create from a ContentId
    pub fn from_content_id(content_id: &ContentId) -> Self {
        Self(content_id.clone())
    }
    
    /// Create a register ID from a string
    pub fn from_str(s: &str) -> TelResult<Self> {
        let content_id = ContentId::from_str(s).map_err(|e| 
            TelError::ParseError(format!("Invalid register ID: {}", e)))?;
        Ok(Self(content_id))
    }
}

impl ContentAddressed for RegisterId {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        hasher.hash(&self.0.to_bytes())
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        ContentId::from_bytes(bytes).map(Self)
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

impl From<RegisterState> for UnifiedRegisterState {
    fn from(state: RegisterState) -> Self {
        match state {
            RegisterState::Active => UnifiedRegisterState::Active,
            RegisterState::Locked => UnifiedRegisterState::Locked,
            RegisterState::Frozen => UnifiedRegisterState::Frozen,
            RegisterState::PendingDeletion => UnifiedRegisterState::Pending,
            RegisterState::Tombstone => UnifiedRegisterState::Consumed,
        }
    }
}

impl From<UnifiedRegisterState> for RegisterState {
    fn from(state: UnifiedRegisterState) -> Self {
        match state {
            UnifiedRegisterState::Initial => RegisterState::Active,
            UnifiedRegisterState::Active => RegisterState::Active,
            UnifiedRegisterState::Locked => RegisterState::Locked,
            UnifiedRegisterState::Frozen => RegisterState::Frozen,
            UnifiedRegisterState::Pending => RegisterState::PendingDeletion,
            UnifiedRegisterState::Consumed => RegisterState::Tombstone,
            UnifiedRegisterState::Archived => RegisterState::Tombstone,
        }
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
    /// Create a binary content
    pub fn binary(data: Vec<u8>) -> Self {
        Self::Binary(data)
    }
    
    /// Create a string content
    pub fn string(data: String) -> Self {
        Self::String(data)
    }
    
    /// Create a JSON content
    pub fn json(data: serde_json::Value) -> Self {
        Self::Json(data)
    }
    
    /// Create a resource content
    pub fn resource(resource: Resource) -> Self {
        Self::Resource(Box::new(resource))
    }
    
    /// Create an empty content
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// Get the approximate size of the contents in bytes
    pub fn size(&self) -> usize {
        match self {
            Self::Binary(data) => data.len(),
            Self::String(data) => data.len(),
            Self::Json(data) => serde_json::to_string(data).unwrap_or_default().len(),
            Self::Resource(resource) => serde_json::to_string(&resource).unwrap_or_default().len(),
            Self::Empty => 0,
        }
    }
}

/// Controller label for tracking resources across domains
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            domain,
            controller_id,
            created_at: now,
            context,
        }
    }
}

/// A time range for resource validity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    
    /// Create a time range from now
    pub fn from_now(duration_ms: Option<u64>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() * 1000;
            
        let end = duration_ms.map(|d| now + d);
        
        Self {
            start: now,
            end,
        }
    }
    
    /// Check if a given time is in the range
    pub fn contains(&self, time: u64) -> bool {
        time >= self.start && (self.end.is_none() || time < self.end.unwrap())
    }
    
    /// Check if the range is expired
    pub fn is_expired(&self) -> bool {
        if self.end.is_none() {
            return false;
        }
        
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() * 1000;
                
        now >= self.end.unwrap()
    }
}

/// Time-related data for a resource
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Create a new time data with current time
    pub fn now(epoch: u64, validity_duration: Option<u64>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() * 1000;
            
        Self {
            validity: TimeRange::from_now(validity_duration),
            created_at: now,
            updated_at: now,
            epoch,
        }
    }
    
    /// Update the time data
    pub fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() * 1000;
            
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

impl From<ResourceLogicType> for UnifiedResourceLogic {
    fn from(logic_type: ResourceLogicType) -> Self {
        match logic_type {
            ResourceLogicType::Normal => UnifiedResourceLogic::Data,
            ResourceLogicType::Fungible => UnifiedResourceLogic::Fungible,
            ResourceLogicType::NonFungible => UnifiedResourceLogic::NonFungible,
            ResourceLogicType::Executable => UnifiedResourceLogic::Custom("executable".to_string()),
            ResourceLogicType::Custom(code) => UnifiedResourceLogic::Custom(format!("custom-{}", code)),
        }
    }
}

/// Resource logic trait
pub trait ResourceLogic: Send + Sync + fmt::Debug {
    /// Get the type of resource logic
    fn logic_type(&self) -> ResourceLogicType;
    
    /// Validate a mutation on the resource
    fn validate_mutation(&self, resource: &Resource, new_contents: &RegisterContents) -> TelResult<()>;
    
    /// Process a transfer of the resource
    fn process_transfer(&self, resource: &Resource, from: &Address, to: &Address) -> TelResult<RegisterContents>;
}

/// A resource in the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for the resource
    pub id: ContentId,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ContentId,
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
            metadata: metadata.unwrap_or_default(),
        }
    }
    
    /// Check if the resource is valid
    pub fn is_valid(&self) -> bool {
        self.time_data.is_valid()
    }
    
    /// Set the validity duration
    pub fn set_validity(&mut self, duration_ms: Option<u64>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() * 1000;
            
        let end = duration_ms.map(|d| now + d);
        
        self.time_data.validity.end = end;
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
    
    /// Convert to a ResourceRegister
    pub fn to_resource_register(&self) -> ResourceRegister {
        // Convert resource logic to unified model
        let resource_logic = self.logic_type.into();
        
        // Extract data as bytes
        let data_bytes = match serde_json::to_vec(&self.data) {
            Ok(bytes) => bytes,
            Err(_) => Vec::new(),
        };
        
        // Convert metadata
        let mut metadata = HashMap::<String, String>::new();
        for (key, value) in &self.metadata {
            // Convert JSON values to strings for the unified model
            if let Ok(value_str) = serde_json::to_string(value) {
                metadata.insert(key.clone(), value_str);
            }
        }
        
        // Add TEL-specific fields
        metadata.insert("tel_resource_type".to_string(), self.resource_type.clone());
        metadata.insert("tel_version".to_string(), self.version.clone());
        metadata.insert("tel_created_at".to_string(), self.time_data.created_at.to_string());
        metadata.insert("tel_updated_at".to_string(), self.time_data.updated_at.to_string());
        metadata.insert("tel_epoch".to_string(), self.time_data.epoch.to_string());
        
        // Add controller if present
        if let Some(controller) = &self.controller {
            metadata.insert("tel_controller_domain".to_string(), controller.domain.clone());
            metadata.insert("tel_controller_id".to_string(), controller.controller_id.clone());
            metadata.insert("tel_controller_created_at".to_string(), controller.created_at.to_string());
            if let Some(context) = &controller.context {
                metadata.insert("tel_controller_context".to_string(), context.clone());
            }
        }
        
        // Create a register using migrate helpers
        migrate_helpers::create_register_with_metadata(
            self.id.to_string(),
            self.resource_type.clone(),
            data_bytes,
            metadata
        )
    }
    
    /// Create from a ResourceRegister
    pub fn from_resource_register(register: &ResourceRegister) -> TelResult<Self> {
        // Extract TEL-specific fields
        let resource_type = register.metadata.get("tel_resource_type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
            
        let version = register.metadata.get("tel_version")
            .cloned()
            .unwrap_or_else(|| "1".to_string());
            
        let epoch = register.metadata.get("tel_epoch")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
            
        // Convert data from bytes to JSON
        let data = match serde_json::from_slice::<serde_json::Value>(&register.contents) {
            Ok(json) => json,
            Err(_) => serde_json::Value::Null,
        };
        
        // Determine resource logic type
        let logic_type = match register.resource_logic {
            UnifiedResourceLogic::Fungible => ResourceLogicType::Fungible,
            UnifiedResourceLogic::NonFungible => ResourceLogicType::NonFungible,
            UnifiedResourceLogic::Data => ResourceLogicType::Normal,
            UnifiedResourceLogic::Custom(ref s) if s == "executable" => ResourceLogicType::Executable,
            UnifiedResourceLogic::Custom(ref s) if s.starts_with("custom-") => {
                let code = s.strip_prefix("custom-")
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(0);
                ResourceLogicType::Custom(code)
            },
            _ => ResourceLogicType::Normal,
        };
        
        // Extract time data
        let created_at = register.metadata.get("tel_created_at")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() * 1000
            });
            
        let updated_at = register.metadata.get("tel_updated_at")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(created_at);
            
        // Create time range with no expiration by default
        let validity = TimeRange::new(created_at, None);
        
        // Create time data
        let time_data = ResourceTimeData {
            validity,
            created_at,
            updated_at,
            epoch,
        };
        
        // Extract controller if present
        let controller = if register.metadata.contains_key("tel_controller_domain") {
            let domain = register.metadata.get("tel_controller_domain")
                .cloned()
                .unwrap_or_default();
                
            let controller_id = register.metadata.get("tel_controller_id")
                .cloned()
                .unwrap_or_default();
                
            let created_at = register.metadata.get("tel_controller_created_at")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
                
            let context = register.metadata.get("tel_controller_context").cloned();
            
            Some(ControllerLabel {
                domain,
                controller_id,
                created_at,
                context,
            })
        } else {
            None
        };
        
        // Extract metadata, filtering out TEL-specific fields
        let mut metadata = HashMap::new();
        for (key, value) in &register.metadata {
            if !key.starts_with("tel_") {
                // Try to convert string back to JSON value
                if let Ok(json_value) = serde_json::from_str(value) {
                    metadata.insert(key.clone(), json_value);
                } else {
                    // Fall back to string value
                    metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
                }
            }
        }
        
        Ok(Self {
            id: register.id.clone(),
            resource_type,
            version,
            data,
            logic_type,
            time_data,
            controller,
            metadata,
        })
    }
}

/// A content-addressed register in the TEL resource system
#[derive(Debug, Clone)]
pub struct Register {
    /// The underlying resource register
    register: ResourceRegister,
    /// The contents of the register
    contents: RegisterContents,
}

impl Register {
    /// Create a new register
    pub fn new(
        id: RegisterId,
        contents: RegisterContents,
        metadata: Option<HashMap<String, serde_json::Value>>,
        state: RegisterState,
    ) -> Self {
        // Construct a resource register
        let register_contents = contents.clone();
        let serialized_contents = match &register_contents {
            RegisterContents::Binary(data) => data.clone(),
            RegisterContents::String(s) => s.as_bytes().to_vec(),
            RegisterContents::Json(json) => serde_json::to_vec(json).unwrap_or_default(),
            RegisterContents::Resource(resource) => {
                // Convert the resource to a resource register directly
                let resource_register = resource.to_resource_register();
                return Self {
                    register: resource_register,
                    contents: register_contents,
                };
            }
            RegisterContents::Empty => Vec::new(),
        };
        
        // Prepare metadata for unified register
        let mut string_metadata = HashMap::new();
        if let Some(meta) = metadata {
            for (key, value) in meta {
                if let Ok(value_str) = serde_json::to_string(&value) {
                    string_metadata.insert(key, value_str);
                }
            }
        }
        
        // Store content type in metadata
        let content_type = match &contents {
            RegisterContents::Binary(_) => "binary",
            RegisterContents::String(_) => "string",
            RegisterContents::Json(_) => "json",
            RegisterContents::Resource(_) => "resource", // Should not reach here
            RegisterContents::Empty => "empty",
        };
        string_metadata.insert("tel_content_type".to_string(), content_type.to_string());
        
        // Create resource register
        let unified_state = UnifiedRegisterState::from(state);
        let resource_register = ResourceRegister::new_with_state(
            id.0,
            UnifiedResourceLogic::Data, // Default for plain register
            "register".to_string(),     // Type for register
            serialized_contents,
            string_metadata,
            unified_state,
            None, // No controller
        );
        
        Self {
            register: resource_register,
            contents: register_contents,
        }
    }
    
    /// Create a register from the unified ResourceRegister model
    pub fn from_resource_register(register: ResourceRegister) -> Self {
        // Determine content type from metadata
        let content_type = register.metadata.get("tel_content_type")
            .cloned()
            .unwrap_or_else(|| "binary".to_string());
            
        let contents = match content_type.as_str() {
            "string" => {
                // Convert bytes to string
                if let Ok(s) = String::from_utf8(register.contents.clone()) {
                    RegisterContents::String(s)
                } else {
                    // Fall back to binary if not valid UTF-8
                    RegisterContents::Binary(register.contents.clone())
                }
            },
            "json" => {
                // Parse as JSON
                if let Ok(json) = serde_json::from_slice(&register.contents) {
                    RegisterContents::Json(json)
                } else {
                    // Fall back to binary if not valid JSON
                    RegisterContents::Binary(register.contents.clone())
                }
            },
            "resource" => {
                // Try to parse the embedded resource
                if let Ok(resource) = Resource::from_resource_register(&register) {
                    RegisterContents::Resource(Box::new(resource))
                } else {
                    // Fall back to binary
                    RegisterContents::Binary(register.contents.clone())
                }
            },
            "empty" => RegisterContents::Empty,
            _ => RegisterContents::Binary(register.contents.clone()),
        };
            
        Self {
            register,
            contents,
        }
    }
    
    /// Get the underlying resource register
    pub fn resource_register(&self) -> &ResourceRegister {
        &self.register
    }
    
    /// Convert to the underlying resource register
    pub fn into_resource_register(self) -> ResourceRegister {
        self.register
    }
    
    /// Get the register ID
    pub fn id(&self) -> &RegisterId {
        // Create a reference to the RegisterId from ContentId
        // This avoids the need to clone
        unsafe {
            &*((&self.register.id) as *const ContentId as *const RegisterId)
        }
    }
    
    /// Get the register contents
    pub fn contents(&self) -> &RegisterContents {
        &self.contents
    }
    
    /// Get the register state
    pub fn state(&self) -> RegisterState {
        RegisterState::from(self.register.state())
    }
    
    /// Get the register metadata
    pub fn metadata(&self) -> HashMap<String, serde_json::Value> {
        // Convert string metadata to JSON values
        let mut result = HashMap::new();
        for (key, value) in &self.register.metadata {
            // Skip TEL-specific metadata
            if key.starts_with("tel_") {
                continue;
            }
            
            // Try to parse as JSON, fall back to string
            if let Ok(json_value) = serde_json::from_str(value) {
                result.insert(key.clone(), json_value);
            } else {
                result.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }
        
        result
    }
    
    /// Check if register is active
    pub fn is_active(&self) -> bool {
        matches!(self.register.state(), UnifiedRegisterState::Active)
    }
    
    /// Check if register is locked
    pub fn is_locked(&self) -> bool {
        matches!(self.register.state(), UnifiedRegisterState::Locked)
    }
    
    /// Check if register is frozen
    pub fn is_frozen(&self) -> bool {
        matches!(self.register.state(), UnifiedRegisterState::Frozen)
    }
    
    /// Check if register is consumed
    pub fn is_consumed(&self) -> bool {
        matches!(self.register.state(), UnifiedRegisterState::Consumed)
    }
    
    /// Check if register is archived
    pub fn is_archived(&self) -> bool {
        matches!(self.register.state(), UnifiedRegisterState::Archived)
    }
    
    /// Get the size of the register in bytes
    pub fn size(&self) -> usize {
        // Calculate size based on serialized form
        let metadata_size = self.register.metadata
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();
            
        self.register.contents.len() + metadata_size
    }
    
    /// Update register contents
    pub fn update_contents(&mut self, contents: RegisterContents) -> TelResult<()> {
        // Special case for resource
        if let RegisterContents::Resource(resource) = &contents {
            // Convert the resource to a resource register
            let resource_register = resource.to_resource_register();
            // Update only necessary fields from resource_register
            self.register.contents = resource_register.contents.clone();
            // Update metadata from resource
            for (key, value) in resource_register.metadata {
                self.register.metadata.insert(key, value);
            }
            self.contents = contents;
            return Ok(());
        }
        
        // Handle other content types
        let serialized_contents = match &contents {
            RegisterContents::Binary(data) => data.clone(),
            RegisterContents::String(s) => s.as_bytes().to_vec(),
            RegisterContents::Json(json) => serde_json::to_vec(json).unwrap_or_default(),
            RegisterContents::Resource(_) => unreachable!(), // Handled above
            RegisterContents::Empty => Vec::new(),
        };
        
        // Update content type in metadata
        let content_type = match &contents {
            RegisterContents::Binary(_) => "binary",
            RegisterContents::String(_) => "string",
            RegisterContents::Json(_) => "json",
            RegisterContents::Resource(_) => "resource", // Should not reach here
            RegisterContents::Empty => "empty",
        };
        self.register.metadata.insert("tel_content_type".to_string(), content_type.to_string());
        
        // Update resource register contents
        self.register.contents = serialized_contents;
        // Update cached contents
        self.contents = contents;
        
        Ok(())
    }
    
    /// Update register metadata
    pub fn update_metadata(&mut self, metadata: HashMap<String, serde_json::Value>) -> TelResult<()> {
        // Convert metadata to strings
        for (key, value) in metadata {
            // Skip TEL-specific metadata
            if key.starts_with("tel_") {
                continue;
            }
            
            if let Ok(value_str) = serde_json::to_string(&value) {
                self.register.metadata.insert(key, value_str);
            }
        }
        
        Ok(())
    }
    
    /// Add metadata to the register
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) -> TelResult<()> {
        // Skip TEL-specific metadata
        if key.starts_with("tel_") {
            return Ok(());
        }
        
        if let Ok(value_str) = serde_json::to_string(&value) {
            self.register.metadata.insert(key, value_str);
        }
        
        Ok(())
    }
    
    /// Set the register state
    pub fn set_state(&mut self, state: RegisterState) -> TelResult<()> {
        // Convert to unified state
        let unified_state = UnifiedRegisterState::from(state);
        
        // Use the unified lifecycle manager to change state
        // TODO: Handle errors properly with conversion
        self.register.state = unified_state;
        
        Ok(())
    }
    
    /// Lock the register
    pub fn lock(&mut self) -> TelResult<()> {
        // Use the unified lock method
        self.register.lock();
        Ok(())
    }
    
    /// Unlock the register
    pub fn unlock(&mut self) -> TelResult<()> {
        // Use the unified unlock method
        self.register.unlock();
        Ok(())
    }
    
    /// Freeze the register
    pub fn freeze(&mut self) -> TelResult<()> {
        // Use the unified freeze method
        self.register.freeze();
        Ok(())
    }
    
    /// Unfreeze the register
    pub fn unfreeze(&mut self) -> TelResult<()> {
        // Use the unified unfreeze method
        self.register.unfreeze();
        Ok(())
    }
    
    /// Consume the register
    pub fn consume(&mut self) -> TelResult<()> {
        // Use the unified consume method
        self.register.consume();
        Ok(())
    }
    
    /// Archive the register
    pub fn archive(&mut self) -> TelResult<()> {
        // Use the unified archive method
        self.register.archive();
        Ok(())
    }
    
    /// Unarchive the register
    pub fn unarchive(&mut self) -> TelResult<()> {
        // Use the unified unarchive method
        self.register.unarchive();
        Ok(())
    }
} 
