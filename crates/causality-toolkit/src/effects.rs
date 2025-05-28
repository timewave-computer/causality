//! Common Reusable Effects for the Causality Toolkit
//!
//! This module defines standard effects that can be used across Causality applications.
//! These effects provide common functionality like logging, system interactions, and
//! utility operations that are useful in most applications.

// use anyhow::Result as AnyhowResult; // aliasing to avoid conflict with core::result::Result
use async_trait::async_trait;
// use causality_effects_macros::register_handler;
use causality_types::{
    primitive::string::Str,
    expression::{value::ValueExpr, r#type::{TypeExpr, TypeExprBox, TypeExprMap}},
    effect::{
        core::{EffectInput, EffectOutput, Effect, EffectHandler, ConversionError, HandlerError},
    },
};
use causality_core::utils::expr::{value_expr_as_string, value_expr_as_map};
use log;
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// LogMessage Effect
//-----------------------------------------------------------------------------

/// Input for the LogMessage effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMessageEffectInput {
    /// The log level of the message (e.g., "info", "warn", "error", "debug", "trace")
    pub level: String,
    /// The message to be logged
    pub message: String,
    /// Optional additional context for the log entry
    pub context: Option<String>
}

impl EffectInput for LogMessageEffectInput {
    fn from_value_expr(value: ValueExpr) -> Result<Self, ConversionError> {
        // Extract parameters from a record or map structure
        if let Some(map) = value_expr_as_map(&value) {
            let level = map.get(&Str::from("level"))
                .and_then(|v| value_expr_as_string(v))
                .map(|s| s.as_str().to_string())
                .ok_or_else(|| ConversionError::MissingField { 
                    field_name: "level".to_string() 
                })?;
                
            let message = map.get(&Str::from("message"))
                .and_then(|v| value_expr_as_string(v))
                .map(|s| s.as_str().to_string())
                .ok_or_else(|| ConversionError::MissingField { 
                    field_name: "message".to_string() 
                })?;
                
            let context = map.get(&Str::from("context")).and_then(|v| value_expr_as_string(v)).map(|s| s.as_str().to_string());
            
            Ok(LogMessageEffectInput {
                level,
                message,
                context,
            })
        } else {
            Err(ConversionError::TypeMismatch { 
                expected: "Map or Record".to_string(),
                found: "Other".to_string() 
            })
        }
    }

    fn schema() -> TypeExpr {
        // Create a record with our fields
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("level"), TypeExpr::String);
        fields.insert(Str::from("message"), TypeExpr::String);
        fields.insert(
            Str::from("context"),
            TypeExpr::Optional(TypeExprBox(Box::new(TypeExpr::String)))
        );
        TypeExpr::Record(TypeExprMap(fields))
    }
}

//-----------------------------------------------------------------------------
// LogMessage Effect Output
//-----------------------------------------------------------------------------

/// Output for the LogMessage effect (empty on success).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMessageEffectOutput;

impl EffectOutput for LogMessageEffectOutput {
    fn to_value_expr(&self) -> Result<ValueExpr, ConversionError> {
        Ok(ValueExpr::Nil)
    }

    fn schema() -> TypeExpr {
        TypeExpr::Unit
    }
}

//-----------------------------------------------------------------------------
// LogMessage Effect Implementation
//-----------------------------------------------------------------------------

/// The LogMessage effect itself.
#[derive(Debug)]
pub struct LogMessageEffect;

impl Effect for LogMessageEffect {
    type Input = LogMessageEffectInput;
    type Output = LogMessageEffectOutput;
    const EFFECT_TYPE: &'static str = "causality.toolkit.effects.LogMessage";
}

//-----------------------------------------------------------------------------
// LogMessage Effect Handler
//-----------------------------------------------------------------------------

/// Handler for the LogMessage effect.
#[derive(Debug, Default)]
pub struct LogMessageHandler;

#[async_trait]
// #[register_handler(effect_type = "causality.toolkit.effects.LogMessage")]
impl EffectHandler for LogMessageHandler {
    type E = LogMessageEffect;

    async fn handle(
        &self,
        input: LogMessageEffectInput,
    ) -> Result<LogMessageEffectOutput, HandlerError> {
        let log_context = input.context.as_deref().unwrap_or("DefaultContext");
        match input.level.to_lowercase().as_str() {
            "error" => log::error!(target: log_context, "{}", input.message),
            "warn" => log::warn!(target: log_context, "{}", input.message),
            "debug" => log::debug!(target: log_context, "{}", input.message),
            "trace" => log::trace!(target: log_context, "{}", input.message),
            "info" => log::info!(target: log_context, "{}", input.message),
            _ => log::info!(target: log_context, "{}", input.message), // Default to info
        }
        Ok(LogMessageEffectOutput)
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    // Comment out inventory test since we're not using the register_handler macro
    /*
    use causality_types::effects_core::inventory;

    // Basic test to ensure the handler is registered via inventory
    #[test]
    fn test_log_message_handler_registered() {
        let mut found = false;
        for item in
            inventory::iter::<causality_types::effects_core::HandlerRegistrationInfo>
        {
            if item.effect_type_str == LogMessageEffect::EFFECT_TYPE {
                if item.handler_struct_path.contains("LogMessageHandler") {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "LogMessageHandler should be registered with inventory for effect type {}",
            LogMessageEffect::EFFECT_TYPE
        );
    }
    */

    #[tokio::test]
    async fn test_log_message_handler_execution() {
        let handler = LogMessageHandler;
        let input = LogMessageEffectInput {
            level: "info".to_string(),
            message: "Test log from toolkit".to_string(),
            context: Some("ToolkitTest".to_string()),
        };

        // This test mainly checks if it runs without panic and returns Ok.
        // Actual log output verification is harder in unit tests without specific setup.
        let result = handler.handle(input).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_log_message_input_schema() {
        let schema = LogMessageEffectInput::schema();
        if let TypeExpr::Record(fields_map) = schema {
            // fields_map is a TypeExprMap which is a wrapper around BTreeMap<Str, TypeExpr>
            
            // Check if the expected keys exist
            assert!(fields_map.contains_key(&Str::from("level")));
            assert!(fields_map.contains_key(&Str::from("message")));
            assert!(fields_map.contains_key(&Str::from("context")));
            
            // Check the types - verify they are string
            if let TypeExpr::String = fields_map.get(&Str::from("level")).unwrap() {
                // Type is correct
            } else {
                panic!("level field should be String type");
            }
            
            if let TypeExpr::String = fields_map.get(&Str::from("message")).unwrap() {
                // Type is correct
            } else {
                panic!("message field should be String type");
            }
            
            // Optional types might need special handling
            if let Some(context_type) = fields_map.get(&Str::from("context")) {
                if let TypeExpr::Optional(inner_type_box) = context_type {
                    // Check if the inner type is String
                    if let TypeExpr::String = &***inner_type_box {
                        // Type is correct
                    } else {
                        panic!("context inner type should be String");
                    }
                } else {
                    panic!("context field should be Optional type");
                }
            } else {
                panic!("context field not found in schema");
            }
        } else {
            panic!("Expected TypeExpr::Record for LogMessageEffectInput schema");
        }
    }

    #[test]
    fn test_log_message_output_schema() {
        let schema = LogMessageEffectOutput::schema();
        assert_eq!(schema, TypeExpr::Unit);
    }
}
