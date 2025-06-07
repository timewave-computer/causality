//! Base types for Layer 1: Linear Lambda Calculus
//!
//! These are the fundamental types that form the basis of the type system
//! for the linear lambda calculus layer. All higher-level types are built 
//! from these primitives.

use crate::{
    system::{
        content_addressing::{EntityId, ContentAddressable}, 
        DecodeWithRemainder,
    },
    effect::row::{RecordType, RowType},
};
use ssz::{Decode, Encode, DecodeError};
use std::marker::PhantomData;
use serde::{Serialize, Deserialize};

//-----------------------------------------------------------------------------
// Type Definitions
//-----------------------------------------------------------------------------

/// Phantom type for linear resources (use exactly once)
pub struct Linear;

/// Phantom type for affine resources (use at most once)
pub struct Affine;

/// Phantom type for relevant resources (use at least once)
pub struct Relevant;

/// Phantom type for unrestricted resources (use any number of times)
pub struct Unrestricted;

//-----------------------------------------------------------------------------
// Base Type Enum
//-----------------------------------------------------------------------------

/// Base types in the Causality type system
/// 
/// These are the primitive value types that can be stored directly 
/// in registers and manipulated by the register machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    /// Unit type - carries no information
    Unit,
    
    /// Boolean type - 1-bit boolean, mapped to u8 in memory, ZK-native
    Bool,
    
    /// Integer type - u32 RISC-V native word size, unsigned for ZK efficiency
    Int,
    
    /// Symbol type - ZK-compatible interned identifiers
    Symbol,
}

// Use the macro for SSZ implementation
crate::impl_ssz_for_unit_enum!(BaseType,
    Unit => 0,
    Bool => 1,
    Int => 2,
    Symbol => 3
);

//-----------------------------------------------------------------------------
// Core Type with Linearity Tracking
//-----------------------------------------------------------------------------

/// Core type expressions with linearity tracking via phantom types.
/// This is the foundation of the type system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type<L = Linear> {
    pub inner: TypeInner,
    pub _phantom: PhantomData<L>,
}

/// The actual type structure without linearity information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeInner {
    /// Base primitive type
    Base(BaseType),
    
    /// Linear product type (τ₁ ⊗ τ₂)
    Product(Box<TypeInner>, Box<TypeInner>),
    
    /// Sum type (τ₁ ⊕ τ₂)
    Sum(Box<TypeInner>, Box<TypeInner>),
    
    /// Linear function type (τ₁ ⊸ τ₂)
    LinearFunction(Box<TypeInner>, Box<TypeInner>),
    
    /// Record type with row polymorphism
    Record(RecordType),
}

// Manual SSZ implementation for TypeInner
impl Encode for TypeInner {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            TypeInner::Base(base) => base.ssz_bytes_len(),
            TypeInner::Product(left, right) |
            TypeInner::Sum(left, right) |
            TypeInner::LinearFunction(left, right) => {
                left.ssz_bytes_len() + right.ssz_bytes_len()
            }
            TypeInner::Record(record) => record.ssz_bytes_len(),
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        use crate::system::encode_enum_variant;
        
        match self {
            TypeInner::Base(base) => {
                encode_enum_variant(0, buf);
                base.ssz_append(buf);
            }
            TypeInner::Product(left, right) => {
                encode_enum_variant(1, buf);
                left.ssz_append(buf);
                right.ssz_append(buf);
            }
            TypeInner::Sum(left, right) => {
                encode_enum_variant(2, buf);
                left.ssz_append(buf);
                right.ssz_append(buf);
            }
            TypeInner::LinearFunction(left, right) => {
                encode_enum_variant(3, buf);
                left.ssz_append(buf);
                right.ssz_append(buf);
            }
            TypeInner::Record(record) => {
                encode_enum_variant(4, buf);
                record.ssz_append(buf);
            }
        }
    }
}

impl Decode for TypeInner {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        use crate::system::decode_enum_variant;
        
        let (variant, data) = decode_enum_variant(bytes)?;
        
        match variant {
            0 => {
                let base = BaseType::from_ssz_bytes(data)?;
                Ok(TypeInner::Base(base))
            }
            1 => {
                let (left, remaining) = TypeInner::decode_with_remainder(data)?;
                let (right, _) = TypeInner::decode_with_remainder(remaining)?;
                Ok(TypeInner::Product(Box::new(left), Box::new(right)))
            }
            2 => {
                let (left, remaining) = TypeInner::decode_with_remainder(data)?;
                let (right, _) = TypeInner::decode_with_remainder(remaining)?;
                Ok(TypeInner::Sum(Box::new(left), Box::new(right)))
            }
            3 => {
                let (left, remaining) = TypeInner::decode_with_remainder(data)?;
                let (right, _) = TypeInner::decode_with_remainder(remaining)?;
                Ok(TypeInner::LinearFunction(Box::new(left), Box::new(right)))
            }
            4 => {
                let record = RecordType::from_ssz_bytes(data)?;
                Ok(TypeInner::Record(record))
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid TypeInner variant: {}", variant).into()
            )),
        }
    }
}

// Implement DecodeWithRemainder for TypeInner
impl DecodeWithRemainder for TypeInner {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        use crate::system::decode_enum_variant;
        
        let (variant, data) = decode_enum_variant(bytes)?;
        
        match variant {
            0 => {
                // Base type is fixed length (1 byte)
                if data.is_empty() {
                    return Err(DecodeError::InvalidByteLength {
                        len: 0,
                        expected: 1,
                    });
                }
                let base = BaseType::from_ssz_bytes(&data[..1])?;
                Ok((TypeInner::Base(base), &data[1..]))
            }
            1 | 2 | 3 => {
                // Product, Sum, and LinearFunction all have two TypeInner children
                let (left, remaining) = Self::decode_with_remainder(data)?;
                let (right, remaining) = Self::decode_with_remainder(remaining)?;
                
                let result = match variant {
                    1 => TypeInner::Product(Box::new(left), Box::new(right)),
                    2 => TypeInner::Sum(Box::new(left), Box::new(right)),
                    3 => TypeInner::LinearFunction(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                };
                
                Ok((result, remaining))
            }
            4 => {
                let record = RecordType::from_ssz_bytes(data)?;
                let record_len = record.ssz_bytes_len();
                Ok((TypeInner::Record(record), &data[record_len..]))
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid TypeInner variant: {}", variant).into()
            )),
        }
    }
}

//-----------------------------------------------------------------------------
// Type Constructor Helpers
//-----------------------------------------------------------------------------

impl<L> Type<L> {
    /// Create a new type with the given inner structure
    pub fn new(inner: TypeInner) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
    
    /// Get the inner type structure
    pub fn inner(&self) -> &TypeInner {
        &self.inner
    }
    
    /// Get the content ID of this type
    pub fn type_id(&self) -> EntityId {
        self.inner.content_id()
    }
}

impl Type<Linear> {
    /// Create a Unit type
    pub fn unit() -> Self {
        Self::new(TypeInner::Base(BaseType::Unit))
    }
    
    /// Create a Bool type
    pub fn bool() -> Self {
        Self::new(TypeInner::Base(BaseType::Bool))
    }
    
    /// Create an Int type
    pub fn int() -> Self {
        Self::new(TypeInner::Base(BaseType::Int))
    }
    
    /// Create a Symbol type
    pub fn symbol() -> Self {
        Self::new(TypeInner::Base(BaseType::Symbol))
    }
    
    /// Create a product type
    pub fn product(left: TypeInner, right: TypeInner) -> Self {
        Self::new(TypeInner::Product(Box::new(left), Box::new(right)))
    }
    
    /// Create a sum type
    pub fn sum(left: TypeInner, right: TypeInner) -> Self {
        Self::new(TypeInner::Sum(Box::new(left), Box::new(right)))
    }
    
    /// Create a linear function type
    pub fn linear_function(input: TypeInner, output: TypeInner) -> Self {
        Self::new(TypeInner::LinearFunction(Box::new(input), Box::new(output)))
    }
}

//-----------------------------------------------------------------------------
// Runtime Value Types
//-----------------------------------------------------------------------------

/// Runtime values corresponding to the type system
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// Unit value
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value
    Int(u32),
    
    /// Symbol value
    Symbol(crate::system::Str),
    
    /// String value
    String(crate::system::Str),
    
    /// Product value (pair)
    Product(Box<Value>, Box<Value>),
    
    /// Sum value (tagged union)
    Sum {
        tag: u8,
        value: Box<Value>,
    },
    
    /// Record value (extensible record)
    Record {
        fields: std::collections::BTreeMap<String, Value>,
    },
}

// Manual SSZ implementation for Value
impl Encode for Value {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            Value::Unit => 0,
            Value::Bool(_) => 1,
            Value::Int(_) => 4,
            Value::Symbol(s) => s.ssz_bytes_len(),
            Value::String(s) => s.ssz_bytes_len(),
            Value::Product(left, right) => left.ssz_bytes_len() + right.ssz_bytes_len(),
            Value::Sum { tag: _, value } => 1 + value.ssz_bytes_len(),
            Value::Record { fields } => {
                4 + fields.iter().map(|(key, value)| {
                    4 + key.len() + value.ssz_bytes_len()
                }).sum::<usize>()
            }
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        use crate::system::encode_enum_variant;
        
        match self {
            Value::Unit => encode_enum_variant(0, buf),
            Value::Bool(b) => {
                encode_enum_variant(1, buf);
                buf.push(if *b { 1u8 } else { 0u8 });
            }
            Value::Int(i) => {
                encode_enum_variant(2, buf);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            Value::Symbol(s) => {
                encode_enum_variant(3, buf);
                s.ssz_append(buf);
            }
            Value::String(s) => {
                encode_enum_variant(4, buf);
                s.ssz_append(buf);
            }
            Value::Product(left, right) => {
                encode_enum_variant(5, buf);
                left.ssz_append(buf);
                right.ssz_append(buf);
            }
            Value::Sum { tag, value } => {
                encode_enum_variant(6, buf);
                buf.push(*tag);
                value.ssz_append(buf);
            }
            Value::Record { fields } => {
                encode_enum_variant(7, buf);
                (fields.len() as u32).ssz_append(buf);
                for (key, value) in fields {
                    (key.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(key.as_bytes());
                    value.ssz_append(buf);
                }
            }
        }
    }
}

impl Decode for Value {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        use crate::system::decode_enum_variant;
        
        let (variant, data) = decode_enum_variant(bytes)?;
        
        match variant {
            0 => Ok(Value::Unit),
            1 => {
                if data.is_empty() {
                    return Err(DecodeError::InvalidByteLength {
                        len: 0,
                        expected: 1,
                    });
                }
                Ok(Value::Bool(data[0] != 0))
            }
            2 => {
                if data.len() < 4 {
                    return Err(DecodeError::InvalidByteLength {
                        len: data.len(),
                        expected: 4,
                    });
                }
                let bytes: [u8; 4] = crate::system::decode_fixed_bytes(&data[..4])?;
                Ok(Value::Int(u32::from_le_bytes(bytes)))
            }
            3 => {
                let s = crate::system::Str::from_ssz_bytes(data)?;
                Ok(Value::Symbol(s))
            }
            4 => {
                let s = crate::system::Str::from_ssz_bytes(data)?;
                let s_len = s.ssz_bytes_len();
                Ok(Value::String(s))
            }
            5 => {
                let (left, remaining) = Value::decode_with_remainder(data)?;
                let (right, _) = Value::decode_with_remainder(remaining)?;
                Ok(Value::Product(Box::new(left), Box::new(right)))
            }
            6 => {
                if data.is_empty() {
                    return Err(DecodeError::InvalidByteLength {
                        len: 0,
                        expected: 1,
                    });
                }
                let tag = data[0];
                let (value, _) = Value::decode_with_remainder(&data[1..])?;
                Ok(Value::Sum {
                    tag,
                    value: Box::new(value),
                })
            }
            7 => {
                let field_count = u32::from_ssz_bytes(&data[0..4])? as usize;
                let mut offset = 4;
                let mut fields = std::collections::BTreeMap::new();
                
                for _ in 0..field_count {
                    // Decode key length
                    let key_len = u32::from_ssz_bytes(&data[offset..offset + 4])? as usize;
                    offset += 4;
                    
                    // Decode key
                    let key = String::from_utf8(data[offset..offset + key_len].to_vec())
                        .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8 in field name".into()))?;
                    offset += key_len;
                    
                    // Decode value
                    let (value, remaining_after_value) = Value::decode_with_remainder(&data[offset..])?;
                    offset = data.len() - remaining_after_value.len();
                    
                    fields.insert(key, value);
                }
                Ok(Value::Record { fields })
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid Value variant: {}", variant).into()
            )),
        }
    }
}

// Implement DecodeWithRemainder for Value
impl DecodeWithRemainder for Value {
    fn decode_with_remainder(bytes: &[u8]) -> Result<(Self, &[u8]), DecodeError> {
        use crate::system::decode_enum_variant;
        
        let (variant, data) = decode_enum_variant(bytes)?;
        
        match variant {
            0 => Ok((Value::Unit, data)),
            1 => {
                if data.is_empty() {
                    return Err(DecodeError::InvalidByteLength {
                        len: 0,
                        expected: 1,
                    });
                }
                Ok((Value::Bool(data[0] != 0), &data[1..]))
            }
            2 => {
                if data.len() < 4 {
                    return Err(DecodeError::InvalidByteLength {
                        len: data.len(),
                        expected: 4,
                    });
                }
                let bytes: [u8; 4] = crate::system::decode_fixed_bytes(&data[..4])?;
                Ok((Value::Int(u32::from_le_bytes(bytes)), &data[4..]))
            }
            3 => {
                // For Str, we need to determine its length
                // Str is encoded as length (4 bytes) + data
                let (str_data, remaining) = crate::system::decode_with_length(data)?;
                let value = String::from_utf8(str_data.to_vec())
                    .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8".into()))?;
                let s = crate::system::Str { value };
                Ok((Value::Symbol(s), remaining))
            }
            4 => {
                let s = crate::system::Str::from_ssz_bytes(data)?;
                let s_len = s.ssz_bytes_len();
                Ok((Value::String(s), &data[s_len..]))
            }
            5 => {
                let (left, remaining) = Value::decode_with_remainder(data)?;
                let (right, _) = Value::decode_with_remainder(remaining)?;
                Ok((Value::Product(Box::new(left), Box::new(right)), remaining))
            }
            6 => {
                if data.is_empty() {
                    return Err(DecodeError::InvalidByteLength {
                        len: 0,
                        expected: 1,
                    });
                }
                let tag = data[0];
                let (value, _) = Value::decode_with_remainder(&data[1..])?;
                Ok((Value::Sum {
                    tag,
                    value: Box::new(value),
                }, &data[1..]))
            }
            7 => {
                let field_count = u32::from_ssz_bytes(&data[0..4])? as usize;
                let mut offset = 4;
                let mut fields = std::collections::BTreeMap::new();
                
                for _ in 0..field_count {
                    // Decode key length
                    let key_len = u32::from_ssz_bytes(&data[offset..offset + 4])? as usize;
                    offset += 4;
                    
                    // Decode key
                    let key = String::from_utf8(data[offset..offset + key_len].to_vec())
                        .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8 in field name".into()))?;
                    offset += key_len;
                    
                    // Decode value
                    let (value, remaining_after_value) = Value::decode_with_remainder(&data[offset..])?;
                    offset = data.len() - remaining_after_value.len();
                    
                    fields.insert(key, value);
                }
                Ok((Value::Record { fields }, &data[offset..]))
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid Value variant: {}", variant).into()
            )),
        }
    }
}

impl Value {
    /// Get the type of this value
    pub fn value_type(&self) -> TypeInner {
        match self {
            Value::Unit => TypeInner::Base(BaseType::Unit),
            Value::Bool(_) => TypeInner::Base(BaseType::Bool),
            Value::Int(_) => TypeInner::Base(BaseType::Int),
            Value::Symbol(_) => TypeInner::Base(BaseType::Symbol),
            Value::String(_) => TypeInner::Base(BaseType::Symbol),
            Value::Product(left, right) => {
                TypeInner::Product(
                    Box::new(left.value_type()),
                    Box::new(right.value_type()),
                )
            }
            Value::Sum { value, .. } => {
                // For sum types, we'd need additional type information
                // This is a simplified version
                value.value_type()
            }
            Value::Record { fields } => {
                // Build a row type from the field types
                let field_types = fields.iter()
                    .map(|(name, value)| (name.clone(), value.value_type()))
                    .collect();
                
                let row = RowType::with_fields(field_types);
                TypeInner::Record(RecordType::from_row(row))
            }
        }
    }
    
    /// Create a product value
    pub fn product(left: Value, right: Value) -> Self {
        Value::Product(Box::new(left), Box::new(right))
    }
    
    /// Create a sum value with tag
    pub fn sum(tag: u8, value: Value) -> Self {
        Value::Sum {
            tag,
            value: Box::new(value),
        }
    }
}

//-----------------------------------------------------------------------------
// Type Registry
//-----------------------------------------------------------------------------

/// Registry for storing and retrieving type definitions by their content ID
#[derive(Debug, Clone)]
pub struct TypeRegistry {
    types: std::collections::BTreeMap<EntityId, TypeInner>,
}

impl TypeRegistry {
    /// Create a new empty type registry
    pub fn new() -> Self {
        Self {
            types: std::collections::BTreeMap::new(),
        }
    }
    
    /// Register a type and return its ID
    pub fn register_type(&mut self, type_inner: TypeInner) -> EntityId {
        let id = type_inner.content_id();
        self.types.insert(id, type_inner);
        id
    }
    
    /// Get a type by its ID
    pub fn get_type(&self, id: &EntityId) -> Option<&TypeInner> {
        self.types.get(id)
    }
    
    /// Check if a type exists
    pub fn contains_type(&self, id: &EntityId) -> bool {
        self.types.contains_key(id)
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_content_addressing() {
        let int_type1 = Type::int();
        let int_type2 = Type::int();
        let bool_type = Type::bool();
        
        // Same types should have same content ID
        assert_eq!(int_type1.content_id(), int_type2.content_id());
        
        // Different types should have different content IDs
        assert_ne!(int_type1.content_id(), bool_type.content_id());
    }
    
    #[test]
    fn test_value_types() {
        let int_val = Value::Int(42);
        let bool_val = Value::Bool(true);
        let product_val = Value::product(int_val.clone(), bool_val.clone());
        
        // Check value types
        assert_eq!(int_val.value_type(), TypeInner::Base(BaseType::Int));
        assert_eq!(bool_val.value_type(), TypeInner::Base(BaseType::Bool));
        
        // Product type should match
        if let TypeInner::Product(left, right) = product_val.value_type() {
            assert_eq!(*left, TypeInner::Base(BaseType::Int));
            assert_eq!(*right, TypeInner::Base(BaseType::Bool));
        } else {
            panic!("Expected product type");
        }
    }
    
    #[test]
    fn test_ssz_serialization() {
        let type_inner = TypeInner::Base(BaseType::Int);
        let value = Value::Int(42);
        
        // Test type serialization
        let type_encoded = type_inner.as_ssz_bytes();
        let type_decoded = TypeInner::from_ssz_bytes(&type_encoded).unwrap();
        assert_eq!(type_inner, type_decoded);
        
        // Test value serialization
        let value_encoded = value.as_ssz_bytes();
        let value_decoded = Value::from_ssz_bytes(&value_encoded).unwrap();
        assert_eq!(value, value_decoded);
    }
    
    #[test]
    fn test_type_registry() {
        let mut registry = TypeRegistry::new();
        
        let int_type = TypeInner::Base(BaseType::Int);
        let product_type = TypeInner::Product(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::Base(BaseType::Bool)),
        );
        
        let int_id = registry.register_type(int_type.clone());
        let product_id = registry.register_type(product_type.clone());
        
        assert_eq!(registry.get_type(&int_id), Some(&int_type));
        assert_eq!(registry.get_type(&product_id), Some(&product_type));
        assert!(registry.contains_type(&int_id));
        assert!(registry.contains_type(&product_id));
    }
}

// Manual SSZ implementation for Type that only serializes the inner field
impl<L> Encode for Type<L> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        self.inner.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.inner.ssz_append(buf);
    }
}

impl<L> Decode for Type<L> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let inner = TypeInner::from_ssz_bytes(bytes)?;
        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }
}

