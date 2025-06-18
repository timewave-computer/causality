// Property tests for session system invariants
use session::layer0::MessageValue;
use session::layer1::{Type, SessionType};
use session::layer2::outcome::{Outcome, Value, StateLocation};
use session::interpreter::Interpreter;

/// Test that message values have consistent equality
#[test]
fn property_message_value_equality() {
    // Test reflexivity: a = a
    let value = MessageValue::Int(42);
    assert_eq!(value, value, "Value should equal itself");
    
    // Test symmetry: if a = b then b = a
    let value1 = MessageValue::Bool(true);
    let value2 = MessageValue::Bool(true);
    assert_eq!(value1, value2, "Equal values should be symmetric");
    assert_eq!(value2, value1, "Equal values should be symmetric");
    
    // Test transitivity: if a = b and b = c then a = c
    let value_a = MessageValue::Int(5);
    let value_b = MessageValue::Int(5);
    let value_c = MessageValue::Int(5);
    assert_eq!(value_a, value_b);
    assert_eq!(value_b, value_c);
    assert_eq!(value_a, value_c, "Equality should be transitive");
    
    // Test inequality
    let different1 = MessageValue::Int(1);
    let different2 = MessageValue::Int(2);
    assert_ne!(different1, different2, "Different values should not be equal");
    
    println!("✅ Message value equality properties verified");
}

/// Test type system properties
#[test]
fn property_type_system() {
    // Test type equality is reflexive
    let int_type = Type::Int;
    assert_eq!(int_type, int_type, "Type should equal itself");
    
    // Test basic type construction
    let bool_type = Type::Bool;
    let unit_type = Type::Unit;
    
    assert_ne!(int_type, bool_type, "Different types should not be equal");
    assert_ne!(bool_type, unit_type, "Different types should not be equal");
    
    // Test session type construction
    let session = SessionType::Send(Box::new(Type::Bool), Box::new(SessionType::End));
    assert!(matches!(session, SessionType::Send(_, _)), "Session type should match pattern");
    
    println!("✅ Type system properties verified");
}

/// Test that outcome operations work correctly
#[test]
fn property_outcome_verification() -> Result<(), Box<dyn std::error::Error>> {
    // Test identity
    let empty = Outcome::empty();
    assert!(empty.verify(), "Identity outcome should verify");
    
    // Test that empty outcome has no declarations
    assert_eq!(empty.declarations.len(), 0, "Empty outcome should have no declarations");
    
    println!("✅ Outcome verification properties verified");
    
    Ok(())
}

/// Test interpreter state consistency
#[test]
fn property_interpreter_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Test initial state
    let initial_state = interpreter.get_state();
    assert!(initial_state.is_empty(), "Initial state should be empty");
    
    // Test state setting
    let location = StateLocation("test_contract_key".to_string());
    interpreter.set_state(location.clone(), Value::Int(42));
    
    let state_after_set = interpreter.get_state();
    assert!(!state_after_set.is_empty(), "State should not be empty after setting");
    assert!(state_after_set.contains_key(&location), "State should contain the key we set");
    
    println!("✅ Interpreter state properties verified");
    
    Ok(())
}

/// Test value type consistency
#[test]
fn property_value_types() {
    // Test different value types
    let int_val = Value::Int(42);
    let bool_val = Value::Bool(true);
    let string_val = Value::String("test".to_string());
    
    // Test that values are distinguishable
    assert!(matches!(int_val, Value::Int(_)), "Int value should match Int pattern");
    assert!(matches!(bool_val, Value::Bool(_)), "Bool value should match Bool pattern");
    assert!(matches!(string_val, Value::String(_)), "String value should match String pattern");
    
    // Test value cloning
    let cloned_int = int_val.clone();
    assert_eq!(int_val, cloned_int, "Cloned values should be equal");
    
    println!("✅ Value type properties verified");
}

/// Test message value construction
#[test]
fn test_message_value_construction() {
    // Test basic message value construction and properties
    let values = vec![
        MessageValue::Unit,
        MessageValue::Bool(true),
        MessageValue::Bool(false),
        MessageValue::Int(42),
        MessageValue::Int(-100),
        MessageValue::Int(0),
    ];
    
    for value in &values {
        // Each value should be equal to itself
        assert_eq!(value, value, "Value should equal itself");
        
        // Values should have consistent debug representation
        let debug_str = format!("{:?}", value);
        assert!(!debug_str.is_empty(), "Debug representation should not be empty");
    }
    
    // Test pair construction
    let pair = MessageValue::Pair(
        Box::new(MessageValue::Int(1)),
        Box::new(MessageValue::Bool(true))
    );
    
    // Test that pair contains the expected values
    if let MessageValue::Pair(left, right) = &pair {
        assert_eq!(**left, MessageValue::Int(1), "Left side should be Int(1)");
        assert_eq!(**right, MessageValue::Bool(true), "Right side should be Bool(true)");
    } else {
        panic!("Should be a pair");
    }
    
    println!("✓ Message value construction properties verified");
}

#[test]
fn test_deterministic_execution() {
    // Test that deterministic execution produces the same results
    let mut interpreter1 = Interpreter::new();
    let mut interpreter2 = Interpreter::new();
    
    // Same initial state
    let location = StateLocation("test_key".to_string());
    interpreter1.set_state(location.clone(), Value::Int(42));
    interpreter2.set_state(location.clone(), Value::Int(42));
    
    // Same operations 
    interpreter1.set_state(StateLocation("key2".to_string()), Value::Bool(true));
    interpreter2.set_state(StateLocation("key2".to_string()), Value::Bool(true));
    
    // Should have identical state
    let state1 = interpreter1.get_state();
    let state2 = interpreter2.get_state();
    
    assert_eq!(state1.len(), state2.len(), "States should have same size");
    
    // Check specific values
    assert_eq!(
        state1.get(&location),
        state2.get(&location),
        "States should have same values"
    );
    
    println!("✓ Deterministic execution verified");
}

#[test]
fn test_state_isolation() {
    // Test that different state locations are isolated
    let mut interpreter = Interpreter::new();
    
    let loc1 = StateLocation("isolated1".to_string());
    let loc2 = StateLocation("isolated2".to_string());
    
    interpreter.set_state(loc1.clone(), Value::Int(100));
    interpreter.set_state(loc2.clone(), Value::Int(200));
    
    let state = interpreter.get_state();
    
    assert_eq!(
        state.get(&loc1),
        Some(&Value::Int(100)),
        "First location should have correct value"
    );
    assert_eq!(
        state.get(&loc2),
        Some(&Value::Int(200)),
        "Second location should have correct value"
    );
    
    // Modify one location
    interpreter.set_state(loc1.clone(), Value::Int(999));
    
    let updated_state = interpreter.get_state();
    assert_eq!(
        updated_state.get(&loc1),
        Some(&Value::Int(999)),
        "First location should be updated"
    );
    assert_eq!(
        updated_state.get(&loc2),
        Some(&Value::Int(200)),
        "Second location should be unchanged"
    );
    
    println!("✓ State isolation verified");
}

#[test] 
fn test_state_location_properties() {
    // Test state location construction and usage
    let location = StateLocation("test_contract_key".to_string());
    
    let mut interpreter = Interpreter::new();
    
    // Test setting and getting
    interpreter.set_state(location.clone(), Value::Bool(false));
    
    let state = interpreter.get_state();
    let retrieved = state.get(&location);
    
    assert_eq!(retrieved, Some(&Value::Bool(false)), "Should retrieve the stored value");
    
    // Test that different locations are distinct
    let other_location = StateLocation("different_key".to_string());
    let other_value = state.get(&other_location);
    assert_eq!(other_value, None, "Different location should not have value");
    
    println!("✓ State location properties verified");
} 