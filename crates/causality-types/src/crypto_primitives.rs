// Core cryptographic primitives and types
// Contains fundamental types previously in causality-crypto/src/hash.rs

use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};

/// Error type for hash operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum HashError {
    /// Invalid hash format
    #[error("Invalid hash format")]
    InvalidFormat,
    
    /// Invalid hash length
    #[error("Invalid hash length")]
    InvalidLength,
    
    /// Unsupported hash algorithm
    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedAlgorithm(String),
    
    /// Internal error during hashing
    #[error("Internal hash error: {0}")]
    InternalError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Supported hash algorithms
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
    Sha512,
    Custom(String),
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::Blake3
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blake3 => write!(f, "Blake3"),
            Self::Sha256 => write!(f, "Sha256"),
            Self::Sha512 => write!(f, "Sha512"),
            Self::Custom(name) => write!(f, "Custom:{}", name),
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(Self::Blake3),
            "sha256" => Ok(Self::Sha256),
            "sha512" => Ok(Self::Sha512),
            _ if s.starts_with("custom:") => Ok(Self::Custom(s[7..].to_string())),
            _ => Err(format!("Unknown hash algorithm: {}", s)),
        }
    }
}

/// Output of a hash function with algorithm awareness
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct HashOutput {
    /// The raw bytes of the hash
    data: [u8; 32],
    /// The algorithm used to generate this hash
    algorithm: HashAlgorithm,
}

impl HashOutput {
    /// Create a new hash output from raw bytes with the specified algorithm
    pub fn new(data: [u8; 32], algorithm: HashAlgorithm) -> Self {
        Self { data, algorithm }
    }
    
    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the algorithm used to generate this hash
    pub fn algorithm(&self) -> &HashAlgorithm {
        &self.algorithm
    }
    
    /// Convert the hash output to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.data)
    }
    
    /// Create a hash output from a hex string with the specified algorithm
    pub fn from_hex(hex_str: &str, algorithm: HashAlgorithm) -> Result<Self, HashError> {
        let bytes = hex::decode(hex_str)
            .map_err(|_| HashError::InvalidFormat)?;
        
        if bytes.len() != 32 {
            return Err(HashError::InvalidLength);
        }
        
        let mut data = [0u8; 32];
        data.copy_from_slice(&bytes);
        Ok(Self::new(data, algorithm))
    }
    
    /// Convert the hash output to a hex string with algorithm prefix
    pub fn to_hex_string(&self) -> String {
        format!("{}:{}", self.algorithm.to_string().to_lowercase(), self.to_hex())
    }
    
    /// Create a hash output from a hex string with algorithm prefix
    pub fn from_hex_string(s: &str) -> Result<Self, HashError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(HashError::InvalidFormat);
        }
        
        let algorithm = HashAlgorithm::from_str(parts[0])
            .map_err(|_| HashError::UnsupportedAlgorithm(parts[0].to_string()))?;
        
        Self::from_hex(parts[1], algorithm)
    }
    
    /// Get a deterministic unique identifier for this hash
    pub fn to_content_id(&self) -> ContentId {
        ContentId::from(self.clone())
    }
}

impl fmt::Display for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// A content-derived identifier replacing UUID for object identification
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentId(HashOutput);

impl ContentId {
    /// Create a new ContentId from a hash output
    pub fn from(hash: HashOutput) -> Self {
        Self(hash)
    }
    
    /// Create a zero-value ContentId for use as a placeholder
    pub fn nil() -> Self {
        let zero_bytes = [0u8; 32];
        Self(HashOutput::new(zero_bytes, HashAlgorithm::default()))
    }
    
    /// Get the underlying hash output
    pub fn hash(&self) -> &HashOutput {
        &self.0
    }
    
    /// Get raw bytes from the content id
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        format!("cid:{}", self.0.to_hex_string())
    }
    
    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, HashError> {
        if let Some(hex) = s.strip_prefix("cid:") {
            let hash = HashOutput::from_hex_string(hex)?;
            Ok(Self(hash))
        } else {
            Err(HashError::InvalidFormat)
        }
    }

    /// Create a new ContentId from a string
    pub fn new(id: impl Into<String>) -> Self {
        // Convert to string first
        let id_str = id.into();
        
        // If it looks like a properly formatted ContentId, try to parse it
        if id_str.starts_with("cid:") {
            if let Ok(content_id) = Self::parse(&id_str) {
                return content_id;
            }
        }
        
        // Otherwise, treat it as raw data and hash it
        ContentId::from_bytes(id_str.as_bytes())
    }
    
    /// Create a ContentId directly from bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        // Create a simple hash for testing purposes - not for production use
        let mut bytes = [0u8; 32];
        for (i, &b) in data.iter().enumerate().take(32) {
            bytes[i] = b;
        }
        Self(HashOutput::new(bytes, HashAlgorithm::Blake3))
    }
    
    /// Unwrap a ContentId from_bytes operation
    pub fn from_bytes_unwrap(data: &[u8]) -> Self {
        Self::from_bytes(data)
    }
    
    /// Try to convert this ContentId to a core ContentHash
    pub fn to_core_content_hash(&self) -> Result<ContentHash, HashError> {
        let hash_output = self.hash();
        Ok(ContentHash::from_hash_output(hash_output))
    }
    
    /// Create a ContentId from a core ContentHash
    pub fn from_core_content_hash(hash: &ContentHash) -> Result<Self, HashError> {
        let hash_output = hash.to_hash_output()?;
        Ok(Self::from(hash_output))
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

// Add From implementations needed for tests
impl From<String> for ContentId {
    fn from(s: String) -> Self {
        ContentId::new(s)
    }
}

impl From<&str> for ContentId {
    fn from(s: &str) -> Self {
        ContentId::new(s)
    }
}

impl From<&[u8]> for ContentId {
    fn from(data: &[u8]) -> Self {
        Self::from_bytes(data)
    }
}

impl FromStr for ContentId {
    type Err = HashError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // If it looks like a properly formatted ContentId with the cid: prefix
        if s.starts_with("cid:") {
            return Self::parse(s);
        }
        
        // Try to parse as a hash output string directly (algorithm:hex_string)
        if s.contains(':') {
            if let Ok(hash) = HashOutput::from_hex_string(s) {
                return Ok(Self(hash));
            }
        }
        
        // Assume it's a raw hex string with the default algorithm
        let bytes = hex::decode(s).map_err(|_| HashError::InvalidFormat)?;
        if bytes.len() != 32 {
            return Err(HashError::InvalidLength);
        }
        
        let mut data = [0u8; 32];
        data.copy_from_slice(&bytes);
        Ok(Self(HashOutput::new(data, HashAlgorithm::default())))
    }
}

/// Trait for content addressing support
pub trait ContentAddressed {
    /// Get the content hash of this object
    fn content_hash(&self) -> Result<HashOutput, HashError>;
    
    /// Get a deterministic identifier derived from content
    fn content_id(&self) -> Result<ContentId, HashError> {
        let hash = self.content_hash()?;
        Ok(ContentId::from(hash))
    }
    
    /// Verify that the object matches its hash
    fn verify(&self, expected_hash: &HashOutput) -> Result<bool, HashError> {
        let actual_hash = self.content_hash()?;
        Ok(actual_hash == *expected_hash)
    }
    
    /// Convert to a serialized form for storage
    fn to_bytes(&self) -> Result<Vec<u8>, HashError>;
    
    /// Create from serialized form
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> where Self: Sized;
}

/// Content hash with algorithm information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    /// The algorithm used for hashing
    pub algorithm: String,
    /// The raw hash bytes
    pub bytes: Vec<u8>,
}

impl ContentHash {
    /// Create a new content hash with the specified algorithm and bytes
    pub fn new(algorithm: &str, bytes: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            bytes,
        }
    }
    
    /// Create a ContentHash from a HashOutput
    pub fn from_hash_output(hash_output: &HashOutput) -> Self {
        let algorithm = hash_output.algorithm().to_string();
        let bytes = hash_output.as_bytes().to_vec();
        Self::new(&algorithm, bytes)
    }
    
    /// Try to convert to a HashOutput
    pub fn to_hash_output(&self) -> Result<HashOutput, HashError> {
        let algorithm = HashAlgorithm::from_str(&self.algorithm)
            .map_err(|_| HashError::UnsupportedAlgorithm(self.algorithm.clone()))?;
            
        if self.bytes.len() != 32 {
            return Err(HashError::InvalidLength);
        }
        
        let mut data = [0u8; 32];
        data.copy_from_slice(&self.bytes);
        
        Ok(HashOutput::new(data, algorithm))
    }
    
    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
    
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        format!("{}:{}", self.algorithm.to_lowercase(), self.to_hex())
    }
    
    /// Create a ContentHash from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, HashError> {
        // Create a simple hash for testing purposes
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(hash.as_bytes());
        
        Ok(Self::new("blake3", bytes))
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Extension trait to add unwrap() for Result<ContentId, E>
pub trait ContentIdResultExt<E> {
    fn unwrap(self) -> ContentId;
}

impl<E> ContentIdResultExt<E> for Result<ContentId, E> {
    fn unwrap(self) -> ContentId {
        match self {
            Ok(id) => id,
            Err(_) => panic!("Failed to unwrap ContentId"),
        }
    }
}

/// Represents a digital signature.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Signature {
    /// The signature data.
    pub data: Vec<u8>, // Assuming SignatureData is Vec<u8> or similar Borsh-compatible type
    /// The algorithm used for the signature.
    pub algorithm: SignatureAlgorithm,
}

/// Supported signature algorithms.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    Secp256k1,
    Custom(String),
} 