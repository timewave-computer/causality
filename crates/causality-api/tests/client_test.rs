//! Integration tests for the ChainClient
//!
//! These tests verify the blockchain client functionality including proof validation,
//! transaction building, and multi-chain support.

use anyhow::Result;
use std::path::PathBuf;
use causality_api::types::*;
use causality_api::client::{ChainClient, TransactionResult};

#[tokio::test]
async fn test_chain_client_creation() -> Result<()> {
    let config = ChainConfig {
        name: "test_chain".to_string(),
        chain_id: 31337, // Hardhat default
        rpc_url: "http://localhost:8545".to_string(),
        explorer_url: "http://localhost:8545".to_string(),
        gas_price_multiplier: 1.0,
        confirmation_blocks: 1,
    };
    
    let client = ChainClient::new(config).await?;
    // Client creation should succeed
    Ok(())
}

#[test]
fn test_proof_data_serialization() -> Result<()> {
    use std::fs;
    
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/sample_proof.json");
    
    // Read and parse the proof file
    let proof_data = fs::read_to_string(&test_data_path)?;
    let proof: ProofData = serde_json::from_str(&proof_data)?;
    
    // Serialize back to JSON
    let serialized = serde_json::to_string(&proof)?;
    let reparsed: ProofData = serde_json::from_str(&serialized)?;
    
    // Verify round-trip consistency
    assert_eq!(proof.proof, reparsed.proof);
    assert_eq!(proof.public_inputs, reparsed.public_inputs);
    assert_eq!(proof.verification_key, reparsed.verification_key);
    assert_eq!(proof.circuit_id, reparsed.circuit_id);
    
    Ok(())
}

#[test]
fn test_transaction_request_creation() -> Result<()> {
    use std::fs;
    
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/complex_proof.json");
    
    let proof_data = fs::read_to_string(&test_data_path)?;
    let proof: ProofData = serde_json::from_str(&proof_data)?;
    
    let tx_request = TransactionRequest {
        proof_data: proof.clone(),
        gas_price: Some(20_000_000_000), // 20 gwei
        gas_limit: Some(500_000),
        dry_run: true,
    };
    
    assert_eq!(tx_request.gas_price, Some(20_000_000_000));
    assert_eq!(tx_request.gas_limit, Some(500_000));
    assert!(tx_request.dry_run);
    assert_eq!(tx_request.proof_data.circuit_id, "causality_defi_bridge_v2");
    
    Ok(())
}

#[test]
fn test_chain_config_validation() {
    let config = ChainConfig {
        name: "ethereum".to_string(),
        chain_id: 1,
        rpc_url: "https://eth-mainnet.g.alchemy.com/v2/demo".to_string(),
        explorer_url: "https://etherscan.io".to_string(),
        gas_price_multiplier: 1.1,
        confirmation_blocks: 12,
    };
    
    assert_eq!(config.name, "ethereum");
    assert_eq!(config.chain_id, 1);
    assert!(config.rpc_url.starts_with("https://"));
    assert!(config.gas_price_multiplier > 1.0);
    assert!(config.confirmation_blocks > 0);
}

#[test]
fn test_transaction_status_serialization() -> Result<()> {
    let statuses = vec![
        TransactionStatus::Success,
        TransactionStatus::Failed,
        TransactionStatus::Pending,
        TransactionStatus::ValidatedSuccess,
        TransactionStatus::ValidatedFailure,
    ];
    
    for status in statuses {
        let serialized = serde_json::to_string(&status)?;
        let deserialized: TransactionStatus = serde_json::from_str(&serialized)?;
        assert_eq!(status, deserialized);
    }
    
    Ok(())
}

#[test]
fn test_invalid_proof_validation() -> Result<()> {
    use std::fs;
    
    let mut test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_path.push("tests/data/invalid_proof.json");
    
    let proof_data = fs::read_to_string(&test_data_path)?;
    let proof: ProofData = serde_json::from_str(&proof_data)?;
    
    // Verify that invalid proof has empty fields
    assert!(proof.proof.is_empty());
    assert!(proof.public_inputs.is_empty());
    assert!(proof.verification_key.is_empty());
    assert!(proof.circuit_id.is_empty());
    
    Ok(())
}

#[test]
fn test_multi_chain_config() {
    use std::collections::HashMap;
    
    let mut chains = HashMap::new();
    
    chains.insert("ethereum".to_string(), ChainConfig {
        name: "ethereum".to_string(),
        chain_id: 1,
        rpc_url: "https://eth-mainnet.g.alchemy.com/v2/demo".to_string(),
        explorer_url: "https://etherscan.io".to_string(),
        gas_price_multiplier: 1.1,
        confirmation_blocks: 12,
    });
    
    chains.insert("polygon".to_string(), ChainConfig {
        name: "polygon".to_string(),
        chain_id: 137,
        rpc_url: "https://polygon-rpc.com".to_string(),
        explorer_url: "https://polygonscan.com".to_string(),
        gas_price_multiplier: 1.2,
        confirmation_blocks: 20,
    });
    
    let multi_config = MultiChainConfig {
        chains: chains.clone(),
        default_gas_limits: HashMap::new(),
        global_settings: GlobalSettings::default(),
    };
    
    assert_eq!(multi_config.chains.len(), 2);
    assert!(multi_config.chains.contains_key("ethereum"));
    assert!(multi_config.chains.contains_key("polygon"));
}
