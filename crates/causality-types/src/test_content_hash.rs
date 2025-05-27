// Test content hash utility for testing

use crate::ContentHash;

/// Create a test content hash
/// 
/// This function creates a ContentHash with a predictable value for testing purposes.
/// It uses the SHA-256 algorithm with a fixed array of zeros as the hash value.
pub fn test_content_hash() -> ContentHash {
    ContentHash::new("sha256", vec![0u8; 32])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_hash_works() {
        let hash = test_content_hash();
        assert_eq!(hash.algorithm, "sha256");
        assert_eq!(hash.bytes.len(), 32);
        for byte in &hash.bytes {
            assert_eq!(*byte, 0);
        }
    }
} 