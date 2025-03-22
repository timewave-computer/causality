use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterState, ArchiveReference,
    SharedArchiveManager, CompressionFormat, ArchiveStorage, InMemoryStorage,
    FileSystemStorage, ArchiveManager
};
use crate::types::{Address, Domain, Hash256};

// Helper function to create a test register
fn create_test_register(size: usize) -> Register {
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());
    metadata.insert("content_type".to_string(), "text/plain".to_string());
    
    // Create content with specified size
    let content = "A".repeat(size);
    
    Register {
        register_id: RegisterId::new_unique(),
        domain: Domain::new("test_domain"),
        owner: Address::new("test_owner"),
        contents: RegisterContents::with_string(&content),
        state: RegisterState::Active,
        created_at: 1000,
        updated_at: 1100,
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
fn test_in_memory_archive_store_retrieve() -> Result<()> {
    // Create a register with some content
    let register = create_test_register(1000);
    let register_id = register.register_id.clone();
    
    // Create an in-memory archive manager
    let manager = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Zstd));
    
    // Archive the register
    let archive_ref = manager.archive_register(&register, 1, 100)?;
    
    // Check the archive reference
    assert_eq!(archive_ref.epoch, 1);
    assert!(archive_ref.archive_hash != Hash256::default());
    
    // Verify the archive exists
    assert!(manager.verify_archive(&archive_ref)?);
    
    // Retrieve the register
    let retrieved = manager.retrieve_register(&archive_ref)?;
    assert!(retrieved.is_some());
    
    let retrieved = retrieved.unwrap();
    
    // Check that the retrieved register matches the original
    assert_eq!(retrieved.register_id, register_id);
    assert_eq!(retrieved.domain, register.domain);
    assert_eq!(retrieved.contents.as_string(), register.contents.as_string());
    assert_eq!(retrieved.metadata, register.metadata);
    
    // The state should be Archived
    assert_eq!(retrieved.state, RegisterState::Archived);
    
    // It should have an archive reference
    assert!(retrieved.archive_reference.is_some());
    let ref_from_register = retrieved.archive_reference.unwrap();
    assert_eq!(ref_from_register.epoch, archive_ref.epoch);
    assert_eq!(ref_from_register.archive_hash, archive_ref.archive_hash);
    
    Ok(())
}

#[test]
fn test_file_system_archive_store_retrieve() -> Result<()> {
    // Create a temporary directory for archives
    let temp_dir = tempfile::tempdir()
        .map_err(|e| crate::error::Error::IoError(format!("Failed to create temp dir: {}", e)))?;
        
    // Create a register with some content
    let register = create_test_register(1000);
    
    // Create a file system archive manager
    let manager = SharedArchiveManager::new_with_fs(temp_dir.path(), Some(CompressionFormat::Gzip))?;
    
    // Archive the register
    let archive_ref = manager.archive_register(&register, 2, 200)?;
    
    // Verify the archive exists
    assert!(manager.verify_archive(&archive_ref)?);
    
    // Retrieve the register
    let retrieved = manager.retrieve_register(&archive_ref)?;
    assert!(retrieved.is_some());
    
    // Check archive list
    let archives = manager.list_archives()?;
    assert_eq!(archives.len(), 1);
    
    // Delete the archive
    let deleted = manager.delete_archive(&archive_ref)?;
    assert!(deleted);
    
    // Should no longer exist
    assert!(!manager.verify_archive(&archive_ref)?);
    
    Ok(())
}

#[test]
fn test_different_compression_formats() -> Result<()> {
    // Create a register with significant content to test compression
    let register = create_test_register(10000); // 10KB of data
    
    // Create archive managers with different compression formats
    let manager_none = SharedArchiveManager::new_in_memory(Some(CompressionFormat::None));
    let manager_gzip = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Gzip));
    let manager_zstd = SharedArchiveManager::new_in_memory(Some(CompressionFormat::Zstd));
    
    // Archive the register with different compression formats
    let ref_none = manager_none.archive_register(&register, 1, 100)?;
    let ref_gzip = manager_gzip.archive_register(&register, 1, 100)?;
    let ref_zstd = manager_zstd.archive_register(&register, 1, 100)?;
    
    // All should be valid and have different hashes
    assert!(manager_none.verify_archive(&ref_none)?);
    assert!(manager_gzip.verify_archive(&ref_gzip)?);
    assert!(manager_zstd.verify_archive(&ref_zstd)?);
    
    assert_ne!(ref_none.archive_hash, ref_gzip.archive_hash);
    assert_ne!(ref_none.archive_hash, ref_zstd.archive_hash);
    assert_ne!(ref_gzip.archive_hash, ref_zstd.archive_hash);
    
    // All should retrieve the same register content
    let retrieve_none = manager_none.retrieve_register(&ref_none)?.unwrap();
    let retrieve_gzip = manager_gzip.retrieve_register(&ref_gzip)?.unwrap();
    let retrieve_zstd = manager_zstd.retrieve_register(&ref_zstd)?.unwrap();
    
    assert_eq!(retrieve_none.contents.as_string(), register.contents.as_string());
    assert_eq!(retrieve_gzip.contents.as_string(), register.contents.as_string());
    assert_eq!(retrieve_zstd.contents.as_string(), register.contents.as_string());
    
    Ok(())
}

#[test]
fn test_archive_integrity() -> Result<()> {
    // Create a register
    let register = create_test_register(1000);
    
    // Create an archive manager
    let storage = Arc::new(InMemoryStorage::new());
    let manager = ArchiveManager::new(storage.clone(), CompressionFormat::Zstd);
    
    // Archive the register
    let archive_ref = manager.archive_register(&register, 1, 100)?;
    
    // Get the archive key
    let keys = storage.list_keys()?;
    assert_eq!(keys.len(), 1);
    let key = &keys[0];
    
    // Retrieve the archive data
    let data = storage.retrieve(key)?.unwrap();
    
    // Tamper with the data
    let mut tampered_data = data.clone();
    if tampered_data.len() > 100 {
        tampered_data[100] = tampered_data[100].wrapping_add(1);
    }
    
    // Replace the archive with tampered data
    storage.delete(key)?;
    storage.store(key, &tampered_data)?;
    
    // Attempt to verify and retrieve
    let verified = manager.verify_archive(&archive_ref)?;
    assert!(!verified);
    
    // Retrieving should fail with an integrity error
    let result = manager.retrieve_register(&archive_ref);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_archive_multiple_registers() -> Result<()> {
    // Create multiple registers
    let register1 = create_test_register(100);
    let register2 = create_test_register(200);
    let register3 = create_test_register(300);
    
    // Note their IDs
    let id1 = register1.register_id.clone();
    let id2 = register2.register_id.clone();
    let id3 = register3.register_id.clone();
    
    // Create an archive manager
    let manager = SharedArchiveManager::new_in_memory(None);
    
    // Archive all registers
    let ref1 = manager.archive_register(&register1, 1, 100)?;
    let ref2 = manager.archive_register(&register2, 2, 200)?;
    let ref3 = manager.archive_register(&register3, 3, 300)?;
    
    // List archives
    let archives = manager.list_archives()?;
    assert_eq!(archives.len(), 3);
    
    // Retrieve each register
    let retrieved1 = manager.retrieve_register(&ref1)?.unwrap();
    let retrieved2 = manager.retrieve_register(&ref2)?.unwrap();
    let retrieved3 = manager.retrieve_register(&ref3)?.unwrap();
    
    // Check they match
    assert_eq!(retrieved1.register_id, id1);
    assert_eq!(retrieved2.register_id, id2);
    assert_eq!(retrieved3.register_id, id3);
    
    // Check they have correct archive references
    assert_eq!(retrieved1.archive_reference.unwrap().epoch, 1);
    assert_eq!(retrieved2.archive_reference.unwrap().epoch, 2);
    assert_eq!(retrieved3.archive_reference.unwrap().epoch, 3);
    
    Ok(())
} 