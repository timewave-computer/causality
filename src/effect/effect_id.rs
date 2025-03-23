use std::fmt;
use uuid::Uuid;

/// Uniquely identifies an effect
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectId {
    /// Unique identifier for the effect
    id: String,
}

impl EffectId {
    /// Create a new effect ID with the given ID
    pub fn new(id: String) -> Self {
        Self { id }
    }
    
    /// Create a new unique effect ID
    pub fn new_unique() -> Self {
        Self { id: Uuid::new_v4().to_string() }
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