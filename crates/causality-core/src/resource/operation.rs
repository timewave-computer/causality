// Resource Operations
//
// This module defines operations that can be performed on resources.

use std::marker::PhantomData;
use std::fmt::Debug;
use causality_types::ContentHash;

// Re-export the operation type from the effect module
pub use crate::effect::resource::ResourceOperation;

/// A capability is a token that grants a specific permission or ability.
/// It can be associated with various resource types.
#[derive(Debug)]
pub struct Capability<T: ?Sized> {
    /// Unique identifier for this capability
    pub id: String,
    
    /// Capability grants (rights)
    pub grants: Vec<String>,
    
    /// Origin of this capability
    pub origin: Option<String>,
    
    /// Content hash for integrity verification
    pub content_hash: Option<String>,
    
    /// Type erasure: we can store the resource itself or a reference to it
    /// This is useful for capabilities that are created for resources that are not
    /// yet stored in the system
    pub phantom: PhantomData<T>,
}

impl<T: ?Sized> Capability<T> {
    /// Create a new capability with the given ID
    pub fn new(id: String, grants: Vec<String>, origin: Option<String>, content_hash: Option<String>) -> Self {
        Self {
            id,
            grants,
            origin,
            content_hash,
            phantom: PhantomData,
        }
    }
    
    /// Get the ID of this capability
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Add a grant to this capability
    pub fn with_grant(mut self, grant: &str) -> Self {
        self.grants.push(grant.to_string());
        self
    }
    
    /// Set the origin of this capability
    pub fn with_origin(mut self, origin: &str) -> Self {
        self.origin = Some(origin.to_string());
        self
    }
    
    /// Set the content hash of this capability
    pub fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = Some(hash.to_string());
        self
    }
}

impl<T: ?Sized> Clone for Capability<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            grants: self.grants.clone(),
            origin: self.origin.clone(),
            content_hash: self.content_hash.clone(),
            phantom: PhantomData,
        }
    }
} 