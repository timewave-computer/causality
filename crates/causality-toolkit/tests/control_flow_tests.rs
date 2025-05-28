//! Tests for control flow effects

use causality_toolkit::{IfEffect, SequenceEffect, WhileEffect, AsTypeSchema, ControlFlowInput, ControlFlowOutput, HandleableEffect};
use causality_toolkit::control_flow::SimpleEffectHandler;
use causality_types::effect::core::Effect;

#[test]
fn test_if_effect_creation() {
    let effect = IfEffect::new(true, DummyEffect);
    assert!(effect.condition);
    assert_eq!(effect.effect_type_name(), "control_flow.IfEffect");
    assert_eq!(IfEffect::EFFECT_TYPE, "control_flow.IfEffect");
}

#[test]
fn test_if_effect_with_else() {
    let effect = IfEffect::new(false, DummyEffect)
        .with_else(DummyEffect);
    assert!(!effect.condition);
    assert!(effect.else_effect.is_some());
}

#[test]
fn test_sequence_effect_creation() {
    let effect = SequenceEffect::new()
        .with_effect(DummyEffect)
        .with_effect(DummyEffect);
    assert_eq!(effect.effects.len(), 2);
    assert_eq!(effect.effect_type_name(), "control_flow.SequenceEffect");
}

#[test]
fn test_while_effect_creation() {
    let effect = WhileEffect::new(|| false, DummyEffect, 10);
    assert_eq!(effect.max_iterations, 10);
    assert_eq!(effect.effect_type_name(), "control_flow.WhileEffect");
}

#[test]
fn test_effect_schemas() {
    let if_effect = IfEffect::new(true, DummyEffect);
    let schema = if_effect.type_schema();
    // Just verify we can get a schema without panicking
    assert!(!format!("{:?}", schema).is_empty());
    
    let seq_effect = SequenceEffect::new();
    let schema = seq_effect.type_schema();
    assert!(!format!("{:?}", schema).is_empty());
    
    let while_effect = WhileEffect::new(|| true, DummyEffect, 5);
    let schema = while_effect.type_schema();
    assert!(!format!("{:?}", schema).is_empty());
}

// Dummy effect for testing
#[derive(Debug, Clone)]
struct DummyEffect;

impl HandleableEffect for DummyEffect {
    fn handle(
        &self,
        _handler: &dyn SimpleEffectHandler,
    ) -> causality_toolkit::ToolkitResult<()> {
        Ok(())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Effect for DummyEffect {
    type Input = ControlFlowInput;
    type Output = ControlFlowOutput;
    
    const EFFECT_TYPE: &'static str = "test.DummyEffect";
}

impl AsTypeSchema for DummyEffect {
    fn type_schema(&self) -> causality_types::expression::r#type::TypeExpr {
        causality_types::expression::r#type::TypeExpr::Unit
    }
    
    fn effect_type_name(&self) -> &'static str {
        "test.DummyEffect"
    }
} 