//! Demonstration of Layer 2 Effect Handler System
//!
//! This example shows how to:
//! 1. Create effect handlers for different operations
//! 2. Register them in a handler registry
//! 3. Evaluate intents using the registered handlers
//! 4. Apply basic constraint validation

use causality_core::{
    effect::{
        handler_registry::{EffectHandlerRegistry, SimpleEffectHandler, EffectHandler},
        intent_evaluator::IntentEvaluator,
        synthesis::FlowSynthesizer,
        intent::{Intent, Constraint},
    },
    lambda::base::{Value, Location},
    system::error::Result,
};
use std::sync::Arc;

/// Example effect handler for logging operations
struct LogHandler;

impl EffectHandler for LogHandler {
    fn execute(&self, params: Vec<Value>) -> Result<Value> {
        if let Some(Value::Symbol(msg)) = params.first() {
            println!("LOG: {}", msg.as_str());
            Ok(Value::Unit)
        } else {
            println!("LOG: (no message)");
            Ok(Value::Unit)
        }
    }
    
    fn effect_tag(&self) -> &str {
        "log"
    }
}

/// Example effect handler for computation operations
struct ComputeHandler;

impl EffectHandler for ComputeHandler {
    fn execute(&self, params: Vec<Value>) -> Result<Value> {
        match params.as_slice() {
            [Value::Int(a), Value::Int(b)] => {
                let result = a + b;
                println!("COMPUTE: {} + {} = {}", a, b, result);
                Ok(Value::Int(result))
            }
            _ => {
                println!("COMPUTE: Invalid parameters for addition");
                Ok(Value::Unit)
            }
        }
    }
    
    fn effect_tag(&self) -> &str {
        "compute"
    }
}

fn main() -> Result<()> {
    println!("=== Causality Layer 2 Effect Handler Demo ===\n");
    
    // 1. Create and configure handler registry
    println!("1. Setting up effect handler registry...");
    let registry = EffectHandlerRegistry::new();
    
    // Register some basic handlers
    let log_handler = Arc::new(LogHandler);
    let compute_handler = Arc::new(ComputeHandler);
    
    registry.register_handler(log_handler)?;
    registry.register_handler(compute_handler)?;
    
    // Register additional handlers using the SimpleEffectHandler
    let greet_handler = Arc::new(SimpleEffectHandler::new(
        "greet".to_string(),
        |params| {
            if let Some(Value::Symbol(name)) = params.first() {
                println!("GREET: Hello, {}!", name.as_str());
            } else {
                println!("GREET: Hello, world!");
            }
            Ok(Value::Unit)
        },
    ));
    
    registry.register_handler(greet_handler)?;
    
    println!("   Registered effects: {:?}", registry.list_effects());
    println!();
    
    // 2. Test direct effect execution
    println!("2. Testing direct effect execution...");
    
    // Test log effect
    registry.execute_effect("log", vec![
        Value::Symbol("Starting computation...".into())
    ])?;
    
    // Test compute effect
    let compute_result = registry.execute_effect("compute", vec![
        Value::Int(10),
        Value::Int(32)
    ])?;
    println!("   Compute result: {:?}", compute_result);
    
    // Test greet effect
    registry.execute_effect("greet", vec![
        Value::Symbol("Alice".into())
    ])?;
    
    println!();
    
    // 3. Set up intent evaluator
    println!("3. Setting up intent evaluator...");
    let domain_id = Location::default();
    let synthesizer = FlowSynthesizer::new(domain_id.clone());
    let evaluator = IntentEvaluator::new(synthesizer, registry);
    
    // 4. Create and evaluate simple intents
    println!("4. Evaluating intents...");
    
    // Create a simple intent with a trivial constraint
    let intent = Intent::new(
        domain_id.clone(),
        vec![], // No specific inputs required
        Constraint::True, // Simple constraint that always passes
    );
    
    println!("   Created intent with constraint: Always True");
    
    // Evaluate the intent
    let results = evaluator.evaluate_intent(&intent)?;
    println!("   Intent evaluation results: {:?}", results);
    
    println!();
    
    // 5. Demonstrate constraint validation
    println!("5. Demonstrating constraint validation...");
    
    // Create an intent with a constraint that always fails
    let failing_intent = Intent::new(
        Location::default(),
        vec![],
        Constraint::False, // This should fail
    );
    
    println!("   Created intent with constraint: Always False");
    
    // This should still work in our minimal implementation
    let failing_results = evaluator.evaluate_intent(&failing_intent)?;
    println!("   Failing intent evaluation results: {:?}", failing_results);
    
    println!("\n=== Demo completed successfully! ===");
    println!("\nLayer 2 Features Demonstrated:");
    println!("✓ Effect handler trait and registry");
    println!("✓ Multiple effect handlers (log, compute, greet)");
    println!("✓ Intent creation with constraints");
    println!("✓ Intent evaluation through handler registry");
    println!("✓ Basic constraint handling (True/False)");
    
    Ok(())
} 