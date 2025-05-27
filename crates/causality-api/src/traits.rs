//! Core Traits for Blockchain Interactions
//!
//! This module defines the core traits that establish the contract between
//! the Causality framework and blockchain implementations. These traits provide
//! a unified interface for operations like transactions, queries, and intent management.
//!
//! ## Trait Organization
//!
//! * **Configuration Traits**: Define constants for chain implementation (`ChainConfig`)
//! * **Intent Traits**: Intent submission and querying (`IntentSubmission`, `IntentQuery`)
//! * **Transaction Traits**: Transaction submission and querying (`Transaction`, `Query`)
//! * **Client Traits**: Connection management and client behavior (`ChainClient`, `ClientBuilder`)

//-----------------------------------------------------------------------------
// Import
//-----------------------------------------------------------------------------

use anyhow::Result;
use async_trait::async_trait;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::fmt::Debug;

use crate::models::{
    IntentQueryInput, IntentQueryOutput, IntentSubmissionInput,
    IntentSubmissionOutput,
};

//-----------------------------------------------------------------------------
// Chain Configuration Trait
//-----------------------------------------------------------------------------

/// Configuration constants for a blockchain implementation
pub trait ChainConfig {
    /// Human-readable chain name
    const CHAIN_NAME: &'static str;

    /// Chain identifier
    const CHAIN_ID: &'static str;

    /// Default RPC port for this chain
    const DEFAULT_RPC_PORT: &'static str;

    /// Chain type category (e.g., "cosmos", "evm")
    const CHAIN_TYPE: &'static str;
}

//-----------------------------------------------------------------------------
// Intent Query Trait
//-----------------------------------------------------------------------------

/// Trait for querying intents from a blockchain
#[async_trait]
pub trait IntentQuery: Send + Sync {
    /// Query an intent by its ID
    async fn query_intent(
        &self,
        input: IntentQueryInput,
    ) -> Result<IntentQueryOutput>;
}

//-----------------------------------------------------------------------------
// Intent Submission Trait
//-----------------------------------------------------------------------------

/// Trait for submitting intents to a blockchain
#[async_trait]
pub trait IntentSubmission: Send + Sync {
    /// Submit an intent to the blockchain
    async fn submit_intent(
        &self,
        input: IntentSubmissionInput,
    ) -> Result<IntentSubmissionOutput>;
}

//-----------------------------------------------------------------------------
// Transaction Trait
//-----------------------------------------------------------------------------

/// Trait for submitting transactions to a blockchain or a mock client.
#[async_trait]
pub trait Transaction: Send + Sync {
    /// Input type for the transaction
    type Input: Send + Sync;
    /// Output type for the transaction result
    type Output: Send + Sync;

    /// Submit a transaction
    async fn submit_transaction(&self, tx: Self::Input) -> Result<Self::Output>;
}

//-----------------------------------------------------------------------------
// Query Trait
//-----------------------------------------------------------------------------

/// Trait for executing queries against a blockchain or a mock client.
#[async_trait]
pub trait Query: Send + Sync {
    /// Input type for the query
    type Input: Send + Sync;
    /// Output type for the query result
    type Output: Send + Sync;

    /// Execute a query
    async fn execute_query(&self, query: Self::Input) -> Result<Self::Output>;
}

//-----------------------------------------------------------------------------
// Connection Configuration
//-----------------------------------------------------------------------------

/// Configuration for blockchain client connections.
/// Contains all necessary parameters to establish a connection.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// URL of the RPC endpoint
    pub rpc_url: String,

    /// Authentication token (if needed)
    pub auth_token: Option<String>,

    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Extra connection parameters (limited to 16 items for ZK compatibility)
    pub extra_params: Option<[(String, String); 16]>,

    /// Connection pooling settings
    pub connection_pooling: bool,
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// ClientConfig
impl Encode for ClientConfig {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.rpc_url.as_ssz_bytes());
        
        // Serialize auth_token as Option
        match &self.auth_token {
            Some(token) => {
                bytes.push(1u8);
                bytes.extend(token.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize timeout_ms as Option
        match self.timeout_ms {
            Some(timeout) => {
                bytes.push(1u8);
                bytes.extend(timeout.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize extra_params as Option
        match &self.extra_params {
            Some(params) => {
                bytes.push(1u8);
                // Serialize fixed-size array of tuples
                for (key, value) in params.iter() {
                    bytes.extend(key.as_ssz_bytes());
                    bytes.extend(value.as_ssz_bytes());
                }
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes.push(if self.connection_pooling { 1u8 } else { 0u8 });
        
        bytes
    }
}

impl Decode for ClientConfig {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode rpc_url
        let rpc_url = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode rpc_url: {}", e) })?;
        let rpc_url_size = rpc_url.as_ssz_bytes().len();
        offset += rpc_url_size;
        
        // Decode auth_token option
        let has_auth_token = bytes[offset];
        offset += 1;
        
        let auth_token = if has_auth_token == 1 {
            let token = String::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode auth_token: {}", e) })?;
            let token_size = token.as_ssz_bytes().len();
            offset += token_size;
            Some(token)
        } else {
            None
        };
        
        // Decode timeout_ms option
        let has_timeout_ms = bytes[offset];
        offset += 1;
        
        let timeout_ms = if has_timeout_ms == 1 {
            let timeout = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode timeout_ms: {}", e) })?;
            offset += 8;
            Some(timeout)
        } else {
            None
        };
        
        // Decode extra_params option
        let has_extra_params = bytes[offset];
        offset += 1;
        
        let extra_params = if has_extra_params == 1 {
            let mut params: [(String, String); 16] = std::array::from_fn(|_| ("".to_string(), "".to_string()));
            for i in 0..16 {
                let key = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode extra_params key {}: {}", i, e) })?;
                let key_size = key.as_ssz_bytes().len();
                offset += key_size;
                
                let value = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode extra_params value {}: {}", i, e) })?;
                let value_size = value.as_ssz_bytes().len();
                offset += value_size;
                
                params[i] = (key, value);
            }
            Some(params)
        } else {
            None
        };
        
        // Decode connection_pooling
        let connection_pooling = bytes[offset] == 1;
        
        Ok(ClientConfig {
            rpc_url,
            auth_token,
            timeout_ms,
            extra_params,
            connection_pooling,
        })
    }
}

impl SimpleSerialize for ClientConfig {}

//-----------------------------------------------------------------------------
// Chain Client Interface
//-----------------------------------------------------------------------------

/// Generic client trait that defines the API for interacting with a blockchain.
/// This is the core interface that all blockchain clients must implement.
#[async_trait]
pub trait ChainClient: Debug + Send + Sync {
    /// Get the client configuration
    fn config(&self) -> &ClientConfig;

    /// Check if the client is connected
    async fn is_connected(&self) -> bool;

    /// Get the chain name
    fn chain_name(&self) -> &str;

    /// Get the chain ID
    fn chain_id(&self) -> &str;

    /// Get the client address (if applicable)
    fn address(&self) -> Option<&str>;

    /// Get the current block height
    async fn get_block_height(&self) -> Result<u64>;
}

//-----------------------------------------------------------------------------
// Client Construction
//-----------------------------------------------------------------------------

/// Trait for clients that can be initialized with a URL and port.
/// Provides methods for creating properly configured client instances.
pub trait ClientBuilder: Sized {
    /// Build a new client from URL
    fn from_url(url: &str) -> Result<Self>;

    /// Build a new client with the given config
    fn from_config(config: ClientConfig) -> Result<Self>;

    /// Build a new client with the given address
    fn with_address(self, address: &str) -> Self;
}
