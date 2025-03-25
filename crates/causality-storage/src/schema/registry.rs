// Schema registry implementation
// Original file: src/schema/registry.rs

//! Schema Migration Registry
//!
//! This module provides a registry for user-defined migrations
//! that can be looked up based on schema name and version.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::schema::{
    Error, Result,
    Schema, SchemaVersion,
    migration::{MigrationHandler, MigrationFn},
};

/// A registry for schema migrations
pub struct MigrationRegistry {
    /// Map of schema name to version map to handlers
    handlers: RwLock<HashMap<String, HashMap<String, Vec<MigrationHandler>>>>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        MigrationRegistry {
            handlers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a migration handler
    pub fn register(&self, handler: MigrationHandler) -> Result<()> {
        let mut handlers = self.handlers.write().map_err(|_| {
            Error::Migration("Failed to acquire write lock on migration registry".to_string())
        })?;
        
        // Get or create the version map for this schema
        let version_map = handlers
            .entry(handler.source_schema.clone())
            .or_insert_with(HashMap::new);
        
        // Get or create the handlers for this version
        let version_handlers = version_map
            .entry(handler.source_version.clone())
            .or_insert_with(Vec::new);
        
        // Add the handler
        version_handlers.push(handler);
        
        Ok(())
    }
    
    /// Register a migration function
    pub fn register_fn(
        &self,
        source_schema: impl Into<String>,
        source_version: impl Into<String>,
        target_schema: impl Into<String>,
        target_version: impl Into<String>,
        migration_fn: MigrationFn,
    ) -> Result<()> {
        let handler = MigrationHandler::new(
            source_schema,
            source_version,
            target_schema,
            target_version,
            migration_fn,
        );
        
        self.register(handler)
    }
    
    /// Find a handler for migrating between schemas
    pub fn find_handler(&self, source: &Schema, target: &Schema) -> Result<Option<MigrationHandler>> {
        let handlers = self.handlers.read().map_err(|_| {
            Error::Migration("Failed to acquire read lock on migration registry".to_string())
        })?;
        
        // Look up by schema name
        if let Some(version_map) = handlers.get(&source.name) {
            // Look up by exact version
            if let Some(version_handlers) = version_map.get(source.version.as_str()) {
                // Look for a matching target
                for handler in version_handlers {
                    if handler.target_schema == target.name && 
                       handler.target_version == target.version.as_str() {
                        return Ok(Some(handler.clone()));
                    }
                }
            }
            
            // If no exact version match, try to find a compatible version
            // This is a more advanced matching strategy that could be implemented
            // to find migrations between compatible versions
        }
        
        Ok(None)
    }
    
    /// List all registered migrations
    pub fn list_migrations(&self) -> Result<Vec<(String, String, String, String)>> {
        let handlers = self.handlers.read().map_err(|_| {
            Error::Migration("Failed to acquire read lock on migration registry".to_string())
        })?;
        
        let mut result = Vec::new();
        
        for (source_schema, version_map) in handlers.iter() {
            for (source_version, version_handlers) in version_map.iter() {
                for handler in version_handlers {
                    result.push((
                        source_schema.clone(),
                        source_version.clone(),
                        handler.target_schema.clone(),
                        handler.target_version.clone(),
                    ));
                }
            }
        }
        
        Ok(result)
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A shared migration registry that can be used throughout the application
pub struct SharedMigrationRegistry {
    /// The shared registry
    registry: Arc<MigrationRegistry>,
}

impl SharedMigrationRegistry {
    /// Create a new shared migration registry
    pub fn new() -> Self {
        SharedMigrationRegistry {
            registry: Arc::new(MigrationRegistry::new()),
        }
    }
    
    /// Get a reference to the registry
    pub fn registry(&self) -> Arc<MigrationRegistry> {
        self.registry.clone()
    }
    
    /// Register a migration handler
    pub fn register(&self, handler: MigrationHandler) -> Result<()> {
        self.registry.register(handler)
    }
    
    /// Register a migration function
    pub fn register_fn(
        &self,
        source_schema: impl Into<String>,
        source_version: impl Into<String>,
        target_schema: impl Into<String>,
        target_version: impl Into<String>,
        migration_fn: MigrationFn,
    ) -> Result<()> {
        self.registry.register_fn(
            source_schema,
            source_version,
            target_schema,
            target_version,
            migration_fn,
        )
    }
    
    /// Find a handler for migrating between schemas
    pub fn find_handler(&self, source: &Schema, target: &Schema) -> Result<Option<MigrationHandler>> {
        self.registry.find_handler(source, target)
    }
}

impl Default for SharedMigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new shared migration registry with common migrations pre-registered
pub fn create_migration_registry() -> SharedMigrationRegistry {
    let registry = SharedMigrationRegistry::new();
    
    // Register common migrations here
    // For example, simple type conversions or field renamings
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    
    #[test]
    fn test_register_and_find_handler() -> Result<()> {
        // Create a registry
        let registry = MigrationRegistry::new();
        
        // Define a migration function
        fn migrate_fn(_value: Value, _source: &Schema, _target: &Schema) -> Result<Value> {
            Ok(json!({"migrated": true}))
        }
        
        // Register a handler
        registry.register_fn(
            "TestSchema", "1.0.0",
            "TestSchema", "2.0.0",
            migrate_fn,
        )?;
        
        // Create source and target schemas
        let source = Schema::new("TestSchema", "1.0.0")?;
        let target = Schema::new("TestSchema", "2.0.0")?;
        
        // Find the handler
        let handler = registry.find_handler(&source, &target)?;
        
        // Verify that a handler was found
        assert!(handler.is_some());
        let handler = handler.unwrap();
        
        // Verify handler properties
        assert_eq!(handler.source_schema, "TestSchema");
        assert_eq!(handler.source_version, "1.0.0");
        assert_eq!(handler.target_schema, "TestSchema");
        assert_eq!(handler.target_version, "2.0.0");
        
        // Try with a non-existent migration
        let other_source = Schema::new("OtherSchema", "1.0.0")?;
        let other_handler = registry.find_handler(&other_source, &target)?;
        
        // Verify that no handler was found
        assert!(other_handler.is_none());
        
        Ok(())
    }
    
    #[test]
    fn test_shared_registry() -> Result<()> {
        // Create a shared registry
        let shared = SharedMigrationRegistry::new();
        
        // Define a migration function
        fn migrate_fn(_value: Value, _source: &Schema, _target: &Schema) -> Result<Value> {
            Ok(json!({"migrated": true}))
        }
        
        // Register a migration
        shared.register_fn(
            "TestSchema", "1.0.0",
            "TestSchema", "2.0.0",
            migrate_fn,
        )?;
        
        // Create source and target schemas
        let source = Schema::new("TestSchema", "1.0.0")?;
        let target = Schema::new("TestSchema", "2.0.0")?;
        
        // Get another reference to the registry
        let registry2 = shared.registry();
        
        // Find the handler through the second reference
        let handler = registry2.find_handler(&source, &target)?;
        
        // Verify that a handler was found
        assert!(handler.is_some());
        
        Ok(())
    }
} 