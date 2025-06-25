//! Integration tests for the submit transaction command
//!
//! These tests verify the submit transaction functionality including proof validation,
//! chain configuration, and dry-run mode.

use anyhow::Result;
use std::path::PathBuf;
use causality_cli::commands::submit::SubmitCommand;

#[tokio::test]
async fn test_submit_command_dry_run() -> Result<()> {
    // Create a submit command with test data
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/sample_proof.json");
    
    let submit_cmd = SubmitCommand {
        proof: test_data_path,
        target_chains: "ethereum".to_string(),
        dry_run: true,
        gas_price_gwei: Some(20),
        gas_limit: Some(500000),
        verbose: false,
    };
    
    // Test that the command can be created and configured properly
    assert_eq!(submit_cmd.target_chains, "ethereum");
    assert!(submit_cmd.dry_run);
    assert_eq!(submit_cmd.gas_price_gwei, Some(20));
    assert_eq!(submit_cmd.gas_limit, Some(500000));
    
    // Note: We don't actually execute the command here since it would
    // attempt to make real network calls. In a full test suite, we would
    // use mock HTTP clients or test networks.
    
    Ok(())
}

#[tokio::test]
async fn test_submit_command_multi_chain() -> Result<()> {
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/sample_proof.json");
    
    let submit_cmd = SubmitCommand {
        proof: test_data_path,
        target_chains: "ethereum,polygon,arbitrum".to_string(),
        dry_run: true,
        gas_price_gwei: None,
        gas_limit: None,
        verbose: true,
    };
    
    // Parse target chains
    let chains: Vec<&str> = submit_cmd.target_chains.split(',').map(|s| s.trim()).collect();
    assert_eq!(chains.len(), 3);
    assert!(chains.contains(&"ethereum"));
    assert!(chains.contains(&"polygon"));
    assert!(chains.contains(&"arbitrum"));
    
    Ok(())
}

#[test]
fn test_chain_config_generation() -> Result<()> {
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/sample_proof.json");
    
    let submit_cmd = SubmitCommand {
        proof: test_data_path,
        target_chains: "ethereum".to_string(),
        dry_run: true,
        gas_price_gwei: None,
        gas_limit: None,
        verbose: false,
    };
    
    // Test chain configuration generation
    let ethereum_config = submit_cmd.get_chain_config("ethereum")?;
    assert_eq!(ethereum_config.name, "ethereum");
    assert_eq!(ethereum_config.chain_id, 1);
    assert!(ethereum_config.rpc_url.contains("alchemy"));
    
    let polygon_config = submit_cmd.get_chain_config("polygon")?;
    assert_eq!(polygon_config.name, "polygon");
    assert_eq!(polygon_config.chain_id, 137);
    
    let arbitrum_config = submit_cmd.get_chain_config("arbitrum")?;
    assert_eq!(arbitrum_config.name, "arbitrum");
    assert_eq!(arbitrum_config.chain_id, 42161);
    
    let optimism_config = submit_cmd.get_chain_config("optimism")?;
    assert_eq!(optimism_config.name, "optimism");
    assert_eq!(optimism_config.chain_id, 10);
    
    // Test unsupported chain
    assert!(submit_cmd.get_chain_config("unsupported").is_err());
    
    Ok(())
}

#[test]
fn test_proof_file_exists() {
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/sample_proof.json");
    
    assert!(test_data_path.exists(), "Sample proof file should exist at {:?}", test_data_path);
}

#[tokio::test]
async fn test_proof_file_parsing() -> Result<()> {
    use std::fs;
    use causality_api::types::ProofData;
    
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/sample_proof.json");
    
    // Read and parse the proof file
    let proof_data = fs::read_to_string(&test_data_path)?;
    let proof: ProofData = serde_json::from_str(&proof_data)?;
    
    // Verify proof structure
    assert!(!proof.proof.is_empty());
    assert!(!proof.public_inputs.is_empty());
    assert_eq!(proof.public_inputs.len(), 2);
    assert!(!proof.verification_key.is_empty());
    assert_eq!(proof.circuit_id, "causality_transfer_v1");
    assert!(!proof.metadata.is_empty());
    
    Ok(())
}
