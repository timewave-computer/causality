//! Control flow effects
//!
//! This module provides effects for controlling the flow of execution, such as conditional
//! execution, loops, and parallel execution.

use anyhow::Result;
use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use causality_types::{
    core::id::TypeExprId,
    core::str::Str,
    expr::{TypeExpr, TypeExprMap, TypeExprBox, ValueExpr},
    effects_core::{Effect, EffectInput, EffectOutput, ConversionError},
};

use crate::core::CloneableEffectBox;
use crate::{AsTypeSchema, ToolkitResult};

//-----------------------------------------------------------------------------
// Trait Definitions
//-----------------------------------------------------------------------------

/// Trait for effects that can be handled by the toolkit
pub trait HandleableEffect: Send + Sync + 'static {
    /// Handle the effect with the given handler
    fn handle(
        &self,
        handler: &dyn SimpleEffectHandler,
    ) -> ToolkitResult<()>;
    
    /// Allow downcasting to a concrete type
    fn as_any(&self) -> &dyn Any;
}

/// Simple effect handler trait
pub trait SimpleEffectHandler: Send + Sync + 'static {
    /// Handle an effect
    fn handle_any(&self, effect: &dyn HandleableEffect) -> ToolkitResult<()>;
}

//-----------------------------------------------------------------------------
// Input/Output Types for Control Flow Effects
//-----------------------------------------------------------------------------

/// Input type for control flow effects
#[derive(Debug, Clone)]
pub struct ControlFlowInput {
    pub condition: bool,
    pub parameters: ValueExpr,
}

impl EffectInput for ControlFlowInput {
    fn from_value_expr(value: ValueExpr) -> Result<Self, ConversionError> {
        match value {
            ValueExpr::Record(record) => {
                let condition = record.0.get(&Str::from("condition"))
                    .and_then(|v| match v {
                        ValueExpr::Bool(b) => Some(*b),
                        _ => None,
                    })
                    .unwrap_or(false);
                
                let parameters = record.0.get(&Str::from("parameters"))
                    .cloned()
                    .unwrap_or(ValueExpr::Nil);
                
                Ok(ControlFlowInput { condition, parameters })
            }
            _ => Err(ConversionError::TypeMismatch {
                expected: "Record".to_string(),
                found: "Other".to_string(),
            }),
        }
    }

    fn schema() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("condition"), TypeExpr::Bool);
        fields.insert(Str::from("parameters"), TypeExpr::Any);
        TypeExpr::Record(TypeExprMap(fields))
    }
}

/// Output type for control flow effects
#[derive(Debug, Clone)]
pub struct ControlFlowOutput {
    pub result: ValueExpr,
}

impl EffectOutput for ControlFlowOutput {
    fn to_value_expr(&self) -> Result<ValueExpr, ConversionError> {
        Ok(self.result.clone())
    }

    fn schema() -> TypeExpr {
        TypeExpr::Any
    }
}

//-----------------------------------------------------------------------------
// If Effect
//-----------------------------------------------------------------------------

/// Effect for conditional execution
#[derive(Clone)]
pub struct IfEffect {
    /// Condition to check
    pub condition: bool,

    /// Effect to execute if condition is true
    pub then_effect: CloneableEffectBox,

    /// Optional effect to execute if condition is false
    pub else_effect: Option<CloneableEffectBox>,
}

impl fmt::Debug for IfEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IfEffect")
            .field("condition", &self.condition)
            .field("then_effect", &self.then_effect)
            .field("else_effect", &self.else_effect)
            .finish()
    }
}

impl IfEffect {
    /// Create a new if effect
    pub fn new<E>(condition: bool, then_effect: E) -> Self
    where
        E: HandleableEffect + Send + Sync + 'static,
    {
        Self {
            condition,
            then_effect: CloneableEffectBox::new(then_effect),
            else_effect: None,
        }
    }

    /// Add an else effect
    pub fn with_else<E>(mut self, else_effect: E) -> Self
    where
        E: HandleableEffect + Send + Sync + 'static,
    {
        self.else_effect = Some(CloneableEffectBox::new(else_effect));
        self
    }
}

//-----------------------------------------------------------------------------
// IfEffect Implementation
//-----------------------------------------------------------------------------

// Core Effect trait implementation
impl Effect for IfEffect {
    type Input = ControlFlowInput;
    type Output = ControlFlowOutput;
    
    const EFFECT_TYPE: &'static str = "control_flow.IfEffect";
}

// Toolkit-specific handling extension
impl HandleableEffect for IfEffect {
    fn handle(
        &self,
        _handler: &dyn SimpleEffectHandler,
    ) -> ToolkitResult<()> {
        // Simplified implementation - just return Ok for now
        // In a real implementation, this would execute the appropriate effect
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implement AsTypeSchema for IfEffect
impl AsTypeSchema for IfEffect {
    fn type_schema(&self) -> TypeExpr {
        // Create a record type schema with the appropriate fields
        let mut fields = BTreeMap::new();

        // Add required fields
        fields.insert(Str::from("condition"), TypeExpr::Bool);
        fields.insert(Str::from("then_effect"), TypeExpr::Any); // CloneableEffectBox is complex
        fields.insert(Str::from("else_effect"), TypeExpr::Optional(TypeExprBox(Box::new(TypeExpr::Any))));

        TypeExpr::Record(TypeExprMap(fields))
    }

    fn effect_type_name(&self) -> &'static str {
        Self::EFFECT_TYPE
    }
}

// Implement AsSchema from causality_types for compatibility
impl causality_types::expr::AsSchema for IfEffect {
    fn schema_id(&self) -> TypeExprId {
        <Self as AsTypeSchema>::schema_id(self)
    }
}

//-----------------------------------------------------------------------------
// Sequence Effect
//-----------------------------------------------------------------------------

/// Effect for sequential execution of multiple effects
#[derive(Clone)]
pub struct SequenceEffect {
    /// Effects to execute in sequence
    pub effects: Vec<CloneableEffectBox>,
}

impl fmt::Debug for SequenceEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SequenceEffect")
            .field("effects", &self.effects)
            .finish()
    }
}

impl SequenceEffect {
    /// Create a new sequence effect
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add an effect to the sequence
    pub fn with_effect<E>(mut self, effect: E) -> Self
    where
        E: HandleableEffect + Send + Sync + 'static,
    {
        self.effects.push(CloneableEffectBox::new(effect));
        self
    }
}

//-----------------------------------------------------------------------------
// SequenceEffect Implementation
//-----------------------------------------------------------------------------

// Core Effect trait implementation
impl Effect for SequenceEffect {
    type Input = ControlFlowInput;
    type Output = ControlFlowOutput;
    
    const EFFECT_TYPE: &'static str = "control_flow.SequenceEffect";
}

// Toolkit-specific handling extension
impl HandleableEffect for SequenceEffect {
    fn handle(
        &self,
        _handler: &dyn SimpleEffectHandler,
    ) -> ToolkitResult<()> {
        // Simplified implementation - just return Ok for now
        // In a real implementation, this would execute all effects in sequence
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implement AsTypeSchema for SequenceEffect
impl AsTypeSchema for SequenceEffect {
    fn type_schema(&self) -> TypeExpr {
        // Create a record type schema with the appropriate fields
        let mut fields = BTreeMap::new();

        // Add required fields
        fields.insert(Str::from("effects"), TypeExpr::List(TypeExprBox(Box::new(TypeExpr::Any)))); // List of effects

        TypeExpr::Record(TypeExprMap(fields))
    }

    fn effect_type_name(&self) -> &'static str {
        Self::EFFECT_TYPE
    }
}

// Implement AsSchema from causality_types for compatibility
impl causality_types::expr::AsSchema for SequenceEffect {
    fn schema_id(&self) -> TypeExprId {
        <Self as AsTypeSchema>::schema_id(self)
    }
}

//-----------------------------------------------------------------------------
// While Effect
//-----------------------------------------------------------------------------

/// Effect for conditionally repeating an effect
pub struct WhileEffect {
    /// Condition function - not cloneable so must be manually managed
    condition_fn: Arc<Mutex<dyn Fn() -> bool + Send + Sync>>,

    /// Body effect to execute in each iteration
    pub body_effect: CloneableEffectBox,

    /// Maximum iterations to prevent infinite loops
    pub max_iterations: usize,
}

// Debug implementation ignores the condition_fn
impl fmt::Debug for WhileEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WhileEffect")
            .field("body_effect", &self.body_effect)
            .field("max_iterations", &self.max_iterations)
            .finish()
    }
}

// Clone implementation creates a dummy condition that always returns false
impl Clone for WhileEffect {
    fn clone(&self) -> Self {
        Self {
            condition_fn: Arc::new(Mutex::new(|| false)),
            body_effect: self.body_effect.clone(),
            max_iterations: self.max_iterations,
        }
    }
}

impl WhileEffect {
    /// Create a new while effect
    pub fn new<F, E>(condition: F, body_effect: E, max_iterations: usize) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
        E: HandleableEffect + Send + Sync + 'static,
    {
        Self {
            condition_fn: Arc::new(Mutex::new(condition)),
            body_effect: CloneableEffectBox::new(body_effect),
            max_iterations,
        }
    }
}

//-----------------------------------------------------------------------------
// WhileEffect Implementation
//-----------------------------------------------------------------------------

// Core Effect trait implementation
impl Effect for WhileEffect {
    type Input = ControlFlowInput;
    type Output = ControlFlowOutput;
    
    const EFFECT_TYPE: &'static str = "control_flow.WhileEffect";
}

// Toolkit-specific handling extension
impl HandleableEffect for WhileEffect {
    fn handle(
        &self,
        _handler: &dyn SimpleEffectHandler,
    ) -> ToolkitResult<()> {
        // Simplified implementation - just return Ok for now
        // In a real implementation, this would execute the body effect while condition is true
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implement AsTypeSchema for WhileEffect
impl AsTypeSchema for WhileEffect {
    fn type_schema(&self) -> TypeExpr {
        // Create a record type schema with the appropriate fields
        let mut fields = BTreeMap::new();

        // Add required fields
        fields.insert(Str::from("body_effect"), TypeExpr::Any);
        fields.insert(Str::from("max_iterations"), TypeExpr::Number);

        TypeExpr::Record(TypeExprMap(fields))
    }

    fn effect_type_name(&self) -> &'static str {
        Self::EFFECT_TYPE
    }
}

// Implement AsSchema from causality_types for compatibility
impl causality_types::expr::AsSchema for WhileEffect {
    fn schema_id(&self) -> TypeExprId {
        <Self as AsTypeSchema>::schema_id(self)
    }
}
