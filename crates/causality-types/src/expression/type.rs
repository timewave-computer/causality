//! Type Expression System for the Causality framework.
//!
//! This module provides the `TypeExpr` enum, which is used to define the
//! structure of data values within the system. It supports a comprehensive set
//! of type constructors for building schemas suitable for static analysis,
//! runtime validation (against `ValueExpr`), and ensuring ZK compatibility.
//! Following refactoring, `TypeExpr` itself is static and does not contain
//! dynamic references or type variables.
//!
//! ## Implicit Schema Generation
//!
//! Each `TypeExpr` is canonically serialized to produce a `TypeExprId` through content
//! addressing, eliminating the need for explicit schema definitions. Benefits include:
//!
//! - **Automatic Deduplication**: Identical type structures produce the same TypeExprId
//! - **Implicit Reuse**: Resources or effects with identical structures share the same schema ID
//! - **Simplified Equality**: Type equality is verified via 32-byte hash comparison
//! - **No Registration Required**: Schemas are derived directly from type structures
//! - **Cross-Domain Compatibility**: Remote domains can validate schemas by hash
//!
//! This approach aligns with the system's goals of content-addressed state, verifiable
//! determinism, and developer ergonomics.

use crate::primitive::ids::TypeExprId;
use crate::primitive::string::Str;
use crate::serialization::{Decode, DecodeError, Encode, SimpleSerialize};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Configuration
//-----------------------------------------------------------------------------

/// Toggle for full type display (useful for debugging)
#[allow(dead_code)]
static FULL_TYPE_DISPLAY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

//-----------------------------------------------------------------------------
// Wrapper Types for Breaking Recursion
//-----------------------------------------------------------------------------

// These wrapper types break recursive type definitions for SSZ serialization.
// They enable the compiler to determine concrete sizes for recursive types and
// prevent stack overflows during serialization.

/// Box wrapper for TypeExpr to allow trait implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExprBox(pub Box<TypeExpr>);

impl Encode for TypeExprBox {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}
impl Decode for TypeExprBox {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(TypeExprBox(Box::new(TypeExpr::from_ssz_bytes(bytes)?)))
    }
}
impl SimpleSerialize for TypeExprBox {}

/// Vector wrapper for `TypeExpr` collections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExprVec(pub Vec<TypeExpr>);

impl Encode for TypeExprVec {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}
impl Decode for TypeExprVec {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(TypeExprVec(Vec::<TypeExpr>::from_ssz_bytes(bytes)?))
    }
}
impl SimpleSerialize for TypeExprVec {}

/// Map wrapper for `TypeExpr` structures with `Str` keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExprMap(pub BTreeMap<Str, TypeExpr>);

impl Encode for TypeExprMap {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}
impl Decode for TypeExprMap {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(TypeExprMap(BTreeMap::<Str, TypeExpr>::from_ssz_bytes(bytes)?))
    }
}
impl SimpleSerialize for TypeExprMap {}

//-----------------------------------------------------------------------------
// Wrapper Implementations
//-----------------------------------------------------------------------------

// Defines common Deref, DerefMut, From traits for the wrapper types.
macro_rules! impl_wrapper {
    ($wrapper:ident, $inner:ty) => {
        impl std::ops::Deref for $wrapper {
            type Target = $inner;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $wrapper {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl From<$wrapper> for $inner {
            #[inline]
            fn from(wrapper: $wrapper) -> Self {
                wrapper.0
            }
        }
        impl From<$inner> for $wrapper {
            #[inline]
            fn from(inner: $inner) -> Self {
                $wrapper(inner)
            }
        }
    };
}

impl_wrapper!(TypeExprBox, Box<TypeExpr>);
impl_wrapper!(TypeExprVec, Vec<TypeExpr>);
impl_wrapper!(TypeExprMap, BTreeMap<Str, TypeExpr>);

// Implement Display for TypeExprBox
impl std::fmt::Display for TypeExprBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::iter::FromIterator<TypeExpr> for TypeExprVec {
    fn from_iter<I: IntoIterator<Item = TypeExpr>>(iter: I) -> Self {
        TypeExprVec(iter.into_iter().collect())
    }
}

//-----------------------------------------------------------------------------
// TypeExpr Enum Definition
//-----------------------------------------------------------------------------

/// Defines the structure of values in the system.
/// This is the canonical, static type representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// Matches any type.
    Any,
    /// Represents a unit type (void/nil).
    Unit,
    /// Represents a boolean type.
    Bool,
    /// Represents a string type (fixed-size via `primitive::string::Str`).
    String,
    /// Represents an integer type (typically `i64`).
    Integer,
    /// Represents a fixed-point decimal type.
    Fixed,
    /// Represents a rational number type.
    Ratio,
    /// Represents a general numeric type (e.g., Integer, Fixed, Ratio).
    Number, // This would wrap crate::primitive::number::Number
    /// Represents a list of elements of a single type.
    List(TypeExprBox),
    /// Represents a map with keys and values of specified types.
    Map(TypeExprBox, TypeExprBox), // Box<KeyType>, Box<ValueType>
    /// Represents a record (struct) with named fields of specified types.
    Record(TypeExprMap),
    /// Represents a union of several possible types (tagged union).
    Union(TypeExprVec),
    /// Represents an intersection of several types.
    Intersection(TypeExprVec),
    /// Represents an optional type (None or Some<T>).
    Optional(TypeExprBox),
    /// Represents a numeric range constraint (min, max).
    Range(i64, i64),
    /// Represents an enumeration of a set of allowed `TypeExpr` values.
    /// Note: Values are `TypeExpr`s themselves, typically simple ones like String/Integer.
    Enum(TypeExprVec),
    /// Represents an ordered sequence of types (tuple).
    Tuple(TypeExprVec),
    /// Represents a function type signature (parameters -> return type).
    Function(TypeExprBox, TypeExprBox), // Box<ParamType>, Box<ReturnType>
}

//-----------------------------------------------------------------------------
// Display Implementation
//-----------------------------------------------------------------------------

/// Provides a human-readable string representation of `TypeExpr` values.
impl std::fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpr::Any => write!(f, "Any"),
            TypeExpr::Unit => write!(f, "Unit"),
            TypeExpr::Bool => write!(f, "Bool"),
            TypeExpr::String => write!(f, "String"),
            TypeExpr::Integer => write!(f, "Integer"),
            TypeExpr::Fixed => write!(f, "Fixed"),
            TypeExpr::Ratio => write!(f, "Ratio"),
            TypeExpr::Number => write!(f, "Number"),
            TypeExpr::List(inner) => write!(f, "List<{}>", inner),
            TypeExpr::Map(key, value) => write!(f, "Map<{}, {}>", key, value),
            TypeExpr::Record(fields) => {
                write!(f, "Record{{")?;
                let mut first = true;
                for (name, ty) in fields.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                    first = false;
                }
                write!(f, "}}")
            }
            TypeExpr::Union(types) => {
                write!(f, "Union<")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ">")
            }
            TypeExpr::Intersection(types) => {
                write!(f, "Intersection<")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " & ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ">")
            }
            TypeExpr::Optional(inner) => write!(f, "Optional<{}>", inner),
            TypeExpr::Range(min, max) => write!(f, "Range<{}, {}>", min, max),
            TypeExpr::Enum(values) => {
                write!(f, "Enum<")?;
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, ">")
            }
            TypeExpr::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            TypeExpr::Function(params, ret) => write!(f, "({}) -> {}", params, ret),
        }
    }
}

//-----------------------------------------------------------------------------
// Traits
//-----------------------------------------------------------------------------

/// Trait for types that can derive a schema ID
pub trait AsSchema {
    /// Derive a schema ID for this type
    fn schema_id(&self) -> TypeExprId;
}

//-----------------------------------------------------------------------------
// Serialization Implementation
//-----------------------------------------------------------------------------

impl Encode for TypeExpr {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            TypeExpr::Any => bytes.push(0u8),
            TypeExpr::Unit => bytes.push(1u8),
            TypeExpr::Bool => bytes.push(2u8),
            TypeExpr::String => bytes.push(3u8),
            TypeExpr::Integer => bytes.push(4u8),
            TypeExpr::Fixed => bytes.push(5u8),
            TypeExpr::Ratio => bytes.push(6u8),
            TypeExpr::Number => bytes.push(7u8),
            TypeExpr::List(inner) => {
                bytes.push(8u8);
                bytes.extend_from_slice(&inner.as_ssz_bytes());
            }
            TypeExpr::Map(key, value) => {
                bytes.push(9u8);
                bytes.extend_from_slice(&key.as_ssz_bytes());
                bytes.extend_from_slice(&value.as_ssz_bytes());
            }
            TypeExpr::Record(fields) => {
                bytes.push(10u8);
                bytes.extend_from_slice(&fields.as_ssz_bytes());
            }
            TypeExpr::Union(types) => {
                bytes.push(11u8);
                bytes.extend_from_slice(&types.as_ssz_bytes());
            }
            TypeExpr::Intersection(types) => {
                bytes.push(12u8);
                bytes.extend_from_slice(&types.as_ssz_bytes());
            }
            TypeExpr::Optional(inner) => {
                bytes.push(13u8);
                bytes.extend_from_slice(&inner.as_ssz_bytes());
            }
            TypeExpr::Range(min, max) => {
                bytes.push(14u8);
                bytes.extend_from_slice(&min.to_le_bytes());
                bytes.extend_from_slice(&max.to_le_bytes());
            }
            TypeExpr::Enum(values) => {
                bytes.push(15u8);
                bytes.extend_from_slice(&values.as_ssz_bytes());
            }
            TypeExpr::Tuple(types) => {
                bytes.push(16u8);
                bytes.extend_from_slice(&types.as_ssz_bytes());
            }
            TypeExpr::Function(params, ret) => {
                bytes.push(17u8);
                bytes.extend_from_slice(&params.as_ssz_bytes());
                bytes.extend_from_slice(&ret.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for TypeExpr {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "TypeExpr requires at least 1 byte for variant tag".to_string(),
            });
        }
        
        let variant = bytes[0];
        let data = &bytes[1..];
        
        match variant {
            0 => Ok(TypeExpr::Any),
            1 => Ok(TypeExpr::Unit),
            2 => Ok(TypeExpr::Bool),
            3 => Ok(TypeExpr::String),
            4 => Ok(TypeExpr::Integer),
            5 => Ok(TypeExpr::Fixed),
            6 => Ok(TypeExpr::Ratio),
            7 => Ok(TypeExpr::Number),
            8 => Ok(TypeExpr::List(TypeExprBox::from_ssz_bytes(data)?)),
            9 => {
                // Map requires two TypeExprBox values
                let key = TypeExprBox::from_ssz_bytes(data)?;
                let key_bytes = key.as_ssz_bytes();
                let value = TypeExprBox::from_ssz_bytes(&data[key_bytes.len()..])?;
                Ok(TypeExpr::Map(key, value))
            }
            10 => Ok(TypeExpr::Record(TypeExprMap::from_ssz_bytes(data)?)),
            11 => Ok(TypeExpr::Union(TypeExprVec::from_ssz_bytes(data)?)),
            12 => Ok(TypeExpr::Intersection(TypeExprVec::from_ssz_bytes(data)?)),
            13 => Ok(TypeExpr::Optional(TypeExprBox::from_ssz_bytes(data)?)),
            14 => {
                if data.len() < 16 {
                    return Err(DecodeError {
                        message: "Range requires 16 bytes for min and max".to_string(),
                    });
                }
                let mut min_bytes = [0u8; 8];
                let mut max_bytes = [0u8; 8];
                min_bytes.copy_from_slice(&data[0..8]);
                max_bytes.copy_from_slice(&data[8..16]);
                let min = i64::from_le_bytes(min_bytes);
                let max = i64::from_le_bytes(max_bytes);
                Ok(TypeExpr::Range(min, max))
            }
            15 => Ok(TypeExpr::Enum(TypeExprVec::from_ssz_bytes(data)?)),
            16 => Ok(TypeExpr::Tuple(TypeExprVec::from_ssz_bytes(data)?)),
            17 => {
                // Function requires two TypeExprBox values
                let params = TypeExprBox::from_ssz_bytes(data)?;
                let params_bytes = params.as_ssz_bytes();
                let ret = TypeExprBox::from_ssz_bytes(&data[params_bytes.len()..])?;
                Ok(TypeExpr::Function(params, ret))
            }
            _ => Err(DecodeError {
                message: format!("Invalid TypeExpr variant: {}", variant),
            }),
        }
    }
}

impl SimpleSerialize for TypeExpr {}

//-----------------------------------------------------------------------------
// Standard Type Definitions
//-----------------------------------------------------------------------------

/// Returns the canonical `TypeExpr` for a "TypeBehaviorDefinition" resource.
///
/// A TypeBehaviorDefinition resource holds mappings from behavior names (strings)
/// to the `ExprId`s (represented as strings) of Lisp functions that implement them.
/// The schema is effectively `Map<String, String>` where the value string is an ExprId.
pub fn type_behavior_definition_type_expr() -> TypeExpr {
    TypeExpr::Map(
        Box::new(TypeExpr::String).into(), // Key: behavior name (e.g., "resource.apply_update")
        Box::new(TypeExpr::String).into(), // Value: ExprId of Lisp function (represented as a string)
    )
} 