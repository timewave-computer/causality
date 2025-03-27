// Sparse Merkle Tree implementation
//
// This module provides the core sparse merkle tree (SMT) functionality,
// including node representation, proof generation and verification.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::crypto::ContentId;
use crate::crypto::ContentAddressed;

/// SMT node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtNode {
    /// Node ID
    pub id: ContentId,
    
    /// Node value
    pub value: Option<SmtValue>,
    
    /// Node children
    pub children: Vec<Option<ContentId>>,
    
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

/// SMT value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtValue {
    /// Value ID
    pub id: ContentId,
    
    /// Value data
    pub data: Vec<u8>,
    
    /// Value metadata
    pub metadata: HashMap<String, String>,
}

/// SMT proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtProof {
    /// Root hash
    pub root_hash: ContentId,
    
    /// Nodes in proof
    pub nodes: Vec<SmtNode>,
    
    /// Proof metadata
    pub metadata: HashMap<String, String>,
}

/// SMT error
#[derive(Debug, thiserror::Error)]
pub enum SmtError {
    /// SMT proof error
    #[error("SMT proof error: {0}")]
    ProofError(String),
    
    /// SMT node error
    #[error("SMT node error: {0}")]
    NodeError(String),
    
    /// SMT value error
    #[error("SMT value error: {0}")]
    ValueError(String),
}

/// SMT result
pub type SmtResult<T> = Result<T, SmtError>;

/// SMT verifier
#[async_trait]
pub trait SmtVerifier: Send + Sync + Debug {
    /// Verify SMT proof
    async fn verify_proof(&self, proof: &SmtProof) -> SmtResult<bool>;
    
    /// Verify value in SMT
    async fn verify_value(
        &self,
        proof: &SmtProof,
        key: &[u8],
        value: &SmtValue,
    ) -> SmtResult<bool>;
}

/// SMT builder
#[async_trait]
pub trait SmtBuilder: Send + Sync + Debug {
    /// Build SMT from key-value pairs
    async fn build_smt(
        &self,
        key_values: Vec<(Vec<u8>, SmtValue)>,
    ) -> SmtResult<SmtNode>;
    
    /// Get proof for key
    async fn get_proof(
        &self,
        root: &SmtNode,
        key: &[u8],
    ) -> SmtResult<SmtProof>;
}

/// SMT storage
#[async_trait]
pub trait SmtStorage: Send + Sync + Debug {
    /// Get node by ID
    async fn get_node(&self, id: &ContentId) -> SmtResult<Option<SmtNode>>;
    
    /// Store node
    async fn store_node(&self, node: &SmtNode) -> SmtResult<()>;
    
    /// Get value by ID
    async fn get_value(&self, id: &ContentId) -> SmtResult<Option<SmtValue>>;
    
    /// Store value
    async fn store_value(&self, value: &SmtValue) -> SmtResult<()>;
}

/// SMT manager
#[async_trait]
pub trait SmtManager: Send + Sync + Debug {
    /// Get builder
    fn get_builder(&self) -> Arc<dyn SmtBuilder>;
    
    /// Get verifier
    fn get_verifier(&self) -> Arc<dyn SmtVerifier>;
    
    /// Get storage
    fn get_storage(&self) -> Arc<dyn SmtStorage>;
}

/// SMT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtConfig {
    /// SMT depth
    pub depth: usize,
    
    /// SMT arity
    pub arity: usize,
    
    /// SMT metadata
    pub metadata: HashMap<String, String>,
}

impl Default for SmtConfig {
    fn default() -> Self {
        Self {
            depth: 32,
            arity: 2,
            metadata: HashMap::new(),
        }
    }
}

/// SMT factory
#[async_trait]
pub trait SmtFactory: Send + Sync + Debug {
    /// Create SMT manager
    async fn create_manager(&self, config: SmtConfig) -> SmtResult<Arc<dyn SmtManager>>;
    
    /// Get supported configurations
    fn supported_configs(&self) -> Vec<SmtConfig>;
} 