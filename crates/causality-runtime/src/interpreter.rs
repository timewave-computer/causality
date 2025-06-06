//! Effect interpreter with support for pure effects and handlers

use causality_core::{
    effect::{EffectExpr, EffectKind, effect_constructors::Intent},
    lambda::{base::Value, Symbol},
    machine::{MachineValue, MachineState, RegisterId},
    system::content_addressing::Timestamp,
};
// use crate::context::RuntimeContext;  // TODO: Implement context module
// use crate::handler::{HandlerRegistry, default_handlers};  // TODO: Implement handler module
use crate::error::{RuntimeError, RuntimeResult};
use std::collections::HashMap;

/// Result of an interpreter operation
pub type InterpreterResult<T> = RuntimeResult<T>;

/// Execution context for the interpreter
pub struct ExecutionContext {
    /// Runtime context
    pub runtime: RuntimeContext,
    
    /// Effect handlers
    pub handlers: HandlerRegistry,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(runtime: RuntimeContext) -> Self {
        Self {
            runtime,
            handlers: default_handlers(),
        }
    }
    
    /// Create execution context with custom handlers
    pub fn with_handlers(runtime: RuntimeContext, handlers: HandlerRegistry) -> Self {
        Self {
            runtime,
            handlers,
        }
    }
}

/// The main effect interpreter
pub struct Interpreter {
    context: ExecutionContext,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new(runtime: RuntimeContext) -> Self {
        Self {
            context: ExecutionContext::new(runtime),
        }
    }
    
    /// Create interpreter with custom handlers
    pub fn with_handlers(runtime: RuntimeContext, handlers: HandlerRegistry) -> Self {
        Self {
            context: ExecutionContext::with_handlers(runtime, handlers),
        }
    }
    
    /// Execute an effect and return the result
    pub fn execute<T>(&mut self, effect: EffectExpr) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        self.context.runtime.enter_effect()?;
        self.context.runtime.consume_gas(1)?; // Basic gas cost
        
        let result = match effect.kind {
            EffectExprKind::Pure(term) => {
                // Extract value from term and execute as pure
                if let Some(value) = self.extract_value_from_term(term) {
                    self.execute_pure_value(value)
                } else {
                    Err(RuntimeError::execution_failed("Cannot extract value from term"))
                }
            }
            EffectExprKind::Bind { effect, var: _, body } => {
                // Execute inner effect, then apply continuation
                self.execute_bind(*effect, *body)
            }
            EffectExprKind::Perform { effect_tag, args: _ } => {
                // Execute the actual effect
                self.execute_perform(effect_tag)
            }
            EffectExprKind::Handle { expr, handlers: _ } => {
                // Apply handler to effect
                self.execute_handle(*expr)
            }
            EffectExprKind::Parallel { left, right } => {
                // Execute effects in parallel (simplified to sequential for now)
                self.execute_parallel(vec![*left, *right])
            }
            EffectExprKind::Race { left, right } => {
                // Execute effects racing (simplified to first for now)
                self.execute_race(vec![*left, *right])
            }
        };
        
        self.context.runtime.exit_effect();
        result
    }
    
    /// Extract a value from a term (simplified implementation)
    fn extract_value_from_term(&self, term: causality_core::lambda::Term) -> Option<Value> {
        // This is a simplified implementation - in practice would need
        // full term evaluation
        match term.kind {
            causality_core::lambda::TermKind::Literal(lit) => {
                match lit {
                    causality_core::lambda::Literal::Int(i) => Some(Value::Int(i)),
                    causality_core::lambda::Literal::Bool(b) => Some(Value::Bool(b)),
                    causality_core::lambda::Literal::Symbol(s) => {
                        // Handle the Option<&str> return from name()
                        if let Some(name) = s.name() {
                            Some(Value::Symbol(causality_core::system::Str::new(name)))
                        } else {
                            // Fallback to hex representation if no name available
                            Some(Value::Symbol(causality_core::system::Str::new(&s.to_hex())))
                        }
                    }
                }
            }
            causality_core::lambda::TermKind::Unit => Some(Value::Unit),
            _ => None, // For now, only support literals and unit
        }
    }
    
    /// Execute a pure value
    fn execute_pure_value<T>(&mut self, value: Value) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        // Convert lambda value to desired type
        match value {
            Value::Int(i) => {
                // Try to convert to target type
                if let Ok(result) = serde_json::from_value(serde_json::Value::Number(i.into())) {
                    Ok(result)
                } else {
                    Err(RuntimeError::type_mismatch("Expected Int type"))
                }
            }
            Value::Bool(b) => {
                if let Ok(result) = serde_json::from_value(serde_json::Value::Bool(b)) {
                    Ok(result)
                } else {
                    Err(RuntimeError::type_mismatch("Expected Bool type"))
                }
            }
            Value::Unit => {
                if let Ok(result) = serde_json::from_value(serde_json::Value::Null) {
                    Ok(result)
                } else {
                    Err(RuntimeError::type_mismatch("Expected Unit type"))
                }
            }
            Value::Symbol(s) => {
                if let Ok(result) = serde_json::from_value(serde_json::Value::String(s.as_str().to_string())) {
                    Ok(result)
                } else {
                    Err(RuntimeError::type_mismatch("Expected Symbol type"))
                }
            }
            _ => Err(RuntimeError::execution_failed("Unsupported value type for pure effect")),
        }
    }
    
    /// Execute a bind operation (effect sequencing)
    fn execute_bind<T>(&mut self, inner: EffectExpr, continuation: EffectExpr) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        // Execute the inner effect first (we ignore the result for simplicity)
        let _inner_result: String = self.execute(inner)?;
        
        // Apply the continuation (simplified - would need proper lambda application)
        self.execute(continuation)
    }
    
    /// Execute a perform operation (actual side effect)
    fn execute_perform<T>(&mut self, effect_tag: String) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        self.context.runtime.consume_gas(10)?; // Higher cost for side effects
        
        match effect_tag.as_str() {
            "witness" => {
                // Default witness value for testing
                self.execute_pure_value(Value::Int(42))
            }
            _ => {
                Err(RuntimeError::unhandled_effect(format!("Custom effect: {}", effect_tag)))
            }
        }
    }
    
    /// Execute a handle operation (apply handler)
    fn execute_handle<T>(&mut self, effect: EffectExpr) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        // Apply handler transformation
        let transformed_effect = self.context.handlers.handle_effect(effect)?;
        
        // Execute the transformed effect
        self.execute(transformed_effect)
    }
    
    /// Execute parallel effects (simplified to sequential)
    fn execute_parallel<T>(&mut self, effects: Vec<EffectExpr>) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        let mut results = Vec::new();
        
        for effect in effects {
            let result: String = self.execute(effect)?;
            results.push(result);
        }
        
        // For now, just return the last result as JSON
        if let Some(last_result) = results.into_iter().last() {
            if let Ok(result) = serde_json::from_str(&last_result) {
                Ok(result)
            } else {
                Err(RuntimeError::execution_failed("Failed to parse result"))
            }
        } else {
            Err(RuntimeError::execution_failed("No effects in parallel"))
        }
    }
    
    /// Execute racing effects (simplified to first)
    fn execute_race<T>(&mut self, effects: Vec<EffectExpr>) -> InterpreterResult<T>
    where
        T: serde::de::DeserializeOwned + Clone,
    {
        // For now, just execute the first effect
        if let Some(first_effect) = effects.into_iter().next() {
            self.execute(first_effect)
        } else {
            Err(RuntimeError::execution_failed("No effects in race"))
        }
    }
    
    /// Execute a machine instruction (simplified without ReductionEngine)
    pub fn execute_instruction(&mut self, instruction: Instruction) -> InterpreterResult<()> {
        self.context.runtime.consume_gas(1)?;
        
        // For now, just simulate basic instruction execution
        // In practice, would integrate with a ReductionEngine
        match instruction {
            Instruction::Move { src, dst } => {
                if let Ok(value) = self.context.runtime.machine_state.load_register(src) {
                    if !value.consumed {
                        self.context.runtime.machine_state.store_register(
                            dst, 
                            value.value.clone(), 
                            value.value_type.clone()
                        );
                        let _ = self.context.runtime.machine_state.consume_register(src);
                    }
                }
                Ok(())
            }
            _ => {
                // Other instructions would be implemented here
                Ok(())
            }
        }
    }
    
    /// Get the current runtime context
    pub fn context(&self) -> &RuntimeContext {
        &self.context.runtime
    }
    
    /// Get a mutable reference to the runtime context
    pub fn context_mut(&mut self) -> &mut RuntimeContext {
        &mut self.context.runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::effect::core::{EffectExpr, EffectExprKind};
    use causality_core::lambda::{Term, TermKind, Literal};
    
    #[test]
    fn test_interpreter_pure_effect() {
        let mut interpreter = Interpreter::new(RuntimeContext::new());
        
        // Create a pure effect with literal 42
        let term = Term::new(TermKind::Literal(Literal::Int(42)));
        let effect = EffectExpr::new(EffectExprKind::Pure(term));
        
        let result: InterpreterResult<i64> = interpreter.execute(effect);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_interpreter_gas_consumption() {
        let mut interpreter = Interpreter::new(RuntimeContext::new());
        let initial_gas = interpreter.context().metadata.gas_remaining;
        
        let term = Term::new(TermKind::Literal(Literal::Int(42)));
        let effect = EffectExpr::new(EffectExprKind::Pure(term));
        
        let _result: InterpreterResult<i64> = interpreter.execute(effect);
        
        // Should have consumed some gas
        assert!(interpreter.context().metadata.gas_remaining < initial_gas);
    }
    
    #[test]
    fn test_interpreter_depth_tracking() {
        let mut interpreter = Interpreter::new(RuntimeContext::new());
        
        let term = Term::new(TermKind::Literal(Literal::Int(42)));
        let effect = EffectExpr::new(EffectExprKind::Pure(term));
        
        assert_eq!(interpreter.context().metadata.depth, 0);
        let _result: InterpreterResult<i64> = interpreter.execute(effect);
        assert_eq!(interpreter.context().metadata.depth, 0); // Should return to 0
        assert_eq!(interpreter.context().metadata.effects_executed, 1);
    }
    
    #[test]
    fn test_interpreter_unhandled_effect() {
        let mut interpreter = Interpreter::new(RuntimeContext::new());
        
        let effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "unknown".to_string(),
            args: vec![],
        });
        
        let result: InterpreterResult<i64> = interpreter.execute(effect);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::UnhandledEffect { .. }));
    }
} 