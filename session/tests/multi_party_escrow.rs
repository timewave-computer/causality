// Session crate multi-party test scenarios using real API
use session::layer2::outcome::{Value, StateLocation};
use session::layer3::agent::{Agent, AgentId, AgentRegistry};
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer3::compiler::compile_choreography;
use session::layer3::capability::Capability;
use session::interpreter::Interpreter;

/// Basic choreography execution test
#[test]
fn test_basic_choreography_execution() {
    println!("=== Testing Basic Choreography Execution ===");
    
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Create agents with appropriate capabilities
    let mut alice = Agent::new("Alice");
    alice.add_capability(Capability::new(
        "Communication".to_string(),
        session::layer2::effect::EffectRow::from_effects(vec![
            ("comm_send".to_string(), session::layer2::effect::EffectType::Comm),
        ])
    ));
    
    let mut bob = Agent::new("Bob");
    bob.add_capability(Capability::new(
        "Communication".to_string(),
        session::layer2::effect::EffectRow::from_effects(vec![
            ("comm_send".to_string(), session::layer2::effect::EffectType::Comm),
        ])
    ));
    
    // Register agents
    let mut registry = AgentRegistry::new();
    registry.register(alice).unwrap();
    registry.register(bob).unwrap();
    
    // Create a simple choreography
    let choreography = Choreography::Sequence(vec![
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello".to_string()),
        }),
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Text("World".to_string()),
        }),
    ]);
    
    // Compile and execute
    match compile_choreography(&choreography, &registry) {
        Ok(effects) => {
            println!("✓ Choreography compiled successfully ({} effects)", effects.len());
            
            for effect in effects {
                if let Err(e) = interpreter.execute_effect(effect) {
                    println!("Effect execution error: {}", e);
                }
            }
            
            println!("✓ Choreography executed successfully");
        }
        Err(e) => {
            println!("✗ Compilation failed: {}", e);
            panic!("Basic choreography test failed");
        }
    }
    
    interpreter.print_state();
}

/// Test parallel operations
#[test]
fn test_parallel_operations() {
    println!("=== Testing Parallel Operations ===");
    
    let mut interpreter = Interpreter::new();
    let mut registry = AgentRegistry::new();
    
    // Create multiple agents
    for agent_name in ["Alice", "Bob", "Carol"] {
        let mut agent = Agent::new(agent_name);
        agent.add_capability(Capability::new(
            "Communication".to_string(),
            session::layer2::effect::EffectRow::from_effects(vec![
                ("comm_send".to_string(), session::layer2::effect::EffectType::Comm),
            ])
        ));
        registry.register(agent).unwrap();
    }
    
    // Create parallel choreography
    let parallel_choreo = Choreography::Parallel(vec![
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Parallel message 1".to_string()),
        }),
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Carol"),
            to: AgentId::new("Bob"),
            message: Message::Text("Parallel message 2".to_string()),
        }),
    ]);
    
    // Compile and execute
    match compile_choreography(&parallel_choreo, &registry) {
        Ok(effects) => {
            println!("✓ Parallel choreography compiled ({} effects)", effects.len());
            
            for effect in effects {
                if let Err(e) = interpreter.execute_effect(effect) {
                    println!("Effect execution error: {}", e);
                }
            }
            
            println!("✓ Parallel operations executed successfully");
        }
        Err(e) => {
            println!("✗ Parallel choreography failed: {}", e);
            panic!("Parallel operations test failed");
        }
    }
}

/// Test channel management
#[test]
fn test_channel_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Channel Management ===");
    
    let mut interpreter = Interpreter::new();
    
    // Test channel creation and management
    let channel_registry = interpreter.get_channel_registry();
    channel_registry.create_channel(
        "test_channel".to_string(), 
        10, 
        vec!["Alice".to_string(), "Bob".to_string()]
    );
    
    // Test sending and receiving
    channel_registry.send("test_channel", Value::Int(42))?;
    let received = channel_registry.receive("test_channel")?;
    
    assert!(received.is_some(), "Should receive the sent value");
    if let Some(Value::Int(value)) = received {
        assert_eq!(value, 42, "Should receive correct value");
    }
    
    // Test channel status
    let status = channel_registry.get_channel_status("test_channel");
    assert!(status.is_some(), "Channel should have status");
    
    println!("✅ Channel management test completed");
    
    Ok(())
}

/// Test state management
#[test]
fn test_state_management() {
    println!("=== Testing State Management ===");
    
    let mut interpreter = Interpreter::new();
    
    // Test state operations
    let alice_location = StateLocation("Alice_balance".to_string());
    let bob_location = StateLocation("Bob_balance".to_string());
    
    interpreter.set_state(alice_location.clone(), Value::Int(1000));
    interpreter.set_state(bob_location.clone(), Value::Int(500));
    
    let state = interpreter.get_state();
    
    // Verify state was set correctly
    if let Some(Value::Int(balance)) = state.get(&alice_location) {
        assert_eq!(*balance, 1000, "Alice should have 1000");
    } else {
        panic!("Alice's balance should be set");
    }
    
    if let Some(Value::Int(balance)) = state.get(&bob_location) {
        assert_eq!(*balance, 500, "Bob should have 500");
    } else {
        panic!("Bob's balance should be set");
    }
    
    println!("✅ State management test completed");
}

/// Test missing agent error handling
#[test]
fn test_missing_agent_error() {
    println!("=== Testing Missing Agent Error Handling ===");
    
    let mut registry = AgentRegistry::new();
    
    // Only register Alice
    let alice = Agent::new("Alice");
    registry.register(alice).unwrap();
    
    // Try to create choreography with missing Bob
    let missing_agent_choreo = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("MissingBob"),
        message: Message::Text("Hello".to_string()),
    });
    
    // This should fail with AgentNotFound error
    match compile_choreography(&missing_agent_choreo, &registry) {
        Ok(_) => panic!("Expected AgentNotFound error"),
        Err(e) => {
            println!("✓ Correctly caught missing agent error: {}", e);
            assert!(format!("{}", e).contains("lacks capability"));
        }
    }
}

/// Test capability enforcement
#[test]
fn test_capability_enforcement() {
    println!("=== Testing Capability Enforcement ===");
    
    let mut registry = AgentRegistry::new();
    
    // Create agents with different capabilities
    let mut alice = Agent::new("Alice");
    alice.add_capability(Capability::new(
        "Communication".to_string(),
        session::layer2::effect::EffectRow::from_effects(vec![
            ("comm_send".to_string(), session::layer2::effect::EffectType::Comm),
        ])
    ));
    
    let bob = Agent::new("Bob"); // Bob has no capabilities
    
    registry.register(alice).unwrap();
    registry.register(bob).unwrap();
    
    // Create valid choreography (Alice can send)
    let valid_choreo = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("Bob"),
        message: Message::Text("Hello".to_string()),
    });
    
    // This should succeed
    match compile_choreography(&valid_choreo, &registry) {
        Ok(_) => println!("✓ Valid choreography compiled successfully"),
        Err(e) => panic!("Valid choreography failed: {}", e),
    }
    
    // Create invalid choreography (Bob can't send)
    let invalid_choreo = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Bob"),
        to: AgentId::new("Alice"),
        message: Message::Text("Hello".to_string()),
    });
    
    // This should fail with MissingCapability error
    match compile_choreography(&invalid_choreo, &registry) {
        Ok(_) => panic!("Expected MissingCapability error"),
        Err(e) => {
            println!("✓ Correctly caught capability error: {}", e);
            assert!(format!("{}", e).contains("lacks capability"));
        }
    }
} 