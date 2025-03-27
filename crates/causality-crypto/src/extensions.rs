//! Extension traits for cryptographic types
//!
//! This module provides extension traits for cryptographic types,
//! such as ContentId and DomainId.

use crate::utils::simple_hash;
use causality_types::{ContentId, domain, block, timestamp, trace};

/// Trait for extended type operations
pub trait TypeExtensions {
    /// Get a string representation of the type
    fn to_string_representation(&self) -> String;
    
    /// Get a stable hash of the type
    fn stable_hash(&self) -> u64;
}

// Implement TypeExtensions for the ContentId struct
impl TypeExtensions for ContentId {
    fn to_string_representation(&self) -> String {
        self.to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Use the simple_hash function from utils module
        simple_hash(self.hash().as_bytes())
    }
}

// Use the DomainId struct from the domain module
// Avoid having two implementations for the same underlying type
impl TypeExtensions for domain::DomainId {
    fn to_string_representation(&self) -> String {
        self.as_str().to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Use the simple_hash function from utils module
        simple_hash(self.as_str())
    }
}

// Implement TypeExtensions for BlockHash struct from block module
impl TypeExtensions for block::BlockHash {
    fn to_string_representation(&self) -> String {
        self.as_str().to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Use the simple_hash function from utils module
        simple_hash(self.as_str())
    }
}

// Implement TypeExtensions for BlockHeight struct from block module
impl TypeExtensions for block::BlockHeight {
    fn to_string_representation(&self) -> String {
        self.value().to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Use the simple_hash function from utils module by converting to string first
        simple_hash(&self.value().to_string())
    }
}

// Implement TypeExtensions for Timestamp struct from timestamp module
impl TypeExtensions for timestamp::Timestamp {
    fn to_string_representation(&self) -> String {
        self.value().to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Use the simple_hash function from utils module by converting to string first
        simple_hash(&self.value().to_string())
    }
}

// Implement TypeExtensions for TraceId struct from trace module
impl TypeExtensions for trace::TraceId {
    fn to_string_representation(&self) -> String {
        self.as_str().to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Use the simple_hash function from utils module
        simple_hash(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_id_extensions() {
        let content_id = ContentId::from_bytes("test".as_bytes());
        
        // Test to_string_representation
        assert!(content_id.to_string_representation().contains("cid:"));
        
        // Test stable_hash provides consistent results
        let hash1 = content_id.stable_hash();
        let hash2 = content_id.stable_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_domain_id_extensions() {
        let domain_id = domain::DomainId::from("test-domain");
        
        // Test to_string_representation
        assert_eq!(domain_id.to_string_representation(), "test-domain");
        
        // Test stable_hash provides consistent results
        let hash1 = domain_id.stable_hash();
        let hash2 = domain_id.stable_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_block_hash_extensions() {
        let block_hash = block::BlockHash::new("test-block-hash");
        
        // Test to_string_representation
        assert_eq!(block_hash.to_string_representation(), "test-block-hash");
        
        // Test stable_hash provides consistent results
        let hash1 = block_hash.stable_hash();
        let hash2 = block_hash.stable_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_block_height_extensions() {
        let block_height = block::BlockHeight::new(42);
        
        // Test to_string_representation
        assert_eq!(block_height.to_string_representation(), "42");
        
        // Test stable_hash provides consistent results
        let hash1 = block_height.stable_hash();
        let hash2 = block_height.stable_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_timestamp_extensions() {
        let timestamp = timestamp::Timestamp::new(1234567890);
        
        // Test to_string_representation
        assert_eq!(timestamp.to_string_representation(), "1234567890");
        
        // Test stable_hash provides consistent results
        let hash1 = timestamp.stable_hash();
        let hash2 = timestamp.stable_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_trace_id_extensions() {
        let trace_id = trace::TraceId::from_str("trace:abc123");
        
        // Test to_string_representation
        assert_eq!(trace_id.to_string_representation(), "trace:abc123");
        
        // Test stable_hash provides consistent results
        let hash1 = trace_id.stable_hash();
        let hash2 = trace_id.stable_hash();
        assert_eq!(hash1, hash2);
    }
} 