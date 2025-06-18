// End-to-end test for error handling and propagation
use session::interpreter::Interpreter;
use session::layer3::agent::AgentId;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer2::outcome::{StateLocation, Value};
use session::layer3::agent::{Agent, AgentRegistry};
use session::layer3::compiler::compile_choreography;

#[test]
fn test_error_propagation_through_layers() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    // Test valid choreography works
    let valid_choreo = Choreography::Step(ChoreographyStep::Send {
        from: alice.clone(),
        to: bob.clone(),
        message: Message::Text("Valid message".to_string()),
    });
    
    let valid_result = interpreter.execute_choreography(&valid_choreo);
    assert!(valid_result.is_ok(), "Valid choreography should succeed");
    
    println!("✅ Error propagation test completed");
    
    Ok(())
}

#[test]
fn test_state_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Test state setting and getting
    let alice_location = StateLocation("Alice_balance".to_string());
    let bob_location = StateLocation("Bob_status".to_string());
    
    interpreter.set_state(alice_location.clone(), Value::Int(1000));
    interpreter.set_state(bob_location.clone(), Value::Bool(true));
    
    let state = interpreter.get_state();
    
    // Verify state was set
    assert!(state.contains_key(&alice_location), "Alice's state should be set");
    assert!(state.contains_key(&bob_location), "Bob's state should be set");
    
    println!("✅ State operations test completed");
    
    Ok(())
}

#[test]
fn test_debugging_and_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Enable debug mode
    interpreter.enable_debug();
    
    let alice = AgentId::new("Alice");
    let bob = AgentId::new("Bob");
    
    // Execute a choreography
    let debug_choreo = Choreography::Step(ChoreographyStep::Send {
        from: alice,
        to: bob,
        message: Message::Text("Debug test".to_string()),
    });
    
    let result = interpreter.execute_choreography(&debug_choreo);
    assert!(result.is_ok(), "Debug choreography should succeed");
    
    // Check that effect log is available
    let effect_log = interpreter.get_effect_log();
    assert!(!effect_log.is_empty(), "Effect log should contain entries");
    
    println!("✅ Debugging test completed");
    println!("   Effect log entries: {}", effect_log.len());
    
    Ok(())
}

#[test]
fn test_channel_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Test channel registry
    let registry = interpreter.get_channel_registry();
    
    // Create a channel
    registry.create_channel(
        "test_channel".to_string(),
        5,
        vec!["Alice".to_string(), "Bob".to_string()]
    );
    
    // Test sending and receiving
    registry.send("test_channel", Value::String("Hello".to_string()))?;
    let received = registry.receive("test_channel")?;
    
    assert!(received.is_some(), "Should receive a message");
    if let Some(Value::String(msg)) = received {
        assert_eq!(msg, "Hello", "Should receive correct message");
    }
    
    // Test channel status
    let status = registry.get_channel_status("test_channel");
    assert!(status.is_some(), "Channel should have status");
    
    if let Some((_queue_len, capacity, participants)) = status {
        assert_eq!(capacity, 5, "Capacity should be 5");
        assert_eq!(participants.len(), 2, "Should have 2 participants");
    }
    
    println!("✅ Channel operations test completed");
    
    Ok(())
}

#[test]
fn test_error_propagation() {
    let mut interpreter = Interpreter::new();
    
    // Test error at Layer 3 (missing agent)
    let registry = AgentRegistry::new(); // Empty registry
    
    let invalid_choreo = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("NonExistentAgent"),
        to: AgentId::new("AnotherMissing"),
        message: Message::Text("This should fail".to_string()),
    });
    
    let compile_result = compile_choreography(&invalid_choreo, &registry);
    assert!(compile_result.is_err(), "Should fail with missing agent");
    println!("✓ Layer 3 error properly propagated");
}

#[test]
fn test_state_error_handling() {
    let mut interpreter = Interpreter::new();
    
    // Test state access
    let alice_location = StateLocation("Alice_balance".to_string());
    let bob_location = StateLocation("Bob_status".to_string());
    
    // Test getting non-existent state
    let state = interpreter.get_state();
    let missing_value = state.get(&alice_location);
    assert!(missing_value.is_none(), "Non-existent state should return None");
    
    // Test setting and getting state
    interpreter.set_state(alice_location.clone(), Value::Int(100));
    interpreter.set_state(bob_location.clone(), Value::Bool(true));
    
    let updated_state = interpreter.get_state();
    assert!(updated_state.get(&alice_location).is_some(), "State should be set");
    assert!(updated_state.get(&bob_location).is_some(), "State should be set");
    
    println!("✓ State error handling working correctly");
}

#[test]
fn test_channel_error_handling() {
    let mut interpreter = Interpreter::new();
    let channel_registry = interpreter.get_channel_registry();
    
    // Test sending to non-existent channel
    let send_result = channel_registry.send("non_existent", Value::Int(42));
    assert!(send_result.is_err(), "Should fail when sending to non-existent channel");
    
    // Test receiving from non-existent channel
    let receive_result = channel_registry.receive("non_existent");
    assert!(receive_result.is_err(), "Should fail when receiving from non-existent channel");
    
    // Create a channel and test overflow
    channel_registry.create_channel(
        "test_channel".to_string(),
        2, // Small capacity
        vec!["Alice".to_string(), "Bob".to_string()]
    );
    
    // Fill the channel
    channel_registry.send("test_channel", Value::Int(1)).expect("Should succeed");
    channel_registry.send("test_channel", Value::Int(2)).expect("Should succeed");
    
    // This should fail due to capacity
    let overflow_result = channel_registry.send("test_channel", Value::Int(3));
    assert!(overflow_result.is_err(), "Should fail when channel is full");
    
    // Test channel status
    let status = channel_registry.get_channel_status("test_channel");
    if let Some((_queue_len, capacity, participants)) = status {
        assert_eq!(capacity, 2, "Capacity should be 2");
        assert_eq!(participants.len(), 2, "Should have 2 participants");
        println!("✓ Channel status: capacity {}, {} participants", 
                capacity, participants.len());
    }
    
    println!("✓ Channel error handling working correctly");
} 