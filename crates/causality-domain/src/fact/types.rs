// Fact type definitions for domains
// Original file: src/domain/fact/types.rs

// Fact Types Module for Causality
//
// This module defines the types used for facts in Causality.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Query for facts in a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactQuery {
    /// Fact type to query
    pub fact_type: String,
    /// Query parameters
    pub parameters: HashMap<String, String>,
    /// Whether verification is required
    pub requires_verification: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl FactQuery {
    /// Create a new fact query
    pub fn new(fact_type: impl Into<String>) -> Self {
        Self {
            fact_type: fact_type.into(),
            parameters: HashMap::new(),
            requires_verification: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Add a parameter to the query
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set verification requirements
    pub fn with_verification(mut self, required: bool) -> Self {
        self.requires_verification = required;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Type of observed fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactType {
    /// Boolean fact (true/false)
    Boolean(bool),
    /// Numeric fact
    Numeric(i64),
    /// String fact
    String(String),
    /// Binary data fact
    Binary(Vec<u8>),
    /// JSON fact
    Json(serde_json::Value),
}

/// Fact with a register proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterFact {
    /// The fact value
    pub fact: FactType,
    /// Proof data
    pub proof: Vec<u8>,
}

/// Fact with a zero-knowledge proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProofFact {
    /// The fact value
    pub fact: FactType,
    /// Proof data
    pub proof: Vec<u8>,
    /// Verification key
    pub verification_key: Vec<u8>,
}
