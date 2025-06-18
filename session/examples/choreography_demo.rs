// Example: Layer 3 Choreography - Multi-party payment protocol

use session::layer3::agent::{Agent, AgentId, AgentRegistry};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer3::compiler::compile_choreography;
use session::layer2::effect::{EffectRow, EffectType};
use session::layer2::outcome::Value;
use session::interpreter::Interpreter;

fn main() {
    println!("=== Layer 3: Choreography Example ===\n");
    
    // Create agents
    let mut registry = AgentRegistry::new();
    
    // Alice can transfer tokens and communicate
    let mut alice = Agent::new("Alice");
    alice.add_capability(Capability::new(
        "StateAccess".to_string(),
        EffectRow::from_effects(vec![
            ("state_read".to_string(), EffectType::State),
            ("state_write".to_string(), EffectType::State),
        ])
    ));
    // Add communication capability
    alice.add_capability(Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
        ])
    ));
    
    // Bob can receive and validate and communicate
    let mut bob = Agent::new("Bob");
    bob.add_capability(Capability::new(
        "StateAccess".to_string(),
        EffectRow::from_effects(vec![
            ("state_read".to_string(), EffectType::State),
            ("state_write".to_string(), EffectType::State),
        ])
    ));
    // Add communication capability  
    bob.add_capability(Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
        ])
    ));
    
    // Register agents
    registry.register(alice).unwrap();
    registry.register(bob).unwrap();
    
    // Define choreography manually (no parser needed)
    let choreography = Choreography::Sequence(vec![
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("PaymentRequest".to_string())),
                ("amount".to_string(), Value::Int(100)),
            ])),
        }),
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("PaymentApproved".to_string())),
                ("amount".to_string(), Value::Int(100)),
            ])),
        }),
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Data(Value::Struct(vec![
                ("type".to_string(), Value::String("Payment".to_string())),
                ("amount".to_string(), Value::Int(100)),
            ])),
        }),
    ]);
    
    println!("Choreography created: Multi-party Payment Protocol");
    println!("Steps:");
    println!("  1. Alice → Bob: PaymentRequest(100)");
    println!("  2. Bob → Alice: PaymentApproved");
    println!("  3. Alice → Bob: Payment(100)");
    
    // Compile choreography to effects
    match compile_choreography(&choreography, &registry) {
        Ok(effects) => {
            println!("\n✅ Choreography compiled successfully!");
            println!("Generated {} effects", effects.len());
            
            // Create interpreter and execute
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
            
            // Execute choreography
            match interpreter.execute_choreography(&choreography) {
                Ok(outcome) => {
                    println!("\n✅ Choreography executed successfully!");
                    println!("Outcome: {:?}", outcome);
                    interpreter.print_state();
                }
                Err(e) => {
                    println!("\n❌ Choreography execution failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("\n❌ Choreography compilation failed: {}", e);
        }
    }
    
    // Demonstrate capability checking
    println!("\n=== Capability Checking Demo ===");
    
    // Try to create an agent without proper capabilities
    let charlie = Agent::new("Charlie"); // No capabilities
    registry.register(charlie).unwrap();
    
    // Try to use Charlie in a send operation (should fail)
    let unauthorized_step = ChoreographyStep::Send {
        from: AgentId::new("Charlie"),
        to: AgentId::new("Bob"),
        message: Message::Text("Unauthorized message".to_string()),
    };
    
    let unauthorized_choreo = Choreography::Step(unauthorized_step);
    
    match compile_choreography(&unauthorized_choreo, &registry) {
        Ok(_) => println!("❌ Unexpected: Unauthorized choreography was allowed"),
        Err(e) => println!("✅ Correctly rejected unauthorized choreography: {}", e),
    }
    
    // Demonstrate agent spawning
    println!("\n=== Agent Spawning Demo ===");
    
    let new_agent = Agent::new("Worker1");
    let spawn_step = ChoreographyStep::Spawn {
        parent: AgentId::new("Bob"),
        agent: new_agent,
    };
    
    let spawn_choreo = Choreography::Step(spawn_step);
    
    match compile_choreography(&spawn_choreo, &registry) {
        Ok(effects) => {
            println!("✅ Agent spawning choreography compiled ({} effects)", effects.len());
        }
        Err(e) => {
            println!("ℹ️  Agent spawning failed (expected without spawn capability): {}", e);
        }
    }
    
    println!("\n=== Summary ===");
    println!("Demonstrated:");
    println!("• Multi-step choreography creation and compilation");
    println!("• Effect generation from choreography steps");
    println!("• Capability-based access control");
    println!("• Agent spawning operations");
    println!("• Error handling for unauthorized operations");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_choreography_demo() {
        // This test just ensures the demo runs without panicking
        main();
    }
    
    #[test]
    fn test_choreography_compilation() {
        let mut registry = AgentRegistry::new();
        
        let mut alice = Agent::new("Alice");
        alice.add_capability(Capability::new(
            "Communication".to_string(),
            EffectRow::from_effects(vec![
                ("comm_send".to_string(), EffectType::Comm),
            ])
        ));
        
        let bob = Agent::new("Bob");
        
        registry.register(alice).unwrap();
        registry.register(bob).unwrap();
        
        let simple_choreo = Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Test".to_string()),
        });
        
        let result = compile_choreography(&simple_choreo, &registry);
        assert!(result.is_ok(), "Simple choreography should compile");
    }
    
    #[test]
    fn test_capability_enforcement() {
        let mut registry = AgentRegistry::new();
        
        // Agent without capabilities
        let alice = Agent::new("Alice");
        let bob = Agent::new("Bob");
        
        registry.register(alice).unwrap();
        registry.register(bob).unwrap();
        
        let unauthorized_choreo = Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Should fail".to_string()),
        });
        
        let result = compile_choreography(&unauthorized_choreo, &registry);
        assert!(result.is_err(), "Unauthorized choreography should fail");
    }
} 