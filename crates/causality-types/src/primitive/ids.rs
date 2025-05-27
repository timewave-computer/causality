//! This module defines the core ID types used throughout the Causality system.
//!
//! All IDs are 32-byte content-addressed identifiers that uniquely identify
//! various objects in the system (resources, effects, expressions, etc).

// Standard library
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;
use hex;

// Local imports
use crate::serialization::{Decode, DecodeWithLength, Encode, SimpleSerialize};
use crate::utils::AsIdentifiable;

// External imports
use sha2::{Digest, Sha256};

/// Core trait for types that serve as identifiers.
/// The trait provides a uniform way to work with ID types.
pub trait AsId: Copy + Debug + Display + PartialEq + Eq + Send + Sync + 'static {
    /// Returns the inner byte representation of the ID.
    fn inner(&self) -> [u8; 32];

    /// Creates a new ID from raw bytes.
    fn new(bytes: [u8; 32]) -> Self;

    /// Creates a null ID (all zeros).
    fn null() -> Self {
        Self::new([0u8; 32])
    }

    /// Checks if this ID is the null ID (all zeros).
    fn is_null(&self) -> bool {
        self.inner() == [0u8; 32]
    }

    /// Returns a hex string representation of the ID.
    fn to_hex(&self) -> String {
        let bytes = self.inner();
        hex::encode(bytes)
    }

    /// Creates an ID from a hex string.
    fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self::new(bytes))
    }
}

/// Trait for converting between different ID types.
pub trait AsIdConverter<T: AsId> {
    /// Convert this ID to another ID type.
    fn to_id(&self) -> T;
}

// Define a macro to generate ID type structs to avoid repetition
macro_rules! define_id_type {
    ($(#[$attr:meta])* $name:ident) => {
        $(#[$attr])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
        #[repr(C)]
        pub struct $name(pub [u8; 32]);

        impl $name {
            /// Create a new ID from the given byte array
            pub fn new(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }
        }

        impl AsId for $name {
            fn inner(&self) -> [u8; 32] {
                self.0
            }

            fn new(bytes: [u8; 32]) -> Self {
                Self::new(bytes)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}", hex::encode(&self.0[..8]))
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), hex::encode(&self.0[..8]))
            }
        }

        impl FromStr for $name {
            type Err = hex::FromHexError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(s, &mut bytes)?;
                Ok(Self(bytes))
            }
        }

        impl From<[u8; 32]> for $name {
            fn from(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }
        }

        // Implement conversion between ID types
        impl<T: AsId> AsIdConverter<T> for $name {
            fn to_id(&self) -> T {
                T::new(self.0)
            }
        }
        
        // Implement SSZ serialization traits
        impl Encode for $name {
            fn as_ssz_bytes(&self) -> Vec<u8> {
                self.0.to_vec()
            }
        }
        
        impl Decode for $name {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
                if bytes.len() != 32 {
                    return Err(crate::serialization::DecodeError {
                        message: format!("Invalid ID length {}, expected 32", bytes.len()),
                    });
                }
                let mut id_bytes = [0u8; 32];
                id_bytes.copy_from_slice(bytes);
                Ok(Self(id_bytes))
            }
        }
        
        impl SimpleSerialize for $name {}
        
        impl DecodeWithLength for $name {
            fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), crate::serialization::DecodeError> {
                if bytes.len() < 32 {
                    return Err(crate::serialization::DecodeError {
                        message: format!("Invalid ID length {}, expected at least 32", bytes.len()),
                    });
                }
                let mut id_bytes = [0u8; 32];
                id_bytes.copy_from_slice(&bytes[..32]);
                Ok((Self(id_bytes), 32))
            }
        }
    };
}

// Define all ID types using the macro
define_id_type!(ResourceId);
define_id_type!(EffectId);
define_id_type!(IntentId);
define_id_type!(HandlerId);
define_id_type!(ExprId);
define_id_type!(ValueExprId);
define_id_type!(TypeExprId);
define_id_type!(DomainId);
define_id_type!(EntityId);

/// Enhanced SMT key generation capabilities for DomainId
impl DomainId {
    /// Generate a namespace prefix for this domain
    /// 
    /// This creates a consistent string prefix used for all SMT keys
    /// within this domain, ensuring proper isolation between domains.
    pub fn namespace_prefix(&self) -> String {
        format!("domain-{}", hex::encode(&self.0[..8]))
    }
    
    /// Generate SMT key for TEG node storage in this domain
    /// 
    /// Creates a hierarchical key structure: domain-{id}-teg-{type}-{node_id}
    /// This ensures all TEG nodes are properly namespaced by domain.
    pub fn generate_teg_node_key(&self, node_type: &str, node_id: &[u8]) -> String {
        format!("{}-teg-{}-{}", 
            self.namespace_prefix(),
            node_type,
            hex::encode(node_id)
        )
    }
    
    /// Generate SMT key for TEG effect storage in this domain
    pub fn generate_teg_effect_key(&self, effect_id: &[u8]) -> String {
        format!("{}-teg-effect-{}", 
            self.namespace_prefix(),
            hex::encode(effect_id)
        )
    }
    
    /// Generate SMT key for TEG resource storage in this domain
    pub fn generate_teg_resource_key(&self, resource_id: &[u8]) -> String {
        format!("{}-teg-resource-{}", 
            self.namespace_prefix(),
            hex::encode(resource_id)
        )
    }
    
    /// Generate SMT key for TEG intent storage in this domain
    pub fn generate_teg_intent_key(&self, intent_id: &[u8]) -> String {
        format!("{}-teg-intent-{}", 
            self.namespace_prefix(),
            hex::encode(intent_id)
        )
    }
    
    /// Generate SMT key for TEG handler storage in this domain
    pub fn generate_teg_handler_key(&self, handler_id: &[u8]) -> String {
        format!("{}-teg-handler-{}", 
            self.namespace_prefix(),
            hex::encode(handler_id)
        )
    }
    
    /// Generate SMT key for TEG constraint storage in this domain
    pub fn generate_teg_constraint_key(&self, constraint_id: &[u8]) -> String {
        format!("{}-teg-constraint-{}", 
            self.namespace_prefix(),
            hex::encode(constraint_id)
        )
    }
    
    /// Generate SMT key for TEG transaction storage in this domain
    pub fn generate_teg_transaction_key(&self, transaction_id: &[u8]) -> String {
        format!("{}-teg-transaction-{}", 
            self.namespace_prefix(),
            hex::encode(transaction_id)
        )
    }
    
    /// Generate SMT key for cross-domain reference storage
    /// 
    /// This enables storing references from this domain to entities in other domains
    /// with the format: domain-{source}-cross-ref-{target_domain}-{target_id}
    pub fn generate_cross_domain_ref_key(&self, target_domain: &DomainId, target_id: &[u8]) -> String {
        format!("{}-cross-ref-{}-{}", 
            self.namespace_prefix(),
            hex::encode(&target_domain.0[..8]),
            hex::encode(target_id)
        )
    }
    
    /// Generate SMT key for domain state metadata
    /// 
    /// Stores domain-level configuration and state information
    pub fn generate_domain_state_key(&self) -> String {
        format!("{}-state", self.namespace_prefix())
    }
    
    /// Generate SMT key for domain configuration storage
    pub fn generate_domain_config_key(&self) -> String {
        format!("{}-config", self.namespace_prefix())
    }
    
    /// Generate SMT key for temporal constraint metadata in this domain
    pub fn generate_temporal_constraint_key(&self, constraint_id: &[u8]) -> String {
        format!("{}-temporal-{}", 
            self.namespace_prefix(),
            hex::encode(constraint_id)
        )
    }
    
    /// Generate SMT key for domain root hash storage
    /// 
    /// This stores the current state root of all TEG data within this domain
    pub fn generate_domain_root_key(&self) -> String {
        format!("{}-root", self.namespace_prefix())
    }
    
    /// Generate a unique subdomain ID for hierarchical domain organization
    /// 
    /// This enables creating child domains under this parent domain
    pub fn generate_subdomain_id(&self, subdomain_name: &str) -> DomainId {
        let mut hasher = Sha256::new();
        hasher.update(self.0);
        hasher.update(b"-subdomain-");
        hasher.update(subdomain_name.as_bytes());
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        DomainId(bytes)
    }
    
    /// Check if this domain ID represents a subdomain of the given parent
    pub fn is_subdomain_of(&self, parent: &DomainId) -> bool {
        // This is a simplified check - in practice would need to store
        // the relationship in the SMT or derive it from construction
        self.0[0..16] == parent.0[0..16]
    }
    
    /// Get all possible key prefixes for this domain
    /// 
    /// This is useful for bulk operations and domain cleanup
    pub fn get_key_prefixes(&self) -> Vec<String> {
        let base_prefix = self.namespace_prefix();
        vec![
            format!("{}-teg-", base_prefix),
            format!("{}-cross-ref-", base_prefix),
            format!("{}-state", base_prefix),
            format!("{}-config", base_prefix),
            format!("{}-temporal-", base_prefix),
            format!("{}-root", base_prefix),
        ]
    }
}

define_id_type!(NodeId);
define_id_type!(EdgeId);
define_id_type!(CircuitId);
define_id_type!(ProgramId);
define_id_type!(SubgraphId);
define_id_type!(GraphId);
define_id_type!(TransactionId);
define_id_type!(MessageId);
define_id_type!(ServiceId);
define_id_type!(DataId);
define_id_type!(NullifierId);

/// Computes a content-addressed ID from the given data using SHA-256
pub fn compute_id(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    bytes
}

/// Implementation of AsIdentifiable for any type that implements AsId
impl<T: AsId> AsIdentifiable for T {
    type Id = T;

    fn id(&self) -> Self::Id {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_id_namespace_prefix() {
        let domain_id = DomainId([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                                 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
        
        let prefix = domain_id.namespace_prefix();
        assert_eq!(prefix, "domain-0102030405060708");
    }
    
    #[test] 
    fn test_domain_id_teg_key_generation() {
        let domain_id = DomainId::default();
        let test_id = [1, 2, 3, 4];
        
        // Test different TEG key types
        let node_key = domain_id.generate_teg_node_key("effect", &test_id);
        assert!(node_key.starts_with("domain-"));
        assert!(node_key.contains("-teg-effect-"));
        assert!(node_key.contains("01020304"));
        
        let effect_key = domain_id.generate_teg_effect_key(&test_id);
        assert!(effect_key.contains("-teg-effect-01020304"));
        
        let resource_key = domain_id.generate_teg_resource_key(&test_id);
        assert!(resource_key.contains("-teg-resource-01020304"));
        
        let intent_key = domain_id.generate_teg_intent_key(&test_id);
        assert!(intent_key.contains("-teg-intent-01020304"));
        
        let handler_key = domain_id.generate_teg_handler_key(&test_id);
        assert!(handler_key.contains("-teg-handler-01020304"));
        
        let constraint_key = domain_id.generate_teg_constraint_key(&test_id);
        assert!(constraint_key.contains("-teg-constraint-01020304"));
        
        let transaction_key = domain_id.generate_teg_transaction_key(&test_id);
        assert!(transaction_key.contains("-teg-transaction-01020304"));
    }
    
    #[test]
    fn test_domain_id_cross_domain_refs() {
        let source_domain = DomainId([1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let target_domain = DomainId([2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0,
                                     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let target_id = [3, 3, 3, 3];
        
        let cross_ref_key = source_domain.generate_cross_domain_ref_key(&target_domain, &target_id);
        
        assert!(cross_ref_key.starts_with("domain-0101010101010101"));
        assert!(cross_ref_key.contains("-cross-ref-"));
        assert!(cross_ref_key.contains("0202020202020202"));
        assert!(cross_ref_key.contains("03030303"));
    }
    
    #[test]
    fn test_domain_id_state_and_config_keys() {
        let domain_id = DomainId::default();
        
        let state_key = domain_id.generate_domain_state_key();
        assert!(state_key.ends_with("-state"));
        
        let config_key = domain_id.generate_domain_config_key();
        assert!(config_key.ends_with("-config"));
        
        let root_key = domain_id.generate_domain_root_key();
        assert!(root_key.ends_with("-root"));
    }
    
    #[test]
    fn test_domain_id_temporal_constraints() {
        let domain_id = DomainId::default();
        let constraint_id = [4, 5, 6, 7];
        
        let temporal_key = domain_id.generate_temporal_constraint_key(&constraint_id);
        assert!(temporal_key.contains("-temporal-"));
        assert!(temporal_key.contains("04050607"));
    }
    
    #[test]
    fn test_domain_id_subdomain_generation() {
        let parent_domain = DomainId([10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160,
                                     10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160]);
        
        let subdomain1 = parent_domain.generate_subdomain_id("child1");
        let subdomain2 = parent_domain.generate_subdomain_id("child2");
        
        // Different subdomains should have different IDs
        assert_ne!(subdomain1, subdomain2);
        
        // Same subdomain name should generate the same ID
        let subdomain1_again = parent_domain.generate_subdomain_id("child1");
        assert_eq!(subdomain1, subdomain1_again);
        
        // Subdomains should be different from parent
        assert_ne!(subdomain1, parent_domain);
    }
    
    #[test]
    fn test_domain_id_key_prefixes() {
        let domain_id = DomainId::default();
        let prefixes = domain_id.get_key_prefixes();
        
        assert!(!prefixes.is_empty());
        assert!(prefixes.iter().any(|p| p.contains("-teg-")));
        assert!(prefixes.iter().any(|p| p.contains("-cross-ref-")));
        assert!(prefixes.iter().any(|p| p.contains("-state")));
        assert!(prefixes.iter().any(|p| p.contains("-config")));
        assert!(prefixes.iter().any(|p| p.contains("-temporal-")));
        assert!(prefixes.iter().any(|p| p.contains("-root")));
    }
    
    #[test]
    fn test_domain_id_isolation() {
        let domain1 = DomainId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                               0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let domain2 = DomainId([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                               0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        let test_id = [42, 42, 42, 42];
        
        let key1 = domain1.generate_teg_effect_key(&test_id);
        let key2 = domain2.generate_teg_effect_key(&test_id);
        
        // Same effect ID in different domains should generate different keys
        assert_ne!(key1, key2);
        assert!(key1.contains("domain-0100000000000000"));
        assert!(key2.contains("domain-0200000000000000"));
    }
}
