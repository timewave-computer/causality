//! Row types for structural typing with row polymorphism
//!
//! This module implements extensible records and variants using row types,
//! which enable safe and efficient structural typing. Extended with location
//! awareness for unified computation and communication.

use crate::lambda::base::{TypeInner, Location};
use crate::system::DecodeWithRemainder;
use ssz::{Decode, Encode, DecodeError};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Row Type Definitions
//-----------------------------------------------------------------------------

/// Row type represents the structure of a record with optional extension
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RowType {
    /// Named fields in the row (ordered for deterministic comparison)
    pub fields: BTreeMap<String, FieldType>,
    
    /// Optional row variable for open row types
    pub extension: Option<RowVariable>,
}

/// Field type with location and access information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FieldType {
    /// The type of the field
    pub ty: TypeInner,
    
    /// Optional location constraint for the field
    pub location: Option<Location>,
    
    /// Access permissions for the field
    pub access: FieldAccess,
}

/// Field access permissions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FieldAccess {
    /// Read-only access
    ReadOnly,
    
    /// Write-only access  
    WriteOnly,
    
    /// Read-write access
    ReadWrite,
    
    /// Linear access (consume exactly once)
    Linear,
    
    /// Location-dependent access
    LocationDependent(BTreeMap<Location, FieldAccess>),
}

/// Row variable for open row types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RowVariable {
    /// Variable name
    pub name: String,
    
    /// Optional constraint on what fields this variable can contain
    pub constraint: Option<RowConstraint>,
    
    /// Optional location constraint for the variable
    pub location_constraint: Option<LocationConstraint>,
}

/// Constraints on row variables
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RowConstraint {
    /// Variable must not contain these field names
    Lacks(Vec<String>),
    
    /// Variable must contain these field types
    Contains(BTreeMap<String, FieldType>),
    
    /// Variable must be compatible with location constraints
    LocationCompatible(LocationConstraint),
}

/// Location constraints for fields and variables
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LocationConstraint {
    /// Must be at specific location
    AtLocation(Location),
    
    /// Must be accessible from location
    AccessibleFrom(Location),
    
    /// Must be co-located with other fields
    CoLocated(Vec<String>),
    
    /// Location must be inferred
    Inferred,
}

/// A record type with a specific row
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RecordType {
    pub row: RowType,
}

//-----------------------------------------------------------------------------
// Row Operations (Compile-time only)
//-----------------------------------------------------------------------------

/// Row operation results - all computed at compile time
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowOpResult {
    /// Successful row operation with result type
    Success(TypeInner),
    
    /// Missing field error
    MissingField(String),
    
    /// Duplicate field error
    DuplicateField(String),
    
    /// Type mismatch error
    TypeMismatch {
        field: String,
        expected: TypeInner,
        found: TypeInner,
    },
    
    /// Location mismatch error
    LocationMismatch {
        field: String,
        expected: Location,
        found: Location,
    },
    
    /// Access denied error
    AccessDenied {
        field: String,
        required: FieldAccess,
        available: FieldAccess,
    },
}

impl FieldType {
    /// Create a simple field type with no location constraints
    pub fn simple(ty: TypeInner) -> Self {
        FieldType {
            ty,
            location: None,
            access: FieldAccess::ReadWrite,
        }
    }
    
    /// Create a field type at a specific location
    pub fn at_location(ty: TypeInner, location: Location) -> Self {
        FieldType {
            ty,
            location: Some(location),
            access: FieldAccess::ReadWrite,
        }
    }
    
    /// Create a linear field type (consumed exactly once)
    pub fn linear(ty: TypeInner) -> Self {
        FieldType {
            ty,
            location: None,
            access: FieldAccess::Linear,
        }
    }
    
    /// Create a read-only field type
    pub fn read_only(ty: TypeInner) -> Self {
        FieldType {
            ty,
            location: None,
            access: FieldAccess::ReadOnly,
        }
    }
    
    /// Check if this field can be accessed with the given access mode
    pub fn allows_access(&self, access: &FieldAccess) -> bool {
        match (&self.access, access) {
            (FieldAccess::ReadWrite, _) => true,
            (FieldAccess::ReadOnly, FieldAccess::ReadOnly) => true,
            (FieldAccess::WriteOnly, FieldAccess::WriteOnly) => true,
            (FieldAccess::Linear, FieldAccess::Linear) => true,
            (FieldAccess::LocationDependent(map), access) => {
                // For now, check if any location allows the access
                map.values().any(|loc_access| {
                    match (loc_access, access) {
                        (FieldAccess::ReadWrite, _) => true,
                        (a, b) if a == b => true,
                        _ => false,
                    }
                })
            }
            _ => false,
        }
    }
    
    /// Check if this field is at the specified location
    pub fn is_at_location(&self, location: &Location) -> bool {
        match &self.location {
            Some(field_loc) => field_loc == location,
            None => location == &Location::Local, // Default to local
        }
    }
    
    /// Get the location of this field
    pub fn get_location(&self) -> Location {
        self.location.clone().unwrap_or(Location::Local)
    }
}

impl RowType {
    /// Create an empty row
    pub fn empty() -> Self {
        Self {
            fields: BTreeMap::new(),
            extension: None,
        }
    }
    
    /// Create a row with fields
    pub fn with_fields(fields: BTreeMap<String, FieldType>) -> Self {
        Self {
            fields,
            extension: None,
        }
    }
    
    /// Create an open row with an extension variable
    pub fn open(fields: BTreeMap<String, FieldType>, var: RowVariable) -> Self {
        Self {
            fields,
            extension: Some(var),
        }
    }
    
    /// Create a singleton row with one field
    pub fn singleton(name: String, field_type: FieldType) -> Self {
        let mut fields = BTreeMap::new();
        fields.insert(name, field_type);
        Self::with_fields(fields)
    }
    
    /// Add a field to the row
    pub fn add_field(&mut self, name: String, field_type: FieldType) {
        self.fields.insert(name, field_type);
    }
    
    /// Get a field from the row
    pub fn get_field(&self, name: &str) -> Option<&FieldType> {
        self.fields.get(name)
    }
    
    /// Project a field from the row (compile-time operation)
    pub fn project(&self, field: &str) -> RowOpResult {
        match self.fields.get(field) {
            Some(field_type) => RowOpResult::Success(field_type.ty.clone()),
            None => RowOpResult::MissingField(field.to_string()),
        }
    }
    
    /// Project a field with access checking
    pub fn project_with_access(&self, field: &str, access: FieldAccess) -> RowOpResult {
        match self.fields.get(field) {
            Some(field_type) => {
                if field_type.allows_access(&access) {
                    RowOpResult::Success(field_type.ty.clone())
                } else {
                    RowOpResult::AccessDenied {
                        field: field.to_string(),
                        required: access,
                        available: field_type.access.clone(),
                    }
                }
            }
            None => RowOpResult::MissingField(field.to_string()),
        }
    }
    
    /// Project a field from a specific location
    pub fn project_from_location(&self, field: &str, location: &Location) -> RowOpResult {
        match self.fields.get(field) {
            Some(field_type) => {
                if field_type.is_at_location(location) {
                    RowOpResult::Success(field_type.ty.clone())
                } else {
                    RowOpResult::LocationMismatch {
                        field: field.to_string(),
                        expected: location.clone(),
                        found: field_type.get_location(),
                    }
                }
            }
            None => RowOpResult::MissingField(field.to_string()),
        }
    }
    
    /// Restrict the row by removing a field (compile-time operation)
    pub fn restrict(&self, field: &str) -> RowOpResult {
        let mut new_fields = self.fields.clone();
        
        if new_fields.remove(field).is_some() {
            let new_row = RowType {
                fields: new_fields,
                extension: self.extension.clone(),
            };
            RowOpResult::Success(TypeInner::Record(RecordType { row: new_row }))
        } else {
            RowOpResult::MissingField(field.to_string())
        }
    }
    
    /// Extend the row with a new field (compile-time operation)
    pub fn extend(&self, field: String, field_type: FieldType) -> RowOpResult {
        if self.fields.contains_key(&field) {
            return RowOpResult::DuplicateField(field);
        }
        
        let mut new_fields = self.fields.clone();
        new_fields.insert(field, field_type);
        
        let new_row = RowType {
            fields: new_fields,
            extension: self.extension.clone(),
        };
        
        RowOpResult::Success(TypeInner::Record(RecordType { row: new_row }))
    }
    
    /// Compute the difference between two rows (compile-time operation)
    pub fn diff(&self, other: &RowType) -> RowOpResult {
        let mut result_fields = BTreeMap::new();
        
        // Include fields that are in self but not in other
        for (field, field_type) in &self.fields {
            if !other.fields.contains_key(field) {
                result_fields.insert(field.clone(), field_type.clone());
            }
        }
        
        let result_row = RowType {
            fields: result_fields,
            extension: self.extension.clone(),
        };
        
        RowOpResult::Success(TypeInner::Record(RecordType { row: result_row }))
    }
    
    /// Check if this row contains all fields from another row
    pub fn contains(&self, other: &RowType) -> bool {
        other.fields.iter().all(|(field, field_type)| {
            self.fields.get(field) == Some(field_type)
        })
    }
    
    /// Merge two rows (fails if there are conflicting field types)
    pub fn merge(&self, other: &RowType) -> RowOpResult {
        let mut result_fields = self.fields.clone();
        
        for (field, field_type) in &other.fields {
            match result_fields.get(field) {
                Some(existing_field_type) if existing_field_type != field_type => {
                    return RowOpResult::TypeMismatch {
                        field: field.clone(),
                        expected: existing_field_type.ty.clone(),
                        found: field_type.ty.clone(),
                    };
                }
                None => {
                    result_fields.insert(field.clone(), field_type.clone());
                }
                _ => {} // Same field type, no conflict
            }
        }
        
        let result_row = RowType {
            fields: result_fields,
            extension: None, // Merging closes the row
        };
        
        RowOpResult::Success(TypeInner::Record(RecordType { row: result_row }))
    }
    
    /// Get all field names in the row
    pub fn field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }
    
    /// Check if the row is closed (no extension variable)
    pub fn is_closed(&self) -> bool {
        self.extension.is_none()
    }
    
    /// Check if the row is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.extension.is_none()
    }
    
    /// Get all locations referenced in this row
    pub fn get_locations(&self) -> Vec<Location> {
        let mut locations = Vec::new();
        for field_type in self.fields.values() {
            if let Some(location) = &field_type.location {
                if !locations.contains(location) {
                    locations.push(location.clone());
                }
            }
        }
        locations
    }
    
    /// Check if all fields are at the same location
    pub fn is_co_located(&self) -> bool {
        let locations = self.get_locations();
        locations.len() <= 1
    }
    
    /// Get fields at a specific location
    pub fn fields_at_location(&self, location: &Location) -> BTreeMap<String, FieldType> {
        self.fields.iter()
            .filter(|(_, field_type)| field_type.is_at_location(location))
            .map(|(name, field_type)| (name.clone(), field_type.clone()))
            .collect()
    }
    
    /// Split row by location
    pub fn split_by_location(&self) -> BTreeMap<Location, RowType> {
        let mut result = BTreeMap::new();
        
        for (field_name, field_type) in &self.fields {
            let location = field_type.get_location();
            let row = result.entry(location).or_insert_with(RowType::empty);
            row.add_field(field_name.clone(), field_type.clone());
        }
        
        result
    }
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Create a row type from field specifications
pub fn row(fields: &[(&str, TypeInner)]) -> RowType {
    let field_map = fields.iter()
        .map(|(name, ty)| (name.to_string(), FieldType::simple(ty.clone())))
        .collect();
    
    RowType::with_fields(field_map)
}

/// Create a row type with location-aware fields
pub fn location_row(fields: &[(&str, TypeInner, Location)]) -> RowType {
    let field_map = fields.iter()
        .map(|(name, ty, loc)| (name.to_string(), FieldType::at_location(ty.clone(), loc.clone())))
        .collect();
    
    RowType::with_fields(field_map)
}

/// Create an open row type with an extension variable
pub fn open_row(fields: &[(&str, TypeInner)], var_name: &str) -> RowType {
    let field_map = fields.iter()
        .map(|(name, ty)| (name.to_string(), FieldType::simple(ty.clone())))
        .collect();
    
    let var = RowVariable {
        name: var_name.to_string(),
        constraint: None,
        location_constraint: None,
    };
    
    RowType::open(field_map, var)
}

/// Create a record type from a row
pub fn record(row: RowType) -> RecordType {
    RecordType { row }
}

//-----------------------------------------------------------------------------
// Integration with Type System
//-----------------------------------------------------------------------------

impl From<RecordType> for TypeInner {
    fn from(record: RecordType) -> Self {
        TypeInner::Record(record)
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for RowType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        // Fields map serialization + optional extension
        let fields_len = 4 + self.fields.iter().map(|(k, v)| {
            4 + k.len() + v.ssz_bytes_len()
        }).sum::<usize>();
        
        let extension_len = match &self.extension {
            Some(ext) => 1 + ext.ssz_bytes_len(),
            None => 1,
        };
        
        fields_len + extension_len
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        // Serialize fields count
        (self.fields.len() as u32).ssz_append(buf);
        
        // Serialize each field
        for (name, ty) in &self.fields {
            (name.len() as u32).ssz_append(buf);
            buf.extend_from_slice(name.as_bytes());
            ty.ssz_append(buf);
        }
        
        // Serialize extension presence and data
        match &self.extension {
            Some(ext) => {
                1u8.ssz_append(buf);
                ext.ssz_append(buf);
            }
            None => {
                0u8.ssz_append(buf);
            }
        }
    }
}

impl Decode for RowType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for RowType {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError::InvalidByteLength { len: bytes.len(), expected: 4 });
        }
        
        let field_count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut offset = 4;
        let mut fields = BTreeMap::new();
        
        for _ in 0..field_count {
            if offset + 4 > bytes.len() {
                return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
            }
            
            // Read name length
            let name_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
            offset += 4;
            
            if offset + name_len > bytes.len() {
                return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: name_len });
            }
            
            // Read name
            let name = String::from_utf8(bytes[offset..offset+name_len].to_vec())
                .map_err(|e| DecodeError::BytesInvalid(format!("Invalid UTF-8 in field name: {}", e)))?;
            offset += name_len;
            
            // Read type
            let (ty, ty_remainder) = TypeInner::decode_with_remainder(&bytes[offset..])?;
            offset = bytes.len() - ty_remainder.len();
            
            fields.insert(name, FieldType {
                ty,
                location: None,
                access: FieldAccess::ReadWrite,
            });
        }
        
        if offset >= bytes.len() {
            return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 1 });
        }
        
        // Check for extension
        let has_extension = bytes[offset];
        offset += 1;
        
        let extension = if has_extension == 1 {
            let (ext, ext_remainder) = RowVariable::decode_with_remainder(&bytes[offset..])?;
            offset = bytes.len() - ext_remainder.len();
            Some(ext)
        } else {
            None
        };
        
        Ok((RowType { fields, extension }, &bytes[offset..]))
    }
}

impl Encode for RowVariable {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        4 + self.name.len() + match &self.constraint {
            Some(constraint) => 1 + constraint.ssz_bytes_len(),
            None => 1,
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        // Serialize name
        (self.name.len() as u32).ssz_append(buf);
        buf.extend_from_slice(self.name.as_bytes());
        
        // Serialize constraint
        match &self.constraint {
            Some(constraint) => {
                1u8.ssz_append(buf);
                constraint.ssz_append(buf);
            }
            None => {
                0u8.ssz_append(buf);
            }
        }
    }
}

impl Decode for RowVariable {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for RowVariable {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError::InvalidByteLength { len: bytes.len(), expected: 4 });
        }
        
        let name_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut offset = 4;
        
        if offset + name_len > bytes.len() {
            return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: name_len });
        }
        
        let name = String::from_utf8(bytes[offset..offset+name_len].to_vec())
            .map_err(|e| DecodeError::BytesInvalid(format!("Invalid UTF-8 in variable name: {}", e)))?;
        offset += name_len;
        
        if offset >= bytes.len() {
            return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 1 });
        }
        
        let has_constraint = bytes[offset];
        offset += 1;
        
        let constraint = if has_constraint == 1 {
            let (cons, cons_remainder) = RowConstraint::decode_with_remainder(&bytes[offset..])?;
            offset = bytes.len() - cons_remainder.len();
            Some(cons)
        } else {
            None
        };
        
        Ok((RowVariable { name, constraint, location_constraint: None }, &bytes[offset..]))
    }
}

impl Encode for FieldType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        // Type serialization + optional location + access permissions
        let ty_len = self.ty.ssz_bytes_len();
        let location_len = match &self.location {
            Some(loc) => 1 + loc.ssz_bytes_len(),
            None => 1,
        };
        let access_len = match &self.access {
            FieldAccess::ReadOnly => 1,
            FieldAccess::WriteOnly => 1,
            FieldAccess::ReadWrite => 1,
            FieldAccess::Linear => 1,
            FieldAccess::LocationDependent(map) => 1 + 4 + map.iter().map(|(k, v)| 4 + k.ssz_bytes_len() + v.ssz_bytes_len()).sum::<usize>(),
        };
        ty_len + location_len + access_len
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.ty.ssz_append(buf);
        
        match &self.location {
            Some(loc) => {
                1u8.ssz_append(buf);
                loc.ssz_append(buf);
            }
            None => {
                0u8.ssz_append(buf);
            }
        }
        
        match &self.access {
            FieldAccess::ReadOnly => 0u8.ssz_append(buf),
            FieldAccess::WriteOnly => 1u8.ssz_append(buf),
            FieldAccess::ReadWrite => 2u8.ssz_append(buf),
            FieldAccess::Linear => 3u8.ssz_append(buf),
            FieldAccess::LocationDependent(map) => {
                4u8.ssz_append(buf);
                (map.len() as u32).ssz_append(buf);
                for (loc, access) in map {
                    loc.ssz_append(buf);
                    access.ssz_append(buf);
                }
            }
        }
    }
}

impl Decode for FieldType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for FieldType {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        let (ty, ty_remainder) = TypeInner::decode_with_remainder(bytes)?;
        let mut offset = bytes.len() - ty_remainder.len();
        
        let location = if offset > 0 && bytes[offset-1] == 1 {
            let (loc, loc_remainder) = Location::decode_with_remainder(&bytes[..offset-1])?;
            let _offset = bytes.len() - loc_remainder.len();
            Some(loc)
        } else {
            None
        };
        
        let access = if offset > 0 && bytes[offset-1] == 1 {
            let (acc, acc_remainder) = FieldAccess::decode_with_remainder(&bytes[..offset-1])?;
            offset = bytes.len() - acc_remainder.len();
            acc
        } else {
            FieldAccess::ReadWrite
        };
        
        Ok((FieldType { ty, location, access }, &bytes[offset..]))
    }
}

impl Encode for FieldAccess {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            FieldAccess::ReadOnly => 0,
            FieldAccess::WriteOnly => 0,
            FieldAccess::ReadWrite => 0,
            FieldAccess::Linear => 0,
            FieldAccess::LocationDependent(map) => 1 + 4 + map.iter().map(|(k, v)| 4 + k.ssz_bytes_len() + v.ssz_bytes_len()).sum::<usize>(),
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            FieldAccess::ReadOnly => 0u8.ssz_append(buf),
            FieldAccess::WriteOnly => 1u8.ssz_append(buf),
            FieldAccess::ReadWrite => 2u8.ssz_append(buf),
            FieldAccess::Linear => 3u8.ssz_append(buf),
            FieldAccess::LocationDependent(map) => {
                4u8.ssz_append(buf);
                (map.len() as u32).ssz_append(buf);
                for (loc, access) in map {
                    loc.ssz_append(buf);
                    access.ssz_append(buf);
                }
            }
        }
    }
}

impl Decode for FieldAccess {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for FieldAccess {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::InvalidByteLength { len: 0, expected: 1 });
        }
        
        let variant_tag = bytes[0];
        let mut offset = 1;

        match variant_tag {
            0 => Ok((FieldAccess::ReadOnly, &bytes[offset..])),
            1 => Ok((FieldAccess::WriteOnly, &bytes[offset..])),
            2 => Ok((FieldAccess::ReadWrite, &bytes[offset..])),
            3 => Ok((FieldAccess::Linear, &bytes[offset..])),
            4 => {
                if offset + 4 > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                }
                
                let field_count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                let mut map = BTreeMap::new();
                for _ in 0..field_count {
                    let (location, loc_remainder) = Location::decode_with_remainder(&bytes[offset..])?;
                    let _offset = bytes.len() - loc_remainder.len();
                    
                    let (access, access_remainder) = FieldAccess::decode_with_remainder(loc_remainder)?;
                    offset = bytes.len() - access_remainder.len();
                    
                    map.insert(location, access);
                }
                Ok((FieldAccess::LocationDependent(map), &bytes[offset..]))
            }
            _ => Err(DecodeError::BytesInvalid(format!("Invalid FieldAccess variant: {}", variant_tag))),
        }
    }
}

impl Encode for RowConstraint {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            RowConstraint::Lacks(fields) => {
                4 + fields.iter().map(|f| 4 + f.len()).sum::<usize>()
            }
            RowConstraint::Contains(fields) => {
                4 + fields.iter().map(|(k, v)| 4 + k.len() + v.ssz_bytes_len()).sum::<usize>()
            }
            RowConstraint::LocationCompatible(constraint) => {
                1 + constraint.ssz_bytes_len()
            }
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            RowConstraint::Lacks(fields) => {
                0u8.ssz_append(buf);
                (fields.len() as u32).ssz_append(buf);
                for field in fields {
                    (field.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(field.as_bytes());
                }
            }
            RowConstraint::Contains(fields) => {
                1u8.ssz_append(buf);
                (fields.len() as u32).ssz_append(buf);
                for (name, field_type) in fields {
                    (name.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(name.as_bytes());
                    field_type.ssz_append(buf);
                }
            }
            RowConstraint::LocationCompatible(constraint) => {
                2u8.ssz_append(buf);
                constraint.ssz_append(buf);
            }
        }
    }
}

impl Decode for RowConstraint {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for RowConstraint {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::InvalidByteLength { len: 0, expected: 1 });
        }
        
        let variant_tag = bytes[0];
        let mut offset = 1;

        match variant_tag {
            0 => { // Lacks
                if offset + 4 > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                }
                
                let field_count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                let mut fields = Vec::new();
                for _ in 0..field_count {
                    if offset + 4 > bytes.len() {
                        return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                    }
                    
                    let name_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                    offset += 4;
                    
                    if offset + name_len > bytes.len() {
                        return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: name_len });
                    }
                    
                    let name = String::from_utf8(bytes[offset..offset+name_len].to_vec())
                        .map_err(|e| DecodeError::BytesInvalid(format!("Invalid UTF-8 in field name: {}", e)))?;
                    offset += name_len;
                    
                    fields.push(name);
                }
                Ok((RowConstraint::Lacks(fields), &bytes[offset..]))
            }
            1 => { // Contains  
                if offset + 4 > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                }
                
                let field_count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                let mut fields = BTreeMap::new();
                for _ in 0..field_count {
                    if offset + 4 > bytes.len() {
                        return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                    }
                    
                    let name_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                    offset += 4;
                    
                    if offset + name_len > bytes.len() {
                        return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: name_len });
                    }
                    
                    let name = String::from_utf8(bytes[offset..offset+name_len].to_vec())
                        .map_err(|e| DecodeError::BytesInvalid(format!("Invalid UTF-8 in field name: {}", e)))?;
                    offset += name_len;
                    
                    let (field_type, field_type_remainder) = FieldType::decode_with_remainder(&bytes[offset..])?;
                    offset = bytes.len() - field_type_remainder.len();
                    
                    fields.insert(name, field_type);
                }
                Ok((RowConstraint::Contains(fields), &bytes[offset..]))
            }
            2 => { // LocationCompatible
                let (constraint, remainder) = LocationConstraint::decode_with_remainder(&bytes[offset..])?;
                Ok((RowConstraint::LocationCompatible(constraint), remainder))
            }
            _ => Err(DecodeError::BytesInvalid(format!("Invalid RowConstraint variant: {}", variant_tag))),
        }
    }
}

impl Encode for LocationConstraint {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            LocationConstraint::AtLocation(loc) => loc.ssz_bytes_len(),
            LocationConstraint::AccessibleFrom(loc) => loc.ssz_bytes_len(),
            LocationConstraint::CoLocated(fields) => 4 + fields.iter().map(|f| 4 + f.len()).sum::<usize>(),
            LocationConstraint::Inferred => 0,
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match self {
            LocationConstraint::AtLocation(loc) => {
                0u8.ssz_append(buf);
                loc.ssz_append(buf);
            }
            LocationConstraint::AccessibleFrom(loc) => {
                1u8.ssz_append(buf);
                loc.ssz_append(buf);
            }
            LocationConstraint::CoLocated(fields) => {
                2u8.ssz_append(buf);
                (fields.len() as u32).ssz_append(buf);
                for field in fields {
                    (field.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(field.as_bytes());
                }
            }
            LocationConstraint::Inferred => {
                3u8.ssz_append(buf);
            }
        }
    }
}

impl Decode for LocationConstraint {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (result, remainder) = Self::decode_with_remainder(bytes)?;
        if !remainder.is_empty() {
            return Err(DecodeError::BytesInvalid("Trailing bytes after decoding".to_string()));
        }
        Ok(result)
    }
}

impl DecodeWithRemainder for LocationConstraint {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::InvalidByteLength { len: 0, expected: 1 });
        }
        
        let variant_tag = bytes[0];
        let mut offset = 1;

        match variant_tag {
            0 => { // AtLocation
                let (loc, remainder) = Location::decode_with_remainder(&bytes[offset..])?;
                Ok((LocationConstraint::AtLocation(loc), remainder))
            }
            1 => { // AccessibleFrom
                let (loc, remainder) = Location::decode_with_remainder(&bytes[offset..])?;
                Ok((LocationConstraint::AccessibleFrom(loc), remainder))
            }
            2 => { // CoLocated
                if offset + 4 > bytes.len() {
                    return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                }
                
                let field_count = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                let mut fields = Vec::new();
                for _ in 0..field_count {
                    if offset + 4 > bytes.len() {
                        return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: 4 });
                    }
                    
                    let name_len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                    offset += 4;
                    
                    if offset + name_len > bytes.len() {
                        return Err(DecodeError::InvalidByteLength { len: bytes.len() - offset, expected: name_len });
                    }
                    
                    let name = String::from_utf8(bytes[offset..offset+name_len].to_vec())
                        .map_err(|e| DecodeError::BytesInvalid(format!("Invalid UTF-8 in field name: {}", e)))?;
                    offset += name_len;
                    
                    fields.push(name);
                }
                Ok((LocationConstraint::CoLocated(fields), &bytes[offset..]))
            }
            3 => { // Inferred
                Ok((LocationConstraint::Inferred, &bytes[offset..]))
            }
            _ => Err(DecodeError::BytesInvalid(format!("Invalid LocationConstraint variant: {}", variant_tag))),
        }
    }
}

impl Encode for RecordType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        self.row.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.row.ssz_append(buf);
    }
}

impl Decode for RecordType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let row = RowType::from_ssz_bytes(bytes)?;
        Ok(RecordType { row })
    }
}

impl DecodeWithRemainder for RecordType {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        let (row, remainder) = RowType::decode_with_remainder(bytes)?;
        Ok((RecordType { row }, remainder))
    }
}

impl Default for RecordType {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordType {
    /// Create a new empty record type
    pub fn new() -> Self {
        Self {
            row: RowType::empty(),
        }
    }
    
    /// Create a record type from a row
    pub fn from_row(row: RowType) -> Self {
        Self { row }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::BaseType;
    
    fn int_type() -> TypeInner {
        TypeInner::Base(BaseType::Int)
    }
    
    fn bool_type() -> TypeInner {
        TypeInner::Base(BaseType::Bool)
    }
    
    fn string_type() -> TypeInner {
        TypeInner::Base(BaseType::Symbol)
    }
    
    #[test]
    fn test_empty_row() {
        let row = RowType::empty();
        assert!(row.is_empty());
        assert!(row.is_closed());
        assert_eq!(row.field_names().len(), 0);
    }
    
    #[test]
    fn test_row_with_fields() {
        let fields = vec![
            ("name", string_type()),
            ("age", int_type()),
            ("active", bool_type()),
        ];
        let row = row(&fields);
        
        assert!(!row.is_empty());
        assert!(row.is_closed());
        assert_eq!(row.field_names().len(), 3);
        assert!(row.field_names().contains(&&"name".to_string()));
        assert!(row.field_names().contains(&&"age".to_string()));
        assert!(row.field_names().contains(&&"active".to_string()));
    }
    
    #[test]
    fn test_row_projection() {
        let fields = vec![
            ("name", string_type()),
            ("age", int_type()),
        ];
        let row = row(&fields);
        
        // Successful projection
        match row.project("name") {
            RowOpResult::Success(ty) => assert_eq!(ty, string_type()),
            _ => panic!("Expected successful projection"),
        }
        
        // Missing field
        match row.project("missing") {
            RowOpResult::MissingField(field) => assert_eq!(field, "missing"),
            _ => panic!("Expected missing field error"),
        }
    }
    
    #[test]
    fn test_open_row() {
        let fields = vec![
            ("name", string_type()),
        ];
        let row = open_row(&fields, "rest");
        
        assert!(!row.is_closed());
        assert_eq!(row.extension.as_ref().unwrap().name, "rest");
        assert!(row.extension.as_ref().unwrap().constraint.is_none());
    }
} 