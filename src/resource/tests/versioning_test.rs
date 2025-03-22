// Tests for register versioning

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterState,
    OneTimeRegisterSystem, OneTimeRegisterConfig,
    SchemaVersion, VersionMigration, MigrationRegistry,
};
use crate::types::{Address, Domain};

#[test]
fn test_register_with_version() -> Result<()> {
    // Create a register system
    let config = OneTimeRegisterConfig {
        current_block_height: 100,
        nullifier_timeout: 20,
        initial_observers: Vec::new(),
        migration_registry: None,
    };
    
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create a version
    let version = SchemaVersion::new(1, 0, 0, "test-schema");
    
    // Create a register with a version
    let register = system.create_register_with_version(
        Address::new("owner"),
        Domain::new("test-domain"),
        RegisterContents::with_string("test contents"),
        "tx-1",
        &version,
    );
    
    // Verify that the version is set
    let schema_version = system.get_register_schema_version(&register)?;
    assert_eq!(schema_version.major, 1);
    assert_eq!(schema_version.minor, 0);
    assert_eq!(schema_version.patch, 0);
    assert_eq!(schema_version.schema_id, "test-schema");
    
    Ok(())
}

#[test]
fn test_register_migration() -> Result<()> {
    // Create a migration registry
    let mut registry = MigrationRegistry::new();
    
    // Define versions
    let v1 = SchemaVersion::new(1, 0, 0, "test-schema");
    let v1_1 = SchemaVersion::new(1, 1, 0, "test-schema");
    let v2 = SchemaVersion::new(2, 0, 0, "test-schema");
    
    // Create migration functions
    let migrate_v1_to_v1_1 = Arc::new(|register: &Register, _| {
        let mut new_register = register.clone();
        let mut contents = new_register.contents.clone();
        contents.update_value(format!("{} - updated to v1.1", contents.as_string()));
        new_register.contents = contents;
        Ok(new_register)
    });
    
    let migrate_v1_1_to_v2 = Arc::new(|register: &Register, _| {
        let mut new_register = register.clone();
        let mut contents = new_register.contents.clone();
        contents.update_value(format!("{} - upgraded to v2", contents.as_string()));
        new_register.contents = contents;
        Ok(new_register)
    });
    
    // Register migrations
    registry.register_migration(VersionMigration {
        from_version: v1.clone(),
        to_version: v1_1.clone(),
        migrate: migrate_v1_to_v1_1,
    })?;
    
    registry.register_migration(VersionMigration {
        from_version: v1_1.clone(),
        to_version: v2.clone(),
        migrate: migrate_v1_1_to_v2,
    })?;
    
    // Create the register system with the migration registry
    let shared_registry = Arc::new(std::sync::Mutex::new(registry));
    let config = OneTimeRegisterConfig {
        current_block_height: 100,
        nullifier_timeout: 20,
        initial_observers: Vec::new(),
        migration_registry: Some(shared_registry),
    };
    
    let system = OneTimeRegisterSystem::new(config)?;
    
    // Create a register with version 1.0.0
    let mut register = system.create_register_with_version(
        Address::new("owner"),
        Domain::new("test-domain"),
        RegisterContents::with_string("test contents"),
        "tx-1",
        &v1,
    );
    
    // Migrate to version 1.1.0
    system.migrate_register_version(&mut register, &v1_1)?;
    
    // Verify that the version is updated
    let schema_version = system.get_register_schema_version(&register)?;
    assert_eq!(schema_version.major, 1);
    assert_eq!(schema_version.minor, 1);
    assert_eq!(schema_version.patch, 0);
    
    // Verify that the content was updated
    assert_eq!(register.contents.as_string(), "test contents - updated to v1.1");
    
    // Check that the version counter was incremented
    assert_eq!(register.version, 2);
    
    // Migrate to version 2.0.0
    system.migrate_register_version(&mut register, &v2)?;
    
    // Verify that the version is updated
    let schema_version = system.get_register_schema_version(&register)?;
    assert_eq!(schema_version.major, 2);
    assert_eq!(schema_version.minor, 0);
    assert_eq!(schema_version.patch, 0);
    
    // Verify that the content was updated
    assert_eq!(
        register.contents.as_string(),
        "test contents - updated to v1.1 - upgraded to v2"
    );
    
    // Check that the version counter was incremented again
    assert_eq!(register.version, 3);
    
    Ok(())
}

#[test]
fn test_version_compatibility() -> Result<()> {
    // Create versions
    let v1 = SchemaVersion::new(1, 0, 0, "test-schema");
    let v1_1 = SchemaVersion::new(1, 1, 0, "test-schema");
    let v2 = SchemaVersion::new(2, 0, 0, "test-schema");
    let other_v1 = SchemaVersion::new(1, 0, 0, "other-schema");
    
    // Test compatibility
    assert!(v1.is_compatible_with(&v1_1), "v1 should be compatible with v1.1");
    assert!(!v1.is_compatible_with(&v2), "v1 should not be compatible with v2");
    assert!(!v1.is_compatible_with(&other_v1), "v1 should not be compatible with other-schema");
    
    // Test newer than
    assert!(v1_1.is_newer_than(&v1), "v1.1 should be newer than v1");
    assert!(v2.is_newer_than(&v1_1), "v2 should be newer than v1.1");
    assert!(!v1.is_newer_than(&v1_1), "v1 should not be newer than v1.1");
    assert!(!v1.is_newer_than(&other_v1), "v1 should not be newer than other-schema");
    
    Ok(())
} 