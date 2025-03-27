// Serialization Utilities
//
// This module provides common serialization helpers and utilities that are
// shared across the codebase. It defines interfaces for consistent serialization
// and deserialization of various data types.

use std::any::Any;
use serde::{Serialize, Deserialize};
use crate::error::{Error, Result};

/// Trait for types that can be serialized to bytes
pub trait ToBytes {
    /// Convert a value to its binary representation
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

/// Trait for types that can be deserialized from bytes
pub trait FromBytes: Sized {
    /// Convert from binary representation to a value
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

/// Implement ToBytes for any type that implements Serialize
impl<T: Serialize> ToBytes for T {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| Error::serialization(format!("Failed to serialize: {}", e)))
    }
}

/// Implement FromBytes for any type that implements Deserialize
impl<'de, T: Deserialize<'de>> FromBytes for T {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| Error::serialization(format!("Failed to deserialize: {}", e)))
    }
}

/// Trait for types that support content-based addressing
pub trait ContentAddressable {
    /// Generate a content-based identifier for this object
    fn content_id(&self) -> Result<String>;
}

/// Trait for schema-aware serialization
pub trait SchemaSerialize: Serialize {
    /// Get the schema identifier for this type
    fn schema_id() -> &'static str;
    
    /// Get the schema version for this type
    fn schema_version() -> &'static str;
    
    /// Serialize with schema information
    fn serialize_with_schema(&self) -> Result<SchemaEnvelope> {
        let data = self.to_bytes()?;
        Ok(SchemaEnvelope {
            schema_id: Self::schema_id().to_string(),
            schema_version: Self::schema_version().to_string(),
            data,
        })
    }
}

/// Container for schema-aware serialized data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEnvelope {
    /// Identifier for the schema
    pub schema_id: String,
    /// Version of the schema
    pub schema_version: String,
    /// Serialized data
    pub data: Vec<u8>,
}

impl SchemaEnvelope {
    /// Deserialize the contained data to the specified type
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        T::from_bytes(&self.data)
    }
    
    /// Verify that the envelope contains the expected schema
    pub fn verify_schema<T: SchemaSerialize>(&self) -> bool {
        self.schema_id == T::schema_id() && self.schema_version == T::schema_version()
    }
}

/// Helper for type-erased serialization
pub trait SerializeAny {
    /// Convert to an Any trait object
    fn as_any(&self) -> &dyn Any;
    
    /// Serialize to bytes
    fn serialize_any(&self) -> Result<Vec<u8>>;
}

impl<T: Serialize + 'static> SerializeAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn serialize_any(&self) -> Result<Vec<u8>> {
        self.to_bytes()
    }
} 