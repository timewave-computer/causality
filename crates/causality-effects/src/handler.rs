// Effect handler framework
// Original file: src/effect/handler.rs

// Effect Handler Module
//
// Provides the core interface for handling effects, allowing the effect system to 
// execute effects through different execution environments.

use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;

use causality_types::Result;
use crate::effect::{Effect, EffectContext, EffectOutcome, EffectResult};
use causality_effects::ExecutionBoundary;

/// The outcome of handling an effect
pub type HandlerResult<T> = Result<T>;

/// Core trait for effect handlers that process and execute effects
#[async_trait]
pub trait EffectHandler: Send + Sync {
    /// Get the execution boundary this handler operates in
    fn execution_boundary(&self) -> ExecutionBoundary;
    
    /// Handle an effect synchronously
    fn handle(&self, effect: &dyn Effect, context: &EffectContext) -> Result<EffectOutcome> {
        // Default implementation defers to the effect's own execution
        effect.execute(context)
    }
    
    /// Handle an effect asynchronously
    async fn handle_async(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Default implementation defers to the effect's own execution
        effect.execute_async(context).await
    }
    
    /// Check if this handler can handle a specific effect
    fn can_handle(&self, effect: &dyn Effect) -> bool {
        effect.can_execute_in(self.execution_boundary())
    }
}

/// A handler that delegates to other handlers based on criteria
pub struct CompositeHandler {
    /// The primary execution boundary for this handler
    boundary: ExecutionBoundary,
    /// The handlers that this composite delegates to
    handlers: Vec<Arc<dyn EffectHandler>>,
}

impl CompositeHandler {
    /// Create a new composite handler with the given boundary
    pub fn new(boundary: ExecutionBoundary) -> Self {
        Self {
            boundary,
            handlers: Vec::new(),
        }
    }
    
    /// Add a handler to this composite
    pub fn add_handler(&mut self, handler: Arc<dyn EffectHandler>) {
        self.handlers.push(handler);
    }
    
    /// Find a handler that can handle the given effect
    fn find_handler(&self, effect: &dyn Effect) -> Option<Arc<dyn EffectHandler>> {
        self.handlers.iter()
            .find(|h| h.can_handle(effect))
            .cloned()
    }
}

#[async_trait]
impl EffectHandler for CompositeHandler {
    fn execution_boundary(&self) -> ExecutionBoundary {
        self.boundary
    }
    
    fn handle(&self, effect: &dyn Effect, context: &EffectContext) -> Result<EffectOutcome> {
        if let Some(handler) = self.find_handler(effect) {
            handler.handle(effect, context)
        } else {
            // Default to the effect's own execution if no handler is found
            effect.execute(context)
        }
    }
    
    async fn handle_async(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        if let Some(handler) = self.find_handler(effect) {
            handler.handle_async(effect, context).await
        } else {
            // Default to the effect's own execution if no handler is found
            effect.execute_async(context).await
        }
    }
}

/// A simple handler for inside-system effects
pub struct InsideSystemHandler;

impl InsideSystemHandler {
    /// Create a new inside-system handler
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EffectHandler for InsideSystemHandler {
    fn execution_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::InsideSystem
    }
}

/// A simple handler for outside-system effects
pub struct OutsideSystemHandler;

impl OutsideSystemHandler {
    /// Create a new outside-system handler
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EffectHandler for OutsideSystemHandler {
    fn execution_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
}

/// Factory for creating effect handlers
pub struct HandlerFactory;

impl HandlerFactory {
    /// Create a handler for the given execution boundary
    pub fn create(boundary: ExecutionBoundary) -> Box<dyn EffectHandler> {
        match boundary {
            ExecutionBoundary::InsideSystem => Box::new(InsideSystemHandler::new()),
            ExecutionBoundary::OutsideSystem => Box::new(OutsideSystemHandler::new()),
        }
    }
    
    /// Create a composite handler for multiple execution boundaries
    pub fn create_composite(primary_boundary: ExecutionBoundary, 
                           additional_handlers: Vec<Arc<dyn EffectHandler>>) -> CompositeHandler {
        let mut handler = CompositeHandler::new(primary_boundary);
        
        for h in additional_handlers {
            handler.add_handler(h);
        }
        
        handler
    }
} 