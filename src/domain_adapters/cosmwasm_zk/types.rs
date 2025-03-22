use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::vm::zk_integration::{Proof, Witness, PublicInputs as ZkPublicInputs};
use crate::domain_adapters::DomainAdapter;
use crate::error::{Error, Result};

/// Result of verification process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationResult {
    /// The verification was successful
    Valid,
    /// The verification failed
    Invalid(String),
}

/// Detailed verification result for CosmWasm ZK proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedVerificationResult {
    /// Whether the proof is valid
    pub is_valid: bool,
    
    /// Time taken to verify in milliseconds
    pub verification_time_ms: u64,
    
    /// Verification message
    pub message: String,
}

impl From<VerificationResult> for DetailedVerificationResult {
    fn from(result: VerificationResult) -> Self {
        match result {
            VerificationResult::Valid => DetailedVerificationResult {
                is_valid: true,
                verification_time_ms: 0, // Default value
                message: "Proof verified successfully".to_string(),
            },
            VerificationResult::Invalid(reason) => DetailedVerificationResult {
                is_valid: false,
                verification_time_ms: 0, // Default value
                message: format!("Proof verification failed: {}", reason),
            },
        }
    }
}

impl From<DetailedVerificationResult> for VerificationResult {
    fn from(result: DetailedVerificationResult) -> Self {
        if result.is_valid {
            VerificationResult::Valid
        } else {
            VerificationResult::Invalid(result.message)
        }
    }
}

/// A CosmWasm ZK Program represents a compiled WASM program with ZK verification
#[derive(Debug, Clone)]
pub struct CosmWasmZkProgram {
    /// Unique identifier for the program
    pub id: String,
    
    /// Compiled WASM bytecode
    pub bytecode: Vec<u8>,
    
    /// Verification information for the program
    pub verification_key: Vec<u8>,
    
    /// Source hash for the program
    pub source_hash: String,
    
    /// Metadata for the program
    pub metadata: HashMap<String, String>,
}

impl CosmWasmZkProgram {
    /// Create a new CosmWasm ZK program
    pub fn new(
        id: String,
        bytecode: Vec<u8>,
        verification_key: Vec<u8>,
        source_hash: String,
    ) -> Self {
        Self {
            id,
            bytecode,
            verification_key,
            source_hash,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the program
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// A CosmWasm ZK Contract represents a deployed contract on a CosmWasm chain
#[derive(Debug, Clone)]
pub struct CosmWasmZkContract {
    /// Contract address on the chain
    pub address: String,
    
    /// Reference to the program used to create this contract
    pub program_id: String,
    
    /// Chain ID where this contract is deployed
    pub chain_id: String,
    
    /// Code ID on the chain
    pub code_id: u64,
    
    /// Instantiation info
    pub init_msg: Option<String>,
    
    /// Metadata for the contract
    pub metadata: HashMap<String, String>,
}

impl CosmWasmZkContract {
    /// Create a new CosmWasm ZK contract
    pub fn new(
        address: String,
        program_id: String,
        chain_id: String,
        code_id: u64,
    ) -> Self {
        Self {
            address,
            program_id,
            chain_id,
            code_id,
            init_msg: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the initialization message
    pub fn with_init_msg(mut self, init_msg: String) -> Self {
        self.init_msg = Some(init_msg);
        self
    }
    
    /// Add metadata to the contract
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Public inputs for a CosmWasm contract call verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmPublicInputs {
    /// Contract address
    pub contract_address: String,
    
    /// Method name being called
    pub method: String,
    
    /// Chain ID where the contract is deployed
    pub chain_id: String,
    
    /// Input parameters encoded as JSON
    pub inputs: String,
    
    /// Expected output encoded as JSON
    pub expected_output: Option<String>,
    
    /// Additional data needed for verification
    pub additional_data: HashMap<String, String>,
}

impl CosmWasmPublicInputs {
    /// Create new public inputs for verification
    pub fn new(
        contract_address: String,
        method: String,
        chain_id: String,
        inputs: String,
    ) -> Self {
        Self {
            contract_address,
            method,
            chain_id,
            inputs,
            expected_output: None,
            additional_data: HashMap::new(),
        }
    }
    
    /// Set expected output for verification
    pub fn with_expected_output(mut self, output: String) -> Self {
        self.expected_output = Some(output);
        self
    }
    
    /// Add additional data for verification
    pub fn with_additional_data(mut self, key: String, value: String) -> Self {
        self.additional_data.insert(key, value);
        self
    }
}

impl From<CosmWasmPublicInputs> for ZkPublicInputs {
    fn from(inputs: CosmWasmPublicInputs) -> Self {
        let mut zk_inputs = ZkPublicInputs::new();
        
        // Convert all fields to the ZkPublicInputs format
        zk_inputs.insert("contract_address".to_string(), inputs.contract_address);
        zk_inputs.insert("method".to_string(), inputs.method);
        zk_inputs.insert("chain_id".to_string(), inputs.chain_id);
        zk_inputs.insert("inputs".to_string(), inputs.inputs);
        
        if let Some(output) = inputs.expected_output {
            zk_inputs.insert("expected_output".to_string(), output);
        }
        
        // Add any additional data
        for (key, value) in inputs.additional_data {
            zk_inputs.insert(key, value);
        }
        
        zk_inputs
    }
}

/// Call data for executing a CosmWasm contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmCallData {
    /// Contract address
    pub contract_address: String,
    
    /// Method name to call
    pub method: String,
    
    /// Input parameters encoded as JSON
    pub inputs: String,
    
    /// Optional funds to send with the call
    pub funds: Option<Vec<Coin>>,
    
    /// Options for the call
    pub options: HashMap<String, String>,
}

/// Representation of a coin in the CosmWasm ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    /// Denomination of the coin
    pub denom: String,
    
    /// Amount as a string to avoid precision issues
    pub amount: String,
}

/// Domain types for CosmWasm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CosmWasmDomainType {
    /// A standard CosmWasm chain
    Standard,
    
    /// Cosmos Hub
    CosmosHub,
    
    /// Osmosis DEX
    Osmosis,
    
    /// Injective Protocol
    Injective,
    
    /// A custom chain
    Custom(String),
} 