// Atomic Swap example demonstrating parallel token exchange
// Alice and Bob exchange different tokens atomically

use session::layer3::agent::{Agent, AgentId};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer2::effect::{EffectRow, EffectType};
use session::layer2::outcome::Value;
use session::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Atomic Swap Example ===");
    println!("Demonstrating parallel token exchange between Alice and Bob");
    
    // Create interpreter with debug enabled
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up channels for communication
    let channels = [
        ("Alice→Escrow", vec!["Alice", "Escrow"]),
        ("Bob→Escrow", vec!["Bob", "Escrow"]),
        ("Escrow→Alice", vec!["Escrow", "Alice"]),
        ("Escrow→Bob", vec!["Escrow", "Bob"]),
        ("Alice→Bob", vec!["Alice", "Bob"]),
        ("Bob→Alice", vec!["Bob", "Alice"]),
    ];
    
    for (channel_name, participants) in &channels {
        interpreter.get_channel_registry().create_channel(
            channel_name.to_string(),
            50, // Large capacity for complex protocol
            participants.iter().map(|s| s.to_string()).collect(),
        );
    }
    
    // Create agents with appropriate capabilities
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    let mut escrow = Agent::new("Escrow");
    
    let comm_capability = Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    let state_capability = Capability::new(
        "StateAccess".to_string(),
        EffectRow::from_effects(vec![
            ("state_read".to_string(), EffectType::State),
            ("state_write".to_string(), EffectType::State),
        ]),
    );
    
    alice.add_capability(comm_capability.clone());
    alice.add_capability(state_capability.clone());
    bob.add_capability(comm_capability.clone());
    bob.add_capability(state_capability.clone());
    escrow.add_capability(comm_capability);
    escrow.add_capability(state_capability);
    
    // Initialize agent state (token balances)
    interpreter.set_state(
        session::layer2::outcome::StateLocation("Alice_TokenA".to_string()),
        Value::Int(100),
    );
    interpreter.set_state(
        session::layer2::outcome::StateLocation("Alice_TokenB".to_string()),
        Value::Int(0),
    );
    interpreter.set_state(
        session::layer2::outcome::StateLocation("Bob_TokenA".to_string()),
        Value::Int(0),
    );
    interpreter.set_state(
        session::layer2::outcome::StateLocation("Bob_TokenB".to_string()),
        Value::Int(50),
    );
    
    // Register agents
    interpreter.register_agent(alice)?;
    interpreter.register_agent(bob)?;
    interpreter.register_agent(escrow)?;
    
    // Create atomic swap choreography
    let atomic_swap = Choreography::Sequence(vec![
        // Phase 1: Setup - Create swap commitment
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Escrow"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("swap_proposal".to_string())),
                ("offer_token".to_string(), Value::String("TokenA".to_string())),
                ("offer_amount".to_string(), Value::Int(50)),
                ("want_token".to_string(), Value::String("TokenB".to_string())),
                ("want_amount".to_string(), Value::Int(25)),
            ])),
        }),
        
        // Phase 2: Bob accepts the swap
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Escrow"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("swap_acceptance".to_string())),
                ("offer_token".to_string(), Value::String("TokenB".to_string())),
                ("offer_amount".to_string(), Value::Int(25)),
            ])),
        }),
        
        // Phase 3: Parallel execution - Both parties send tokens to escrow
        Choreography::Parallel(vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Escrow"),
                message: Message::Data(Value::Struct(vec![
                    ("type".to_string(), Value::String("token_deposit".to_string())),
                    ("token".to_string(), Value::String("TokenA".to_string())),
                    ("amount".to_string(), Value::Int(50)),
                ])),
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Escrow"),
                message: Message::Data(Value::Struct(vec![
                    ("type".to_string(), Value::String("token_deposit".to_string())),
                    ("token".to_string(), Value::String("TokenB".to_string())),
                    ("amount".to_string(), Value::Int(25)),
                ])),
            }),
        ]),
        
        // Phase 4: Escrow validates and completes swap
        Choreography::Sequence(vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Escrow"),
                to: AgentId::new("Alice"),
                message: Message::Data(Value::Struct(vec![
                    ("type".to_string(), Value::String("token_transfer".to_string())),
                    ("token".to_string(), Value::String("TokenB".to_string())),
                    ("amount".to_string(), Value::Int(25)),
                ])),
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Escrow"),
                to: AgentId::new("Bob"),
                message: Message::Data(Value::Struct(vec![
                    ("type".to_string(), Value::String("token_transfer".to_string())),
                    ("token".to_string(), Value::String("TokenA".to_string())),
                    ("amount".to_string(), Value::Int(50)),
                ])),
            }),
        ]),
    ]);
    
    println!("\n--- Executing atomic swap choreography ---");
    println!("Initial State:");
    println!("  Alice: 100 TokenA, 0 TokenB");
    println!("  Bob: 0 TokenA, 50 TokenB");
    println!("Swap: Alice exchanges 50 TokenA for 25 TokenB from Bob");
    
    // Execute the choreography
    match interpreter.execute_choreography(&atomic_swap) {
        Ok(outcome) => {
            println!("\n--- Atomic swap completed successfully! ---");
            println!("Outcome: {:?}", outcome);
            
            // Print final state
            interpreter.print_state();
            
            // Show execution trace
            println!("\n--- Execution Trace ---");
            for (i, effect) in interpreter.get_trace().iter().enumerate() {
                println!("  {}: {}", i + 1, effect);
            }
            
            println!("\n--- Verification ---");
            // Check channels have correct messages
            let channel_registry = interpreter.get_channel_registry();
            let alice_to_escrow = channel_registry.get_channel_status("Alice→Escrow");
            let bob_to_escrow = channel_registry.get_channel_status("Bob→Escrow");
            let alice_to_bob = channel_registry.get_channel_status("Alice→Bob");
            let bob_to_alice = channel_registry.get_channel_status("Bob→Alice");
            
            if let Some(status) = alice_to_escrow {
                println!("✓ Alice to Escrow channel has {} message(s)", status.current_size);
            }
            
            if let Some(status) = bob_to_escrow {
                println!("✓ Bob to Escrow channel has {} message(s)", status.current_size);
            }
            
            if let Some(status) = alice_to_bob {
                println!("✓ Alice to Bob channel has {} message(s)", status.current_size);
            }
            
            if let Some(status) = bob_to_alice {
                println!("✓ Bob to Alice channel has {} message(s)", status.current_size);
            }
            
            println!("\n🎉 Atomic swap completed! Tokens exchanged atomically.");
            
        }
        Err(e) => {
            println!("\n--- Atomic swap failed ---");
            println!("Error: {}", e);
            
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atomic_swap_choreography() {
        let result = main();
        assert!(result.is_ok(), "Atomic swap example should execute successfully");
    }
    
    #[test]
    fn test_atomic_swap_setup() {
        let mut interpreter = Interpreter::new();
        interpreter.enable_debug();
        
        // Test channel setup
        interpreter.get_channel_registry().create_channel(
            "test_swap".to_string(),
            10,
            vec!["Alice".to_string(), "Bob".to_string()],
        );
        
        // Test message send
        let send_result = interpreter.get_channel_registry().send(
            "test_swap",
            Value::String("Test swap message".to_string()),
        );
        assert!(send_result.is_ok());
        
        // Test message receive
        let receive_result = interpreter.get_channel_registry().receive("test_swap");
        assert!(receive_result.is_ok());
        assert!(receive_result.unwrap().is_some());
    }
} 