// Example demonstrating effect handlers as natural transformations

use session::layer2::effect::{Effect, EffectRow};
use session::layer2::outcome::{Value, StateLocation};
use session::interpreter::Interpreter;

fn main() {
    println!("=== Effect Handler Natural Transformation Example ===\n");
    
    // Create an interpreter to execute effects
    let mut interpreter = Interpreter::new();
    interpreter.enable_debug();
    
    // Example 1: Simple state operations without handler
    println!("1. Direct state operations:");
    let loc = StateLocation("counter".to_string());
    
    // Write effect
    let write_effect: Effect<(), EffectRow> = Effect::write(loc.clone(), Value::Int(42));
    match interpreter.execute_effect(write_effect) {
        Ok(_) => println!("✓ Write operation completed"),
        Err(e) => println!("✗ Write failed: {}", e),
    }
    
    // Read effect
    let read_effect: Effect<Value, EffectRow> = Effect::read(loc.clone());
    match interpreter.execute_effect(read_effect) {
        Ok(value) => println!("✓ Read value: {}", value),
        Err(e) => println!("✗ Read failed: {}", e),
    }
    
    println!("\nState after operations:");
    interpreter.print_state();
    
    // Example 2: More complex state transformations
    println!("\n2. Complex state transformations:");
    
    // Write sequence of values
    let locations_values = [
        (StateLocation("x".to_string()), Value::Int(10)),
        (StateLocation("y".to_string()), Value::String("hello".to_string())),
        (StateLocation("z".to_string()), Value::Int(20)),
    ];
    
    for (location, value) in locations_values {
        let write_effect = Effect::write(location, value);
        if let Err(e) = interpreter.execute_effect(write_effect) {
            println!("✗ Write failed: {}", e);
        }
    }
    
    // Read all values back
    println!("\n3. Reading back all values:");
    let read_locations = [
        StateLocation("counter".to_string()),
        StateLocation("x".to_string()),
        StateLocation("y".to_string()),
        StateLocation("z".to_string()),
    ];
    
    for location in read_locations {
        let read_effect = Effect::read(location.clone());
        match interpreter.execute_effect(read_effect) {
            Ok(value) => println!("  {}: {}", location.0, value),
            Err(e) => println!("  {}: Error - {}", location.0, e),
        }
    }
    
    // Example 4: Communication effects
    println!("\n4. Communication effects:");
    
    // Send a message
    let send_effect = Effect::send("test-channel".to_string(), Value::String("Hello, World!".to_string()));
    match interpreter.execute_effect(send_effect) {
        Ok(_) => println!("✓ Message sent"),
        Err(e) => println!("✗ Send failed: {}", e),
    }
    
    // Receive the message
    let receive_effect = Effect::receive("test-channel".to_string());
    match interpreter.execute_effect(receive_effect) {
        Ok(value) => println!("✓ Message received: {}", value),
        Err(e) => println!("✗ Receive failed: {}", e),
    }
    
    println!("\nFinal interpreter state:");
    interpreter.print_state();
    
    println!("\n=== Effect Log ===");
    for (i, log_entry) in interpreter.get_effect_log().iter().enumerate() {
        println!("  {}: {}", i + 1, log_entry);
    }
    
    println!("\n=== Example Complete ===");
    println!("Demonstrated natural transformation of effects through interpreter execution");
} 