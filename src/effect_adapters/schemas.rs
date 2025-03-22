//! Effect Adapter Schemas
//!
//! This module provides schema definitions for effect adapters.
//! Schemas define the structure and behavior of an adapter for a specific Domain.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::types::DomainId;
use crate::error::{Error, Result};

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
    pub fn new(domain_id: DomainId, domain_type: impl Into<String>) -> Self {
        AdapterSchema {
            domain_id,
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
    pub fn validate(&self) -> Result<()> {
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
        
        // Check that time sync has appropriate RPC methods
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
    
    /// Load a schema from TOML
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        let schema: AdapterSchema = toml::from_str(toml_str)
            .map_err(|e| Error::DeserializationError(format!("Failed to parse TOML: {}", e)))?;
        
        // Validate the schema
        schema.validate()?;
        
        Ok(schema)
    }
    
    /// Convert schema to TOML
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize to TOML: {}", e)))
    }

    /// Create an adapter schema from TOML content
    pub fn from_toml(content: &str) -> Result<Self, Error> {
        match toml::from_str::<AdapterSchema>(content) {
            Ok(schema) => Ok(schema),
            Err(e) => Err(Error::ParseError(format!("Failed to parse TOML: {}", e))),
        }
    }
}

// Example schema modules
pub mod ethereum;
// TODO: Add more schema modules as they are implemented
// pub mod solana;
// pub mod rest;
// pub mod substrate;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adapter_schema_creation() {
        let mut schema = AdapterSchema::new(DomainId::new("ethereum"), "blockchain");
        
        // Add an effect definition
        schema.add_effect(EffectDefinition {
            effect_type: "transfer".to_string(),
            tx_format: "RLP".to_string(),
            proof_format: "MPT".to_string(),
            rpc_call: "eth_sendTransaction".to_string(),
            required_fields: vec!["from".to_string(), "to".to_string(), "value".to_string()],
            optional_fields: vec!["gas".to_string(), "gasPrice".to_string()],
            field_mappings: HashMap::new(),
            serialization: None,
            gas_estimation: None,
            metadata: HashMap::new(),
        });
        
        // Add an RPC interface with the required method
        schema.add_rpc_interface(RpcDefinition {
            name: "ethereum-json-rpc".to_string(),
            protocol: "http".to_string(),
            endpoint_template: "https://{network}.infura.io".to_string(),
            auth_method: None,
            rate_limit: None,
            timeout_ms: None,
            methods: {
                let mut methods = HashMap::new();
                methods.insert("eth_sendTransaction".to_string(), "POST".to_string());
                methods.insert("eth_getBlockNumber".to_string(), "POST".to_string());
                methods
            },
            metadata: HashMap::new(),
        });
        
        // Set time sync settings
        schema.set_time_sync(TimeSyncDefinition {
            time_model: "block-based".to_string(),
            time_point_call: "eth_getBlockNumber".to_string(),
            finality_window: Some(12),
            block_time: Some(15),
            drift_tolerance: Some(60),
            time_format: "number".to_string(),
            metadata: HashMap::new(),
        });
        
        // Validate the schema
        assert!(schema.validate().is_ok());
        
        // Test TOML serialization
        let toml = schema.to_toml().unwrap();
        let from_toml = AdapterSchema::from_toml(&toml).unwrap();
        
        assert_eq!(from_toml.domain_id.as_ref(), "ethereum");
        assert_eq!(from_toml.domain_type, "blockchain");
        assert_eq!(from_toml.effects.len(), 1);
        assert_eq!(from_toml.effects[0].effect_type, "transfer");
        assert_eq!(from_toml.effects[0].required_fields.len(), 3);
    }
    
    #[test]
    fn test_schema_validation_failure() {
        let mut schema = AdapterSchema::new(DomainId::new("ethereum"), "blockchain");
        
        // Add an effect definition with a non-existent RPC method
        schema.add_effect(EffectDefinition {
            effect_type: "transfer".to_string(),
            tx_format: "RLP".to_string(),
            proof_format: "MPT".to_string(),
            rpc_call: "eth_sendTransaction".to_string(), // Not defined in any RPC interface
            required_fields: vec!["from".to_string(), "to".to_string(), "value".to_string()],
            optional_fields: vec!["gas".to_string(), "gasPrice".to_string()],
            field_mappings: HashMap::new(),
            serialization: None,
            gas_estimation: None,
            metadata: HashMap::new(),
        });
        
        // Validation should fail
        assert!(schema.validate().is_err());
    }
} 