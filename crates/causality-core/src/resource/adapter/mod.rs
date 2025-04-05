//! Resource adapter module
//!
//! Provides adapters between resource types and content addressing.

use causality_types::ContentId;
use super::types::ResourceId;
use crate::utils::content_addressing;

/// Create a ContentId from a resource identifier
pub fn resource_id_to_content_id(id: &crate::resource::types::ResourceId) -> ContentId {
    // Assume the ResourceId contains a ContentHash that can be converted
    ContentId::from_core_content_hash(&id.hash)
        .expect("Valid content hash for resource ID conversion")
}

/// Convert a ContentId to a ResourceId
pub fn content_id_to_resource_id(content_id: &ContentId) -> ResourceId {
    // Use the content_id_to_hash function instead of trying to use From<ContentId>
    let hash = content_addressing::content_id_to_hash(content_id);
    ResourceId::new(hash)
}

/// Adapter between capability and resource identifiers
pub mod id_adapter {
    
    
    /// Convert from capability::ResourceId to resource::types::ResourceId
    pub fn capability_to_resource(id: &crate::capability::ResourceId) -> crate::resource::types::ResourceId {
        crate::resource::types::ResourceId {
            hash: id.hash.clone(),
            name: id.name.clone(),
        }
    }

    /// Convert from resource::types::ResourceId to capability::ResourceId
    pub fn resource_to_capability(id: &crate::resource::types::ResourceId) -> crate::capability::ResourceId {
        crate::capability::ResourceId {
            hash: id.hash.clone(),
            name: id.name.clone(),
        }
    }
}

/// Convert from domain-specific ResourceId to ContentId
pub fn from_resource_id(id: &ResourceId) -> ContentId {
    ContentId::from_core_content_hash(&id.hash)
        .expect("Valid content hash for resource ID")
}
