// Domain module
//
// Represents the concept of a domain in Causality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use causality_types::{ContentId, Timestamp};

/// Unique identifier for a domain
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct DomainId(String);

impl DomainId {
    /// Create a new DomainId
    pub fn new(id: &str) -> Self {
        DomainId(id.to_string())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DomainId({})", self.0)
    }
}

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for DomainId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for DomainId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<&str> for DomainId {
    fn from(s: &str) -> Self {
        DomainId(s.to_string())
    }
}

// TimeMap is now imported from causality-core
// We remove the local definition

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_id_creation() {
        let id = DomainId::new("test-domain");
        assert_eq!(id.as_str(), "test-domain");
    }

    #[test]
    fn test_domain_id_equality() {
        let id1 = DomainId::new("domain1");
        let id2 = DomainId::new("domain1");
        let id3 = DomainId::new("domain2");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_domain_id_hashing() {
        let mut map = HashMap::new();
        let id1 = DomainId::new("domain1");
        let id2 = DomainId::new("domain1");
        map.insert(id1, "value1");
        assert!(map.contains_key(&id2));
    }

    #[test]
    fn test_domain_id_from_str() {
        let id: DomainId = "test-domain".into();
        assert_eq!(id.as_str(), "test-domain");
    }

    #[test]
    fn test_domain_id_display_debug() {
        let id = DomainId::new("test-domain");
        assert_eq!(format!("{}", id), "test-domain");
        assert_eq!(format!("{:?}", id), "DomainId(test-domain)");
    }
} 