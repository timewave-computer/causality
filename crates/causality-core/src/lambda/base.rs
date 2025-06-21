//! Base types for Layer 1: Linear Lambda Calculus
//!
//! These are the fundamental types that form the basis of the type system
//! for the linear lambda calculus layer. All higher-level types are built 
//! from these primitives.

use crate::{
    system::{
        content_addressing::{EntityId, ContentAddressable}, 
        DecodeWithRemainder,
        encode_enum_variant, decode_enum_variant,
    },
    effect::row::{RecordType, RowType, FieldType},
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    
    /// Session type - communication protocols
    Session(Box<SessionType>),
    
    /// Transform type - unifies functions and protocols
    Transform {
        input: Box<TypeInner>,
        output: Box<TypeInner>,
        location: Location,
    },
    
    /// Located type - type with location information
    Located(Box<TypeInner>, Location),
}

/// Session types describe communication protocols
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SessionType {
    /// Send a value of type T, then continue as S
    Send(Box<TypeInner>, Box<SessionType>),
    
    /// Receive a value of type T, then continue as S
    Receive(Box<TypeInner>, Box<SessionType>),
    
    /// Internal choice - we choose which branch to take
    InternalChoice(Vec<(String, SessionType)>),
    
    /// External choice - other party chooses which branch
    ExternalChoice(Vec<(String, SessionType)>),
    
    /// End of communication
    End,
    
    /// Recursive session type
    Recursive(String, Box<SessionType>),
    
    /// Session variable (for recursion)
    Variable(String),
}

// Re-export the comprehensive Location type from the location module
pub use crate::lambda::location::Location;

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
                format!("Invalid Value variant: {}", variant)
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
                let s = crate::system::Str::from_ssz_bytes(data)?;
                Ok((Value::Symbol(s), data))
            }
            4 => {
                let s = crate::system::Str::from_ssz_bytes(data)?;
                Ok((Value::String(s), data))
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
                format!("Invalid Value variant: {}", variant)
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
                    .map(|(name, value)| (name.clone(), FieldType::simple(value.value_type())))
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
// Session Type Environment
//-----------------------------------------------------------------------------

/// Environment for tracking session types of channels during type checking
#[derive(Debug, Clone)]
pub struct SessionEnvironment {
    /// Maps channel names to their session types
    channels: std::collections::BTreeMap<String, SessionType>,
    
    /// Stack of scopes for lexical scoping
    scopes: Vec<std::collections::BTreeMap<String, SessionType>>,
    
    /// Current scope depth
    depth: usize,
}

impl SessionEnvironment {
    /// Create a new empty session environment
    pub fn new() -> Self {
        Self {
            channels: std::collections::BTreeMap::new(),
            scopes: Vec::new(),
            depth: 0,
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(std::collections::BTreeMap::new());
        self.depth += 1;
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Result<(), SessionEnvironmentError> {
        if self.scopes.is_empty() {
            return Err(SessionEnvironmentError::NoScopeToExit);
        }
        
        // Remove bindings from the exited scope
        let exited_scope = self.scopes.pop().unwrap();
        for channel_name in exited_scope.keys() {
            self.channels.remove(channel_name);
        }
        
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }
    
    /// Bind a channel to a session type in the current scope
    pub fn bind_channel(&mut self, channel_name: String, session_type: SessionType) -> Result<(), SessionEnvironmentError> {
        // Check if channel is already bound in current scope
        if let Some(current_scope) = self.scopes.last() {
            if current_scope.contains_key(&channel_name) {
                return Err(SessionEnvironmentError::ChannelAlreadyBound(channel_name));
            }
        } else {
            // In global scope, check if channel already exists
            if self.channels.contains_key(&channel_name) {
                return Err(SessionEnvironmentError::ChannelAlreadyBound(channel_name));
            }
        }
        
        // Add to current scope if we're in one, otherwise to global scope
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(channel_name.clone(), session_type.clone());
        }
        
        // Add to global channel map
        self.channels.insert(channel_name, session_type);
        Ok(())
    }
    
    /// Look up the session type of a channel
    pub fn lookup_channel(&self, channel_name: &str) -> Option<&SessionType> {
        self.channels.get(channel_name)
    }
    
    /// Update the session type of a channel (for protocol progression)
    pub fn update_channel(&mut self, channel_name: &str, new_session_type: SessionType) -> Result<(), SessionEnvironmentError> {
        if !self.channels.contains_key(channel_name) {
            return Err(SessionEnvironmentError::ChannelNotFound(channel_name.to_string()));
        }
        
        self.channels.insert(channel_name.to_string(), new_session_type);
        Ok(())
    }
    
    /// Remove a channel from the environment (when it's consumed/closed)
    pub fn consume_channel(&mut self, channel_name: &str) -> Result<SessionType, SessionEnvironmentError> {
        match self.channels.remove(channel_name) {
            Some(session_type) => {
                // Also remove from current scope
                if let Some(current_scope) = self.scopes.last_mut() {
                    current_scope.remove(channel_name);
                }
                Ok(session_type)
            }
            None => Err(SessionEnvironmentError::ChannelNotFound(channel_name.to_string())),
        }
    }
    
    /// Check if a channel is bound in the environment
    pub fn contains_channel(&self, channel_name: &str) -> bool {
        self.channels.contains_key(channel_name)
    }
    
    /// Get all channels in the current environment
    pub fn all_channels(&self) -> Vec<(String, SessionType)> {
        self.channels.iter()
            .map(|(name, session_type)| (name.clone(), session_type.clone()))
            .collect()
    }
    
    /// Check if the environment is consistent (all channels have well-formed types)
    pub fn is_consistent(&self) -> bool {
        self.channels.values().all(|session_type| session_type.is_well_formed())
    }
    
    /// Get channels that are not at End state (still have obligations)
    pub fn get_active_channels(&self) -> Vec<(String, SessionType)> {
        self.channels.iter()
            .filter(|(_, session_type)| !matches!(session_type, SessionType::End))
            .map(|(name, session_type)| (name.clone(), session_type.clone()))
            .collect()
    }
    
    /// Check if all channels have reached End state
    pub fn all_channels_closed(&self) -> bool {
        self.channels.values().all(|session_type| matches!(session_type, SessionType::End))
    }
    
    /// Merge another environment into this one (for parallel composition)
    pub fn merge(&mut self, other: &SessionEnvironment) -> Result<(), SessionEnvironmentError> {
        for (channel_name, session_type) in &other.channels {
            if self.channels.contains_key(channel_name) {
                return Err(SessionEnvironmentError::ChannelConflict(channel_name.clone()));
            }
            self.channels.insert(channel_name.clone(), session_type.clone());
        }
        Ok(())
    }
    
    /// Split the environment for parallel composition
    /// Returns two environments that partition the channels
    pub fn split(&self, left_channels: &[String]) -> Result<(SessionEnvironment, SessionEnvironment), SessionEnvironmentError> {
        let mut left_env = SessionEnvironment::new();
        let mut right_env = SessionEnvironment::new();
        
        for (channel_name, session_type) in &self.channels {
            if left_channels.contains(channel_name) {
                left_env.channels.insert(channel_name.clone(), session_type.clone());
            } else {
                right_env.channels.insert(channel_name.clone(), session_type.clone());
            }
        }
        
        Ok((left_env, right_env))
    }
}

impl Default for SessionEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during session environment operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEnvironmentError {
    /// Attempted to bind a channel that's already bound in the current scope
    ChannelAlreadyBound(String),
    
    /// Attempted to look up or modify a channel that doesn't exist
    ChannelNotFound(String),
    
    /// Attempted to exit a scope when no scope is active
    NoScopeToExit,
    
    /// Channel conflict during environment merge
    ChannelConflict(String),
    
    /// Type mismatch during channel operations
    TypeMismatch {
        channel: String,
        expected: SessionType,
        found: SessionType,
    },
}

impl std::fmt::Display for SessionEnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionEnvironmentError::ChannelAlreadyBound(name) => {
                write!(f, "Channel '{}' is already bound in the current scope", name)
            }
            SessionEnvironmentError::ChannelNotFound(name) => {
                write!(f, "Channel '{}' not found in environment", name)
            }
            SessionEnvironmentError::NoScopeToExit => {
                write!(f, "No scope to exit")
            }
            SessionEnvironmentError::ChannelConflict(name) => {
                write!(f, "Channel '{}' conflicts during environment merge", name)
            }
            SessionEnvironmentError::TypeMismatch { channel, expected, found } => {
                write!(f, "Type mismatch for channel '{}': expected {:?}, found {:?}", 
                       channel, expected, found)
            }
        }
    }
}

impl std::error::Error for SessionEnvironmentError {}

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
    
    #[test]
    fn test_session_type_duality() {
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let receive_int = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Test duality
        assert_eq!(send_int.dual(), receive_int);
        assert_eq!(receive_int.dual(), send_int);
        assert!(send_int.is_dual_to(&receive_int));
        assert!(receive_int.is_dual_to(&send_int));
        
        // Test choice duality
        let internal_choice = SessionType::InternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        
        let external_choice = SessionType::ExternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        
        assert_eq!(internal_choice.dual(), external_choice);
        assert!(internal_choice.is_dual_to(&external_choice));
    }
    
    #[test]
    fn test_session_type_substitution() {
        // Test basic variable substitution
        let var_x = SessionType::Variable("X".to_string());
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let result = var_x.substitute("X", &send_int);
        assert_eq!(result, send_int);
        
        // Test substitution in compound types
        let send_var = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Variable("X".to_string()))
        );
        
        let result = send_var.substitute("X", &SessionType::End);
        let expected = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        assert_eq!(result, expected);
        
        // Test bound variable protection
        let recursive = SessionType::Recursive(
            "X".to_string(),
            Box::new(SessionType::Variable("X".to_string()))
        );
        
        let result = recursive.substitute("X", &send_int);
        assert_eq!(result, recursive); // Should be unchanged
    }
    
    #[test]
    fn test_session_type_unfolding() {
        // Test unfolding recursive types
        let recursive = SessionType::Recursive(
            "X".to_string(),
            Box::new(SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(SessionType::Variable("X".to_string()))
            ))
        );
        
        let unfolded = recursive.unfold();
        let expected = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(recursive.clone())
        );
        
        assert_eq!(unfolded, expected);
    }
    
    #[test]
    fn test_session_type_free_variables() {
        // Test free variable detection
        let var_x = SessionType::Variable("X".to_string());
        assert!(var_x.has_free_variable("X"));
        assert!(!var_x.has_free_variable("Y"));
        
        // Test bound variables
        let recursive = SessionType::Recursive(
            "X".to_string(),
            Box::new(SessionType::Variable("X".to_string()))
        );
        assert!(!recursive.has_free_variable("X"));
        
        // Test free variable in compound type
        let send_var = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Variable("Y".to_string()))
        );
        assert!(send_var.has_free_variable("Y"));
        assert!(!send_var.has_free_variable("X"));
    }
    
    #[test]
    fn test_session_type_well_formed() {
        // Well-formed types
        assert!(SessionType::End.is_well_formed());
        
        let send_end = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        assert!(send_end.is_well_formed());
        
        let recursive = SessionType::Recursive(
            "X".to_string(),
            Box::new(SessionType::Variable("X".to_string()))
        );
        assert!(recursive.is_well_formed());
        
        // Ill-formed type (free variable)
        let free_var = SessionType::Variable("X".to_string());
        assert!(!free_var.is_well_formed());
        
        let send_free = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Variable("Y".to_string()))
        );
        assert!(!send_free.is_well_formed());
    }
    
    #[test]
    fn test_session_type_subtyping_basic() {
        // Reflexivity
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        assert!(send_int.is_subtype_of(&send_int));
        assert!(SessionType::End.is_subtype_of(&SessionType::End));
        
        // Send subtyping (covariant)
        let send_int_end = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let send_int_send = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Bool)),
                Box::new(SessionType::End)
            ))
        );
        
        // send_int_send is NOT a subtype of send_int_end (more obligations)
        assert!(!send_int_send.is_subtype_of(&send_int_end));
        
        // Receive subtyping (contravariant in message type)
        let recv_int = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let recv_bool = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        
        // Different types, so no subtyping (simplified type system)
        assert!(!recv_int.is_subtype_of(&recv_bool));
        assert!(!recv_bool.is_subtype_of(&recv_int));
    }
    
    #[test]
    fn test_session_type_choice_subtyping() {
        // Internal choice subtyping: more choices is a subtype (covariant)
        let internal_two = SessionType::InternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        
        let internal_one = SessionType::InternalChoice(vec![
            ("left".to_string(), SessionType::End),
        ]);
        
        // internal_two is a subtype of internal_one (can provide more choices)
        assert!(internal_two.is_subtype_of(&internal_one));
        assert!(!internal_one.is_subtype_of(&internal_two));
        
        // External choice subtyping: fewer choices is a subtype (contravariant)
        let external_one = SessionType::ExternalChoice(vec![
            ("left".to_string(), SessionType::End),
        ]);
        
        let external_two = SessionType::ExternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        
        // external_one is a subtype of external_two (can handle fewer choices)
        assert!(external_one.is_subtype_of(&external_two));
        assert!(!external_two.is_subtype_of(&external_one));
    }
    
    #[test]
    fn test_session_type_recursive_subtyping() {
        // Simple recursive types
        let rec1 = SessionType::Recursive(
            "X".to_string(),
            Box::new(SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(SessionType::Variable("X".to_string()))
            ))
        );
        
        let rec2 = SessionType::Recursive(
            "Y".to_string(),
            Box::new(SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(SessionType::Variable("Y".to_string()))
            ))
        );
        
        // Same structure, different variable names - should be equivalent
        assert!(rec1.is_subtype_of(&rec2));
        assert!(rec2.is_subtype_of(&rec1));
        assert!(rec1.is_equivalent_to(&rec2));
    }
    
    #[test]
    fn test_session_type_equivalence() {
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let send_int2 = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        assert!(send_int.is_equivalent_to(&send_int2));
        
        let recv_int = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        assert!(!send_int.is_equivalent_to(&recv_int));
    }
    
    #[test]
    fn test_session_type_capabilities() {
        // Test capability extraction
        let send_recv = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Bool)),
                Box::new(SessionType::End)
            ))
        );
        
        let caps = send_recv.required_capabilities();
        assert!(caps.contains("send"));
        assert!(caps.contains("receive"));
        assert!(!caps.contains("select"));
        
        // Test choice capabilities
        let internal_choice = SessionType::InternalChoice(vec![
            ("opt1".to_string(), SessionType::End),
            ("opt2".to_string(), SessionType::End),
        ]);
        
        let choice_caps = internal_choice.required_capabilities();
        assert!(choice_caps.contains("select"));
        
        let external_choice = SessionType::ExternalChoice(vec![
            ("opt1".to_string(), SessionType::End),
        ]);
        
        let branch_caps = external_choice.required_capabilities();
        assert!(branch_caps.contains("branch"));
        
        // Test recursive capabilities
        let recursive = SessionType::Recursive(
            "X".to_string(),
            Box::new(SessionType::Variable("X".to_string()))
        );
        
        let rec_caps = recursive.required_capabilities();
        assert!(rec_caps.contains("recursion"));
    }
    
    #[test]
    fn test_session_environment_basic() {
        let mut env = SessionEnvironment::new();
        
        // Test basic channel binding
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        assert!(env.bind_channel("ch1".to_string(), send_int.clone()).is_ok());
        assert!(env.contains_channel("ch1"));
        assert_eq!(env.lookup_channel("ch1"), Some(&send_int));
        
        // Test channel not found
        assert!(!env.contains_channel("ch2"));
        assert_eq!(env.lookup_channel("ch2"), None);
        
        // Test double binding in same scope should fail
        let recv_bool = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        assert!(env.bind_channel("ch1".to_string(), recv_bool).is_err());
    }
    
    #[test]
    fn test_session_environment_scoping() {
        let mut env = SessionEnvironment::new();
        
        // Bind in global scope
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        assert!(env.bind_channel("global_ch".to_string(), send_int.clone()).is_ok());
        
        // Enter new scope
        env.enter_scope();
        
        // Can still see global channel
        assert!(env.contains_channel("global_ch"));
        
        // Bind in local scope
        let recv_bool = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        assert!(env.bind_channel("local_ch".to_string(), recv_bool.clone()).is_ok());
        assert!(env.contains_channel("local_ch"));
        
        // Exit scope
        assert!(env.exit_scope().is_ok());
        
        // Local channel should be gone
        assert!(!env.contains_channel("local_ch"));
        // Global channel should still be there
        assert!(env.contains_channel("global_ch"));
        
        // Can't exit scope when none exists
        assert!(env.exit_scope().is_err());
    }
    
    #[test]
    fn test_session_environment_updates() {
        let mut env = SessionEnvironment::new();
        
        // Bind a channel
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        assert!(env.bind_channel("ch".to_string(), send_int).is_ok());
        
        // Update the channel type (protocol progression)
        let new_type = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        assert!(env.update_channel("ch", new_type.clone()).is_ok());
        assert_eq!(env.lookup_channel("ch"), Some(&new_type));
        
        // Update non-existent channel should fail
        assert!(env.update_channel("nonexistent", SessionType::End).is_err());
        
        // Consume the channel
        let consumed = env.consume_channel("ch");
        assert!(consumed.is_ok());
        assert_eq!(consumed.unwrap(), new_type);
        assert!(!env.contains_channel("ch"));
        
        // Consume non-existent channel should fail
        assert!(env.consume_channel("ch").is_err());
    }
    
    #[test]
    fn test_session_environment_active_channels() {
        let mut env = SessionEnvironment::new();
        
        // Bind some channels
        assert!(env.bind_channel("active1".to_string(), SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        )).is_ok());
        
        assert!(env.bind_channel("active2".to_string(), SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        )).is_ok());
        
        assert!(env.bind_channel("closed".to_string(), SessionType::End).is_ok());
        
        // Check active channels
        let active = env.get_active_channels();
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|(name, _)| name == "active1"));
        assert!(active.iter().any(|(name, _)| name == "active2"));
        assert!(!active.iter().any(|(name, _)| name == "closed"));
        
        // Not all channels are closed
        assert!(!env.all_channels_closed());
        
        // Close active channels
        assert!(env.update_channel("active1", SessionType::End).is_ok());
        assert!(env.update_channel("active2", SessionType::End).is_ok());
        
        // Now all channels should be closed
        assert!(env.all_channels_closed());
        assert_eq!(env.get_active_channels().len(), 0);
    }
    
    #[test]
    fn test_session_environment_merge_split() {
        let mut env1 = SessionEnvironment::new();
        let mut env2 = SessionEnvironment::new();
        
        // Set up different channels in each environment
        let send_int = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        let recv_bool = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Bool)),
            Box::new(SessionType::End)
        );
        
        assert!(env1.bind_channel("ch1".to_string(), send_int.clone()).is_ok());
        assert!(env2.bind_channel("ch2".to_string(), recv_bool.clone()).is_ok());
        
        // Merge should work
        assert!(env1.merge(&env2).is_ok());
        assert!(env1.contains_channel("ch1"));
        assert!(env1.contains_channel("ch2"));
        
        // Merge with conflict should fail
        let mut env3 = SessionEnvironment::new();
        assert!(env3.bind_channel("ch1".to_string(), recv_bool.clone()).is_ok());
        assert!(env1.merge(&env3).is_err());
        
        // Test splitting
        let (left, right) = env1.split(&["ch1".to_string()]).unwrap();
        assert!(left.contains_channel("ch1"));
        assert!(!left.contains_channel("ch2"));
        assert!(!right.contains_channel("ch1"));
        assert!(right.contains_channel("ch2"));
    }
    
    #[test]
    fn test_session_environment_consistency() {
        let mut env = SessionEnvironment::new();
        
        // Well-formed session types
        let well_formed = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        assert!(env.bind_channel("good".to_string(), well_formed).is_ok());
        assert!(env.is_consistent());
        
        // Ill-formed session type (free variable)
        let ill_formed = SessionType::Variable("X".to_string());
        assert!(env.bind_channel("bad".to_string(), ill_formed).is_ok());
        assert!(!env.is_consistent());
    }
}

impl SessionType {
    /// Compute the dual of a session type
    pub fn dual(&self) -> SessionType {
        match self {
            SessionType::Send(t, s) => {
                SessionType::Receive(t.clone(), Box::new(s.dual()))
            }
            SessionType::Receive(t, s) => {
                SessionType::Send(t.clone(), Box::new(s.dual()))
            }
            SessionType::InternalChoice(branches) => {
                SessionType::ExternalChoice(
                    branches.iter()
                        .map(|(label, session)| (label.clone(), session.dual()))
                        .collect()
                )
            }
            SessionType::ExternalChoice(branches) => {
                SessionType::InternalChoice(
                    branches.iter()
                        .map(|(label, session)| (label.clone(), session.dual()))
                        .collect()
                )
            }
            SessionType::End => SessionType::End,
            SessionType::Recursive(var, body) => {
                SessionType::Recursive(var.clone(), Box::new(body.dual()))
            }
            SessionType::Variable(var) => SessionType::Variable(var.clone()),
        }
    }
    
    /// Check if this session type is dual to another
    pub fn is_dual_to(&self, other: &SessionType) -> bool {
        self == &other.dual()
    }
    
    /// Substitute a variable with a session type (for unfolding recursive types)
    pub fn substitute(&self, var: &str, replacement: &SessionType) -> SessionType {
        match self {
            SessionType::Send(t, s) => {
                SessionType::Send(t.clone(), Box::new(s.substitute(var, replacement)))
            }
            SessionType::Receive(t, s) => {
                SessionType::Receive(t.clone(), Box::new(s.substitute(var, replacement)))
            }
            SessionType::InternalChoice(branches) => {
                SessionType::InternalChoice(
                    branches.iter()
                        .map(|(label, session)| (label.clone(), session.substitute(var, replacement)))
                        .collect()
                )
            }
            SessionType::ExternalChoice(branches) => {
                SessionType::ExternalChoice(
                    branches.iter()
                        .map(|(label, session)| (label.clone(), session.substitute(var, replacement)))
                        .collect()
                )
            }
            SessionType::End => SessionType::End,
            SessionType::Recursive(bound_var, body) => {
                if bound_var == var {
                    // Variable is bound here, don't substitute inside
                    SessionType::Recursive(bound_var.clone(), body.clone())
                } else {
                    // Variable is free, substitute inside the body
                    SessionType::Recursive(bound_var.clone(), Box::new(body.substitute(var, replacement)))
                }
            }
            SessionType::Variable(var_name) => {
                if var_name == var {
                    replacement.clone()
                } else {
                    SessionType::Variable(var_name.clone())
                }
            }
        }
    }
    
    /// Unfold a recursive session type one level
    pub fn unfold(&self) -> SessionType {
        match self {
            SessionType::Recursive(var, body) => {
                // Substitute the recursive variable with the entire recursive type
                body.substitute(var, self)
            }
            _ => self.clone(),
        }
    }
    
    /// Check if a session type contains a free variable
    pub fn has_free_variable(&self, var: &str) -> bool {
        self.has_free_variable_with_bound(var, &std::collections::BTreeSet::new())
    }
    
    /// Helper method to check for free variables with a set of bound variables
    fn has_free_variable_with_bound(&self, var: &str, bound_vars: &std::collections::BTreeSet<String>) -> bool {
        match self {
            SessionType::Send(_, s) | SessionType::Receive(_, s) => {
                s.has_free_variable_with_bound(var, bound_vars)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                branches.iter().any(|(_, session)| session.has_free_variable_with_bound(var, bound_vars))
            }
            SessionType::End => false,
            SessionType::Recursive(bound_var, body) => {
                let mut new_bound = bound_vars.clone();
                new_bound.insert(bound_var.clone());
                body.has_free_variable_with_bound(var, &new_bound)
            }
            SessionType::Variable(var_name) => {
                var_name == var && !bound_vars.contains(var)
            }
        }
    }
    
    /// Check if this session type is well-formed (no unbound variables)
    pub fn is_well_formed(&self) -> bool {
        self.is_well_formed_with_bound(&std::collections::BTreeSet::new())
    }
    
    /// Helper method to check well-formedness with bound variables
    fn is_well_formed_with_bound(&self, bound_vars: &std::collections::BTreeSet<String>) -> bool {
        match self {
            SessionType::Send(_, s) | SessionType::Receive(_, s) => {
                s.is_well_formed_with_bound(bound_vars)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                branches.iter().all(|(_, session)| session.is_well_formed_with_bound(bound_vars))
            }
            SessionType::End => true,
            SessionType::Recursive(bound_var, body) => {
                let mut new_bound = bound_vars.clone();
                new_bound.insert(bound_var.clone());
                body.is_well_formed_with_bound(&new_bound)
            }
            SessionType::Variable(var_name) => {
                bound_vars.contains(var_name)
            }
        }
    }
    
    /// Check if this session type is a subtype of another
    /// Implements session type subtyping rules based on:
    /// - Contravariance for inputs (receive types)
    /// - Covariance for outputs (send types) 
    /// - Choice subtyping (more choices in external, fewer in internal)
    pub fn is_subtype_of(&self, other: &SessionType) -> bool {
        self.is_subtype_of_with_context(other, &mut std::collections::BTreeMap::new())
    }
    
    /// Helper method for subtyping with recursive type context
    fn is_subtype_of_with_context(
        &self,
        other: &SessionType,
        context: &mut std::collections::BTreeMap<(String, String), bool>
    ) -> bool {
        match (self, other) {
            // Reflexivity: T <: T
            (s1, s2) if s1 == s2 => true,
            
            // End is subtype of End only
            (SessionType::End, SessionType::End) => true,
            (SessionType::End, _) => false,
            (_, SessionType::End) => false,
            
            // Send subtyping: covariant in message type and continuation
            (SessionType::Send(t1, s1), SessionType::Send(t2, s2)) => {
                self.is_type_subtype(t1, t2) && s1.is_subtype_of_with_context(s2, context)
            }
            
            // Receive subtyping: contravariant in message type, covariant in continuation
            (SessionType::Receive(t1, s1), SessionType::Receive(t2, s2)) => {
                self.is_type_subtype(t2, t1) && s1.is_subtype_of_with_context(s2, context)
            }
            
            // Internal choice subtyping: more choices is a subtype (covariant)
            (SessionType::InternalChoice(branches1), SessionType::InternalChoice(branches2)) => {
                // Every branch in branches2 must have a corresponding subtype in branches1
                // This means branches1 can have more choices than branches2
                branches2.iter().all(|(label2, session2)| {
                    branches1.iter().any(|(label1, session1)| {
                        label1 == label2 && session1.is_subtype_of_with_context(session2, context)
                    })
                })
            }
            
            // External choice subtyping: fewer choices is a subtype (contravariant)
            (SessionType::ExternalChoice(branches1), SessionType::ExternalChoice(branches2)) => {
                // Every branch in branches1 must have a corresponding subtype in branches2
                // This means branches1 can have fewer choices than branches2
                branches1.iter().all(|(label1, session1)| {
                    branches2.iter().any(|(label2, session2)| {
                        label1 == label2 && session1.is_subtype_of_with_context(session2, context)
                    })
                })
            }
            
            // Recursive type subtyping
            (SessionType::Recursive(var1, body1), SessionType::Recursive(var2, body2)) => {
                let key = (var1.clone(), var2.clone());
                
                // Check if we've already assumed this subtyping relationship
                if let Some(&result) = context.get(&key) {
                    return result;
                }
                
                // Assume the subtyping holds and check the bodies
                context.insert(key.clone(), true);
                
                // Rename variables to match and check body subtyping
                let renamed_body1 = body1.substitute(var1, &SessionType::Variable(var2.clone()));
                let result = renamed_body1.is_subtype_of_with_context(body2, context);
                
                context.insert(key, result);
                result
            }
            
            // Variable subtyping (only if they're the same variable)
            (SessionType::Variable(var1), SessionType::Variable(var2)) => var1 == var2,
            
            // Unfold recursive types for subtyping
            (SessionType::Recursive(_, _), other) => {
                self.unfold().is_subtype_of_with_context(other, context)
            }
            (other, SessionType::Recursive(_, _)) => {
                other.is_subtype_of_with_context(&other.unfold(), context)
            }
            
            // No other subtyping relationships
            _ => false,
        }
    }
    
    /// Helper to check if one type is a subtype of another
    /// This is a simplified version - in practice would need full type system
    fn is_type_subtype(&self, t1: &TypeInner, t2: &TypeInner) -> bool {
        // For now, just check equality
        // In a full implementation, this would check structural subtyping
        t1 == t2
    }
    
    /// Check if two session types are equivalent (mutually subtypes)
    pub fn is_equivalent_to(&self, other: &SessionType) -> bool {
        self.is_subtype_of(other) && other.is_subtype_of(self)
    }
    
    /// Get the capability requirements for this session type
    /// Returns the set of capabilities needed to implement this session
    pub fn required_capabilities(&self) -> std::collections::BTreeSet<String> {
        let mut caps = std::collections::BTreeSet::new();
        self.collect_capabilities(&mut caps);
        caps
    }
    
    /// Helper to collect capabilities recursively
    fn collect_capabilities(&self, caps: &mut std::collections::BTreeSet<String>) {
        match self {
            SessionType::Send(_, cont) => {
                caps.insert("send".to_string());
                cont.collect_capabilities(caps);
            }
            SessionType::Receive(_, cont) => {
                caps.insert("receive".to_string());
                cont.collect_capabilities(caps);
            }
            SessionType::InternalChoice(branches) => {
                caps.insert("select".to_string());
                for (_, session) in branches {
                    session.collect_capabilities(caps);
                }
            }
            SessionType::ExternalChoice(branches) => {
                caps.insert("branch".to_string());
                for (_, session) in branches {
                    session.collect_capabilities(caps);
                }
            }
            SessionType::Recursive(_, body) => {
                caps.insert("recursion".to_string());
                body.collect_capabilities(caps);
            }
            SessionType::End | SessionType::Variable(_) => {
                // No additional capabilities needed
            }
        }
    }
}

// SSZ implementation for SessionType
impl Encode for SessionType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        1 + match self {
            SessionType::Send(t, s) | SessionType::Receive(t, s) => {
                t.ssz_bytes_len() + s.ssz_bytes_len()
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                4 + branches.iter().map(|(label, session)| {
                    4 + label.len() + session.ssz_bytes_len()
                }).sum::<usize>()
            }
            SessionType::End => 0,
            SessionType::Recursive(var, body) => {
                4 + var.len() + body.ssz_bytes_len()
            }
            SessionType::Variable(var) => 4 + var.len(),
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        use crate::system::encode_enum_variant;
        
        match self {
            SessionType::Send(t, s) => {
                encode_enum_variant(0, buf);
                t.ssz_append(buf);
                s.ssz_append(buf);
            }
            SessionType::Receive(t, s) => {
                encode_enum_variant(1, buf);
                t.ssz_append(buf);
                s.ssz_append(buf);
            }
            SessionType::InternalChoice(branches) => {
                encode_enum_variant(2, buf);
                (branches.len() as u32).ssz_append(buf);
                for (label, session) in branches {
                    (label.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(label.as_bytes());
                    session.ssz_append(buf);
                }
            }
            SessionType::ExternalChoice(branches) => {
                encode_enum_variant(3, buf);
                (branches.len() as u32).ssz_append(buf);
                for (label, session) in branches {
                    (label.len() as u32).ssz_append(buf);
                    buf.extend_from_slice(label.as_bytes());
                    session.ssz_append(buf);
                }
            }
            SessionType::End => encode_enum_variant(4, buf),
            SessionType::Recursive(var, body) => {
                encode_enum_variant(5, buf);
                (var.len() as u32).ssz_append(buf);
                buf.extend_from_slice(var.as_bytes());
                body.ssz_append(buf);
            }
            SessionType::Variable(var) => {
                encode_enum_variant(6, buf);
                (var.len() as u32).ssz_append(buf);
                buf.extend_from_slice(var.as_bytes());
            }
        }
    }
}

impl Decode for SessionType {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        use crate::system::decode_enum_variant;
        
        let (variant, data) = decode_enum_variant(bytes)?;
        
        match variant {
            0 => {
                let (t, remaining) = TypeInner::decode_with_remainder(data)?;
                let s = SessionType::from_ssz_bytes(remaining)?;
                Ok(SessionType::Send(Box::new(t), Box::new(s)))
            }
            1 => {
                let (t, remaining) = TypeInner::decode_with_remainder(data)?;
                let s = SessionType::from_ssz_bytes(remaining)?;
                Ok(SessionType::Receive(Box::new(t), Box::new(s)))
            }
            2 | 3 => {
                let branch_count = u32::from_ssz_bytes(&data[..4])? as usize;
                let mut offset = 4;
                let mut branches = Vec::new();
                
                for _ in 0..branch_count {
                    let label_len = u32::from_ssz_bytes(&data[offset..offset+4])? as usize;
                    offset += 4;
                    
                    let label = String::from_utf8(data[offset..offset+label_len].to_vec())
                        .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8".into()))?;
                    offset += label_len;
                    
                    let session = SessionType::from_ssz_bytes(&data[offset..])?;
                    offset += session.ssz_bytes_len();
                    
                    branches.push((label, session));
                }
                
                match variant {
                    2 => Ok(SessionType::InternalChoice(branches)),
                    3 => Ok(SessionType::ExternalChoice(branches)),
                    _ => unreachable!(),
                }
            }
            4 => Ok(SessionType::End),
            5 => {
                let var_len = u32::from_ssz_bytes(&data[..4])? as usize;
                let var = String::from_utf8(data[4..4+var_len].to_vec())
                    .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8".into()))?;
                let body = SessionType::from_ssz_bytes(&data[4+var_len..])?;
                Ok(SessionType::Recursive(var, Box::new(body)))
            }
            6 => {
                let var_len = u32::from_ssz_bytes(&data[..4])? as usize;
                let var = String::from_utf8(data[4..4+var_len].to_vec())
                    .map_err(|_| DecodeError::BytesInvalid("Invalid UTF-8".into()))?;
                Ok(SessionType::Variable(var))
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid SessionType variant: {}", variant)
            )),
        }
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
            TypeInner::Session(_) => 0,
            TypeInner::Transform { input, output, location } => {
                input.ssz_bytes_len() + output.ssz_bytes_len() + location.ssz_bytes_len()
            }
            TypeInner::Located(inner, location) => inner.ssz_bytes_len() + location.ssz_bytes_len(),
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
            TypeInner::Session(_) => {
                encode_enum_variant(5, buf);
            }
            TypeInner::Transform { input, output, location } => {
                encode_enum_variant(6, buf);
                input.ssz_append(buf);
                output.ssz_append(buf);
                location.ssz_append(buf);
            }
            TypeInner::Located(inner, location) => {
                encode_enum_variant(7, buf);
                inner.ssz_append(buf);
                location.ssz_append(buf);
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
            5 => {
                Ok(TypeInner::Session(Box::new(SessionType::End)))
            }
            6 => {
                let (input, remaining) = TypeInner::decode_with_remainder(data)?;
                let (output, remaining) = TypeInner::decode_with_remainder(remaining)?;
                let (location, _) = Location::decode_with_remainder(remaining)?;
                Ok(TypeInner::Transform {
                    input: Box::new(input),
                    output: Box::new(output),
                    location,
                })
            }
            7 => {
                let (inner, remaining) = TypeInner::decode_with_remainder(data)?;
                let (location, _) = Location::decode_with_remainder(remaining)?;
                Ok(TypeInner::Located(Box::new(inner), location))
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid TypeInner variant: {}", variant)
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
            1..=3 => {
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
            5 => {
                Ok((TypeInner::Session(Box::new(SessionType::End)), data))
            }
            6 => {
                let (input, remaining) = Self::decode_with_remainder(data)?;
                let (output, remaining) = Self::decode_with_remainder(remaining)?;
                let (location, _) = Location::decode_with_remainder(remaining)?;
                Ok((TypeInner::Transform {
                    input: Box::new(input),
                    output: Box::new(output),
                    location,
                }, remaining))
            }
            7 => {
                let (inner, remaining) = Self::decode_with_remainder(data)?;
                let (location, _) = Location::decode_with_remainder(remaining)?;
                Ok((TypeInner::Located(Box::new(inner), location), remaining))
            }
            _ => Err(DecodeError::BytesInvalid(
                format!("Invalid TypeInner variant: {}", variant)
            )),
        }
    }
}

