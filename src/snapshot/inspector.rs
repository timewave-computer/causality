// State inspection for the Causality Content-Addressed Code System
//
// This module provides functionality for inspecting the state of an execution
// context at a specific point in time, enabling detailed debugging.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::execution::context::{ExecutionContext, Value, ContextId};
use crate::execution::trace::ExecutionTracer;
use crate::snapshot::navigator::{TimeTravel, DebugError};

/// Represents variable state at a point in time
#[derive(Debug, Clone)]
pub struct VariableState {
    /// Name of the variable
    pub name: String,
    /// Current value of the variable
    pub value: Value,
    /// Type of the variable (if available)
    pub var_type: Option<String>,
    /// Scope level where the variable is defined
    pub scope_level: Option<usize>,
}

/// Interface for state inspection during time-travel debugging
pub trait StateInspector: Send + Sync {
    /// Get all variable bindings at the current position
    fn get_all_variables(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<Vec<VariableState>, DebugError>;
    
    /// Get a specific variable at the current position
    fn get_variable(
        &self,
        context: &ExecutionContext,
        name: &str,
    ) -> std::result::Result<Option<VariableState>, DebugError>;
    
    /// Get the call stack at the current position
    fn get_call_stack(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<Vec<String>, DebugError>;
    
    /// Get the current execution position information
    fn get_position_info(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<PositionInfo, DebugError>;
    
    /// Search for variables matching a pattern
    fn search_variables(
        &self,
        context: &ExecutionContext,
        pattern: &str,
    ) -> std::result::Result<Vec<VariableState>, DebugError>;
}

/// Information about the current execution position
#[derive(Debug, Clone)]
pub struct PositionInfo {
    /// Absolute position in the execution trace
    pub position: usize,
    /// Current function or code block being executed
    pub function_name: Option<String>,
    /// Source code location (if available)
    pub source_location: Option<String>,
    /// Event type at this position
    pub event_type: String,
    /// Timestamp of this event
    pub timestamp: Option<u64>,
}

/// Implementation of the StateInspector trait
pub struct ContextStateInspector {
    /// Reference to the TimeTravel implementation
    time_travel: Arc<dyn TimeTravel>,
    /// Execution tracer for accessing trace data
    tracer: Arc<ExecutionTracer>,
}

impl ContextStateInspector {
    /// Create a new context state inspector
    pub fn new(time_travel: Arc<dyn TimeTravel>, tracer: Arc<ExecutionTracer>) -> Self {
        ContextStateInspector {
            time_travel,
            tracer,
        }
    }
    
    /// Extract type information from a value if possible
    fn extract_type_info(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(_) => Some("String".to_string()),
            Value::Number(_) => Some("Number".to_string()),
            Value::Boolean(_) => Some("Boolean".to_string()),
            Value::Array(_) => Some("Array".to_string()),
            Value::Object(_) => Some("Object".to_string()),
            Value::Null => Some("Null".to_string()),
            Value::Function(_, _) => Some("Function".to_string()),
            _ => None,
        }
    }
    
    /// Parse variable scopes from context
    fn parse_variable_scopes(&self, context: &ExecutionContext) -> HashMap<String, usize> {
        let mut scopes = HashMap::new();
        // In a real implementation, we would analyze the context's scope chain
        // For now, we'll return an empty map
        scopes
    }
}

impl StateInspector for ContextStateInspector {
    fn get_all_variables(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<Vec<VariableState>, DebugError> {
        // Get variable bindings from the TimeTravel implementation
        let bindings = self.time_travel.inspect_state(context)?;
        
        // Get scope information
        let scopes = self.parse_variable_scopes(context);
        
        // Convert bindings to VariableState objects
        let mut variables = Vec::new();
        for (name, value) in bindings {
            let var_type = self.extract_type_info(&value);
            let scope_level = scopes.get(&name).cloned();
            
            variables.push(VariableState {
                name,
                value,
                var_type,
                scope_level,
            });
        }
        
        // Sort variables by name for consistency
        variables.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(variables)
    }
    
    fn get_variable(
        &self,
        context: &ExecutionContext,
        name: &str,
    ) -> std::result::Result<Option<VariableState>, DebugError> {
        // Get all variables and find the requested one
        let all_vars = self.get_all_variables(context)?;
        let var = all_vars.into_iter().find(|v| v.name == name);
        
        Ok(var)
    }
    
    fn get_call_stack(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<Vec<String>, DebugError> {
        // In a real implementation, we would extract the call stack from the context
        // For now, we'll try to get it from the context if available, or return a placeholder
        let stack = context.call_stack()
            .map_err(|_| DebugError::InspectionError("Failed to get call stack".to_string()))?;
        
        Ok(stack)
    }
    
    fn get_position_info(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<PositionInfo, DebugError> {
        // Get the current position
        let current_trace = context.execution_trace()
            .map_err(|e| DebugError::InspectionError(format!("Failed to get execution trace: {:?}", e)))?;
        
        let position = context.execution_position()
            .map_err(|e| DebugError::InspectionError(format!("Failed to get execution position: {:?}", e)))?;
        
        if position >= current_trace.len() {
            return Err(DebugError::InvalidPosition(format!(
                "Position {} is beyond trace length {}", position, current_trace.len()
            )));
        }
        
        let event = &current_trace[position];
        
        // Extract event information
        let event_type = match event {
            crate::execution::context::ExecutionEvent::FunctionCall { .. } => "FunctionCall",
            crate::execution::context::ExecutionEvent::FunctionReturn { .. } => "FunctionReturn",
            crate::execution::context::ExecutionEvent::VariableAssignment { .. } => "VariableAssignment",
            crate::execution::context::ExecutionEvent::EffectApplied { .. } => "EffectApplied",
            crate::execution::context::ExecutionEvent::ErrorThrown { .. } => "ErrorThrown",
            _ => "Unknown",
        }.to_string();
        
        // Extract function name and source location if available
        let (function_name, source_location) = match event {
            crate::execution::context::ExecutionEvent::FunctionCall { function_name, source_location, .. } => {
                (Some(function_name.clone()), source_location.clone())
            },
            crate::execution::context::ExecutionEvent::FunctionReturn { function_name, .. } => {
                (Some(function_name.clone()), None)
            },
            _ => (None, None),
        };
        
        // Get timestamp if available
        let timestamp = event.timestamp();
        
        Ok(PositionInfo {
            position,
            function_name,
            source_location,
            event_type,
            timestamp,
        })
    }
    
    fn search_variables(
        &self,
        context: &ExecutionContext,
        pattern: &str,
    ) -> std::result::Result<Vec<VariableState>, DebugError> {
        // Get all variables
        let all_vars = self.get_all_variables(context)?;
        
        // Filter variables by pattern (case-insensitive substring match)
        let pattern = pattern.to_lowercase();
        let matching_vars = all_vars.into_iter()
            .filter(|v| v.name.to_lowercase().contains(&pattern))
            .collect();
        
        Ok(matching_vars)
    }
}

// Add tests for the StateInspector implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    // This would be a real test in the actual implementation
    // For now, we'll just have a placeholder
    #[test]
    fn test_get_all_variables() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_get_variable() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_get_call_stack() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_get_position_info() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_search_variables() {
        // Test implementation would go here
    }
} 