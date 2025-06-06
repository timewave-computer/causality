//! Pure effect handlers - stateless transformations

use causality_core::effect::core::EffectExpr;
use crate::error::{RuntimeError, RuntimeResult};

/// Result of a handler operation
pub type HandlerResult<T> = RuntimeResult<T>;

/// A pure effect handler trait
pub trait Handler: Send + Sync {
    /// Handle an effect expression and return a transformed effect
    fn handle(&self, effect: EffectExpr) -> HandlerResult<EffectExpr>;
    
    /// Check if this handler can handle the given effect tag
    fn can_handle(&self, effect_tag: &str) -> bool;
    
    /// Check if this handler is pure (no side effects)
    fn is_pure(&self) -> bool;
    
    /// Get the handler's name for debugging
    fn name(&self) -> &str;
}

/// A concrete pure handler implementation
#[derive(Clone)]
pub struct PureHandler {
    name: String,
    transform_fn: fn(EffectExpr) -> HandlerResult<EffectExpr>,
}

impl PureHandler {
    /// Create a new pure handler
    pub fn new(
        name: impl Into<String>,
        transform_fn: fn(EffectExpr) -> HandlerResult<EffectExpr>,
    ) -> Self {
        Self {
            name: name.into(),
            transform_fn,
        }
    }
    
    /// Create an identity handler (no transformation)
    pub fn identity() -> Self {
        Self::new("identity", |effect| Ok(effect))
    }
    
    /// Create a logging handler (for debugging)
    pub fn logging(name: impl Into<String>) -> Self {
        Self::new(name, |effect| {
            log::debug!("Handler processing effect: {:?}", effect);
            Ok(effect)
        })
    }
    
    /// Compose two handlers (left-to-right composition)
    pub fn compose(self, other: PureHandler) -> ComposedHandler {
        ComposedHandler {
            name: format!("{} -> {}", self.name, other.name),
            handlers: vec![self, other],
        }
    }
}

impl Handler for PureHandler {
    fn handle(&self, effect: EffectExpr) -> HandlerResult<EffectExpr> {
        (self.transform_fn)(effect)
    }
    
    fn can_handle(&self, _effect_tag: &str) -> bool {
        true // Pure handlers can handle any effect
    }
    
    fn is_pure(&self) -> bool {
        true
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// A composed handler that chains multiple handlers
#[derive(Clone)]
pub struct ComposedHandler {
    name: String,
    handlers: Vec<PureHandler>,
}

impl ComposedHandler {
    /// Create a new composed handler
    pub fn new(name: impl Into<String>, handlers: Vec<PureHandler>) -> Self {
        Self {
            name: name.into(),
            handlers,
        }
    }
    
    /// Add another handler to the composition
    pub fn then(mut self, handler: PureHandler) -> Self {
        let handler_name = handler.name.clone();
        self.handlers.push(handler);
        self.name = format!("{} -> {}", self.name, handler_name);
        self
    }
}

impl Handler for ComposedHandler {
    fn handle(&self, mut effect: EffectExpr) -> HandlerResult<EffectExpr> {
        for handler in &self.handlers {
            effect = handler.handle(effect)?;
        }
        Ok(effect)
    }
    
    fn can_handle(&self, effect_tag: &str) -> bool {
        self.handlers.iter().all(|h| h.can_handle(effect_tag))
    }
    
    fn is_pure(&self) -> bool {
        self.handlers.iter().all(|h| h.is_pure())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Handler registry for managing effect handlers
#[derive(Default)]
pub struct HandlerRegistry {
    handlers: Vec<Box<dyn Handler>>,
}

impl HandlerRegistry {
    /// Create a new empty handler registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// Register a handler
    pub fn register<H: Handler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }
    
    /// Find the first handler that can handle the given effect tag
    pub fn find_handler(&self, effect_tag: &str) -> Option<&dyn Handler> {
        self.handlers
            .iter()
            .find(|h| h.can_handle(effect_tag))
            .map(|h| h.as_ref())
    }
    
    /// Handle an effect using the registered handlers
    pub fn handle_effect(&self, effect: EffectExpr) -> HandlerResult<EffectExpr> {
        // Extract effect tag for matching
        let effect_tag = match &effect.kind {
            causality_core::effect::core::EffectExprKind::Perform { effect_tag, .. } => effect_tag.clone(),
            _ => "pure".to_string(), // Default for non-perform effects
        };
        
        match self.find_handler(&effect_tag) {
            Some(handler) => handler.handle(effect),
            None => Err(RuntimeError::unhandled_effect(effect_tag)),
        }
    }
    
    /// Get all pure handlers
    pub fn pure_handlers(&self) -> Vec<&dyn Handler> {
        self.handlers
            .iter()
            .filter(|h| h.is_pure())
            .map(|h| h.as_ref())
            .collect()
    }
}

impl std::fmt::Debug for HandlerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerRegistry")
            .field("handler_count", &self.handlers.len())
            .field("handler_names", &self.handlers.iter().map(|h| h.name()).collect::<Vec<_>>())
            .finish()
    }
}

/// Create default handlers for core effects
pub fn default_handlers() -> HandlerRegistry {
    let mut registry = HandlerRegistry::new();
    
    // Register identity handler as fallback
    registry.register(PureHandler::identity());
    
    // Register logging handler for debugging
    registry.register(PureHandler::logging("debug"));
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::effect::core::{EffectExpr, EffectExprKind};
    use causality_core::lambda::{Term, TermKind, Literal};
    
    #[test]
    fn test_pure_handler() {
        let handler = PureHandler::identity();
        assert!(handler.is_pure());
        assert_eq!(handler.name(), "identity");
        
        let term = Term::new(TermKind::Literal(Literal::Int(42)));
        let effect = EffectExpr::new(EffectExprKind::Pure(term));
        
        let result = handler.handle(effect.clone()).unwrap();
        assert_eq!(result, effect);
    }
    
    #[test]
    fn test_handler_composition() {
        let handler1 = PureHandler::identity();
        let handler2 = PureHandler::logging("test");
        
        let composed = handler1.compose(handler2);
        assert!(composed.is_pure());
        assert!(composed.name().contains("identity"));
        assert!(composed.name().contains("test"));
    }
    
    #[test]
    fn test_handler_registry() {
        let mut registry = HandlerRegistry::new();
        registry.register(PureHandler::identity());
        
        let term = Term::new(TermKind::Literal(Literal::Int(42)));
        let effect = EffectExpr::new(EffectExprKind::Pure(term));
        
        let result = registry.handle_effect(effect.clone()).unwrap();
        assert_eq!(result, effect);
    }
    
    #[test]
    fn test_unhandled_effect() {
        let registry = HandlerRegistry::new(); // Empty registry
        
        let effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "unknown".to_string(),
            args: vec![],
        });
        
        let result = registry.handle_effect(effect);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::UnhandledEffect { .. }));
    }
    
    #[test]
    fn test_default_handlers() {
        let registry = default_handlers();
        assert!(!registry.handlers.is_empty());
        
        let pure_handlers = registry.pure_handlers();
        assert!(!pure_handlers.is_empty());
    }
} 