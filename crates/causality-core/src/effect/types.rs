//! Effect system type definitions
//!
//! This module contains core type definitions for the effect system

use std::fmt::{self, Display, Formatter};
use serde::{Serialize, Deserialize};
use causality_types::crypto_primitives::ContentId;
use std::str::FromStr;

/// An effect identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectId(pub String);

impl EffectId {
    /// Create a new content-addressed effect ID
    pub fn new() -> Self {
        // Generate random bytes and create ContentId
        let random_bytes = rand::random::<[u8; 16]>();
        let content_id = ContentId::from_bytes(&random_bytes);
        Self(format!("effect:{}", content_id))
    }
    
    /// Create an effect ID from a string
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the underlying string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for EffectId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EffectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EffectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// An effect type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectTypeId(pub String);

impl EffectTypeId {
    /// Create a new effect type ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the underlying string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for EffectTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EffectTypeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EffectTypeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A right that can be granted to a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Right {
    /// Right to read a resource
    Read,
    /// Right to write to a resource
    Write,
    /// Right to create a resource
    Create,
    /// Right to delete a resource
    Delete,
    /// Right to delegate access to a resource
    Delegate,
    /// Custom right
    Custom(String),
}

impl Display for Right {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Right::Read => write!(f, "read"),
            Right::Write => write!(f, "write"),
            Right::Create => write!(f, "create"),
            Right::Delete => write!(f, "delete"),
            Right::Delegate => write!(f, "delegate"),
            Right::Custom(c) => write!(f, "custom:{}", c),
        }
    }
}

impl FromStr for Right {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Right::Read),
            "write" => Ok(Right::Write),
            "create" => Ok(Right::Create),
            "delete" => Ok(Right::Delete),
            "delegate" => Ok(Right::Delegate),
            s if s.starts_with("custom:") => {
                let custom = s.strip_prefix("custom:").unwrap().to_string();
                Ok(Right::Custom(custom))
            },
            _ => Err(format!("Invalid right: {}", s)),
        }
    }
}

/// A boundary for effect execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionBoundary {
    /// No boundary - effect can access any resource
    None,
    /// Local boundary - effect can only access local resources
    Local,
    /// Domain boundary - effect can only access resources in its domain
    Domain(String),
    /// Any boundary - for backward compatibility
    Any,
    /// Boundary - generic boundary for backward compatibility
    Boundary,
    /// Custom boundary
    Custom(String),
}

impl Default for ExecutionBoundary {
    fn default() -> Self {
        Self::None
    }
}

impl Display for ExecutionBoundary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionBoundary::None => write!(f, "none"),
            ExecutionBoundary::Local => write!(f, "local"),
            ExecutionBoundary::Domain(d) => write!(f, "domain:{}", d),
            ExecutionBoundary::Any => write!(f, "any"),
            ExecutionBoundary::Boundary => write!(f, "boundary"),
            ExecutionBoundary::Custom(c) => write!(f, "custom:{}", c),
        }
    }
} 