//! Unified Resource type definition for the Causality framework.
//!
//! This module contains the single, canonical Resource type that replaces
//! all previous scattered resource definitions.

use crate::{
    primitive::{
        ids::{EntityId, DomainId, AsId, NodeId},
        time::Timestamp,
        string::Str,
        trait_::{AsIdentifiable, HasDomainId, HasTimestamp, AsResource}, 
    },
    graph::r#trait::AsNode,
    system::serialization::{Encode, Decode, DecodeError, SimpleSerialize}, 
};
use super::flow::ResourcePattern;
use std::default::Default;

/// Unified Resource type representing a quantifiable asset or capability
#[derive(Debug, Clone)]
pub struct Resource {
    /// Unique identifier for this resource (using EntityId for unified identification)
    pub id: EntityId,
    
    /// Human-readable name or description
    pub name: Str,
    
    /// Domain this resource belongs to
    pub domain_id: DomainId,
    
    /// Resource type identifier (e.g., "token", "compute_credits", "bandwidth")
    pub resource_type: Str,
    
    /// Current quantity/amount of this resource
    pub quantity: u64,
    
    /// When this resource was created or last updated
    pub timestamp: Timestamp,
}



impl Resource {
    /// Create a new Resource instance
    pub fn new(
        id: EntityId,
        name: Str,
        domain_id: DomainId,
        resource_type: Str,
        quantity: u64,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            id,
            name,
            domain_id,
            resource_type,
            quantity,
            timestamp,
        }
    }
}

impl Default for Resource {
    fn default() -> Self {
        let default_domain = DomainId::new([0u8; 32]);
        Self {
            id: EntityId::new([0u8; 32]),
            name: Str::from("default_resource"),
            domain_id: default_domain,
            resource_type: Str::from("default"),
            quantity: 0,
            timestamp: Timestamp::now(),
        }
    }
}

impl AsNode for Resource {
    fn to_node_id(&self) -> NodeId {
        // Convert ResourceId to NodeId - they use the same internal format
        NodeId::new(self.id.inner())
    }

    fn from_node_id(id: NodeId) -> Option<Self> {
        Some(Self {
            id: EntityId::new(id.inner()),
            ..Default::default()
        })
    }
}



//-----------------------------------------------------------------------------
// Trait Implementations
//-----------------------------------------------------------------------------

impl AsIdentifiable for Resource {
    fn id(&self) -> &EntityId {
        &self.id
    }
    
    fn name(&self) -> &Str {
        &self.name
    }
}

impl HasDomainId for Resource {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl HasTimestamp for Resource {
    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl AsResource for Resource {
    fn resource_type(&self) -> &Str {
        &self.resource_type
    }
    
    fn quantity(&self) -> u64 {
        self.quantity
    }
    
    fn matches_pattern(&self, pattern: &ResourcePattern) -> bool {
        if self.resource_type != pattern.resource_type {
            return false;
        }
        
        if let Some(domain_id) = &pattern.domain_id {
            if self.domain_id != *domain_id {
                return false;
            }
        }
        
        // For now, ignore constraints - would need more complex matching logic
        true
    }
}

// Need to implement PartialEq manually since we want to be explicit about equality
impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.name == other.name &&
        self.domain_id == other.domain_id &&
        self.resource_type == other.resource_type &&
        self.quantity == other.quantity &&
        self.timestamp == other.timestamp
    }
}

impl Eq for Resource {}



//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for Resource {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Encode each field in order
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.resource_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.quantity.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for Resource {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode each field in order
        let id = EntityId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_id.as_ssz_bytes().len();
        
        let resource_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += resource_type.as_ssz_bytes().len();
        
        if offset + 8 > bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for quantity".to_string() });
        }
        let quantity = u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]
        ]);
        offset += 8;
        
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Resource {
            id,
            name,
            domain_id,
            resource_type,
            quantity,
            timestamp,
        })
    }
}

impl SimpleSerialize for Resource {}

 


