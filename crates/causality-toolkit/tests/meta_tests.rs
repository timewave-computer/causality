//! Tests for meta effects

use causality_toolkit::meta::*;
use causality_toolkit::AsTypeSchema;
use causality_types::effects_core::Effect;
use causality_types::expr::TypeExpr;

#[test]
fn test_declare_type_effect_creation() {
    let effect = DeclareTypeEffect::new("MyType", TypeExpr::String);
    assert_eq!(effect.name, "MyType");
    assert_eq!(effect.effect_type_name(), "meta.DeclareTypeEffect");
    assert_eq!(DeclareTypeEffect::EFFECT_TYPE, "meta.DeclareTypeEffect");
}

#[test]
fn test_declare_type_effect_schema() {
    let effect = DeclareTypeEffect::new("TestType", TypeExpr::Integer);
    let schema = effect.type_schema();
    
    // Verify we can get a schema without panicking
    assert!(!format!("{:?}", schema).is_empty());
    
    // Verify it's a record type
    match schema {
        TypeExpr::Record(_) => {},
        _ => panic!("Expected Record type schema"),
    }
}

#[test]
fn test_declare_type_effect_with_complex_schema() {
    use std::collections::BTreeMap;
    use causality_types::primitive::string::Str;
    use causality_types::expr::TypeExprMap;
    
    let mut fields = BTreeMap::new();
    fields.insert(Str::from("id"), TypeExpr::String);
    fields.insert(Str::from("value"), TypeExpr::Integer);
    
    let complex_schema = TypeExpr::Record(TypeExprMap(fields));
    let effect = DeclareTypeEffect::new("ComplexType", complex_schema);
    
    assert_eq!(effect.name, "ComplexType");
    
    let schema = effect.type_schema();
    assert!(!format!("{:?}", schema).is_empty());
}

#[test]
fn test_declare_type_effect_input_output() {
    // Test that we can create input and output types
    let input = DeclareTypeEffectInput;
    let output = DeclareTypeEffectOutput;
    
    // Verify schemas
    let input_schema = DeclareTypeEffectInput::schema();
    let output_schema = DeclareTypeEffectOutput::schema();
    
    assert_eq!(input_schema, TypeExpr::Unit);
    assert_eq!(output_schema, TypeExpr::Unit);
    
    // Test conversion methods
    use causality_types::expr::value::ValueExpr;
    use causality_types::effects_core::{EffectInput, EffectOutput};
    
    let input_result = DeclareTypeEffectInput::from_value_expr(ValueExpr::Unit);
    assert!(input_result.is_ok());
    
    let output_result = output.to_value_expr();
    assert!(output_result.is_ok());
    assert_eq!(output_result.unwrap(), ValueExpr::Unit);
} 