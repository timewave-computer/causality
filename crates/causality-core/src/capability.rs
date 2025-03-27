// Capability model implementations
//
// This module provides types and functions for capability-based security,
// with a focus on content addressing and cryptographic verification.

use std::fmt::{self, Display, Debug};
use std::str::FromStr;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use hex::{FromHex, ToHex};

/// Content hash for content addressing
///
/// A secure cryptographic hash that uniquely identifies content
/// and can be used for content addressing.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentHash {
    /// Raw hash bytes (SHA-256 by default)
    bytes: Vec<u8>,
}

impl ContentHash {
    /// Create a new content hash from raw bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    
    /// Create a content hash from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { bytes: bytes.to_vec() }
    }
    
    /// Create a content hash from a string
    pub fn from_string(s: &str) -> Result<Self, String> {
        let bytes = Vec::from_hex(s).map_err(|e| format!("Invalid hex string: {}", e))?;
        Ok(Self { bytes })
    }
    
    /// Create a content hash by hashing content
    pub fn hash_content(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        Self { bytes: result.to_vec() }
    }
    
    /// Get the raw bytes of this hash
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    
    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        self.bytes.encode_hex::<String>()
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        // Default to all zeros
        Self { bytes: vec![0; 32] }
    }
}

impl Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self.to_hex())
    }
}

impl FromStr for ContentHash {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ContentHash::from_string(s)
    }
}

/// Content addressable trait
///
/// A trait for types that can be content-addressed.
pub trait ContentAddressable {
    /// Get the content hash of this object
    fn content_hash(&self) -> ContentHash;
    
    /// Verify that this object's content matches a given hash
    fn verify_content(&self, expected_hash: &ContentHash) -> bool {
        let actual_hash = self.content_hash();
        &actual_hash == expected_hash
    }
}

/// Capability set for authorization
///
/// A collection of capabilities that can be used to authorize operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// The capabilities in this set
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    /// Create a new empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }
    
    /// Add a capability to this set
    pub fn add(&mut self, capability: Capability) {
        self.capabilities.push(capability);
    }
    
    /// Check if this set contains a capability
    pub fn contains(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Check if this set allows an operation
    pub fn allows(&self, operation: &Operation) -> bool {
        self.capabilities.iter().any(|cap| cap.allows(operation))
    }
}

/// Capability for authorization
///
/// A capability represents the ability to perform an operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// The target of this capability
    target: CapabilityTarget,
    
    /// The operations this capability allows
    operations: Vec<Operation>,
    
    /// The constraints on this capability
    constraints: Vec<Constraint>,
    
    /// The attestation for this capability
    attestation: Option<Attestation>,
}

impl Capability {
    /// Create a new capability
    pub fn new(target: CapabilityTarget, operations: Vec<Operation>) -> Self {
        Self {
            target,
            operations,
            constraints: Vec::new(),
            attestation: None,
        }
    }
    
    /// Check if this capability allows an operation
    pub fn allows(&self, operation: &Operation) -> bool {
        self.operations.contains(operation)
    }
    
    /// Add a constraint to this capability
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    
    /// Set the attestation for this capability
    pub fn set_attestation(&mut self, attestation: Attestation) {
        self.attestation = Some(attestation);
    }
}

/// Capability target
///
/// The target of a capability, which can be a specific object,
/// a type of object, or a pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityTarget {
    /// A specific object identified by its content hash
    Object(ContentHash),
    
    /// A type of object
    Type(String),
    
    /// A pattern that matches objects
    Pattern(String),
    
    /// All objects
    All,
}

/// Operation
///
/// An operation that can be performed on an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Read an object
    Read,
    
    /// Write to an object
    Write,
    
    /// Delete an object
    Delete,
    
    /// Create an object
    Create,
    
    /// Custom operation
    Custom(String),
}

/// Constraint
///
/// A constraint on a capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constraint {
    /// Require that an attribute has a specific value
    RequireAttribute { name: String, value: String },
    
    /// Require that a time is before a specific timestamp
    RequireBefore(u64),
    
    /// Require that a time is after a specific timestamp
    RequireAfter(u64),
    
    /// Require all constraints to be satisfied
    All(Vec<Constraint>),
    
    /// Require any constraint to be satisfied
    Any(Vec<Constraint>),
}

/// Attestation
///
/// An attestation that a capability is valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    /// The issuer of this attestation
    issuer: String,
    
    /// The signature of this attestation
    signature: Vec<u8>,
    
    /// The time this attestation was issued
    timestamp: u64,
}

impl Attestation {
    /// Create a new attestation
    pub fn new(issuer: String, signature: Vec<u8>, timestamp: u64) -> Self {
        Self {
            issuer,
            signature,
            timestamp,
        }
    }
    
    /// Get the issuer of this attestation
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    
    /// Get the signature of this attestation
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
    
    /// Get the timestamp of this attestation
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
} 