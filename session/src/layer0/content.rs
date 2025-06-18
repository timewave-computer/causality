// Content addressing for messages using SHA256

use sha2::{Sha256, Digest};
use std::fmt;
use serde::{Serialize, Deserialize};

/// A content-addressed message ID (SHA256 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MessageId([u8; 32]);

impl MessageId {
    /// Create a MessageId from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        MessageId(bytes)
    }

    /// Get the raw bytes of the MessageId
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Compute MessageId from arbitrary data
    pub fn from_data(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        MessageId(bytes)
    }
}

impl fmt::Debug for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageId({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

// Helper module for hex encoding (minimal implementation)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
