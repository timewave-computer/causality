// Effect content handling
// Original file: src/effect/content.rs

// Content module for Effect System
//
// This module provides types for content-addressable effects and code.
// It replaces the older effect_adapters hash module with standardized types.

use std::fmt;
use std::convert::TryFrom;
use std::str::FromStr;
use std::collections::HashMap;

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use blake3::Hasher as Blake3Hasher;
use hex;

/// The type of hash algorithm used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// Blake3 hash algorithm
    Blake3,
    /// Poseidon hash algorithm (not yet implemented)
    #[allow(dead_code)]
    Poseidon,
}

impl std::hash::Hash for HashAlgorithm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl HashAlgorithm {
    /// Get the digest size of the algorithm in bytes
    pub fn digest_size(&self) -> usize {
        match self {
            HashAlgorithm::Blake3 => 32, // 256 bits
            HashAlgorithm::Poseidon => 32, // 256 bits
        }
    }
    
    /// Convert to a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::Blake3 => "blake3",
            HashAlgorithm::Poseidon => "poseidon",
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(HashAlgorithm::Blake3),
            "poseidon" => Ok(HashAlgorithm::Poseidon),
            _ => Err(format!("Unknown hash algorithm: {}", s)),
        }
    }
}

/// A content hash with algorithm information
#[derive(Clone, PartialEq, Eq)]
pub struct ContentHash {
    /// The hash algorithm used
    pub algorithm: HashAlgorithm,
    /// The hash bytes
    pub bytes: Vec<u8>,
}

impl ContentHash {
    /// Create a new content hash with the given algorithm and bytes
    pub fn new(algorithm: HashAlgorithm, bytes: Vec<u8>) -> Self {
        Self { algorithm, bytes }
    }
    
    /// Create a Blake3 hash from bytes
    pub fn blake3(data: &[u8]) -> Self {
        let mut hasher = Blake3Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        Self {
            algorithm: HashAlgorithm::Blake3,
            bytes: hash.as_bytes().to_vec(),
        }
    }
    
    /// Create a hash from a string using Blake3
    pub fn from_string(s: &str) -> Self {
        Self::blake3(s.as_bytes())
    }
    
    /// Convert the hash to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
    
    /// Check if this is an empty/default hash
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.as_str(), self.to_hex())
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.as_str(), self.to_hex())
    }
}

impl Serialize for ContentHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.algorithm.as_str(), self.to_hex()))
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ContentHash::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for ContentHash {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check format
        if !s.contains(':') {
            return Err(format!("Invalid hash format: {}", s));
        }
        
        // Split algorithm and hash
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid hash format: {}", s));
        }
        
        // Parse algorithm
        let algorithm = match parts[0] {
            "blake3" => HashAlgorithm::Blake3,
            "poseidon" => HashAlgorithm::Poseidon,
            _ => return Err(format!("Unknown hash algorithm: {}", parts[0])),
        };
        
        // Parse hash
        let hash_hex = parts[1];
        let hash_bytes = match hex::decode(hash_hex) {
            Ok(bytes) => bytes,
            Err(e) => return Err(format!("Invalid hex in hash: {}", e)),
        };
        
        Ok(ContentHash {
            algorithm,
            bytes: hash_bytes,
        })
    }
}

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
    /// Create a new code definition builder
    pub fn builder() -> CodeDefinitionBuilder {
        CodeDefinitionBuilder::new()
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
        Self {
            name: None,
            content: None,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the name
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set the content
    pub fn content(mut self, content: CodeContent) -> Self {
        self.content = Some(content);
        self
    }
    
    /// Add a dependency
    pub fn dependency(mut self, hash: ContentHash) -> Self {
        self.dependencies.push(hash);
        self
    }
    
    /// Add dependencies
    pub fn dependencies(mut self, hashes: Vec<ContentHash>) -> Self {
        self.dependencies.extend(hashes);
        self
    }
    
    /// Add metadata
    pub fn metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
    
    /// Build the code definition
    pub fn build(self) -> Result<CodeDefinition, String> {
        let content = self.content.ok_or_else(|| "Content is required".to_string())?;
        
        // Serialize content to calculate hash
        let content_bytes = match &content {
            CodeContent::Bytes(bytes) => bytes.clone(),
            CodeContent::Json(json) => json.as_bytes().to_vec(),
            CodeContent::RiscV(bytes) => bytes.clone(),
            CodeContent::Ast(ast) => serde_json::to_vec(ast).map_err(|e| e.to_string())?,
            CodeContent::Wasm(bytes) => bytes.clone(),
        };
        
        let hash = ContentHash::blake3(&content_bytes);
        
        Ok(CodeDefinition {
            hash,
            name: self.name,
            content,
            dependencies: self.dependencies,
            metadata: self.metadata,
        })
    }
} 