// Example: Layer 3 Choreography - Multi-party payment protocol

use session::layer3::{Agent, AgentId, Capability, Choreography};
use session::layer3::agent::AgentRegistry;
use session::layer3::choreography::{ChoreographyParser, ChoreographyStep, Message, LocalAction};
use session::layer3::compiler::compile_choreography;
use session::interpreter::Interpreter;

fn main() {
    println!("=== Layer 3: Choreography Example ===\n");
    
    // Create agents
    let mut registry = AgentRegistry::new();
    
    // Alice can transfer tokens and communicate
    let mut alice = Agent::new("Alice");
    alice.add_capability(Capability::data_access(
        vec!["balance".to_string()],
        vec![]
    ));
    // Add communication capability
    alice.add_capability(Capability::new(
        "Communication".to_string(),
        session::layer2::effect::EffectRow::from_effects(vec![
            ("comm_send".to_string(), session::layer2::effect::EffectType::Comm),
        ])
    ));
    alice.state.set("balance".to_string(), "1000".to_string());
    
    // Bob can receive and validate and communicate
    let mut bob = Agent::new("Bob");
    bob.add_capability(Capability::data_access(
        vec!["payment_requests".to_string()],
        vec![]
    ));
    // Add communication capability  
    bob.add_capability(Capability::new(
        "Communication".to_string(),
        session::layer2::effect::EffectRow::from_effects(vec![
            ("comm_send".to_string(), session::layer2::effect::EffectType::Comm),
        ])
    ));
    bob.state.set("balance".to_string(), "0".to_string());
    
    // Register agents
    registry.register(alice).unwrap();
    registry.register(bob).unwrap();
    
    // Define choreography
    let choreography_text = r#"
        Alice sends PaymentRequest(100) to Bob
        Bob validates payment
        Bob sends PaymentApproved to Alice
        Alice sends Payment(100) to Bob
        Alice logs Payment sent
        Bob logs Payment received
    "#;
    
    println!("Choreography:");
    println!("{}", choreography_text);
    println!();
    
    // Parse choreography
    let parser = ChoreographyParser::new();
    let choreo = parser.parse_simple(choreography_text).unwrap();
    
    println!("Parsed choreography: {:?}", choreo);
    println!();
    
    // Compile to Layer 2 effects
    match compile_choreography(&choreo, &registry) {
        Ok(effects) => {
            println!("Compiled to {} effects successfully!", effects.len());
            
            // Execute the effects using interpreter
            let mut interpreter = Interpreter::new();
            interpreter.enable_debug();
            
            for (i, effect) in effects.into_iter().enumerate() {
                println!("Executing effect {}: ", i + 1);
                match interpreter.execute_effect(effect) {
                    Ok(_) => println!("  Success"),
                    Err(e) => println!("  Error: {}", e),
                }
            }
            
            println!("\nFinal interpreter state:");
            interpreter.print_state();
        }
        Err(e) => {
            println!("Compilation error: {}", e);
        }
    }
    
    // Show more complex choreography with parallel execution
    println!("\n=== Complex Choreography Example ===\n");
    
    let complex_choreo = Choreography::Parallel(vec![
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello Bob".to_string()),
        }),
        Choreography::Step(ChoreographyStep::Send {
            from: AgentId::new("Bob"),
            to: AgentId::new("Alice"),
            message: Message::Text("Hello Alice".to_string()),
        }),
    ]);
    
    println!("Parallel choreography: {:?}", complex_choreo);
    
    match compile_choreography(&complex_choreo, &registry) {
        Ok(effects) => {
            println!("Parallel choreography compiled successfully! {} effects", effects.len());
            
            // Execute parallel choreography
            let mut interpreter = Interpreter::new();
            interpreter.enable_debug();
            
            for effect in effects {
                let _ = interpreter.execute_effect(effect);
            }
            
            println!("Parallel execution completed");
        }
        Err(e) => println!("Error: {}", e),
    }
    
    // Demonstrate capability checking
    println!("\n=== Capability Example ===\n");
    
    // Try to have Bob transfer (he doesn't have the capability)
    let unauthorized_step = ChoreographyStep::Local {
        agent: AgentId::new("Bob"),
        action: LocalAction::Compute {
            operation: "transfer".to_string(),
            args: vec!["100".to_string(), "Carol".to_string()],
        },
    };
    
    let unauthorized_choreo = Choreography::Step(unauthorized_step);
    
    // This should succeed because we're not checking transfer capability for compute
    match compile_choreography(&unauthorized_choreo, &registry) {
        Ok(_) => println!("Compute action compiled (no capability check for compute)"),
        Err(e) => println!("Error: {}", e),
    }
    
    // Try spawning without capability
    let spawn_step = ChoreographyStep::Spawn {
        creator: AgentId::new("Bob"),
        new_agent: AgentId::new("Worker"),
        agent_type: "compute".to_string(),
    };
    
    let spawn_choreo = Choreography::Step(spawn_step);
    
    match compile_choreography(&spawn_choreo, &registry) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected capability error: {}", e),
    }
} 