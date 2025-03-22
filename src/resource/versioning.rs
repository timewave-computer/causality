// Register versioning system
//
// This module implements versioning support for registers,
// including compatibility checks and migration mechanisms.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::resource::{Register, RegisterId, RegisterContents, RegisterState};
use crate::types::{ContentHash, Hash256};

/// Schema version for register contents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVersion {
    /// Major version (incompatible changes)
    pub major: u16,
    /// Minor version (backward compatible changes)
    pub minor: u16,
    /// Patch version (fixes without API changes)
    pub patch: u16,
    /// Schema identifier
    pub schema_id: String,
}

impl SchemaVersion {
    /// Create a new schema version
    pub fn new(major: u16, minor: u16, patch: u16, schema_id: &str) -> Self {
        SchemaVersion {
            major,
            minor,
            patch,
            schema_id: schema_id.to_string(),
        }
    }
    
    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        self.schema_id == other.schema_id && self.major == other.major
    }
    
    /// Check if this version is newer than another version
    pub fn is_newer_than(&self, other: &SchemaVersion) -> bool {
        if self.schema_id != other.schema_id {
            return false;
        }
        
        if self.major > other.major {
            return true;
        }
        
        if self.major == other.major && self.minor > other.minor {
            return true;
        }
        
        if self.major == other.major && self.minor == other.minor && self.patch > other.patch {
            return true;
        }
        
        false
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-v{}.{}.{}", self.schema_id, self.major, self.minor, self.patch)
    }
}

/// Register version migration function type
pub type MigrationFn = Arc<dyn Fn(&Register, &SchemaVersion) -> Result<Register> + Send + Sync>;

/// Version migration specification
#[derive(Clone)]
pub struct VersionMigration {
    /// From version
    pub from_version: SchemaVersion,
    /// To version
    pub to_version: SchemaVersion,
    /// Migration function
    pub migrate: MigrationFn,
}

/// Registry of version migrations
pub struct MigrationRegistry {
    /// Migrations by schema ID and version range
    migrations: HashMap<String, Vec<VersionMigration>>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        MigrationRegistry {
            migrations: HashMap::new(),
        }
    }
    
    /// Register a migration
    pub fn register_migration(&mut self, migration: VersionMigration) -> Result<()> {
        // Verify that the migration is for the same schema
        if migration.from_version.schema_id != migration.to_version.schema_id {
            return Err(Error::InvalidInput(
                "Migration must be for the same schema ID".to_string()
            ));
        }
        
        // Verify that the target version is newer
        if !migration.to_version.is_newer_than(&migration.from_version) {
            return Err(Error::InvalidInput(
                "Target version must be newer than source version".to_string()
            ));
        }
        
        // Get or create the migrations for this schema
        let schema_migrations = self.migrations
            .entry(migration.from_version.schema_id.clone())
            .or_insert_with(Vec::new);
        
        // Add the migration
        schema_migrations.push(migration);
        
        Ok(())
    }
    
    /// Find a migration path from one version to another
    pub fn find_migration_path(
        &self,
        from_version: &SchemaVersion,
        to_version: &SchemaVersion,
    ) -> Result<Vec<VersionMigration>> {
        // Check if versions are compatible
        if from_version.schema_id != to_version.schema_id {
            return Err(Error::InvalidInput(
                "Cannot migrate between different schemas".to_string()
            ));
        }
        
        // If versions are the same, return empty path
        if from_version == to_version {
            return Ok(Vec::new());
        }
        
        // If target version is older, return error
        if !to_version.is_newer_than(from_version) {
            return Err(Error::InvalidInput(
                "Cannot migrate to an older version".to_string()
            ));
        }
        
        // Get migrations for this schema
        let schema_migrations = match self.migrations.get(&from_version.schema_id) {
            Some(migrations) => migrations,
            None => return Err(Error::NotFound(format!(
                "No migrations registered for schema '{}'", from_version.schema_id
            ))),
        };
        
        // Find a path using BFS
        let mut queue = Vec::new();
        let mut visited = HashMap::new();
        
        // Start with the from_version
        queue.push(from_version.clone());
        visited.insert(from_version.clone(), Vec::new());
        
        while !queue.is_empty() {
            let current_version = queue.remove(0);
            let current_path = visited.get(&current_version).unwrap().clone();
            
            // Check if we've reached the target version
            if &current_version == to_version {
                return Ok(current_path);
            }
            
            // Find all migrations from current version
            for migration in schema_migrations.iter() {
                if migration.from_version == current_version {
                    let next_version = migration.to_version.clone();
                    
                    // If we haven't visited this version yet
                    if !visited.contains_key(&next_version) {
                        let mut new_path = current_path.clone();
                        new_path.push(migration.clone());
                        
                        // Add to queue and mark as visited
                        queue.push(next_version.clone());
                        visited.insert(next_version, new_path);
                    }
                }
            }
        }
        
        // If we get here, no path was found
        Err(Error::InvalidInput(format!(
            "No migration path found from {} to {}", from_version, to_version
        )))
    }
    
    /// Migrate a register from one version to another
    pub fn migrate_register(
        &self,
        register: &Register,
        to_version: &SchemaVersion,
    ) -> Result<Register> {
        // Get the current version
        let from_version = match register.metadata.get("schema_version") {
            Some(version_str) => {
                // Parse version string (format: schema_id-vX.Y.Z)
                let parts: Vec<&str> = version_str.split('-').collect();
                if parts.len() != 2 || !parts[1].starts_with('v') {
                    return Err(Error::InvalidInput(format!(
                        "Invalid version format: {}", version_str
                    )));
                }
                
                let schema_id = parts[0].to_string();
                let version_parts: Vec<&str> = parts[1][1..].split('.').collect();
                if version_parts.len() != 3 {
                    return Err(Error::InvalidInput(format!(
                        "Invalid version number format: {}", parts[1]
                    )));
                }
                
                let major = version_parts[0].parse::<u16>().map_err(|_| 
                    Error::InvalidInput(format!("Invalid major version: {}", version_parts[0]))
                )?;
                
                let minor = version_parts[1].parse::<u16>().map_err(|_| 
                    Error::InvalidInput(format!("Invalid minor version: {}", version_parts[1]))
                )?;
                
                let patch = version_parts[2].parse::<u16>().map_err(|_| 
                    Error::InvalidInput(format!("Invalid patch version: {}", version_parts[2]))
                )?;
                
                SchemaVersion::new(major, minor, patch, &schema_id)
            },
            None => return Err(Error::InvalidInput(
                "Register does not have a schema version".to_string()
            )),
        };
        
        // Find migration path
        let path = self.find_migration_path(&from_version, to_version)?;
        
        // If the path is empty, return the original register
        if path.is_empty() {
            return Ok(register.clone());
        }
        
        // Apply migrations sequentially
        let mut current_register = register.clone();
        
        for migration in path {
            current_register = (migration.migrate)(&current_register, &migration.to_version)?;
            
            // Update the schema version in metadata
            let mut new_metadata = current_register.metadata.clone();
            new_metadata.insert(
                "schema_version".to_string(),
                migration.to_version.to_string(),
            );
            current_register.metadata = new_metadata;
        }
        
        Ok(current_register)
    }
}

/// Thread-safe migration registry
pub struct SharedMigrationRegistry {
    /// Inner migration registry
    inner: Arc<Mutex<MigrationRegistry>>,
}

impl SharedMigrationRegistry {
    /// Create a new shared migration registry
    pub fn new() -> Self {
        SharedMigrationRegistry {
            inner: Arc::new(Mutex::new(MigrationRegistry::new())),
        }
    }
    
    /// Register a migration
    pub fn register_migration(&self, migration: VersionMigration) -> Result<()> {
        let mut registry = self.inner.lock().map_err(|_| 
            Error::LockError("Failed to acquire migration registry lock".to_string())
        )?;
        
        registry.register_migration(migration)
    }
    
    /// Migrate a register from its current version to the target version
    pub fn migrate_register(
        &self,
        register: &Register,
        to_version: &SchemaVersion,
    ) -> Result<Register> {
        let registry = self.inner.lock().map_err(|_| 
            Error::LockError("Failed to acquire migration registry lock".to_string())
        )?;
        
        registry.migrate_register(register, to_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_version_compatibility() {
        let v1 = SchemaVersion::new(1, 0, 0, "test");
        let v1_1 = SchemaVersion::new(1, 1, 0, "test");
        let v2 = SchemaVersion::new(2, 0, 0, "test");
        let other_v1 = SchemaVersion::new(1, 0, 0, "other");
        
        assert!(v1.is_compatible_with(&v1_1));
        assert!(!v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&other_v1));
    }
    
    #[test]
    fn test_schema_version_newer() {
        let v1 = SchemaVersion::new(1, 0, 0, "test");
        let v1_1 = SchemaVersion::new(1, 1, 0, "test");
        let v1_1_1 = SchemaVersion::new(1, 1, 1, "test");
        let v2 = SchemaVersion::new(2, 0, 0, "test");
        let other_v2 = SchemaVersion::new(2, 0, 0, "other");
        
        assert!(v1_1.is_newer_than(&v1));
        assert!(v1_1_1.is_newer_than(&v1_1));
        assert!(v2.is_newer_than(&v1_1_1));
        assert!(!other_v2.is_newer_than(&v1)); // Different schema
        assert!(!v1.is_newer_than(&v1)); // Same version
    }
    
    #[test]
    fn test_register_migration() {
        // Create a migration registry
        let mut registry = MigrationRegistry::new();
        
        // Define versions
        let v1 = SchemaVersion::new(1, 0, 0, "test");
        let v1_1 = SchemaVersion::new(1, 1, 0, "test");
        let v2 = SchemaVersion::new(2, 0, 0, "test");
        
        // Create migration functions
        let migrate_v1_to_v1_1: MigrationFn = Arc::new(|register, _| {
            let mut new_register = register.clone();
            let mut contents = new_register.contents.clone();
            contents.update_value(format!("{} - updated to v1.1", contents.as_string()));
            new_register.contents = contents;
            Ok(new_register)
        });
        
        let migrate_v1_1_to_v2: MigrationFn = Arc::new(|register, _| {
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
        }).unwrap();
        
        registry.register_migration(VersionMigration {
            from_version: v1_1.clone(),
            to_version: v2.clone(),
            migrate: migrate_v1_1_to_v2,
        }).unwrap();
        
        // Create a test register
        let mut register = Register {
            register_id: RegisterId::new("test_register"),
            state: RegisterState::Active,
            owner: crate::types::Address::new("owner"),
            domain: crate::types::Domain::new("test_domain"),
            contents: RegisterContents::with_string("test_contents"),
            created_at: 100,
            updated_at: 100,
            version: 1,
            metadata: HashMap::new(),
            archive_reference: None,
            summarizes: Vec::new(),
            summarized_by: None,
            successors: Vec::new(),
            predecessors: Vec::new(),
        };
        
        // Add version to metadata
        register.metadata.insert("schema_version".to_string(), v1.to_string());
        
        // Migrate to v1.1
        let migrated_v1_1 = registry.migrate_register(&register, &v1_1).unwrap();
        assert_eq!(
            migrated_v1_1.metadata.get("schema_version").unwrap(),
            &v1_1.to_string()
        );
        assert_eq!(
            migrated_v1_1.contents.as_string(),
            "test_contents - updated to v1.1"
        );
        
        // Migrate directly to v2
        let migrated_v2 = registry.migrate_register(&register, &v2).unwrap();
        assert_eq!(
            migrated_v2.metadata.get("schema_version").unwrap(),
            &v2.to_string()
        );
        assert_eq!(
            migrated_v2.contents.as_string(),
            "test_contents - updated to v1.1 - upgraded to v2"
        );
    }
    
    #[test]
    fn test_migration_errors() {
        // Create a migration registry
        let mut registry = MigrationRegistry::new();
        
        // Define versions
        let v1 = SchemaVersion::new(1, 0, 0, "test");
        let v1_1 = SchemaVersion::new(1, 1, 0, "test");
        let v2 = SchemaVersion::new(2, 0, 0, "test");
        let other_v1 = SchemaVersion::new(1, 0, 0, "other");
        
        // Create migration function
        let migrate_v1_to_v1_1: MigrationFn = Arc::new(|register, _| {
            Ok(register.clone())
        });
        
        // Register migration
        registry.register_migration(VersionMigration {
            from_version: v1.clone(),
            to_version: v1_1.clone(),
            migrate: migrate_v1_to_v1_1,
        }).unwrap();
        
        // Try to register invalid migrations
        
        // Different schema IDs
        let result = registry.register_migration(VersionMigration {
            from_version: v1.clone(),
            to_version: other_v1.clone(),
            migrate: Arc::new(|r, _| Ok(r.clone())),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("same schema ID"));
        
        // Older version
        let result = registry.register_migration(VersionMigration {
            from_version: v1_1.clone(),
            to_version: v1.clone(),
            migrate: Arc::new(|r, _| Ok(r.clone())),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("newer than source"));
        
        // Find paths
        
        // No path to v2
        let result = registry.find_migration_path(&v1, &v2);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No migration path"));
        
        // Cannot migrate between schemas
        let result = registry.find_migration_path(&v1, &other_v1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("different schemas"));
    }
} 