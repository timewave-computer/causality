//! Meta-effects for schema and type declarations
//!
//! This module defines effects for managing schemas and type declarations within
//! the causality system. These meta-effects allow for runtime definition and
//! validation of types that can be used by other effects in the system.

use causality_types::{
    effects_core::{Effect, EffectInput, EffectOutput, ConversionError},
    expr::{value::ValueExpr, TypeExpr, TypeExprId},
    core::str::Str,
};
use crate::AsTypeSchema;
use anyhow::Result;
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// DeclareType Effect
//-----------------------------------------------------------------------------

/// Effect for declaring a new named type schema in the system.
///
/// This effect introduces a new type identified by `name` with the structure
/// defined by `schema`. Once declared, this named type can be referenced
/// elsewhere.
#[derive(Debug, Clone)]
pub struct DeclareTypeEffect {
    /// The unique name for the type being declared (e.g., "MyCustomAsset").
    /// This name will be used to reference the schema.
    pub name: String,

    /// The `TypeExpr` defining the structure of the new type.
    pub schema: TypeExpr,
    // Note: TypeExpr itself is a complex enum. For schema generation,
    // it might be represented as a generic "Object" or a stringified version,
    // depending on how detailed the schema representation needs to be for effects.
    // For now, we'll treat it as a field that needs to exist.
    // A more detailed schema representation for TypeExpr itself might be needed
    // if we want to validate the schema's content through this effect's schema.
}

impl DeclareTypeEffect {
    /// Creates a new `DeclareTypeEffect`.
    ///
    /// # Arguments
    /// * `name` - The name of the type to declare.
    /// * `schema` - The `TypeExpr` defining the type's structure.
    pub fn new(name: impl Into<String>, schema: TypeExpr) -> Self {
        Self {
            name: name.into(),
            schema,
        }
    }
}

//-----------------------------------------------------------------------------
// Effect Trait Implementation
//-----------------------------------------------------------------------------

/// Empty input type for DeclareTypeEffect
#[derive(Debug, Clone)]
pub struct DeclareTypeEffectInput;

/// Empty output type for DeclareTypeEffect
#[derive(Debug, Clone)]
pub struct DeclareTypeEffectOutput;

impl EffectInput for DeclareTypeEffectInput {
    fn schema() -> TypeExpr {
        TypeExpr::Unit
    }
    
    fn from_value_expr(_value: ValueExpr) -> Result<Self, ConversionError> {
        Ok(Self)
    }
}

impl EffectOutput for DeclareTypeEffectOutput {
    fn schema() -> TypeExpr {
        TypeExpr::Unit
    }
    
    fn to_value_expr(&self) -> Result<ValueExpr, ConversionError> {
        Ok(ValueExpr::Nil)
    }
}

impl Effect for DeclareTypeEffect {
    type Input = DeclareTypeEffectInput;
    type Output = DeclareTypeEffectOutput;
    
    const EFFECT_TYPE: &'static str = "meta.DeclareTypeEffect";
}

// Implement the AsTypeSchema trait

impl AsTypeSchema for DeclareTypeEffect {
    fn type_schema(&self) -> TypeExpr {
        // Create a record type schema with the appropriate fields
        let mut fields = BTreeMap::new();

        // Add required fields
        fields.insert(Str::from("name"), TypeExpr::String);
        fields.insert(Str::from("schema"), TypeExpr::Any); // TypeExpr is complex, using Any for now

        TypeExpr::Record(causality_types::expr::TypeExprMap(fields))
    }

    fn effect_type_name(&self) -> &'static str {
        "meta.DeclareTypeEffect"
    }
}

// Implement AsSchema from causality_types for compatibility
impl causality_types::expr::AsSchema for DeclareTypeEffect {
    fn schema_id(&self) -> TypeExprId {
        <Self as AsTypeSchema>::schema_id(self)
    }
}

// Define the schema for the DeclareTypeEffect itself.
// This describes the structure of DeclareTypeEffect for validation,
// introspection, and potential serialization.
