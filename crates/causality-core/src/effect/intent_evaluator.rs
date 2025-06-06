//! Intent Evaluation System
//!
//! This module implements the intent evaluation system that converts high-level
//! declarative intents into concrete effect sequences using algebraic effects
//! and handlers.

use std::collections::HashMap;
use crate::{
    effect::{
        intent::{Intent, IntentError},
        synthesis::{FlowSynthesizer, SynthesisError},
        handler_registry::{EffectHandlerRegistry, EffectExecutionError},
    },
    lambda::base::Value,
    system::error::Result,
};

/// Result type for intent evaluation
pub type IntentEvaluationResult = Result<Vec<Value>>;

/// Errors that can occur during intent evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentEvaluationError {
    /// Intent synthesis failed
    SynthesisFailed(SynthesisError),
    
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
    /// Flow synthesizer for converting intents to effect sequences
    synthesizer: FlowSynthesizer,
    
    /// Handler registry for executing effects
    handler_registry: EffectHandlerRegistry,
    
    /// Evaluation configuration
    config: IntentEvaluationConfig,
    
    /// Context for evaluation (resources, capabilities, etc.)
    context: EvaluationContext,
}

/// Context for intent evaluation
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    /// Available resources by name
    pub resources: HashMap<String, Value>,
    
    /// Available capabilities
    pub capabilities: Vec<String>,
    
    /// Metadata for evaluation
    pub metadata: HashMap<String, Value>,
}

impl IntentEvaluator {
    /// Create a new intent evaluator
    pub fn new(synthesizer: FlowSynthesizer, handler_registry: EffectHandlerRegistry) -> Self {
        Self {
            synthesizer,
            handler_registry,
            config: IntentEvaluationConfig::default(),
            context: EvaluationContext::default(),
        }
    }
    
    /// Create a new intent evaluator with custom configuration
    pub fn with_config(
        synthesizer: FlowSynthesizer,
        handler_registry: EffectHandlerRegistry,
        config: IntentEvaluationConfig,
    ) -> Self {
        Self {
            synthesizer,
            handler_registry,
            config,
            context: EvaluationContext::default(),
        }
    }
    
    /// Set the evaluation context
    pub fn with_context(mut self, context: EvaluationContext) -> Self {
        self.context = context;
        self
    }
    
    /// Evaluate an intent and return the results
    pub fn evaluate_intent(&self, intent: &Intent) -> IntentEvaluationResult {
        // For now, return a simple result to get compilation working
        // Full implementation will come in subsequent steps
        Ok(vec![Value::Unit])
    }
    
    /// Get the handler registry for external configuration
    pub fn handler_registry(&self) -> &EffectHandlerRegistry {
        &self.handler_registry
    }
    
    /// Get a mutable reference to the handler registry
    pub fn handler_registry_mut(&mut self) -> &mut EffectHandlerRegistry {
        &mut self.handler_registry
    }
    
    /// Helper method to evaluate literal values
    fn evaluate_literal(&self, lit: &crate::lambda::Literal) -> Result<Value> {
        match lit {
            crate::lambda::Literal::Bool(b) => Ok(Value::Bool(*b)),
            crate::lambda::Literal::Int(i) => Ok(Value::Int(*i)),
            crate::lambda::Literal::Symbol(s) => Ok(Value::Symbol(s.as_str().into())),
            crate::lambda::Literal::Unit => Ok(Value::Unit),
        }
    }
    
    /// Helper method to evaluate term expressions
    fn evaluate_term(&self, term: &crate::lambda::Term) -> Result<Value> {
        match &term.kind {
            crate::lambda::TermKind::Literal(lit) => self.evaluate_literal(lit),
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
            IntentEvaluationError::SynthesisFailed(err) => {
                write!(f, "Intent synthesis failed: {}", err)
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
    use crate::{
        system::content_addressing::DomainId,
        effect::{
            synthesis::FlowSynthesizer,
            intent::Constraint,
        },
    };
    
    
    fn create_test_evaluator() -> IntentEvaluator {
        let synthesizer = FlowSynthesizer::new(DomainId::default());
        let handler_registry = EffectHandlerRegistry::new();
        
        IntentEvaluator::new(synthesizer, handler_registry)
    }
    
    #[test]
    fn test_intent_evaluator_creation() {
        let evaluator = create_test_evaluator();
        // Check that the evaluator was created successfully
        assert!(evaluator.handler_registry().list_effects().is_empty());
    }
    
    #[test]
    fn test_simple_intent_evaluation() {
        let evaluator = create_test_evaluator();
        
        // Create a simple intent with trivial constraint
        let intent = Intent::new(
            DomainId::default(),
            vec![], // No inputs for this test
            Constraint::True,
        );
        
        let result = evaluator.evaluate_intent(&intent);
        assert!(result.is_ok());
    }
} 