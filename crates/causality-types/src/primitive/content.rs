// Purpose: Base trait and types for content-addressable nodes with domain awareness

use crate::{
    primitive::ids::{DomainId, NodeId},
    serialization::{Encode, Decode},
};
use anyhow::Result;
use sha2::{Digest, Sha256};

/// Base trait for all content-addressable nodes in the system
pub trait ContentAddressable: Encode + Decode + Clone + Send + Sync {
    /// Get the domain this node belongs to
    fn domain_id(&self) -> DomainId;
    
    /// Get the content-addressable ID of this node based on its serialized content
    fn content_id(&self) -> NodeId {
        let serialized = self.as_ssz_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        NodeId::new(hash_array)
    }
    
    /// Get the namespace prefix for this node type within its domain
    fn namespace_prefix(&self) -> String {
        format!("{}-{}", self.domain_id().namespace_prefix(), self.node_type_name())
    }
    
    /// Get the type name for this node (used in key generation)
    fn node_type_name(&self) -> &'static str;
    
    /// Validate the node content and domain constraints
    fn validate(&self) -> Result<()> {
        // Default validation - override for domain-specific constraints
        Ok(())
    }
    
    /// Get the SMT key for storing this node
    fn smt_key(&self) -> String {
        format!("{}-{}", self.namespace_prefix(), self.content_id())
    }
}

/// Trait for nodes that can traverse and inspect other content-addressable nodes
pub trait ContentTraversable: ContentAddressable {
    /// Get all direct child node references
    fn child_refs(&self) -> Vec<NodeId>;
    
    /// Get all nodes this node references across domains
    fn cross_domain_refs(&self) -> Vec<(DomainId, NodeId)>;
    
    /// Check if this node references another node
    fn references(&self, node_id: &NodeId) -> bool {
        self.child_refs().contains(node_id)
    }
    
    /// Check if this node has cross-domain references
    fn has_cross_domain_refs(&self) -> bool {
        !self.cross_domain_refs().is_empty()
    }
}

/// Trait for nodes that support domain-specific validation
pub trait DomainValidated: ContentAddressable {
    /// Validate this node within its domain context
    fn validate_in_domain(&self, domain_id: &DomainId) -> Result<()>;
    
    /// Check if this node is compatible with a target domain
    fn is_compatible_with_domain(&self, domain_id: &DomainId) -> bool;
    
    /// Get domain-specific metadata for this node
    fn domain_metadata(&self) -> Option<Vec<u8>> {
        None // Default implementation
    }
}

/// Common interfaces for content-addressable node collections
pub trait ContentAddressableCollection<T: ContentAddressable> {
    /// Insert a node, returning its content ID
    fn insert(&mut self, node: T) -> Result<NodeId>;
    
    /// Get a node by its content ID
    fn get(&self, id: &NodeId) -> Option<&T>;
    
    /// Check if a node exists by content ID
    fn contains(&self, id: &NodeId) -> bool;
    
    /// Remove a node by content ID
    fn remove(&mut self, id: &NodeId) -> Option<T>;
    
    /// Get all nodes in a specific domain
    fn get_by_domain(&self, domain_id: &DomainId) -> Vec<&T>;
    
    /// Get count of nodes in this collection
    fn len(&self) -> usize;
    
    /// Check if collection is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::DomainId;
    use crate::serialization::{Encode, Decode, DecodeError};
    
    #[derive(Clone, Debug, PartialEq)]
    struct TestNode {
        domain: DomainId,
        data: String,
    }
    
    impl Encode for TestNode {
        fn as_ssz_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend(self.domain.as_ssz_bytes());
            bytes.extend(self.data.as_ssz_bytes());
            bytes
        }
    }
    
    impl Decode for TestNode {
        fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
            // Simple implementation for testing
            let domain = DomainId::from_ssz_bytes(&bytes[0..32])?;
            let data = String::from_ssz_bytes(&bytes[32..])?;
            Ok(TestNode { domain, data })
        }
    }
    
    impl ContentAddressable for TestNode {
        fn domain_id(&self) -> DomainId {
            self.domain
        }
        
        fn node_type_name(&self) -> &'static str {
            "test-node"
        }
    }
    
    #[test]
    fn test_content_addressable_id() {
        let domain = DomainId::new([1u8; 32]);
        let node = TestNode {
            domain,
            data: "test data".to_string(),
        };
        
        let id1 = node.content_id();
        let id2 = node.content_id();
        
        // Content ID should be deterministic
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_smt_key_generation() {
        let domain = DomainId::new([1u8; 32]);
        let node = TestNode {
            domain,
            data: "test data".to_string(),
        };
        
        let key = node.smt_key();
        assert!(key.starts_with(&domain.namespace_prefix()));
        assert!(key.contains("test-node"));
    }
} 