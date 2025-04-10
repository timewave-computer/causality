//! Effect Types for the Temporal Effect Language
//!
//! This module implements the effect type system for tracking and handling
//! effects in TEL. It builds on the row type system to represent extensible
//! sets of effects.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::sync::Arc;
use std::any::Any;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use super::{TelType, BaseType, RecordType, row::{RowType, RowError}};
use causality_core::effect::{
    Effect as CoreEffect, 
    EffectError as CoreEffectError, 
    EffectType as CoreEffectType,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
    EffectResult as CoreEffectResult,
    EffectRegistry as CoreEffectRegistry
};
use crate::combinators::Combinator;

/// Error type for effect operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    /// Error during row operations
    RowError(RowError),
    /// The effect was not found
    EffectNotFound(String),
    /// Invalid effect type
    InvalidEffectType(String),
    /// Invalid handler type
    InvalidHandlerType(String),
    /// Core effect error
    CoreError(String),
    /// Missing capability
    MissingCapability(String),
    /// Resource error
    ResourceError(String),
    /// Validation error
    ValidationError(String),
    /// Execution error
    ExecutionError(String),
}

impl From<RowError> for EffectError {
    fn from(err: RowError) -> Self {
        EffectError::RowError(err)
    }
}

impl From<CoreEffectError> for EffectError {
    fn from(err: CoreEffectError) -> Self {
        match err {
            CoreEffectError::NotFound(msg) => EffectError::EffectNotFound(msg),
            CoreEffectError::HandlerNotFound(msg) => EffectError::EffectNotFound(msg), 
            CoreEffectError::ValidationError(msg) => EffectError::ValidationError(msg),
            CoreEffectError::InvalidArgument(msg) => EffectError::InvalidHandlerType(msg),
            CoreEffectError::MissingCapability(msg) => EffectError::MissingCapability(msg),
            CoreEffectError::MissingResource(msg) => EffectError::ResourceError(msg),
            CoreEffectError::ResourceAccessDenied(msg) => EffectError::ResourceError(msg),
            _ => EffectError::CoreError(err.to_string()),
        }
    }
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::RowError(err) => write!(f, "Row error: {}", err),
            EffectError::EffectNotFound(effect) => write!(f, "Effect not found: {}", effect),
            EffectError::InvalidEffectType(msg) => write!(f, "Invalid effect type: {}", msg),
            EffectError::InvalidHandlerType(msg) => write!(f, "Invalid handler type: {}", msg),
            EffectError::CoreError(msg) => write!(f, "Core effect error: {}", msg),
            EffectError::MissingCapability(msg) => write!(f, "Missing capability: {}", msg),
            EffectError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            EffectError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            EffectError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

/// An effect row in the TEL type system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectRow {
    /// The effects in the row
    pub effects: BTreeMap<String, TelType>,
    
    /// The extension variable, if any
    pub extension: Option<String>,
}

impl EffectRow {
    /// Create a new empty effect row with no extension
    pub fn empty() -> Self {
        EffectRow {
            effects: BTreeMap::new(),
            extension: None,
        }
    }
    
    /// Create a new effect row with the given effects and no extension
    pub fn from_effects(effects: BTreeMap<String, TelType>) -> Self {
        EffectRow {
            effects,
            extension: None,
        }
    }
    
    /// Create a new effect row with the given effects and extension
    pub fn with_extension(effects: BTreeMap<String, TelType>, extension: String) -> Self {
        EffectRow {
            effects,
            extension: Some(extension),
        }
    }
    
    /// Create an effect row with just an extension variable
    pub fn from_extension(extension: String) -> Self {
        EffectRow {
            effects: BTreeMap::new(),
            extension: Some(extension),
        }
    }
    
    /// Check if the effect row has an effect
    pub fn has_effect(&self, effect: &str) -> bool {
        self.effects.contains_key(effect)
    }
    
    /// Get the type of an effect
    pub fn get_effect(&self, effect: &str) -> Option<&TelType> {
        self.effects.get(effect)
    }
    
    /// Add an effect to the row
    pub fn with_effect(mut self, effect: String, effect_type: TelType) -> Result<Self, RowError> {
        if self.has_effect(&effect) {
            return Err(RowError::FieldAlreadyExists(effect));
        }
        self.effects.insert(effect, effect_type);
        Ok(self)
    }
    
    /// Remove an effect from the row
    pub fn without_effect(mut self, effect: &str) -> Result<Self, RowError> {
        if !self.has_effect(effect) {
            return Err(RowError::FieldDoesNotExist(effect.to_string()));
        }
        self.effects.remove(effect);
        Ok(self)
    }
    
    /// Combine two effect rows if they have no effects in common
    pub fn union(&self, other: &EffectRow) -> Result<EffectRow, RowError> {
        // Check for effect conflicts
        for effect in self.effects.keys() {
            if other.has_effect(effect) {
                return Err(RowError::FieldConflict(effect.clone()));
            }
        }
        
        // Merge effects
        let mut merged_effects = self.effects.clone();
        for (effect, effect_type) in &other.effects {
            merged_effects.insert(effect.clone(), effect_type.clone());
        }
        
        // Handle extensions
        let extension = match (&self.extension, &other.extension) {
            (Some(ext1), Some(ext2)) => {
                // Both have extensions - create a union constraint
                // For now, we'll use the first extension and assume a constraint exists
                Some(ext1.clone())
            },
            (Some(ext), None) => Some(ext.clone()),
            (None, Some(ext)) => Some(ext.clone()),
            (None, None) => None,
        };
        
        Ok(EffectRow {
            effects: merged_effects,
            extension,
        })
    }
    
    /// Check if this effect row is a subtype of another effect row
    pub fn is_subtype(&self, other: &EffectRow) -> bool {
        // All effects in other must be in self with compatible types
        for (effect, other_type) in &other.effects {
            match self.get_effect(effect) {
                Some(self_type) => {
                    if !self_type.is_subtype(other_type) {
                        return false;
                    }
                },
                None => return false,
            }
        }
        
        // Handle extensions
        match (&self.extension, &other.extension) {
            // If other has an extension and self doesn't, not a subtype
            (None, Some(_)) => false,
            // Other cases are compatible
            _ => true,
        }
    }
}

impl fmt::Display for EffectRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        
        for (effect, effect_type) in &self.effects {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", effect, effect_type)?;
            first = false;
        }
        
        if let Some(ext) = &self.extension {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "| {}", ext)?;
        }
        
        Ok(())
    }
}

/// TEL Effect - bridges TEL's effect system with causality-core
pub struct TelEffect {
    /// Name of the effect
    pub name: String,
    
    /// Parameters for the effect
    pub parameters: serde_json::Value,
    
    /// Return type expected from the effect
    pub return_type: TelType,
    
    /// Effect metadata
    pub metadata: BTreeMap<String, String>,

    /// Combinator for the effect
    pub combinator: Combinator,
}

impl TelEffect {
    /// Create a new TEL effect
    pub fn new(
        name: impl Into<String>,
        parameters: serde_json::Value,
        return_type: TelType,
    ) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            parameters,
            return_type,
            metadata: BTreeMap::new(),
            combinator: Combinator::Effect {
                effect_name: name_str,
                args: vec![],
                core_effect: None,
            },
        }
    }
    
    /// Add metadata to the effect
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Functions for working with effect types in the TEL type system
pub mod operations {
    use super::*;
    use crate::types::TypeEnvironment;
    
    /// Get the effect type for a specific effect name
    pub fn effect_type(
        effect_name: &str,
        effect_row: &EffectRow,
    ) -> Result<TelType, EffectError> {
        effect_row.get_effect(effect_name)
            .cloned()
            .ok_or_else(|| EffectError::EffectNotFound(effect_name.to_string()))
    }
    
    /// Get the effect row from a type (typically a function type)
    pub fn get_effect_row(
        tel_type: &TelType,
    ) -> Result<EffectRow, EffectError> {
        // For now, just return an empty effect row since our type system 
        // doesn't explicitly store effect information in the function type
        Ok(EffectRow::empty())
    }
    
    /// Get the result type of an effect invocation
    pub fn get_result_type(
        effect_name: &str,
        effect_row: &EffectRow,
    ) -> Result<TelType, EffectError> {
        let effect_type = effect_type(effect_name, effect_row)?;
        
        // Typically, effect type is a function where return type is the result type
        match effect_type {
            TelType::Function(_, return_type) => Ok(*return_type),
            _ => Err(EffectError::InvalidEffectType(
                format!("Effect {} is not a function type", effect_name)
            ))
        }
    }
    
    /// Get the handler type for a specific effect
    pub fn handler_type(
        effect_name: &str,
        effect_row: &EffectRow,
    ) -> Result<TelType, EffectError> {
        let effect_type = effect_type(effect_name, effect_row)?;
        
        // A handler is a function that takes the effect's parameter type
        // and returns the effect's return type
        match effect_type {
            TelType::Function(param_type, return_type) => {
                Ok(TelType::Function(
                    param_type.clone(),
                    return_type.clone(),
                ))
            },
            _ => Err(EffectError::InvalidEffectType(
                format!("Effect {} is not a function type", effect_name)
            ))
        }
    }
    
    /// Apply a handler to a function with effects
    pub fn apply_handler(
        handler: &TelType,
        func: &TelType,
        effect_name: &str,
    ) -> Result<TelType, EffectError> {
        // Check that both handler and func are function types
        match (handler, func) {
            (TelType::Function(h_param, h_return), TelType::Function(f_param, f_return)) => {
                // Check parameter compatibility
                if !f_param.is_subtype(h_param) {
                    return Err(EffectError::InvalidHandlerType(
                        "Handler parameter type isn't compatible with function".to_string()
                    ));
                }
                
                // Return a function with the same signature as func,
                // which would have the effect handled
                Ok(TelType::Function(
                    f_param.clone(),
                    f_return.clone(),
                ))
            },
            _ => Err(EffectError::InvalidHandlerType(
                "Both handler and function must be function types".to_string()
            ))
        }
    }
    
    /// Sequence two effect rows
    pub fn sequence_effects(
        first: &EffectRow,
        second: &EffectRow,
    ) -> Result<EffectRow, EffectError> {
        first.union(second)
            .map_err(|e| e.into())
    }

    /// Add an effect to a function's effect set
    pub fn add_effect(
        func_type: &TelType,
        effect: &str,
        effect_type: &TelType,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match func_type {
            TelType::Function(param_type, return_type) => {
                // Create a new function type with the effect added
                let effect_row = EffectRow::empty().with_effect(
                    effect.to_string(),
                    effect_type.clone(),
                )?;
                
                // Create a new function type with the effect
                Ok(TelType::Function(
                    param_type.clone(),
                    return_type.clone(),
                ))
            },
            _ => {
                // Only function types can have effects added
                Err(RowError::FieldDoesNotExist(
                    "Cannot add effect to a non-function type".to_string(),
                ))
            },
        }
    }
    
    /// Remove an effect from a function's effect set (e.g., after handling)
    pub fn remove_effect(
        func_type: &TelType,
        effect: &str,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        // For now, we just return the function type unchanged
        // as our current TelType::Function doesn't store effects directly
        match func_type {
            TelType::Function(param_type, return_type) => {
                Ok(TelType::Function(
                    param_type.clone(),
                    return_type.clone(),
                ))
            },
            _ => {
                // Only function types can have effects removed
                Err(RowError::FieldDoesNotExist(
                    "Cannot remove effect from a non-function type".to_string(),
                ))
            },
        }
    }
    
    /// Create an effect handler for a specific effect
    pub fn handle_effect(
        handler_type: &TelType,
        effect: &str,
        func_type: &TelType,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        // A handler must be a function that takes an effect operation and returns
        // a result of the appropriate type
        match (handler_type, func_type) {
            (
                TelType::Function(handler_param, handler_return),
                TelType::Function(func_param, func_return),
            ) => {
                // Handler must accommodate the function's parameter type
                if !func_param.is_subtype(handler_param) {
                    return Err(RowError::FieldDoesNotExist(
                        "Handler parameter type doesn't match function".to_string(),
                    ));
                }
                
                // Return a function with the same signature
                Ok(TelType::Function(
                    func_param.clone(),
                    func_return.clone(),
                ))
            },
            _ => {
                // Both types must be functions
                Err(RowError::FieldDoesNotExist(
                    "Both handler and function must be function types".to_string(),
                ))
            },
        }
    }
    
    /// Composite effect handlers for multiple effects
    pub fn compose_handlers(
        handler1: &TelType,
        handler2: &TelType,
        env: &TypeEnvironment,
    ) -> Result<TelType, RowError> {
        match (handler1, handler2) {
            (
                TelType::Function(h1_param, h1_return),
                TelType::Function(h2_param, h2_return),
            ) => {
                // For simplicity, we'll just check parameter compatibility
                if !h2_param.is_subtype(h1_param) {
                    return Err(RowError::FieldDoesNotExist(
                        "Handler parameter types aren't compatible".to_string()
                    ));
                }
                
                // Return a function with the first parameter and second return type
                Ok(TelType::Function(
                    h1_param.clone(),
                    h2_return.clone(),
                ))
            },
            _ => {
                // Both types must be functions
                Err(RowError::FieldDoesNotExist(
                    "Both handler types must be function types".to_string(),
                ))
            },
        }
    }

    /// Convert a TEL effect name to a causality-core effect type
    pub fn to_core_effect_type(effect_name: &str) -> CoreEffectType {
        // Use From trait provided by causality-core
        CoreEffectType::from(effect_name)
    }
    
    /// Create a TEL effect from parameters
    pub fn create_tel_effect(
        effect_name: &str,
        parameters: serde_json::Value,
        return_type: TelType,
    ) -> TelEffect {
        TelEffect::new(effect_name, parameters, return_type)
    }
}

/// Helper macros for working with effect types
pub mod macros {
    /// Create an effect row from a list of effect:type pairs
    #[macro_export]
    macro_rules! effects {
        // Empty effect row
        () => {
            EffectRow {
                effects: BTreeMap::new(),
                extension: None,
            }
        };
        
        // Effect row with effects and no extension
        ($($effect:ident : $type:expr),* $(,)?) => {{
            let mut effects = BTreeMap::new();
            $(
                effects.insert(stringify!($effect).to_string(), $type);
            )*
            EffectRow {
                effects,
                extension: None,
            }
        }};
        
        // Effect row with effects and extension
        ($($effect:ident : $type:expr),* ; $ext:ident) => {{
            let mut effects = BTreeMap::new();
            $(
                effects.insert(stringify!($effect).to_string(), $type);
            )*
            EffectRow {
                effects,
                extension: Some(stringify!($ext).to_string()),
            }
        }};
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TelType, BaseType, TypeEnvironment};
    
    #[test]
    fn test_effect_row_display() {
        let mut effects = BTreeMap::new();
        effects.insert("state".to_string(), TelType::Base(BaseType::String));
        effects.insert("io".to_string(), TelType::Base(BaseType::Unit));
        
        let row = EffectRow {
            effects,
            extension: Some("e".to_string()),
        };
        
        // BTreeMap keys are sorted
        assert_eq!(format!("{}", row), "io: Unit, state: String, | e");
    }
    
    #[test]
    fn test_effect_row_union() {
        let mut effects1 = BTreeMap::new();
        effects1.insert("state".to_string(), TelType::Base(BaseType::String));
        
        let mut effects2 = BTreeMap::new();
        effects2.insert("io".to_string(), TelType::Base(BaseType::Unit));
        
        let row1 = EffectRow {
            effects: effects1,
            extension: None,
        };
        
        let row2 = EffectRow {
            effects: effects2,
            extension: None,
        };
        
        let row3 = row1.union(&row2).unwrap();
        
        assert!(row3.has_effect("state"));
        assert!(row3.has_effect("io"));
        assert_eq!(row3.extension, None);
    }
    
    #[test]
    fn test_effect_operations() {
        let mut env = TypeEnvironment::new();
        
        // Create a function type with no effects
        let func_type = TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String)),
        );
        
        // Add an effect
        let with_state = operations::add_effect(
            &func_type,
            "state",
            &TelType::Base(BaseType::String),
            &env,
        ).unwrap();
        
        // Remove the effect (no effect stored in current implementation)
        let without_state = operations::remove_effect(
            &with_state,
            "state",
            &env,
        ).unwrap();
    }
    
    #[test]
    fn test_handler_operations() {
        let mut env = TypeEnvironment::new();
        
        // Create a function type
        let func_type = TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String)),
        );
        
        // Create a handler for the state effect
        let handler_type = TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String)),
        );
        
        // Handle the state effect
        let handled = operations::handle_effect(
            &handler_type,
            "state",
            &func_type,
            &env,
        ).unwrap();
        
        // Test compose handlers
        let composed = operations::compose_handlers(
            &handler_type,
            &handler_type,
            &env,
        ).unwrap();
    }
} 