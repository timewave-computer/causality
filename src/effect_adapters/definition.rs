// Definition module for Content-Addressable Code System
//
// This module provides the core types for code definitions in the content-addressed system.

use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::effect_adapters::hash::Hash as ContentHash;

/// Represents the content of a code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeContent {
    /// Raw bytes
    Bytes(Vec<u8>),
    /// JSON-encoded data
    Json(String),
    /// RISC-V binary
    RiscV(Vec<u8>),
    /// Abstract Syntax Tree (AST) representation
    Ast(serde_json::Value),
    /// WebAssembly binary
    Wasm(Vec<u8>),
}

/// A code definition with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDefinition {
    /// The content hash of this definition
    pub hash: ContentHash,
    /// The human-readable name (if any)
    pub name: Option<String>,
    /// The actual code representation (AST or bytecode)
    pub content: CodeContent,
    /// Dependencies of this code definition
    pub dependencies: Vec<ContentHash>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CodeDefinition {
    /// Create a new code definition
    pub fn new(
        hash: ContentHash, 
        name: Option<String>, 
        content: CodeContent, 
        dependencies: Vec<ContentHash>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        CodeDefinition {
            hash,
            name,
            content,
            dependencies,
            metadata,
        }
    }
    
    /// Create a new code definition builder
    pub fn builder() -> CodeDefinitionBuilder {
        CodeDefinitionBuilder::new()
    }
    
    /// Get the hash of this definition
    pub fn hash(&self) -> &ContentHash {
        &self.hash
    }
    
    /// Get the name of this definition
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// Check if this code has dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }
    
    /// Get the dependencies for this code
    pub fn dependencies(&self) -> &[ContentHash] {
        &self.dependencies
    }
    
    /// Get a metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Builder for code definitions
pub struct CodeDefinitionBuilder {
    name: Option<String>,
    content: Option<CodeContent>,
    dependencies: Vec<ContentHash>,
    metadata: HashMap<String, serde_json::Value>,
}

impl CodeDefinitionBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        CodeDefinitionBuilder {
            name: None,
            content: None,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set the content
    pub fn with_content(mut self, content: CodeContent) -> Self {
        self.content = Some(content);
        self
    }
    
    /// Add a dependency
    pub fn with_dependency(mut self, dependency: ContentHash) -> Self {
        self.dependencies.push(dependency);
        self
    }
    
    /// Add multiple dependencies
    pub fn with_dependencies(mut self, dependencies: Vec<ContentHash>) -> Self {
        self.dependencies.extend(dependencies);
        self
    }
    
    /// Add a metadata field
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
    
    /// Build the code definition
    pub fn build(self, hasher: &dyn crate::effect_adapters::hash::ContentHasher) -> Result<CodeDefinition, String> {
        let content = self.content.ok_or_else(|| "Content is required".to_string())?;
        
        // Create a hashable representation for the content
        let content_bytes = match &content {
            CodeContent::Bytes(bytes) => bytes.clone(),
            CodeContent::Json(json) => json.as_bytes().to_vec(),
            CodeContent::RiscV(binary) => binary.clone(),
            CodeContent::Ast(ast) => serde_json::to_vec(ast)
                .map_err(|e| format!("Failed to serialize AST: {}", e))?,
            CodeContent::Wasm(wasm) => wasm.clone(),
        };
        
        // Calculate the hash based on content and dependencies
        let hash = hasher.hash_bytes(&content_bytes);
        
        Ok(CodeDefinition {
            hash,
            name: self.name,
            content,
            dependencies: self.dependencies,
            metadata: self.metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_adapters::hash::{Blake3ContentHasher, ContentHasher};
    
    #[test]
    fn test_code_definition_creation() {
        let hasher = Blake3ContentHasher::new();
        let content = CodeContent::Json(r#"{"type": "function", "body": "return x + y;"}"#.to_string());
        
        let def = CodeDefinitionBuilder::new()
            .with_name("add")
            .with_content(content)
            .with_metadata("author", serde_json::Value::String("test".to_string()))
            .build(&hasher)
            .unwrap();
        
        assert_eq!(def.name(), Some("add"));
        assert!(def.get_metadata("author").is_some());
        assert!(!def.has_dependencies());
    }
    
    #[test]
    fn test_dependencies() {
        let hasher = Blake3ContentHasher::new();
        
        // Create a dependency
        let dep_content = CodeContent::Bytes(vec![1, 2, 3, 4]);
        let dep = CodeDefinitionBuilder::new()
            .with_name("dependency")
            .with_content(dep_content)
            .build(&hasher)
            .unwrap();
        
        // Create a definition with a dependency
        let content = CodeContent::Bytes(vec![5, 6, 7, 8]);
        let def = CodeDefinitionBuilder::new()
            .with_name("main")
            .with_content(content)
            .with_dependency(dep.hash().clone())
            .build(&hasher)
            .unwrap();
        
        assert!(def.has_dependencies());
        assert_eq!(def.dependencies().len(), 1);
        assert_eq!(def.dependencies()[0], *dep.hash());
    }
} 