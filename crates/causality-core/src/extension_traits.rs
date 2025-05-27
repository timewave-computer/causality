//! Extension traits for the causality-types crate
//!
//! This module provides extension traits that add functionality to types from 
//! causality-types without violating Rust's orphan rules. These traits allow us to
//! maintain a clean separation between type definitions and implementations.

// use crate::expr_utils::{expr_as_value, value_as_expr}; // Removed, as value_as_expr doesn't exist and expr_as_value was unused.
use causality_types::{
    core::id::{CapabilityId, DomainId, ExprId, ResourceId, TypeExprId},
    effects_core::ConversionError, // EffectInput/Output unused
    expr::{
        ast::Expr,
        expr_type::TypeExpr,
        value::ValueExpr,
    },
    serialization::{Decode, Encode},
};
use sha2::{Digest, Sha256};

//-----------------------------------------------------------------------------
// Extension Traits
//-----------------------------------------------------------------------------

/// Extension trait for DomainId
pub trait DomainIdExt {
    /// Create a new deterministic DomainId from a string and salt
    fn new_deterministic(name: &str, salt: &[u8]) -> DomainId;
}

impl DomainIdExt for DomainId {
    fn new_deterministic(name: &str, salt: &[u8]) -> DomainId {
        let mut input = Vec::with_capacity(name.len() + salt.len() + 8);
        input.extend_from_slice(name.as_bytes());
        input.extend_from_slice(salt);
        
        let mut hasher = Sha256::new();
        hasher.update(&input);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hasher.finalize());
        DomainId::new(bytes)
    }
}

/// Extension trait for ValueExpr
pub trait ValueExprExt {
    /// Get a unique ID for this ValueExpr
    fn id(&self) -> causality_types::primitive::ids::ValueExprId;
    
    /// Convert to bytes
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> where Self: Sized;
}

/// Extension trait for TypeExpr
pub trait TypeExprExt {
    /// Get a unique ID for this TypeExpr
    fn id(&self) -> causality_types::primitive::ids::TypeExprId;
    
    /// Convert to bytes
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> where Self: Sized;
}

/// Extension trait for Expr
pub trait ExprExt {
    /// Get a unique ID for this Expr
    fn id(&self) -> causality_types::primitive::ids::ExprId;
    
    /// Convert to bytes
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> where Self: Sized;
}

/// Extension trait for primitive types that can be used as effect inputs
pub trait PrimitiveEffectInput {
    /// Convert from a ValueExpr
    fn from_value_expr_ext(value: ValueExpr) -> Result<Self, ConversionError> where Self: Sized;
    
    /// Get the schema
    fn schema_ext() -> TypeExpr;
}

/// Extension trait for primitive types that can be used as effect outputs
pub trait PrimitiveEffectOutput {
    /// Convert to a ValueExpr
    fn to_value_expr_ext(&self) -> Result<ValueExpr, ConversionError>;
    
    /// Get the schema
    fn schema_ext() -> TypeExpr;
}

/// Extension trait for parsing IDs from strings
pub trait IdFromStr {
    /// Parse from a string
    fn from_str_ext(s: &str) -> Result<Self, anyhow::Error> where Self: Sized;
}

//-----------------------------------------------------------------------------
// Extension Trait Implementations
//-----------------------------------------------------------------------------

// Implementations for ValueExpr
impl ValueExprExt for ValueExpr {
    fn id(&self) -> causality_types::primitive::ids::ValueExprId {
        let bytes = self.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hasher.finalize());
        causality_types::primitive::ids::ValueExprId::new(hash)
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Self::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("Failed to deserialize ValueExpr: {}", e))
    }
}

// Implementations for TypeExpr
impl TypeExprExt for TypeExpr {
    fn id(&self) -> causality_types::primitive::ids::TypeExprId {
        let bytes = self.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hasher.finalize());
        causality_types::primitive::ids::TypeExprId::new(hash)
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Self::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("Failed to deserialize TypeExpr: {}", e))
    }
}

// Implementations for Expr
impl ExprExt for Expr {
    fn id(&self) -> causality_types::primitive::ids::ExprId {
        let bytes = self.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hasher.finalize());
        causality_types::primitive::ids::ExprId::new(hash)
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Self::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("Failed to deserialize Expr: {}", e))
    }
}

// Implementations for primitive types as effect inputs/outputs
impl PrimitiveEffectInput for String {
    fn from_value_expr_ext(value: ValueExpr) -> Result<Self, ConversionError> {
        match value {
            ValueExpr::String(s) => Ok(s.to_string()),
            other => Err(ConversionError::TypeMismatch {
                expected: "String".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }
    
    fn schema_ext() -> TypeExpr {
        TypeExpr::String
    }
}

impl PrimitiveEffectOutput for String {
    fn to_value_expr_ext(&self) -> Result<ValueExpr, ConversionError> {
        use causality_types::primitive::string::Str;
        Ok(ValueExpr::String(Str::from(self.clone())))
    }
    
    fn schema_ext() -> TypeExpr {
        TypeExpr::String
    }
}

impl PrimitiveEffectInput for i64 {
    fn from_value_expr_ext(value: ValueExpr) -> Result<Self, ConversionError> {
        match value {
            ValueExpr::Number(num) => match num {
                causality_types::primitive::number::Number::Integer(i) => Ok(i),
                causality_types::primitive::number::Number::Decimal(d) => {
                    // Convert rational to integer by truncating
                    let integer_part = d.trunc();
                    // Convert to i64 safely
                    if let Ok(val) = integer_part.to_string().parse::<i64>() {
                        Ok(val)
                    } else {
                        Ok(0) // Fallback for very large numbers
                    }
                },
            },
            other => Err(ConversionError::TypeMismatch {
                expected: "Number".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }

    fn schema_ext() -> TypeExpr {
        TypeExpr::Integer
    }
}

impl PrimitiveEffectOutput for i64 {
    fn to_value_expr_ext(&self) -> Result<ValueExpr, ConversionError> {
        use causality_types::primitive::number::Number;
        Ok(ValueExpr::Number(Number::new_integer(*self)))
    }

    fn schema_ext() -> TypeExpr {
        TypeExpr::Integer
    }
}

impl PrimitiveEffectInput for bool {
    fn from_value_expr_ext(value: ValueExpr) -> Result<Self, ConversionError> {
        match value {
            ValueExpr::Bool(b) => Ok(b),
            other => Err(ConversionError::TypeMismatch {
                expected: "Bool".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }
    
    fn schema_ext() -> TypeExpr {
        TypeExpr::Bool
    }
}

impl PrimitiveEffectOutput for bool {
    fn to_value_expr_ext(&self) -> Result<ValueExpr, ConversionError> {
        Ok(ValueExpr::Bool(*self))
    }
    
    fn schema_ext() -> TypeExpr {
        TypeExpr::Bool
    }
}

// Implementations for ID types
impl IdFromStr for ResourceId {
    fn from_str_ext(s: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid ResourceId length"));
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&bytes);
        Ok(ResourceId::new(id_bytes))
    }
}

impl IdFromStr for ExprId {
    fn from_str_ext(s: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid ExprId length"));
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&bytes);
        Ok(ExprId::new(id_bytes))
    }
}

impl IdFromStr for TypeExprId {
    fn from_str_ext(s: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid TypeExprId length"));
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&bytes);
        Ok(TypeExprId::new(id_bytes))
    }
}

impl IdFromStr for DomainId {
    fn from_str_ext(s: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid DomainId length"));
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&bytes);
        Ok(DomainId::new(id_bytes))
    }
}

impl IdFromStr for CapabilityId {
    fn from_str_ext(s: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid CapabilityId length"));
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&bytes);
        Ok(CapabilityId::new(id_bytes))
    }
}
