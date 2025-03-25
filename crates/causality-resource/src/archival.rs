// Resource archival functionality
// Original file: src/resource/archival.rs

// Register archival system
//
// This module implements the register archival system as described in ADR-006.
// It provides functionality for:
// - Archiving registers to persistent storage
// - Retrieving archived registers from storage
// - Verifying archive integrity

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use crate::resource::register::{
    Register, RegisterId, RegisterContents, RegisterState, BlockHeight, ArchiveReference
};
use causality_resource::EpochId;
use causality_crypto::ContentId;
use causality_types::{Hash256, Domain};

/// Archive storage interface for persisting register archives
pub trait ArchiveStorage: Send + Sync {
    /// Store an archive
    fn store(&self, key: &str, data: &[u8]) -> Result<()>;
    
    /// Retrieve an archive
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Check if an archive exists
    fn exists(&self, key: &str) -> Result<bool>;
    
    /// Delete an archive
    fn delete(&self, key: &str) -> Result<()>;
    
    /// List all archive keys
    fn list_keys(&self) -> Result<Vec<String>>;
}

/// File system based archive storage implementation
pub struct FileSystemStorage {
    /// Base directory for storing archives
    base_dir: PathBuf,
}

impl FileSystemStorage {
    /// Create a new file system storage instance
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // Ensure directory exists
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)
                .map_err(|e| Error::IoError(format!("Failed to create archive directory: {}", e)))?;
        }
        
        Ok(Self { base_dir })
    }
    
    /// Get the full path for an archive key
    fn path_for_key(&self, key: &str) -> PathBuf {
        self.base_dir.join(format!("{}.archive", key))
    }
}

impl ArchiveStorage for FileSystemStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = self.path_for_key(key);
        
        std::fs::write(&path, data)
            .map_err(|e| Error::IoError(format!("Failed to write archive {}: {}", key, e)))?;
            
        Ok(())
    }
    
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path_for_key(key);
        
        if path.exists() {
            let data = std::fs::read(&path)
                .map_err(|e| Error::IoError(format!("Failed to read archive {}: {}", key, e)))?;
                
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
    
    fn exists(&self, key: &str) -> Result<bool> {
        let path = self.path_for_key(key);
        Ok(path.exists())
    }
    
    fn delete(&self, key: &str) -> Result<()> {
        let path = self.path_for_key(key);
        
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| Error::IoError(format!("Failed to delete archive {}: {}", key, e)))?;
        }
        
        Ok(())
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        let entries = std::fs::read_dir(&self.base_dir)
            .map_err(|e| Error::IoError(format!("Failed to read archive directory: {}", e)))?;
            
        let mut keys = Vec::new();
        
        for entry in entries {
            let entry = entry
                .map_err(|e| Error::IoError(format!("Failed to read directory entry: {}", e)))?;
                
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "archive") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    keys.push(filename.to_string());
                }
            }
        }
        
        Ok(keys)
    }
}

/// In-memory storage for testing and development
pub struct InMemoryStorage {
    /// Map of archive key to data
    archives: RwLock<HashMap<String, Vec<u8>>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            archives: RwLock::new(HashMap::new()),
        }
    }
}

impl ArchiveStorage for InMemoryStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut archives = self.archives.write()
            .map_err(|_| Error::LockError("Failed to acquire archives lock for writing".to_string()))?;
            
        archives.insert(key.to_string(), data.to_vec());
        
        Ok(())
    }
    
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let archives = self.archives.read()
            .map_err(|_| Error::LockError("Failed to acquire archives lock".to_string()))?;
            
        Ok(archives.get(key).cloned())
    }
    
    fn exists(&self, key: &str) -> Result<bool> {
        let archives = self.archives.read()
            .map_err(|_| Error::LockError("Failed to acquire archives lock".to_string()))?;
            
        Ok(archives.contains_key(key))
    }
    
    fn delete(&self, key: &str) -> Result<()> {
        let mut archives = self.archives.write()
            .map_err(|_| Error::LockError("Failed to acquire archives lock for writing".to_string()))?;
            
        archives.remove(key);
        
        Ok(())
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        let archives = self.archives.read()
            .map_err(|_| Error::LockError("Failed to acquire archives lock".to_string()))?;
            
        Ok(archives.keys().cloned().collect())
    }
}

/// Archive compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionFormat {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// Zstd compression
    Zstd,
}

impl fmt::Display for CompressionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Gzip => write!(f, "gzip"),
            Self::Zstd => write!(f, "zstd"),
        }
    }
}

impl Default for CompressionFormat {
    fn default() -> Self {
        Self::Zstd
    }
}

/// Archive for a register
pub struct RegisterArchive {
    /// Archive format version
    pub version: u32,
    
    /// ID of the archival register
    pub register_id: ContentId,
    
    /// Domain of the register
    pub domain: Domain,
    
    /// Original creation timestamp
    pub created_at: u64,
    
    /// Archival timestamp
    pub archived_at: u64,
    
    /// Block height at archival time
    pub block_height: BlockHeight,
    
    /// Epoch when archived
    pub epoch: EpochId,
    
    /// Original register state
    pub original_state: RegisterState,
    
    /// Register contents
    pub contents: Vec<u8>,
    
    /// Register metadata
    pub metadata: HashMap<String, String>,
    
    /// Compression format
    pub compression: CompressionFormat,
    
    /// Content hash for integrity verification
    pub content_hash: Hash256,
    
    /// Archive hash for integrity verification
    pub archive_hash: Hash256,
}

impl RegisterArchive {
    /// Create an archive from a resource register
    pub fn from_resource_register(
        register: &Register,
        epoch: EpochId,
        block_height: BlockHeight,
        compression: CompressionFormat,
    ) -> Result<Self> {
        // Create timestamp for archival
        let archived_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::TimeError(format!("Failed to get current time: {}", e)))?
            .as_secs();
            
        // Create the archive with uncompressed contents
        let mut archive = Self {
            version: 1, // Current archive format version
            register_id: register.register_id.clone(),
            domain: register.domain.clone(),
            created_at: register.created_at,
            archived_at,
            block_height,
            epoch,
            original_state: register.state.clone(),
            contents: register.contents.as_bytes().to_vec(),
            metadata: register.metadata.clone(),
            compression: CompressionFormat::None, // Start with no compression
            content_hash: register.content_hash.clone(),
            archive_hash: Hash256::default(), // Will be computed after compression
        };
        
        // Apply compression if specified
        if compression != CompressionFormat::None {
            archive.compression = compression;
            archive.compress()?;
        }
        
        // Compute the archive hash for verification
        archive.archive_hash = archive.compute_archive_hash()?;
        
        Ok(archive)
    }
    
    /// Backward compatibility function
    #[deprecated(since = "0.8.0", note = "Use from_resource_register instead")]
    pub fn from_register(
        register: &Register,
        epoch: EpochId,
        block_height: BlockHeight,
        compression: CompressionFormat,
    ) -> Result<Self> {
        Self::from_resource_register(register, epoch, block_height, compression)
    }
    
    /// Compress the archive contents
    fn compress(&mut self) -> Result<()> {
        match self.compression {
            CompressionFormat::None => { /* Do nothing */ },
            CompressionFormat::Gzip => {
                use std::io::Write;
                
                let mut encoder = flate2::write::GzEncoder::new(
                    Vec::new(), 
                    flate2::Compression::default()
                );
                
                encoder.write_all(&self.contents)
                    .map_err(|e| Error::DataError(format!("Failed to compress archive: {}", e)))?;
                    
                self.contents = encoder.finish()
                    .map_err(|e| Error::DataError(format!("Failed to finish compression: {}", e)))?;
            },
            CompressionFormat::Zstd => {
                self.contents = zstd::encode_all(&self.contents[..], 0)
                    .map_err(|e| Error::DataError(format!("Failed to compress archive: {}", e)))?;
            },
        }
        
        Ok(())
    }
    
    /// Decompress the archive contents
    fn decompress(&mut self) -> Result<()> {
        match self.compression {
            CompressionFormat::None => { /* Do nothing */ },
            CompressionFormat::Gzip => {
                use std::io::Read;
                
                let mut decoder = flate2::read::GzDecoder::new(&self.contents[..]);
                let mut decompressed = Vec::new();
                
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| Error::DataError(format!("Failed to decompress archive: {}", e)))?;
                    
                self.contents = decompressed;
            },
            CompressionFormat::Zstd => {
                self.contents = zstd::decode_all(&self.contents[..])
                    .map_err(|e| Error::DataError(format!("Failed to decompress archive: {}", e)))?;
            },
        }
        
        // Reset compression format
        self.compression = CompressionFormat::None;
        
        Ok(())
    }
    
    /// Serialize the archive to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| Error::DataError(format!("Failed to serialize archive: {}", e)))
    }
    
    /// Deserialize an archive from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| Error::DataError(format!("Failed to deserialize archive: {}", e)))
    }
    
    /// Verify archive integrity
    pub fn verify_integrity(&self) -> Result<bool> {
        // Calculate archive hash (excluding the hash itself)
        let mut hash_data = Vec::new();
        hash_data.extend_from_slice(&self.version.to_be_bytes());
        hash_data.extend_from_slice(self.register_id.to_string().as_bytes());
        hash_data.extend_from_slice(self.domain.to_string().as_bytes());
        hash_data.extend_from_slice(&self.created_at.to_be_bytes());
        hash_data.extend_from_slice(&self.archived_at.to_be_bytes());
        hash_data.extend_from_slice(&self.block_height.to_be_bytes());
        hash_data.extend_from_slice(&self.epoch.to_be_bytes());
        hash_data.extend_from_slice(&[self.original_state as u8]);
        hash_data.extend_from_slice(&self.contents);
        hash_data.extend_from_slice(self.content_hash.as_bytes());
        
        for (key, value) in &self.metadata {
            hash_data.extend_from_slice(key.as_bytes());
            hash_data.extend_from_slice(value.as_bytes());
        }
        
        let calculated_hash = Hash256::digest(&hash_data);
        
        Ok(calculated_hash == self.archive_hash)
    }
    
    /// Restore register from archive
    pub fn to_register(&mut self) -> Result<Register> {
        // Decompress if needed
        if self.compression != CompressionFormat::None {
            self.decompress()?;
        }
        
        // Verify content hash
        let content_hash = Hash256::digest(&self.contents);
        
        if content_hash != self.content_hash {
            return Err(Error::IntegrityError(
                "Archive content hash mismatch".to_string()
            ));
        }
        
        // Create register reference to the archive
        let archive_reference = ArchiveReference {
            archived_at: self.archived_at,
            epoch: self.epoch,
            archive_hash: self.archive_hash.clone(),
        };
        
        // Create the register
        let register = Register {
            register_id: self.register_id.clone(),
            domain: self.domain.clone(),
            owner: Default::default(), // Use default address for archived registers
            contents: RegisterContents::from_bytes(&self.contents),
            state: RegisterState::Archived,
            created_at: self.created_at,
            updated_at: self.archived_at,
            version: 1,
            metadata: self.metadata.clone(),
            archive_reference: Some(archive_reference),
            summarizes: Vec::new(),
            summarized_by: None,
            successors: Vec::new(),
            predecessors: Vec::new(),
        };
        
        Ok(register)
    }
    
    /// Generate a storage key for this archive
    pub fn storage_key(&self) -> String {
        format!("{}_{}_{}", self.register_id, self.epoch, self.archived_at)
    }
}

/// Archive manager for persisting and retrieving register archives
pub struct ArchiveManager {
    /// Storage backend
    storage: Arc<dyn ArchiveStorage>,
    
    /// Default compression format
    default_compression: CompressionFormat,
}

impl ArchiveManager {
    /// Create a new archive manager
    pub fn new(storage: Arc<dyn ArchiveStorage>, default_compression: CompressionFormat) -> Self {
        Self {
            storage,
            default_compression,
        }
    }
    
    /// Archive a register
    pub fn archive_register(
        &self,
        register: &Register,
        epoch: EpochId,
        block_height: BlockHeight,
    ) -> Result<ArchiveReference> {
        // Create an archive from the register
        let mut archive = RegisterArchive::from_resource_register(
            register,
            epoch,
            block_height,
            self.default_compression,
        )?;
        
        // Generate storage key
        let key = archive.storage_key();
        
        // Serialize archive
        let data = archive.to_bytes()?;
        
        // Store archive
        self.storage.store(&key, &data)?;
        
        // Create archive reference
        let reference = ArchiveReference {
            archived_at: archive.archived_at,
            epoch,
            archive_hash: archive.archive_hash.clone(),
        };
        
        Ok(reference)
    }
    
    /// Retrieve a register from archive
    pub fn retrieve_register(&self, reference: &ArchiveReference) -> Result<Option<Register>> {
        // Generate a pattern to search for
        let pattern = format!("{}_{}", reference.archive_hash, reference.epoch);
        
        // List archives
        let keys = self.storage.list_keys()?;
        
        // Find matching key
        let mut matching_key = None;
        
        for key in keys {
            if key.contains(&pattern) {
                matching_key = Some(key);
                break;
            }
        }
        
        if let Some(key) = matching_key {
            // Retrieve archive data
            if let Some(data) = self.storage.retrieve(&key)? {
                // Deserialize archive
                let mut archive = RegisterArchive::from_bytes(&data)?;
                
                // Verify archive integrity
                if !archive.verify_integrity()? {
                    return Err(Error::IntegrityError(
                        "Archive integrity check failed".to_string()
                    ));
                }
                
                // Restore register
                let register = archive.to_register()?;
                
                return Ok(Some(register));
            }
        }
        
        Ok(None)
    }
    
    /// Delete an archive
    pub fn delete_archive(&self, reference: &ArchiveReference) -> Result<bool> {
        // Generate a pattern to search for
        let pattern = format!("{}_{}", reference.archive_hash, reference.epoch);
        
        // List archives
        let keys = self.storage.list_keys()?;
        
        // Find matching key
        let mut matching_key = None;
        
        for key in keys {
            if key.contains(&pattern) {
                matching_key = Some(key);
                break;
            }
        }
        
        if let Some(key) = matching_key {
            self.storage.delete(&key)?;
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// List all archives
    pub fn list_archives(&self) -> Result<Vec<String>> {
        self.storage.list_keys()
    }
    
    /// Verify archive exists and is intact
    pub fn verify_archive(&self, reference: &ArchiveReference) -> Result<bool> {
        // Generate a pattern to search for
        let pattern = format!("{}_{}", reference.archive_hash, reference.epoch);
        
        // List archives
        let keys = self.storage.list_keys()?;
        
        // Find matching key
        let mut matching_key = None;
        
        for key in keys {
            if key.contains(&pattern) {
                matching_key = Some(key);
                break;
            }
        }
        
        if let Some(key) = matching_key {
            // Retrieve archive data
            if let Some(data) = self.storage.retrieve(&key)? {
                // Deserialize archive
                let archive = RegisterArchive::from_bytes(&data)?;
                
                // Verify archive integrity
                return archive.verify_integrity();
            }
        }
        
        Ok(false)
    }
}

/// Thread-safe archive manager
pub struct SharedArchiveManager {
    /// Inner archive manager
    inner: Arc<ArchiveManager>,
}

impl SharedArchiveManager {
    /// Create a new shared archive manager with file system storage
    pub fn new_with_fs<P: AsRef<Path>>(
        base_dir: P,
        compression: Option<CompressionFormat>,
    ) -> Result<Self> {
        let storage = Arc::new(FileSystemStorage::new(base_dir)?);
        
        Ok(Self {
            inner: Arc::new(ArchiveManager::new(
                storage,
                compression.unwrap_or_default(),
            )),
        })
    }
    
    /// Create a new shared archive manager with in-memory storage
    pub fn new_in_memory(compression: Option<CompressionFormat>) -> Self {
        let storage = Arc::new(InMemoryStorage::new());
        
        Self {
            inner: Arc::new(ArchiveManager::new(
                storage,
                compression.unwrap_or_default(),
            )),
        }
    }
    
    /// Get the inner archive manager
    pub fn inner(&self) -> Arc<ArchiveManager> {
        self.inner.clone()
    }
    
    // Delegate methods to inner manager
    
    /// Archive a register
    pub fn archive_register(
        &self,
        register: &Register,
        epoch: EpochId,
        block_height: BlockHeight,
    ) -> Result<ArchiveReference> {
        self.inner.archive_register(register, epoch, block_height)
    }
    
    /// Retrieve a register from archive
    pub fn retrieve_register(&self, reference: &ArchiveReference) -> Result<Option<Register>> {
        self.inner.retrieve_register(reference)
    }
    
    /// Delete an archive
    pub fn delete_archive(&self, reference: &ArchiveReference) -> Result<bool> {
        self.inner.delete_archive(reference)
    }
    
    /// List all archives
    pub fn list_archives(&self) -> Result<Vec<String>> {
        self.inner.list_archives()
    }
    
    /// Verify archive exists and is intact
    pub fn verify_archive(&self, reference: &ArchiveReference) -> Result<bool> {
        self.inner.verify_archive(reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use causality_types::Address;
    
    fn create_test_register() -> Register {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());
        
        Register {
            register_id: ContentId::new_unique(),
            domain: Domain::new("test_domain"),
            owner: Address::new("test_owner"),
            contents: RegisterContents::with_string("Test register contents"),
            state: RegisterState::Active,
            created_at: 1000,
            updated_at: 1000,
            version: 1,
            metadata,
            archive_reference: None,
            summarizes: Vec::new(),
            summarized_by: None,
            successors: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    #[test]
    fn test_archive_creation() -> Result<()> {
        let register = create_test_register();
        
        // Create archive
        let archive = RegisterArchive::from_resource_register(
            &register,
            1, // epoch
            100, // block height
            CompressionFormat::None,
        )?;
        
        // Check basic fields
        assert_eq!(archive.register_id, register.register_id);
        assert_eq!(archive.domain, register.domain);
        assert_eq!(archive.created_at, register.created_at);
        assert_eq!(archive.epoch, 1);
        assert_eq!(archive.block_height, 100);
        assert_eq!(archive.original_state, RegisterState::Active);
        
        // Check content hash
        let content_hash = Hash256::digest(&archive.contents);
        assert_eq!(archive.content_hash, content_hash);
        
        // Check integrity
        assert!(archive.verify_integrity()?);
        
        Ok(())
    }
    
    #[test]
    fn test_compression() -> Result<()> {
        let register = create_test_register();
        
        // Create uncompressed archive for comparison
        let uncompressed = RegisterArchive::from_resource_register(
            &register,
            1,
            100,
            CompressionFormat::None,
        )?;
        
        // Create gzip compressed archive
        let gzip = RegisterArchive::from_resource_register(
            &register,
            1,
            100,
            CompressionFormat::Gzip,
        )?;
        
        // Create zstd compressed archive
        let zstd = RegisterArchive::from_resource_register(
            &register,
            1,
            100,
            CompressionFormat::Zstd,
        )?;
        
        // Compression should reduce size
        assert!(gzip.contents.len() < uncompressed.contents.len());
        assert!(zstd.contents.len() < uncompressed.contents.len());
        
        // Check that all archives have different hashes
        assert_ne!(uncompressed.archive_hash, gzip.archive_hash);
        assert_ne!(uncompressed.archive_hash, zstd.archive_hash);
        assert_ne!(gzip.archive_hash, zstd.archive_hash);
        
        // Check integrity
        assert!(gzip.verify_integrity()?);
        assert!(zstd.verify_integrity()?);
        
        Ok(())
    }
    
    #[test]
    fn test_serialize_deserialize() -> Result<()> {
        let register = create_test_register();
        
        // Create archive
        let archive = RegisterArchive::from_resource_register(
            &register,
            1,
            100,
            CompressionFormat::Zstd,
        )?;
        
        // Serialize
        let data = archive.to_bytes()?;
        
        // Deserialize
        let deserialized = RegisterArchive::from_bytes(&data)?;
        
        // Check fields
        assert_eq!(deserialized.register_id, archive.register_id);
        assert_eq!(deserialized.domain, archive.domain);
        assert_eq!(deserialized.created_at, archive.created_at);
        assert_eq!(deserialized.archived_at, archive.archived_at);
        assert_eq!(deserialized.epoch, archive.epoch);
        assert_eq!(deserialized.block_height, archive.block_height);
        assert_eq!(deserialized.original_state, archive.original_state);
        assert_eq!(deserialized.contents, archive.contents);
        assert_eq!(deserialized.metadata, archive.metadata);
        assert_eq!(deserialized.compression, archive.compression);
        assert_eq!(deserialized.content_hash, archive.content_hash);
        assert_eq!(deserialized.archive_hash, archive.archive_hash);
        
        // Check integrity
        assert!(deserialized.verify_integrity()?);
        
        Ok(())
    }
    
    #[test]
    fn test_register_roundtrip() -> Result<()> {
        let original = create_test_register();
        
        // Create archive
        let mut archive = RegisterArchive::from_resource_register(
            &original,
            1,
            100,
            CompressionFormat::Zstd,
        )?;
        
        // Convert back to register
        let restored = archive.to_register()?;
        
        // Check fields
        assert_eq!(restored.register_id, original.register_id);
        assert_eq!(restored.domain, original.domain);
        assert_eq!(restored.contents.to_bytes(), original.contents.to_bytes());
        assert_eq!(restored.created_at, original.created_at);
        assert_eq!(restored.metadata, original.metadata);
        
        // State should be Archived
        assert_eq!(restored.state, RegisterState::Archived);
        
        // Should have archive reference
        assert!(restored.archive_reference.is_some());
        let reference = restored.archive_reference.as_ref().unwrap();
        assert_eq!(reference.epoch, 1);
        assert_eq!(reference.archive_hash, archive.archive_hash);
        
        Ok(())
    }
    
    #[test]
    fn test_in_memory_storage() -> Result<()> {
        let storage = InMemoryStorage::new();
        
        // Store data
        storage.store("test1", b"test data 1")?;
        storage.store("test2", b"test data 2")?;
        
        // Check existence
        assert!(storage.exists("test1")?);
        assert!(storage.exists("test2")?);
        assert!(!storage.exists("test3")?);
        
        // Retrieve data
        let data1 = storage.retrieve("test1")?;
        assert_eq!(data1, Some(b"test data 1".to_vec()));
        
        // List keys
        let keys = storage.list_keys()?;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"test1".to_string()));
        assert!(keys.contains(&"test2".to_string()));
        
        // Delete data
        storage.delete("test1")?;
        assert!(!storage.exists("test1")?);
        assert!(storage.exists("test2")?);
        
        Ok(())
    }
    
    #[test]
    fn test_file_system_storage() -> Result<()> {
        // Create temporary directory
        let temp_dir = tempfile::tempdir()
            .map_err(|e| Error::IoError(format!("Failed to create temp dir: {}", e)))?;
            
        let storage = FileSystemStorage::new(temp_dir.path())?;
        
        // Store data
        storage.store("test1", b"test data 1")?;
        storage.store("test2", b"test data 2")?;
        
        // Check existence
        assert!(storage.exists("test1")?);
        assert!(storage.exists("test2")?);
        assert!(!storage.exists("test3")?);
        
        // Retrieve data
        let data1 = storage.retrieve("test1")?;
        assert_eq!(data1, Some(b"test data 1".to_vec()));
        
        // List keys
        let keys = storage.list_keys()?;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"test1".to_string()));
        assert!(keys.contains(&"test2".to_string()));
        
        // Delete data
        storage.delete("test1")?;
        assert!(!storage.exists("test1")?);
        assert!(storage.exists("test2")?);
        
        Ok(())
    }
    
    #[test]
    fn test_archive_manager() -> Result<()> {
        let storage = Arc::new(InMemoryStorage::new());
        let manager = ArchiveManager::new(storage, CompressionFormat::Zstd);
        
        let register = create_test_register();
        
        // Archive a register
        let reference = manager.archive_register(&register, 1, 100)?;
        
        // Verify archive
        let exists = manager.verify_archive(&reference)?;
        assert!(exists);
        
        // List archives
        let archives = manager.list_archives()?;
        assert_eq!(archives.len(), 1);
        
        // Retrieve the register
        let retrieved = manager.retrieve_register(&reference)?;
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.register_id, register.register_id);
        assert_eq!(retrieved.domain, register.domain);
        assert_eq!(retrieved.contents.as_string(), register.contents.as_string());
        assert_eq!(retrieved.state, RegisterState::Archived);
        
        // Delete the archive
        let deleted = manager.delete_archive(&reference)?;
        assert!(deleted);
        
        // Verify it's gone
        let exists = manager.verify_archive(&reference)?;
        assert!(!exists);
        
        Ok(())
    }
    
    #[test]
    fn test_shared_archive_manager() -> Result<()> {
        let manager = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Gzip));
        
        let register = create_test_register();
        
        // Archive a register
        let reference = manager.archive_register(&register, 1, 100)?;
        
        // Verify archive
        let exists = manager.verify_archive(&reference)?;
        assert!(exists);
        
        // Retrieve the register
        let retrieved = manager.retrieve_register(&reference)?;
        assert!(retrieved.is_some());
        
        Ok(())
    }
} 
