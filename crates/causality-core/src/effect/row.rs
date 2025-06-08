//! Row types for structural typing with row polymorphism
//!
//! This module implements extensible records and variants using row types,
//! which enable safe and efficient structural typing.

use crate::lambda::base::TypeInner;
use crate::system::DecodeWithRemainder;
use ssz::{Decode, Encode, DecodeError};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Row Type Definitions
//-----------------------------------------------------------------------------

/// A row type represents an extensible record with named fields
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RowType {
    /// Named fields in the row (ordered for deterministic comparison)
    pub fields: BTreeMap<String, TypeInner>,
    
    /// Optional row variable for open row types
    pub extension: Option<RowVariable>,
}

/// A row variable represents an unknown extension to a row type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RowVariable {
    /// Variable name
    pub name: String,
    
    /// Optional constraint on what fields this variable can contain
    pub constraint: Option<RowConstraint>,
}

/// Constraints on row variables
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RowConstraint {
    /// Variable must not contain these field names
    Lacks(Vec<String>),
    
    /// Variable must contain these field types
    Contains(BTreeMap<String, TypeInner>),
}

/// A record type with a specific row
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn with_fields(fields: BTreeMap<String, TypeInner>) -> Self {
        Self {
            fields,
            extension: None,
        }
    }
    
    /// Create an open row with an extension variable
    pub fn open(fields: BTreeMap<String, TypeInner>, var: RowVariable) -> Self {
        Self {
            fields,
            extension: Some(var),
        }
    }
    
    /// Project a field from the row (compile-time operation)
    pub fn project(&self, field: &str) -> RowOpResult {
        match self.fields.get(field) {
            Some(ty) => RowOpResult::Success(ty.clone()),
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
    pub fn extend(&self, field: String, ty: TypeInner) -> RowOpResult {
        if self.fields.contains_key(&field) {
            return RowOpResult::DuplicateField(field);
        }
        
        let mut new_fields = self.fields.clone();
        new_fields.insert(field, ty);
        
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
        for (field, ty) in &self.fields {
            if !other.fields.contains_key(field) {
                result_fields.insert(field.clone(), ty.clone());
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
        other.fields.iter().all(|(field, ty)| {
            self.fields.get(field) == Some(ty)
        })
    }
    
    /// Merge two rows (fails if there are conflicting field types)
    pub fn merge(&self, other: &RowType) -> RowOpResult {
        let mut result_fields = self.fields.clone();
        
        for (field, ty) in &other.fields {
            match result_fields.get(field) {
                Some(existing_ty) if existing_ty != ty => {
                    return RowOpResult::TypeMismatch {
                        field: field.clone(),
                        expected: existing_ty.clone(),
                        found: ty.clone(),
                    };
                }
                None => {
                    result_fields.insert(field.clone(), ty.clone());
                }
                _ => {} // Same type, no conflict
            }
        }
        
        let result_row = RowType {
            fields: result_fields,
            extension: None, // Merging closes the row
        };
        
        RowOpResult::Success(TypeInner::Record(RecordType { row: result_row }))
    }
    
    /// Get all field names in the row
    pub fn field_names(&self) -> Vec<&String> {
        self.fields.keys().collect()
    }
    
    /// Check if the row is closed (no extension variable)
    pub fn is_closed(&self) -> bool {
        self.extension.is_none()
    }
    
    /// Check if the row is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.extension.is_none()
    }
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Create a row type from field specifications
pub fn row(fields: &[(&str, TypeInner)]) -> RowType {
    let field_map = fields.iter()
        .map(|(name, ty)| (name.to_string(), ty.clone()))
        .collect();
    
    RowType::with_fields(field_map)
}

/// Create an open row type with an extension variable
pub fn open_row(fields: &[(&str, TypeInner)], var_name: &str) -> RowType {
    let field_map = fields.iter()
        .map(|(name, ty)| (name.to_string(), ty.clone()))
        .collect();
    
    let var = RowVariable {
        name: var_name.to_string(),
        constraint: None,
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
            
            fields.insert(name, ty);
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
        
        Ok((RowVariable { name, constraint }, &bytes[offset..]))
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
                for (name, ty) in fields {
                    (name.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(name.as_bytes());
                    ty.ssz_append(buf);
                }
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
                    
                    let (ty, ty_remainder) = TypeInner::decode_with_remainder(&bytes[offset..])?;
                    offset = bytes.len() - ty_remainder.len();
                    
                    fields.insert(name, ty);
                }
                Ok((RowConstraint::Contains(fields), &bytes[offset..]))
            }
            _ => Err(DecodeError::BytesInvalid(format!("Invalid RowConstraint variant: {}", variant_tag))),
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