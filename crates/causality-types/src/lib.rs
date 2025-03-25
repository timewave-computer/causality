// Core type definitions and basic data structures used throughout the system
// Original file: src/types.rs

// Common types used throughout the Causality system

use std::fmt;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json;
use borsh::{BorshSerialize, BorshDeserialize};

// Re-export only hash types that aren't defined in this file
use causality_crypto::{ContentAddressed, HashOutput, HashFactory, HashError, HashAlgorithm, HashFunction};

// Re-export ContentId from crypto::hash
pub use causality_crypto::ContentId;

// LamportTime definition for logical clock timestamps
pub type LamportTime = u64;

// Export all types for use throughout the codebase
pub use self::domain::DomainId;
pub use self::block::{BlockHash, BlockHeight};
pub use self::timestamp::Timestamp;
pub use self::content::{ContentHash};
pub use self::trace::TraceId;

/// Convert a byte vector to a fixed-size [u8; 32] array
/// 
/// This is commonly used for converting variable-length byte vectors to fixed-length
/// arrays required by cryptographic functions, block hashes, etc.
/// 
/// If the input vector is shorter than 32 bytes, the remaining bytes are filled with zeros.
/// If the input vector is longer than 32 bytes, only the first 32 bytes are used.
pub fn to_fixed_bytes(bytes: Vec<u8>) -> [u8; 32] {
    let mut result = [0u8; 32];
    let len = std::cmp::min(bytes.len(), 32);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

// Trace-related types
pub mod trace {
    use super::*;
    
    /// Trace identifier for tracing related operations
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct TraceId(pub String);
    
    /// Content type for trace ID generation
    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    struct TraceIdContent {
        /// Creation timestamp
        timestamp: i64,
        /// Random nonce for uniqueness
        nonce: [u8; 16],
        /// Optional parent trace ID
        parent: Option<String>,
        /// Optional operation name
        operation: Option<String>,
    }
    
    impl ContentAddressed for TraceIdContent {
        fn content_hash(&self) -> HashOutput {
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher().unwrap();
            let data = self.try_to_vec().unwrap();
            hasher.hash(&data)
        }
        
        fn verify(&self) -> bool {
            let hash = self.content_hash();
            let serialized = self.to_bytes();
            
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher().unwrap();
            hasher.hash(&serialized) == hash
        }
        
        fn to_bytes(&self) -> Vec<u8> {
            self.try_to_vec().unwrap()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            BorshDeserialize::try_from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    impl TraceId {
        /// Create a new trace ID with content-derived identifier
        pub fn new() -> Self {
            let content = TraceIdContent {
                timestamp: chrono::Utc::now().timestamp(),
                nonce: rand::random::<[u8; 16]>(),
                parent: None,
                operation: None,
            };
            
            let content_id = content.content_id();
            TraceId(format!("trace:{}", content_id))
        }
        
        /// Create a child trace ID from a parent trace ID
        pub fn child_of(parent: &TraceId, operation: Option<&str>) -> Self {
            let content = TraceIdContent {
                timestamp: chrono::Utc::now().timestamp(),
                nonce: rand::random::<[u8; 16]>(),
                parent: Some(parent.as_str().to_string()),
                operation: operation.map(|s| s.to_string()),
            };
            
            let content_id = content.content_id();
            TraceId(format!("trace:{}", content_id))
        }
        
        /// Create a trace ID from a string
        pub fn from_str(s: &str) -> Self {
            TraceId(s.to_string())
        }
        
        /// Get the trace ID as a string
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl Default for TraceId {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl fmt::Display for TraceId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    impl From<ContentId> for TraceId {
        fn from(content_id: ContentId) -> Self {
            Self(format!("trace:{}", content_id))
        }
    }
}

// Domain-related types
pub mod domain {
    use super::*;
    
    /// Domain identifier
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct DomainId(pub String);
    
    impl DomainId {
        /// Create a new DomainId
        pub fn new(id: impl Into<String>) -> Self {
            Self(id.into())
        }
        
        /// Get the string representation
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl fmt::Display for DomainId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    impl FromStr for DomainId {
        type Err = String;
        
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Valid domain IDs must be non-empty and contain valid chars
            if s.is_empty() {
                return Err("Domain ID cannot be empty".to_string());
            }
            
            // All alphanumeric plus hyphens and periods
            if !s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.') {
                return Err("Domain ID contains invalid characters".to_string());
            }
            
            Ok(DomainId(s.to_string()))
        }
    }
}

// Block-related types
pub mod block {
    use super::*;
    
    /// Block hash type
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct BlockHash(pub String);
    
    impl BlockHash {
        pub fn new(hash: &str) -> Self {
            BlockHash(hash.to_string())
        }
        
        pub fn as_str(&self) -> &str {
            &self.0
        }
        
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    
    impl Default for BlockHash {
        fn default() -> Self {
            BlockHash("".to_string())
        }
    }
    
    impl fmt::Display for BlockHash {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    /// Block height type
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct BlockHeight(pub u64);
    
    impl BlockHeight {
        pub fn new(height: u64) -> Self {
            BlockHeight(height)
        }
        
        pub fn value(&self) -> u64 {
            self.0
        }
    }
    
    impl Default for BlockHeight {
        fn default() -> Self {
            BlockHeight(0)
        }
    }
    
    impl fmt::Display for BlockHeight {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    // Additional operations for BlockHeight
    impl Add<u64> for BlockHeight {
        type Output = Self;
        
        fn add(self, rhs: u64) -> Self::Output {
            BlockHeight(self.0 + rhs)
        }
    }
    
    impl AddAssign<u64> for BlockHeight {
        fn add_assign(&mut self, rhs: u64) {
            self.0 += rhs;
        }
    }
    
    impl Sub<u64> for BlockHeight {
        type Output = Self;
        
        fn sub(self, rhs: u64) -> Self::Output {
            if self.0 < rhs {
                BlockHeight(0)
            } else {
                BlockHeight(self.0 - rhs)
            }
        }
    }
    
    impl SubAssign<u64> for BlockHeight {
        fn sub_assign(&mut self, rhs: u64) {
            if self.0 < rhs {
                self.0 = 0;
            } else {
                self.0 -= rhs;
            }
        }
    }
    
    impl Add<BlockHeight> for BlockHeight {
        type Output = Self;
        
        fn add(self, rhs: BlockHeight) -> Self::Output {
            BlockHeight(self.0 + rhs.0)
        }
    }
    
    impl Sub<BlockHeight> for BlockHeight {
        type Output = Self;
        
        fn sub(self, rhs: BlockHeight) -> Self::Output {
            if self.0 < rhs.0 {
                BlockHeight(0)
            } else {
                BlockHeight(self.0 - rhs.0)
            }
        }
    }
}

// Timestamp types
pub mod timestamp {
    use super::*;
    
    /// Timestamp type (in seconds since UNIX epoch)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Timestamp(pub u64);
    
    impl Timestamp {
        pub fn new(timestamp: u64) -> Self {
            Timestamp(timestamp)
        }
        
        pub fn now() -> Self {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Timestamp(now)
        }
        
        pub fn value(&self) -> u64 {
            self.0
        }

        pub fn as_i64(&self) -> i64 {
            self.0 as i64
        }
        
        /// Check if this timestamp is older than the given seconds
        pub fn is_older_than(&self, seconds: u64) -> bool {
            let now = Self::now();
            now.0.saturating_sub(self.0) > seconds
        }
        
        /// Get the difference between this timestamp and another
        pub fn difference(&self, other: &Timestamp) -> u64 {
            if self.0 > other.0 {
                self.0 - other.0
            } else {
                other.0 - self.0
            }
        }

        /// Get the timestamp value in milliseconds
        pub fn as_millis(&self) -> u64 {
            self.0 * 1000
        }
    }
    
    impl Default for Timestamp {
        fn default() -> Self {
            Timestamp(0)
        }
    }
    
    impl fmt::Display for Timestamp {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    // Additional operations for Timestamp
    impl Add<u64> for Timestamp {
        type Output = Self;
        
        fn add(self, rhs: u64) -> Self::Output {
            Timestamp(self.0 + rhs)
        }
    }
    
    impl AddAssign<u64> for Timestamp {
        fn add_assign(&mut self, rhs: u64) {
            self.0 += rhs;
        }
    }
    
    impl Sub<u64> for Timestamp {
        type Output = Self;
        
        fn sub(self, rhs: u64) -> Self::Output {
            if self.0 < rhs {
                Timestamp(0)
            } else {
                Timestamp(self.0 - rhs)
            }
        }
    }
    
    impl SubAssign<u64> for Timestamp {
        fn sub_assign(&mut self, rhs: u64) {
            if self.0 < rhs {
                self.0 = 0;
            } else {
                self.0 -= rhs;
            }
        }
    }
    
    impl Add<Timestamp> for Timestamp {
        type Output = Self;
        
        fn add(self, rhs: Timestamp) -> Self::Output {
            Timestamp(self.0 + rhs.0)
        }
    }
    
    impl Sub<Timestamp> for Timestamp {
        type Output = Self;
        
        fn sub(self, rhs: Timestamp) -> Self::Output {
            if self.0 < rhs.0 {
                Timestamp(0)
            } else {
                Timestamp(self.0 - rhs.0)
            }
        }
    }
}

// Content addressing types
pub mod content {
    use super::*;
    
    /// Content hash type
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ContentHash(pub String);
    
    impl ContentHash {
        /// Create a new content hash
        pub fn new(hash: &str) -> Self {
            ContentHash(hash.to_string())
        }
        
        /// Get the hash as a string slice
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl fmt::Display for ContentHash {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

/// An asset identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Asset(pub String);

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An amount of an asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount(pub u128);

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// State of a resource register in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegisterState {
    /// Initial state when created
    Initial,
    
    /// Active and usable
    Active,
    
    /// Temporarily locked
    Locked,
    
    /// Permanently frozen
    Frozen,
    
    /// Consumed (used up)
    Consumed,
    
    /// Pending consumption
    Pending,
    
    /// Archived (in long-term storage)
    Archived,
}

impl fmt::Display for RegisterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterState::Initial => write!(f, "Initial"),
            RegisterState::Active => write!(f, "Active"),
            RegisterState::Locked => write!(f, "Locked"),
            RegisterState::Frozen => write!(f, "Frozen"),
            RegisterState::Consumed => write!(f, "Consumed"),
            RegisterState::Pending => write!(f, "Pending"),
            RegisterState::Archived => write!(f, "Archived"),
        }
    }
}

/// A general purpose metadata struct for storing key-value pairs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// Internal storage for metadata key-value pairs
    pub values: HashMap<String, String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl Serialize for Metadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.values.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HashMap::<String, String>::deserialize(deserializer).map(|values| Metadata { values })
    }
}

impl Metadata {
    /// Create a new empty metadata container
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(key.into(), value.into())
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get the number of key-value pairs
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the metadata is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Represents a version string, typically a hash or incrementing number
pub type Version = String;

/// Represents the type of a domain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainType {
    /// A user domain
    User,
    
    /// A system domain
    System,
    
    /// A shared domain
    Shared,
    
    /// A temporary domain
    Temporary,
}

/// Represents a visibility level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Visible to everyone
    Public,
    
    /// Visible only to the owner
    Private,
    
    /// Visible to specific entities
    Restricted(Vec<String>),
}

/// Trait for extended type operations
pub trait TypeExtensions {
    /// Get a string representation of the type
    fn to_string_representation(&self) -> String;
    
    /// Get a stable hash of the type
    fn stable_hash(&self) -> u64;
}

// Implement TypeExtensions for the ContentId struct
impl TypeExtensions for ContentId {
    fn to_string_representation(&self) -> String {
        self.0.clone()
    }
    
    fn stable_hash(&self) -> u64 {
        // Simple hash function for example purposes
        let mut hash = 0;
        for byte in self.0.as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        hash
    }
}

// Use the DomainId struct from the domain module
// Avoid having two implementations for the same underlying type
impl TypeExtensions for domain::DomainId {
    fn to_string_representation(&self) -> String {
        self.as_str().to_string()
    }
    
    fn stable_hash(&self) -> u64 {
        // Simple hash function for example purposes
        let mut hash = 0;
        for byte in self.as_str().as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for to_fixed_bytes
    #[test]
    fn test_to_fixed_bytes_shorter() {
        let input = vec![1, 2, 3, 4];
        let expected = {
            let mut arr = [0u8; 32];
            arr[0] = 1;
            arr[1] = 2;
            arr[2] = 3;
            arr[3] = 4;
            arr
        };
        assert_eq!(to_fixed_bytes(input), expected);
    }

    #[test]
    fn test_to_fixed_bytes_equal() {
        let mut input = vec![0u8; 32];
        for i in 0..32 {
            input[i] = i as u8;
        }
        let expected = {
            let mut arr = [0u8; 32];
            for i in 0..32 {
                arr[i] = i as u8;
            }
            arr
        };
        assert_eq!(to_fixed_bytes(input), expected);
    }

    #[test]
    fn test_to_fixed_bytes_longer() {
        let mut input = vec![0u8; 40];
        for i in 0..40 {
            input[i] = i as u8;
        }
        let expected = {
            let mut arr = [0u8; 32];
            for i in 0..32 {
                arr[i] = i as u8;
            }
            arr
        };
        assert_eq!(to_fixed_bytes(input), expected);
    }

    #[test]
    fn test_resource_id_creation() {
        let id1 = ContentId::new("resource-1".to_string());
        let id2: ContentId = "resource-2".into();
        let id3: ContentId = String::from("resource-3").into();
        
        assert_eq!(id1.0, "resource-1");
        assert_eq!(id2.0, "resource-2");
        assert_eq!(id3.0, "resource-3");
    }
    
    #[test]
    fn test_domain_id_creation() {
        let id1 = DomainId("domain-1".to_string());
        let id2: DomainId = "domain-2".into();
        let id3: DomainId = String::from("domain-3").into();
        
        assert_eq!(id1.0, "domain-1");
        assert_eq!(id2.0, "domain-2");
        assert_eq!(id3.0, "domain-3");
    }
    
    #[test]
    fn test_register_state_display() {
        assert_eq!(format!("{}", RegisterState::Active), "Active");
        assert_eq!(format!("{}", RegisterState::Locked), "Locked");
        assert_eq!(format!("{}", RegisterState::Consumed), "Consumed");
    }
} 
