// Effect interpreter for Causality
//
// This module provides the infrastructure for interpreting Causality effects
// by executing them in a Rust environment.

use std::sync::Arc;
use std::time::SystemTime;
use std::collections::HashMap;

use crate::effect::{Effect, AsyncEffect, EffectContext, EffectOutcome, EffectResult, EffectError};
use crate::error::{Error, Result};
use crate::effect::handler::EffectHandler;
use crate::effect::boundary::ExecutionBoundary;

/// An interpreter for executing Causality effects
///
/// The Interpreter is responsible for executing effects by resolving them
/// with appropriate handlers and managing continuations.
pub struct Interpreter {
    /// The handler to use for executing effects
    handler: Box<dyn EffectHandler>,
}

impl Interpreter {
    /// Create a new interpreter with the given handler
    pub fn new(handler: impl EffectHandler + 'static) -> Self {
        Interpreter {
            handler: Box::new(handler),
        }
    }
    
    /// Execute an effect synchronously and return its outcome
    pub fn execute(&self, effect: &dyn Effect) -> Result<EffectOutcome> {
        // First check if this effect can be executed in the current boundary
        if !effect.can_execute_in(self.handler.execution_boundary()) {
            return Err(Error::BoundaryViolation);
        }
        
        // Call the handler to execute the effect
        effect.execute(&EffectContext::new())
    }
    
    /// Execute an effect asynchronously and return its outcome
    pub async fn execute_async(&self, effect: &dyn AsyncEffect) -> EffectResult<EffectOutcome> {
        // First check if this effect can execute in the current boundary
        if !effect.can_execute_in(self.handler.execution_boundary()) {
            return Err(EffectError::UnsupportedOperation(format!("Effect cannot be executed in boundary {:?}", self.handler.execution_boundary())));
        }
        
        // Call the handler to execute the effect
        effect.execute_async(&EffectContext::new()).await
    }
}

/// An interpreter that keeps a trace of executed effects
pub struct TracingInterpreter {
    /// The inner handler
    handler: Box<dyn EffectHandler>,
    /// The trace of executed effects
    trace: Vec<TraceEntry>,
}

/// An entry in the trace of executed effects
pub struct TraceEntry {
    /// The type of effect that was executed
    effect_type: String,
    /// The timestamp when the effect was executed
    timestamp: SystemTime,
}

impl TraceEntry {
    /// Create a new trace entry
    fn new(effect: &dyn Effect) -> Self {
        TraceEntry {
            effect_type: effect.display_name(),
            timestamp: SystemTime::now(),
        }
    }
    
    /// Get the type of effect that was executed
    pub fn effect_type(&self) -> &str {
        &self.effect_type
    }
    
    /// Get the timestamp when the effect was executed
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl TracingInterpreter {
    /// Create a new tracing interpreter
    pub fn new(handler: impl EffectHandler + 'static) -> Self {
        TracingInterpreter {
            handler: Box::new(handler),
            trace: Vec::new(),
        }
    }
    
    /// Execute an effect and add it to the trace
    pub fn execute(&mut self, effect: &dyn Effect) -> Result<EffectOutcome> {
        // Add to trace
        self.trace.push(TraceEntry::new(effect));
        
        // First check if this effect can be executed in the current boundary
        if !effect.can_execute_in(self.handler.execution_boundary()) {
            return Err(Error::BoundaryViolation);
        }
        
        // Call the handler to execute the effect
        effect.execute(&EffectContext::new())
    }
    
    /// Execute an effect asynchronously and add it to the trace
    pub async fn execute_async(&mut self, effect: &dyn AsyncEffect) -> EffectResult<EffectOutcome> {
        // Add to trace
        self.trace.push(TraceEntry::new(effect));
        
        // First check if this effect can be executed in the current boundary
        if !effect.can_execute_in(self.handler.execution_boundary()) {
            return Err(EffectError::UnsupportedOperation(format!("Effect cannot be executed in boundary {:?}", self.handler.execution_boundary())));
        }
        
        // Call the handler to execute the effect
        effect.execute_async(&EffectContext::new()).await
    }
    
    /// Get the trace of executed effects
    pub fn trace(&self) -> &[TraceEntry] {
        &self.trace
    }
    
    /// Clear the trace
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }
} 