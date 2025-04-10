// TEG Execution Tests
//
// This file contains tests for the TEL → TEG → Execution pipeline.

use std::sync::Arc;

use causality_tel::ast::Program;
use causality_tel::parser;
use causality_ir::TemporalEffectGraph;

use crate::effect::tel::TegExecutor;
use crate::effect::executor::EffectExecutor;
use crate::effect::registry::EffectRegistry;
use crate::resource::manager::BasicResourceManager;

#[tokio::test]
async fn test_simple_identity_program() {
    // Parse a simple TEL program
    let tel_source = r#"
    flow identity(x: string) -> string {
        return x;
    }
    "#;
    
    let program = parser::parse_program(tel_source).expect("Failed to parse program");
    
    // Convert to TEG
    let teg = program.to_teg().expect("Failed to convert program to TEG");
    
    // Set up execution environment
    let effect_registry = EffectRegistry::new();
    let executor = Arc::new(EffectExecutor::new(Arc::new(effect_registry)));
    let resource_manager = Arc::new(BasicResourceManager::new());
    let teg_executor = TegExecutor::new(executor, resource_manager);
    
    // Execute the TEG
    let result = teg_executor.execute(&teg).await.expect("Failed to execute TEG");
    
    // Verify result
    assert!(!result.outputs.is_empty(), "TEG execution should produce outputs");
}

#[tokio::test]
async fn test_effect_program() {
    // Parse a program with effects
    let tel_source = r#"
    effect log(message: string) -> null;
    
    flow log_message() -> null {
        perform log("Hello, TEG!");
        return null;
    }
    "#;
    
    let program = parser::parse_program(tel_source).expect("Failed to parse program");
    
    // Convert to TEG
    let teg = program.to_teg().expect("Failed to convert program to TEG");
    
    // Set up execution environment
    let mut effect_registry = EffectRegistry::new();
    
    // Register log effect handler
    effect_registry.register_handler_fn(
        "log",
        |params, _context| {
            println!("Log effect executed with params: {:?}", params);
            Ok(serde_json::json!(null))
        }
    ).expect("Failed to register handler");
    
    let executor = Arc::new(EffectExecutor::new(Arc::new(effect_registry)));
    let resource_manager = Arc::new(BasicResourceManager::new());
    let teg_executor = TegExecutor::new(executor, resource_manager);
    
    // Execute the TEG
    let result = teg_executor.execute(&teg).await.expect("Failed to execute TEG");
    
    // Verify execution
    assert_eq!(result.trace.len(), 2, "Should have two effect executions");
}

#[tokio::test]
async fn test_resource_program() {
    // Parse a program with resource operations
    let tel_source = r#"
    flow manage_resource() -> string {
        let resource_id = perform resource("create", "user", null, {
            "name": "Test User",
            "email": "test@example.com"
        });
        
        let user = perform resource("get", "user", resource_id, {});
        
        return resource_id;
    }
    "#;
    
    let program = parser::parse_program(tel_source).expect("Failed to parse program");
    
    // Convert to TEG
    let teg = program.to_teg().expect("Failed to convert program to TEG");
    
    // Set up execution environment
    let effect_registry = EffectRegistry::new();
    let executor = Arc::new(EffectExecutor::new(Arc::new(effect_registry)));
    let resource_manager = Arc::new(BasicResourceManager::new());
    let teg_executor = TegExecutor::new(executor, resource_manager);
    
    // Execute the TEG
    let result = teg_executor.execute(&teg).await.expect("Failed to execute TEG");
    
    // Verify resource operations were executed
    println!("Execution trace: {:?}", result.trace);
    println!("Outputs: {:?}", result.outputs);
} 