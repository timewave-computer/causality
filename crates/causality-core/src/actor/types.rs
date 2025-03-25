// Actor-specific type definitions
// Original file: src/actor/types.rs

// Actor Types Module
//
// This module provides concrete implementations of actor-related types
// for the Causality system.

use std::fmt;
use std::hash::Hash;
use std::any::Any;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use std::hash::{Hasher, BuildHasher};
use std::collections::hash_map::DefaultHasher;
use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// Concrete implementation of an actor ID using a string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericActorId(pub String);

impl GenericActorId {
    /// Create a new random actor ID
    pub fn new() -> Self {
        // Create a simple content-derived ID
        let data = format!("actor:{}", rand::random::<u64>());
        let id = format!("actor:{}", HashFactory::default().create_hasher().unwrap().hash(data.as_bytes()));
        Self(id)
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

/// Implementation for a content-addressed actor ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedActorId {
    /// Actor data for content addressing
    data: ActorData,
    /// Content ID
    id: ContentId,
}

/// Actor data used for content addressing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
struct ActorData {
    /// Timestamp
    timestamp: i64,
    /// Random nonce
    nonce: [u8; 8],
    /// Optional name
    name: Option<String>,
}

impl ContentAddressed for ActorData {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl ContentAddressedActorId {
    /// Create a new random actor ID
    pub fn new() -> Self {
        let data = ActorData {
            timestamp: chrono::Utc::now().timestamp(),
            nonce: rand::random::<[u8; 8]>(),
            name: None,
        };
        
        let id = data.content_id();
        
        Self {
            data,
            id,
        }
    }
    
    /// Create an actor ID with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        let data = ActorData {
            timestamp: chrono::Utc::now().timestamp(),
            nonce: rand::random::<[u8; 8]>(),
            name: Some(name.into()),
        };
        
        let id = data.content_id();
        
        Self {
            data,
            id,
        }
    }
    
    /// Create an actor ID from a ContentId
    pub fn from_content_id(id: ContentId) -> Self {
        let data = ActorData {
            timestamp: chrono::Utc::now().timestamp(),
            nonce: rand::random::<[u8; 8]>(),
            name: None,
        };
        
        Self {
            data,
            id,
        }
    }
    
    /// Get the content ID
    pub fn content_id(&self) -> ContentId {
        self.id.clone()
    }

    /// Hash the ID
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        hasher.finish()
    }

    /// Get this as Any for downcasting
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Display for ContentAddressedActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "actor:{}", self.id)
    }
}

// Implement From<ContentId> for ContentAddressedActorId
impl From<ContentId> for ContentAddressedActorId {
    fn from(id: ContentId) -> Self {
        ContentAddressedActorId::from_content_id(id)
    }
}

/// Enum wrapper for ActorId types to use in place of trait objects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorIdBox {
    /// Generic string-based ID
    Generic(GenericActorId),
    /// Content-addressed ID
    ContentAddressed(ContentAddressedActorId),
}

impl ActorIdBox {
    /// Create a new random actor ID using GenericActorId
    pub fn new() -> Self {
        Self::ContentAddressed(ContentAddressedActorId::new())
    }
    
    /// Create from a GenericActorId
    pub fn from_generic(id: GenericActorId) -> Self {
        Self::Generic(id)
    }
    
    /// Create from a ContentAddressedActorId
    pub fn from_content_addressed(id: ContentAddressedActorId) -> Self {
        Self::ContentAddressed(id)
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> String {
        match self {
            Self::Generic(id) => id.as_str().to_string(),
            Self::ContentAddressed(id) => id.to_string(),
        }
    }

    /// Hash the ID
    pub fn hash(&self) -> u64 {
        match self {
            Self::Generic(id) => id.hash(),
            Self::ContentAddressed(id) => id.hash(),
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
            Self::ContentAddressed(id) => write!(f, "{}", id),
        }
    }
}

// From implementations
impl From<GenericActorId> for ActorIdBox {
    fn from(id: GenericActorId) -> Self {
        Self::Generic(id)
    }
}

impl From<ContentAddressedActorId> for ActorIdBox {
    fn from(id: ContentAddressedActorId) -> Self {
        Self::ContentAddressed(id)
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

impl From<ContentId> for ActorIdBox {
    fn from(id: ContentId) -> Self {
        Self::ContentAddressed(ContentAddressedActorId::from_content_id(id))
    }
} 