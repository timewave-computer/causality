//! Causal proof chain for tracking resource transformations
//!
//! This module provides basic causal proof structures to track the provenance
//! and transformation history of resources.

use crate::system::content_addressing::{ResourceId, Timestamp};
use ssz::{Encode, Decode};

/// Basic causal proof structure for tracking resource transformation history
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalProof {
    /// The previous resource ID in the transformation chain
    pub previous_id: Option<ResourceId>,
    
    /// The operation that created this resource
    pub operation: String,
    
    /// When this transformation occurred
    pub timestamp: Timestamp,
}

impl CausalProof {
    /// Create a new causal proof for resource creation
    pub fn genesis(operation: impl Into<String>) -> Self {
        Self {
            previous_id: None,
            operation: operation.into(),
            timestamp: Timestamp::now(),
        }
    }
    
    /// Create a new causal proof for resource transformation
    pub fn transform(previous_id: ResourceId, operation: impl Into<String>) -> Self {
        Self {
            previous_id: Some(previous_id),
            operation: operation.into(),
            timestamp: Timestamp::now(),
        }
    }
    
    /// Check if this is a genesis proof (no previous resource)
    pub fn is_genesis(&self) -> bool {
        self.previous_id.is_none()
    }
    
    /// Get the previous resource ID if this is a transformation
    pub fn previous(&self) -> Option<ResourceId> {
        self.previous_id
    }
}

impl Encode for CausalProof {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        // Option<ResourceId>: 1 byte discriminant + optional 32 bytes
        let option_len = match self.previous_id {
            None => 1,
            Some(_) => 1 + 32,
        };
        option_len +
        4 + self.operation.len() + // length prefix for string + string content
        self.timestamp.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        // Encode Option<ResourceId> manually
        match &self.previous_id {
            None => buf.push(0), // discriminant for None
            Some(id) => {
                buf.push(1); // discriminant for Some
                id.ssz_append(buf);
            }
        }
        
        // Encode operation string
        (self.operation.len() as u32).to_le_bytes().iter().for_each(|b| buf.push(*b));
        buf.extend_from_slice(self.operation.as_bytes());
        
        // Encode timestamp
        self.timestamp.ssz_append(buf);
    }
}

impl Decode for CausalProof {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let mut offset = 0;
        
        // Decode previous_id (Option<ResourceId> - variable length)
        // Read discriminant byte first
        if bytes.len() < offset + 1 {
            return Err(ssz::DecodeError::InvalidByteLength { len: bytes.len(), expected: offset + 1 });
        }
        
        let previous_id = if bytes[offset] == 0 {
            // None case - just discriminant byte
            offset += 1;
            None
        } else if bytes[offset] == 1 {
            // Some case - discriminant + 32 bytes
            offset += 1;
            if bytes.len() < offset + 32 {
                return Err(ssz::DecodeError::InvalidByteLength { len: bytes.len(), expected: offset + 32 });
            }
            let id = ResourceId::from_ssz_bytes(&bytes[offset..offset + 32])?;
            offset += 32;
            Some(id)
        } else {
            return Err(ssz::DecodeError::BytesInvalid("Invalid Option discriminant".to_string()));
        };
        
        // Decode operation string
        if bytes.len() < offset + 4 {
            return Err(ssz::DecodeError::InvalidByteLength { len: bytes.len(), expected: offset + 4 });
        }
        let operation_len = u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]) as usize;
        offset += 4;
        
        if bytes.len() < offset + operation_len {
            return Err(ssz::DecodeError::InvalidByteLength { len: bytes.len(), expected: offset + operation_len });
        }
        let operation = String::from_utf8(bytes[offset..offset + operation_len].to_vec())
            .map_err(|_| ssz::DecodeError::BytesInvalid("Invalid UTF-8 in operation string".to_string()))?;
        offset += operation_len;
        
        // Decode timestamp
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Self {
            previous_id,
            operation,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::content_addressing::EntityId;

    #[test]
    fn test_genesis_proof() {
        let proof = CausalProof::genesis("create_token");
        assert!(proof.is_genesis());
        assert_eq!(proof.operation, "create_token");
        assert!(proof.previous().is_none());
    }

    #[test]
    fn test_transform_proof() {
        let previous_id = EntityId::ZERO;
        let proof = CausalProof::transform(previous_id, "transfer");
        assert!(!proof.is_genesis());
        assert_eq!(proof.operation, "transfer");
        assert_eq!(proof.previous(), Some(previous_id));
    }

    #[test]
    fn test_ssz_serialization() {
        let proof = CausalProof::genesis("test_operation");
        let encoded = proof.as_ssz_bytes();
        let decoded = CausalProof::from_ssz_bytes(&encoded).unwrap();
        assert_eq!(proof, decoded);
    }
} 