use std::collections::HashMap;

use causality::domain_adapters::succinct::{
    SuccinctAdapter, SuccinctVm, PublicInputs, default_adapter, ZkVirtualMachine
};
use causality::error::{Result, Error};

/// Test adapter initialization with default settings
#[test]
fn test_succinct_adapter_init() -> Result<()> {
    // Create a Succinct adapter with a test API key
    let adapter = default_adapter()?
        .with_api_key("test-api-key");
    
    // Simple verification that adapter is properly initialized
    assert!(adapter.is_initialized(), "Adapter should be initialized");
    Ok(())
}

/// Test program compilation in Succinct ZK-VM
#[test]
fn test_succinct_program_compilation() -> Result<()> {
    // Create adapter with test key
    let adapter = default_adapter()?
        .with_api_key("test-api-key");
    
    // Define a simple program that doubles a value
    let source_code = r#"
fn main() {
    // Get the input value from environment
    let input = env::get_input("value").unwrap();
    let value: u32 = serde_json::from_str(&input).unwrap();
    
    // Double the value and return it
    let result = value * 2;
    
    // Output the result
    env::set_output("result", &result);
}
    "#;
    
    // Test program compilation
    let program_id = adapter.compile_program(source_code, Some("double-value"))?;
    
    // Verify we got a valid program ID back
    assert!(!program_id.is_empty(), "Program ID should not be empty");
    Ok(())
}

/// Test complete proof generation and verification flow with Succinct ZK-VM
#[test]
#[ignore] // Ignore by default as it requires network access and API key
fn test_succinct_proof_generation_verification() -> Result<()> {
    // Create adapter with test key - in actual testing, this would be a real key
    let adapter = default_adapter()?
        .with_api_key("test-api-key");
    
    // Define a simple program that doubles a value
    let source_code = r#"
fn main() {
    // Get the input value from environment
    let input = env::get_input("value").unwrap();
    let value: u32 = serde_json::from_str(&input).unwrap();
    
    // Double the value and return it
    let result = value * 2;
    
    // Output the result
    env::set_output("result", &result);
}
    "#;
    
    // Compile the program
    let program_id = adapter.compile_program(source_code, Some("double-value"))?;
    
    // Prepare inputs
    let mut public_inputs = PublicInputs::new();
    let input_value = 42u32;
    public_inputs.add("value", &input_value)?;
    
    // Generate a proof
    let private_inputs = HashMap::new();
    let proof = adapter.prove(&program_id, &public_inputs, &private_inputs, None)?;
    
    // Verify the proof
    let is_valid = adapter.verify(&program_id, &proof, &public_inputs)?;
    assert!(is_valid, "Proof should be valid");
    
    // Check the output
    if let Some(journal) = &proof.journal {
        if let Some(result_str) = journal.get("result") {
            // Parse result
            if let Ok(result) = result_str.parse::<u32>() {
                assert_eq!(result, input_value * 2, "Result should be double the input value");
            } else {
                return Err(Error::Validation("Failed to parse result".to_string()));
            }
        } else {
            return Err(Error::Validation("No result in journal".to_string()));
        }
    } else {
        return Err(Error::Validation("No journal in proof".to_string()));
    }
    
    Ok(())
}

/// Test error handling for invalid programs
#[test]
fn test_succinct_invalid_program() -> Result<()> {
    // Create adapter with test key
    let adapter = default_adapter()?
        .with_api_key("test-api-key");
    
    // Invalid program with syntax error
    let invalid_source_code = r#"
fn main() {
    This is not valid Rust!
}
    "#;
    
    // Should fail with a compilation error
    let result = adapter.compile_program(invalid_source_code, Some("invalid-program"));
    assert!(result.is_err(), "Compilation of invalid program should fail");
    
    Ok(())
}

/// Test the API key masking utility (moved from the demo file)
#[test]
fn test_api_key_masking() {
    assert_eq!(mask_api_key("short"), "[masked]");
    assert_eq!(mask_api_key("12345678"), "[masked]");
    assert_eq!(mask_api_key("1234567890"), "1234...7890");
    assert_eq!(mask_api_key("abcdefghijklmnopqrstuvwxyz"), "abcd...wxyz");
}

/// Masks most of the API key for security
fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "[masked]".to_string();
    }
    
    let visible_prefix = &key[0..4];
    let visible_suffix = &key[key.len() - 4..];
    format!("{}...{}", visible_prefix, visible_suffix)
} 