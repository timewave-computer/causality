//! Tests for the complete linearity type system

use causality_core::{LinearResource, Linear, Affine, Relevant, Unrestricted, LinearityError};

#[test]
fn test_linear_resource_must_be_consumed() {
    // Linear resources must be consumed exactly once
    let resource: LinearResource<i32, Linear> = LinearResource::new(42);
    
    // Consume the resource
    let value = resource.consume().unwrap();
    assert_eq!(value, 42);
    
    // Cannot drop without consuming - tested in drop_linear_panics
}

#[test]
#[should_panic(expected = "Linear resource dropped without being consumed")]
fn test_drop_linear_panics() {
    // This should panic because linear resources must be consumed
    let _resource: LinearResource<i32, Linear> = LinearResource::new(42);
    // Resource is dropped here without being consumed
}

#[test]
fn test_affine_resource_can_be_dropped() {
    // Affine resources can be used at most once (0 or 1 times)
    let resource: LinearResource<i32, Affine> = LinearResource::new(42);
    
    // We can drop it without consuming
    resource.drop_resource();
    
    // Or we can consume it
    let resource2: LinearResource<i32, Affine> = LinearResource::new(84);
    let value = resource2.consume().unwrap();
    assert_eq!(value, 84);
}

#[test]
fn test_relevant_resource_must_be_used() {
    // Relevant resources must be used at least once but can be copied
    let resource: LinearResource<String, Relevant> = LinearResource::new("hello".to_string());
    
    // We can copy the value multiple times
    let copy1 = resource.copy_resource().unwrap();
    let copy2 = resource.copy_resource().unwrap();
    assert_eq!(copy1, "hello");
    assert_eq!(copy2, "hello");
    
    // Finally consume it
    let final_value = resource.consume().unwrap();
    assert_eq!(final_value, "hello");
}

#[test]
#[should_panic(expected = "Relevant resource dropped without being used")]
fn test_drop_relevant_without_use_panics() {
    // This should panic because relevant resources must be used at least once
    let _resource: LinearResource<i32, Relevant> = LinearResource::new(42);
    // Resource is dropped here without being used
}

#[test]
fn test_unrestricted_resource_can_be_copied_and_dropped() {
    // Unrestricted resources can be used any number of times
    let resource: LinearResource<String, Unrestricted> = LinearResource::new("world".to_string());
    
    // Copy multiple times
    let copy1 = resource.copy_resource().unwrap();
    let copy2 = resource.copy_resource().unwrap();
    let copy3 = resource.copy_resource().unwrap();
    
    assert_eq!(copy1, "world");
    assert_eq!(copy2, "world");
    assert_eq!(copy3, "world");
    
    // Can be dropped without consuming
    drop(resource);
}

#[test]
fn test_linearity_combinations() {
    // Test that we can create resources with all linearity types
    let linear: LinearResource<i32, Linear> = LinearResource::new(1);
    let affine: LinearResource<i32, Affine> = LinearResource::new(2);
    let relevant: LinearResource<i32, Relevant> = LinearResource::new(3);
    let unrestricted: LinearResource<i32, Unrestricted> = LinearResource::new(4);
    
    // Consume them appropriately
    assert_eq!(linear.consume().unwrap(), 1);
    assert_eq!(affine.consume().unwrap(), 2);
    assert_eq!(relevant.consume().unwrap(), 3);
    assert_eq!(unrestricted.consume().unwrap(), 4);
}

#[test]
fn test_multiple_use_error() {
    let mut resource: LinearResource<i32, Linear> = LinearResource::new(42);
    
    // First consumption succeeds
    let value = resource.try_consume().unwrap();
    assert_eq!(value, 42);
    
    // Second consumption fails
    match resource.try_consume() {
        Err(LinearityError::MultipleUse) => {
            // Expected error
        }
        _ => panic!("Expected MultipleUse error"),
    }
}

#[test]
fn test_copy_after_drop_error() {
    let mut resource: LinearResource<String, Unrestricted> = LinearResource::new("test".to_string());
    
    // Consume the resource
    let _ = resource.try_consume().unwrap();
    
    // Try to copy after consumption
    match resource.copy_resource() {
        Err(LinearityError::UseAfterDrop) => {
            // Expected error
        }
        _ => panic!("Expected UseAfterDrop error"),
    }
}

#[test]
fn test_is_consumed_tracking() {
    let mut resource: LinearResource<i32, Linear> = LinearResource::new(42);
    
    // Initially not consumed
    assert!(!resource.is_consumed());
    
    // After consumption
    let _ = resource.try_consume().unwrap();
    assert!(resource.is_consumed());
}

#[test]
fn test_relevant_has_been_used() {
    let mut resource: LinearResource<i32, Relevant> = LinearResource::new(42);
    
    // Initially not used
    assert!(!resource.has_been_used());
    
    // After consumption it's considered used
    let _ = resource.try_consume().unwrap();
    assert!(resource.has_been_used());
}

#[test]
fn test_relevant_copy_marks_as_used() {
    // Relevant resources must be used at least once
    let resource: LinearResource<String, Relevant> = LinearResource::new("test".to_string());
    
    // Initially not used
    assert!(!resource.has_been_used());
    
    // Copy the resource - this should mark it as used
    let copy = resource.copy_resource().unwrap();
    assert_eq!(copy, "test");
    assert!(resource.has_been_used());
    
    // Now it can be safely dropped without panic
    drop(resource); // Should not panic
}

#[test]
fn test_unrestricted_copy_marks_as_used() {
    // Unrestricted resources can be used any number of times
    let resource: LinearResource<String, Unrestricted> = LinearResource::new("unrestricted".to_string());
    
    // Initially not used
    assert!(!resource.has_been_used());
    
    // Copy marks as used
    let copy = resource.copy_resource().unwrap();
    assert_eq!(copy, "unrestricted");
    assert!(resource.has_been_used());
}

#[test]
fn test_linear_has_been_used() {
    let resource: LinearResource<i32, Linear> = LinearResource::new(42);
    
    // Initially not used
    assert!(!resource.has_been_used());
    
    // Consume marks as used
    let value = resource.consume().unwrap();
    assert_eq!(value, 42);
    // Can't check has_been_used after consume since resource is moved
} 