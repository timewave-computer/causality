use std::collections::HashMap;
use std::sync::Arc;

use crate::content::ContentId;
use crate::resource::{
    ResourceTypeId,
    ResourceSchema,
    ResourceTypeDefinition,
    ResourceTypeRegistry,
    InMemoryResourceTypeRegistry,
    ContentAddressedResourceTypeRegistry,
};
use crate::storage::InMemoryContentAddressedStorage;

// Helper to create a test schema
fn create_test_schema() -> ResourceSchema {
    ResourceSchema {
        format: "json-schema".to_string(),
        definition: r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#.to_string(),
        version: "1.0".to_string(),
        content_hash: None,
    }
}

// Helper to create a test resource type definition
fn create_test_resource_type(name: &str, namespace: &str, version: &str) -> ResourceTypeDefinition {
    ResourceTypeDefinition {
        id: ResourceTypeId::with_version(namespace, name, version),
        schema: create_test_schema(),
        description: Some(format!("Test resource type {}", name)),
        documentation: None,
        deprecated: false,
        compatible_with: Vec::new(),
        required_capabilities: HashMap::new(),
        created_at: 12345,
        updated_at: 12345,
    }
}

#[tokio::test]
async fn test_resource_type_id() {
    // Basic creation
    let id1 = ResourceTypeId::new("user");
    assert_eq!(id1.name(), "user");
    assert_eq!(id1.namespace(), None);
    assert_eq!(id1.version(), None);
    assert_eq!(id1.qualified_name(), "user");

    // With namespace
    let id2 = ResourceTypeId::with_namespace("core", "user");
    assert_eq!(id2.name(), "user");
    assert_eq!(id2.namespace(), Some("core"));
    assert_eq!(id2.version(), None);
    assert_eq!(id2.qualified_name(), "core:user");

    // With version
    let id3 = ResourceTypeId::with_version("core", "user", "1.0");
    assert_eq!(id3.name(), "user");
    assert_eq!(id3.namespace(), Some("core"));
    assert_eq!(id3.version(), Some("1.0"));
    assert_eq!(id3.qualified_name(), "core:user:1.0");

    // Version parsing
    assert_eq!(id3.major_version(), Some(1));
    assert_eq!(id3.minor_version(), Some(0));

    // New version
    let id4 = id3.with_new_version("2.0");
    assert_eq!(id4.version(), Some("2.0"));
    assert_eq!(id4.major_version(), Some(2));
}

#[tokio::test]
async fn test_resource_type_compatibility() {
    let type1 = ResourceTypeId::with_version("test", "user", "1.0");
    let type2 = ResourceTypeId::with_version("test", "user", "1.1");
    let type3 = ResourceTypeId::with_version("test", "user", "2.0");
    let type4 = ResourceTypeId::with_version("test", "profile", "1.0");
    let type5 = ResourceTypeId::with_version("other", "user", "1.0");

    // Same name, namespace
    assert!(type1.is_compatible_with(&type1)); // Self-compatibility
    assert!(type1.is_compatible_with(&type2)); // Minor version difference

    // Different major version
    assert!(!type1.is_compatible_with(&type3));

    // Different name
    assert!(!type1.is_compatible_with(&type4));

    // Different namespace
    assert!(!type1.is_compatible_with(&type5));
}

#[tokio::test]
async fn test_in_memory_registry() {
    let registry = InMemoryResourceTypeRegistry::new();

    // Create test resource types
    let type1 = create_test_resource_type("user", "test", "1.0");
    let type2 = create_test_resource_type("user", "test", "1.1");
    let type3 = create_test_resource_type("profile", "test", "1.0");

    // Register resource types
    let id1 = registry.register_resource_type(type1.clone()).await.unwrap();
    let id2 = registry.register_resource_type(type2.clone()).await.unwrap();
    let id3 = registry.register_resource_type(type3.clone()).await.unwrap();

    // Verify resource types exist
    assert!(registry.has_resource_type(&id1).await.unwrap());
    assert!(registry.has_resource_type(&id2).await.unwrap());
    assert!(registry.has_resource_type(&id3).await.unwrap());

    // Get resource types
    let retrieved1 = registry.get_resource_type(&id1).await.unwrap();
    let retrieved2 = registry.get_resource_type(&id2).await.unwrap();

    // Verify properties
    assert_eq!(retrieved1.id, id1);
    assert_eq!(retrieved2.id, id2);
    assert_eq!(retrieved1.description, Some("Test resource type user".to_string()));

    // Get latest version
    let latest = registry.get_latest_version("user", Some("test")).await.unwrap();
    assert_eq!(latest, id2); // 1.1 is newer than 1.0

    // Get all versions
    let versions = registry.get_all_versions("user", Some("test")).await.unwrap();
    assert_eq!(versions.len(), 2);
    assert!(versions.contains(&id1));
    assert!(versions.contains(&id2));
}

#[tokio::test]
async fn test_content_addressed_registry() {
    let storage = Arc::new(InMemoryContentAddressedStorage::new());
    let registry = ContentAddressedResourceTypeRegistry::new(storage.clone());

    // Create test resource type
    let type_def = create_test_resource_type("document", "content", "1.0");
    
    // Register resource type
    let type_id = registry.register_resource_type(type_def.clone()).await.unwrap();
    
    // Verify content hash is set
    assert!(type_id.content_hash().is_some());
    
    // Retrieve by ID
    let retrieved = registry.get_resource_type(&type_id).await.unwrap();
    assert_eq!(retrieved.id, type_id);
    assert_eq!(retrieved.description, Some("Test resource type document".to_string()));
    
    // Verify schema content hash is set
    assert!(retrieved.schema.content_hash.is_some());
} 