//! Core API types for blockchain interaction and transaction submission
//!
//! This module defines the types used for interacting with multiple blockchain
//! networks, including transaction requests, chain configurations, and proof data.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Transaction Types
//-----------------------------------------------------------------------------

/// Request to submit a transaction to a blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// ZK proof data to be verified on-chain
    pub proof_data: ProofData,
    
    /// Gas price in wei (optional, will use network default if not specified)
    pub gas_price: Option<u64>,
    
    /// Maximum gas limit for the transaction
    pub gas_limit: Option<u64>,
    
    /// Whether this is a dry run (validation only)
    pub dry_run: bool,
}

/// Response from transaction submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Transaction hash (if submitted)
    pub tx_hash: Option<String>,
    
    /// Block number where transaction was included
    pub block_number: Option<u64>,
    
    /// Gas used by the transaction
    pub gas_used: u64,
    
    /// Transaction status
    pub status: TransactionStatus,
    
    /// Error message if transaction failed
    pub error: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction was successfully submitted and confirmed
    Success,
    
    /// Transaction failed during execution
    Failed,
    
    /// Transaction is pending confirmation
    Pending,
    
    /// Dry run validation passed
    ValidatedSuccess,
    
    /// Dry run validation failed
    ValidatedFailure,
}

//-----------------------------------------------------------------------------
// Proof Data Types
//-----------------------------------------------------------------------------

/// Zero-knowledge proof data for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    /// The proof itself (serialized)
    pub proof: String,
    
    /// Public inputs to the proof
    pub public_inputs: Vec<String>,
    
    /// Verification key identifier
    pub verification_key: String,
    
    /// Circuit identifier
    pub circuit_id: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

//-----------------------------------------------------------------------------
// Chain Configuration Types
//-----------------------------------------------------------------------------

/// Configuration for a specific blockchain network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Human-readable chain name
    pub name: String,
    
    /// Chain ID for the network
    pub chain_id: u64,
    
    /// RPC endpoint URL
    pub rpc_url: String,
    
    /// Block explorer base URL
    pub explorer_url: String,
    
    /// Gas price multiplier for fee estimation
    pub gas_price_multiplier: f64,
    
    /// Number of blocks to wait for confirmation
    pub confirmation_blocks: u64,
}

/// Multi-chain deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiChainConfig {
    /// Configurations for each supported chain
    pub chains: HashMap<String, ChainConfig>,
    
    /// Default gas limits by operation type
    pub default_gas_limits: HashMap<String, u64>,
    
    /// Global settings
    pub global_settings: GlobalSettings,
}

/// Global settings for multi-chain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Maximum number of concurrent chain submissions
    pub max_concurrent_submissions: usize,
    
    /// Timeout for transaction confirmation (in seconds)
    pub confirmation_timeout_seconds: u64,
    
    /// Whether to continue on partial failures
    pub continue_on_failure: bool,
    
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Configuration for transaction retry logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    
    /// Initial delay between retries (in milliseconds)
    pub initial_delay_ms: u64,
    
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    
    /// Maximum delay between retries (in milliseconds)
    pub max_delay_ms: u64,
}

//-----------------------------------------------------------------------------
// Session Types for API Communication
//-----------------------------------------------------------------------------

/// Session context for API operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: String,
    
    /// User authentication token
    pub auth_token: Option<String>,
    
    /// Session metadata
    pub metadata: HashMap<String, String>,
    
    /// Session creation timestamp
    pub created_at: u64,
    
    /// Session expiration timestamp
    pub expires_at: u64,
}

/// API request wrapper with session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest<T> {
    /// Session context
    pub session: SessionContext,
    
    /// Request payload
    pub payload: T,
    
    /// Request timestamp
    pub timestamp: u64,
    
    /// Request ID for tracking
    pub request_id: String,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response payload
    pub data: Option<T>,
    
    /// Error information if request failed
    pub error: Option<ApiError>,
    
    /// Response timestamp
    pub timestamp: u64,
    
    /// Request ID that this response corresponds to
    pub request_id: String,
}

/// API error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional error details
    pub details: HashMap<String, String>,
}

//-----------------------------------------------------------------------------
// Default Implementations
//-----------------------------------------------------------------------------

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
        }
    }
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            max_concurrent_submissions: 5,
            confirmation_timeout_seconds: 300,
            continue_on_failure: false,
            retry_config: RetryConfig::default(),
        }
    }
}

//-----------------------------------------------------------------------------
// Conversion Utilities
//-----------------------------------------------------------------------------

impl TransactionResponse {
    /// Convert to a simple result tuple for CLI usage
    pub fn to_result_tuple(&self) -> (bool, String, u64, u64) {
        match self.status {
            TransactionStatus::Success | TransactionStatus::ValidatedSuccess => {
                (true, self.tx_hash.clone().unwrap_or_default(), self.gas_used, self.block_number.unwrap_or_default())
            }
            _ => {
                (false, self.error.clone().unwrap_or("Unknown error".to_string()), self.gas_used, 0)
            }
        }
    }
}
