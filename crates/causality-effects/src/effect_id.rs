// Effect identification system
// Original file: src/effect/effect_id.rs

use std::fmt;
use borsh::{BorshSerialize, BorshDeserialize};
use causality_crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// Uniquely identifies an effect
#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct EffectId {
    /// Unique identifier for the effect
    id: String,
}

/// Simple content data for effect ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct EffectIdContent {
    /// Type of effect
    effect_type: String,
    /// Creation timestamp
    timestamp: u64,
    /// Random component
    nonce: [u8; 8],
}

impl ContentAddressed for EffectIdContent {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl EffectId {
    /// Create a new effect ID with the given ID
    pub fn new(id: String) -> Self {
        Self { id }
    }
    
    /// Create a new unique effect ID using content-derived identifier
    pub fn new_unique() -> Self {
        // Create content for the effect ID
        let content = EffectIdContent {
            effect_type: "generic".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Get content ID
        let content_id = content.content_id();
        
        // Format as string
        Self { id: format!("effect:{}", content_id) }
    }
    
    /// Create a new effect ID for a specific effect type
    pub fn for_effect_type(effect_type: &str) -> Self {
        // Create content for the effect ID
        let content = EffectIdContent {
            effect_type: effect_type.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Get content ID
        let content_id = content.content_id();
        
        // Format as string
        Self { id: format!("effect:{}:{}", effect_type, content_id) }
    }
    
    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<String> for EffectId {
    fn from(id: String) -> Self {
        Self { id }
    }
}

impl From<&str> for EffectId {
    fn from(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl From<ContentId> for EffectId {
    fn from(content_id: ContentId) -> Self {
        Self { id: format!("effect:{}", content_id) }
    }
} 