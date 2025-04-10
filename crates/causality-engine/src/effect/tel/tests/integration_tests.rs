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