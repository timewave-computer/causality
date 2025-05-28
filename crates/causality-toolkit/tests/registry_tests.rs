//! Tests for effect registry

use causality_toolkit::{Handles, SimpleEffectHandler, EffectRegistry, EffectAny};
use causality_types::effect::core::{Effect, EffectInput, EffectOutput, ConversionError};
use causality_types::expression::{r#type::TypeExpr, value::ValueExpr};

#[test]
fn test_registry_creation() {
    let registry = EffectRegistry::new();
    // Just verify we can create a registry
    assert!(!format!("{:?}", registry).is_empty());
}

#[test]
fn test_registry_register_and_handle() {
    let registry = EffectRegistry::new();
    let handler = TestHandler;
    
    registry.register::<TestEffect, _>(handler);
    
    let effect = TestEffect { value: 42 };
    let result = registry.handle(&effect);
    assert!(result.is_ok());
}

#[test]
fn test_registry_unhandled_effect() {
    let registry = EffectRegistry::new();
    let effect = UnhandledEffect;
    
    let result = registry.handle(&effect);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unhandled effect"));
}

// Test effect
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TestEffect {
    value: i32,
}

#[derive(Debug, Clone)]
struct TestEffectInput;

#[derive(Debug, Clone)]
struct TestEffectOutput;

impl EffectInput for TestEffectInput {
    fn schema() -> TypeExpr {
        TypeExpr::Unit
    }
    
    fn from_value_expr(_value: ValueExpr) -> Result<Self, ConversionError> {
        Ok(Self)
    }
}

impl EffectOutput for TestEffectOutput {
    fn schema() -> TypeExpr {
        TypeExpr::Unit
    }
    
    fn to_value_expr(&self) -> Result<ValueExpr, ConversionError> {
        Ok(ValueExpr::Nil)
    }
}

impl Effect for TestEffect {
    type Input = TestEffectInput;
    type Output = TestEffectOutput;
    
    const EFFECT_TYPE: &'static str = "test.TestEffect";
}



// Test handler
struct TestHandler;

impl Handles<TestEffect> for TestHandler {
    fn handle_effect(&self, _effect: &TestEffect) -> anyhow::Result<()> {
        Ok(())
    }
}

impl SimpleEffectHandler for TestHandler {
    fn handle_any(&self, _effect: &dyn EffectAny) -> anyhow::Result<()> {
        Ok(())
    }
}

// Unhandled effect for testing error cases
#[derive(Debug, Clone)]
struct UnhandledEffect;

impl Effect for UnhandledEffect {
    type Input = TestEffectInput;
    type Output = TestEffectOutput;
    
    const EFFECT_TYPE: &'static str = "test.UnhandledEffect";
}

// Registry integration tests

 