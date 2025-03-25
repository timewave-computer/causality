// Effect repository for storage and retrieval
// Original file: src/effect/repository.rs

// Repository module for Content-Addressable Effects
//
// This module provides the repository for storing, retrieving, and managing
// content-addressable code and effects.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use causality_effects::{ContentHash, CodeDefinition, CodeContent};

/// Entry in the code repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntry {
    /// The code definition
    pub definition: CodeDefinition,
    /// Metadata about this entry
    pub metadata: CodeMetadata,
}

/// Metadata about a code entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    /// When the code was added to the repository
    pub added_at: chrono::DateTime<chrono::Utc>,
    /// Tags for this code
    pub tags: Vec<String>,
    /// Versions available
    pub versions: Vec<String>,
    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

impl CodeMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self {
            added_at: chrono::Utc::now(),
            tags: Vec::new(),
            versions: Vec::new(),
            properties: HashMap::new(),
        }
    }
    
    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    /// Add a version
    pub fn with_version(mut self, version: &str) -> Self {
        self.versions.push(version.to_string());
        self
    }
    
    /// Add a property
    pub fn with_property(mut self, key: &str, value: serde_json::Value) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }
}

/// Repository for content-addressable code
#[async_trait]
pub trait CodeRepository: Send + Sync {
    /// Add a code definition to the repository
    async fn add_code(&self, definition: CodeDefinition, metadata: Option<CodeMetadata>) -> Result<ContentHash>;
    
    /// Get a code definition by its hash
    async fn get_code(&self, hash: &ContentHash) -> Result<Option<CodeEntry>>;
    
    /// Check if a code definition exists
    async fn has_code(&self, hash: &ContentHash) -> Result<bool>;
    
    /// List all code definitions
    async fn list_all(&self) -> Result<Vec<CodeEntry>>;
    
    /// Find code by tag
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<CodeEntry>>;
    
    /// Find code by name
    async fn find_by_name(&self, name: &str) -> Result<Vec<CodeEntry>>;
}

/// In-memory repository implementation
pub struct InMemoryCodeRepository {
    entries: RwLock<HashMap<ContentHash, CodeEntry>>,
}

impl InMemoryCodeRepository {
    /// Create a new in-memory repository
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CodeRepository for InMemoryCodeRepository {
    async fn add_code(&self, definition: CodeDefinition, metadata: Option<CodeMetadata>) -> Result<ContentHash> {
        let hash = definition.hash.clone();
        let metadata = metadata.unwrap_or_else(CodeMetadata::new);
        
        let entry = CodeEntry {
            definition,
            metadata,
        };
        
        let mut entries = self.entries.write().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        entries.insert(hash.clone(), entry);
        
        Ok(hash)
    }
    
    async fn get_code(&self, hash: &ContentHash) -> Result<Option<CodeEntry>> {
        let entries = self.entries.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        Ok(entries.get(hash).cloned())
    }
    
    async fn has_code(&self, hash: &ContentHash) -> Result<bool> {
        let entries = self.entries.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        Ok(entries.contains_key(hash))
    }
    
    async fn list_all(&self) -> Result<Vec<CodeEntry>> {
        let entries = self.entries.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        Ok(entries.values().cloned().collect())
    }
    
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<CodeEntry>> {
        let entries = self.entries.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        
        let matching = entries.values()
            .filter(|entry| entry.metadata.tags.iter().any(|t| t == tag))
            .cloned()
            .collect();
            
        Ok(matching)
    }
    
    async fn find_by_name(&self, name: &str) -> Result<Vec<CodeEntry>> {
        let entries = self.entries.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        
        let matching = entries.values()
            .filter(|entry| {
                if let Some(entry_name) = &entry.definition.name {
                    entry_name.contains(name)
                } else {
                    false
                }
            })
            .cloned()
            .collect();
            
        Ok(matching)
    }
}

/// File-based repository implementation
pub struct FileCodeRepository {
    base_path: PathBuf,
    index: RwLock<HashMap<ContentHash, PathBuf>>,
}

impl FileCodeRepository {
    /// Create a new file-based repository
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;
        
        let index_path = base_path.join("index.json");
        let index = if index_path.exists() {
            let index_data = fs::read_to_string(&index_path)?;
            serde_json::from_str(&index_data)?
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            base_path,
            index: RwLock::new(index),
        })
    }
    
    /// Save the index to disk
    fn save_index(&self) -> Result<()> {
        let index = self.index.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        let index_data = serde_json::to_string(&*index)?;
        let index_path = self.base_path.join("index.json");
        fs::write(index_path, index_data)?;
        Ok(())
    }
}

#[async_trait]
impl CodeRepository for FileCodeRepository {
    async fn add_code(&self, definition: CodeDefinition, metadata: Option<CodeMetadata>) -> Result<ContentHash> {
        let hash = definition.hash.clone();
        let metadata = metadata.unwrap_or_else(CodeMetadata::new);
        
        let entry = CodeEntry {
            definition,
            metadata,
        };
        
        let entry_data = serde_json::to_string(&entry)?;
        let hash_hex = hash.to_hex();
        let file_path = self.base_path.join(format!("{}.json", hash_hex));
        
        fs::write(&file_path, entry_data)?;
        
        let mut index = self.index.write().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        index.insert(hash.clone(), file_path);
        drop(index);
        
        self.save_index()?;
        
        Ok(hash)
    }
    
    async fn get_code(&self, hash: &ContentHash) -> Result<Option<CodeEntry>> {
        let index = self.index.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        
        if let Some(file_path) = index.get(hash) {
            if !file_path.exists() {
                return Ok(None);
            }
            
            let data = fs::read_to_string(file_path)?;
            let entry: CodeEntry = serde_json::from_str(&data)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
    
    async fn has_code(&self, hash: &ContentHash) -> Result<bool> {
        let index = self.index.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        Ok(index.contains_key(hash))
    }
    
    async fn list_all(&self) -> Result<Vec<CodeEntry>> {
        let index = self.index.read().map_err(|_| Error::Internal("Lock acquisition failed".into()))?;
        
        let mut entries = Vec::new();
        for file_path in index.values() {
            if file_path.exists() {
                let data = fs::read_to_string(file_path)?;
                let entry: CodeEntry = serde_json::from_str(&data)?;
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<CodeEntry>> {
        let all_entries = self.list_all().await?;
        
        Ok(all_entries
            .into_iter()
            .filter(|entry| entry.metadata.tags.iter().any(|t| t == tag))
            .collect())
    }
    
    async fn find_by_name(&self, name: &str) -> Result<Vec<CodeEntry>> {
        let all_entries = self.list_all().await?;
        
        Ok(all_entries
            .into_iter()
            .filter(|entry| {
                if let Some(entry_name) = &entry.definition.name {
                    entry_name.contains(name)
                } else {
                    false
                }
            })
            .collect())
    }
} 