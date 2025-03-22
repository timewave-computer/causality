//! Ethereum Effect Adapter Schema
//!
//! This module provides a ready-to-use schema for the Ethereum blockchain domain.

use std::collections::HashMap;
use crate::types::DomainId;
use super::{
    AdapterSchema, EffectDefinition, FactDefinition, ProofDefinition,
    TimeSyncDefinition, RpcDefinition,
};

/// Create a new Ethereum adapter schema with standard definitions
pub fn create_ethereum_schema() -> AdapterSchema {
    let mut schema = AdapterSchema::new(DomainId::new("ethereum"), "blockchain");
    
    // Add Ethereum RPC interface
    schema.add_rpc_interface(create_ethereum_rpc_interface());
    
    // Add Ethereum time sync settings
    schema.set_time_sync(create_ethereum_time_sync());
    
    // Add standard Ethereum effects
    schema.add_effect(create_transfer_effect());
    schema.add_effect(create_contract_deploy_effect());
    schema.add_effect(create_contract_call_effect());
    
    // Add standard Ethereum facts
    schema.add_fact(create_balance_fact());
    schema.add_fact(create_transaction_fact());
    schema.add_fact(create_block_fact());
    schema.add_fact(create_contract_state_fact());
    
    // Add standard Ethereum proofs
    schema.add_proof(create_transaction_proof());
    schema.add_proof(create_receipt_proof());
    schema.add_proof(create_account_proof());
    
    // Add common metadata
    schema.add_metadata("chain_id", "1");
    schema.add_metadata("network", "mainnet");
    schema.add_metadata("consensus", "proof-of-stake");
    
    schema
}

/// Create Ethereum JSON-RPC interface definition
fn create_ethereum_rpc_interface() -> RpcDefinition {
    RpcDefinition {
        name: "ethereum-json-rpc".to_string(),
        protocol: "http".to_string(),
        endpoint_template: "https://{network}.infura.io/v3/{api_key}".to_string(),
        auth_method: Some("api_key".to_string()),
        rate_limit: Some(100),
        timeout_ms: Some(10000),
        methods: {
            let mut methods = HashMap::new();
            // Transaction methods
            methods.insert("eth_sendRawTransaction".to_string(), "POST".to_string());
            methods.insert("eth_sendTransaction".to_string(), "POST".to_string());
            methods.insert("eth_call".to_string(), "POST".to_string());
            methods.insert("eth_estimateGas".to_string(), "POST".to_string());
            methods.insert("eth_getTransactionCount".to_string(), "POST".to_string());
            methods.insert("eth_getTransactionByHash".to_string(), "POST".to_string());
            methods.insert("eth_getTransactionReceipt".to_string(), "POST".to_string());
            
            // Block methods
            methods.insert("eth_blockNumber".to_string(), "POST".to_string());
            methods.insert("eth_getBlockByNumber".to_string(), "POST".to_string());
            methods.insert("eth_getBlockByHash".to_string(), "POST".to_string());
            
            // State methods
            methods.insert("eth_getBalance".to_string(), "POST".to_string());
            methods.insert("eth_getCode".to_string(), "POST".to_string());
            methods.insert("eth_getStorageAt".to_string(), "POST".to_string());
            
            // Chain methods
            methods.insert("net_version".to_string(), "POST".to_string());
            methods.insert("eth_chainId".to_string(), "POST".to_string());
            
            // Gas price methods
            methods.insert("eth_gasPrice".to_string(), "POST".to_string());
            methods.insert("eth_maxPriorityFeePerGas".to_string(), "POST".to_string());
            methods.insert("eth_feeHistory".to_string(), "POST".to_string());
            
            methods
        },
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("version".to_string(), "1.0".to_string());
            metadata.insert("spec".to_string(), "ethereum-json-rpc".to_string());
            metadata
        },
    }
}

/// Create Ethereum time synchronization settings
fn create_ethereum_time_sync() -> TimeSyncDefinition {
    TimeSyncDefinition {
        time_model: "block-based".to_string(),
        time_point_call: "eth_blockNumber".to_string(),
        finality_window: Some(12),
        block_time: Some(12),
        drift_tolerance: Some(60),
        time_format: "number".to_string(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("blockTimestampFormat".to_string(), "unix".to_string());
            metadata
        },
    }
}

/// Create Ethereum transfer effect definition
fn create_transfer_effect() -> EffectDefinition {
    EffectDefinition {
        effect_type: "transfer".to_string(),
        tx_format: "RLP".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_sendRawTransaction".to_string(),
        required_fields: vec![
            "from".to_string(),
            "to".to_string(),
            "value".to_string(),
        ],
        optional_fields: vec![
            "gas".to_string(),
            "gasPrice".to_string(),
            "maxFeePerGas".to_string(),
            "maxPriorityFeePerGas".to_string(),
            "nonce".to_string(),
            "data".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("source".to_string(), "from".to_string());
            mappings.insert("destination".to_string(), "to".to_string());
            mappings.insert("amount".to_string(), "value".to_string());
            mappings
        },
        serialization: Some("ethereum_tx".to_string()),
        gas_estimation: Some("21000 + (data.len() * 16)".to_string()),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("eip1559".to_string(), "true".to_string());
            metadata
        },
    }
}

/// Create Ethereum contract deployment effect definition
fn create_contract_deploy_effect() -> EffectDefinition {
    EffectDefinition {
        effect_type: "contract_deploy".to_string(),
        tx_format: "RLP".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_sendRawTransaction".to_string(),
        required_fields: vec![
            "from".to_string(),
            "data".to_string(),
        ],
        optional_fields: vec![
            "gas".to_string(),
            "gasPrice".to_string(),
            "maxFeePerGas".to_string(),
            "maxPriorityFeePerGas".to_string(),
            "value".to_string(),
            "nonce".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("source".to_string(), "from".to_string());
            mappings.insert("bytecode".to_string(), "data".to_string());
            mappings
        },
        serialization: Some("ethereum_tx".to_string()),
        gas_estimation: Some("21000 + (data.len() * 200)".to_string()),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("eip1559".to_string(), "true".to_string());
            metadata
        },
    }
}

/// Create Ethereum contract call effect definition
fn create_contract_call_effect() -> EffectDefinition {
    EffectDefinition {
        effect_type: "contract_call".to_string(),
        tx_format: "RLP".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_sendRawTransaction".to_string(),
        required_fields: vec![
            "from".to_string(),
            "to".to_string(),
            "data".to_string(),
        ],
        optional_fields: vec![
            "gas".to_string(),
            "gasPrice".to_string(),
            "maxFeePerGas".to_string(),
            "maxPriorityFeePerGas".to_string(),
            "value".to_string(),
            "nonce".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("source".to_string(), "from".to_string());
            mappings.insert("contract".to_string(), "to".to_string());
            mappings.insert("calldata".to_string(), "data".to_string());
            mappings
        },
        serialization: Some("ethereum_tx".to_string()),
        gas_estimation: Some("21000 + (data.len() * 16)".to_string()),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("eip1559".to_string(), "true".to_string());
            metadata
        },
    }
}

/// Create Ethereum balance fact definition
fn create_balance_fact() -> FactDefinition {
    FactDefinition {
        fact_type: "balance".to_string(),
        data_format: "json".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_getBalance".to_string(),
        required_fields: vec![
            "address".to_string(),
            "blockNumber".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("account".to_string(), "address".to_string());
            mappings.insert("block".to_string(), "blockNumber".to_string());
            mappings
        },
        update_frequency: Some(12),
        extraction_rules: Some("hex_to_decimal".to_string()),
        metadata: HashMap::new(),
    }
}

/// Create Ethereum transaction fact definition
fn create_transaction_fact() -> FactDefinition {
    FactDefinition {
        fact_type: "transaction".to_string(),
        data_format: "json".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_getTransactionByHash".to_string(),
        required_fields: vec![
            "txHash".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("id".to_string(), "txHash".to_string());
            mappings
        },
        update_frequency: Some(1),
        extraction_rules: None,
        metadata: HashMap::new(),
    }
}

/// Create Ethereum block fact definition
fn create_block_fact() -> FactDefinition {
    FactDefinition {
        fact_type: "block".to_string(),
        data_format: "json".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_getBlockByNumber".to_string(),
        required_fields: vec![
            "blockNumber".to_string(),
            "fullTransactions".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("number".to_string(), "blockNumber".to_string());
            mappings.insert("includeTx".to_string(), "fullTransactions".to_string());
            mappings
        },
        update_frequency: Some(12),
        extraction_rules: None,
        metadata: HashMap::new(),
    }
}

/// Create Ethereum contract state fact definition
fn create_contract_state_fact() -> FactDefinition {
    FactDefinition {
        fact_type: "contract_state".to_string(),
        data_format: "json".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_call".to_string(),
        required_fields: vec![
            "to".to_string(),
            "data".to_string(),
            "blockNumber".to_string(),
        ],
        field_mappings: {
            let mut mappings = HashMap::new();
            mappings.insert("contract".to_string(), "to".to_string());
            mappings.insert("calldata".to_string(), "data".to_string());
            mappings.insert("block".to_string(), "blockNumber".to_string());
            mappings
        },
        update_frequency: Some(12),
        extraction_rules: None,
        metadata: HashMap::new(),
    }
}

/// Create Ethereum transaction proof definition
fn create_transaction_proof() -> ProofDefinition {
    ProofDefinition {
        proof_type: "transaction".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_getTransactionByHash".to_string(),
        verification_method: "verify_transaction_inclusion".to_string(),
        required_fields: vec![
            "txHash".to_string(),
        ],
        metadata: HashMap::new(),
    }
}

/// Create Ethereum receipt proof definition
fn create_receipt_proof() -> ProofDefinition {
    ProofDefinition {
        proof_type: "receipt".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_getTransactionReceipt".to_string(),
        verification_method: "verify_receipt_inclusion".to_string(),
        required_fields: vec![
            "txHash".to_string(),
        ],
        metadata: HashMap::new(),
    }
}

/// Create Ethereum account proof definition
fn create_account_proof() -> ProofDefinition {
    ProofDefinition {
        proof_type: "account".to_string(),
        proof_format: "MPT".to_string(),
        rpc_call: "eth_getProof".to_string(),
        verification_method: "verify_account_inclusion".to_string(),
        required_fields: vec![
            "address".to_string(),
            "storageKeys".to_string(),
            "blockNumber".to_string(),
        ],
        metadata: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ethereum_schema_creation() {
        let schema = create_ethereum_schema();
        
        // Test schema structure
        assert_eq!(schema.domain_id.as_ref(), "ethereum");
        assert_eq!(schema.domain_type, "blockchain");
        
        // Test effect definitions
        assert_eq!(schema.effects.len(), 3);
        assert!(schema.effects.iter().any(|e| e.effect_type == "transfer"));
        assert!(schema.effects.iter().any(|e| e.effect_type == "contract_deploy"));
        assert!(schema.effects.iter().any(|e| e.effect_type == "contract_call"));
        
        // Test fact definitions
        assert_eq!(schema.facts.len(), 4);
        assert!(schema.facts.iter().any(|f| f.fact_type == "balance"));
        assert!(schema.facts.iter().any(|f| f.fact_type == "transaction"));
        assert!(schema.facts.iter().any(|f| f.fact_type == "block"));
        assert!(schema.facts.iter().any(|f| f.fact_type == "contract_state"));
        
        // Test proof definitions
        assert_eq!(schema.proofs.len(), 3);
        assert!(schema.proofs.iter().any(|p| p.proof_type == "transaction"));
        assert!(schema.proofs.iter().any(|p| p.proof_type == "receipt"));
        assert!(schema.proofs.iter().any(|p| p.proof_type == "account"));
        
        // Test RPC interfaces
        assert_eq!(schema.rpc_interfaces.len(), 1);
        assert_eq!(schema.rpc_interfaces[0].name, "ethereum-json-rpc");
        
        // Test time sync
        assert_eq!(schema.time_sync.time_model, "block-based");
        assert_eq!(schema.time_sync.finality_window, Some(12));
        
        // Validate the schema
        assert!(schema.validate().is_ok());
    }
} 