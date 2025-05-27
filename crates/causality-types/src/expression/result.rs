//! Expression Result Type
//!
//! This module defines the result types returned when evaluating expressions in the
//! Causality Combinator Lisp system. It provides a deterministic error handling system
//! compatible with ZK circuits, allowing expressions to safely indicate success or
//! various failure modes without relying on non-deterministic error propagation.

use std::fmt::{self, Display};

use crate::primitive::ids::{ExprId, ResourceId};
use crate::primitive::string::Str;
use crate::expression::ast::{Atom, AtomicCombinator};
use crate::expression::value::ValueExpr;
use crate::serialization::{Decode, DecodeError, Encode, SimpleSerialize};

//-----------------------------------------------------------------------------
// Type Definition
//-----------------------------------------------------------------------------

/// Type for content-addressed ExprResult references
pub type ExprResultId = ExprId;

/// Box wrapper for ExprResult to break recursive type definition
#[derive(Debug, Clone, PartialEq)]
pub struct ExprResultBox(pub Box<ExprResult>);

impl Encode for ExprResultBox {
    fn as_ssz_bytes(&self) -> Vec<u8> { self.0.as_ssz_bytes() }
}
impl Decode for ExprResultBox {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ExprResultBox(Box::new(ExprResult::from_ssz_bytes(bytes)?)))
    }
}
impl SimpleSerialize for ExprResultBox {}

/// Vector wrapper for ExprResult collections
#[derive(Debug, Clone, PartialEq)]
pub struct ExprResultVec(pub Vec<ExprResult>);

impl Encode for ExprResultVec {
    fn as_ssz_bytes(&self) -> Vec<u8> { self.0.as_ssz_bytes() }
}
impl Decode for ExprResultVec {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ExprResultVec(Vec::<ExprResult>::from_ssz_bytes(bytes)?))
    }
}
impl SimpleSerialize for ExprResultVec {}

/// Result of evaluating an expression in the combinator Lisp
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[repr(align(4))]
pub enum ExprResult {
    /// Atomic value result
    Atom(Atom),

    /// Value result (for backward compatibility)
    Value(ValueExpr),

    /// Boolean result
    Bool(bool),

    /// Resource reference result
    Resource(ResourceId),

    /// Unit/void result (for expressions that don't return a value)
    Unit,

    /// Combinator result (representing a combinator as a value)
    Combinator(AtomicCombinator),

    /// Function reference (for referencing known functions by ID)
    Function(u64),

    /// Reference to an externally implemented host function, identified by name.
    ExternalHostFnRef(Str),
}

impl Encode for ExprResult {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ExprResult::Atom(a) => { bytes.push(0); bytes.extend(a.as_ssz_bytes()); }
            ExprResult::Value(v) => { bytes.push(1); bytes.extend(v.as_ssz_bytes()); }
            ExprResult::Bool(b) => { bytes.push(2); bytes.extend(b.as_ssz_bytes()); }
            ExprResult::Resource(r) => { bytes.push(3); bytes.extend(r.as_ssz_bytes()); }
            ExprResult::Unit => bytes.push(4),
            ExprResult::Combinator(c) => { bytes.push(5); bytes.extend(c.as_ssz_bytes()); }
            ExprResult::Function(f) => { bytes.push(6); bytes.extend(f.as_ssz_bytes()); }
            ExprResult::ExternalHostFnRef(s) => { bytes.push(7); bytes.extend(s.as_ssz_bytes()); }
        }
        bytes
    }
}

impl Decode for ExprResult {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() { return Err(DecodeError{message: "Cannot decode ExprResult from empty bytes".to_string()}); }
        let variant = bytes[0];
        let offset = 1;
        match variant {
            0 => Ok(ExprResult::Atom(Atom::from_ssz_bytes(&bytes[offset..])?)),
            1 => Ok(ExprResult::Value(ValueExpr::from_ssz_bytes(&bytes[offset..])?)),
            2 => Ok(ExprResult::Bool(bool::from_ssz_bytes(&bytes[offset..])?)),
            3 => Ok(ExprResult::Resource(ResourceId::from_ssz_bytes(&bytes[offset..])?)),
            4 => Ok(ExprResult::Unit),
            5 => Ok(ExprResult::Combinator(AtomicCombinator::from_ssz_bytes(&bytes[offset..])?)),
            6 => Ok(ExprResult::Function(u64::from_ssz_bytes(&bytes[offset..])?)),
            7 => Ok(ExprResult::ExternalHostFnRef(Str::from_ssz_bytes(&bytes[offset..])?)),
            _ => Err(DecodeError{message: format!("Invalid ExprResult variant: {}", variant)}),
        }
    }
}
impl SimpleSerialize for ExprResult {}

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Error that may occur during expression evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
#[repr(align(4))]
pub enum ExprError {
    /// Type error (mismatched or unexpected type)
    TypeError(Box<TypeErrorData>),

    /// Reference error (unknown variable or symbol)
    ReferenceError {
        /// Name of the reference that couldn't be resolved
        name: Str,
    },

    /// Execution error (runtime failure)
    ExecutionError {
        /// Description of the execution error
        message: Str,
    },

    /// Permission error (insufficient permissions)
    PermissionError {
        /// Description of the permission error
        message: Str,
        /// Resource ID that couldn't be accessed
        resource: Option<ResourceId>,
    },
}

/// Data for type errors (boxed to reduce enum size)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeErrorData {
    /// Description of the type error
    pub message: Str,
    /// Expression that caused the error
    pub expr: Option<Str>,
}

impl Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError(type_error) => {
                if let Some(expr_str) = &type_error.expr {
                    write!(f, "Type error: {} in expression {}", type_error.message, expr_str)
                } else {
                    write!(f, "Type error: {}", type_error.message)
                }
            }
            Self::ReferenceError { name } => {
                write!(f, "Reference error: unresolved name {}", name)
            }
            Self::ExecutionError { message } => {
                write!(f, "Execution error: {}", message)
            }
            Self::PermissionError { message, resource } => {
                if let Some(res_id) = resource {
                    write!(
                        f,
                        "Permission error: {} for resource {}",
                        message, res_id
                    )
                } else {
                    write!(f, "Permission error: {}", message)
                }
            }
        }
    }
}

impl std::error::Error for ExprError {}

impl Encode for ExprError {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ExprError::TypeError(type_error) => {
                bytes.push(0);
                bytes.extend(type_error.message.as_ssz_bytes());
                bytes.extend(type_error.expr.as_ssz_bytes());
            }
            ExprError::ReferenceError { name } => {
                bytes.push(1);
                bytes.extend(name.as_ssz_bytes());
            }
            ExprError::ExecutionError { message } => {
                bytes.push(2);
                bytes.extend(message.as_ssz_bytes());
            }
            ExprError::PermissionError { message, resource } => {
                bytes.push(3);
                bytes.extend(message.as_ssz_bytes());
                bytes.extend(resource.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for ExprError {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() { return Err(DecodeError{message: "Cannot decode ExprError from empty bytes".to_string()}); }
        let variant = bytes[0];
        let mut offset = 1;
        match variant {
            0 => {
                let message = Str::from_ssz_bytes(&bytes[offset..])?;
                offset += message.as_ssz_bytes().len();
                let expr = Option::<Str>::from_ssz_bytes(&bytes[offset..])?;
                Ok(ExprError::TypeError(Box::new(TypeErrorData { message, expr })))
            }
            1 => {
                let name = Str::from_ssz_bytes(&bytes[offset..])?;
                Ok(ExprError::ReferenceError { name })
            }
            2 => {
                let message = Str::from_ssz_bytes(&bytes[offset..])?;
                Ok(ExprError::ExecutionError { message })
            }
            3 => {
                let message = Str::from_ssz_bytes(&bytes[offset..])?;
                offset += message.as_ssz_bytes().len();
                let resource = Option::<ResourceId>::from_ssz_bytes(&bytes[offset..])?;
                Ok(ExprError::PermissionError { message, resource })
            }
            _ => Err(DecodeError{message: format!("Invalid ExprError variant: {}", variant)}),
        }
    }
}
impl SimpleSerialize for ExprError {}

impl Encode for TypeErrorData {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.message.as_ssz_bytes());
        bytes.extend(self.expr.as_ssz_bytes());
        bytes
    }
}

impl Decode for TypeErrorData {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        let message = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += message.as_ssz_bytes().len();
        let expr = Option::<Str>::from_ssz_bytes(&bytes[offset..])?;
        Ok(TypeErrorData { message, expr })
    }
}

impl SimpleSerialize for TypeErrorData {}
