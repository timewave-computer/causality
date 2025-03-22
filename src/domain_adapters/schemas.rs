//! Effect Adapter Schemas
//!
//! This module provides schema definitions for effect adapters.
//! Schemas define the structure and behavior of an adapter for a specific Domain.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::path::Path;
use std::fs;

/// Custom domain ID type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DomainId(String);

impl DomainId {
    /// Create a new domain ID
    pub fn new(id: impl Into<String>) -> Self {
        DomainId(id.into())
    }
    
    /// Get the domain ID as a string reference
    pub fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DomainId {
    fn from(s: &str) -> Self {
        DomainId(s.to_string())
    }
}

impl From<String> for DomainId {
    fn from(s: String) -> Self {
        DomainId(s)
    }
}

/// Error type for schema operations
#[derive(Debug, Error)]
pub enum Error {
    /// Error during schema parsing
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Error during schema validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error during schema serialization
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Error during deserialization
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    /// Error during I/O operations
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Effect definition in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDefinition {
    /// Effect type
    pub effect_type: String,
    /// Transaction format
    pub tx_format: String,
    /// Proof format
    pub proof_format: String,
    /// RPC call to execute the effect
    pub rpc_call: String,
    /// Required fields for this effect
    pub required_fields: Vec<String>,
    /// Optional fields for this effect
    pub optional_fields: Vec<String>,
    /// Field transformations (mapping from schema field to Domain field)
    pub field_mappings: HashMap<String, String>,
    /// Custom serialization rules
    pub serialization: Option<String>,
    /// Gas estimation formula
    pub gas_estimation: Option<String>,
    /// Additional effect metadata
    pub metadata: HashMap<String, String>,
}

/// Fact definition in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactDefinition {
    /// Fact type
    pub fact_type: String,
    /// Data format
    pub data_format: String,
    /// Proof format
    pub proof_format: String,
    /// RPC call to observe the fact
    pub rpc_call: String,
    /// Required fields for this fact
    pub required_fields: Vec<String>,
    /// Field transformations (mapping from Domain field to schema field)
    pub field_mappings: HashMap<String, String>,
    /// Update frequency (in seconds)
    pub update_frequency: Option<u64>,
    /// Custom extraction rules
    pub extraction_rules: Option<String>,
    /// Additional fact metadata
    pub metadata: HashMap<String, String>,
}

/// Proof definition in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDefinition {
    /// Proof type
    pub proof_type: String,
    /// Proof format
    pub proof_format: String,
    /// RPC call to retrieve the proof
    pub rpc_call: String,
    /// Verification method
    pub verification_method: String,
    /// Required fields for this proof
    pub required_fields: Vec<String>,
    /// Additional proof metadata
    pub metadata: HashMap<String, String>,
}

/// Time synchronization settings in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSyncDefinition {
    /// Time model (block-based, timestamp-based, etc.)
    pub time_model: String,
    /// RPC call to get the current time point
    pub time_point_call: String,
    /// Finality window (number of blocks or time for finality)
    pub finality_window: Option<u64>,
    /// Block time (in seconds)
    pub block_time: Option<u64>,
    /// Time drift tolerance (in seconds)
    pub drift_tolerance: Option<u64>,
    /// Time format
    pub time_format: String,
    /// Additional time synchronization metadata
    pub metadata: HashMap<String, String>,
}

/// RPC interface definition in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcDefinition {
    /// RPC interface name
    pub name: String,
    /// RPC protocol (HTTP, WebSocket, etc.)
    pub protocol: String,
    /// Endpoint path template
    pub endpoint_template: String,
    /// Authentication method
    pub auth_method: Option<String>,
    /// Rate limiting settings
    pub rate_limit: Option<u64>,
    /// Request timeout (in milliseconds)
    pub timeout_ms: Option<u64>,
    /// Available methods
    pub methods: HashMap<String, String>,
    /// Additional RPC interface metadata
    pub metadata: HashMap<String, String>,
}

/// Adapter schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterSchema {
    /// Domain ID
    pub domain_id: DomainId,
    /// Schema version
    pub version: String,
    /// Domain type
    pub domain_type: String,
    /// Effect definitions
    pub effects: Vec<EffectDefinition>,
    /// Fact definitions
    pub facts: Vec<FactDefinition>,
    /// Proof definitions
    pub proofs: Vec<ProofDefinition>,
    /// Time synchronization settings
    pub time_sync: TimeSyncDefinition,
    /// RPC interface definitions
    pub rpc_interfaces: Vec<RpcDefinition>,
    /// Custom code snippets
    pub custom_code: HashMap<String, String>,
    /// Additional schema metadata
    pub metadata: HashMap<String, String>,
}

impl AdapterSchema {
    /// Create a new adapter schema
    pub fn new(domain_id: impl Into<DomainId>, domain_type: impl Into<String>) -> Self {
        AdapterSchema {
            domain_id: domain_id.into(),
            version: "1.0.0".to_string(),
            domain_type: domain_type.into(),
            effects: Vec::new(),
            facts: Vec::new(),
            proofs: Vec::new(),
            time_sync: TimeSyncDefinition {
                time_model: "block-based".to_string(),
                time_point_call: "getBlockNumber".to_string(),
                finality_window: None,
                block_time: None,
                drift_tolerance: None,
                time_format: "number".to_string(),
                metadata: HashMap::new(),
            },
            rpc_interfaces: Vec::new(),
            custom_code: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add an effect definition to the schema
    pub fn add_effect(&mut self, effect: EffectDefinition) -> &mut Self {
        self.effects.push(effect);
        self
    }
    
    /// Add a fact definition to the schema
    pub fn add_fact(&mut self, fact: FactDefinition) -> &mut Self {
        self.facts.push(fact);
        self
    }
    
    /// Add a proof definition to the schema
    pub fn add_proof(&mut self, proof: ProofDefinition) -> &mut Self {
        self.proofs.push(proof);
        self
    }
    
    /// Set the time synchronization settings
    pub fn set_time_sync(&mut self, time_sync: TimeSyncDefinition) -> &mut Self {
        self.time_sync = time_sync;
        self
    }
    
    /// Add an RPC interface definition to the schema
    pub fn add_rpc_interface(&mut self, rpc: RpcDefinition) -> &mut Self {
        self.rpc_interfaces.push(rpc);
        self
    }
    
    /// Add a custom code snippet to the schema
    pub fn add_custom_code(&mut self, name: impl Into<String>, code: impl Into<String>) -> &mut Self {
        self.custom_code.insert(name.into(), code.into());
        self
    }
    
    /// Add metadata to the schema
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Validate the schema for completeness and correctness
    pub fn validate(&self) -> Result<(), Error> {
        // Check that each effect has appropriate RPC methods
        for effect in &self.effects {
            if !self.has_rpc_method(&effect.rpc_call) {
                return Err(Error::ValidationError(format!(
                    "Effect '{}' uses RPC method '{}' which is not defined in any RPC interface",
                    effect.effect_type, effect.rpc_call
                )));
            }
        }
        
        // Check that each fact has appropriate RPC methods
        for fact in &self.facts {
            if !self.has_rpc_method(&fact.rpc_call) {
                return Err(Error::ValidationError(format!(
                    "Fact '{}' uses RPC method '{}' which is not defined in any RPC interface",
                    fact.fact_type, fact.rpc_call
                )));
            }
        }
        
        // Check that each proof has appropriate RPC methods
        for proof in &self.proofs {
            if !self.has_rpc_method(&proof.rpc_call) {
                return Err(Error::ValidationError(format!(
                    "Proof '{}' uses RPC method '{}' which is not defined in any RPC interface",
                    proof.proof_type, proof.rpc_call
                )));
            }
        }
        
        // Check that the time sync has appropriate RPC methods
        if !self.has_rpc_method(&self.time_sync.time_point_call) {
            return Err(Error::ValidationError(format!(
                "Time sync uses RPC method '{}' which is not defined in any RPC interface",
                self.time_sync.time_point_call
            )));
        }
        
        Ok(())
    }
    
    // Check if an RPC method is defined in any of the RPC interfaces
    fn has_rpc_method(&self, method_name: &str) -> bool {
        for rpc in &self.rpc_interfaces {
            if rpc.methods.contains_key(method_name) {
                return true;
            }
        }
        false
    }
    
    /// Create an adapter schema from TOML content
    pub fn from_toml(content: &str) -> Result<Self, Error> {
        match toml::from_str::<AdapterSchema>(content) {
            Ok(schema) => Ok(schema),
            Err(e) => Err(Error::ParseError(format!("Failed to parse TOML: {}", e))),
        }
    }
    
    /// Convert schema to TOML
    pub fn to_toml(&self) -> Result<String, Error> {
        toml::to_string(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize to TOML: {}", e)))
    }
    
    /// Load a schema from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        Self::from_toml(&content)
    }
    
    /// Save a schema to a file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let content = self.to_toml()?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Create an example Ethereum schema
pub fn create_ethereum_schema() -> AdapterSchema {
    let mut schema = AdapterSchema::new("ethereum", "blockchain");
    
    // Add an RPC interface
    let mut methods = HashMap::new();
    methods.insert("eth_sendTransaction".to_string(), "Send a transaction".to_string());
    methods.insert("eth_getTransactionReceipt".to_string(), "Get transaction receipt".to_string());
    methods.insert("eth_blockNumber".to_string(), "Get current block number".to_string());
    methods.insert("eth_getBalance".to_string(), "Get account balance".to_string());
    
    let rpc = RpcDefinition {
        name: "Ethereum JSON-RPC".to_string(),
        protocol: "HTTP".to_string(),
        endpoint_template: "{{node_url}}".to_string(),
        auth_method: None,
        rate_limit: Some(100),
        timeout_ms: Some(30000),
        methods,
        metadata: HashMap::new(),
    };
    
    schema.add_rpc_interface(rpc);
    
    // Add a transfer effect
    let mut field_mappings = HashMap::new();
    field_mappings.insert("to".to_string(), "to".to_string());
    field_mappings.insert("value".to_string(), "value".to_string());
    field_mappings.insert("gas".to_string(), "gas".to_string());
    
    let effect = EffectDefinition {
        effect_type: "transfer".to_string(),
        tx_format: "json".to_string(),
        proof_format: "receipt".to_string(),
        rpc_call: "eth_sendTransaction".to_string(),
        required_fields: vec!["to".to_string(), "value".to_string()],
        optional_fields: vec!["gas".to_string(), "gasPrice".to_string()],
        field_mappings,
        serialization: None,
        gas_estimation: Some("21000".to_string()),
        metadata: HashMap::new(),
    };
    
    schema.add_effect(effect);
    
    // Add a balance fact
    let mut field_mappings = HashMap::new();
    field_mappings.insert("address".to_string(), "address".to_string());
    
    let fact = FactDefinition {
        fact_type: "balance".to_string(),
        data_format: "hex".to_string(),
        proof_format: "none".to_string(),
        rpc_call: "eth_getBalance".to_string(),
        required_fields: vec!["address".to_string()],
        field_mappings,
        update_frequency: Some(10),
        extraction_rules: None,
        metadata: HashMap::new(),
    };
    
    schema.add_fact(fact);
    
    // Add a transaction proof
    let proof = ProofDefinition {
        proof_type: "transaction".to_string(),
        proof_format: "json".to_string(),
        rpc_call: "eth_getTransactionReceipt".to_string(),
        verification_method: "status_check".to_string(),
        required_fields: vec!["txHash".to_string()],
        metadata: HashMap::new(),
    };
    
    schema.add_proof(proof);
    
    // Set time sync
    let time_sync = TimeSyncDefinition {
        time_model: "block-based".to_string(),
        time_point_call: "eth_blockNumber".to_string(),
        finality_window: Some(12),
        block_time: Some(15),
        drift_tolerance: Some(60),
        time_format: "number".to_string(),
        metadata: HashMap::new(),
    };
    
    schema.set_time_sync(time_sync);
    
    // Add custom code
    schema.add_custom_code("utils", "function hexToNumber(hex) { return parseInt(hex, 16); }");
    
    // Add metadata
    schema.add_metadata("chain_id", "1");
    schema.add_metadata("network", "mainnet");
    
    schema
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adapter_schema_creation() {
        let schema = create_ethereum_schema();
        
        assert_eq!(schema.domain_id.as_ref(), "ethereum");
        assert_eq!(schema.domain_type, "blockchain");
        assert_eq!(schema.version, "1.0.0");
        
        assert_eq!(schema.effects.len(), 1);
        assert_eq!(schema.facts.len(), 1);
        assert_eq!(schema.proofs.len(), 1);
        
        assert_eq!(schema.rpc_interfaces.len(), 1);
        assert_eq!(schema.time_sync.time_model, "block-based");
        
        assert_eq!(schema.custom_code.len(), 1);
        assert_eq!(schema.metadata.len(), 2);
    }
    
    #[test]
    fn test_schema_validation() {
        let schema = create_ethereum_schema();
        assert!(schema.validate().is_ok());
    }
    
    #[test]
    fn test_schema_to_toml() {
        let schema = create_ethereum_schema();
        let toml = schema.to_toml().unwrap();
        
        assert!(toml.contains("domain_id"));
        assert!(toml.contains("ethereum"));
        assert!(toml.contains("blockchain"));
        
        // Parse it back
        let parsed = AdapterSchema::from_toml(&toml).unwrap();
        assert_eq!(parsed.domain_id.as_ref(), schema.domain_id.as_ref());
    }
}

// Example schema modules
pub mod ethereum {
    use super::*;
    
    pub fn create_ethereum_schema() -> AdapterSchema {
        super::create_ethereum_schema()
    }
} 