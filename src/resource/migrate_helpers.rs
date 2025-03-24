// Migration helpers for resource register unification
//
// This module contains utilities to help with migration from legacy code to the unified system

use crate::resource::{ResourceRegister, ContentId};
use crate::error::Result;
use crate::resource::resource_register::{ResourceLogic, FungibilityDomain, Quantity, StorageStrategy, StateVisibility};
use std::collections::HashMap;

/// Mark a file as migrated to the unified resource model
pub fn mark_file_as_migrated(file_path: &str) -> Result<()> {
    // Implementation can be added as needed
    println!("File marked as migrated: {}", file_path);
    Ok(())
}

/// Check if a file has been migrated
pub fn is_file_migrated(file_path: &str) -> Result<bool> {
    // Implementation can be added as needed
    // For now, just return false
    Ok(false)
}

/// Convert legacy resource to unified register
pub fn convert_to_unified_register(resource_id: &ContentId) -> Result<ResourceRegister> {
    // Placeholder for conversion logic
    Err(crate::error::Error::NotImplemented("Resource conversion not yet implemented".to_string()))
}

/// Create a new ResourceRegister with basic information
pub fn create_resource_register(
    domain: impl Into<String>,
    resource_type: impl Into<String>,
    data: Vec<u8>
) -> ResourceRegister {
    // Create resource ID using content addressing
    let hasher = crate::crypto::hash::HashFactory::default().create_hasher().unwrap();
    let id = ContentId::from(hasher.hash(&data));
    
    // Setup resource logic based on type
    let resource_logic = ResourceLogic::Custom(resource_type.into());
    
    // Create base metadata
    let mut metadata = HashMap::new();
    metadata.insert("domain".to_string(), domain.into());
    metadata.insert("data_hash".to_string(), hex::encode(&id.0.to_vec()));
    metadata.insert("version".to_string(), "1".to_string());
    
    // Create the resource register
    let mut register = ResourceRegister::new(
        id,
        resource_logic,
        FungibilityDomain("non-fungible".to_string()),
        Quantity(1),
        metadata,
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Store data in contents
    register.contents = data;
    
    register
}

/// Create a ResourceRegister with additional metadata
pub fn create_register_with_metadata(
    domain: impl Into<String>,
    resource_type: impl Into<String>,
    data: Vec<u8>,
    additional_metadata: HashMap<String, String>
) -> ResourceRegister {
    // Create resource ID using content addressing
    let hasher = crate::crypto::hash::HashFactory::default().create_hasher().unwrap();
    let id = ContentId::from(hasher.hash(&data));
    
    // Setup resource logic based on type
    let resource_logic = ResourceLogic::Custom(resource_type.into());
    
    // Create base metadata
    let mut metadata = HashMap::new();
    metadata.insert("domain".to_string(), domain.into());
    metadata.insert("data_hash".to_string(), hex::encode(&id.0.to_vec()));
    metadata.insert("version".to_string(), "1".to_string());
    
    // Add additional metadata
    for (key, value) in additional_metadata {
        metadata.insert(key, value);
    }
    
    // Create the resource register
    let mut register = ResourceRegister::new(
        id,
        resource_logic,
        FungibilityDomain("non-fungible".to_string()),
        Quantity(1),
        metadata,
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Store data in contents
    register.contents = data;
    
    register
}

/// Update data in a ResourceRegister
pub fn update_register_data(
    register: &mut ResourceRegister,
    new_data: Vec<u8>
) -> Result<()> {
    // Update the contents
    register.contents = new_data;
    
    // Update the data hash in metadata
    let hasher = crate::crypto::hash::HashFactory::default().create_hasher().unwrap();
    let data_hash = hasher.hash(&new_data);
    register.metadata.insert("data_hash".to_string(), hex::encode(&data_hash.to_vec()));
    
    // Update version
    let current_version = register.metadata.get("version")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    register.metadata.insert("version".to_string(), (current_version + 1).to_string());
    
    Ok(())
} 