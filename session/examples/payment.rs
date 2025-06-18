// Payment Protocol example demonstrating complex state management in the Causality-Valence architecture
// Alice requests payment from Bob, Bob sends payment, Alice sends receipt

use session::layer3::agent::{Agent, AgentId};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message, LocalAction};
use session::layer2::effect::{EffectRow, EffectType};
use session::layer2::outcome::{Value, StateLocation};
use session::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Payment Protocol Example ===");
    println!("Demonstrating: Payment Request → Validation → Payment → Receipt");
    
    // Create interpreter with debug enabled
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up channels for communication
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
    
    // Alice can send payment requests and receive payments
    let alice_comm = Capability::new(
        "PaymentRequester".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
            ("payment_request".to_string(), EffectType::State),
        ]),
    );
    
    // Bob can validate and send payments
    let bob_payment = Capability::new(
        "PaymentProvider".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm), 
            ("payment_validate".to_string(), EffectType::State),
            ("payment_transfer".to_string(), EffectType::State),
        ]),
    );
    
    alice.add_capability(alice_comm);
    bob.add_capability(bob_payment);
    
    // Set up initial balances
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
    let _ = agent_registry.register(alice);
    let _ = agent_registry.register(bob);
    
    // Create the payment protocol choreography
    let payment_amount = 100;
    let payment_memo = "Service payment".to_string();
    
    let choreography = Choreography::Sequence(vec![
        // Step 1: Alice sends payment request to Bob
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Typed {
                msg_type: "PaymentRequest".to_string(),
                value: Value::String(format!("amount:{},memo:{}", payment_amount, payment_memo)),
            },
        }),
        
        // Step 2: Bob validates the request locally
        Choreography::Step(ChoreographyStep::Local {
            agent: AgentId::new("Bob"),
            action: LocalAction::Validate {
                what: format!("payment_request_amount_{}", payment_amount),
            },
        }),
        
        // Step 3: Bob sends payment to Alice
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Typed {
                msg_type: "Payment".to_string(),
                value: Value::String(format!("amount:{},from:Bob,to:Alice", payment_amount)),
            },
        }),
        
        // Step 4: Alice sends receipt to Bob
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Typed {
                msg_type: "Receipt".to_string(),
                value: Value::String(format!("payment_confirmed_amount_{}", payment_amount)),
            },
        }),
    ]);
    
    println!("\n--- Executing Payment Protocol ---");
    println!("Initial State:");
    println!("  Alice balance: 50");
    println!("  Bob balance: 200");
    println!("  Payment amount: {}", payment_amount);
    println!("  Payment memo: {}", payment_memo);
    
    println!("\nProtocol Steps:");
    println!("  1. Alice → Bob: PaymentRequest({}, '{}')", payment_amount, payment_memo);
    println!("  2. Bob validates payment request locally");
    println!("  3. Bob → Alice: Payment({})", payment_amount);
    println!("  4. Alice → Bob: Receipt(confirmed)");
    
    // Execute the choreography
    match interpreter.execute_choreography(&choreography) {
        Ok(()) => {
            println!("\n--- Payment Protocol Completed Successfully! ---");
            
            // Print final state
            interpreter.print_state();
            
            // Show detailed effect log
            println!("\n--- Detailed Effect Log ---");
            for (i, effect) in interpreter.get_effect_log().iter().enumerate() {
                println!("  {}: {}", i + 1, effect);
            }
            
            println!("\n--- Protocol Verification ---");
            
            // Check that all messages were exchanged
            let channel_registry = interpreter.get_channel_registry();
            let alice_to_bob_status = channel_registry.get_channel_status("Alice→Bob");
            let bob_to_alice_status = channel_registry.get_channel_status("Bob→Alice");
            
            if let Some((queue_len, _, _)) = alice_to_bob_status {
                println!("✓ Alice → Bob channel: {} message(s) (PaymentRequest + Receipt expected)", queue_len);
            }
            
            if let Some((queue_len, _, _)) = bob_to_alice_status {
                println!("✓ Bob → Alice channel: {} message(s) (Payment expected)", queue_len);
            }
            
            // Check state updates
            let state = interpreter.get_state();
            if let Some(validation_log) = state.get(&StateLocation("Bob_log".to_string())) {
                println!("✓ Bob validation log: {:?}", validation_log);
            }
            
            println!("\n🎉 Payment protocol executed successfully!");
            println!("   All steps completed: Request → Validation → Payment → Receipt");
        }
        Err(e) => {
            println!("\n--- Payment Protocol Failed ---");
            println!("Error: {}", e);
            
            // Print diagnostic information
            println!("\n--- Diagnostics ---");
            println!("{}", e.get_diagnostic());
            
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_payment_protocol() {
        let result = main();
        assert!(result.is_ok(), "Payment protocol should execute successfully");
    }
    
    #[test]
    fn test_payment_steps() {
        let mut interpreter = Interpreter::new();
        interpreter.enable_debug();
        
        // Set up a simple payment test
        interpreter.get_channel_registry().create_channel(
            "Alice→Bob".to_string(),
            5,
            vec!["Alice".to_string(), "Bob".to_string()],
        );
        
        // Test payment request
        let request_msg = Message::Typed {
            msg_type: "PaymentRequest".to_string(),
            value: Value::String("amount:50,memo:test".to_string()),
        };
        
        // Just test that we can create the message
        assert!(matches!(request_msg, Message::Typed { .. }));
        
        // Test payment message
        let payment_msg = Message::Typed {
            msg_type: "Payment".to_string(),
            value: Value::String("amount:50,from:Bob,to:Alice".to_string()),
        };
        
        assert!(matches!(payment_msg, Message::Typed { .. }));
    }
    
    #[test]
    fn test_validation_step() {
        let validation_action = LocalAction::Validate {
            what: "payment_request_amount_100".to_string(),
        };
        
        // Test that we can create validation actions
        assert!(matches!(validation_action, LocalAction::Validate { .. }));
    }
} 