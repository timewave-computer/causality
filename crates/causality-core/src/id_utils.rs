// ID Utilities
//
// Utilities for generating and working with identifiers.

use std::fmt;
use causality_types::crypto_primitives::ContentId;
use blake3;
use crate::serialization::Serializable;

/// Generate a content-addressed ID string
pub fn generate_content_id() -> String {
    // Generate random bytes and create ContentId
    let random_bytes = rand::random::<[u8; 16]>();
    let content_id = ContentId::from_bytes(&random_bytes);
    content_id.to_string()
}

/// Generate a random hex string of specified length
pub fn generate_random_hex(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; (length + 1) / 2];
    rng.fill(&mut bytes[..]);
    hex::encode(&bytes[0..(length + 1) / 2])[..length].to_string()
}

/// Generate a timestamp-based hex ID with random suffix
pub fn generate_timestamp_id() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let random_suffix = generate_random_hex(8);
    format!("{:x}{}", timestamp, random_suffix)
}

/// Fact identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactId {
    /// The content ID of the fact
    pub content_id: ContentId,
    /// The domain of the fact
    pub domain: String,
}

impl FactId {
    /// Create a new fact ID from parts
    pub fn new(content_id: ContentId, domain: impl Into<String>) -> Self {
        Self {
            content_id,
            domain: domain.into(),
        }
    }
    
    /// Create a fact ID from raw components
    pub fn from_raw(hash: &str, domain: impl Into<String>) -> Self {
        // Create a ContentId from the hash
        let content_id = ContentId::new(hash);
        Self::new(content_id, domain)
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.content_id, self.domain)
    }
    
    /// Create from string representation
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            return Err("Invalid FactId format");
        }
        
        // First part is the ContentId
        let content_id = ContentId::new(parts[0]);
        
        // Remaining parts are the domain, rejoined with ':' in case domain contains ':'
        let domain = parts[1..].join(":");
        
        Ok(Self::new(content_id, domain))
    }
    
    /// Create a composite hash for this fact ID
    pub fn composite_hash(&self) -> String {
        // Combine content ID and domain for composite hash
        let combined = format!("{}:{}", self.content_id, self.domain).into_bytes();
        let hash = blake3::hash(&combined);
        hex::encode(hash.as_bytes())
    }
    
    /// Get the numerical value of this fact ID (first 8 bytes of content ID hash as u64)
    pub fn value(&self) -> u64 {
        let hash_bytes = self.content_id.as_bytes();
        let mut bytes = [0u8; 8];
        let copy_len = std::cmp::min(hash_bytes.len(), 8);
        bytes[..copy_len].copy_from_slice(&hash_bytes[..copy_len]);
        u64::from_le_bytes(bytes)
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.content_id, self.domain)
    }
}

impl From<String> for FactId {
    fn from(s: String) -> Self {
        // Parse the string or create a default if parsing fails
        Self::from_string(&s).unwrap_or_else(|_| {
            Self::from_raw(&s, "default")
        })
    }
}

impl From<&str> for FactId {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

/// Generate a unique decision ID for committee operations
pub fn generate_decision_id() -> String {
    // Create a ContentId with decision prefix
    let seed = format!("decision_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let content_id = ContentId::from_bytes(seed.as_bytes());
    format!("decision_{}", content_id)
}

/// Generate a unique ID for system operations
pub fn generate_system_operation_id() -> String {
    // Create a ContentId with system operation prefix
    let seed = format!("sys_op_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let content_id = ContentId::from_bytes(seed.as_bytes());
    format!("sys_op_{}", content_id)
}

/// Generate a unique ID for maintenance windows
pub fn generate_maintenance_window_id() -> String {
    // Create a ContentId with maintenance prefix
    let seed = format!("maint_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let content_id = ContentId::from_bytes(seed.as_bytes());
    format!("maint_{}", content_id)
}

/// Generate a unique ID for transfers
pub fn generate_transfer_id() -> String {
    // Create a ContentId with transfer prefix
    let seed = format!("transfer_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let content_id = ContentId::from_bytes(seed.as_bytes());
    format!("transfer_{}", content_id)
}

/// Convert from causality_types::crypto_primitives::ContentId to causality_types::ContentId
pub fn convert_to_types_content_id(content_id: &ContentId) -> causality_types::ContentId {
    // Convert the crypto_primitives ContentId to standard ContentId
    let hash_hex = content_id.to_string();
    causality_types::ContentId::new(hash_hex)
}

/// Convert from causality_types::ContentId to causality_types::crypto_primitives::ContentId
pub fn convert_from_types_content_id(content_id: &causality_types::ContentId) -> ContentId {
    // Get the hex value from ContentId
    let content_hex = content_id.to_string();
    ContentId::new(content_hex)
}
