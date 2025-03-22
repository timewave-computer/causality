// Core type definitions for TEL
//
// This module provides the core type definitions
// used throughout the Temporal Effect Language (TEL).

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Identifier for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

impl ResourceId {
    /// Create a new random resource ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Address of an actor in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub String);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Domain(pub String);

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Domain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Metadata as key-value pairs
pub type Metadata = HashMap<String, serde_json::Value>;

/// Identifier for an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub Uuid);

impl OperationId {
    /// Create a new random operation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof for an operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    /// Type of proof
    pub proof_type: String,
    /// Proof data
    pub data: Vec<u8>,
    /// Verification key
    pub verification_key: Option<Vec<u8>>,
}

/// Parameters for an operation
pub type Parameters = HashMap<String, serde_json::Value>;

/// Time point in milliseconds since UNIX epoch
pub type Timestamp = u64;

/// Type of effect in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    /// State transition
    StateTransition,
    /// Resource transfer
    ResourceTransfer,
    /// Computation
    Computation,
    /// Data operation
    DataOperation,
    /// Communication
    Communication,
    /// Access control
    AccessControl,
    /// Custom effect type
    Custom(String),
}

/// Effect identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectId(pub String);

impl EffectId {
    /// Create a new random effect ID
    pub fn new() -> Self {
        Self(format!("effect-{}", Uuid::new_v4()))
    }
}

impl Default for EffectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectStatus {
    /// Effect is pending
    Pending,
    /// Effect is being processed
    Processing,
    /// Effect has completed successfully
    Completed,
    /// Effect has failed
    Failed,
    /// Effect has been cancelled
    Cancelled,
}

/// Result of an effect
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectResult {
    /// No result
    None,
    /// Boolean result
    Boolean(bool),
    /// Integer result
    Integer(i64),
    /// Float result
    Float(f64),
    /// String result
    String(String),
    /// Binary result
    Binary(Vec<u8>),
    /// JSON result
    Json(serde_json::Value),
    /// Error result
    Error(String),
} 