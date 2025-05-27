// TEG Execution Tests
//
// This file contains tests for the TEL → TEG → Execution pipeline.

use std::sync::Arc;
use std::collections::HashMap;

use causality_core::effect::{
    Effect as CoreEffect,
    EffectId as CoreEffectId,
    EffectOutcome as CoreEffectOutcome,
    EffectContext as CoreEffectContext,
    registry::EffectRegistry as CoreEffectRegistry
};

use causality_core::resource::{
    ResourceManager,
    ResourceResult
};

use causality_tel::{
    Parser, Compiler, Program,
    combinators::Combinator
};

use causality_ir::{
    TemporalEffectGraph,
    tel::to_teg::ToTEGFragment
};

use crate::effect::executor::EffectExecutor;
use crate::effect::tel::teg_executor::TegExecutor;
use crate::effect::tel::adapter::TelEffectAdapter;
use crate::effect::tel::tests::teg_executor_tests::TestResourceManager;

/// Simple example TEL program for testing
const EXAMPLE_TEL_PROGRAM: &str = r#"
// Define resources
resource User {
    name: String,
    email: String,
    active: Bool
}

// Define effects
effect create_user(name, email);
effect get_user(id);
effect update_user(id, name, email, active);
effect send_email(to, subject, body);

// Define the workflow
flow user_registration {
    // Create a new user
    let user_id = create_user("Test User", "test@example.com");
    
    // Get the created user
    let user = get_user(user_id);
    
    // Send a welcome email
    let email_sent = send_email(
        user.email,
        "Welcome to our service",
        "Thank you for registering with our service."
    );
    
    // Mark user as active
    let updated_user = update_user(
        user_id,
        user.name,
        user.email,
        true
    );
    
    // Return the updated user 
    updated_user
}
"#;

/// Demonstrate the full TEL-to-TEG execution pipeline
#[tokio::test]
async fn test_tel_teg_execution_pipeline() {
    // Step 1: Parse TEL code
    let parser = Parser::new();
    let ast = parser.parse(EXAMPLE_TEL_PROGRAM).expect("Failed to parse TEL program");
    
    // Step 2: Compile TEL to internal representation
    let compiler = Compiler::new();
    let program = compiler.compile(&ast).expect("Failed to compile TEL program");
    
    // Step 3: Convert to TEG (this is the key step in the migration)
    let teg = program.to_teg().expect("Failed to convert program to TEG");
    
    // Step 4: Set up execution environment
    let resource_manager = Arc::new(TestResourceManager::default());
    let effect_registry = Arc::new(CoreEffectRegistry::new());
    let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
    
    // Step 5: Register effect handlers
    register_test_effect_handlers(&effect_registry);
    
    // Step 6: Create TEG executor
    let executor = TegExecutor::new(effect_executor, resource_manager);
    
    // Step 7: Execute TEG
    let result = executor.execute(&teg).await.expect("Failed to execute TEG");
    
    // Step 8: Verify execution results
    assert!(!result.trace.is_empty(), "Execution trace should not be empty");
    
    // Verify all effects were executed
    let has_create = result.trace.iter().any(|entry| entry.effect_type.contains("create_user"));
    let has_get = result.trace.iter().any(|entry| entry.effect_type.contains("get_user"));
    let has_send = result.trace.iter().any(|entry| entry.effect_type.contains("send_email"));
    let has_update = result.trace.iter().any(|entry| entry.effect_type.contains("update_user"));
    
    assert!(has_create, "create_user effect not found in trace");
    assert!(has_get, "get_user effect not found in trace");
    assert!(has_send, "send_email effect not found in trace");
    assert!(has_update, "update_user effect not found in trace");
    
    // Example of checking execution order (simplified for illustration)
    let trace_effects: Vec<&str> = result.trace.iter()
        .map(|entry| entry.effect_type.as_str())
        .collect();
    
    println!("Execution trace: {:?}", trace_effects);
}

/// Demonstrate the adapter-based approach for single effects
#[tokio::test]
async fn test_tel_effect_adapter() {
    // Step 1: Create a TelEffectAdapter
    let adapter = TelEffectAdapter::new(
        "log",
        Combinator::Literal(causality_tel::combinators::Literal::String("Test message".to_string()))
    );
    
    // Step 2: Set up execution environment
    let resource_manager = Arc::new(TestResourceManager::default());
    let effect_registry = Arc::new(CoreEffectRegistry::new());
    let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
    
    // Step 3: Register effect handlers
    let log_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        println!("Log: {}", params);
        let mut data = HashMap::new();
        data.insert("logged".to_string(), "true".to_string());
        Ok(CoreEffectOutcome::success(data))
    };
    
    effect_registry.register_handler("log", log_handler).expect("Failed to register handler");
    
    // Step 4: Create execution context
    let context = effect_executor.create_context();
    
    // Step 5: Execute the effect through the adapter
    let effect: Box<dyn CoreEffect> = adapter.to_core_effect();
    let result = effect_executor.execute_effect(effect, &*context).await
        .expect("Failed to execute effect");
    
    // Step 6: Verify execution result
    assert!(result.is_success(), "Effect execution should succeed");
    assert_eq!(result.get_output().get("logged").unwrap(), "true");
}

/// Register all test effect handlers
fn register_test_effect_handlers(effect_registry: &CoreEffectRegistry) {
    // Create user handler
    let create_user_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let email = params.get("email").and_then(|v| v.as_str()).unwrap_or("unknown@example.com");
        
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user-123".to_string());
        data.insert("name".to_string(), name.to_string());
        data.insert("email".to_string(), email.to_string());
        data.insert("active".to_string(), "false".to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    // Get user handler
    let get_user_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let _id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
        
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user-123".to_string());
        data.insert("name".to_string(), "Test User".to_string());
        data.insert("email".to_string(), "test@example.com".to_string());
        data.insert("active".to_string(), "false".to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    // Update user handler
    let update_user_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let _id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let email = params.get("email").and_then(|v| v.as_str()).unwrap_or("unknown@example.com");
        let active = params.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
        
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user-123".to_string());
        data.insert("name".to_string(), name.to_string());
        data.insert("email".to_string(), email.to_string());
        data.insert("active".to_string(), active.to_string());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    // Send email handler
    let send_email_handler = |params: serde_json::Value, _context: &dyn CoreEffectContext| {
        let to = params.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let subject = params.get("subject").and_then(|v| v.as_str()).unwrap_or("");
        let _body = params.get("body").and_then(|v| v.as_str()).unwrap_or("");
        
        let mut data = HashMap::new();
        data.insert("to".to_string(), to.to_string());
        data.insert("subject".to_string(), subject.to_string());
        data.insert("sent".to_string(), "true".to_string());
        data.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        
        Ok(CoreEffectOutcome::success(data))
    };
    
    // Register handlers
    effect_registry.register_handler("create_user", create_user_handler).expect("Failed to register create_user handler");
    effect_registry.register_handler("get_user", get_user_handler).expect("Failed to register get_user handler");
    effect_registry.register_handler("update_user", update_user_handler).expect("Failed to register update_user handler");
    effect_registry.register_handler("send_email", send_email_handler).expect("Failed to register send_email handler");
} 