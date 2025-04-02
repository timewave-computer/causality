// Identity module
//
// This module provides identity management functionality for the Causality system.

use std::fmt;
use causality_types::ContentId;
use causality_crypto::hash::random_hash;

/// Identity identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentityId(ContentId);

impl IdentityId {
    /// Create a new random identity ID
    pub fn new() -> Self {
        // Generate content-addressed ID using cryptographically secure random hash
        let random_hash = random_hash();
        Self(ContentId::from(random_hash))
    }
    
    /// Create an identity ID from an existing content ID
    pub fn from_content_id(content_id: ContentId) -> Self {
        Self(content_id)
    }
    
    /// Get the underlying content ID
    pub fn as_content_id(&self) -> &ContentId {
        &self.0
    }
}

impl fmt::Display for IdentityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 