// Common types used throughout the Causality system

use std::fmt;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json;

// Export all types for use throughout the codebase
pub use self::domain::DomainId;
pub use self::block::{BlockHash, BlockHeight};
pub use self::timestamp::Timestamp;
pub use self::content::{ContentHash, ContentId};
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
    
    impl TraceId {
        /// Create a new trace ID with a random UUID
        pub fn new() -> Self {
            TraceId(Uuid::new_v4().to_string())
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
    
    #[cfg(feature = "content-addressing")]
    use blake3::Hasher;
    #[cfg(feature = "content-addressing")]
    use hex;
    
    /// Content hash representing a Blake3 hash of content
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ContentHash(pub String);
    
    impl ContentHash {
        /// Create a new content hash from a string
        pub fn new(hash: &str) -> Self {
            ContentHash(hash.to_string())
        }
        
        /// Get the hash as a string
        pub fn as_str(&self) -> &str {
            &self.0
        }
        
        /// Check if the hash is empty
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        
        /// Create a content hash from data
        #[cfg(feature = "content-addressing")]
        pub fn from_data<T: AsRef<[u8]>>(data: T) -> Self {
            let mut hasher = Hasher::new();
            hasher.update(data);
            let result = hasher.finalize();
            let hash_str = hex::encode(result.as_bytes());
            ContentHash(hash_str)
        }
        
        /// Create a content hash from a string
        #[cfg(feature = "content-addressing")]
        pub fn from_string(data: &str) -> Self {
            Self::from_data(data.as_bytes())
        }
        
        /// Create a content hash by combining multiple hashes
        #[cfg(feature = "content-addressing")]
        pub fn combine(hashes: &[ContentHash]) -> Self {
            let mut hasher = Hasher::new();
            for hash in hashes {
                hasher.update(hash.as_str().as_bytes());
            }
            let result = hasher.finalize();
            let hash_str = hex::encode(result.as_bytes());
            ContentHash(hash_str)
        }
    }
    
    impl Default for ContentHash {
        fn default() -> Self {
            ContentHash("".to_string())
        }
    }
    
    impl fmt::Display for ContentHash {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    /// Content ID type representing a domain-specific content identifier
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ContentId {
        /// The content hash
        pub hash: ContentHash,
        /// The content type
        pub content_type: String,
        /// The domain ID (optional)
        pub domain_id: Option<String>,
        /// Additional metadata
        pub metadata: HashMap<String, String>,
    }
    
    impl ContentId {
        /// Create a new content ID
        pub fn new(hash: ContentHash, content_type: &str) -> Self {
            ContentId {
                hash,
                content_type: content_type.to_string(),
                domain_id: None,
                metadata: HashMap::new(),
            }
        }
        
        /// Set the domain ID
        pub fn with_domain(mut self, domain_id: &str) -> Self {
            self.domain_id = Some(domain_id.to_string());
            self
        }
        
        /// Add metadata
        pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
            self.metadata.insert(key.to_string(), value.to_string());
            self
        }
        
        /// Get a canonical string representation
        pub fn canonical(&self) -> String {
            let mut parts = vec![self.hash.as_str().to_string()];
            
            parts.push(self.content_type.clone());
            
            if let Some(domain) = &self.domain_id {
                parts.push(domain.clone());
            }
            
            let mut metadata_parts: Vec<String> = self.metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            
            metadata_parts.sort();
            parts.extend(metadata_parts);
            
            parts.join(":")
        }
        
        /// Create from canonical string
        pub fn from_canonical(s: &str) -> Option<Self> {
            let parts: Vec<&str> = s.split(':').collect();
            
            if parts.len() < 2 {
                return None;
            }
            
            let hash = ContentHash::new(parts[0]);
            let content_type = parts[1].to_string();
            
            let mut result = ContentId::new(hash, &content_type);
            
            if parts.len() > 2 && !parts[2].contains('=') {
                result.domain_id = Some(parts[2].to_string());
            }
            
            for i in 2..parts.len() {
                let part = parts[i];
                if part.contains('=') {
                    if let Some((key, value)) = part.split_once('=') {
                        result.metadata.insert(key.to_string(), value.to_string());
                    }
                }
            }
            
            Some(result)
        }
    }
    
    impl fmt::Display for ContentId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.canonical())
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

/// Resource identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ResourceId {
    fn from(s: String) -> Self {
        ResourceId(s)
    }
}

impl From<&str> for ResourceId {
    fn from(s: &str) -> Self {
        ResourceId(s.to_string())
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(ResourceId)
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

// Re-export time types from time module
pub use crate::time::LamportTime;

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

/// Represents an ID for a resource
pub type ResourceId = String;

/// Represents an ID for a domain
pub type DomainId = String;

/// Represents a trace ID for tracking operations
pub type TraceId = String;

/// Represents a point in time (Unix timestamp)
pub type Timestamp = u64;

/// Metadata is a collection of key-value pairs
pub type Metadata = HashMap<String, serde_json::Value>;

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

/// Extension methods for types
pub trait TypeExtensions {
    /// Get a string representation of the type
    fn to_string_representation(&self) -> String;
    
    /// Get a stable hash of the type
    fn stable_hash(&self) -> u64;
}

/// Implementation of TypeExtensions for ResourceId
impl TypeExtensions for ResourceId {
    fn to_string_representation(&self) -> String {
        format!("resource:{}", self)
    }
    
    fn stable_hash(&self) -> u64 {
        // Simple hash function for example purposes
        let mut hash = 0;
        for byte in self.as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        hash
    }
}

/// Implementation of TypeExtensions for DomainId
impl TypeExtensions for DomainId {
    fn to_string_representation(&self) -> String {
        format!("domain:{}", self)
    }
    
    fn stable_hash(&self) -> u64 {
        // Simple hash function for example purposes
        let mut hash = 0;
        for byte in self.as_bytes() {
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
        let id1 = ResourceId("resource-1".to_string());
        let id2: ResourceId = "resource-2".into();
        let id3: ResourceId = String::from("resource-3").into();
        
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