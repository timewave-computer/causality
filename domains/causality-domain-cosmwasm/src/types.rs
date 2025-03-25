// CosmWasm-specific type definitions
// Original file: src/domain_adapters/cosmwasm/types.rs

// CosmWasm Types
//
// This module defines the core type definitions for interacting with
// CosmWasm-based blockchains.

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use causality_types::{*};
use causality_crypto::ContentId;;

/// Represents a CosmWasm contract address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CosmWasmAddress(pub String);

impl fmt::Display for CosmWasmAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CosmWasmAddress {
    fn from(s: String) -> Self {
        CosmWasmAddress(s)
    }
}

impl From<&str> for CosmWasmAddress {
    fn from(s: &str) -> Self {
        CosmWasmAddress(s.to_string())
    }
}

/// Represents a CosmWasm message (execute, instantiate, migrate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmMessage {
    /// Message type (execute, instantiate, migrate)
    pub message_type: CosmWasmMessageType,
    /// Contract address (for execute, migrate)
    pub contract: Option<CosmWasmAddress>,
    /// Code ID (for instantiate)
    pub code_id: Option<String>,
    /// Label (for instantiate)
    pub label: Option<String>,
    /// Raw JSON message
    pub msg: serde_json::Value,
    /// Funds to send with the message
    pub funds: Vec<Coin>,
    /// Admin address (for instantiate)
    pub admin: Option<CosmWasmAddress>,
}

/// CosmWasm message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CosmWasmMessageType {
    /// Execute a message on an existing contract
    Execute,
    /// Instantiate a new contract from code
    Instantiate,
    /// Migrate a contract to a new code ID
    Migrate,
}

/// Represents a coin amount
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coin {
    /// Denomination
    pub denom: String,
    /// Amount (as string to preserve precision)
    pub amount: String,
}

impl Coin {
    /// Create a new coin
    pub fn new(denom: String, amount: impl ToString) -> Self {
        Self {
            denom,
            amount: amount.to_string(),
        }
    }
}

/// Result of a CosmWasm query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmQueryResult {
    /// Contract address
    pub contract: CosmWasmAddress,
    /// Query result as JSON
    pub result: serde_json::Value,
    /// Block height at which the query was executed
    pub height: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Result of a CosmWasm execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmExecutionResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Block height
    pub height: u64,
    /// Gas used
    pub gas_used: u64,
    /// Gas wanted
    pub gas_wanted: u64,
    /// Log messages
    pub logs: Vec<String>,
    /// Events
    pub events: Vec<CosmWasmEvent>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Event emitted during contract execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmEvent {
    /// Event type
    pub event_type: String,
    /// Event attributes
    pub attributes: Vec<(String, String)>,
}

/// Represents compiled CosmWasm code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmCode {
    /// Code ID on chain
    pub code_id: String,
    /// Creator address
    pub creator: CosmWasmAddress,
    /// Code hash
    pub hash: String,
    /// Size in bytes
    pub size: u64,
    /// Domain ID
    pub domain_id: DomainId,
    /// Block height when uploaded
    pub block_height: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Helper function to create a coin
pub fn coin(amount: impl ToString, denom: impl ToString) -> Coin {
    Coin {
        denom: denom.to_string(),
        amount: amount.to_string(),
    }
} 
