// Hello World example demonstrating basic message passing in the Causality-Valence architecture
// Alice sends "Hello" to Bob, Bob sends "World" back to Alice

use session::layer3::agent::{Agent, AgentId};
use session::layer3::capability::Capability;
use session::layer3::choreography::{Choreography, ChoreographyStep, Message};
use session::layer2::effect::{EffectRow, EffectType};
use session::layer2::outcome::Value;
use session::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hello World Example ===");
    println!("Demonstrating basic message passing between Alice and Bob");
    
    // Create interpreter with debug enabled
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Set up channel for communication
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
    
    // Create agents with communication capabilities
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
    interpreter.register_agent(alice)?;
    interpreter.register_agent(bob)?;
    
    // Create the choreography
    let choreography = Choreography::Sequence(vec![
        // Step 1: Alice sends "Hello" to Bob
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello".to_string()),
        }),
        
        // Step 2: Bob sends "World" to Alice
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Text("World".to_string()),
        }),
    ]);
    
    println!("\n--- Executing choreography ---");
    println!("Choreography: HelloWorld");
    println!("  Step 1: Alice sends 'Hello' to Bob");
    println!("  Step 2: Bob sends 'World' to Alice");
    
    // Execute the choreography
    match interpreter.execute_choreography(&choreography) {
        Ok(outcome) => {
            println!("\n--- Execution completed successfully! ---");
            println!("Outcome: {:?}", outcome);
            
            // Print final state
            interpreter.print_state();
            
            // Show effect log
            println!("\n--- Effect Log ---");
            for effect in interpreter.get_trace() {
                println!("  {}", effect);
            }
            
            println!("\n--- Verification ---");
            // Check that messages were sent
            let channel_registry = interpreter.get_channel_registry();
            let alice_to_bob_status = channel_registry.get_channel_status("Alice→Bob");
            let bob_to_alice_status = channel_registry.get_channel_status("Bob→Alice");
                
            if let Some(status) = alice_to_bob_status {
                println!("✓ Alice to Bob channel has {} message(s)", status.current_size);
            }
            
            if let Some(status) = bob_to_alice_status {
                println!("✓ Bob to Alice channel has {} message(s)", status.current_size);
            }
            
            println!("\n🎉 Hello World choreography executed successfully!");
            
        }
        Err(e) => {
            println!("\n--- Execution failed ---");
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
    fn test_hello_world_choreography() {
        let result = main();
        assert!(result.is_ok(), "Hello world example should execute successfully");
    }
    
    #[test] 
    fn test_hello_world_step_by_step() {
        let mut interpreter = Interpreter::new();
        interpreter.enable_debug();
        
        // Set up channels
        interpreter.get_channel_registry().create_channel(
            "test_channel".to_string(),
            5,
            vec!["Alice".to_string(), "Bob".to_string()],
        );
        
        // Test individual send
        let send_result = interpreter.get_channel_registry().send(
            "test_channel",
            Value::String("Test message".to_string()),
        );
        assert!(send_result.is_ok());
        
        // Test receive
        let receive_result = interpreter.get_channel_registry().receive("test_channel");
        assert!(receive_result.is_ok());
        assert!(receive_result.unwrap().is_some());
    }
} 