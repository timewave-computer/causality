// Name Registry module for Causality Content-Addressed Code System
//
// This module provides a registry for mapping between names and content hashes,
// allowing for human-readable references to content-addressed code.

use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Write};
use std::sync::{Arc, RwLock};

use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::effect_adapters::hash::Hash as ContentHash;

/// A record of a name-to-hash mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameRecord {
    /// The name
    pub name: String,
    /// The content hash
    pub hash: ContentHash,
    /// The version string
    pub version: String,
    /// The timestamp of this mapping
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl NameRecord {
    /// Create a new name record
    pub fn new(name: String, hash: ContentHash, version: String) -> Self {
        NameRecord {
            name,
            hash,
            version,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// A name registry for mapping between names and content hashes
#[derive(Debug)]
pub struct NameRegistry {
    /// The root directory for storage
    root_dir: PathBuf,
    /// The current name-to-hash mappings (name -> version -> hash)
    registry: Arc<RwLock<HashMap<String, BTreeMap<String, NameRecord>>>>,
}

impl NameRegistry {
    /// Create a new name registry
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        
        // Create directories if they don't exist
        fs::create_dir_all(&root_dir)?;
        
        // Load existing registry
        let registry = Self::load_registry(&root_dir)?;
        
        Ok(NameRegistry {
            root_dir,
            registry: Arc::new(RwLock::new(registry)),
        })
    }
    
    /// Register a name-to-hash mapping
    pub fn register(&self, name: &str, hash: ContentHash, version: &str) -> Result<()> {
        let record = NameRecord::new(name.to_string(), hash.clone(), version.to_string());
        
        // Update in-memory registry
        {
            let mut registry = self.registry.write().map_err(|_| Error::LockError)?;
            let versions = registry.entry(name.to_string()).or_insert_with(BTreeMap::new);
            versions.insert(version.to_string(), record.clone());
        }
        
        // Save to disk
        self.save_record(&record)?;
        
        Ok(())
    }
    
    /// Register a name-to-hash mapping as the latest version
    pub fn register_as_latest(&self, name: &str, version: &str, hash: ContentHash) -> Result<()> {
        // First, register the version
        self.register(name, hash.clone(), version)?;
        
        // Check if this should be the latest
        let should_update_latest = {
            let registry = self.registry.read().map_err(|_| Error::LockError)?;
            if let Some(versions) = registry.get(name) {
                if let Some(latest) = versions.iter().next_back() {
                    Self::is_version_greater(version, latest.0)
                } else {
                    true
                }
            } else {
                true
            }
        };
        
        if should_update_latest {
            // Create a symlink to the latest version
            self.save_latest(name, version)?;
        }
        
        Ok(())
    }
    
    /// Resolve a name to a hash
    pub fn resolve(&self, name: &str) -> Result<ContentHash> {
        // Check in-memory cache first
        {
            let registry = self.registry.read().map_err(|_| Error::LockError)?;
            if let Some(versions) = registry.get(name) {
                if let Some(latest) = versions.iter().next_back() {
                    return Ok(latest.1.hash.clone());
                }
            }
        }
        
        // Not found
        Err(Error::NameNotFound(name.to_string()))
    }
    
    /// Resolve a name and version to a hash
    pub fn resolve_version(&self, name: &str, version: &str) -> Result<ContentHash> {
        // Check in-memory cache
        {
            let registry = self.registry.read().map_err(|_| Error::LockError)?;
            if let Some(versions) = registry.get(name) {
                if let Some(record) = versions.get(version) {
                    return Ok(record.hash.clone());
                }
            }
        }
        
        // Not found
        Err(Error::VersionNotFound(name.to_string(), version.to_string()))
    }
    
    /// Get all versions for a name
    pub fn get_versions(&self, name: &str) -> Result<Vec<String>> {
        let registry = self.registry.read().map_err(|_| Error::LockError)?;
        if let Some(versions) = registry.get(name) {
            Ok(versions.keys().cloned().collect())
        } else {
            Err(Error::NameNotFound(name.to_string()))
        }
    }
    
    /// Check if a version string is greater than another
    fn is_version_greater(a: &str, b: &str) -> bool {
        // Simple semver-like comparison (without pre-release identifiers)
        let parse_version = |v: &str| -> Vec<u32> {
            v.split('.')
                .filter_map(|part| part.parse::<u32>().ok())
                .collect()
        };
        
        let a_parts = parse_version(a);
        let b_parts = parse_version(b);
        
        for (i, a_part) in a_parts.iter().enumerate() {
            if i >= b_parts.len() {
                return true; // a has more parts
            }
            if a_part > &b_parts[i] {
                return true;
            }
            if a_part < &b_parts[i] {
                return false;
            }
        }
        
        a_parts.len() > b_parts.len()
    }
    
    /// Save a name record to disk
    fn save_record(&self, record: &NameRecord) -> Result<()> {
        let name_dir = self.root_dir.join("names").join(&record.name);
        fs::create_dir_all(&name_dir)?;
        
        let version_file = name_dir.join(format!("{}.json", record.version));
        let content = serde_json::to_string_pretty(record)?;
        
        fs::write(version_file, content)?;
        
        Ok(())
    }
    
    /// Save a name as the latest version
    fn save_latest(&self, name: &str, version: &str) -> Result<()> {
        let name_dir = self.root_dir.join("names").join(name);
        let latest_file = name_dir.join("latest.json");
        
        let version_file = name_dir.join(format!("{}.json", version));
        if !version_file.exists() {
            return Err(Error::FileNotFound(version_file.to_string_lossy().to_string()));
        }
        
        // Read the content
        let content = fs::read_to_string(&version_file)?;
        
        // Write to latest
        fs::write(latest_file, content)?;
        
        Ok(())
    }
    
    /// Load the registry from disk
    fn load_registry(root_dir: &Path) -> Result<HashMap<String, BTreeMap<String, NameRecord>>> {
        let names_dir = root_dir.join("names");
        if !names_dir.exists() {
            return Ok(HashMap::new());
        }
        
        let mut registry = HashMap::new();
        
        // Iterate through name directories
        for entry in fs::read_dir(&names_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            
            let name = entry.file_name().to_string_lossy().to_string();
            let name_dir = entry.path();
            let mut versions = BTreeMap::new();
            
            // Read each version file
            for file_entry in fs::read_dir(&name_dir)? {
                let file_entry = file_entry?;
                if !file_entry.file_type()?.is_file() {
                    continue;
                }
                
                let file_name = file_entry.file_name().to_string_lossy().to_string();
                if file_name == "latest.json" {
                    continue; // Skip latest.json, we'll handle it separately
                }
                
                if let Some(version) = file_name.strip_suffix(".json") {
                    let content = fs::read_to_string(file_entry.path())?;
                    let record: NameRecord = serde_json::from_str(&content)?;
                    versions.insert(version.to_string(), record);
                }
            }
            
            if !versions.is_empty() {
                registry.insert(name, versions);
            }
        }
        
        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_registry_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let registry = NameRegistry::new(temp_dir.path())?;
        
        Ok(())
    }
    
    #[test]
    fn test_name_registration() -> Result<()> {
        let temp_dir = tempdir()?;
        let registry = NameRegistry::new(temp_dir.path())?;
        
        let hash = ContentHash::from_hex("000102030405060708090a0b0c0d0e0f")?;
        registry.register("test", hash.clone(), "1.0.0")?;
        
        let resolved = registry.resolve("test")?;
        assert_eq!(resolved, hash);
        
        Ok(())
    }
    
    #[test]
    fn test_version_comparison() {
        assert!(NameRegistry::is_version_greater("1.2.0", "1.1.0"));
        assert!(NameRegistry::is_version_greater("1.10.0", "1.2.0"));
        assert!(NameRegistry::is_version_greater("2.0.0", "1.9.9"));
        assert!(NameRegistry::is_version_greater("1.0.1", "1.0.0"));
        assert!(!NameRegistry::is_version_greater("1.0.0", "1.0.1"));
        assert!(!NameRegistry::is_version_greater("1.2.0", "1.2.0"));
    }
    
    #[test]
    fn test_versioning() -> Result<()> {
        let temp_dir = tempdir()?;
        let registry = NameRegistry::new(temp_dir.path())?;
        
        let hash1 = ContentHash::from_hex("000102030405060708090a0b0c0d0e0f")?;
        let hash2 = ContentHash::from_hex("0f0e0d0c0b0a09080706050403020100")?;
        
        // Register v1.0.0
        registry.register_as_latest("test", "1.0.0", hash1.clone())?;
        
        // Register v2.0.0
        registry.register_as_latest("test", "2.0.0", hash2.clone())?;
        
        // Latest should be v2.0.0
        let latest = registry.resolve("test")?;
        assert_eq!(latest, hash2);
        
        // We should be able to get both versions
        let versions = registry.get_versions("test")?;
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));
        
        // We should be able to resolve a specific version
        let v1 = registry.resolve_version("test", "1.0.0")?;
        assert_eq!(v1, hash1);
        
        let v2 = registry.resolve_version("test", "2.0.0")?;
        assert_eq!(v2, hash2);
        
        Ok(())
    }
} 