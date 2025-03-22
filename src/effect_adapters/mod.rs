//! Effect Adapter Generation System
//!
//! This module provides the framework for generating and using effect adapters
//! that connect Causality programs to external Domains (blockchains, APIs, etc.).
//!
//! Effect adapters serve as the boundary between programs and external systems,
//! handling:
//! - Encoding outgoing effects into Domain-specific transactions
//! - Validating incoming proofs and facts from external Domains
//! - Converting external facts into canonical Causality facts
//! - Preserving external time observations into time map snapshots

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::path::Path;

use crate::error::{Error, Result};
use crate::types::{ContentId, DomainId};
use crate::log::fact_types::FactType;
#[cfg(feature = "domain")]
use crate::domain::map::{TimeMap, TimePoint};
#[cfg(feature = "domain")]
use crate::domain_adapters::schemas::{AdapterSchema, EffectDefinition, FactDefinition, ProofDefinition};

// Effect adapter code generation
pub mod codegen;

// Hash and content addressing
pub mod hash;

// Repository implementation
pub mod repository;

// Domain adapter registry
pub mod registry;

// Adapter schemas
pub mod schemas;

// ZK adapter functionality
pub mod zk;

// CLI functionality
pub mod cli;

// RISC-V metadata
pub mod riscv_metadata;

// Code definition types
pub mod definition;

// Name registry for content-addressable code
pub mod name_registry;

// Compatibility modules
pub mod compatibility;

// Executor modules
pub mod executor;

// Re-exports for convenience
pub use hash::{Hash, HashAlgorithm, ContentHasher, Blake3ContentHasher, HasherFactory};
pub use riscv_metadata::{RiscVMetadata, RiscVCompatibilityChecker, RiscVMetadataExporter};
pub use definition::{CodeDefinition, CodeContent, CodeDefinitionBuilder};
pub use name_registry::{NameRegistry, NameRecord};
pub use compatibility::CompatibilityChecker;
pub use executor::{ContentAddressableExecutor, ExecutionContext, Value, ExecutionEvent, SecuritySandbox, EffectWrapper};

/// Effect adapter error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterError {
    /// Communication error with external Domain
    CommunicationError(String),
    /// Invalid transaction format
    InvalidTransactionFormat(String),
    /// Insufficient funds or resources
    InsufficientFunds(String),
    /// Unauthorized operation
    Unauthorized(String),
    /// External Domain is unavailable
    DomainUnavailable(String),
    /// Transaction rejected by external Domain
    TransactionRejected(String),
    /// Unsupported operation
    UnsupportedOperation(String),
    /// Other errors
    Other(String),
}

/// Proof validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofError {
    /// Invalid proof format
    InvalidFormat(String),
    /// Proof verification failed
    VerificationFailed(String),
    /// Incomplete proof
    IncompleteProof(String),
    /// Expired proof
    ExpiredProof(String),
    /// Missing data in proof
    MissingData(String),
    /// Other errors
    Other(String),
}

/// Fact observation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservationError {
    /// Unable to observe external Domain
    ObservationFailed(String),
    /// Invalid data format
    InvalidFormat(String),
    /// Missing required fields
    MissingFields(String),
    /// Time inconsistency
    TimeInconsistency(String),
    /// Unauthorized observer
    UnauthorizedObserver(String),
    /// Other errors
    Other(String),
}

/// Observed fact metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactObservationMeta {
    /// Domain ID where this fact was observed
    pub domain_id: DomainId,
    /// Fact content ID
    pub content_id: ContentId,
    /// Time point when this fact was observed
    pub observed_at: u64,
    /// Proof of the fact
    pub proof: Option<Vec<u8>>,
    /// Time map snapshot at observation time
    #[cfg(feature = "domain")]
    pub time_snapshot: Option<TimeMap>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Transaction receipt from an external Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Domain ID where this transaction was executed
    pub domain_id: DomainId,
    /// Transaction hash or ID
    pub transaction_id: String,
    /// Receipt data
    pub data: Vec<u8>,
    /// Status (success or failure)
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Block number or height
    pub block_number: Option<u64>,
    /// Time point when the transaction was included
    pub included_at: Option<u64>,
    /// Proof of transaction inclusion
    pub inclusion_proof: Option<Vec<u8>>,
    /// Time map snapshot at inclusion time
    #[cfg(feature = "domain")]
    pub time_snapshot: Option<TimeMap>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// External Domain effect parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectParams {
    /// Effect type
    pub effect_type: String,
    /// Effect parameters
    pub params: HashMap<String, Vec<u8>>,
    /// Source account (if applicable)
    pub source: Option<String>,
    /// Destination account (if applicable)
    pub destination: Option<String>,
    /// Asset identifier (if applicable)
    pub asset: Option<String>,
    /// Amount (if applicable)
    pub amount: Option<String>,
    /// Additional effect-specific data
    pub data: Option<Vec<u8>>,
    /// Signature (if required)
    pub signature: Option<Vec<u8>>,
    /// Gas limit (if applicable)
    pub gas_limit: Option<u64>,
    /// Gas price (if applicable)
    pub gas_price: Option<u64>,
    /// Nonce (if applicable)
    pub nonce: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// External Domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Domain ID
    pub domain_id: DomainId,
    /// RPC endpoints
    pub rpc_endpoints: Vec<String>,
    /// Chain ID (if applicable)
    pub chain_id: Option<String>,
    /// Network ID (if applicable)
    pub network_id: Option<String>,
    /// Default gas limit (if applicable)
    pub default_gas_limit: Option<u64>,
    /// Default gas price (if applicable)
    pub default_gas_price: Option<u64>,
    /// Block confirmation count
    pub confirmation_blocks: Option<u64>,
    /// Block time in seconds
    pub block_time_seconds: Option<u64>,
    /// Authentication credentials (if needed)
    pub auth: Option<HashMap<String, String>>,
    /// Additional configuration
    pub config: HashMap<String, String>,
}

/// Interface for effect adapters
///
/// This trait defines the interface for adapters that connect Causality
/// with external systems like blockchains, APIs, and other Domains.
/// This allows Causality programs to interact with external Domains
/// through a consistent interface.
#[cfg_attr(feature = "domain", async_trait::async_trait)]
pub trait EffectAdapter: Send + Sync + Debug {
    /// Get the Domain ID this adapter handles
    fn domain_id(&self) -> &DomainId;
    
    /// Apply an effect to an external Domain
    #[cfg(feature = "domain")]
    async fn apply_effect(&self, params: EffectParams) -> std::result::Result<TransactionReceipt, AdapterError>;
    
    /// Validate a proof from an external Domain
    #[cfg(feature = "domain")]
    async fn validate_proof(&self, effect_type: &str, proof: &[u8]) -> std::result::Result<bool, ProofError>;
    
    /// Observe a fact from an external Domain
    #[cfg(feature = "domain")]
    async fn observe_fact(&self, fact_type: &str, query_params: &HashMap<String, String>) -> std::result::Result<(FactType, FactObservationMeta), ObservationError>;
    
    /// Get the current time point from the external Domain
    #[cfg(feature = "domain")]
    async fn get_time_point(&self) -> std::result::Result<TimePoint, ObservationError>;
    
    /// Check if this adapter supports a specific effect type
    fn supports_effect(&self, effect_type: &str) -> bool;
    
    /// Check if this adapter supports a specific fact type
    fn supports_fact(&self, fact_type: &str) -> bool;
    
    /// Get the adapter configuration
    fn get_config(&self) -> &DomainConfig;
    
    /// Update the adapter configuration
    fn update_config(&mut self, config: DomainConfig) -> std::result::Result<(), Error>;
}

/// Compile an adapter schema into code
///
/// # Arguments
/// * `schema_path` - Path to the adapter schema file
/// * `output_path` - Path to write the generated code
/// * `language` - Target language (rust, typescript)
///
/// # Returns
/// Result indicating success or failure
pub fn compile_schema<P: AsRef<Path>, Q: AsRef<Path>>(
    schema_path: P,
    output_path: Q,
    language: &str,
) -> Result<()> {
    // Load the schema from disk
    let schema_content = std::fs::read_to_string(schema_path)?;
    
    #[cfg(feature = "domain")]
    {
        // Parse the schema (only available with domain feature)
        let schema = match crate::domain_adapters::schemas::AdapterSchema::from_toml(&schema_content) {
            Ok(schema) => schema,
            Err(e) => return Err(Error::InvalidInput(
                format!("Failed to parse schema: {}", e)
            )),
        };
        
        // Generate code based on the target language
        match language.to_lowercase().as_str() {
            "rust" => codegen::rust::generate_rust_adapter(&schema, output_path)?,
            "typescript" | "ts" => codegen::javascript::generate_typescript_adapter(&schema, output_path)?,
            _ => return Err(Error::InvalidInput(
                format!("Unsupported language: {}", language)
            )),
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "domain"))]
    {
        Err(Error::FeatureNotEnabled(
            "Domain feature must be enabled to compile schemas".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_params_creation() {
        let params = EffectParams {
            effect_type: "transfer".to_string(),
            params: HashMap::new(),
            source: Some("alice".to_string()),
            destination: Some("bob".to_string()),
            asset: Some("token".to_string()),
            amount: Some("10.5".to_string()),
            data: None,
            signature: None,
            gas_limit: Some(100000),
            gas_price: Some(20),
            nonce: Some(5),
            metadata: HashMap::new(),
        };
        
        assert_eq!(params.effect_type, "transfer");
        assert_eq!(params.source, Some("alice".to_string()));
        assert_eq!(params.amount, Some("10.5".to_string()));
    }
    
    #[test]
    fn test_domain_config_creation() {
        let config = DomainConfig {
            domain_id: DomainId::new("eth"),
            rpc_endpoints: vec!["https://mainnet.infura.io".to_string()],
            chain_id: Some("1".to_string()),
            network_id: Some("mainnet".to_string()),
            default_gas_limit: Some(21000),
            default_gas_price: Some(20),
            confirmation_blocks: Some(12),
            block_time_seconds: Some(15),
            auth: None,
            config: HashMap::new(),
        };
        
        assert_eq!(config.chain_id, Some("1".to_string()));
        assert_eq!(config.confirmation_blocks, Some(12));
    }
} 