//! Tests for core toolkit functionality

use causality_toolkit::core::*;
use causality_types::primitive::ids::ResourceId;

#[test]
fn test_typed_resource_creation() {
    let resource_id = ResourceId::new([1u8; 32]);
    let typed_resource: TypedResource<String> = TypedResource::new(resource_id);
    
    assert_eq!(typed_resource.id(), resource_id);
}

#[test]
fn test_resource_consumption() {
    let resource_id = ResourceId::new([2u8; 32]);
    let typed_resource: TypedResource<i32> = TypedResource::new(resource_id);
    
    let consumed = ConsumedResource::consume(typed_resource);
    assert_eq!(consumed.id(), resource_id);
    
    // Test nullifier creation
    let nullifier = consumed.nullifier();
    let nullifier_id = consumed.nullifier_id();
    
    // Just verify we can create these without panicking
    assert!(!format!("{:?}", nullifier).is_empty());
    assert!(!format!("{:?}", nullifier_id).is_empty());
}

#[test]
fn test_cloneable_effect_box() {
    let effect_box = CloneableEffectBox::new("test_effect");
    let cloned_box = effect_box.clone();
    
    // Verify they can be cloned and debugged
    assert!(!format!("{:?}", effect_box).is_empty());
    assert!(!format!("{:?}", cloned_box).is_empty());
}

#[test]
fn test_effect_expr_creation() {
    let pure_expr = EffectExpr::pure();
    assert!(!format!("{:?}", pure_expr).is_empty());
    
    let single_expr = EffectExpr::single("test_effect");
    assert!(!format!("{:?}", single_expr).is_empty());
    
    let sequence_expr = EffectExpr::sequence(vec![
        EffectExpr::pure(),
        EffectExpr::single("another_effect"),
    ]);
    assert!(!format!("{:?}", sequence_expr).is_empty());
}

#[test]
fn test_effect_expr_chaining() {
    let expr = EffectExpr::pure()
        .then("first_effect")
        .then("second_effect");
    
    // Verify we can chain effects
    assert!(!format!("{:?}", expr).is_empty());
}

#[test]
fn test_resource_states() {
    use std::mem;
    
    // Test that ResourceState enum has expected variants
    let active = ResourceState::Active;
    let consumed = ResourceState::Consumed;
    let created = ResourceState::Created;
    
    assert_eq!(mem::discriminant(&active), mem::discriminant(&ResourceState::Active));
    assert_eq!(mem::discriminant(&consumed), mem::discriminant(&ResourceState::Consumed));
    assert_eq!(mem::discriminant(&created), mem::discriminant(&ResourceState::Created));
    
    // Test that they're different
    assert_ne!(mem::discriminant(&active), mem::discriminant(&consumed));
    assert_ne!(mem::discriminant(&consumed), mem::discriminant(&created));
    assert_ne!(mem::discriminant(&active), mem::discriminant(&created));
} 