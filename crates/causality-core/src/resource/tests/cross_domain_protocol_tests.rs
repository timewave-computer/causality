use std::collections::HashMap;
use std::sync::Arc;

use crate::content::ContentId;
use crate::domain::DomainId;
use crate::capability::BasicCapability;
use crate::effect::context::EffectContextBuilder;
use crate::resource::{
    ResourceTypeId,
    CrossDomainResourceId,
    ResourceProjectionType,
    VerificationLevel,
    ResourceReference,
    VerificationResult,
    TransferStatus,
    ResourceTransferOperation,
    CrossDomainProtocolError,
    CrossDomainResourceProtocol,
    DomainResourceAdapter,
    InMemoryResourceTypeRegistry,
    create_cross_domain_protocol,
};

/// A more extensive test for the cross-domain resource protocol
#[tokio::test]
async fn test_cross_domain_resource_protocol() {
    // Test setup
    let resource_type_registry = Arc::new(InMemoryResourceTypeRegistry::new());
    let protocol = create_cross_domain_protocol(resource_type_registry.clone());
    
    // Create test domains
    let domain1 = DomainId::new("domain1");
    let domain2 = DomainId::new("domain2");
    
    // Create test resource types
    let resource_type = ResourceTypeId::new("document");
    
    // Create test resource IDs
    let content_id1 = ContentId::from_bytes(&[1, 2, 3, 4]).unwrap();
    let resource_id1 = CrossDomainResourceId::new(
        content_id1.clone(),
        domain1.clone(),
        resource_type.clone(),
    );
    
    // Create test context
    let context = EffectContextBuilder::new()
        .with_capability(BasicCapability::new("resource.read"))
        .with_capability(BasicCapability::new("resource.transfer"))
        .build();
    
    // Test reference creation
    // Note: This will fail due to mocked implementation
    let reference_result = protocol.create_reference(
        resource_id1.clone(),
        domain2.clone(),
        ResourceProjectionType::Shadow,
        VerificationLevel::Hash,
        &context,
    ).await;
    
    // Reference creation failed because the domain adapters are not registered
    assert!(reference_result.is_err());
    assert!(matches!(reference_result.unwrap_err(), CrossDomainProtocolError::DomainNotSupported(_)));
    
    // Additional test cases would be added for a real implementation:
    // - Testing verification of references
    // - Testing resource transfer between domains
    // - Testing synchronization of references
    // - Testing transfer status tracking
    // - Testing reference resolution
    
    // These would require proper domain adapter implementations and
    // resource type registration
}

/// Test resource projection types
#[test]
fn test_resource_projection_types() {
    // Test different projection types
    let shadow = ResourceProjectionType::Shadow;
    let bridged = ResourceProjectionType::Bridged;
    let locked = ResourceProjectionType::Locked;
    let transferred = ResourceProjectionType::Transferred;
    let custom = ResourceProjectionType::Custom(42);
    
    // Verify they're different
    assert_ne!(shadow, bridged);
    assert_ne!(shadow, locked);
    assert_ne!(shadow, transferred);
    assert_ne!(shadow, custom);
    
    assert_ne!(bridged, locked);
    assert_ne!(bridged, transferred);
    assert_ne!(bridged, custom);
    
    assert_ne!(locked, transferred);
    assert_ne!(locked, custom);
    
    assert_ne!(transferred, custom);
    
    // Test custom types with same value are equal
    let custom1 = ResourceProjectionType::Custom(42);
    let custom2 = ResourceProjectionType::Custom(42);
    assert_eq!(custom1, custom2);
    
    let custom3 = ResourceProjectionType::Custom(43);
    assert_ne!(custom1, custom3);
}

/// Test verification levels
#[test]
fn test_verification_levels() {
    // Test different verification levels
    let none = VerificationLevel::None;
    let hash = VerificationLevel::Hash;
    let merkle = VerificationLevel::MerkleProof;
    let zk = VerificationLevel::ZkProof;
    let consensus = VerificationLevel::Consensus;
    let multisig = VerificationLevel::MultiSig;
    let custom = VerificationLevel::Custom(42);
    
    // Verify they're different
    assert_ne!(none, hash);
    assert_ne!(none, merkle);
    assert_ne!(none, zk);
    assert_ne!(none, consensus);
    assert_ne!(none, multisig);
    assert_ne!(none, custom);
    
    assert_ne!(hash, merkle);
    assert_ne!(hash, zk);
    assert_ne!(hash, consensus);
    assert_ne!(hash, multisig);
    assert_ne!(hash, custom);
    
    assert_ne!(merkle, zk);
    assert_ne!(merkle, consensus);
    assert_ne!(merkle, multisig);
    assert_ne!(merkle, custom);
    
    assert_ne!(zk, consensus);
    assert_ne!(zk, multisig);
    assert_ne!(zk, custom);
    
    assert_ne!(consensus, multisig);
    assert_ne!(consensus, custom);
    
    assert_ne!(multisig, custom);
    
    // Test custom levels with same value are equal
    let custom1 = VerificationLevel::Custom(42);
    let custom2 = VerificationLevel::Custom(42);
    assert_eq!(custom1, custom2);
    
    let custom3 = VerificationLevel::Custom(43);
    assert_ne!(custom1, custom3);
}

/// Test resource reference
#[test]
fn test_resource_reference() {
    // Create test data
    let domain1 = DomainId::new("domain1");
    let domain2 = DomainId::new("domain2");
    let resource_type = ResourceTypeId::new("document");
    let content_id = ContentId::from_bytes(&[1, 2, 3, 4]).unwrap();
    
    let resource_id = CrossDomainResourceId::new(
        content_id.clone(),
        domain1.clone(),
        resource_type.clone(),
    );
    
    // Create a reference
    let reference = ResourceReference::new(
        resource_id.clone(),
        ResourceProjectionType::Shadow,
        VerificationLevel::Hash,
        domain2.clone(),
    );
    
    // Test properties
    assert_eq!(reference.id, resource_id);
    assert_eq!(reference.projection_type, ResourceProjectionType::Shadow);
    assert_eq!(reference.verification_level, VerificationLevel::Hash);
    assert_eq!(reference.target_domain, domain2);
    assert!(reference.metadata.is_empty());
    
    // Test with metadata
    let reference = reference.with_metadata("key1", "value1")
                             .with_metadata("key2", "value2");
    assert_eq!(reference.metadata.len(), 2);
    assert_eq!(reference.metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(reference.metadata.get("key2"), Some(&"value2".to_string()));
    
    // Test expiration
    assert!(!reference.is_expired());
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let expired_reference = reference.with_expiration(now - 100); // 100 seconds in the past
    assert!(expired_reference.is_expired());
    
    let future_reference = reference.with_expiration(now + 3600); // 1 hour in the future
    assert!(!future_reference.is_expired());
}

/// Test transfer operation
#[test]
fn test_transfer_operation() {
    // Create test data
    let domain1 = DomainId::new("domain1");
    let domain2 = DomainId::new("domain2");
    let resource_type = ResourceTypeId::new("document");
    let content_id = ContentId::from_bytes(&[1, 2, 3, 4]).unwrap();
    
    let resource_id = CrossDomainResourceId::new(
        content_id.clone(),
        domain1.clone(),
        resource_type.clone(),
    );
    
    let capability = BasicCapability::new("resource.transfer");
    
    // Create a transfer operation
    let operation = ResourceTransferOperation::new(
        resource_id.clone(),
        domain1.clone(),
        domain2.clone(),
        ResourceProjectionType::Transferred,
        VerificationLevel::Hash,
        capability.clone(),
    );
    
    // Test properties
    assert_eq!(operation.resource_id, resource_id);
    assert_eq!(operation.source_domain, domain1);
    assert_eq!(operation.target_domain, domain2);
    assert_eq!(operation.projection_type, ResourceProjectionType::Transferred);
    assert_eq!(operation.verification_level, VerificationLevel::Hash);
    assert_eq!(operation.authorization, capability);
    assert!(matches!(operation.status, TransferStatus::Pending));
    assert!(operation.resource_data.is_none());
    assert!(operation.metadata.is_empty());
    
    // Test with status
    let operation = operation.with_status(TransferStatus::InProgress(50.0));
    assert!(matches!(operation.status, TransferStatus::InProgress(p) if p == 50.0));
    
    // Test with data
    let data = vec![5, 6, 7, 8];
    let operation = operation.with_data(data.clone());
    assert_eq!(operation.resource_data, Some(data));
    
    // Test with metadata
    let operation = operation.with_metadata("key1", "value1")
                             .with_metadata("key2", "value2");
    assert_eq!(operation.metadata.len(), 2);
    assert_eq!(operation.metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(operation.metadata.get("key2"), Some(&"value2".to_string()));
} 