// Log entry types
// This file defines the core types for log entries

use std::collections::HashMap;
use std::fmt; // Import fmt
use std::str::FromStr; // Add FromStr trait
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize, io::Read, io::Write as BorshWrite};
use byteorder::WriteBytesExt; // Add WriteBytesExt import
use causality_types::{TraceId, Timestamp, DomainId, ContentId, ContentAddressingError};
use crate::log::event_entry::{EventEntry, EventSeverity}; // Keep EventEntry import
use crate::log::event::EventMetadata; // Keep EventMetadata import
use causality_types::crypto_primitives::{ContentHash, HashAlgorithm, HashOutput};
use causality_core::effect::EffectType;
use serde_json::{Value, json};
use anyhow::Result;

/// Wrapper around serde_json::Value for Borsh serialization.
/// Serializes the JSON Value to its canonical string representation and then serializes the string with Borsh.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BorshJsonValue(pub serde_json::Value);

// Manual implementation of BorshSerialize for BorshJsonValue
impl BorshSerialize for BorshJsonValue {
    fn serialize<W: BorshWrite>(&self, writer: &mut W) -> std::io::Result<()> {
        // Use serde_json::to_vec for canonical representation, handling potential errors
        let json_bytes = serde_json::to_vec(&self.0)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("JSON serialization failed: {}", e)))?;
        // Explicitly use BorshSerialize for the Vec<u8>
        borsh::BorshSerialize::serialize(&json_bytes, writer)
    }
}

// Manual implementation of BorshDeserialize for BorshJsonValue
impl BorshDeserialize for BorshJsonValue {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Deserialize the byte vector using Borsh
        let json_bytes: Vec<u8> = BorshDeserialize::deserialize_reader(reader)?;
        // Deserialize the JSON Value from the byte vector, handling potential errors
        let value = serde_json::from_slice(&json_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("JSON deserialization failed: {}", e)))?;
        Ok(BorshJsonValue(value))
    }
}

// Implement Display for BorshJsonValue to forward to inner Value's to_string()
impl fmt::Display for BorshJsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Wrapper for EffectType to implement Serialize and Deserialize
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SerializableEffectType(pub String);

impl From<EffectType> for SerializableEffectType {
    fn from(effect_type: EffectType) -> Self {
        SerializableEffectType(effect_type.to_string())
    }
}

impl From<SerializableEffectType> for EffectType {
    fn from(serializable: SerializableEffectType) -> Self {
        serializable.0.parse().unwrap_or_else(|_| EffectType::Custom(serializable.0))
    }
}

impl std::ops::Deref for SerializableEffectType {
    type Target = String;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToString for SerializableEffectType {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Enum identifying the type of log entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EntryType {
    Fact,
    Effect,
    ResourceAccess,
    SystemEvent,
    Operation,
    Event, // Added Event variant for compatibility
    Custom(String), // Allow custom string types
}

/// Represents a single fact observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactEntry {
    /// The domain where this fact was observed
    pub domain: DomainId,
    /// The block height where this fact was observed
    pub block_height: u64,
    /// The block hash where this fact was observed
    pub block_hash: Option<String>,
    /// The timestamp when this fact was observed
    pub observed_at: i64,
    /// The fact type
    pub fact_type: String,
    /// The resources related to this fact
    pub resources: Vec<ContentId>,
    /// The serialized fact data
    pub data: BorshJsonValue,
    /// Whether the fact was verified
    pub verified: bool,
    /// Compatibility field - maps to domain
    #[serde(skip)]
    pub domain_id: DomainId,
    /// Compatibility field - maps to fact_type
    #[serde(skip)]
    pub fact_id: String,
}

impl FactEntry {
    /// Create a new fact entry
    pub fn new(
        domain: DomainId,
        block_height: u64,
        block_hash: Option<String>,
        observed_at: i64,
        fact_type: String,
        resources: Vec<ContentId>,
        data: BorshJsonValue,
        verified: bool,
    ) -> Self {
        let domain_id = domain.clone();
        let fact_id = fact_type.clone();
        
        Self {
            domain,
            block_height,
            block_hash,
            observed_at,
            fact_type,
            resources,
            data,
            verified,
            domain_id,
            fact_id,
        }
    }
}

// Manual implementation of BorshSerialize for FactEntry
impl BorshSerialize for FactEntry {
    fn serialize<W: BorshWrite>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.domain, writer)?;
        BorshSerialize::serialize(&self.block_height, writer)?;
        BorshSerialize::serialize(&self.block_hash, writer)?;
        BorshSerialize::serialize(&self.observed_at, writer)?;
        BorshSerialize::serialize(&self.fact_type, writer)?;
        BorshSerialize::serialize(&self.resources, writer)?;
        BorshSerialize::serialize(&self.data, writer)?;
        BorshSerialize::serialize(&self.verified, writer)?;
        Ok(())
    }
}

// Manual implementation of BorshDeserialize for FactEntry
impl BorshDeserialize for FactEntry {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let domain: DomainId = BorshDeserialize::deserialize_reader(reader)?;
        let block_height: u64 = BorshDeserialize::deserialize_reader(reader)?;
        let block_hash: Option<String> = BorshDeserialize::deserialize_reader(reader)?;
        let observed_at: i64 = BorshDeserialize::deserialize_reader(reader)?;
        let fact_type: String = BorshDeserialize::deserialize_reader(reader)?;
        let resources: Vec<ContentId> = BorshDeserialize::deserialize_reader(reader)?;
        let data: BorshJsonValue = BorshDeserialize::deserialize_reader(reader)?;
        let verified: bool = BorshDeserialize::deserialize_reader(reader)?;
        
        let domain_id = domain.clone();
        let fact_id = fact_type.clone();
        
        Ok(Self {
            domain,
            block_height,
            block_hash,
            observed_at,
            fact_type,
            resources,
            data,
            verified,
            domain_id,
            fact_id,
        })
    }
}

/// Represents the result of an effect execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectEntry {
    /// The type of effect
    pub effect_type: SerializableEffectType,
    /// The resources affected by this effect
    pub resources: Vec<ContentId>,
    /// The domains involved in this effect
    pub domains: Vec<DomainId>,
    /// The hash of the effect code
    pub code_hash: Option<String>,
    /// The serialized effect parameters
    pub parameters: HashMap<String, BorshJsonValue>,
    /// The result of the effect execution
    pub result: Option<BorshJsonValue>,
    /// Whether the effect was successful
    pub success: bool,
    /// An error message, if the effect failed
    pub error: Option<String>,
    /// Compatibility field - maps to domains[0] if available
    #[serde(skip)]
    pub domain_id: DomainId,
    /// Compatibility field - maps to effect_type
    #[serde(skip)]
    pub effect_id: String,
    /// Compatibility field
    #[serde(skip)]
    pub status: String,
}

impl EffectEntry {
    /// Create a new effect entry
    pub fn new(
        effect_type: SerializableEffectType,
        resources: Vec<ContentId>,
        domains: Vec<DomainId>,
        code_hash: Option<String>,
        parameters: HashMap<String, BorshJsonValue>,
        result: Option<BorshJsonValue>,
        success: bool,
        error: Option<String>,
    ) -> Self {
        let domain_id_compat = domains.first().cloned().unwrap_or_default();
        let effect_id = effect_type.to_string();
        let status = if success { "success".to_string() } else { "failed".to_string() };
        
        Self {
            effect_type,
            resources,
            domains,
            code_hash,
            parameters,
            result,
            success,
            error,
            domain_id: domain_id_compat,
            effect_id,
            status,
        }
    }
}

// Manual implementation of BorshSerialize for EffectEntry
impl BorshSerialize for EffectEntry {
    fn serialize<W: BorshWrite>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.effect_type, writer)?;
        BorshSerialize::serialize(&self.resources, writer)?;
        BorshSerialize::serialize(&self.domains, writer)?;
        BorshSerialize::serialize(&self.code_hash, writer)?;
        BorshSerialize::serialize(&self.parameters, writer)?;
        BorshSerialize::serialize(&self.result, writer)?;
        BorshSerialize::serialize(&self.success, writer)?;
        BorshSerialize::serialize(&self.error, writer)?;
        Ok(())
    }
}

// Manual implementation of BorshDeserialize for EffectEntry
impl BorshDeserialize for EffectEntry {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let effect_type: SerializableEffectType = BorshDeserialize::deserialize_reader(reader)?;
        let resources: Vec<ContentId> = BorshDeserialize::deserialize_reader(reader)?;
        let domains: Vec<DomainId> = BorshDeserialize::deserialize_reader(reader)?;
        let code_hash: Option<String> = BorshDeserialize::deserialize_reader(reader)?;
        let parameters: HashMap<String, BorshJsonValue> = BorshDeserialize::deserialize_reader(reader)?;
        let result: Option<BorshJsonValue> = BorshDeserialize::deserialize_reader(reader)?;
        let success: bool = BorshDeserialize::deserialize_reader(reader)?;
        let error: Option<String> = BorshDeserialize::deserialize_reader(reader)?;
        
        let domain_id_compat = domains.first().cloned().unwrap_or_default();
        let effect_id = effect_type.to_string();
        let status = if success { "success".to_string() } else { "failed".to_string() };
        
        Ok(Self {
            effect_type,
            resources,
            domains,
            code_hash,
            parameters,
            result,
            success,
            error,
            domain_id: domain_id_compat,
            effect_id,
            status,
        })
    }
}

/// Represents access to a resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceAccessEntry {
    pub resource_id: String, // Identifier for the accessed resource
    pub action: String, // e.g., "Read", "Write", "Create", "Delete"
    pub details: BorshJsonValue, // Changed to BorshJsonValue
    // Add resources field for compatibility with filter
    #[serde(skip)]
    pub resources: Vec<ContentId>,
    // Add domains field for compatibility with filter
    #[serde(skip)]
    pub domains: Vec<DomainId>,
}

impl ResourceAccessEntry {
    /// Create a new resource access entry
    pub fn new(
        resource_id: String,
        action: String,
        details: BorshJsonValue,
    ) -> Self {
        // Try to parse resource_id as ContentId and add to resources
        let resources = match ContentId::parse(&resource_id) {
            Ok(cid) => vec![cid],
            Err(_) => Vec::new(),
        };
        
        Self {
            resource_id,
            action,
            details,
            resources,
            domains: Vec::new(),
        }
    }
}

/// Represents a system-level event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SystemEventEntry {
    pub event_type: String, // e.g., "Startup", "Shutdown", "ConfigurationChange"
    pub data: BorshJsonValue, // Changed to BorshJsonValue
    // Add resources field for compatibility with filter
    #[serde(skip)]
    pub resources: Vec<ContentId>,
    // Add domains field for compatibility with filter
    #[serde(skip)]
    pub domains: Vec<DomainId>,
}

/// Represents a high-level operation or transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct OperationEntry {
    pub operation_id: String, // Unique ID for the operation
    pub operation_type: String, // Type of operation
    pub status: String, // e.g., "Started", "Completed", "Failed"
    pub details: BorshJsonValue, // Changed to BorshJsonValue
    // Add resources field for compatibility with filter
    #[serde(skip)]
    pub resources: Vec<ContentId>,
    // Add domains field for compatibility with filter
    #[serde(skip)]
    pub domains: Vec<DomainId>,
}

/// Union type for different kinds of entry data.
/// Note: EventEntry is handled separately for now due to its complexity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryData {
    Fact(FactEntry),
    Effect(EffectEntry),
    ResourceAccess(ResourceAccessEntry),
    SystemEvent(SystemEventEntry),
    Operation(OperationEntry),
    Event(EventEntry), // Keep EventEntry here
    Custom(String, BorshJsonValue), // Wrap custom data
}

// Manual BorshSerialize for EntryData
impl BorshSerialize for EntryData {
    fn serialize<W: BorshWrite>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            EntryData::Fact(data) => {
                writer.write_u8(0)?; // Variant index 0
                BorshSerialize::serialize(data, writer)?;
            }
            EntryData::Effect(data) => {
                writer.write_u8(1)?; // Variant index 1
                BorshSerialize::serialize(data, writer)?;
            }
            EntryData::ResourceAccess(data) => {
                writer.write_u8(2)?; // Variant index 2
                BorshSerialize::serialize(data, writer)?;
            }
            EntryData::SystemEvent(data) => {
                writer.write_u8(3)?; // Variant index 3
                BorshSerialize::serialize(data, writer)?;
            }
            EntryData::Operation(data) => {
                writer.write_u8(4)?; // Variant index 4
                BorshSerialize::serialize(data, writer)?;
            }
            EntryData::Event(data) => {
                writer.write_u8(5)?; // Variant index 5
                 // Use EventEntry's manual implementation
                BorshSerialize::serialize(data, writer)?;
            }
            EntryData::Custom(type_str, data) => {
                writer.write_u8(6)?; // Variant index 6
                BorshSerialize::serialize(type_str, writer)?;
                 // Use BorshJsonValue's manual implementation
                BorshSerialize::serialize(data, writer)?;
            }
        }
        Ok(())
    }
}

// Manual BorshDeserialize for EntryData
impl BorshDeserialize for EntryData {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let variant_idx: u8 = BorshDeserialize::deserialize_reader(reader)?;
        match variant_idx {
            0 => Ok(EntryData::Fact(BorshDeserialize::deserialize_reader(reader)?)),
            1 => Ok(EntryData::Effect(BorshDeserialize::deserialize_reader(reader)?)),
            2 => Ok(EntryData::ResourceAccess(BorshDeserialize::deserialize_reader(reader)?)),
            3 => Ok(EntryData::SystemEvent(BorshDeserialize::deserialize_reader(reader)?)),
            4 => Ok(EntryData::Operation(BorshDeserialize::deserialize_reader(reader)?)),
            5 => Ok(EntryData::Event(BorshDeserialize::deserialize_reader(reader)?)), // Uses EventEntry's manual impl
            6 => {
                let type_str = BorshDeserialize::deserialize_reader(reader)?;
                let data = BorshDeserialize::deserialize_reader(reader)?; // Uses BorshJsonValue's manual impl
                Ok(EntryData::Custom(type_str, data))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid variant index for EntryData",
            )),
        }
    }
}

/// Represents a single entry in the causality log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct LogEntry {
    /// Content-addressed identifier (hash of the entry).
    pub id: String,
    /// Timestamp of when the entry was created.
    pub timestamp: Timestamp,
    /// Type of the log entry.
    pub entry_type: EntryType,
    /// Specific data associated with the entry type.
    pub data: EntryData,
    /// Optional trace ID for correlating related entries.
    pub trace_id: Option<TraceId>,
    /// Optional ID of the parent log entry in a causal chain.
    pub parent_id: Option<String>,
    /// Additional metadata (key-value pairs).
    pub metadata: HashMap<String, String>,
    // Removed entry_hash field
}

impl LogEntry {
    /// Calculate the content hash of the log entry using Borsh serialization.
    pub fn calculate_hash(&self) -> Result<HashOutput, ContentAddressingError> {
        // Create a temporary copy without the ID field for hashing
        let entry_to_hash = LogEntry {
             id: String::new(), // Exclude ID from hash calculation
             timestamp: self.timestamp.clone(),
             entry_type: self.entry_type.clone(),
             data: self.data.clone(),
             trace_id: self.trace_id.clone(),
             parent_id: self.parent_id.clone(),
             metadata: self.metadata.clone(),
        };

        let bytes = borsh::to_vec(&entry_to_hash)
            // Use ValidationError as suggested by compiler/previous errors
            .map_err(|e| ContentAddressingError::ValidationError(format!("Borsh serialize failed: {}", e)))?;
        // Use full path to content_hash_from_bytes
        Ok(causality_types::content_addressing::content_hash_from_bytes(&bytes))
    }

    /// Create a new log entry, calculating its content-addressed ID.
    pub fn new(
        entry_type: EntryType, data: EntryData, trace_id: Option<TraceId>,
        parent_id: Option<String>, metadata: HashMap<String, String>,
    ) -> Result<Self, ContentAddressingError> { 
        let mut entry = LogEntry {
            id: String::new(), // Placeholder
            timestamp: Timestamp::now(),
            entry_type,
            data,
            trace_id,
            parent_id,
            metadata,
        };
        // Calculate hash - result is HashOutput
        let hash_output = entry.calculate_hash()?;
        // Convert HashOutput to ContentHash for the ID
        let content_hash = ContentHash::new(
             &hash_output.algorithm().to_string(), 
             // Use as_bytes() as suggested by compiler
             hash_output.as_bytes().to_vec()    
        );
        // Set the final ID
        entry.id = content_hash.to_string(); 
        Ok(entry)
    }
    
    /// Set the trace ID
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Set the parent ID
    pub fn with_parent_id(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// Convenience functions for creating specific log entry types

/// Create an event log entry.
pub fn create_event_entry(
    event_entry: EventEntry, // Takes the full EventEntry struct
    trace_id: Option<TraceId>,
    parent_id: Option<String>,
    metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    LogEntry::new(
        EntryType::Custom("Event".to_string()), // Use Custom type for events
        EntryData::Event(event_entry),
        trace_id,
        parent_id,
        metadata,
    )
}

/// Create a fact observation log entry.
pub fn create_fact_observation(
    domain_id: DomainId,
    fact_id: String,
    details: Option<Value>, // Takes Option<Value>
    trace_id: Option<TraceId>,
    parent_id: Option<String>,
    metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    // Create a FactEntry with reasonable defaults for test/demo purposes
    let fact_entry = FactEntry::new(
        domain_id, // Pass ownership here
        0, // Default value
        None, // Default value
        Timestamp::now().timestamp(),
        "observation".to_string(), // Default type
        Vec::new(), // Empty resources by default 
        BorshJsonValue(details.unwrap_or(json!({}))), // Use provided details or empty object
        false, // Not verified by default
        // Compatibility fields are set inside FactEntry::new
    );
    
    LogEntry::new(
        EntryType::Fact,
        EntryData::Fact(fact_entry),
        trace_id,
        parent_id,
        metadata,
    )
}

/// Create a resource access log entry.
pub fn create_resource_access(
    resource_id: String,
    action: String,
    details: Option<Value>, // Takes Option<Value>
    trace_id: Option<TraceId>,
    parent_id: Option<String>,
    metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    let resource_entry = ResourceAccessEntry {
        resource_id,
        action,
        details: BorshJsonValue(details.unwrap_or(json!({}))),
        resources: Vec::new(),
        domains: Vec::new(),
    };
    
    LogEntry::new(
        EntryType::ResourceAccess,
        EntryData::ResourceAccess(resource_entry),
        trace_id,
        parent_id,
        metadata,
    )
}

/// Create a system event log entry.
pub fn create_system_event(
    event_type: String,
    data: Option<Value>, // Takes Option<Value>
    trace_id: Option<TraceId>,
    parent_id: Option<String>,
    metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    let system_event = SystemEventEntry {
        event_type,
        data: BorshJsonValue(data.unwrap_or(json!({}))),
        resources: Vec::new(),
        domains: Vec::new(),
    };
    
    LogEntry::new(
        EntryType::SystemEvent,
        EntryData::SystemEvent(system_event),
        trace_id,
        parent_id,
        metadata,
    )
}

/// Create a domain effect log entry.
pub fn create_domain_effect(
     domain_id: DomainId,
     effect_id: String,
     status: String,
     outcome: Option<Value>, // Takes Option<Value>
     error: Option<String>,
     trace_id: Option<TraceId>,
     parent_id: Option<String>,
     metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    // Create an EffectEntry 
    let effect_entry = EffectEntry::new(
        SerializableEffectType("domain_effect".to_string()),
        Vec::new(), // Empty resources by default
        vec![domain_id], // Pass ownership of domain_id into the vec here
        None, // No code hash by default
        HashMap::new(), // Empty parameters by default
        outcome.map(BorshJsonValue),
        error.is_none(), // Success if no error
        error,
        // Compatibility fields are set inside EffectEntry::new
    );
    
    LogEntry::new(
        EntryType::Effect,
        EntryData::Effect(effect_entry),
        trace_id,
        parent_id,
        metadata,
    )
}

/// Create an operation log entry.
pub fn create_operation(
    operation_id: String,
    operation_type: String,
    status: String,
    details: Option<Value>, // Takes Option<Value>
    trace_id: Option<TraceId>,
    parent_id: Option<String>,
    metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    let operation_entry = OperationEntry {
        operation_id,
        operation_type,
        status,
        details: BorshJsonValue(details.unwrap_or(json!({}))),
        resources: Vec::new(),
        domains: Vec::new(),
    };
    
    LogEntry::new(
        EntryType::Operation,
        EntryData::Operation(operation_entry),
        trace_id,
        parent_id,
        metadata,
    )
}

/// Create a custom log entry.
pub fn create_custom_entry(
    custom_type: String,
    data: Value, // Takes Value
    trace_id: Option<TraceId>,
    parent_id: Option<String>,
    metadata: HashMap<String, String>,
) -> Result<LogEntry, ContentAddressingError> {
    LogEntry::new(
        EntryType::Custom(custom_type.clone()),
        EntryData::Custom(custom_type, BorshJsonValue(data)),
        trace_id,
        parent_id,
        metadata,
    )
} 