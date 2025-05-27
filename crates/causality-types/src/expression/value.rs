//! Value Expression System
//!
//! This module defines the data layer of the Causality type system, implementing a
//! minimalistic set of deterministic value types that can be safely used in ZK circuits.
//!
//! ValueExpr supports Null, Bool, String, Int, List, Map, Record, and Ref types only,
//! deliberately excluding floating point values to ensure complete determinism in ZK
//! execution environments.
//!
//! Values are content-addressed with IDs representing the Merkle root hash of the
//! serialized data, enabling efficient deduplication and verification.

use crate::primitive::ids::{ExprId, ValueExprId};
pub use crate::primitive::number::Number;
use crate::primitive::string::Str;
use crate::serialization::{Decode, DecodeError, DecodeWithLength, Encode, SimpleSerialize};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Wrapper Types for Breaking Recursion
//-----------------------------------------------------------------------------

/// These wrapper types break recursive type definitions for SSZ serialization.
/// They prevent infinite type recursion during serialization of self-referential
/// value structures, critical for deterministic content addressing in ZK circuits.
/// BTreeMaps are used for deterministic ordering of key-value pairs.
///
/// Box wrapper for ValueExpr
#[derive(Debug, Clone, PartialEq)]
pub struct ValueExprBox(pub Box<ValueExpr>);

impl Encode for ValueExprBox {
    fn as_ssz_bytes(&self) -> Vec<u8> { self.0.as_ssz_bytes() }
}
impl Decode for ValueExprBox {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ValueExprBox(Box::new(ValueExpr::from_ssz_bytes(bytes)?)))
    }
}
impl SimpleSerialize for ValueExprBox {}

impl DecodeWithLength for ValueExprBox {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (value_expr, consumed) = ValueExpr::from_ssz_bytes_with_length(bytes)?;
        Ok((ValueExprBox(Box::new(value_expr)), consumed))
    }
}

/// A wrapper around Vec<ValueExpr> that implements additional traits and methods
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct ValueExprVec(pub Vec<ValueExpr>);

/// A map of string keys to ValueExpr values, used for representing objects
/// and structured data.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ValueExprMap(pub BTreeMap<Str, ValueExpr>);

impl Encode for ValueExprMap {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Write the number of entries as a 4-byte prefix
        let count = self.0.len() as u32;
        result.extend_from_slice(&count.to_le_bytes());
        
        // Write each key-value pair
        for (key, value) in &self.0 {
            // Write key bytes
            let key_bytes = key.as_ssz_bytes();
            let key_len = key_bytes.len() as u32;
            result.extend_from_slice(&key_len.to_le_bytes());
            result.extend_from_slice(&key_bytes);
            
            // Write value bytes
            let value_bytes = value.as_ssz_bytes();
            let value_len = value_bytes.len() as u32;
            result.extend_from_slice(&value_len.to_le_bytes());
            result.extend_from_slice(&value_bytes);
        }
        
        result
    }
}

impl Decode for ValueExprMap {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError {
                message: format!("Invalid ValueExprMap bytes: too short ({})", bytes.len())
            });
        }
        
        // Read the count of entries
        let mut count_bytes = [0u8; 4];
        count_bytes.copy_from_slice(&bytes[0..4]);
        let count = u32::from_le_bytes(count_bytes) as usize;
        
        let mut map = BTreeMap::new();
        let mut offset = 4; // Start after the count
        
        for _ in 0..count {
            // Check if we have enough bytes for the key length
            if offset + 4 > bytes.len() {
                return Err(DecodeError {
                    message: format!("Invalid ValueExprMap: unexpected end of bytes at offset {}", offset)
                });
            }
            
            // Read key length
            let mut key_len_bytes = [0u8; 4];
            key_len_bytes.copy_from_slice(&bytes[offset..offset+4]);
            let key_len = u32::from_le_bytes(key_len_bytes) as usize;
            offset += 4;
            
            // Check if we have enough bytes for the key
            if offset + key_len > bytes.len() {
                return Err(DecodeError {
                    message: format!("Invalid ValueExprMap: key bytes exceed buffer length at offset {}", offset)
                });
            }
            
            // Read key
            let key = Str::from_ssz_bytes(&bytes[offset..offset+key_len])?;
            offset += key_len;
            
            // Check if we have enough bytes for the value length
            if offset + 4 > bytes.len() {
                return Err(DecodeError {
                    message: format!("Invalid ValueExprMap: unexpected end of bytes at offset {}", offset)
                });
            }
            
            // Read value length
            let mut value_len_bytes = [0u8; 4];
            value_len_bytes.copy_from_slice(&bytes[offset..offset+4]);
            let value_len = u32::from_le_bytes(value_len_bytes) as usize;
            offset += 4;
            
            // Check if we have enough bytes for the value
            if offset + value_len > bytes.len() {
                return Err(DecodeError {
                    message: format!("Invalid ValueExprMap: value bytes exceed buffer length at offset {}", offset)
                });
            }
            
            // Read value
            let value = ValueExpr::from_ssz_bytes(&bytes[offset..offset+value_len])?;
            offset += value_len;
            
            map.insert(key, value);
        }
        
        Ok(ValueExprMap(map))
    }
}

impl SimpleSerialize for ValueExprMap {}

impl DecodeWithLength for ValueExprMap {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError {
                message: format!("Invalid ValueExprMap bytes: too short ({})", bytes.len())
            });
        }
        
        // Read the count of entries
        let mut count_bytes = [0u8; 4];
        count_bytes.copy_from_slice(&bytes[0..4]);
        let count = u32::from_le_bytes(count_bytes) as usize;
        
        let mut map = BTreeMap::new();
        let mut offset = 4; // Start after the count
        
        for _ in 0..count {
            // Read key
            let (key, key_consumed) = Str::from_ssz_bytes_with_length(&bytes[offset..])?;
            offset += key_consumed;
            
            // Read value
            let (value, value_consumed) = ValueExpr::from_ssz_bytes_with_length(&bytes[offset..])?;
            offset += value_consumed;
            
            map.insert(key, value);
        }
        
        Ok((ValueExprMap(map), offset))
    }
}

/// Represents different kinds of references a ValueExpr can hold.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub enum ValueExprRef {
    /// Reference to another ValueExpr by its ID.
    Value(ValueExprId),
    /// Reference to a quoted an Expr (AST node) by its ID.
    Expr(ExprId),
    // TODO: Consider if other reference types are needed here, e.g., Resource(ResourceId)
}

impl Encode for ValueExprRef {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        match self {
            ValueExprRef::Value(id) => {
                result.push(0); // Tag for Value reference
                result.extend_from_slice(&id.as_ssz_bytes());
            }
            ValueExprRef::Expr(id) => {
                result.push(1); // Tag for Expr reference
                result.extend_from_slice(&id.as_ssz_bytes());
            }
        }
        
        result
    }
}

impl Decode for ValueExprRef {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Empty bytes cannot be decoded as ValueExprRef".to_string()
            });
        }
        
        let tag = bytes[0];
        let data = &bytes[1..]; // Skip the tag byte
        
        match tag {
            0 => {
                // Value reference
                let id = ValueExprId::from_ssz_bytes(data)?;
                Ok(ValueExprRef::Value(id))
            }
            1 => {
                // Expr reference
                let id = ExprId::from_ssz_bytes(data)?;
                Ok(ValueExprRef::Expr(id))
            }
            _ => {
                Err(DecodeError {
                    message: format!("Invalid ValueExprRef tag: {}", tag)
                })
            }
        }
    }
}

impl SimpleSerialize for ValueExprRef {}

impl DecodeWithLength for ValueExprRef {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Cannot decode ValueExprRef from empty bytes".to_string(),
            });
        }
        
        let tag = bytes[0];
        match tag {
            0 => {
                let (id, consumed) = ValueExprId::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExprRef::Value(id), 1 + consumed))
            }
            1 => {
                let (id, consumed) = ExprId::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExprRef::Expr(id), 1 + consumed))
            }
            _ => Err(DecodeError {
                message: format!("Invalid ValueExprRef tag: {}", tag),
            }),
        }
    }
}

/// A structured data value
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub enum ValueExpr {
    /// Alias for Unit to maintain compatibility
    Nil,

    /// Boolean value
    Bool(bool),

    /// String value (fixed-size 32 bytes)
    String(Str),

    /// Numeric value (integer, fixed-point, or ratio)
    Number(Number),

    /// List of values
    List(ValueExprVec),

    /// Key-value map
    Map(ValueExprMap),

    /// Record (structured object with named fields)
    Record(ValueExprMap),

    /// Reference to another value or expression.
    Ref(ValueExprRef),

    /// Lambda closure value.
    Lambda {
        params: Vec<Str>,
        body_expr_id: ExprId,
        captured_env: ValueExprMap, // Captures free variables from the lexical environment
    },
}

//-----------------------------------------------------------------------------
// Wrapper Implementation
//-----------------------------------------------------------------------------

// FromIterator implementation to allow collecting into ValueExprVec
impl std::iter::FromIterator<ValueExpr> for ValueExprVec {
    fn from_iter<I: IntoIterator<Item = ValueExpr>>(iter: I) -> Self {
        ValueExprVec(iter.into_iter().collect())
    }
}

// Deref implementation to allow transparent access to inner Vec
impl std::ops::Deref for ValueExprVec {
    type Target = Vec<ValueExpr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// DerefMut implementation to allow mutable access to inner Vec
impl std::ops::DerefMut for ValueExprVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// From<ValueExprVec> for Vec<ValueExpr> implementation
impl From<ValueExprVec> for Vec<ValueExpr> {
    fn from(vec: ValueExprVec) -> Self {
        vec.0
    }
}

// From<Vec<ValueExpr>> implementation
impl From<Vec<ValueExpr>> for ValueExprVec {
    fn from(vec: Vec<ValueExpr>) -> Self {
        ValueExprVec(vec)
    }
}

// Similar implementations for ValueExprMap
impl std::ops::Deref for ValueExprMap {
    type Target = BTreeMap<Str, ValueExpr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ValueExprMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ValueExprMap> for BTreeMap<Str, ValueExpr> {
    fn from(map: ValueExprMap) -> Self {
        map.0
    }
}

impl From<BTreeMap<Str, ValueExpr>> for ValueExprMap {
    fn from(map: BTreeMap<Str, ValueExpr>) -> Self {
        ValueExprMap(map)
    }
}

//-----------------------------------------------------------------------------
// Trait for converting to ValueExpr (Definition can stay if simple)
//-----------------------------------------------------------------------------
pub trait AsValueExpr {
    /// Convert to a value expression
    fn to_value_expr(&self) -> ValueExpr;
}

//-----------------------------------------------------------------------------
// From implementations
//-----------------------------------------------------------------------------

// Implement SSZ serialization for ValueExpr
impl Encode for ValueExpr {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ValueExpr::Nil => bytes.push(0),
            ValueExpr::Bool(b) => { bytes.push(1); bytes.extend(b.as_ssz_bytes()); }
            ValueExpr::String(s) => { bytes.push(2); bytes.extend(s.as_ssz_bytes()); }
            ValueExpr::Number(n) => { bytes.push(3); bytes.extend(n.as_ssz_bytes()); }
            ValueExpr::List(l) => { bytes.push(4); bytes.extend(l.as_ssz_bytes()); }
            ValueExpr::Map(m) => { bytes.push(5); bytes.extend(m.as_ssz_bytes()); }
            ValueExpr::Record(r) => { bytes.push(6); bytes.extend(r.as_ssz_bytes()); }
            ValueExpr::Ref(r) => { bytes.push(7); bytes.extend(r.as_ssz_bytes()); }
            ValueExpr::Lambda{ params, body_expr_id, captured_env } => {
                bytes.push(8);
                bytes.extend(params.as_ssz_bytes());
                bytes.extend(body_expr_id.as_ssz_bytes());
                bytes.extend(captured_env.as_ssz_bytes());
            }
        }
        bytes
    }
}

// Implement SSZ deserialization for ValueExpr
impl Decode for ValueExpr {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() { return Err(DecodeError{message: "Cannot decode ValueExpr from empty bytes".to_string()}); }
        let variant = bytes[0];
        let mut offset = 1;
        match variant {
            0 => Ok(ValueExpr::Nil),
            1 => Ok(ValueExpr::Bool(bool::from_ssz_bytes(&bytes[offset..])?)),
            2 => Ok(ValueExpr::String(Str::from_ssz_bytes(&bytes[offset..])?)),
            3 => Ok(ValueExpr::Number(Number::from_ssz_bytes(&bytes[offset..])?)),
            4 => Ok(ValueExpr::List(ValueExprVec::from_ssz_bytes(&bytes[offset..])?)),
            5 => Ok(ValueExpr::Map(ValueExprMap::from_ssz_bytes(&bytes[offset..])?)),
            6 => Ok(ValueExpr::Record(ValueExprMap::from_ssz_bytes(&bytes[offset..])?)),
            7 => Ok(ValueExpr::Ref(ValueExprRef::from_ssz_bytes(&bytes[offset..])?)),
            8 => {
                let params = Vec::<Str>::from_ssz_bytes(&bytes[offset..])?;
                offset += params.as_ssz_bytes().len();
                let body_expr_id = ExprId::from_ssz_bytes(&bytes[offset..])?;
                offset += body_expr_id.as_ssz_bytes().len();
                let captured_env = ValueExprMap::from_ssz_bytes(&bytes[offset..])?;
                Ok(ValueExpr::Lambda { params, body_expr_id, captured_env })
            }
            _ => Err(DecodeError{message: format!("Invalid ValueExpr variant: {}", variant)}),
        }
    }
}
impl SimpleSerialize for ValueExpr {}

impl DecodeWithLength for ValueExpr {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Cannot decode ValueExpr from empty bytes".to_string(),
            });
        }
        
        let variant = bytes[0];
        match variant {
            0 => Ok((ValueExpr::Nil, 1)),
            1 => {
                let (value, consumed) = bool::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::Bool(value), 1 + consumed))
            }
            2 => {
                let (value, consumed) = Str::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::String(value), 1 + consumed))
            }
            3 => {
                let (value, consumed) = Number::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::Number(value), 1 + consumed))
            }
            4 => {
                let (value, consumed) = ValueExprVec::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::List(value), 1 + consumed))
            }
            5 => {
                let (value, consumed) = ValueExprMap::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::Map(value), 1 + consumed))
            }
            6 => {
                let (value, consumed) = ValueExprMap::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::Record(value), 1 + consumed))
            }
            7 => {
                let (value, consumed) = ValueExprRef::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((ValueExpr::Ref(value), 1 + consumed))
            }
            8 => {
                // Lambda variant
                let mut offset = 1;
                
                // Read params
                let (params, params_consumed) = Vec::<Str>::from_ssz_bytes_with_length(&bytes[offset..])?;
                offset += params_consumed;
                
                // Read body_expr_id
                let (body_expr_id, body_consumed) = ExprId::from_ssz_bytes_with_length(&bytes[offset..])?;
                offset += body_consumed;
                
                // Read captured_env
                let (captured_env, env_consumed) = ValueExprMap::from_ssz_bytes_with_length(&bytes[offset..])?;
                offset += env_consumed;
                
                Ok((ValueExpr::Lambda { params, body_expr_id, captured_env }, offset))
            }
            _ => Err(DecodeError {
                message: format!("Invalid ValueExpr variant: {}", variant),
            }),
        }
    }
}

// Add manual implementations for ValueExprVec
impl Encode for ValueExprVec {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}

impl Decode for ValueExprVec {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ValueExprVec(Vec::<ValueExpr>::from_ssz_bytes(bytes)?))
    }
}

impl SimpleSerialize for ValueExprVec {}

impl DecodeWithLength for ValueExprVec {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (vec, consumed) = Vec::<ValueExpr>::from_ssz_bytes_with_length(bytes)?;
        Ok((ValueExprVec(vec), consumed))
    }
}

//-----------------------------------------------------------------------------
// Value Conversion System (from value_conversion.rs)
//-----------------------------------------------------------------------------

/// Trait for types that can be converted into a `ValueExpr`.
pub trait AsToValueExpr {
    /// Converts the type into a `ValueExpr`.
    /// This conversion should be infallible if possible, or use a specific error type.
    fn to_value_expr(&self) -> Result<ValueExpr, ValueConversionError>;
}

/// Trait for types that can be created by trying to convert from a `ValueExpr`.
pub trait AsTryFromValueExpr: Sized {
    /// Attempts to convert a `ValueExpr` into an instance of `Self`.
    fn try_from_value_expr(value: &ValueExpr) -> Result<Self, ValueConversionError>;
}

/// Error type for `ValueExpr` conversions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueConversionError {
    /// Error for when a value is of the wrong type
    InvalidType(String),

    /// Error for when a required field is missing
    MissingField(String),

    /// Error for when a value is invalid
    InvalidValue(String),

    /// General error with a message
    Other(String),
}

impl Encode for ValueConversionError {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ValueConversionError::InvalidType(msg) => {
                bytes.push(0u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ValueConversionError::MissingField(msg) => {
                bytes.push(1u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ValueConversionError::InvalidValue(msg) => {
                bytes.push(2u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ValueConversionError::Other(msg) => {
                bytes.push(3u8);
                bytes.extend(msg.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for ValueConversionError {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "ValueConversionError requires at least 1 byte for variant tag".to_string(),
            });
        }
        
        let variant = bytes[0];
        let data = &bytes[1..];
        
        match variant {
            0 => {
                let msg = String::from_ssz_bytes(data)
                    .map_err(|e| DecodeError { message: format!("Failed to decode InvalidType message: {}", e) })?;
                Ok(ValueConversionError::InvalidType(msg))
            }
            1 => {
                let msg = String::from_ssz_bytes(data)
                    .map_err(|e| DecodeError { message: format!("Failed to decode MissingField message: {}", e) })?;
                Ok(ValueConversionError::MissingField(msg))
            }
            2 => {
                let msg = String::from_ssz_bytes(data)
                    .map_err(|e| DecodeError { message: format!("Failed to decode InvalidValue message: {}", e) })?;
                Ok(ValueConversionError::InvalidValue(msg))
            }
            3 => {
                let msg = String::from_ssz_bytes(data)
                    .map_err(|e| DecodeError { message: format!("Failed to decode Other message: {}", e) })?;
                Ok(ValueConversionError::Other(msg))
            }
            _ => Err(DecodeError {
                message: format!("Invalid ValueConversionError variant: {}", variant),
            }),
        }
    }
}

impl SimpleSerialize for ValueConversionError {}

impl ValueConversionError {
    /// Creates a new general-purpose conversion error with the provided message.
    pub fn new(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl std::fmt::Display for ValueConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
            Self::MissingField(msg) => write!(f, "Missing field: {}", msg),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValueConversionError {}

impl From<String> for ValueConversionError {
    fn from(message: String) -> Self {
        ValueConversionError::Other(message)
    }
}

impl From<&str> for ValueConversionError {
    fn from(message: &str) -> Self {
        ValueConversionError::Other(message.to_string())
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::string::Str;
    use std::collections::BTreeMap;

    #[test]
    fn test_value_expr_primitive_creation() {
        // Test creating primitive ValueExpr types
        let nil = ValueExpr::Nil;
        let boolean = ValueExpr::Bool(true);
        let integer = ValueExpr::Number(crate::primitive::number::Number::Integer(42));
        let string = ValueExpr::String(Str::from("test string"));
        
        // Verify type and content
        assert!(matches!(nil, ValueExpr::Nil));
        assert!(matches!(boolean, ValueExpr::Bool(true)));
        assert!(matches!(integer, ValueExpr::Number(_)));
        assert!(matches!(string, ValueExpr::String(_)));
        
        if let ValueExpr::String(s) = string {
            assert_eq!(s.as_str(), "test string");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_value_expr_complex_types() {
        // Create a list
        let list = ValueExpr::List(vec![
            ValueExpr::Number(crate::primitive::number::Number::Integer(1)),
            ValueExpr::Number(crate::primitive::number::Number::Integer(2)),
            ValueExpr::Number(crate::primitive::number::Number::Integer(3)),
        ].into());
        
        // Create a map
        let mut map = BTreeMap::new();
        map.insert(Str::from("name"), ValueExpr::String(Str::from("Alice")));
        map.insert(Str::from("age"), ValueExpr::Number(crate::primitive::number::Number::Integer(30)));
        let map_expr = ValueExpr::Map(map.into());
        
        // Create a record
        let mut record = BTreeMap::new();
        record.insert(Str::from("id"), ValueExpr::String(Str::from("user-123")));
        record.insert(Str::from("active"), ValueExpr::Bool(true));
        let record_expr = ValueExpr::Record(record.into());
        
        // Verify list content
        if let ValueExpr::List(items) = list {
            assert_eq!(items.len(), 3);
            if let ValueExpr::Number(n) = &items[0] {
                if let crate::primitive::number::Number::Integer(i) = n {
                    assert_eq!(*i, 1);
                }
            }
        } else {
            panic!("Expected List variant");
        }
        
        // Verify map content
        if let ValueExpr::Map(m) = map_expr {
            assert_eq!(m.len(), 2);
            assert!(m.contains_key(&Str::from("name")));
            assert!(m.contains_key(&Str::from("age")));
        } else {
            panic!("Expected Map variant");
        }
        
        // Verify record content
        if let ValueExpr::Record(r) = record_expr {
            assert_eq!(r.len(), 2);
            assert!(r.contains_key(&Str::from("id")));
            assert!(r.contains_key(&Str::from("active")));
        } else {
            panic!("Expected Record variant");
        }
    }

    #[test]
    fn test_value_expr_ref_and_lambda() {
        // Create a ValueExprRef
        let ref_id = crate::primitive::ids::ValueExprId::random();
        let ref_expr = ValueExpr::Ref(ValueExprRef::Value(ref_id.clone()));
        
        // Create a Lambda
        let params = vec![Str::from("x"), Str::from("y")];
        let body_expr_id = crate::primitive::ids::ExprId::random();
        let mut captured_env = BTreeMap::new();
        captured_env.insert(Str::from("z"), ValueExpr::Number(crate::primitive::number::Number::Integer(10)));
        
        let lambda = ValueExpr::Lambda {
            params: params.clone(),
            body_expr_id: body_expr_id.clone(),
            captured_env: captured_env.clone().into(),
        };
        
        // Verify Ref content
        if let ValueExpr::Ref(r) = ref_expr {
            match r {
                ValueExprRef::Value(id) => {
                    assert_eq!(id, ref_id);
                },
                _ => panic!("Expected Value variant in Ref"),
            }
        } else {
            panic!("Expected Ref variant");
        }
        
        // Verify Lambda content
        if let ValueExpr::Lambda { params: p, body_expr_id: id, captured_env: env } = lambda {
            assert_eq!(p.len(), 2);
            assert_eq!(p[0].as_str(), "x");
            assert_eq!(p[1].as_str(), "y");
            assert_eq!(id, body_expr_id);
            assert_eq!(env.len(), 1);
            assert!(env.contains_key(&Str::from("z")));
        } else {
            panic!("Expected Lambda variant");
        }
    }

    #[test]
    fn test_value_expr_equality() {
        // Create identical ValueExpr instances
        let str1 = ValueExpr::String(Str::from("test"));
        let str2 = ValueExpr::String(Str::from("test"));
        
        let num1 = ValueExpr::Number(crate::primitive::number::Number::Integer(42));
        let num2 = ValueExpr::Number(crate::primitive::number::Number::Integer(42));
        let num3 = ValueExpr::Number(crate::primitive::number::Number::Integer(43));
        
        // Test equality
        assert_eq!(str1, str2);
        assert_eq!(num1, num2);
        assert_ne!(num1, num3);
        assert_ne!(str1, num1);
        
        // Test complex type equality
        let list1 = ValueExpr::List(vec![str1.clone(), num1.clone()].into());
        let list2 = ValueExpr::List(vec![str2.clone(), num2.clone()].into());
        let list3 = ValueExpr::List(vec![str2.clone(), num3.clone()].into());
        
        assert_eq!(list1, list2);
        assert_ne!(list1, list3);
    }

    #[test]
    fn test_value_expr_ssz_serialization() {
        use crate::serialization::{Encode, Decode};
        
        // Create a ValueExpr for serialization
        let mut map = BTreeMap::new();
        map.insert(Str::from("name"), ValueExpr::String(Str::from("Bob")));
        map.insert(Str::from("score"), ValueExpr::Number(crate::primitive::number::Number::Integer(95)));
        
        let original = ValueExpr::Record(map.into());
        
        // Serialize
        let serialized = original.as_ssz_bytes();
        
        // Deserialize
        let deserialized = ValueExpr::from_ssz_bytes(&serialized).expect("Deserialization failed");
        
        // Verify equality
        assert_eq!(original, deserialized);
        
        // Check content
        if let ValueExpr::Record(record) = deserialized {
            assert_eq!(record.len(), 2);
            
            if let Some(ValueExpr::String(name)) = record.get(&Str::from("name")) {
                assert_eq!(name.as_str(), "Bob");
            } else {
                panic!("Expected String value for 'name'");
            }
            
            if let Some(ValueExpr::Number(score)) = record.get(&Str::from("score")) {
                if let crate::primitive::number::Number::Integer(i) = score {
                    assert_eq!(*i, 95);
                } else {
                    panic!("Expected Integer Number");
                }
            } else {
                panic!("Expected Number value for 'score'");
            }
        } else {
            panic!("Expected Record variant");
        }
    }
}
