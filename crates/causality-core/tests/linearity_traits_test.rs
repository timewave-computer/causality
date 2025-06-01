//! Tests for the systematic linearity traits covering the 2x2 matrix

use causality_core::{
    LinearResource, Linear, Affine, Relevant, Unrestricted,
    SingleUse, Droppable, Copyable, MustUse, LinearityCheck,
};

/// Test the 2x2 matrix of linearity behaviors:
/// |                | CAN_DROP=false      | CAN_DROP=true       |
/// |----------------|-------------------- |---------------------|
/// | CAN_COPY=false | Linear: use once    | Affine: use ≤1      |
/// | CAN_COPY=true  | Relevant: use ≥1    | Unrestricted: any   |

#[test]
fn test_linear_traits() {
    // Linear: CAN_DROP=false, CAN_COPY=false
    let linear: LinearResource<i32, Linear> = LinearResource::new(42);
    
    // Linear implements SingleUse and MustUse
    assert!(!LinearResource::<i32, Linear>::can_drop());
    assert!(!LinearResource::<i32, Linear>::can_copy());
    assert!(LinearResource::<i32, Linear>::must_use());
    
    // Must be consumed
    assert!(!linear.check_used());
    let _ = SingleUse::consume(linear).unwrap();
}

#[test]
fn test_affine_traits() {
    // Affine: CAN_DROP=true, CAN_COPY=false
    let affine1: LinearResource<i32, Affine> = LinearResource::new(42);
    let affine2: LinearResource<i32, Affine> = LinearResource::new(84);
    
    // Affine implements SingleUse and Droppable
    assert!(LinearResource::<i32, Affine>::can_drop());
    assert!(!LinearResource::<i32, Affine>::can_copy());
    assert!(!LinearResource::<i32, Affine>::must_use());
    
    // Can be dropped without use
    Droppable::drop_unused(affine1).unwrap();
    
    // Or can be consumed once
    let _ = SingleUse::consume(affine2).unwrap();
}

#[test]
fn test_relevant_traits() {
    // Relevant: CAN_DROP=false, CAN_COPY=true
    let relevant: LinearResource<String, Relevant> = LinearResource::new("test".to_string());
    
    // Relevant implements Copyable and MustUse
    assert!(!LinearResource::<String, Relevant>::can_drop());
    assert!(LinearResource::<String, Relevant>::can_copy());
    assert!(LinearResource::<String, Relevant>::must_use());
    
    // Must be used at least once
    assert!(!relevant.check_used());
    
    // Can copy (which marks as used)
    let copy = Copyable::copy_value(&relevant).unwrap();
    assert_eq!(copy, "test");
    assert!(relevant.check_used());
    
    // Now can be safely dropped
    drop(relevant);
}

#[test]
fn test_unrestricted_traits() {
    // Unrestricted: CAN_DROP=true, CAN_COPY=true
    let unrestricted: LinearResource<String, Unrestricted> = LinearResource::new("free".to_string());
    
    // Unrestricted implements Copyable and Droppable
    assert!(LinearResource::<String, Unrestricted>::can_drop());
    assert!(LinearResource::<String, Unrestricted>::can_copy());
    assert!(!LinearResource::<String, Unrestricted>::must_use());
    
    // Can copy multiple times
    let copy1 = Copyable::copy_value(&unrestricted).unwrap();
    let copy2 = Copyable::copy_value(&unrestricted).unwrap();
    assert_eq!(copy1, "free");
    assert_eq!(copy2, "free");
    
    // Can drop without use (original resource)
    Droppable::drop_unused(unrestricted).unwrap();
}

#[test]
fn test_must_use_enforcement() {
    // Test Linear must use
    let linear: LinearResource<i32, Linear> = LinearResource::new(1);
    assert!(linear.ensure_will_be_used().is_err()); // Not used yet
    let _ = linear.consume().unwrap();
    
    // Test Relevant must use
    let relevant: LinearResource<i32, Relevant> = LinearResource::new(2);
    assert!(relevant.ensure_will_be_used().is_err()); // Not used yet
    let _ = relevant.copy_resource().unwrap();
    assert!(relevant.ensure_will_be_used().is_ok()); // Now used
}

#[test]
fn test_trait_combinations() {
    // Verify the 2x2 matrix systematically
    
    // Linear (false, false): SingleUse + MustUse
    fn check_linear<T>(_: &LinearResource<T, Linear>) 
    where
        LinearResource<T, Linear>: SingleUse + MustUse,
    {}
    
    // Affine (true, false): SingleUse + Droppable
    fn check_affine<T>(_: &LinearResource<T, Affine>)
    where
        LinearResource<T, Affine>: SingleUse + Droppable,
    {}
    
    // Relevant (false, true): Copyable + MustUse
    fn check_relevant<T: Clone>(_: &LinearResource<T, Relevant>)
    where
        LinearResource<T, Relevant>: Copyable + MustUse,
    {}
    
    // Unrestricted (true, true): Copyable + Droppable
    fn check_unrestricted<T: Clone>(_: &LinearResource<T, Unrestricted>)
    where
        LinearResource<T, Unrestricted>: Copyable + Droppable,
    {}
    
    // These should compile, verifying trait implementations
    let linear = LinearResource::<i32, Linear>::new(1);
    let affine = LinearResource::<i32, Affine>::new(2);
    let relevant = LinearResource::<i32, Relevant>::new(3);
    let unrestricted = LinearResource::<i32, Unrestricted>::new(4);
    
    check_linear(&linear);
    check_affine(&affine);
    check_relevant(&relevant);
    check_unrestricted(&unrestricted);
    
    // Clean up
    let _ = linear.consume().unwrap();
    drop(affine); // Affine can be dropped directly
    let _ = relevant.consume().unwrap();
    drop(unrestricted); // Unrestricted can be dropped directly
}

#[test]
fn test_linearity_aliases() {
    // Test the semantic aliases for linearity properties
    
    // Linear: must_use (cannot drop) AND use_once (cannot copy)
    assert!(LinearResource::<i32, Linear>::must_use());
    assert!(LinearResource::<i32, Linear>::use_once());
    
    // Affine: !must_use (can drop) AND use_once (cannot copy)
    assert!(!LinearResource::<i32, Affine>::must_use());
    assert!(LinearResource::<i32, Affine>::use_once());
    
    // Relevant: must_use (cannot drop) AND !use_once (can copy)
    assert!(LinearResource::<i32, Relevant>::must_use());
    assert!(!LinearResource::<i32, Relevant>::use_once());
    
    // Unrestricted: !must_use (can drop) AND !use_once (can copy)
    assert!(!LinearResource::<i32, Unrestricted>::must_use());
    assert!(!LinearResource::<i32, Unrestricted>::use_once());
    
    // Verify the aliases match the underlying properties
    assert_eq!(LinearResource::<i32, Linear>::must_use(), !LinearResource::<i32, Linear>::can_drop());
    assert_eq!(LinearResource::<i32, Linear>::use_once(), !LinearResource::<i32, Linear>::can_copy());
} 