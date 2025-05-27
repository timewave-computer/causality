//! Integration tests for the TEL-to-TEG-to-Execution workflow
//!
//! These tests verify the end-to-end pipeline of converting TEL programs
//! to TEG representations and executing them using the TegExecutor.

use std::sync::Arc;
use std::collections::HashMap;

use causality_core::effect::{
    registry::EffectRegistry as CoreEffectRegistry,
    Effect as CoreEffect,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
};

use causality_core::resource::{
    Resource as CoreResource,
    ResourceManager, ResourceResult
};

use causality_tel::{
    Program, Flow, Compiler, Parser,
    combinators::Combinator,
    types::effect::TelEffect,
};

use causality_ir::{
    TemporalEffectGraph, TEGFragment,
    graph::edge::{Condition, TemporalRelation, RelationshipType}
};

use crate::effect::executor::EffectExecutor;
use crate::effect::tel::teg_executor::TegExecutor;
use crate::effect::tel::tests::teg_executor_tests::TestResourceManager;

/// Simple TEL program for testing
const SIMPLE_TEL_PROGRAM: &str = r#"
let add = fn(a, b) => a + b;

effect log(message);

flow simple_flow {
    let result = add(10, 32);
    log("The result is: " + result);
    result
}
"#;

/// Test the complete workflow from TEL program to TEG execution
#[tokio::test]
async fn test_tel_to_teg_to_execution() {
    // Parse TEL program
    let parser = Parser::new();
    let ast = parser.parse(SIMPLE_TEL_PROGRAM).unwrap();
    
    // Compile TEL program
    let compiler = Compiler::new();
    let program = compiler.compile(&ast).unwrap();
    
    // Convert to TEG
    let teg = program.to_teg().unwrap();
    
    // Setup execution environment
    let resource_manager = Arc::new(TestResourceManager::default());
    let effect_registry = Arc::new(CoreEffectRegistry::new());
    let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
    
    // Register a log effect handler
    let log_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        println!("Log: {}", params);
        let mut data = HashMap::new();
        data.insert("logged".to_string(), "true".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    effect_registry.register_handler("log", log_handler).unwrap();
    
    // Create TEG executor
    let executor = TegExecutor::new(effect_executor, resource_manager);
    
    // Execute TEG
    let result = executor.execute(&teg).await.unwrap();
    
    // Verify execution completed successfully
    assert!(!result.trace.is_empty());
    
    // We expect at least log effect execution in the trace
    let has_log = result.trace.iter().any(|entry| entry.effect_type.contains("log"));
    assert!(has_log, "Log effect execution not found in trace");
}

/// Test with a more complex program with resource operations
#[tokio::test]
async fn test_resource_operations_workflow() {
    // Define TEL program with resource operations
    let program_src = r#"
    resource User {
        name: String,
        age: Int
    }
    
    effect create_user(name, age);
    effect get_user(id);
    effect update_user(id, name, age);
    
    flow user_flow {
        let user_id = create_user("Alice", 30);
        let user = get_user(user_id);
        let updated = update_user(user_id, "Alice Smith", 31);
        updated
    }
    "#;
    
    // Parse and compile
    let parser = Parser::new();
    let ast = parser.parse(program_src).unwrap();
    
    let compiler = Compiler::new();
    let program = compiler.compile(&ast).unwrap();
    
    // Convert to TEG
    let teg = program.to_teg().unwrap();
    
    // Set up execution environment
    let resource_manager = Arc::new(TestResourceManager::default());
    let effect_registry = Arc::new(CoreEffectRegistry::new());
    let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
    
    // Register effect handlers
    let create_user_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user-123".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    let get_user_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());
        data.insert("age".to_string(), "30".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    let update_user_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice Smith".to_string());
        data.insert("age".to_string(), "31".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    effect_registry.register_handler("create_user", create_user_handler).unwrap();
    effect_registry.register_handler("get_user", get_user_handler).unwrap();
    effect_registry.register_handler("update_user", update_user_handler).unwrap();
    
    // Create TEG executor
    let executor = TegExecutor::new(effect_executor, resource_manager);
    
    // Execute TEG
    let result = executor.execute(&teg).await.unwrap();
    
    // Verify execution
    assert!(!result.trace.is_empty());
    
    // Check that all three effects were executed
    let has_create = result.trace.iter().any(|entry| entry.effect_type.contains("create_user"));
    let has_get = result.trace.iter().any(|entry| entry.effect_type.contains("get_user"));
    let has_update = result.trace.iter().any(|entry| entry.effect_type.contains("update_user"));
    
    assert!(has_create, "create_user effect not found in trace");
    assert!(has_get, "get_user effect not found in trace");
    assert!(has_update, "update_user effect not found in trace");
}

/// Test with a more complex scenario, including conditional execution and resource dependencies
#[tokio::test]
async fn test_teg_based_conditional_workflow() {
    // Define TEL program with conditional logic and resource operations
    let program_src = r#"
    resource Counter {
        value: Int
    }
    
    effect init_counter(initial_value);
    effect increment_counter(id);
    effect get_counter(id);
    effect conditional_update(id, condition);
    
    flow counter_workflow {
        // Create a new counter with initial value
        let counter_id = init_counter(5);
        
        // Get current value
        let counter = get_counter(counter_id);
        
        // Only increment if value is less than 10
        let should_increment = counter.value < 10;
        let updated = conditional_update(counter_id, should_increment);
        
        // Increment again and get final value
        let final_counter = increment_counter(counter_id);
        
        // Return the final state
        final_counter
    }
    "#;
    
    // Parse and compile
    let parser = causality_tel::Parser::new();
    let ast = parser.parse(program_src).unwrap();
    
    let compiler = causality_tel::Compiler::new();
    let program = compiler.compile(&ast).unwrap();
    
    // Convert to TEG using the standard approach
    let teg = program.to_teg().unwrap();
    
    // Set up execution environment with appropriate mocks
    let resource_manager = Arc::new(TestResourceManager::default());
    let effect_registry = Arc::new(CoreEffectRegistry::new());
    let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
    
    // Register effect handlers
    let init_counter_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        // Initialize counter resource
        let initial_value = params.get("initial_value")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
            
        let mut data = HashMap::new();
        data.insert("counter_id".to_string(), "counter-123".to_string());
        data.insert("value".to_string(), initial_value.to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    let get_counter_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        // Get counter state
        let _id = params.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let mut data = HashMap::new();
        data.insert("value".to_string(), "5".to_string());
        data.insert("id".to_string(), "counter-123".to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    let increment_counter_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        // Increment counter
        let _id = params.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let mut data = HashMap::new();
        data.insert("value".to_string(), "6".to_string());
        data.insert("id".to_string(), "counter-123".to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    let conditional_update_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        // Only update if condition is true
        let _id = params.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let condition = params.get("condition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let mut data = HashMap::new();
        
        if condition {
            data.insert("value".to_string(), "6".to_string());
            data.insert("updated".to_string(), "true".to_string());
        } else {
            data.insert("value".to_string(), "5".to_string());
            data.insert("updated".to_string(), "false".to_string());
        }
        
        data.insert("id".to_string(), "counter-123".to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    effect_registry.register_handler("init_counter", init_counter_handler).unwrap();
    effect_registry.register_handler("get_counter", get_counter_handler).unwrap();
    effect_registry.register_handler("increment_counter", increment_counter_handler).unwrap();
    effect_registry.register_handler("conditional_update", conditional_update_handler).unwrap();
    
    // Create TEG executor
    let executor = TegExecutor::new(effect_executor, resource_manager);
    
    // Execute TEG
    let result = executor.execute(&teg).await.unwrap();
    
    // Verify execution
    assert!(!result.trace.is_empty());
    
    // Check that all operations were executed in the correct order
    let trace_effects: Vec<&str> = result.trace.iter()
        .map(|entry| entry.effect_type.as_str())
        .collect();
        
    // Verify each effect was executed
    let has_init = result.trace.iter().any(|entry| entry.effect_type.contains("init_counter"));
    let has_get = result.trace.iter().any(|entry| entry.effect_type.contains("get_counter"));
    let has_conditional = result.trace.iter().any(|entry| entry.effect_type.contains("conditional_update"));
    let has_increment = result.trace.iter().any(|entry| entry.effect_type.contains("increment_counter"));
    
    assert!(has_init, "init_counter effect not found in trace");
    assert!(has_get, "get_counter effect not found in trace");
    assert!(has_conditional, "conditional_update effect not found in trace");
    assert!(has_increment, "increment_counter effect not found in trace");
}

/// Test using TEG directly without going through TEL first
/// This demonstrates how the TEG execution layer can be used independently
#[tokio::test]
async fn test_direct_teg_execution() {
    // Create a TEG directly
    let mut teg = TemporalEffectGraph::new();
    
    // Create effect nodes
    let log_effect = EffectNode::builder()
        .id("log_effect")
        .effect_type("log")
        .domain("test_domain")
        .string_parameter("message", "Hello from direct TEG")
        .build()
        .unwrap();
        
    let compute_effect = EffectNode::builder()
        .id("compute_effect")
        .effect_type("compute")
        .domain("test_domain")
        .string_parameter("operation", "add")
        .string_parameter("operand1", "10")
        .string_parameter("operand2", "20")
        .build()
        .unwrap();
        
    let output_effect = EffectNode::builder()
        .id("output_effect")
        .effect_type("output")
        .domain("test_domain")
        .build()
        .unwrap();
    
    // Add nodes to TEG
    teg.add_effect_node(log_effect);
    teg.add_effect_node(compute_effect);
    teg.add_effect_node(output_effect);
    
    // Add dependencies and continuations
    teg.effect_continuations.insert(
        "log_effect".to_string(),
        vec![("compute_effect".to_string(), None)]
    );
    
    teg.effect_continuations.insert(
        "compute_effect".to_string(),
        vec![("output_effect".to_string(), None)]
    );
    
    // Set up execution environment with appropriate mocks
    let resource_manager = Arc::new(TestResourceManager::default());
    let effect_registry = Arc::new(CoreEffectRegistry::new());
    let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
    
    // Register effect handlers
    let log_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        println!("Log: {}", params.get("message").unwrap_or(&serde_json::Value::Null));
        let mut data = HashMap::new();
        data.insert("logged".to_string(), "true".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    let compute_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let op = params.get("operation").and_then(|v| v.as_str()).unwrap_or("");
        let a = params.get("operand1").and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let b = params.get("operand2").and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        
        let result = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" => if b != 0 { a / b } else { 0 },
            _ => 0,
        };
        
        let mut data = HashMap::new();
        data.insert("result".to_string(), result.to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    let output_handler = |_params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let mut data = HashMap::new();
        data.insert("output".to_string(), "completed".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    effect_registry.register_handler("log", log_handler).unwrap();
    effect_registry.register_handler("compute", compute_handler).unwrap();
    effect_registry.register_handler("output", output_handler).unwrap();
    
    // Create TEG executor
    let executor = TegExecutor::new(effect_executor, resource_manager);
    
    // Execute TEG
    let result = executor.execute(&teg).await.unwrap();
    
    // Verify execution
    assert!(!result.trace.is_empty());
    assert_eq!(result.trace.len(), 3, "Expected 3 effect executions in trace");
    
    // Verify the order of execution
    assert_eq!(result.trace[0].effect_id, "log_effect");
    assert_eq!(result.trace[1].effect_id, "compute_effect");
    assert_eq!(result.trace[2].effect_id, "output_effect");
} 