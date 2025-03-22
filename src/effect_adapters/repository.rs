// Repository module for Causality Effect Adapters
//
// This module provides a content-addressed code repository for
// storing and retrieving code objects.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Write};
use std::sync::{Arc, RwLock};
use std::str::FromStr;

use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::effect_adapters::hash::{Hash, HashAlgorithm, ContentHasher, HasherFactory};
use crate::effect_adapters::riscv_metadata::RiscVMetadata;
use crate::effect::types::EffectType;

/// Metadata for a code entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    /// The name of the code, if any
    pub name: Option<String>,
    /// The description of the code, if any
    pub description: Option<String>,
    /// The format of the code (e.g., "risc-v", "json")
    pub format: String,
    /// The version of the code
    pub version: String,
    /// The required version of the executor
    pub required_version: Option<String>,
    /// The required effects for this code
    #[cfg(feature = "full-effect")]
    pub required_effects: Option<HashSet<EffectType>>,
    #[cfg(not(feature = "full-effect"))]
    pub required_effects: Option<HashSet<String>>,  // Use String instead when full-effect is disabled
    /// The dependencies of this code
    pub dependencies: Option<HashMap<String, String>>,
    /// The RISC-V compatibility metadata, if applicable
    pub riscv_metadata: Option<RiscVMetadata>,
    /// Additional metadata as key-value pairs
    pub additional: HashMap<String, String>,
}

/// Builder for CodeMetadata
pub struct CodeMetadataBuilder {
    name: Option<String>,
    description: Option<String>,
    format: String,
    version: String,
    required_version: Option<String>,
    #[cfg(feature = "full-effect")]
    required_effects: Option<HashSet<EffectType>>,
    #[cfg(not(feature = "full-effect"))]
    required_effects: Option<HashSet<String>>,  // Use String instead when full-effect is disabled
    dependencies: Option<HashMap<String, String>>,
    riscv_metadata: Option<RiscVMetadata>,
    additional: HashMap<String, String>,
}

impl CodeMetadataBuilder {
    /// Create a new CodeMetadataBuilder
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            format: "json".to_string(), // Default format
            version: "0.1.0".to_string(), // Default version
            required_version: None,
            required_effects: None,
            dependencies: None,
            riscv_metadata: None,
            additional: HashMap::new(),
        }
    }

    /// Set the name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set the description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set the format
    pub fn with_format(mut self, format: &str) -> Self {
        self.format = format.to_string();
        self
    }
    
    /// Set the version
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }
    
    /// Set the required version
    pub fn with_required_version(mut self, required_version: &str) -> Self {
        self.required_version = Some(required_version.to_string());
        self
    }
    
    /// Set the required effects
    #[cfg(feature = "full-effect")]
    pub fn with_required_effects(mut self, required_effects: HashSet<EffectType>) -> Self {
        self.required_effects = Some(required_effects);
        self
    }
    
    /// Set the required effects (string version for minimal build)
    #[cfg(not(feature = "full-effect"))]
    pub fn with_required_effects(mut self, required_effects: HashSet<String>) -> Self {
        self.required_effects = Some(required_effects);
        self
    }
    
    /// Set the dependencies
    pub fn with_dependencies(mut self, dependencies: HashMap<String, String>) -> Self {
        self.dependencies = Some(dependencies);
        self
    }
    
    /// Set the RISC-V metadata
    pub fn with_riscv_metadata(mut self, metadata: Option<RiscVMetadata>) -> Self {
        self.riscv_metadata = metadata;
        self
    }
    
    /// Add a custom metadata field
    pub fn with_additional_field(mut self, key: &str, value: &str) -> Self {
        self.additional.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Build the CodeMetadata
    pub fn build(self) -> CodeMetadata {
        CodeMetadata {
            name: self.name,
            description: self.description,
            format: self.format,
            version: self.version,
            required_version: self.required_version,
            required_effects: self.required_effects,
            dependencies: self.dependencies,
            riscv_metadata: self.riscv_metadata,
            additional: self.additional,
        }
    }
}

impl CodeMetadata {
    /// Create a new metadata instance with minimum required fields
    pub fn builder(format: &str) -> CodeMetadataBuilder {
        CodeMetadataBuilder::new()
            .with_format(format)
            .with_version("1.0.0")
    }
}

/// A code entry in the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntry {
    /// The content hash
    pub hash: Hash,
    /// The metadata
    pub metadata: CodeMetadata,
    /// The serialized content
    #[serde(skip)]
    pub content: Option<Vec<u8>>,
}

impl CodeEntry {
    /// Create a new code entry
    pub fn new(hash: Hash, metadata: CodeMetadata, content: Option<Vec<u8>>) -> Self {
        CodeEntry {
            hash,
            metadata,
            content,
        }
    }
    
    /// Check if this code entry is compatible with RISC-V
    pub fn is_risc_v_compatible(&self) -> bool {
        self.metadata.riscv_metadata.is_some()
    }
}

/// Repository trait for content-addressed code
pub trait CodeRepository: Send + Sync {
    /// Store an object in the repository
    fn store<T: Serialize + Debug>(&self, object: &T, metadata: CodeMetadata) -> Result<Hash>;

    /// Store raw bytes in the repository
    fn store_bytes(&self, bytes: &[u8], metadata: CodeMetadata) -> Result<Hash>;

    /// Load an object from the repository
    fn load<T: for<'de> Deserialize<'de>>(&self, hash: &Hash) -> Result<T>;

    /// Load raw bytes from the repository
    fn load_bytes(&self, hash: &Hash) -> Result<Vec<u8>>;

    /// Register a name for a hash
    fn register_name(&self, name: &str, hash: Hash) -> Result<()>;

    /// Resolve a name to a hash
    fn resolve_name(&self, name: &str) -> Result<Hash>;

    /// Check if a hash exists in the repository
    fn exists(&self, hash: &Hash) -> bool;

    /// Get metadata for a hash
    fn get_metadata(&self, hash: &Hash) -> Result<CodeMetadata>;

    /// Check if a hash is compatible with RISC-V
    fn is_risc_v_compatible(&self, hash: &Hash) -> Result<bool>;

    /// Get all versions of a named object
    fn get_versions(&self, name: &str) -> Result<Vec<String>>;
}

/// Implementation of CodeRepository
#[derive(Debug, Clone)]
pub struct FileSystemCodeRepository {
    /// The root directory for storage
    root_dir: PathBuf,
    /// The hasher factory
    hasher_factory: HasherFactory,
    /// The default hash algorithm
    default_algorithm: HashAlgorithm,
    /// In-memory cache of entries for quick lookup
    entry_cache: Arc<RwLock<HashMap<Hash, CodeEntry>>>,
    /// In-memory name registry for resolving names to hashes
    name_registry: Arc<RwLock<HashMap<String, Hash>>>,
}

impl FileSystemCodeRepository {
    /// Create a new code repository
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        
        // Create directories if they don't exist
        fs::create_dir_all(&root_dir)?;
        fs::create_dir_all(root_dir.join("objects"))?;
        fs::create_dir_all(root_dir.join("metadata"))?;
        fs::create_dir_all(root_dir.join("names"))?;
        
        // Initialize the registry
        let name_registry = Self::load_name_registry(&root_dir)?;
        
        Ok(FileSystemCodeRepository {
            root_dir,
            hasher_factory: HasherFactory::default(),
            default_algorithm: HashAlgorithm::Blake3,
            entry_cache: Arc::new(RwLock::new(HashMap::new())),
            name_registry: Arc::new(RwLock::new(name_registry)),
        })
    }
    
    /// Get the path to the objects directory
    fn objects_dir(&self) -> PathBuf {
        self.root_dir.join("objects")
    }
    
    /// Get the path to the metadata directory
    fn metadata_dir(&self) -> PathBuf {
        self.root_dir.join("metadata")
    }
    
    /// Get the path to the names directory
    fn names_dir(&self) -> PathBuf {
        self.root_dir.join("names")
    }
    
    /// Get the path to an object file based on its hash
    fn object_path(&self, hash: &Hash) -> PathBuf {
        let hash_str = hash.to_string();
        let prefix = &hash_str[..2];
        let suffix = &hash_str[2..];
        self.objects_dir().join(prefix).join(suffix)
    }
    
    /// Get the path to a metadata file based on its hash
    fn metadata_path(&self, hash: &Hash) -> PathBuf {
        let hash_str = hash.to_string();
        let prefix = &hash_str[..2];
        let suffix = &hash_str[2..];
        self.metadata_dir().join(prefix).join(suffix).with_extension("json")
    }
    
    /// Store an entry in the repository
    fn store_entry(&self, entry: CodeEntry) -> Result<()> {
        let hash = entry.hash.clone();
        
        // Check if the entry already exists
        if self.exists(&hash) {
            // Update cache and return early
            let mut cache = self.entry_cache.write().map_err(|_| Error::LockError)?;
            cache.insert(hash, entry);
            return Ok(());
        }
        
        // Create directories if needed
        let object_path = self.object_path(&hash);
        let metadata_path = self.metadata_path(&hash);
        
        fs::create_dir_all(object_path.parent().unwrap())?;
        fs::create_dir_all(metadata_path.parent().unwrap())?;
        
        // Write content to file
        if let Some(content) = &entry.content {
            let mut file = fs::File::create(object_path)?;
            file.write_all(content)?;
        }
        
        // Write metadata to file
        let metadata_json = serde_json::to_string_pretty(&entry.metadata)?;
        let mut metadata_file = fs::File::create(metadata_path)?;
        metadata_file.write_all(metadata_json.as_bytes())?;
        
        // Register name if provided
        if let Some(name) = &entry.metadata.name {
            self.register_name(name, hash.clone())?;
            
            // Register as latest if version is provided
            if let Some(version) = entry.metadata.additional.get("version") {
                self.register_as_latest(name, version, hash.clone())?;
            }
        }
        
        // Update cache
        let mut cache = self.entry_cache.write().map_err(|_| Error::LockError)?;
        cache.insert(hash, entry);
        
        Ok(())
    }
    
    /// Register a name as the latest version
    fn register_as_latest(&self, name: &str, new_version: &str, hash: Hash) -> Result<()> {
        let versions_dir = self.names_dir().join(name);
        fs::create_dir_all(&versions_dir)?;
        
        // Write hash to version file
        let version_file = versions_dir.join(format!("{}.hash", new_version));
        fs::write(&version_file, hash.to_string())?;
        
        // Check if we need to update the latest
        let latest_file = versions_dir.join("latest");
        let should_update = if latest_file.exists() {
            let latest_version = fs::read_to_string(latest_file.with_extension("version"))?;
            Self::is_version_greater(new_version, &latest_version)
        } else {
            true
        };
        
        if should_update {
            // Write to latest files
            fs::write(&latest_file, hash.to_string())?;
            fs::write(latest_file.with_extension("version"), new_version)?;
        }
        
        Ok(())
    }
    
    /// Compare version strings (semver-like)
    fn is_version_greater(a: &str, b: &str) -> bool {
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
    
    /// Load an entry from the repository
    fn load_entry(&self, hash: &Hash) -> Result<CodeEntry> {
        // Check cache first
        {
            let cache = self.entry_cache.read().map_err(|_| Error::LockError)?;
            if let Some(entry) = cache.get(hash) {
                return Ok(entry.clone());
            }
        }
        
        // Load from disk
        let metadata_path = self.metadata_path(hash);
        if !metadata_path.exists() {
            return Err(Error::HashNotFound(hash.to_string()));
        }
        
        let metadata_json = fs::read_to_string(metadata_path)?;
        let metadata: CodeMetadata = serde_json::from_str(&metadata_json)?;
        
        // Don't load content by default, it will be loaded on demand
        let entry = CodeEntry::new(hash.clone(), metadata, None);
        
        // Update cache
        let mut cache = self.entry_cache.write().map_err(|_| Error::LockError)?;
        cache.insert(hash.clone(), entry.clone());
        
        Ok(entry)
    }
    
    /// Load the content for a hash
    fn load_content(&self, hash: &Hash) -> Result<Vec<u8>> {
        let object_path = self.object_path(hash);
        if !object_path.exists() {
            return Err(Error::HashNotFound(hash.to_string()));
        }
        
        Ok(fs::read(object_path)?)
    }
    
    /// Load name registry from disk
    fn load_name_registry(root_dir: &Path) -> Result<HashMap<String, Hash>> {
        let names_dir = root_dir.join("names");
        if !names_dir.exists() {
            return Ok(HashMap::new());
        }
        
        let mut registry = HashMap::new();
        
        for name_entry in fs::read_dir(names_dir)? {
            let name_entry = name_entry?;
            if !name_entry.file_type()?.is_dir() {
                continue;
            }
            
            let name = name_entry.file_name().to_string_lossy().to_string();
            let latest_file = name_entry.path().join("latest");
            
            if latest_file.exists() {
                let hash_str = fs::read_to_string(latest_file)?;
                match Hash::from_str(&hash_str) {
                    Ok(hash) => {
                        registry.insert(name, hash);
                    }
                    Err(_) => {
                        // Skip invalid hash strings
                        continue;
                    }
                }
            }
        }
        
        Ok(registry)
    }
    
    /// Get the path to the registry file
    fn registry_path(&self) -> PathBuf {
        self.root_dir.join("registry.json")
    }
    
    /// Load registry from disk
    fn load_registry(&self) -> Result<HashMap<String, Hash>> {
        let registry_path = self.registry_path();
        if !registry_path.exists() {
            return Ok(HashMap::new());
        }
        
        let contents = fs::read_to_string(registry_path)?;
        let registry: HashMap<String, String> = serde_json::from_str(&contents)?;
        
        let mut result = HashMap::new();
        for (name, hash_str) in registry {
            if let Ok(hash) = Hash::from_str(&hash_str) {
                result.insert(name, hash);
            }
        }
        
        Ok(result)
    }
    
    /// Convert dependencies from a map to a list of hashes
    pub fn dependencies_to_map(&self, entry: &CodeEntry) -> Result<HashMap<String, String>> {
        if let Some(deps) = &entry.metadata.dependencies {
            Ok(deps.clone())
        } else {
            Ok(HashMap::new())
        }
    }
}

impl CodeRepository for FileSystemCodeRepository {
    fn store<T: Serialize + Debug>(&self, object: &T, metadata: CodeMetadata) -> Result<Hash> {
        // Serialize the object
        let content = bincode::serialize(object)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize: {}", e)))?;
        
        // Hash the content
        let hasher = self.hasher_factory.create_hasher(self.default_algorithm);
        let hash = hasher.hash_bytes(&content);
        
        // Create and store the entry
        let entry = CodeEntry::new(hash.clone(), metadata, Some(content));
        self.store_entry(entry)?;
        
        Ok(hash)
    }
    
    fn store_bytes(&self, bytes: &[u8], metadata: CodeMetadata) -> Result<Hash> {
        // Hash the content
        let hasher = self.hasher_factory.create_hasher(self.default_algorithm);
        let hash = hasher.hash_bytes(bytes);
        
        // Create and store the entry
        let entry = CodeEntry::new(hash.clone(), metadata, Some(bytes.to_vec()));
        self.store_entry(entry)?;
        
        Ok(hash)
    }
    
    fn load<T: for<'de> Deserialize<'de>>(&self, hash: &Hash) -> Result<T> {
        let content = self.load_bytes(hash)?;
        bincode::deserialize(&content)
            .map_err(|e| Error::DeserializationError(format!("Failed to deserialize: {}", e)))
    }
    
    fn load_bytes(&self, hash: &Hash) -> Result<Vec<u8>> {
        self.load_content(hash)
    }
    
    fn register_name(&self, name: &str, hash: Hash) -> Result<()> {
        // Update the registry
        {
            let mut registry = self.name_registry.write().map_err(|_| Error::LockError)?;
            registry.insert(name.to_string(), hash.clone());
        }
        
        // Create directories
        let name_dir = self.names_dir().join(name);
        fs::create_dir_all(&name_dir)?;
        
        // Write to latest file
        let latest_file = name_dir.join("latest");
        fs::write(latest_file, hash.to_string())?;
        
        Ok(())
    }
    
    fn resolve_name(&self, name: &str) -> Result<Hash> {
        // Check cache first
        {
            let registry = self.name_registry.read().map_err(|_| Error::LockError)?;
            if let Some(hash) = registry.get(name) {
                return Ok(hash.clone());
            }
        }
        
        // Look on disk
        let latest_file = self.names_dir().join(name).join("latest");
        if latest_file.exists() {
            let hash_str = fs::read_to_string(latest_file)?;
            let hash = Hash::from_str(&hash_str)
                .map_err(|_| Error::ParseError(format!("Invalid hash string: {}", hash_str)))?;
            
            // Update cache
            {
                let mut registry = self.name_registry.write().map_err(|_| Error::LockError)?;
                registry.insert(name.to_string(), hash.clone());
            }
            
            return Ok(hash);
        }
        
        Err(Error::NameNotFound(name.to_string()))
    }
    
    fn exists(&self, hash: &Hash) -> bool {
        // Check cache first
        {
            if let Ok(cache) = self.entry_cache.read() {
                if cache.contains_key(hash) {
                    return true;
                }
            }
        }
        
        // Check on disk
        self.object_path(hash).exists() && self.metadata_path(hash).exists()
    }
    
    fn get_metadata(&self, hash: &Hash) -> Result<CodeMetadata> {
        let entry = self.load_entry(hash)?;
        Ok(entry.metadata)
    }
    
    fn is_risc_v_compatible(&self, hash: &Hash) -> Result<bool> {
        let entry = self.load_entry(hash)?;
        Ok(entry.is_risc_v_compatible())
    }
    
    fn get_versions(&self, name: &str) -> Result<Vec<String>> {
        let dir = self.names_dir().join(name);
        if !dir.exists() {
            return Err(Error::NameNotFound(name.to_string()));
        }
        
        let mut versions = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if file_name == "latest" || file_name == "latest.version" {
                continue;
            }
            
            if let Some(version) = file_name.strip_suffix(".hash") {
                versions.push(version.to_string());
            }
        }
        
        // Sort versions in ascending order
        versions.sort_by(|a, b| {
            if Self::is_version_greater(a, b) {
                std::cmp::Ordering::Greater
            } else if Self::is_version_greater(b, a) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[derive(Debug, Serialize, Deserialize)]
    struct TestObject {
        name: String,
        value: i32,
    }
    
    #[test]
    fn test_store_and_load() -> Result<()> {
        let temp_dir = tempdir()?;
        let repo = FileSystemCodeRepository::new(temp_dir.path())?;
        
        // Create a test object
        let test_obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };
        
        // Create metadata
        let metadata = CodeMetadata::builder("json")
            .with_name("test-object")
            .with_description("A test object")
            .build();
        
        // Store the object
        let hash = repo.store(&test_obj, metadata)?;
        
        // Load the object
        let loaded: TestObject = repo.load(&hash)?;
        
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.value, 42);
        
        Ok(())
    }
    
    #[test]
    fn test_versioning() -> Result<()> {
        let temp_dir = tempdir()?;
        let repo = FileSystemCodeRepository::new(temp_dir.path())?;
        
        // Create test objects with different versions
        let obj_v1 = TestObject {
            name: "test-v1".to_string(),
            value: 1,
        };
        
        let obj_v2 = TestObject {
            name: "test-v2".to_string(),
            value: 2,
        };
        
        // Create metadata with versions
        let metadata_v1 = CodeMetadataBuilder::new()
            .with_name("versioned-object")
            .with_format("json")
            .with_version("1.0.0")
            .with_additional_field("version", "1.0.0")
            .build();
        
        let metadata_v2 = CodeMetadataBuilder::new()
            .with_name("versioned-object")
            .with_format("json")
            .with_version("2.0.0")
            .with_additional_field("version", "2.0.0")
            .build();
        
        // Store the objects
        let hash_v1 = repo.store(&obj_v1, metadata_v1)?;
        let hash_v2 = repo.store(&obj_v2, metadata_v2)?;
        
        // Resolve the latest version
        let latest_hash = repo.resolve_name("versioned-object")?;
        
        // Latest should be v2
        assert_eq!(latest_hash, hash_v2);
        
        // Get all versions
        let versions = repo.get_versions("versioned-object")?;
        
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));
        
        Ok(())
    }
}

// Create a re-export module in the code directory to maintain compatibility
pub mod compatibility {
    pub use super::{CodeRepository, CodeEntry, CodeMetadata, CodeMetadataBuilder, FileSystemCodeRepository};
} 