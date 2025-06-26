//! Intent Evaluation System
//!
//! This module implements the intent evaluation system that converts high-level
//! declarative intents into concrete effect sequences using the unified
//! transform-based constraint system.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use crate::{
    lambda::base::Value,
    system::error::Result,
    effect::{Intent, IntentError},
};
use crate::effect::{
    handler_registry::{EffectHandlerRegistry, EffectExecutionError},
    transform_constraint::{TransformConstraintSystem, TransformConstraintError},
};

/// Result type for intent evaluation
pub type IntentEvaluationResult = Result<Vec<Value>>;

/// Errors that can occur during intent evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentEvaluationError {
    /// Transform constraint solving failed
    ConstraintSolvingFailed(TransformConstraintError),
    
    /// Intent validation failed
    IntentValidationFailed(IntentError),
    
    /// Constraint evaluation failed
    ConstraintEvaluationFailed(String),
    
    /// Required resource not available
    ResourceUnavailable(String),
    
    /// Intent evaluation timeout
    EvaluationTimeout,
    
    /// Effect execution failed
    EffectExecutionFailed(EffectExecutionError),
}

/// Configuration for intent evaluation
#[derive(Debug, Clone)]
pub struct IntentEvaluationConfig {
    /// Maximum number of effects to execute
    pub max_effects: usize,
    
    /// Timeout for evaluation in milliseconds
    pub timeout_ms: u64,
    
    /// Whether to validate constraints before execution
    pub validate_constraints: bool,
    
    /// Whether to simulate execution (dry run)
    pub simulation_mode: bool,
}

impl Default for IntentEvaluationConfig {
    fn default() -> Self {
        Self {
            max_effects: 100,
            timeout_ms: 5000,
            validate_constraints: true,
            simulation_mode: false,
        }
    }
}

/// Main intent evaluator that orchestrates intent evaluation
pub struct IntentEvaluator {
    /// Transform constraint system for converting intents to operations
    constraint_system: TransformConstraintSystem,
    
    /// Handler registry for effect execution
    handler_registry: Arc<Mutex<EffectHandlerRegistry>>,
    
    /// Intent evaluation configuration
    _config: IntentEvaluationConfig,
    
    /// Context for evaluation (resources, capabilities, etc.)
    context: EvaluationContext,
}

/// Context for intent evaluation
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    /// Available resources by name
    pub resources: BTreeMap<String, Value>,
    
    /// Available capabilities
    pub capabilities: Vec<String>,
    
    /// Metadata for evaluation
    pub metadata: BTreeMap<String, Value>,
}

impl IntentEvaluator {
    /// Create a new intent evaluator
    pub fn new(constraint_system: TransformConstraintSystem, handler_registry: EffectHandlerRegistry) -> Self {
        Self {
            constraint_system,
            handler_registry: Arc::new(Mutex::new(handler_registry)),
            _config: IntentEvaluationConfig::default(),
            context: EvaluationContext::default(),
        }
    }
    
    /// Create a new intent evaluator with custom configuration
    pub fn with_config(
        constraint_system: TransformConstraintSystem,
        handler_registry: EffectHandlerRegistry,
        config: IntentEvaluationConfig,
    ) -> Self {
        Self {
            constraint_system,
            handler_registry: Arc::new(Mutex::new(handler_registry)),
            _config: config,
            context: EvaluationContext::default(),
        }
    }
    
    /// Set the evaluation context
    pub fn with_context(mut self, context: EvaluationContext) -> Self {
        self.context = context;
        self
    }
    
    /// Evaluate an intent and return the results
    pub fn evaluate_intent(&self, _intent: &Intent) -> IntentEvaluationResult {
        // For now, return a simple result to get compilation working
        // Full implementation will use the constraint system to solve intents
        // and execute the resulting operations
        Ok(vec![Value::Unit])
    }
    
    /// Get the handler registry for external configuration
    pub fn handler_registry(&self) -> Arc<Mutex<EffectHandlerRegistry>> {
        Arc::clone(&self.handler_registry)
    }
    
    /// Get a mutable reference to the handler registry
    pub fn handler_registry_mut(&mut self) -> Arc<Mutex<EffectHandlerRegistry>> {
        Arc::clone(&self.handler_registry)
    }
    
    /// Get the constraint system for external configuration
    pub fn constraint_system(&self) -> &TransformConstraintSystem {
        &self.constraint_system
    }
    
    /// Helper method to evaluate literal values
    fn _evaluate_literal(&self, lit: &crate::lambda::Literal) -> Result<Value> {
        match lit {
            crate::lambda::Literal::Bool(b) => Ok(Value::Bool(*b)),
            crate::lambda::Literal::Int(i) => Ok(Value::Int(*i)),
            crate::lambda::Literal::Symbol(s) => Ok(Value::Symbol(s.as_str().into())),
            crate::lambda::Literal::Unit => Ok(Value::Unit),
        }
    }
    
    /// Helper method to evaluate term expressions
    fn _evaluate_term(&self, term: &crate::lambda::Term) -> Result<Value> {
        match &term.kind {
            crate::lambda::TermKind::Literal(lit) => self._evaluate_literal(lit),
            crate::lambda::TermKind::Var(name) => {
                // Look up variable in context
                self.context.resources.get(name)
                    .cloned()
                    .ok_or_else(|| crate::system::error::Error::serialization(
                        format!("Variable not found: {}", name)))
            }
            // Add other term kinds as needed
            _ => Ok(Value::Unit), // Simplified for now
        }
    }
}

impl std::fmt::Display for IntentEvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentEvaluationError::ConstraintSolvingFailed(err) => {
                write!(f, "Constraint solving failed: {}", err)
            }
            IntentEvaluationError::IntentValidationFailed(err) => {
                write!(f, "Intent validation failed: {}", err)
            }
            IntentEvaluationError::ConstraintEvaluationFailed(msg) => {
                write!(f, "Constraint evaluation failed: {}", msg)
            }
            IntentEvaluationError::ResourceUnavailable(resource) => {
                write!(f, "Resource unavailable: {}", resource)
            }
            IntentEvaluationError::EvaluationTimeout => {
                write!(f, "Intent evaluation timed out")
            }
            IntentEvaluationError::EffectExecutionFailed(err) => {
                write!(f, "Effect execution failed: {}", err)
            }
        }
    }
}

impl std::error::Error for IntentEvaluationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::Location;
    use crate::effect::{
        handler_registry::EffectHandlerRegistry,
        transform_constraint::TransformConstraintSystem,
    };

    fn create_test_evaluator() -> IntentEvaluator {
        let constraint_system = TransformConstraintSystem::new();
        let handler_registry = EffectHandlerRegistry::new();
        IntentEvaluator::new(constraint_system, handler_registry)
    }

    #[test]
    fn test_evaluator_creation() {
        let evaluator = create_test_evaluator();
        
        // Verify basic properties
        assert!(evaluator.handler_registry().lock().is_ok());
        // Note: TransformConstraintSystem doesn't expose constraints directly
        // This is by design for encapsulation
    }

    #[test]
    fn test_simple_intent_evaluation() {
        let evaluator = create_test_evaluator();
        let intent = Intent::new(Location::Local);
        
        let result = evaluator.evaluate_intent(&intent);
        assert!(result.is_ok());
        
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::Unit);
    }

    #[test]
    fn test_intent_evaluator_basic() {
        let constraint_system = TransformConstraintSystem::new();
        let handler_registry = EffectHandlerRegistry::new();
        let evaluator = IntentEvaluator::new(constraint_system, handler_registry);
        
        // Test basic functionality
        assert!(evaluator.handler_registry().lock().is_ok());
    }
} 