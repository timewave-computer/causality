// Tests for content-addressable code execution

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tempfile::TempDir;

use causality::code::{
    CodeRepository, ContentAddressableExecutor, ContentHash, HashAlgorithm, Value,
};
use causality::error::Result;
use causality::resource::ResourceManager;

#[derive(Debug, Serialize, Deserialize)]
struct TestCode {
    function: String,
    arguments: Vec<Value>,
}

#[test]
fn test_basic_execution() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);
    let resource_manager = Arc::new(ResourceManager::new());

    // Create an executor
    let executor = ContentAddressableExecutor::new(repository.clone(), resource_manager.clone());

    // Create a simple test code object
    let test_code = TestCode {
        function: "add".to_string(),
        arguments: vec![Value::Int(5), Value::Int(10)],
    };

    // Serialize and store the code
    let serialized = bincode::serialize(&test_code)?;
    let metadata = repository
        .create_metadata()
        .with_name(Some("add_function".to_string()))
        .with_format("json".to_string())
        .build();

    let hash = repository.store_with_metadata(serialized, metadata.clone())?;

    // Register the name
    repository.register_name("add_function", &hash)?;

    // Create a context and execute the code
    let context = executor.create_context("test_execution".to_string(), None)?;

    // Execute by hash
    let result_by_hash = executor.execute_by_hash(&hash, vec![], &context);

    // This should return a placeholder value since we haven't implemented
    // the actual execution logic yet
    assert!(result_by_hash.is_ok());

    // Execute by name
    let result_by_name = executor.execute_by_name("add_function", vec![], &context);
    assert!(result_by_name.is_ok());

    // Get the execution trace
    let trace = context.execution_trace()?;

    // Verify trace contains the expected events
    assert_eq!(trace.len(), 4); // 2 invocations and 2 returns

    Ok(())
}

#[test]
fn test_security_sandbox() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);
    let resource_manager = Arc::new(ResourceManager::new());

    // Create a sandbox with custom security settings
    let sandbox = causality::code::executor::SecuritySandbox::default()
        .with_timeout(100)
        .with_memory_limit(1024 * 1024) // 1 MB
        .with_instruction_limit(1000);

    // Create an executor with the sandbox
    let executor = ContentAddressableExecutor::new(repository.clone(), resource_manager.clone())
        .with_sandbox(sandbox);

    // Create a simple test code object
    let test_code = TestCode {
        function: "simple".to_string(),
        arguments: vec![],
    };

    // Serialize and store the code
    let serialized = bincode::serialize(&test_code)?;
    let metadata = repository
        .create_metadata()
        .with_name(Some("simple_function".to_string()))
        .with_format("json".to_string())
        .build();

    let hash = repository.store_with_metadata(serialized, metadata.clone())?;

    // Execute with sandbox
    let result = executor.execute_with_sandbox(&hash, vec![], "sandbox_test".to_string());

    // Check that execution completed successfully
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_context_isolation() -> Result<()> {
    // Set up test environment
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();
    let repository = Arc::new(CodeRepository::new(repo_path)?);
    let resource_manager = Arc::new(ResourceManager::new());

    // Create an executor
    let executor = ContentAddressableExecutor::new(repository.clone(), resource_manager.clone());

    // Create a parent context
    let parent_context = executor.create_context("parent".to_string(), None)?;

    // Set a variable in the parent context
    parent_context.set_variable(
        "parent_var".to_string(),
        Value::String("parent_value".to_string()),
    )?;

    // Create a child context
    let child_context =
        executor.create_context("child".to_string(), Some(parent_context.clone()))?;

    // Set a variable in the child context
    child_context.set_variable(
        "child_var".to_string(),
        Value::String("child_value".to_string()),
    )?;

    // Verify parent can't see child's variable
    assert!(parent_context.get_variable("child_var").is_none());

    // Verify child can see parent's variable
    let parent_var = child_context.get_variable("parent_var").unwrap();
    match parent_var {
        Value::String(s) => assert_eq!(s, "parent_value"),
        _ => panic!("Expected String value"),
    }

    // Create a second child context
    let child2_context =
        executor.create_context("child2".to_string(), Some(parent_context.clone()))?;

    // Set a variable in the second child context
    child2_context.set_variable(
        "child2_var".to_string(),
        Value::String("child2_value".to_string()),
    )?;

    // Verify first child can't see second child's variable
    assert!(child_context.get_variable("child2_var").is_none());

    // Verify second child can't see first child's variable
    assert!(child2_context.get_variable("child_var").is_none());

    Ok(())
}
