use anyhow::Result;
use causality_tel::{
    parser::parse_program,
    runtime::{TelRuntime, RuntimeConfig, RuntimeResult},
    ast::Program,
};
use std::fs;
use std::path::Path;

/// Load a TEL program from a file
fn load_program(file_path: &str) -> Result<Program> {
    let source = fs::read_to_string(file_path)?;
    let program = parse_program(&source)?;
    Ok(program)
}

/// Run a TEL program and return the result
fn run_program(program: &Program) -> Result<RuntimeResult> {
    let config = RuntimeConfig::default();
    let runtime = TelRuntime::new(config);
    let result = runtime.execute(program)?;
    Ok(result)
}

/// Test harness for running TEL examples
#[cfg(test)]
mod examples_tests {
    use super::*;

    #[test]
    fn test_hello_world() -> Result<()> {
        let program = load_program("examples/basic/hello_world.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result is the expected greeting message
        match &result.value {
            serde_json::Value::String(message) => {
                assert_eq!(message, "Hello, TEL world!");
                Ok(())
            }
            _ => panic!("Expected string result, got {:?}", result.value),
        }
    }

    #[test]
    fn test_combinators() -> Result<()> {
        let program = load_program("examples/basic/combinators.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result contains the expected values
        match &result.value {
            serde_json::Value::Object(obj) => {
                // Check identity result
                assert_eq!(obj.get("identity"), Some(&serde_json::json!(42)));
                
                // Check constant result
                assert_eq!(obj.get("constant"), Some(&serde_json::json!("first")));
                
                // Check arithmetic results
                if let Some(serde_json::Value::Object(arithmetic)) = obj.get("arithmetic") {
                    assert_eq!(arithmetic.get("sum"), Some(&serde_json::json!(42)));
                    assert_eq!(arithmetic.get("difference"), Some(&serde_json::json!(38)));
                    assert_eq!(arithmetic.get("product"), Some(&serde_json::json!(80)));
                    assert_eq!(arithmetic.get("quotient"), Some(&serde_json::json!(20)));
                } else {
                    panic!("Expected arithmetic object");
                }
                
                // Check boolean results
                if let Some(serde_json::Value::Object(boolean)) = obj.get("boolean") {
                    assert_eq!(boolean.get("and"), Some(&serde_json::json!(false)));
                    assert_eq!(boolean.get("or"), Some(&serde_json::json!(true)));
                    assert_eq!(boolean.get("not"), Some(&serde_json::json!(false)));
                } else {
                    panic!("Expected boolean object");
                }
                
                // Check string result
                assert_eq!(obj.get("string"), Some(&serde_json::json!("Hello, TEL!")));
                
                Ok(())
            }
            _ => panic!("Expected object result, got {:?}", result.value),
        }
    }

    #[test]
    fn test_variables() -> Result<()> {
        let program = load_program("examples/basic/variables.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result contains the expected values
        match &result.value {
            serde_json::Value::Object(obj) => {
                assert_eq!(obj.get("x"), Some(&serde_json::json!(42)));
                assert_eq!(obj.get("name"), Some(&serde_json::json!("Alice")));
                assert_eq!(obj.get("age"), Some(&serde_json::json!(30)));
                assert_eq!(obj.get("is_active"), Some(&serde_json::json!(true)));
                assert_eq!(obj.get("outer"), Some(&serde_json::json!(20)));
                assert_eq!(obj.get("sum"), Some(&serde_json::json!(12)));
                assert_eq!(obj.get("value"), Some(&serde_json::json!(25)));
                assert_eq!(obj.get("greeting"), Some(&serde_json::json!("Hello, TEL!")));
                assert_eq!(obj.get("status"), Some(&serde_json::json!("Active")));
                
                // Check the user object
                if let Some(serde_json::Value::Object(user)) = obj.get("user") {
                    assert_eq!(user.get("name"), Some(&serde_json::json!("Alice")));
                    assert_eq!(user.get("age"), Some(&serde_json::json!(30)));
                    assert_eq!(user.get("status"), Some(&serde_json::json!("Active")));
                    assert_eq!(user.get("score"), Some(&serde_json::json!(95)));
                } else {
                    panic!("Expected user object");
                }
                
                Ok(())
            }
            _ => panic!("Expected object result, got {:?}", result.value),
        }
    }

    #[test]
    fn test_resources() -> Result<()> {
        let program = load_program("examples/domain/resources.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result contains the expected resource data
        match &result.value {
            serde_json::Value::Object(obj) => {
                // Check schemas section
                if let Some(serde_json::Value::Object(schemas)) = obj.get("schemas") {
                    assert!(schemas.contains_key("user"));
                    assert!(schemas.contains_key("order"));
                } else {
                    panic!("Expected schemas object");
                }
                
                // Check resources section
                if let Some(serde_json::Value::Object(resources)) = obj.get("resources") {
                    assert!(resources.contains_key("user1"));
                    assert!(resources.contains_key("user2"));
                    assert!(resources.contains_key("order"));
                } else {
                    panic!("Expected resources object");
                }
                
                // Check content_ids section
                if let Some(serde_json::Value::Object(content_ids)) = obj.get("content_ids") {
                    assert!(content_ids.contains_key("manual"));
                    assert!(content_ids.contains_key("stored"));
                } else {
                    panic!("Expected content_ids object");
                }
                
                Ok(())
            }
            _ => panic!("Expected object result, got {:?}", result.value),
        }
    }

    #[test]
    fn test_integration() -> Result<()> {
        let program = load_program("examples/domain/integration.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result contains the expected host integration data
        match &result.value {
            serde_json::Value::Object(obj) => {
                // Check functions section
                if let Some(serde_json::Value::Object(functions)) = obj.get("functions") {
                    assert!(functions.contains_key("add"));
                    assert!(functions.contains_key("processData"));
                    assert!(functions.contains_key("fetchUser"));
                    assert!(functions.contains_key("readFile"));
                    assert!(functions.contains_key("writeFile"));
                    assert!(functions.contains_key("executeCommand"));
                    assert!(functions.contains_key("createUser"));
                } else {
                    panic!("Expected functions object");
                }
                
                // Check description
                assert!(obj.contains_key("description"));
                
                Ok(())
            }
            _ => panic!("Expected object result, got {:?}", result.value),
        }
    }

    #[test]
    fn test_domain_models() -> Result<()> {
        let program = load_program("examples/domain/domain_models.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result contains the expected domain model data
        match &result.value {
            serde_json::Value::Object(obj) => {
                // Check domain_model
                assert!(obj.contains_key("domain_model"));
                
                // Check validation
                assert!(obj.contains_key("validation"));
                
                // Check resources
                if let Some(serde_json::Value::Object(resources)) = obj.get("resources") {
                    assert!(resources.contains_key("users"));
                    assert!(resources.contains_key("products"));
                    assert!(resources.contains_key("orders"));
                    assert!(resources.contains_key("order_items"));
                } else {
                    panic!("Expected resources object");
                }
                
                // Check queries
                if let Some(serde_json::Value::Object(queries)) = obj.get("queries") {
                    assert!(queries.contains_key("user_orders"));
                    assert!(queries.contains_key("order_items"));
                } else {
                    panic!("Expected queries object");
                }
                
                Ok(())
            }
            _ => panic!("Expected object result, got {:?}", result.value),
        }
    }

    #[test]
    fn test_todo_app() -> Result<()> {
        let program = load_program("examples/domain/todo_app.tel")?;
        let result = run_program(&program)?;
        
        // Check that the result contains the expected todo app data
        match &result.value {
            serde_json::Value::Object(obj) => {
                // Check domain_model
                assert!(obj.contains_key("domain_model"));
                
                // Check user
                assert!(obj.contains_key("user"));
                
                // Check todos
                if let Some(serde_json::Value::Object(todos)) = obj.get("todos") {
                    assert!(todos.contains_key("all"));
                    
                    if let Some(serde_json::Value::Object(by_tag)) = todos.get("by_tag") {
                        assert!(by_tag.contains_key("work"));
                        assert!(by_tag.contains_key("urgent"));
                    } else {
                        panic!("Expected todos.by_tag object");
                    }
                } else {
                    panic!("Expected todos object");
                }
                
                // Check tags
                assert!(obj.contains_key("tags"));
                
                Ok(())
            }
            _ => panic!("Expected object result, got {:?}", result.value),
        }
    }
} 