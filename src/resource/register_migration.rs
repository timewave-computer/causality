// Register Migration Adapter
//
// This module provides adapters and utilities to migrate from the old TEL register
// implementation to the new unified register implementation.

use std::sync::Arc;
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::types::{Address, Domain, BlockHeight};
use crate::tel::types::Metadata as TelMetadata;
use crate::tel::resource::model::register::{Register as TelRegister, RegisterContents as TelRegisterContents};

use super::register::{Register, RegisterId, RegisterContents, RegisterState, TimeRange, RegisterNullifier, Metadata};

/// Adapter for converting between TEL registers and the new unified register implementation
#[derive(Debug, Clone)]
pub struct RegisterMigrationAdapter {
    /// Mapping of TEL register IDs to the new register IDs
    register_id_mapping: HashMap<String, RegisterId>,
}

impl RegisterMigrationAdapter {
    /// Create a new register migration adapter
    pub fn new() -> Self {
        Self {
            register_id_mapping: HashMap::new(),
        }
    }
    
    /// Convert TEL register contents to the new register contents
    pub fn convert_tel_register_contents(
        &self,
        tel_contents: &TelRegisterContents,
    ) -> Result<RegisterContents> {
        match tel_contents {
            TelRegisterContents::Binary(data) => {
                Ok(RegisterContents::with_binary(data.clone()))
            },
            TelRegisterContents::String(data) => {
                Ok(RegisterContents::with_string(data))
            },
            TelRegisterContents::Json(data) => {
                Ok(RegisterContents::with_json(&serde_json::to_string(data).map_err(|e| {
                    Error::SerializationError(format!("Failed to serialize JSON: {}", e))
                })?))
            },
            _ => {
                // For other TEL register content types, serialize to JSON and store as JSON
                let json = serde_json::to_string(tel_contents).map_err(|e| {
                    Error::SerializationError(format!("Failed to serialize register contents: {}", e))
                })?;
                
                Ok(RegisterContents::with_json(&json))
            }
        }
    }
    
    /// Convert the new register contents to TEL register contents
    pub fn convert_register_contents_to_tel(
        &self,
        contents: &RegisterContents,
    ) -> Result<TelRegisterContents> {
        match contents {
            RegisterContents::Binary(data) => {
                Ok(TelRegisterContents::Binary(data.clone()))
            },
            RegisterContents::String(data) => {
                Ok(TelRegisterContents::String(data.clone()))
            },
            RegisterContents::Json(json) => {
                let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
                    Error::DeserializationError(format!("Failed to deserialize JSON: {}", e))
                })?;
                
                Ok(TelRegisterContents::Json(value))
            },
            RegisterContents::Empty => {
                // For empty contents, create an empty JSON object
                Ok(TelRegisterContents::Json(serde_json::json!({})))
            }
        }
    }
    
    /// Convert a TEL register to the new register format
    pub fn convert_tel_register_to_register(
        &mut self,
        tel_register: &TelRegister,
    ) -> Result<Register> {
        // Convert register ID
        let register_id = self.get_or_create_register_id(&tel_register.id.to_string())?;
        
        // Convert register contents
        let contents = self.convert_tel_register_contents(&tel_register.contents)?;
        
        // Convert metadata
        let metadata = self.convert_tel_metadata_to_metadata(&tel_register.metadata);
        
        // Create the new register
        let mut register = Register::new(
            register_id,
            tel_register.owner.clone(),
            tel_register.domain.clone(),
            contents,
            Some(metadata),
            Some(tel_register.created_at),
            Some(tel_register.last_updated),
        );
        
        // Set additional properties
        register.last_updated_height = tel_register.last_updated_height;
        register.set_epoch(tel_register.epoch);
        register.set_created_by_tx(&tel_register.created_by_tx);
        
        // Set the state to match the TEL register state
        match tel_register.state {
            crate::tel::resource::model::register::RegisterState::Active => {
                // Already active
            },
            crate::tel::resource::model::register::RegisterState::Locked => {
                register.lock()?;
            },
            crate::tel::resource::model::register::RegisterState::Frozen => {
                register.freeze()?;
            },
            crate::tel::resource::model::register::RegisterState::Consumed => {
                // Mark as consumed - we can't call consume() directly as it requires a transaction ID,
                // so we'll directly set the state
                register.state = RegisterState::Consumed;
                register.consumed_by_tx = tel_register.consumed_by_tx.clone();
            },
            crate::tel::resource::model::register::RegisterState::Archived => {
                if let Some(ref_str) = &tel_register.archive_reference {
                    register.archive(ref_str)?;
                } else {
                    register.state = RegisterState::Archived;
                }
            },
            crate::tel::resource::model::register::RegisterState::Summary => {
                // Convert and set summarized registers
                if let Some(summarized) = &tel_register.summarizes {
                    let summarized_ids = summarized
                        .iter()
                        .map(|id| self.get_or_create_register_id(&id.to_string()))
                        .collect::<Result<Vec<_>>>()?;
                        
                    register.mark_as_summary(summarized_ids)?;
                } else {
                    register.state = RegisterState::Summary;
                }
            },
            crate::tel::resource::model::register::RegisterState::PendingDeletion => {
                register.mark_for_deletion()?;
            },
            crate::tel::resource::model::register::RegisterState::Tombstone => {
                // Can't call convert_to_tombstone directly as it requires the register to be in PendingDeletion state
                register.state = RegisterState::Tombstone;
            },
        }
        
        // Set the validity period
        register.set_validity(
            tel_register.validity.start,
            tel_register.validity.end,
        );
        
        Ok(register)
    }
    
    /// Convert a register to TEL register format
    pub fn convert_register_to_tel_register(
        &mut self,
        register: &Register,
    ) -> Result<TelRegister> {
        // Convert register contents
        let contents = self.convert_register_contents_to_tel(&register.contents)?;
        
        // Convert state
        let state = match register.state {
            RegisterState::Active => crate::tel::resource::model::register::RegisterState::Active,
            RegisterState::Locked => crate::tel::resource::model::register::RegisterState::Locked,
            RegisterState::Frozen => crate::tel::resource::model::register::RegisterState::Frozen,
            RegisterState::Consumed => crate::tel::resource::model::register::RegisterState::Consumed,
            RegisterState::Archived => crate::tel::resource::model::register::RegisterState::Archived,
            RegisterState::Summary => crate::tel::resource::model::register::RegisterState::Summary,
            RegisterState::PendingDeletion => crate::tel::resource::model::register::RegisterState::PendingDeletion,
            RegisterState::Tombstone => crate::tel::resource::model::register::RegisterState::Tombstone,
        };
        
        // Convert validity period
        let validity = crate::tel::resource::model::register::TimeRange {
            start: register.validity.start,
            end: register.validity.end,
        };
        
        // Convert metadata
        let metadata = self.convert_metadata_to_tel_metadata(&register.metadata);
        
        // Convert summarizes IDs if present
        let summarizes = register.summarizes.as_ref().map(|ids| {
            ids.iter()
                .map(|id| id.to_string().into())
                .collect::<Vec<_>>()
        });
        
        // Build the TEL register
        let tel_register = TelRegister {
            id: register.register_id.to_string().into(),
            owner: register.owner.clone(),
            domain: register.domain.clone(),
            contents,
            state,
            created_at: register.created_at,
            last_updated: register.last_updated,
            last_updated_height: register.last_updated_height,
            validity,
            epoch: register.epoch,
            created_by_tx: register.created_by_tx.clone(),
            consumed_by_tx: register.consumed_by_tx.clone(),
            successors: register.successors.iter()
                .map(|id| id.to_string().into())
                .collect(),
            summarizes,
            archive_reference: register.archive_reference.clone(),
            metadata,
        };
        
        Ok(tel_register)
    }
    
    /// Get or create a register ID based on a TEL register ID string
    fn get_or_create_register_id(&mut self, tel_id: &str) -> Result<RegisterId> {
        if let Some(id) = self.register_id_mapping.get(tel_id) {
            Ok(id.clone())
        } else {
            // For migration, try to parse the TEL ID as a UUID if possible
            let register_id = match uuid::Uuid::parse_str(tel_id) {
                Ok(uuid) => RegisterId::from_uuid(uuid),
                Err(_) => {
                    // If it's not a valid UUID, create a deterministic one based on the TEL ID
                    RegisterId::deterministic("tel_migration", tel_id)
                }
            };
            
            self.register_id_mapping.insert(tel_id.to_string(), register_id.clone());
            Ok(register_id)
        }
    }
    
    /// Convert TEL metadata to the new metadata format
    fn convert_tel_metadata_to_metadata(&self, tel_metadata: &TelMetadata) -> Metadata {
        let mut metadata = Metadata::new();
        
        for (key, value) in tel_metadata.iter() {
            metadata.insert(key.clone(), value.to_string());
        }
        
        metadata
    }
    
    /// Convert the new metadata format to TEL metadata
    fn convert_metadata_to_tel_metadata(&self, metadata: &Metadata) -> TelMetadata {
        let mut tel_metadata = TelMetadata::new();
        
        for (key, value) in metadata {
            tel_metadata.insert(key.clone(), value.clone());
        }
        
        tel_metadata
    }
}

/// Implement the RegistryAdapter trait for working with both old and new register systems
pub struct TelRegisterAdapter {
    /// The migration adapter for converting between register types
    adapter: RegisterMigrationAdapter,
    /// Cache of converted registers to avoid redundant conversions
    register_cache: HashMap<RegisterId, Register>,
}

impl TelRegisterAdapter {
    /// Create a new TEL register adapter
    pub fn new() -> Self {
        Self {
            adapter: RegisterMigrationAdapter::new(),
            register_cache: HashMap::new(),
        }
    }
    
    /// Import a TEL register into the new register system
    pub fn import_tel_register(&mut self, tel_register: &TelRegister) -> Result<Register> {
        let register = self.adapter.convert_tel_register_to_register(tel_register)?;
        
        // Cache the converted register
        self.register_cache.insert(register.register_id.clone(), register.clone());
        
        Ok(register)
    }
    
    /// Export a register to the TEL system
    pub fn export_register_to_tel(&mut self, register: &Register) -> Result<TelRegister> {
        self.adapter.convert_register_to_tel_register(register)
    }
    
    /// Get a register from the cache or convert it if not cached
    pub fn get_or_import_register(&mut self, tel_register: &TelRegister) -> Result<Register> {
        let id_str = tel_register.id.to_string();
        
        // Try to get the register ID from the mapping
        if let Some(register_id) = self.adapter.register_id_mapping.get(&id_str) {
            // If we have the ID in the mapping, check the cache
            if let Some(register) = self.register_cache.get(register_id) {
                return Ok(register.clone());
            }
        }
        
        // If not in cache, import and cache it
        self.import_tel_register(tel_register)
    }
    
    /// Clear the cache to free memory
    pub fn clear_cache(&mut self) {
        self.register_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_id_mapping() {
        let mut adapter = RegisterMigrationAdapter::new();
        
        // Test mapping the same ID twice returns the same RegisterId
        let id1 = adapter.get_or_create_register_id("test-id-1").unwrap();
        let id2 = adapter.get_or_create_register_id("test-id-1").unwrap();
        
        assert_eq!(id1, id2);
        
        // Test mapping different IDs returns different RegisterIds
        let id3 = adapter.get_or_create_register_id("test-id-2").unwrap();
        
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_convert_register_contents() {
        let adapter = RegisterMigrationAdapter::new();
        
        // Binary contents
        let binary_data = vec![1, 2, 3, 4];
        let tel_binary = TelRegisterContents::Binary(binary_data.clone());
        let unified_binary = adapter.convert_tel_register_contents(&tel_binary).unwrap();
        
        assert_eq!(
            unified_binary.as_binary().unwrap(),
            &binary_data
        );
        
        // String contents
        let string_data = "Hello, world!";
        let tel_string = TelRegisterContents::String(string_data.to_string());
        let unified_string = adapter.convert_tel_register_contents(&tel_string).unwrap();
        
        assert_eq!(
            unified_string.as_string().unwrap(),
            string_data
        );
        
        // JSON contents
        let json_data = serde_json::json!({"key": "value"});
        let tel_json = TelRegisterContents::Json(json_data.clone());
        let unified_json = adapter.convert_tel_register_contents(&tel_json).unwrap();
        
        assert!(unified_json.as_json().is_some());
    }
} 