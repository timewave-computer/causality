// Integration tests for the Causality-Valence session crate
// Tests all examples end-to-end and verifies system properties

use session::layer2::outcome::{Value, StateLocation};
use session::layer3::agent::{Agent, AgentId};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message, LocalAction};
use session::layer2::effect::{EffectRow, EffectType};
use session::interpreter::Interpreter;

/// Test the hello world example end-to-end
#[test]
fn test_hello_world_integration() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up channels
    interpreter.get_channel_registry().create_channel(
        "Alice→Bob".to_string(),
        10,
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    
    interpreter.get_channel_registry().create_channel(
        "Bob→Alice".to_string(),
        10,
        vec!["Bob".to_string(), "Alice".to_string()],
    );
    
    // Create agents
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    
    let comm_capability = Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    alice.add_capability(comm_capability.clone());
    bob.add_capability(comm_capability);
    
    // Register agents
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    
    // Create choreography
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
    
    // Execute and verify
    let result = interpreter.execute_choreography(&choreography);
    assert!(result.is_ok(), "Hello world choreography should succeed");
    
    // Verify messages were sent
    let channel_registry = interpreter.get_channel_registry();
    let alice_to_bob = channel_registry.get_channel_status("Alice→Bob");
    let bob_to_alice = channel_registry.get_channel_status("Bob→Alice");
    
    assert!(alice_to_bob.is_some());
    assert!(bob_to_alice.is_some());
    
    let (alice_queue, _, _) = alice_to_bob.unwrap();
    let (bob_queue, _, _) = bob_to_alice.unwrap();
    
    assert_eq!(alice_queue, 1, "Alice should have sent 1 message to Bob");
    assert_eq!(bob_queue, 1, "Bob should have sent 1 message to Alice");
}

/// Test the payment protocol with validation
#[test]
fn test_payment_protocol_integration() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up channels
    interpreter.get_channel_registry().create_channel(
        "Alice→Bob".to_string(),
        10,
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    
    interpreter.get_channel_registry().create_channel(
        "Bob→Alice".to_string(),
        10,
        vec!["Bob".to_string(), "Alice".to_string()],
    );
    
    // Create agents with capabilities
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    
    let alice_cap = Capability::new(
        "PaymentRequester".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    let bob_cap = Capability::new(
        "PaymentProvider".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    alice.add_capability(alice_cap);
    bob.add_capability(bob_cap);
    
    // Set initial state
    interpreter.set_state(
        StateLocation("Alice_balance".to_string()),
        Value::Int(50),
    );
    interpreter.set_state(
        StateLocation("Bob_balance".to_string()),
        Value::Int(200),
    );
    
    // Register agents
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    
    // Create payment choreography
    let payment_amount = 100;
    let choreography = Choreography::Sequence(vec![
        // Payment request
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Typed {
                msg_type: "PaymentRequest".to_string(),
                value: Value::String(format!("amount:{}", payment_amount)),
            },
        }),
        
        // Validation
        Choreography::Step(ChoreographyStep::Local {
            agent: AgentId::new("Bob"),
            action: LocalAction::Validate {
                what: format!("payment_request_amount_{}", payment_amount),
            },
        }),
        
        // Payment
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Typed {
                msg_type: "Payment".to_string(),
                value: Value::String(format!("amount:{}", payment_amount)),
            },
        }),
        
        // Receipt
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Typed {
                msg_type: "Receipt".to_string(),
                value: Value::String("confirmed".to_string()),
            },
        }),
    ]);
    
    // Execute and verify
    let result = interpreter.execute_choreography(&choreography);
    assert!(result.is_ok(), "Payment protocol should succeed");
    
    // Verify state is preserved
    let state = interpreter.get_state();
    let alice_balance = state.get(&StateLocation("Alice_balance".to_string()));
    let bob_balance = state.get(&StateLocation("Bob_balance".to_string()));
    
    assert_eq!(alice_balance, Some(&Value::Int(50)), "Alice balance should be preserved");
    assert_eq!(bob_balance, Some(&Value::Int(200)), "Bob balance should be preserved");
    
    // Verify messages were exchanged (2 from Alice, 1 from Bob)
    let channel_registry = interpreter.get_channel_registry();
    let alice_to_bob = channel_registry.get_channel_status("Alice→Bob");
    let bob_to_alice = channel_registry.get_channel_status("Bob→Alice");
    
    let (alice_queue, _, _) = alice_to_bob.unwrap();
    let (bob_queue, _, _) = bob_to_alice.unwrap();
    
    assert_eq!(alice_queue, 2, "Alice should send request and receipt");
    assert_eq!(bob_queue, 1, "Bob should send payment");
}

/// Test atomic swap with parallel execution
#[test]
fn test_atomic_swap_integration() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up all required channels
    let channels = [
        "Alice→Bob", "Bob→Alice", 
        "Alice→Escrow", "Bob→Escrow",
    ];
    
    for channel in &channels {
        interpreter.get_channel_registry().create_channel(
            channel.to_string(),
            10,
            vec!["Alice".to_string(), "Bob".to_string(), "Escrow".to_string()],
        );
    }
    
    // Create agents
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    let mut escrow = Agent::new("Escrow");
    
    let swap_capability = Capability::new(
        "TokenSwapper".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    alice.add_capability(swap_capability.clone());
    bob.add_capability(swap_capability.clone());
    escrow.add_capability(swap_capability);
    
    // Set token balances
    interpreter.set_state(StateLocation("Alice_TokenA".to_string()), Value::Int(50));
    interpreter.set_state(StateLocation("Alice_TokenB".to_string()), Value::Int(0));
    interpreter.set_state(StateLocation("Bob_TokenA".to_string()), Value::Int(0));
    interpreter.set_state(StateLocation("Bob_TokenB".to_string()), Value::Int(75));
    
    // Register agents
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    agent_registry.register(escrow).unwrap();
    
    // Create parallel atomic swap
    let atomic_swap = Choreography::Parallel(vec![
        // Alice's branch
        Choreography::Sequence(vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Escrow"),
                message: Message::Typed {
                    msg_type: "TokenDeposit".to_string(),
                    value: Value::String("token:TokenA,amount:50".to_string()),
                },
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Bob"),
                message: Message::Text("Ready".to_string()),
            }),
        ]),
        
        // Bob's branch
        Choreography::Sequence(vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Escrow"),
                message: Message::Typed {
                    msg_type: "TokenDeposit".to_string(),
                    value: Value::String("token:TokenB,amount:75".to_string()),
                },
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Alice"),
                message: Message::Text("Ready".to_string()),
            }),
        ]),
    ]);
    
    // Execute and verify
    let result = interpreter.execute_choreography(&atomic_swap);
    assert!(result.is_ok(), "Atomic swap should succeed");
    
    // Verify all deposits and confirmations
    let channel_registry = interpreter.get_channel_registry();
    
    let alice_to_escrow = channel_registry.get_channel_status("Alice→Escrow").unwrap();
    let bob_to_escrow = channel_registry.get_channel_status("Bob→Escrow").unwrap();
    let alice_to_bob = channel_registry.get_channel_status("Alice→Bob").unwrap();
    let bob_to_alice = channel_registry.get_channel_status("Bob→Alice").unwrap();
    
    assert_eq!(alice_to_escrow.0, 1, "Alice should deposit to escrow");
    assert_eq!(bob_to_escrow.0, 1, "Bob should deposit to escrow");
    assert_eq!(alice_to_bob.0, 1, "Alice should send ready confirmation");
    assert_eq!(bob_to_alice.0, 1, "Bob should send ready confirmation");
}

/// Test error handling and diagnostics
#[test]
fn test_error_handling() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Try to execute choreography with missing agent
    let choreography = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("NonExistent"),
        to: AgentId::new("Alice"),
        message: Message::Text("Hello".to_string()),
    });
    
    let result = interpreter.execute_choreography(&choreography);
    assert!(result.is_err(), "Should fail with missing agent");
    
    // Check error type and message
    let error = result.unwrap_err();
    let diagnostic = error.get_diagnostic();
    assert!(diagnostic.contains("AgentNotFound"), "Should indicate agent not found");
}

/// Test channel capacity limits
#[test]
fn test_channel_capacity() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Create a channel with capacity 1
    interpreter.get_channel_registry().create_channel(
        "Alice→Bob".to_string(),
        1, // Small capacity
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    
    // Create and register agents
    let mut alice = Agent::new("Alice");
    let comm_cap = Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
        ]),
    );
    alice.add_capability(comm_cap);
    
    // Bob doesn't need any capabilities for receiving, but must be registered
    let bob = Agent::new("Bob");
    
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    
    // Send first message (should succeed)
    let first_send = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("Bob"),
        message: Message::Text("Message1".to_string()),
    });
    
    let result1 = interpreter.execute_choreography(&first_send);
    assert!(result1.is_ok(), "First message should succeed");
    
    // Send second message (should fail due to capacity)
    let second_send = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("Bob"),
        message: Message::Text("Message2".to_string()),
    });
    
    let result2 = interpreter.execute_choreography(&second_send);
    assert!(result2.is_err(), "Second message should fail due to capacity");
    
    let error = result2.unwrap_err();
    let diagnostic = error.get_diagnostic();
    assert!(diagnostic.contains("capacity"), "Should mention capacity issue");
}

/// Test debug and snapshot functionality
#[test]
fn test_debug_functionality() {
    let mut interpreter = Interpreter::new();
    
    // Enable snapshots
    let mut debug_options = session::interpreter::DebugOptions::default();
    debug_options.log_enabled = true;
    debug_options.snapshots_enabled = true;
    interpreter.set_debug_options(debug_options);
    
    // Set up basic choreography
    interpreter.get_channel_registry().create_channel(
        "Alice→Bob".to_string(),
        10,
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    
    let mut alice = Agent::new("Alice");
    let comm_cap = Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
        ]),
    );
    alice.add_capability(comm_cap);
    
    // Bob must be registered for Alice to send to him
    let bob = Agent::new("Bob");
    
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    
    let choreography = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("Bob"),
        message: Message::Text("Debug test".to_string()),
    });
    
    // Execute
    let result = interpreter.execute_choreography(&choreography);
    assert!(result.is_ok(), "Debug choreography should succeed");
    
    // Check that logs and snapshots were created
    let effect_log = interpreter.get_effect_log();
    assert!(!effect_log.is_empty(), "Effect log should not be empty");
    
    let snapshots = interpreter.get_snapshots();
    assert!(!snapshots.is_empty(), "Snapshots should be created");
}

/// Test all capabilities work together
#[test]
fn test_capability_system() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Create agents with different capabilities
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    
    // Alice can only send
    let send_cap = Capability::new(
        "Sender".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
        ]),
    );
    
    // Bob can send and receive
    let full_comm_cap = Capability::new(
        "FullComm".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    alice.add_capability(send_cap);
    bob.add_capability(full_comm_cap);
    
    // Set up channel
    interpreter.get_channel_registry().create_channel(
        "Alice→Bob".to_string(),
        10,
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    
    // Register agents
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    
    // Test that Alice can send (has capability)
    let alice_send = Choreography::Step(ChoreographyStep::Send {
        from: AgentId::new("Alice"),
        to: AgentId::new("Bob"),
        message: Message::Text("Hello".to_string()),
    });
    
    let result = interpreter.execute_choreography(&alice_send);
    assert!(result.is_ok(), "Alice should be able to send");
    
    // Verify the message was sent
    let channel_registry = interpreter.get_channel_registry();
    let status = channel_registry.get_channel_status("Alice→Bob").unwrap();
    assert_eq!(status.0, 1, "Message should be sent");
}

/// Comprehensive end-to-end system test
#[test]
fn test_full_system_integration() {
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up comprehensive test environment
    let channels = ["Alice→Bob", "Bob→Alice", "Alice→Carol", "Carol→Alice"];
    for channel in &channels {
        let parts: Vec<&str> = channel.split('→').collect();
        interpreter.get_channel_registry().create_channel(
            channel.to_string(),
            10,
            vec![parts[0].to_string(), parts[1].to_string()],
        );
    }
    
    // Create three agents with different roles
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    let mut carol = Agent::new("Carol");
    
    let full_cap = Capability::new(
        "FullAgent".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
            ("state_access".to_string(), EffectType::State),
        ]),
    );
    
    alice.add_capability(full_cap.clone());
    bob.add_capability(full_cap.clone());
    carol.add_capability(full_cap);
    
    // Set up initial state
    interpreter.set_state(StateLocation("system_state".to_string()), Value::String("initialized".to_string()));
    
    // Register all agents
    let agent_registry = interpreter.get_agent_registry();
    agent_registry.register(alice).unwrap();
    agent_registry.register(bob).unwrap();
    agent_registry.register(carol).unwrap();
    
    // Create complex multi-party choreography
    let complex_choreography = Choreography::Sequence(vec![
        // Round 1: Alice initiates
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Typed {
                msg_type: "InitRequest".to_string(),
                value: Value::String("start_protocol".to_string()),
            },
        }),
        
        // Round 2: Parallel responses
        Choreography::Parallel(vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Alice"),
                message: Message::Text("Ack from Bob".to_string()),
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Carol"),
                message: Message::Text("Involving Carol".to_string()),
            }),
        ]),
        
        // Round 3: Carol responds and logs
        Choreography::Sequence(vec![
            Choreography::Step(ChoreographyStep::Local {
                agent: AgentId::new("Carol"),
                action: LocalAction::Log("Protocol completed".to_string()),
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Carol"),
                to: AgentId::new("Alice"),
                message: Message::Typed {
                    msg_type: "FinalResponse".to_string(),
                    value: Value::String("protocol_complete".to_string()),
                },
            }),
        ]),
    ]);
    
    // Execute the complex choreography
    let result = interpreter.execute_choreography(&complex_choreography);
    assert!(result.is_ok(), "Complex choreography should succeed");
    
    // Verify all expected communications occurred
    let channel_registry = interpreter.get_channel_registry();
    
    let alice_to_bob = channel_registry.get_channel_status("Alice→Bob").unwrap();
    let bob_to_alice = channel_registry.get_channel_status("Bob→Alice").unwrap();
    let alice_to_carol = channel_registry.get_channel_status("Alice→Carol").unwrap();
    let carol_to_alice = channel_registry.get_channel_status("Carol→Alice").unwrap();
    
    assert_eq!(alice_to_bob.0, 1, "Alice should send init request to Bob");
    assert_eq!(bob_to_alice.0, 1, "Bob should ack to Alice");
    assert_eq!(alice_to_carol.0, 1, "Alice should involve Carol");
    assert_eq!(carol_to_alice.0, 1, "Carol should send final response");
    
    // Verify state is preserved
    let state = interpreter.get_state();
    let system_state = state.get(&StateLocation("system_state".to_string()));
    assert_eq!(system_state, Some(&Value::String("initialized".to_string())));
    
    // Verify Carol's log was created
    let carol_log = state.get(&StateLocation("Carol_log".to_string()));
    assert!(carol_log.is_some(), "Carol should have logged");
    
    // Check effect log completeness
    let effect_log = interpreter.get_effect_log();
    assert!(effect_log.len() > 10, "Should have comprehensive effect log");
    
    println!("✅ Full system integration test passed!");
}
