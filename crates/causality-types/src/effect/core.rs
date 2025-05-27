//! Algebraic Effects System
//!
//! Defines core traits and types for the Rust-based algebraic effect system.
//! This module provides the foundation for defining and handling effects
//! with proper type safety and serialization.

use crate::primitive::string::Str;
use crate::expression::{TypeExpr, ValueExpr};
use async_trait::async_trait; // For async methods in traits
use std::fmt::Debug;

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionError {
    TypeMismatch {
        expected: String, // Could be TypeExpr in the future
        found: String,    // Could be ValueExpr::variant_name() or TypeExpr
    },
    MissingField {
        field_name: String,
    },
    UnexpectedValue {
        description: String,
    },
    Custom(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            ConversionError::MissingField { field_name } => {
                write!(f, "Missing field: {}", field_name)
            }
            ConversionError::UnexpectedValue { description } => {
                write!(f, "Unexpected value: {}", description)
            }
            ConversionError::Custom(msg) => {
                write!(f, "Conversion error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

#[derive(Debug, Clone, PartialEq)]
pub enum HandlerError {
    EffectExecutionFailed(String), // Error during the actual effect logic
    InputConversionFailed(ConversionError),
    OutputConversionFailed(ConversionError),
    InternalError(String), // For issues within the handler/runtime machinery
    LispError(String),     // Error originating from Lisp evaluation
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::EffectExecutionFailed(msg) => {
                write!(f, "Effect execution failed: {}", msg)
            }
            HandlerError::InputConversionFailed(err) => {
                write!(f, "Input conversion failed: {}", err)
            }
            HandlerError::OutputConversionFailed(err) => {
                write!(f, "Output conversion failed: {}", err)
            }
            HandlerError::InternalError(msg) => {
                write!(f, "Handler internal error: {}", msg)
            }
            HandlerError::LispError(msg) => {
                write!(f, "Lisp error: {}", msg)
            }
        }
    }
}

impl std::error::Error for HandlerError {}

//-----------------------------------------------------------------------------
// Core Traits
//-----------------------------------------------------------------------------

/// Represents a type that can be used as an input parameter for an algebraic effect.
pub trait EffectInput: Sized + Send + Sync + Debug + 'static {
    /// Deserializes a `ValueExpr` into an instance of the input struct.
    fn from_value_expr(value: ValueExpr) -> Result<Self, ConversionError>;

    /// Provides the canonical schema (`TypeExpr`) for this input type.
    fn schema() -> TypeExpr;
}

/// Represents a type that can be used as an output/result of an algebraic effect.
pub trait EffectOutput: Sized + Send + Sync + Debug + 'static {
    /// Serializes the output struct into a `ValueExpr`.
    fn to_value_expr(&self) -> Result<ValueExpr, ConversionError>;

    /// Provides the canonical schema (`TypeExpr`) for this output type.
    fn schema() -> TypeExpr;
}

/// Defines an algebraic effect, including its type identifier, input, and output types.
pub trait Effect: Send + Sync + Debug + 'static {
    /// The type of the input parameters for this effect.
    type Input: EffectInput;
    /// The type of the output/result of this effect.
    type Output: EffectOutput;

    /// A unique string literal identifying this specific kind of effect.
    /// This should be namespaced to avoid collisions (e.g., "com.example.MyEffect").
    const EFFECT_TYPE: &'static str;

    /// Gets the unique type identifier for this effect.
    fn effect_type(&self) -> Str {
        Str::new(Self::EFFECT_TYPE)
    }

    /// Gets the schema for the input parameters of this effect.
    fn input_schema(&self) -> TypeExpr {
        Self::Input::schema()
    }

    /// Gets the schema for the output/result of this effect.
    fn output_schema(&self) -> TypeExpr {
        Self::Output::schema()
    }
}

/// Defines a handler for a specific algebraic effect.
#[async_trait]
pub trait EffectHandler: Send + Sync + 'static {
    /// The specific effect this handler is responsible for.
    type E: Effect;

    /// Handles an instance of the effect.
    ///
    /// # Arguments
    /// * `input`: The deserialized input parameters for the effect.
    ///
    /// # Returns
    /// A result containing the effect's output or a `HandlerError`.
    async fn handle(
        &self,
        input: <Self::E as Effect>::Input,
    ) -> Result<<Self::E as Effect>::Output, HandlerError>;
}

//-----------------------------------------------------------------------------
// Handler Registration
//-----------------------------------------------------------------------------

/// Information needed to register an effect handler dynamically.
/// This might be collected at compile time using a procedural macro.
pub struct HandlerRegistrationInfo {
    /// The unique type string of the effect this handler processes (e.g., "my.domain.MyEffect").
    /// This comes from `Effect::EFFECT_TYPE`.
    pub effect_type_str: &'static str,
    /// The fully qualified path to the handler struct (e.g., "my_crate::my_module::MyHandler").
    pub handler_struct_path: &'static str,
    /// The fully qualified path to the effect struct this handler is for (e.g., "my_crate::my_module::MyEffect").
    pub effect_struct_path: &'static str,
}

// Submit this struct to inventory when a handler is registered by the macro.
// Example (to be done by the proc macro):
// inventory::submit!(HandlerRegistrationInfo { /* ... */ });

// To iterate over all registered handlers:
// for registration_info in inventory::iter::<HandlerRegistrationInfo> { /* ... */ }

// Re-export inventory for the macro to use if it's in a different crate
pub use inventory;
