// Actor Types Module
//
// This module provides concrete implementations of actor-related types
// for the Causality system.

use std::fmt;
use std::hash::Hash;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::actor::ActorId;

/// Concrete implementation of the ActorId trait
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
}

impl fmt::Display for GenericActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ActorId for GenericActorId {}

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
}

impl fmt::Display for UuidActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ActorId for UuidActorId {}

// Implement From<Uuid> for UuidActorId
impl From<Uuid> for UuidActorId {
    fn from(id: Uuid) -> Self {
        UuidActorId(id)
    }
} 