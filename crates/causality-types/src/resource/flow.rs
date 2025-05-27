//! Resource flow types for tracking resource movement

use crate::primitive::ids::DomainId;
use crate::primitive::string::Str;
use crate::serialization::{Encode, Decode, DecodeError, SimpleSerialize};
use std::collections::BTreeMap;

/// ResourceFlow represents the flow of resources between components
#[derive(Debug, Clone)]
pub struct ResourceFlow {
    pub resource_type: Str,
    pub quantity: u64,
    pub domain_id: DomainId,
}

/// ResourcePattern represents a pattern for matching resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePattern {
    pub resource_type: Str,
    pub domain_id: Option<DomainId>,
    pub constraints: BTreeMap<Str, Str>,
}

impl ResourceFlow {
    pub fn new(resource_type: Str, quantity: u64, domain_id: DomainId) -> Self {
        Self {
            resource_type,
            quantity,
            domain_id,
        }
    }
}

impl ResourcePattern {
    pub fn new(resource_type: Str) -> Self {
        Self {
            resource_type,
            domain_id: None,
            constraints: BTreeMap::new(),
        }
    }
    
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    pub fn with_constraint(mut self, key: Str, value: Str) -> Self {
        self.constraints.insert(key, value);
        self
    }
}

impl PartialEq for ResourceFlow {
    fn eq(&self, other: &Self) -> bool {
        self.resource_type == other.resource_type
            && self.quantity == other.quantity
            && self.domain_id == other.domain_id
    }
}

impl Eq for ResourceFlow {}

impl Encode for ResourceFlow {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.resource_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.quantity.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes
    }
}

impl Decode for ResourceFlow {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 40 {
            return Err(DecodeError {
                message: format!("ResourceFlow requires at least 40 bytes, got {}", bytes.len()),
            });
        }
        
        // This is a simplified implementation - proper SSZ would handle variable-length strings
        let resource_type = Str::from_ssz_bytes(&bytes[0..8])?;
        let quantity = u64::from_ssz_bytes(&bytes[8..16])?;
        let domain_id = DomainId::from_ssz_bytes(&bytes[16..48])?;
        
        Ok(Self {
            resource_type,
            quantity,
            domain_id,
        })
    }
}

impl SimpleSerialize for ResourceFlow {}
impl SimpleSerialize for ResourcePattern {} 