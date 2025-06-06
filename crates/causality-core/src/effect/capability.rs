//! Rich capability system for Object integration and Record Operations
//!
//! This module provides structured capability levels and enhanced capability 
//! checking for the Object system, with special support for record field access
//! operations that compile to Layer 1 tensor operations.

use std::collections::{HashSet, HashMap};
use ssz::{Encode, Decode};

/// Field name type for record operations
pub type FieldName = String;

/// Record schema definition for capability-based record operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSchema {
    /// Field name to type mapping
    pub fields: HashMap<FieldName, String>, // Type names as strings for simplicity
    /// Required capabilities for schema operations
    pub required_capabilities: HashSet<String>,
}

impl std::hash::Hash for RecordSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Create a sorted vector for deterministic hashing
        let mut fields: Vec<_> = self.fields.iter().collect();
        fields.sort_by_key(|(k, _)| *k);
        for (k, v) in fields {
            k.hash(state);
            v.hash(state);
        }
        
        let mut capabilities: Vec<_> = self.required_capabilities.iter().collect();
        capabilities.sort();
        for cap in capabilities {
            cap.hash(state);
        }
    }
}

impl RecordSchema {
    /// Create a new record schema
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            required_capabilities: HashSet::new(),
        }
    }
    
    /// Add a field to the schema
    pub fn with_field(mut self, name: impl Into<String>, type_name: impl Into<String>) -> Self {
        self.fields.insert(name.into(), type_name.into());
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.insert(capability.into());
        self
    }
}

/// Capability types for record operations
/// These provide fine-grained access control for field operations that
/// compile down to Layer 1 tensor operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecordCapability {
    /// Read access to a specific field
    ReadField(FieldName),
    /// Write access to a specific field  
    WriteField(FieldName),
    /// Create a new record with given schema
    CreateRecord(RecordSchema),
    /// Delete an entire record
    DeleteRecord,
    /// Project (select) specific fields from a record
    ProjectFields(Vec<FieldName>),
    /// Extend a record with additional fields
    ExtendRecord(RecordSchema),
    /// Restrict a record by removing fields
    RestrictRecord(Vec<FieldName>),
    /// Full access to all record operations (administrative)
    FullRecordAccess,
}

impl RecordCapability {
    /// Create a read field capability
    pub fn read_field(field: impl Into<String>) -> Self {
        RecordCapability::ReadField(field.into())
    }
    
    /// Create a write field capability
    pub fn write_field(field: impl Into<String>) -> Self {
        RecordCapability::WriteField(field.into())
    }
    
    /// Create a project capability for multiple fields
    pub fn project_fields(fields: Vec<impl Into<String>>) -> Self {
        RecordCapability::ProjectFields(fields.into_iter().map(|f| f.into()).collect())
    }
    
    /// Check if this capability implies another record capability
    pub fn implies(&self, other: &RecordCapability) -> bool {
        match (self, other) {
            // Full access implies everything
            (RecordCapability::FullRecordAccess, _) => true,
            
            // Exact matches
            (RecordCapability::ReadField(f1), RecordCapability::ReadField(f2)) => f1 == f2,
            (RecordCapability::WriteField(f1), RecordCapability::WriteField(f2)) => f1 == f2,
            
            // Write implies read for the same field
            (RecordCapability::WriteField(f1), RecordCapability::ReadField(f2)) => f1 == f2,
            
            // Project capabilities
            (RecordCapability::ProjectFields(fields1), RecordCapability::ReadField(f2)) => {
                fields1.contains(f2)
            }
            (RecordCapability::ProjectFields(fields1), RecordCapability::ProjectFields(fields2)) => {
                fields2.iter().all(|f| fields1.contains(f))
            }
            
            // No other implications
            _ => false,
        }
    }
    
    /// Get the fields that this capability provides access to
    pub fn accessible_fields(&self) -> Vec<FieldName> {
        match self {
            RecordCapability::ReadField(f) | RecordCapability::WriteField(f) => vec![f.clone()],
            RecordCapability::ProjectFields(fields) => fields.clone(),
            RecordCapability::FullRecordAccess => vec![], // Represents access to all fields
            _ => vec![],
        }
    }
}

/// Structured capability levels for common access patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityLevel {
    /// Read-only access
    Read,
    /// Write access (implies Read)
    Write,
    /// Execute access (implies Read)
    Execute,
    /// Administrative access (implies all others)
    Admin,
}

/// Enhanced capability with structured levels and record operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Structured capability level
    pub level: CapabilityLevel,
    /// Optional record-specific capability
    pub record_capability: Option<RecordCapability>,
}

impl Capability {
    /// Create a new capability with level
    pub fn new(name: impl Into<String>, level: CapabilityLevel) -> Self {
        Self {
            name: name.into(),
            level,
            record_capability: None,
        }
    }
    
    /// Create an admin capability
    pub fn admin(name: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Admin)
    }
    
    /// Create a basic capability (defaults to read level for backwards compatibility)
    pub fn basic(name: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Read)
    }
    
    /// Create a capability with record-specific access
    pub fn with_record_capability(mut self, record_cap: RecordCapability) -> Self {
        self.record_capability = Some(record_cap);
        self
    }
    
    /// Create a field read capability
    pub fn read_field(name: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Read)
            .with_record_capability(RecordCapability::read_field(field))
    }
    
    /// Create a field write capability  
    pub fn write_field(name: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Write)
            .with_record_capability(RecordCapability::write_field(field))
    }
    
    /// Create a record projection capability
    pub fn project_record(name: impl Into<String>, fields: Vec<impl Into<String>>) -> Self {
        Self::new(name, CapabilityLevel::Read)
            .with_record_capability(RecordCapability::project_fields(fields))
    }
    
    /// Check if this capability implies another capability
    pub fn implies(&self, other: &Capability) -> bool {
        if self.name != other.name {
            return false;
        }
        
        // Check capability level implication
        let level_implies = self.level.implies_level(&other.level);
        
        // Check record capability implication
        let record_implies = match (&self.record_capability, &other.record_capability) {
            (None, None) => true, // Both have no record capabilities
            (Some(_), None) => true, // More specific implies less specific
            (None, Some(_)) => false, // Less specific doesn't imply more specific
            (Some(self_rec), Some(other_rec)) => self_rec.implies(other_rec),
        };
        
        level_implies && record_implies
    }
    
    /// Get the accessible fields from this capability
    pub fn accessible_fields(&self) -> Vec<FieldName> {
        match &self.record_capability {
            Some(record_cap) => record_cap.accessible_fields(),
            None => vec![], // No record-specific access
        }
    }
    
    /// Create a capability with read level
    pub fn read(name: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Read)
    }
    
    /// Create a capability with write level
    pub fn write(name: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Write)
    }
    
    /// Create a capability with execute level
    pub fn execute(name: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Execute)
    }
}

impl CapabilityLevel {
    /// Get all capability levels that this level implies
    pub fn implies(&self) -> Vec<CapabilityLevel> {
        use CapabilityLevel::*;
        match self {
            Read => vec![Read],
            Write => vec![Read, Write],
            Execute => vec![Read, Execute],
            Admin => vec![Read, Write, Execute, Admin],
        }
    }
    
    /// Check if this level implies another level
    pub fn implies_level(&self, other: &CapabilityLevel) -> bool {
        self.implies().contains(other)
    }
}

/// Capability set with enhanced checking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create a new capability set
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }
    
    /// Create from a vector of capabilities
    pub fn from_capabilities(capabilities: Vec<Capability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }
    
    /// Add a capability to the set
    pub fn add(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
    }
    
    /// Check if the set has a specific capability (with implication)
    pub fn has_capability(&self, required: &Capability) -> bool {
        self.capabilities.iter().any(|cap| cap.implies(required))
    }
    
    /// Check if the set has all required capabilities
    pub fn has_all_capabilities(&self, required: &[Capability]) -> bool {
        required.iter().all(|req| self.has_capability(req))
    }
    
    /// Get all capabilities in the set
    pub fn capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Capability>> for CapabilitySet {
    fn from(capabilities: Vec<Capability>) -> Self {
        Self::from_capabilities(capabilities)
    }
}

impl From<HashSet<Capability>> for CapabilitySet {
    fn from(capabilities: HashSet<Capability>) -> Self {
        Self { capabilities }
    }
}

// SSZ implementations
impl Encode for CapabilityLevel {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_bytes_len(&self) -> usize {
        1
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let byte = match self {
            CapabilityLevel::Read => 0u8,
            CapabilityLevel::Write => 1u8,
            CapabilityLevel::Execute => 2u8,
            CapabilityLevel::Admin => 3u8,
        };
        buf.push(byte);
    }
}

impl Decode for CapabilityLevel {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.len() != 1 {
            return Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 1,
            });
        }
        
        match bytes[0] {
            0 => Ok(CapabilityLevel::Read),
            1 => Ok(CapabilityLevel::Write),
            2 => Ok(CapabilityLevel::Execute),
            3 => Ok(CapabilityLevel::Admin),
            _ => Err(ssz::DecodeError::BytesInvalid("Invalid CapabilityLevel".to_string())),
        }
    }
}

impl Encode for Capability {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        4 + self.name.len() + 1
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        (self.name.len() as u32).ssz_append(buf);
        buf.extend_from_slice(self.name.as_bytes());
        self.level.ssz_append(buf);
    }
}

impl Decode for Capability {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.len() < 5 {
            return Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 5,
            });
        }
        
        let name_len = u32::from_ssz_bytes(&bytes[0..4])? as usize;
        if bytes.len() < 4 + name_len + 1 {
            return Err(ssz::DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 4 + name_len + 1,
            });
        }
        
        let name = String::from_utf8(bytes[4..4 + name_len].to_vec())
            .map_err(|_| ssz::DecodeError::BytesInvalid("Invalid UTF-8 in capability name".to_string()))?;
        
        let level = CapabilityLevel::from_ssz_bytes(&bytes[4 + name_len..4 + name_len + 1])?;
        
        Ok(Capability { name, level, record_capability: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let read_cap = Capability::read("file");
        assert_eq!(read_cap.name, "file");
        assert_eq!(read_cap.level, CapabilityLevel::Read);
        
        let admin_cap = Capability::admin("system");
        assert_eq!(admin_cap.level, CapabilityLevel::Admin);
    }

    #[test]
    fn test_capability_implication() {
        let admin_cap = Capability::admin("file");
        let write_cap = Capability::write("file");
        let read_cap = Capability::read("file");
        let other_read_cap = Capability::read("other");
        
        // Admin implies everything for same resource
        assert!(admin_cap.implies(&write_cap));
        assert!(admin_cap.implies(&read_cap));
        assert!(admin_cap.implies(&admin_cap));
        
        // Write implies read for same resource
        assert!(write_cap.implies(&read_cap));
        assert!(!write_cap.implies(&admin_cap));
        
        // Different resources don't imply each other
        assert!(!read_cap.implies(&other_read_cap));
    }

    #[test]
    fn test_capability_level_implication() {
        use CapabilityLevel::*;
        
        assert!(Admin.implies_level(&Read));
        assert!(Admin.implies_level(&Write));
        assert!(Admin.implies_level(&Execute));
        assert!(Admin.implies_level(&Admin));
        
        assert!(Write.implies_level(&Read));
        assert!(!Write.implies_level(&Execute));
        assert!(!Write.implies_level(&Admin));
        
        assert!(Execute.implies_level(&Read));
        assert!(!Execute.implies_level(&Write));
        
        assert!(Read.implies_level(&Read));
        assert!(!Read.implies_level(&Write));
    }

    #[test]
    fn test_capability_set() {
        let mut cap_set = CapabilitySet::new();
        cap_set.add(Capability::write("file"));
        cap_set.add(Capability::execute("script"));
        
        // Write implies read
        assert!(cap_set.has_capability(&Capability::read("file")));
        assert!(cap_set.has_capability(&Capability::write("file")));
        
        // Execute implies read
        assert!(cap_set.has_capability(&Capability::read("script")));
        assert!(cap_set.has_capability(&Capability::execute("script")));
        
        // Admin not granted
        assert!(!cap_set.has_capability(&Capability::admin("file")));
        
        // Different resource not granted
        assert!(!cap_set.has_capability(&Capability::read("other")));
    }

    #[test]
    fn test_capability_set_multiple_requirements() {
        let cap_set = CapabilitySet::from_capabilities(vec![
            Capability::admin("system"),
            Capability::read("config"),
        ]);
        
        let required = vec![
            Capability::write("system"), // Admin implies write
            Capability::read("config"),
        ];
        
        assert!(cap_set.has_all_capabilities(&required));
        
        let required_with_missing = vec![
            Capability::write("system"),
            Capability::write("config"), // Only have read for config
        ];
        
        assert!(!cap_set.has_all_capabilities(&required_with_missing));
    }

    #[test]
    fn test_ssz_serialization() {
        let cap = Capability::admin("test");
        let encoded = cap.as_ssz_bytes();
        let decoded = Capability::from_ssz_bytes(&encoded).unwrap();
        assert_eq!(cap, decoded);
        
        let level = CapabilityLevel::Execute;
        let encoded = level.as_ssz_bytes();
        let decoded = CapabilityLevel::from_ssz_bytes(&encoded).unwrap();
        assert_eq!(level, decoded);
    }
} 