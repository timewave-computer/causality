//! Rich capability system for Object integration and Record Operations
//!
//! This module provides structured capability levels and enhanced capability
//! checking for the Object system, with special support for record field access
//! operations that compile to Layer 1 tensor operations.
//!
//! **Phase 3 Extensions**: Added distributed access capabilities, session-based
//! capability delegation, and cross-location capability verification.

use crate::lambda::base::{Location, SessionType};
use ssz::{Decode, Encode};
use std::collections::{BTreeMap, BTreeSet};

/// Field name type for record operations
pub type FieldName = String;

/// Record schema definition for capability-based record operations
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct RecordSchema {
    /// Field name to type mapping
    pub fields: BTreeMap<FieldName, String>, // Type names as strings for simplicity
    /// Required capabilities for schema operations
    pub required_capabilities: BTreeSet<String>,
    /// Location constraints for distributed access
    pub location_constraints: BTreeMap<FieldName, LocationConstraint>,
}

/// Location constraint for field access
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum LocationConstraint {
    /// Field must be accessed locally
    LocalOnly,
    /// Field can be accessed from specific locations
    AllowedLocations(BTreeSet<Location>),
    /// Field can be accessed from any location
    AnyLocation,
    /// Field requires specific protocol for remote access
    RequiresProtocol(SessionType),
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

        let mut constraints: Vec<_> = self.location_constraints.iter().collect();
        constraints.sort_by_key(|(k, _)| *k);
        for (k, v) in constraints {
            k.hash(state);
            format!("{:?}", v).hash(state); // Simplified hash for location constraints
        }
    }
}

impl Default for RecordSchema {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordSchema {
    /// Create a new record schema
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
            required_capabilities: BTreeSet::new(),
            location_constraints: BTreeMap::new(),
        }
    }

    /// Add a field to the schema
    pub fn with_field(
        mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> Self {
        self.fields.insert(name.into(), type_name.into());
        self
    }

    /// Add a required capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.insert(capability.into());
        self
    }

    /// Add a location constraint for a field
    pub fn with_location_constraint(
        mut self,
        field: impl Into<String>,
        constraint: LocationConstraint,
    ) -> Self {
        self.location_constraints.insert(field.into(), constraint);
        self
    }

    /// Check if a field can be accessed from a given location
    pub fn can_access_from_location(
        &self,
        field: &str,
        location: &Location,
    ) -> bool {
        match self.location_constraints.get(field) {
            Some(LocationConstraint::LocalOnly) => {
                matches!(location, Location::Local)
            }
            Some(LocationConstraint::AllowedLocations(allowed)) => {
                allowed.contains(location)
            }
            Some(LocationConstraint::AnyLocation) => true,
            Some(LocationConstraint::RequiresProtocol(_)) => true, // Requires protocol validation
            None => true, // No constraints = allowed from anywhere
        }
    }

    /// Get the required protocol for accessing a field from a remote location
    pub fn required_protocol(
        &self,
        field: &str,
        from_location: &Location,
    ) -> Option<&SessionType> {
        if matches!(from_location, Location::Local) {
            return None; // No protocol needed for local access
        }

        match self.location_constraints.get(field) {
            Some(LocationConstraint::RequiresProtocol(protocol)) => Some(protocol),
            _ => None,
        }
    }
}

/// Capability types for record operations
/// These provide fine-grained access control for field operations that
/// compile down to Layer 1 tensor operations
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
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
    /// Distributed access to fields across locations
    DistributedAccess {
        fields: Vec<FieldName>,
        allowed_locations: BTreeSet<Location>,
        required_protocol: Option<SessionType>,
    },
    /// Session-based capability delegation
    SessionDelegation {
        session_type: SessionType,
        delegated_capabilities: Vec<RecordCapability>,
        expiration: Option<u64>, // Timestamp for expiration
    },
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
        RecordCapability::ProjectFields(
            fields.into_iter().map(|f| f.into()).collect(),
        )
    }

    /// Create a distributed access capability
    pub fn distributed_access(
        fields: Vec<impl Into<String>>,
        locations: BTreeSet<Location>,
        protocol: Option<SessionType>,
    ) -> Self {
        RecordCapability::DistributedAccess {
            fields: fields.into_iter().map(|f| f.into()).collect(),
            allowed_locations: locations,
            required_protocol: protocol,
        }
    }

    /// Create a session delegation capability
    pub fn session_delegation(
        session_type: SessionType,
        capabilities: Vec<RecordCapability>,
        expiration: Option<u64>,
    ) -> Self {
        RecordCapability::SessionDelegation {
            session_type,
            delegated_capabilities: capabilities,
            expiration,
        }
    }

    /// Check if this capability implies another record capability
    pub fn implies(&self, other: &RecordCapability) -> bool {
        match (self, other) {
            // Full access implies everything
            (RecordCapability::FullRecordAccess, _) => true,

            // Exact matches
            (RecordCapability::ReadField(f1), RecordCapability::ReadField(f2)) => {
                f1 == f2
            }
            (RecordCapability::WriteField(f1), RecordCapability::WriteField(f2)) => {
                f1 == f2
            }

            // Write implies read for the same field
            (RecordCapability::WriteField(f1), RecordCapability::ReadField(f2)) => {
                f1 == f2
            }

            // Project capabilities
            (
                RecordCapability::ProjectFields(fields1),
                RecordCapability::ReadField(f2),
            ) => fields1.contains(f2),
            (
                RecordCapability::ProjectFields(fields1),
                RecordCapability::ProjectFields(fields2),
            ) => fields2.iter().all(|f| fields1.contains(f)),

            // Distributed access implications
            (
                RecordCapability::DistributedAccess {
                    fields: fields1,
                    allowed_locations: _locs1,
                    ..
                },
                RecordCapability::ReadField(f2),
            ) => {
                fields1.contains(f2) // Simplified - should also check location
            }

            (
                RecordCapability::DistributedAccess {
                    fields: fields1,
                    allowed_locations: _locs1,
                    ..
                },
                RecordCapability::DistributedAccess {
                    fields: fields2,
                    allowed_locations: _locs2,
                    ..
                },
            ) => {
                fields2.iter().all(|f| fields1.contains(f))
                    && _locs2.iter().all(|l| _locs1.contains(l))
            }

            // Session delegation implications
            (
                RecordCapability::SessionDelegation {
                    delegated_capabilities,
                    expiration,
                    ..
                },
                other,
            ) => {
                // Check if not expired
                if let Some(exp) = expiration {
                    let expiry_time =
                        crate::system::deterministic::deterministic_timestamp()
                            .as_secs();
                    if expiry_time > *exp {
                        return false; // Expired delegation
                    }
                }

                // Check if any delegated capability implies the required one
                delegated_capabilities.iter().any(|cap| cap.implies(other))
            }

            // No other implications
            _ => false,
        }
    }

    /// Get the fields that this capability provides access to
    pub fn accessible_fields(&self) -> Vec<FieldName> {
        match self {
            RecordCapability::ReadField(f) | RecordCapability::WriteField(f) => {
                vec![f.clone()]
            }
            RecordCapability::ProjectFields(fields) => fields.clone(),
            RecordCapability::DistributedAccess { fields, .. } => fields.clone(),
            RecordCapability::SessionDelegation {
                delegated_capabilities,
                ..
            } => delegated_capabilities
                .iter()
                .flat_map(|cap| cap.accessible_fields())
                .collect(),
            RecordCapability::FullRecordAccess => vec![], // Represents access to all fields
            _ => vec![],
        }
    }

    /// Check if this capability allows access from a specific location
    pub fn allows_access_from(&self, location: &Location) -> bool {
        match self {
            RecordCapability::DistributedAccess {
                allowed_locations, ..
            } => allowed_locations.contains(location),
            RecordCapability::SessionDelegation {
                delegated_capabilities,
                ..
            } => delegated_capabilities
                .iter()
                .any(|cap| cap.allows_access_from(location)),
            _ => true, // Local capabilities allow access from any location by default
        }
    }

    /// Get the required protocol for this capability
    pub fn required_protocol(&self) -> Option<&SessionType> {
        match self {
            RecordCapability::DistributedAccess {
                required_protocol, ..
            } => required_protocol.as_ref(),
            RecordCapability::SessionDelegation { session_type, .. } => {
                Some(session_type)
            }
            _ => None,
        }
    }
}

/// Structured capability levels for common access patterns
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum CapabilityLevel {
    /// Read-only access
    Read,
    /// Write access (implies Read)
    Write,
    /// Execute access (implies Read)
    Execute,
    /// Administrative access (implies all others)
    Admin,
    /// Distributed access with location constraints
    Distributed {
        base_level: Box<CapabilityLevel>,
        allowed_locations: BTreeSet<Location>,
    },
}

/// Enhanced capability with structured levels and record operations
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Structured capability level
    pub level: CapabilityLevel,
    /// Optional record-specific capability
    pub record_capability: Option<RecordCapability>,
    /// Location where this capability is valid
    pub valid_at: Option<Location>,
    /// Session type required for using this capability
    pub required_session: Option<SessionType>,
}

impl Capability {
    /// Create a new capability with level
    pub fn new(name: impl Into<String>, level: CapabilityLevel) -> Self {
        Self {
            name: name.into(),
            level,
            record_capability: None,
            valid_at: None,
            required_session: None,
        }
    }

    /// Create an admin capability
    pub fn admin(name: impl Into<String>) -> Self {
        Self::new(name, CapabilityLevel::Admin)
    }

    /// Create a capability with record-specific access
    pub fn with_record_capability(mut self, record_cap: RecordCapability) -> Self {
        self.record_capability = Some(record_cap);
        self
    }

    /// Create a capability valid at a specific location
    pub fn at_location(mut self, location: Location) -> Self {
        self.valid_at = Some(location);
        self
    }

    /// Create a capability that requires a specific session type
    pub fn with_session(mut self, session_type: SessionType) -> Self {
        self.required_session = Some(session_type);
        self
    }

    /// Create a distributed capability
    pub fn distributed(
        name: impl Into<String>,
        base_level: CapabilityLevel,
        locations: BTreeSet<Location>,
    ) -> Self {
        Self::new(
            name,
            CapabilityLevel::Distributed {
                base_level: Box::new(base_level),
                allowed_locations: locations,
            },
        )
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
    pub fn project_record(
        name: impl Into<String>,
        fields: Vec<impl Into<String>>,
    ) -> Self {
        Self::new(name, CapabilityLevel::Read)
            .with_record_capability(RecordCapability::project_fields(fields))
    }

    /// Create a distributed field access capability
    pub fn distributed_field_access(
        name: impl Into<String>,
        fields: Vec<impl Into<String>>,
        locations: BTreeSet<Location>,
        protocol: Option<SessionType>,
    ) -> Self {
        Self::new(name, CapabilityLevel::Read).with_record_capability(
            RecordCapability::distributed_access(fields, locations, protocol),
        )
    }

    /// Check if this capability implies another capability
    pub fn implies(&self, other: &Capability) -> bool {
        if self.name != other.name {
            return false;
        }

        // Check capability level implication
        let level_implies = self.level.implies_level(&other.level);

        // Check record capability implication
        let record_implies =
            match (&self.record_capability, &other.record_capability) {
                (None, None) => true,     // Both have no record capabilities
                (Some(_), None) => true,  // More specific implies less specific
                (None, Some(_)) => false, // Less specific doesn't imply more specific
                (Some(self_rec), Some(other_rec)) => self_rec.implies(other_rec),
            };

        // Check location validity
        let location_valid = match (&self.valid_at, &other.valid_at) {
            (None, _) => true,        // Valid everywhere implies valid anywhere
            (Some(_), None) => false, // Specific location doesn't imply everywhere
            (Some(self_loc), Some(other_loc)) => self_loc == other_loc,
        };

        level_implies && record_implies && location_valid
    }

    /// Check if this capability can be used at a specific location
    pub fn can_use_at(&self, location: &Location) -> bool {
        match &self.valid_at {
            None => true, // Valid everywhere
            Some(valid_loc) => valid_loc == location,
        }
    }

    /// Check if this capability can be delegated via a session
    pub fn can_delegate_via_session(&self, session_type: &SessionType) -> bool {
        match &self.required_session {
            None => true, // No session requirement
            Some(required) => {
                // Check if the provided session is compatible
                // In a full implementation, this would check session type compatibility
                required == session_type
            }
        }
    }

    /// Delegate this capability via a session
    pub fn delegate_via_session(
        &self,
        session_type: SessionType,
        expiration: Option<u64>,
    ) -> Capability {
        let delegated_record_cap = self.record_capability.as_ref().map(|cap| {
            RecordCapability::session_delegation(
                session_type.clone(),
                vec![cap.clone()],
                expiration,
            )
        });

        Capability {
            name: format!("delegated_{}", self.name),
            level: self.level.clone(),
            record_capability: delegated_record_cap,
            valid_at: self.valid_at.clone(),
            required_session: Some(session_type),
        }
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
            Distributed {
                base_level,
                allowed_locations,
            } => {
                let mut implied = base_level.implies();
                implied.push(Distributed {
                    base_level: base_level.clone(),
                    allowed_locations: allowed_locations.clone(),
                });
                implied
            }
        }
    }

    /// Check if this level implies another level
    pub fn implies_level(&self, other: &CapabilityLevel) -> bool {
        match (self, other) {
            // Distributed capabilities
            (
                CapabilityLevel::Distributed {
                    base_level: base1,
                    allowed_locations: locs1,
                },
                CapabilityLevel::Distributed {
                    base_level: base2,
                    allowed_locations: locs2,
                },
            ) => {
                base1.implies_level(base2)
                    && locs2.iter().all(|loc| locs1.contains(loc))
            }

            // Distributed implies base level
            (CapabilityLevel::Distributed { base_level, .. }, other) => {
                base_level.implies_level(other)
            }

            // Regular implications
            _ => self.implies().iter().any(|level| {
                match (level, other) {
                    (CapabilityLevel::Distributed { .. }, _) => false, // Skip distributed in regular check
                    _ => level == other,
                }
            }),
        }
    }
}

/// Enhanced capability set with location and session awareness
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    capabilities: BTreeSet<Capability>,
    /// Current location for capability validation
    current_location: Option<Location>,
    /// Active session types for delegation
    active_sessions: BTreeMap<String, SessionType>,
}

impl CapabilitySet {
    /// Create a new capability set
    pub fn new() -> Self {
        Self {
            capabilities: BTreeSet::new(),
            current_location: None,
            active_sessions: BTreeMap::new(),
        }
    }

    /// Create from a vector of capabilities
    pub fn from_capabilities(capabilities: Vec<Capability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
            current_location: None,
            active_sessions: BTreeMap::new(),
        }
    }

    /// Set the current location for capability validation
    pub fn at_location(mut self, location: Location) -> Self {
        self.current_location = Some(location);
        self
    }

    /// Add an active session for capability delegation
    pub fn with_session(
        mut self,
        session_id: String,
        session_type: SessionType,
    ) -> Self {
        self.active_sessions.insert(session_id, session_type);
        self
    }

    /// Add a capability to the set
    pub fn add(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
    }

    /// Check if the set has a specific capability (with implication and location checking)
    pub fn has_capability(&self, required: &Capability) -> bool {
        self.capabilities.iter().any(|cap| {
            // Check basic implication
            if !cap.implies(required) {
                return false;
            }

            // Check location constraints
            if let Some(current_loc) = &self.current_location {
                if !cap.can_use_at(current_loc) {
                    return false;
                }
            }

            // Check session requirements
            if let Some(required_session) = &required.required_session {
                // Check if we have an active session that satisfies the requirement
                return self.active_sessions.values().any(|session| {
                    cap.can_delegate_via_session(session)
                        && session == required_session
                });
            }

            true
        })
    }

    /// Check if the set has all required capabilities
    pub fn has_all_capabilities(&self, required: &[Capability]) -> bool {
        required.iter().all(|req| self.has_capability(req))
    }

    /// Verify cross-location capability access
    pub fn verify_cross_location_access(
        &self,
        field: &str,
        _from_location: &Location,
        to_location: &Location,
        required_protocol: Option<&SessionType>,
    ) -> bool {
        // Check if we have distributed access capability
        let has_distributed_cap =
            self.capabilities
                .iter()
                .any(|cap| match &cap.record_capability {
                    Some(RecordCapability::DistributedAccess {
                        fields,
                        allowed_locations,
                        required_protocol: cap_protocol,
                    }) => {
                        fields.contains(&field.to_string())
                            && allowed_locations.contains(to_location)
                            && match (required_protocol, cap_protocol) {
                                (Some(req), Some(cap)) => req == cap,
                                (None, _) => true,
                                (Some(_), None) => false,
                            }
                    }
                    _ => false,
                });

        if has_distributed_cap {
            return true;
        }

        // Check session-based delegation
        if let Some(protocol) = required_protocol {
            return self
                .active_sessions
                .values()
                .any(|session| session == protocol);
        }

        false
    }

    /// Get all capabilities in the set
    pub fn capabilities(&self) -> &BTreeSet<Capability> {
        &self.capabilities
    }

    /// Get capabilities valid at a specific location
    pub fn capabilities_at(&self, location: &Location) -> Vec<&Capability> {
        self.capabilities
            .iter()
            .filter(|cap| cap.can_use_at(location))
            .collect()
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

impl From<BTreeSet<Capability>> for CapabilitySet {
    fn from(capabilities: BTreeSet<Capability>) -> Self {
        Self {
            capabilities,
            current_location: None,
            active_sessions: BTreeMap::new(),
        }
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
            CapabilityLevel::Distributed { base_level, .. } => {
                let mut byte = 4u8;
                byte |= base_level.implies_level(&CapabilityLevel::Read) as u8;
                byte |=
                    (base_level.implies_level(&CapabilityLevel::Write) as u8) << 1;
                byte |=
                    (base_level.implies_level(&CapabilityLevel::Execute) as u8) << 2;
                byte |=
                    (base_level.implies_level(&CapabilityLevel::Admin) as u8) << 3;
                byte
            }
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
            4 => Ok(CapabilityLevel::Distributed {
                base_level: Box::new(CapabilityLevel::Read),
                allowed_locations: BTreeSet::new(),
            }),
            5 => Ok(CapabilityLevel::Distributed {
                base_level: Box::new(CapabilityLevel::Write),
                allowed_locations: BTreeSet::new(),
            }),
            6 => Ok(CapabilityLevel::Distributed {
                base_level: Box::new(CapabilityLevel::Execute),
                allowed_locations: BTreeSet::new(),
            }),
            7 => Ok(CapabilityLevel::Distributed {
                base_level: Box::new(CapabilityLevel::Admin),
                allowed_locations: BTreeSet::new(),
            }),
            _ => Err(ssz::DecodeError::BytesInvalid(
                "Invalid CapabilityLevel".to_string(),
            )),
        }
    }
}

impl Encode for Capability {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        4 + self.name.len() + 1 // Simplified for compatibility
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        (self.name.len() as u32).ssz_append(buf);
        buf.extend_from_slice(self.name.as_bytes());
        self.level.ssz_append(buf);
        // Note: Simplified encoding - full implementation would include valid_at and required_session
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

        let name =
            String::from_utf8(bytes[4..4 + name_len].to_vec()).map_err(|_| {
                ssz::DecodeError::BytesInvalid(
                    "Invalid UTF-8 in capability name".to_string(),
                )
            })?;

        let level =
            CapabilityLevel::from_ssz_bytes(&bytes[4 + name_len..4 + name_len + 1])?;

        Ok(Capability {
            name,
            level,
            record_capability: None,
            valid_at: None,
            required_session: None,
        })
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
