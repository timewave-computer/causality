// Fact log entry implementation
// Original file: src/log/entry/fact_entry.rs

// Fact entry implementation for Causality Unified Log System
//
// This module provides the FactEntry struct for representing facts in the log.

use serde::{Serialize, Deserialize};

use causality_types::{*};
use causality_crypto::ContentId;;

/// An entry representing an observed fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactEntry {
    /// The domain where this fact was observed
    pub domain: DomainId,
    /// The block height where this fact was observed
    pub block_height: BlockHeight,
    /// The block hash where this fact was observed
    pub block_hash: Option<BlockHash>,
    /// The timestamp when this fact was observed
    pub observed_at: Timestamp,
    /// The fact type
    pub fact_type: String,
    /// The resources related to this fact
    pub resources: Vec<ContentId>,
    /// The serialized fact data
    pub data: serde_json::Value,
    /// Whether the fact was verified
    pub verified: bool,
}

impl FactEntry {
    /// Create a new fact entry
    pub fn new(
        domain: DomainId,
        block_height: BlockHeight,
        block_hash: Option<BlockHash>,
        observed_at: Timestamp,
        fact_type: String,
        resources: Vec<ContentId>,
        data: serde_json::Value,
        verified: bool,
    ) -> Self {
        Self {
            domain,
            block_height,
            block_hash,
            observed_at,
            fact_type,
            resources,
            data,
            verified,
        }
    }
    
    /// Get the domain where this fact was observed
    pub fn domain(&self) -> &DomainId {
        &self.domain
    }
    
    /// Get the block height where this fact was observed
    pub fn block_height(&self) -> &BlockHeight {
        &self.block_height
    }
    
    /// Get the block hash where this fact was observed
    pub fn block_hash(&self) -> Option<&BlockHash> {
        self.block_hash.as_ref()
    }
    
    /// Get the timestamp when this fact was observed
    pub fn observed_at(&self) -> &Timestamp {
        &self.observed_at
    }
    
    /// Get the fact type
    pub fn fact_type(&self) -> &str {
        &self.fact_type
    }
    
    /// Get the resources related to this fact
    pub fn resources(&self) -> &[ContentId] {
        &self.resources
    }
    
    /// Get the fact data
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
    
    /// Check if this fact was verified
    pub fn is_verified(&self) -> bool {
        self.verified
    }
    
    /// Mark this fact as verified
    pub fn mark_verified(mut self) -> Self {
        self.verified = true;
        self
    }
    
    /// Mark this fact as unverified
    pub fn mark_unverified(mut self) -> Self {
        self.verified = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fact_entry_creation() {
        let domain = DomainId::new(1);
        let block_height = BlockHeight::new(100);
        let block_hash = Some(BlockHash::new("abc123".to_string()));
        let observed_at = Timestamp::now();
        let fact_type = "balance".to_string();
        let resources = vec![ContentId::new(1)];
        let data = serde_json::json!({"balance": 100});
        
        let entry = FactEntry::new(
            domain,
            block_height.clone(),
            block_hash.clone(),
            observed_at.clone(),
            fact_type.clone(),
            resources.clone(),
            data.clone(),
            false,
        );
        
        assert_eq!(entry.domain, domain);
        assert_eq!(entry.block_height, block_height);
        assert_eq!(entry.block_hash, block_hash);
        assert_eq!(entry.observed_at, observed_at);
        assert_eq!(entry.fact_type, fact_type);
        assert_eq!(entry.resources, resources);
        assert_eq!(entry.data, data);
        assert!(!entry.verified);
        
        // Test verification
        let verified = entry.clone().mark_verified();
        assert!(verified.verified);
        
        let unverified = verified.mark_unverified();
        assert!(!unverified.verified);
    }
} 
