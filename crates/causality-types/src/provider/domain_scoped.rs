//! Domain Scoped Provider Interface
//!
//! Defines the AsDomainScoped Service Provider Interface (SPI).
//! This trait allows associating components with a specific domain
//! and optionally providing access to the Domain object.
//! Extended with SMT integration for direct TEG data writes.

use crate::core::domain::Domain;
use crate::primitive::ids::DomainId;
use crate::serialization::Encode;

/// Associates components with a domain and provides TEG data access capabilities.
///
/// This trait extends the basic domain scoping functionality with SMT integration
/// for direct TEG data writes, cross-domain operations, and content-addressable storage.
pub trait AsDomainScoped {
    /// Get the ID of the domain this component belongs to.
    fn domain_id(&self) -> DomainId;

    /// Check if this component is in the specified domain.
    fn is_in_domain(&self, domain_id: &DomainId) -> bool {
        self.domain_id() == *domain_id
    }

    /// Optionally provides a reference to the full Domain object.
    ///
    /// Implementors can override this method if they can provide direct access
    /// to their associated Domain object. Returns `None` by default, indicating
    /// that the Domain object might need to be fetched via other means
    /// (e.g., using the `domain_id()` with a resolver or context).
    fn get_domain_object(&self) -> Option<&Domain> {
        None
    }
    
    /// Generate a domain-scoped SMT key for this component's data
    /// 
    /// Creates keys that are isolated within this component's domain,
    /// ensuring proper separation and content-addressable storage.
    fn generate_scoped_key(&self, data_type: &str, data_id: &[u8]) -> String {
        let domain_id = self.domain_id();
        format!("{}-{}-{}", 
            domain_id.namespace_prefix(),
            data_type,
            hex::encode(data_id)
        )
    }
    
    /// Generate a domain-scoped TEG node key for this component
    fn generate_scoped_teg_node_key(&self, node_type: &str, node_id: &[u8]) -> String {
        self.domain_id().generate_teg_node_key(node_type, node_id)
    }
    
    /// Generate a domain-scoped TEG effect key for this component
    fn generate_scoped_teg_effect_key(&self, effect_id: &[u8]) -> String {
        self.domain_id().generate_teg_effect_key(effect_id)
    }
    
    /// Generate a domain-scoped TEG resource key for this component
    fn generate_scoped_teg_resource_key(&self, resource_id: &[u8]) -> String {
        self.domain_id().generate_teg_resource_key(resource_id)
    }
    
    /// Generate a domain-scoped TEG intent key for this component
    fn generate_scoped_teg_intent_key(&self, intent_id: &[u8]) -> String {
        self.domain_id().generate_teg_intent_key(intent_id)
    }
    
    /// Generate a domain-scoped TEG handler key for this component
    fn generate_scoped_teg_handler_key(&self, handler_id: &[u8]) -> String {
        self.domain_id().generate_teg_handler_key(handler_id)
    }
    
    /// Generate a domain-scoped TEG constraint key for this component
    fn generate_scoped_teg_constraint_key(&self, constraint_id: &[u8]) -> String {
        self.domain_id().generate_teg_constraint_key(constraint_id)
    }
    
    /// Check if this component supports direct TEG writes in its domain
    /// 
    /// Components can override this to indicate their TEG write capabilities.
    /// Default implementation assumes no direct write support.
    fn supports_direct_teg_writes(&self) -> bool {
        false
    }
    
    /// Check if this component supports cross-domain TEG references
    /// 
    /// Components can override this to enable cross-domain operations.
    /// Default implementation enables cross-domain support.
    fn supports_cross_domain_teg_refs(&self) -> bool {
        true
    }
    
    /// Get the maximum number of TEG nodes this component can handle per transaction
    /// 
    /// Components can override this to set domain-specific limits.
    /// Default implementation allows 1000 nodes per transaction.
    fn max_teg_nodes_per_transaction(&self) -> u32 {
        1000
    }
    
    /// Check if this component validates temporal constraints for TEG operations
    /// 
    /// Components can override this to control temporal validation behavior.
    /// Default implementation enables temporal constraint validation.
    fn validates_teg_temporal_constraints(&self) -> bool {
        true
    }
}

/// Extended trait for domain-scoped components that provide direct SMT TEG operations
/// 
/// This trait extends `AsDomainScoped` with methods for components that can directly
/// interact with SMT storage for TEG data operations, including writes, reads, and
/// cross-domain references.
pub trait AsDomainScopedSmtProvider: AsDomainScoped {
    /// Write TEG data directly to the domain's SMT storage
    /// 
    /// This bypasses validation for rapid development iteration.
    /// Should only be used by trusted components during development.
    fn write_teg_data_direct(&self, _key: &str, _data: &[u8]) -> Result<(), String> {
        // Default implementation returns an error - components must override
        Err(format!("Direct TEG write not supported for component in domain {}", 
                   self.domain_id()))
    }
    
    /// Read TEG data from the domain's SMT storage
    /// 
    /// Retrieves data using domain-scoped keys with proper isolation.
    fn read_teg_data(&self, _key: &str) -> Result<Option<Vec<u8>>, String> {
        // Default implementation returns an error - components must override
        Err(format!("TEG read not supported for component in domain {}", 
                   self.domain_id()))
    }
    
    /// Store a TEG node with automatic key generation and domain scoping
    fn store_teg_node<T: Encode>(&self, node_type: &str, node_id: &[u8], node_data: &T) -> Result<String, String> {
        let key = self.generate_scoped_teg_node_key(node_type, node_id);
        let serialized_data = node_data.as_ssz_bytes();
        self.write_teg_data_direct(&key, &serialized_data)?;
        Ok(key)
    }
    
    /// Store a TEG effect with automatic key generation and domain scoping
    fn store_teg_effect<T: Encode>(&self, effect_id: &[u8], effect_data: &T) -> Result<String, String> {
        let key = self.generate_scoped_teg_effect_key(effect_id);
        let serialized_data = effect_data.as_ssz_bytes();
        self.write_teg_data_direct(&key, &serialized_data)?;
        Ok(key)
    }
    
    /// Store a TEG resource with automatic key generation and domain scoping
    fn store_teg_resource<T: Encode>(&self, resource_id: &[u8], resource_data: &T) -> Result<String, String> {
        let key = self.generate_scoped_teg_resource_key(resource_id);
        let serialized_data = resource_data.as_ssz_bytes();
        self.write_teg_data_direct(&key, &serialized_data)?;
        Ok(key)
    }
    
    /// Store a TEG intent with automatic key generation and domain scoping
    fn store_teg_intent<T: Encode>(&self, intent_id: &[u8], intent_data: &T) -> Result<String, String> {
        let key = self.generate_scoped_teg_intent_key(intent_id);
        let serialized_data = intent_data.as_ssz_bytes();
        self.write_teg_data_direct(&key, &serialized_data)?;
        Ok(key)
    }
    
    /// Store a TEG handler with automatic key generation and domain scoping
    fn store_teg_handler<T: Encode>(&self, handler_id: &[u8], handler_data: &T) -> Result<String, String> {
        let key = self.generate_scoped_teg_handler_key(handler_id);
        let serialized_data = handler_data.as_ssz_bytes();
        self.write_teg_data_direct(&key, &serialized_data)?;
        Ok(key)
    }
    
    /// Store a TEG constraint with automatic key generation and domain scoping
    fn store_teg_constraint<T: Encode>(&self, constraint_id: &[u8], constraint_data: &T) -> Result<String, String> {
        let key = self.generate_scoped_teg_constraint_key(constraint_id);
        let serialized_data = constraint_data.as_ssz_bytes();
        self.write_teg_data_direct(&key, &serialized_data)?;
        Ok(key)
    }
    
    /// Create a cross-domain reference from this domain to another
    /// 
    /// This enables TEG data in this domain to reference data in other domains
    /// while maintaining proper isolation and validation.
    fn create_cross_domain_teg_ref(&self, target_domain: &DomainId, target_id: &[u8], ref_data: &[u8]) -> Result<String, String> {
        if !self.supports_cross_domain_teg_refs() {
            return Err("Cross-domain TEG references not supported by this component".to_string());
        }
        
        let key = self.domain_id().generate_cross_domain_ref_key(target_domain, target_id);
        self.write_teg_data_direct(&key, ref_data)?;
        Ok(key)
    }
    
    /// Get the current state root for this component's domain
    /// 
    /// This provides access to the cryptographic state root that represents
    /// all TEG data within the component's domain.
    fn get_domain_state_root(&self) -> Result<Option<[u8; 32]>, String> {
        // Default implementation returns None - components must override
        Ok(None)
    }
    
    /// Update the state root for this component's domain
    /// 
    /// This should be called after batch TEG operations to update the
    /// cryptographic commitment to the domain's current state.
    fn update_domain_state_root(&self, _new_root: [u8; 32]) -> Result<(), String> {
        // Default implementation returns an error - components must override
        Err(format!("State root update not supported for component in domain {}", 
                   self.domain_id()))
    }
    
    /// Batch write multiple TEG operations atomically
    /// 
    /// This enables efficient bulk operations while maintaining atomicity
    /// and proper state root updates.
    fn batch_write_teg_data(&self, operations: Vec<(String, Vec<u8>)>) -> Result<(), String> {
        // Default implementation uses individual writes - components can override for optimization
        for (key, data) in operations {
            self.write_teg_data_direct(&key, &data)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    // Mock component for testing
    struct MockDomainScopedComponent {
        domain_id: DomainId,
        supports_direct_writes: bool,
        supports_cross_domain: bool,
        max_nodes: u32,
        validates_temporal: bool,
    }

    impl AsDomainScoped for MockDomainScopedComponent {
        fn domain_id(&self) -> DomainId {
            self.domain_id
        }

        fn supports_direct_teg_writes(&self) -> bool {
            self.supports_direct_writes
        }

        fn supports_cross_domain_teg_refs(&self) -> bool {
            self.supports_cross_domain
        }

        fn max_teg_nodes_per_transaction(&self) -> u32 {
            self.max_nodes
        }

        fn validates_teg_temporal_constraints(&self) -> bool {
            self.validates_temporal
        }
    }

    impl AsDomainScopedSmtProvider for MockDomainScopedComponent {
        fn write_teg_data_direct(&self, _key: &str, _data: &[u8]) -> Result<(), String> {
            Ok(()) // Mock implementation for testing
        }

        fn read_teg_data(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
            // Mock implementation returns some test data
            if key.contains("test") {
                Ok(Some(vec![1, 2, 3, 4]))
            } else {
                Ok(None)
            }
        }

        fn get_domain_state_root(&self) -> Result<Option<[u8; 32]>, String> {
            Ok(Some([42u8; 32]))
        }

        fn update_domain_state_root(&self, _new_root: [u8; 32]) -> Result<(), String> {
            Ok(()) // Mock implementation for testing
        }
    }

    // Simple test data structure
    #[derive(Debug, Clone, PartialEq)]
    struct TestTegData {
        value: u32,
    }

    impl Encode for TestTegData {
        fn as_ssz_bytes(&self) -> Vec<u8> {
            self.value.to_le_bytes().to_vec()
        }
    }

    fn create_test_component() -> MockDomainScopedComponent {
        MockDomainScopedComponent {
            domain_id: DomainId([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]),
            supports_direct_writes: true,
            supports_cross_domain: true,
            max_nodes: 500,
            validates_temporal: true,
        }
    }

    #[test]
    fn test_basic_domain_scoped_functionality() {
        let component = create_test_component();
        
        // Test basic domain scoping
        assert_eq!(component.domain_id(), component.domain_id);
        assert!(component.is_in_domain(&component.domain_id));
        
        let other_domain = DomainId::default();
        assert!(!component.is_in_domain(&other_domain));
        
        // Test TEG configuration
        assert!(component.supports_direct_teg_writes());
        assert!(component.supports_cross_domain_teg_refs());
        assert_eq!(component.max_teg_nodes_per_transaction(), 500);
        assert!(component.validates_teg_temporal_constraints());
    }

    #[test]
    fn test_scoped_key_generation() {
        let component = create_test_component();
        let test_id = [1, 2, 3, 4];
        
        // Test generic scoped key
        let scoped_key = component.generate_scoped_key("custom", &test_id);
        assert!(scoped_key.contains("domain-0102030405060708"));
        assert!(scoped_key.contains("custom"));
        assert!(scoped_key.contains("01020304"));
        
        // Test TEG-specific keys
        let node_key = component.generate_scoped_teg_node_key("effect", &test_id);
        assert!(node_key.contains("domain-0102030405060708"));
        assert!(node_key.contains("-teg-effect-"));
        assert!(node_key.contains("01020304"));
        
        let effect_key = component.generate_scoped_teg_effect_key(&test_id);
        assert!(effect_key.contains("-teg-effect-01020304"));
        
        let resource_key = component.generate_scoped_teg_resource_key(&test_id);
        assert!(resource_key.contains("-teg-resource-01020304"));
        
        let intent_key = component.generate_scoped_teg_intent_key(&test_id);
        assert!(intent_key.contains("-teg-intent-01020304"));
        
        let handler_key = component.generate_scoped_teg_handler_key(&test_id);
        assert!(handler_key.contains("-teg-handler-01020304"));
        
        let constraint_key = component.generate_scoped_teg_constraint_key(&test_id);
        assert!(constraint_key.contains("-teg-constraint-01020304"));
    }

    #[test]
    fn test_teg_data_storage() {
        let component = create_test_component();
        let test_data = TestTegData { value: 42 };
        let test_id = [5, 6, 7, 8];
        
        // Test storing different types of TEG data
        let node_key = component.store_teg_node("effect", &test_id, &test_data).unwrap();
        assert!(node_key.contains("-teg-effect-05060708"));
        
        let effect_key = component.store_teg_effect(&test_id, &test_data).unwrap();
        assert!(effect_key.contains("-teg-effect-05060708"));
        
        let resource_key = component.store_teg_resource(&test_id, &test_data).unwrap();
        assert!(resource_key.contains("-teg-resource-05060708"));
        
        let intent_key = component.store_teg_intent(&test_id, &test_data).unwrap();
        assert!(intent_key.contains("-teg-intent-05060708"));
        
        let handler_key = component.store_teg_handler(&test_id, &test_data).unwrap();
        assert!(handler_key.contains("-teg-handler-05060708"));
        
        let constraint_key = component.store_teg_constraint(&test_id, &test_data).unwrap();
        assert!(constraint_key.contains("-teg-constraint-05060708"));
    }

    #[test]
    fn test_cross_domain_references() {
        let component = create_test_component();
        let target_domain = DomainId([9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0,
                                     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let target_id = [10, 11, 12, 13];
        let ref_data = vec![20, 21, 22, 23];
        
        // Test successful cross-domain reference creation
        let cross_ref_key = component.create_cross_domain_teg_ref(&target_domain, &target_id, &ref_data).unwrap();
        assert!(cross_ref_key.contains("domain-0102030405060708"));
        assert!(cross_ref_key.contains("-cross-ref-"));
        assert!(cross_ref_key.contains("0908070605040302"));
        assert!(cross_ref_key.contains("0a0b0c0d"));
        
        // Test with component that doesn't support cross-domain refs
        let mut no_cross_domain_component = create_test_component();
        no_cross_domain_component.supports_cross_domain = false;
        
        let result = no_cross_domain_component.create_cross_domain_teg_ref(&target_domain, &target_id, &ref_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cross-domain TEG references not supported"));
    }

    #[test]
    fn test_state_root_operations() {
        let component = create_test_component();
        
        // Test getting state root
        let state_root = component.get_domain_state_root().unwrap();
        assert_eq!(state_root, Some([42u8; 32]));
        
        // Test updating state root
        let new_root = [100u8; 32];
        assert!(component.update_domain_state_root(new_root).is_ok());
    }

    #[test]
    fn test_batch_operations() {
        let component = create_test_component();
        
        // Test batch write operations
        let operations = vec![
            ("key1".to_string(), vec![1, 2, 3]),
            ("key2".to_string(), vec![4, 5, 6]),
            ("key3".to_string(), vec![7, 8, 9]),
        ];
        
        assert!(component.batch_write_teg_data(operations).is_ok());
    }

    #[test]
    fn test_teg_data_reads() {
        let component = create_test_component();
        
        // Test reading existing data
        let result = component.read_teg_data("test-key");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(vec![1, 2, 3, 4]));
        
        // Test reading non-existing data
        let result = component.read_teg_data("non-existent-key");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_default_implementations() {
        // Test component with all default implementations
        struct MinimalComponent {
            domain_id: DomainId,
        }

        impl AsDomainScoped for MinimalComponent {
            fn domain_id(&self) -> DomainId {
                self.domain_id
            }
        }

        let minimal = MinimalComponent {
            domain_id: DomainId::default(),
        };

        // Test default values
        assert!(!minimal.supports_direct_teg_writes());
        assert!(minimal.supports_cross_domain_teg_refs());
        assert_eq!(minimal.max_teg_nodes_per_transaction(), 1000);
        assert!(minimal.validates_teg_temporal_constraints());
        assert!(minimal.get_domain_object().is_none());
    }

    #[test]
    fn test_domain_isolation_in_keys() {
        let domain1 = DomainId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                               0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let domain2 = DomainId([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                               0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let component1 = MockDomainScopedComponent {
            domain_id: domain1,
            supports_direct_writes: true,
            supports_cross_domain: true,
            max_nodes: 1000,
            validates_temporal: true,
        };

        let component2 = MockDomainScopedComponent {
            domain_id: domain2,
            supports_direct_writes: true,
            supports_cross_domain: true,
            max_nodes: 1000,
            validates_temporal: true,
        };

        let test_id = [42, 42, 42, 42];

        // Same effect ID in different domains should generate different keys
        let key1 = component1.generate_scoped_teg_effect_key(&test_id);
        let key2 = component2.generate_scoped_teg_effect_key(&test_id);

        assert_ne!(key1, key2);
        assert!(key1.contains("domain-0100000000000000"));
        assert!(key2.contains("domain-0200000000000000"));
    }
}
