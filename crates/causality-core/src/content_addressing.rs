// content_addressing.rs
//
// Unified content addressing using SSZ serialization + SHA256 hashing.
// This provides the canonical method for generating content-addressed IDs
// across both Rust and OCaml systems.

use sha2::{Sha256, Digest};
use causality_types::serialization::Encode;

// Re-export the comprehensive ContentAddressable trait from causality-types
pub use causality_types::primitive::content::{
    ContentAddressable, ContentTraversable, DomainValidated, ContentAddressableCollection
};

/// Generate content-addressed ID from SSZ bytes
pub fn content_id_from_bytes(ssz_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ssz_bytes);
    hasher.finalize().into()
}

/// Generate hex-encoded content-addressed ID from SSZ bytes
pub fn content_id_hex_from_bytes(ssz_bytes: &[u8]) -> String {
    hex::encode(content_id_from_bytes(ssz_bytes))
}

/// Convenience trait for simple content addressing without domain awareness
/// This provides the simpler interface that was previously in this module
pub trait SimpleContentAddressable: Encode {
    /// Get the content-addressed ID using SSZ + SHA256
    fn content_id(&self) -> [u8; 32] {
        let ssz_bytes = self.as_ssz_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&ssz_bytes);
        hasher.finalize().into()
    }
    
    /// Get the hex-encoded content-addressed ID
    fn content_id_hex(&self) -> String {
        hex::encode(self.content_id())
    }
}

/// Automatically implement SimpleContentAddressable for all types that implement Encode
impl<T: Encode> SimpleContentAddressable for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::serialization::Encode;
    
    // Simple test struct for basic functionality
    #[derive(Debug, Clone, PartialEq)]
    struct TestStruct {
        value: u32,
        name: String,
    }
    
    // Manual implementation of Encode for testing
    impl Encode for TestStruct {
        fn as_ssz_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&self.value.to_le_bytes());
            bytes.extend_from_slice(&(self.name.len() as u32).to_le_bytes());
            bytes.extend_from_slice(self.name.as_bytes());
            bytes
        }
    }
    
    #[test]
    fn test_simple_content_addressing() {
        let struct1 = TestStruct {
            value: 42,
            name: "test".to_string(),
        };
        
        let struct2 = TestStruct {
            value: 42,
            name: "test".to_string(),
        };
        
        let struct3 = TestStruct {
            value: 43,
            name: "test".to_string(),
        };
        
        // Same content should produce same ID
        let id1 = struct1.content_id();
        let id2 = struct2.content_id();
        assert_eq!(id1, id2);
        
        // Different content should produce different ID
        let id3 = struct3.content_id();
        assert_ne!(id1, id3);
        
        // Test hex encoding
        let hex1 = struct1.content_id_hex();
        let hex2 = struct2.content_id_hex();
        assert_eq!(hex1, hex2);
        
        println!("Content ID: {}", hex1);
    }
    
    #[test]
    fn test_content_id_from_bytes() {
        let test_data = b"hello world";
        let id1 = content_id_from_bytes(test_data);
        let id2 = content_id_from_bytes(test_data);
        assert_eq!(id1, id2);
        
        let hex_id = content_id_hex_from_bytes(test_data);
        assert_eq!(hex_id, hex::encode(id1));
    }
} 