// Atomic Swap example demonstrating parallel execution in the Causality-Valence architecture
// Alice and Bob simultaneously exchange different tokens in an atomic operation

use session::layer3::agent::{Agent, AgentId};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer2::effect::{EffectRow, EffectType};
use session::layer2::outcome::{Value, StateLocation};
use session::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Atomic Swap Example ===");
    println!("Demonstrating parallel token exchange between Alice and Bob");
    
    // Create interpreter with debug enabled
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up channels for bidirectional communication
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
    
    // For atomic swap, we also need escrow channels
    interpreter.get_channel_registry().create_channel(
        "Alice→Escrow".to_string(),
        10,
        vec!["Alice".to_string(), "Escrow".to_string()],
    );
    
    interpreter.get_channel_registry().create_channel(
        "Bob→Escrow".to_string(),
        10,
        vec!["Bob".to_string(), "Escrow".to_string()],
    );
    
    // Create agents with swap capabilities
    let mut alice = Agent::new("Alice");
    let mut bob = Agent::new("Bob");
    let mut escrow = Agent::new("Escrow");
    
    // Alice can send TokenA and receive TokenB
    let alice_swap = Capability::new(
        "TokenSwapper".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
            ("token_transfer".to_string(), EffectType::State),
            ("swap_initiate".to_string(), EffectType::State),
        ]),
    );
    
    // Bob can send TokenB and receive TokenA
    let bob_swap = Capability::new(
        "TokenSwapper".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
            ("token_transfer".to_string(), EffectType::State),
            ("swap_initiate".to_string(), EffectType::State),
        ]),
    );
    
    // Escrow can hold and transfer both tokens
    let escrow_capability = Capability::new(
        "EscrowService".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
            ("escrow_hold".to_string(), EffectType::State),
            ("escrow_release".to_string(), EffectType::State),
        ]),
    );
    
    alice.add_capability(alice_swap);
    bob.add_capability(bob_swap);
    escrow.add_capability(escrow_capability);
    
    // Set up initial token balances
    interpreter.set_state(
        StateLocation("Alice_TokenA".to_string()),
        Value::Int(50), // Alice has 50 TokenA
    );
    interpreter.set_state(
        StateLocation("Alice_TokenB".to_string()),
        Value::Int(0),  // Alice has 0 TokenB
    );
    interpreter.set_state(
        StateLocation("Bob_TokenA".to_string()),
        Value::Int(0),  // Bob has 0 TokenA
    );
    interpreter.set_state(
        StateLocation("Bob_TokenB".to_string()),
        Value::Int(75), // Bob has 75 TokenB
    );
    
    // Register agents
    let agent_registry = interpreter.get_agent_registry();
    let _ = agent_registry.register(alice);
    let _ = agent_registry.register(bob);
    let _ = agent_registry.register(escrow);
    
    // Define swap parameters
    let alice_offers = 50; // TokenA
    let bob_offers = 75;   // TokenB
    
    // Create atomic swap choreography with parallel execution
    let atomic_swap = Choreography::Parallel(vec![
        // Branch 1: Alice's side of the swap
        Choreography::Sequence(vec![
            // Alice sends her tokens to escrow
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Escrow"),
                message: Message::Typed {
                    msg_type: "TokenDeposit".to_string(),
                    value: Value::String(format!("token:TokenA,amount:{},for_swap_with:Bob", alice_offers)),
                },
            }),
            
            // Alice confirms readiness
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Bob"),
                message: Message::Typed {
                    msg_type: "SwapReady".to_string(),
                    value: Value::String(format!("deposited:TokenA:{}", alice_offers)),
                },
            }),
        ]),
        
        // Branch 2: Bob's side of the swap
        Choreography::Sequence(vec![
            // Bob sends his tokens to escrow
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Escrow"),
                message: Message::Typed {
                    msg_type: "TokenDeposit".to_string(),
                    value: Value::String(format!("token:TokenB,amount:{},for_swap_with:Alice", bob_offers)),
                },
            }),
            
            // Bob confirms readiness
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Alice"),
                message: Message::Typed {
                    msg_type: "SwapReady".to_string(),
                    value: Value::String(format!("deposited:TokenB:{}", bob_offers)),
                },
            }),
        ]),
    ]);
    
    println!("\n--- Executing Atomic Swap Protocol ---");
    println!("Initial State:");
    println!("  Alice: {} TokenA, {} TokenB", alice_offers, 0);
    println!("  Bob: {} TokenA, {} TokenB", 0, bob_offers);
    println!("  Escrow: 0 TokenA, 0 TokenB");
    
    println!("\nSwap Agreement:");
    println!("  Alice offers: {} TokenA", alice_offers);
    println!("  Bob offers: {} TokenB", bob_offers);
    
    println!("\nProtocol Steps (Parallel):");
    println!("  [Alice] → Escrow: Deposit {} TokenA", alice_offers);
    println!("  [Bob] → Escrow: Deposit {} TokenB", bob_offers);
    println!("  [Alice] → Bob: SwapReady confirmation");
    println!("  [Bob] → Alice: SwapReady confirmation");
    
    // Execute the atomic swap choreography
    match interpreter.execute_choreography(&atomic_swap) {
        Ok(()) => {
            println!("\n--- Atomic Swap Completed Successfully! ---");
            
            // Print final state
            interpreter.print_state();
            
            // Show effect log
            println!("\n--- Effect Execution Log ---");
            for (i, effect) in interpreter.get_effect_log().iter().enumerate() {
                println!("  {}: {}", i + 1, effect);
            }
            
            println!("\n--- Swap Verification ---");
            
            // Check that all deposits were made
            let channel_registry = interpreter.get_channel_registry();
            let alice_to_escrow = channel_registry.get_channel_status("Alice→Escrow");
            let bob_to_escrow = channel_registry.get_channel_status("Bob→Escrow");
            let alice_to_bob = channel_registry.get_channel_status("Alice→Bob");
            let bob_to_alice = channel_registry.get_channel_status("Bob→Alice");
            
            if let Some((queue_len, _, _)) = alice_to_escrow {
                println!("✓ Alice → Escrow: {} message(s) (TokenA deposit)", queue_len);
            }
            
            if let Some((queue_len, _, _)) = bob_to_escrow {
                println!("✓ Bob → Escrow: {} message(s) (TokenB deposit)", queue_len);
            }
            
            if let Some((queue_len, _, _)) = alice_to_bob {
                println!("✓ Alice → Bob: {} message(s) (SwapReady confirmation)", queue_len);
            }
            
            if let Some((queue_len, _, _)) = bob_to_alice {
                println!("✓ Bob → Alice: {} message(s) (SwapReady confirmation)", queue_len);
            }
            
            // Check initial balances are still recorded
            let state = interpreter.get_state();
            let alice_token_a = state.get(&StateLocation("Alice_TokenA".to_string()));
            let alice_token_b = state.get(&StateLocation("Alice_TokenB".to_string()));
            let bob_token_a = state.get(&StateLocation("Bob_TokenA".to_string()));
            let bob_token_b = state.get(&StateLocation("Bob_TokenB".to_string()));
            
            println!("\nFinal Balances (before escrow release):");
            if let Some(Value::Int(amount)) = alice_token_a {
                println!("  Alice TokenA: {}", amount);
            }
            if let Some(Value::Int(amount)) = alice_token_b {
                println!("  Alice TokenB: {}", amount);
            }
            if let Some(Value::Int(amount)) = bob_token_a {
                println!("  Bob TokenA: {}", amount);
            }
            if let Some(Value::Int(amount)) = bob_token_b {
                println!("  Bob TokenB: {}", amount);
            }
            
            println!("\n🎉 Atomic swap protocol executed successfully!");
            println!("   Both parties deposited tokens and confirmed readiness");
            println!("   (Note: In a complete implementation, escrow would now release tokens)");
        }
        Err(e) => {
            println!("\n--- Atomic Swap Failed ---");
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
    fn test_atomic_swap() {
        let result = main();
        assert!(result.is_ok(), "Atomic swap should execute successfully");
    }
    
    #[test]
    fn test_parallel_choreography() {
        // Test that we can create parallel choreographies
        let parallel_choreo = Choreography::Parallel(vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Bob"),
                message: Message::Text("Hello".to_string()),
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Alice"),
                message: Message::Text("Hi".to_string()),
            }),
        ]);
        
        // Just test creation
        assert!(matches!(parallel_choreo, Choreography::Parallel(_)));
    }
    
    #[test]
    fn test_token_deposit_message() {
        let deposit_msg = Message::Typed {
            msg_type: "TokenDeposit".to_string(),
            value: Value::String("token:TokenA,amount:50,for_swap_with:Bob".to_string()),
        };
        
        assert!(matches!(deposit_msg, Message::Typed { .. }));
    }
} 