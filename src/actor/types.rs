// Actor Types Module
//
// This module provides concrete implementations of actor-related types
// for the Causality system.

use std::fmt;
use std::hash::Hash;
use std::any::Any;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::hash::{Hasher, BuildHasher};
use std::collections::hash_map::DefaultHasher;

/// Concrete implementation of an actor ID using a string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericActorId(pub String);

impl GenericActorId {
    /// Create a new random actor ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create an actor ID from a string
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Hash the ID
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }

    /// Get this as Any for downcasting
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Display for GenericActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement From<String> for GenericActorId
impl From<String> for GenericActorId {
    fn from(s: String) -> Self {
        GenericActorId(s)
    }
}

// Implement From<&str> for GenericActorId
impl From<&str> for GenericActorId {
    fn from(s: &str) -> Self {
        GenericActorId(s.to_string())
    }
}

/// Implementation for a UUID-based actor ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UuidActorId(pub Uuid);

impl UuidActorId {
    /// Create a new random actor ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create an actor ID from a UUID
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    
    /// Get the underlying UUID
    pub fn uuid(&self) -> Uuid {
        self.0
    }

    /// Hash the ID
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }

    /// Get this as Any for downcasting
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Display for UuidActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement From<Uuid> for UuidActorId
impl From<Uuid> for UuidActorId {
    fn from(id: Uuid) -> Self {
        UuidActorId(id)
    }
}

/// Enum wrapper for ActorId types to use in place of trait objects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorIdBox {
    /// Generic string-based ID
    Generic(GenericActorId),
    /// UUID-based ID
    Uuid(UuidActorId),
}

impl ActorIdBox {
    /// Create a new random actor ID using GenericActorId
    pub fn new() -> Self {
        Self::Generic(GenericActorId::new())
    }
    
    /// Create from a GenericActorId
    pub fn from_generic(id: GenericActorId) -> Self {
        Self::Generic(id)
    }
    
    /// Create from a UuidActorId
    pub fn from_uuid(id: UuidActorId) -> Self {
        Self::Uuid(id)
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> String {
        match self {
            Self::Generic(id) => id.as_str().to_string(),
            Self::Uuid(id) => id.to_string(),
        }
    }

    /// Hash the ID
    pub fn hash(&self) -> u64 {
        match self {
            Self::Generic(id) => id.hash(),
            Self::Uuid(id) => id.hash(),
        }
    }

    /// Get this as Any for downcasting
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Display for ActorIdBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(id) => write!(f, "{}", id),
            Self::Uuid(id) => write!(f, "{}", id),
        }
    }
}

// From implementations
impl From<GenericActorId> for ActorIdBox {
    fn from(id: GenericActorId) -> Self {
        Self::Generic(id)
    }
}

impl From<UuidActorId> for ActorIdBox {
    fn from(id: UuidActorId) -> Self {
        Self::Uuid(id)
    }
}

impl From<String> for ActorIdBox {
    fn from(s: String) -> Self {
        Self::Generic(GenericActorId(s))
    }
}

impl From<&str> for ActorIdBox {
    fn from(s: &str) -> Self {
        Self::Generic(GenericActorId(s.to_string()))
    }
}

impl From<Uuid> for ActorIdBox {
    fn from(id: Uuid) -> Self {
        Self::Uuid(UuidActorId(id))
    }
} 