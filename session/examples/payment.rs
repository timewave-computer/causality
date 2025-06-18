// Payment protocol demonstrating request-response pattern with state updates
// Alice requests payment from Bob, Bob processes and responds

use session::layer3::agent::{Agent, AgentId};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer2::effect::{EffectRow, EffectType};
use session::layer2::outcome::Value;
use session::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Payment Protocol Example ===");
    println!("Demonstrating request-response pattern with payment processing");
    
    // Create interpreter with debug enabled
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up bidirectional communication channels
    interpreter.get_channel_registry().create_channel(
        "Alice→Bob".to_string(),
        10, // capacity
        vec!["Alice".to_string(), "Bob".to_string()], // participants
    );
    
    interpreter.get_channel_registry().create_channel(
        "Bob→Alice".to_string(),
        10, // capacity  
        vec!["Bob".to_string(), "Alice".to_string()], // participants
    );
    
    // Create agents with appropriate capabilities
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    
    let comm_capability = Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ]),
    );
    
    let payment_capability = Capability::new(
        "PaymentProcessing".to_string(),
        EffectRow::from_effects(vec![
            ("state_read".to_string(), EffectType::State),
            ("state_write".to_string(), EffectType::State),
            ("payment_validate".to_string(), EffectType::State),
        ]),
    );
    
    alice.add_capability(comm_capability.clone());
    alice.add_capability(payment_capability.clone());
    bob.add_capability(comm_capability);
    bob.add_capability(payment_capability);
    
    // Set up initial account balances
    interpreter.set_state(
        session::layer2::outcome::StateLocation("Alice_balance".to_string()),
        Value::Int(1000),
    );
    interpreter.set_state(
        session::layer2::outcome::StateLocation("Bob_balance".to_string()),
        Value::Int(500),
    );
    interpreter.set_state(
        session::layer2::outcome::StateLocation("payment_id_counter".to_string()),
        Value::Int(1001),
    );
    
    // Register agents
    interpreter.register_agent(alice)?;
    interpreter.register_agent(bob)?;
    
    // Create payment protocol choreography
    let choreography = Choreography::Sequence(vec![
        // Step 1: Alice sends payment request to Bob
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("payment_request".to_string())),
                ("amount".to_string(), Value::Int(100)),
                ("currency".to_string(), Value::String("USD".to_string())),
                ("reason".to_string(), Value::String("Service payment".to_string())),
                ("request_id".to_string(), Value::String("pay_001".to_string())),
            ])),
        }),
        
        // Step 2: Bob sends payment confirmation
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("payment_confirmation".to_string())),
                ("amount".to_string(), Value::Int(100)),
                ("status".to_string(), Value::String("approved".to_string())),
                ("transaction_id".to_string(), Value::String("txn_001".to_string())),
                ("request_id".to_string(), Value::String("pay_001".to_string())),
            ])),
        }),
        
        // Step 3: Alice sends payment receipt acknowledgment
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("receipt_acknowledgment".to_string())),
                ("transaction_id".to_string(), Value::String("txn_001".to_string())),
                ("timestamp".to_string(), Value::String("2024-01-01T10:00:00Z".to_string())),
            ])),
        }),
    ]);
    
    println!("\n--- Executing payment protocol ---");
    println!("Protocol Steps:");
    println!("  1. Alice → Bob: Payment request ($100)");
    println!("  2. Bob → Alice: Payment confirmation");
    println!("  3. Alice → Bob: Receipt acknowledgment");
    
    println!("\nInitial State:");
    println!("  Alice balance: $1000");
    println!("  Bob balance: $500");
    
    // Execute the choreography
    match interpreter.execute_choreography(&choreography) {
        Ok(outcome) => {
            println!("\n--- Payment protocol completed successfully! ---");
            println!("Outcome: {:?}", outcome);
            
            // Print final state
            interpreter.print_state();
            
            // Show execution trace
            println!("\n--- Execution Trace ---");
            for (i, effect) in interpreter.get_trace().iter().enumerate() {
                println!("  {}: {}", i + 1, effect);
            }
            
            println!("\n--- Verification ---");
            // Check that messages were exchanged
            let channel_registry = interpreter.get_channel_registry();
            let alice_to_bob_status = channel_registry.get_channel_status("Alice→Bob");
            let bob_to_alice_status = channel_registry.get_channel_status("Bob→Alice");
            
            if let Some(status) = alice_to_bob_status {
                println!("✓ Alice to Bob channel has {} message(s)", status.current_size);
            }
            
            if let Some(status) = bob_to_alice_status {
                println!("✓ Bob to Alice channel has {} message(s)", status.current_size);
            }
            
            // Check final balances (unchanged in this protocol version)
            let state = interpreter.get_state();
            let alice_balance = state.get(&session::layer2::outcome::StateLocation("Alice_balance".to_string()));
            let bob_balance = state.get(&session::layer2::outcome::StateLocation("Bob_balance".to_string()));
            
            println!("\nFinal Balances:");
            if let Some(Value::Int(amount)) = alice_balance {
                println!("  Alice: ${}", amount);
            }
            if let Some(Value::Int(amount)) = bob_balance {
                println!("  Bob: ${}", amount);
            }
            
            println!("\n🎉 Payment protocol executed successfully!");
            
        }
        Err(e) => {
            println!("\n--- Payment protocol failed ---");
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
    fn test_payment_choreography() {
        let result = main();
        assert!(result.is_ok(), "Payment protocol should execute successfully");
    }
    
    #[test]
    fn test_payment_message_structure() {
        // Test payment request message structure
        let payment_request = Message::Data(Value::Struct(vec![
            ("type".to_string(), Value::String("payment_request".to_string())),
            ("amount".to_string(), Value::Int(100)),
            ("currency".to_string(), Value::String("USD".to_string())),
        ]));
        
        assert!(matches!(payment_request, Message::Data(_)));
    }
    
    #[test] 
    fn test_payment_setup() {
        let mut interpreter = Interpreter::new();
        interpreter.enable_debug();
        
        // Test channel setup
        interpreter.get_channel_registry().create_channel(
            "payment_test".to_string(),
            5,
            vec!["Alice".to_string(), "Bob".to_string()],
        );
        
        // Test payment message
        let payment_msg = Value::Struct(vec![
            ("amount".to_string(), Value::Int(50)),
            ("currency".to_string(), Value::String("USD".to_string())),
        ]);
        
        let send_result = interpreter.get_channel_registry().send("payment_test", payment_msg);
        assert!(send_result.is_ok());
        
        let receive_result = interpreter.get_channel_registry().receive("payment_test");
        assert!(receive_result.is_ok());
        assert!(receive_result.unwrap().is_some());
    }
} 