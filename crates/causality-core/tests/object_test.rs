//! Tests for linear type object system

use causality_core::{
    effect::{
        object::{AffineObject, LinearObject, Object, RelevantObject, UnrestrictedObject},
        capability::Capability,
        CapabilityError,
    },
};

#[test]
fn test_object_linearity_matrix() {
    // Test the 2x2 linearity matrix for objects
    
    // Linear: must use exactly once
    let linear_obj = LinearObject::linear(42);
    assert!(!LinearObject::<i32>::can_drop());
    assert!(!LinearObject::<i32>::can_copy());
    assert!(LinearObject::<i32>::must_use());
    assert!(LinearObject::<i32>::use_once());
    let value = linear_obj.consume().unwrap();
    assert_eq!(value, 42);
    
    // Affine: may use at most once
    let affine_obj = AffineObject::affine("test".to_string());
    assert!(AffineObject::<String>::can_drop());
    assert!(!AffineObject::<String>::can_copy());
    assert!(!AffineObject::<String>::must_use());
    assert!(AffineObject::<String>::use_once());
    affine_obj.drop_unused(); // Can drop without use
    
    // Relevant: must use at least once
    let relevant_obj = RelevantObject::relevant(vec![1, 2, 3]);
    assert!(!RelevantObject::<Vec<i32>>::can_drop());
    assert!(RelevantObject::<Vec<i32>>::can_copy());
    assert!(RelevantObject::<Vec<i32>>::must_use());
    assert!(!RelevantObject::<Vec<i32>>::use_once());
    let copy1 = relevant_obj.copy_data().unwrap();
    let copy2 = relevant_obj.copy_data().unwrap();
    assert_eq!(copy1, vec![1, 2, 3]);
    assert_eq!(copy2, vec![1, 2, 3]);
    
    // Unrestricted: may use any number of times
    let unrestricted_obj = UnrestrictedObject::unrestricted(100);
    assert!(UnrestrictedObject::<i32>::can_drop());
    assert!(UnrestrictedObject::<i32>::can_copy());
    assert!(!UnrestrictedObject::<i32>::must_use());
    assert!(!UnrestrictedObject::<i32>::use_once());
    let copy1 = unrestricted_obj.copy_data().unwrap();
    let copy2 = unrestricted_obj.copy_data().unwrap();
    assert_eq!(copy1, 100);
    assert_eq!(copy2, 100);
    let value = unrestricted_obj.consume().unwrap();
    assert_eq!(value, 100);
}

#[test]
fn test_object_capabilities() {
    let read_cap = Capability::read("read");
    let write_cap = Capability::write("write");
    let admin_cap = Capability::admin("admin");
    
    let obj = LinearObject::linear("sensitive_data".to_string())
        .with_capability(read_cap.clone())
        .with_capability(write_cap.clone());
    
    // Should have the capabilities we added
    assert!(obj.has_capability(&read_cap));
    assert!(obj.has_capability(&write_cap));
    assert!(!obj.has_capability(&admin_cap));
    
    // Should have all required capabilities
    assert!(obj.has_all_capabilities(&[read_cap.clone(), write_cap.clone()]));
    assert!(!obj.has_all_capabilities(&[admin_cap.clone()]));
    
    // Capability-checked operations should work
    let result = obj.with_capability_check(&read_cap, |data| data.len());
    assert_eq!(result.unwrap(), 14);
    
    // Missing capability should fail
    let result = obj.with_capability_check(&admin_cap, |data| data.len());
    assert!(matches!(result, Err(CapabilityError::MissingCapability { .. })));
    
    // Multi-capability check should work
    let result = obj.with_capabilities_check(
        &[read_cap, write_cap],
        |data| data.to_uppercase()
    );
    assert_eq!(result.unwrap(), "SENSITIVE_DATA");
}

#[test]
fn test_object_with_multiple_capabilities() {
    let caps = vec![
        Capability::read("read"),
        Capability::write("write"),
        Capability::execute("execute"),
    ];
    
    let obj = LinearObject::linear(42)
        .with_capabilities(caps.clone());
    
    // Should have all capabilities
    for cap in &caps {
        assert!(obj.has_capability(cap));
    }
    
    assert_eq!(obj.get_capabilities().len(), 3);
    assert!(obj.has_all_capabilities(&caps));
}

#[test]
fn test_object_linearity_names() {
    // Test that linearity names are correct for debugging
    assert!(LinearObject::<i32>::linearity_name().contains("Linear"));
    assert!(AffineObject::<i32>::linearity_name().contains("Affine"));
    assert!(RelevantObject::<i32>::linearity_name().contains("Relevant"));
    assert!(UnrestrictedObject::<i32>::linearity_name().contains("Unrestricted"));
}

#[test]
fn test_object_creation_patterns() {
    // Test different ways to create objects
    
    // Direct creation with capabilities
    let caps = vec![Capability::basic("test")].into_iter().collect();
    let obj1 = Object::<i32>::new(42, caps);
    assert_eq!(obj1.data, 42);
    assert_eq!(obj1.get_capabilities().len(), 1);
    
    // Creation from data (no capabilities)
    let obj2 = Object::<String>::from_data("hello".to_string());
    assert_eq!(obj2.data, "hello");
    assert!(obj2.get_capabilities().is_empty());
    
    // Linearity-specific constructors
    let linear = LinearObject::linear(1);
    let affine = AffineObject::affine(2);
    let relevant = RelevantObject::relevant(3);
    let unrestricted = UnrestrictedObject::unrestricted(4);
    
    assert_eq!(linear.data, 1);
    assert_eq!(affine.data, 2);
    assert_eq!(relevant.data, 3);
    assert_eq!(unrestricted.data, 4);
} 