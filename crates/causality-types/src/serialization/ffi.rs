//! FFI Serialization Utilities
//!
//! This module provides utilities for serializing and deserializing data
//! for use in foreign function interfaces (FFI). It is designed to be compatible
//! with other languages that might interact with the Causality framework,
//! particularly OCaml for ZK circuit integration.

use crate::serialization::{Decode, DecodeError, Encode};
use std::fmt;

/// Error type for FFI serialization
#[derive(Debug)]
pub struct FfiSerializationError {
    /// Error message
    pub message: String,
}

impl fmt::Display for FfiSerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FFI Serialization Error: {}", self.message)
    }
}

impl std::error::Error for FfiSerializationError {}

impl From<DecodeError> for FfiSerializationError {
    fn from(err: DecodeError) -> Self {
        FfiSerializationError {
            message: err.message,
        }
    }
}

impl From<hex::FromHexError> for FfiSerializationError {
    fn from(err: hex::FromHexError) -> Self {
        FfiSerializationError {
            message: format!("Hex decode error: {}", err),
        }
    }
}

/// Serialize a value for FFI use
///
/// This function serializes a value using SSZ and returns a byte vector
/// that can be passed across FFI boundaries.
///
/// # Arguments
///
/// * `value` - The value to serialize
///
/// # Returns
///
/// A byte vector containing the serialized value
pub fn serialize_for_ffi<T: Encode>(value: &T) -> Vec<u8> {
    value.as_ssz_bytes()
}

/// Deserialize a value from FFI data
///
/// This function deserializes a value from a byte slice that was passed
/// across FFI boundaries.
///
/// # Arguments
///
/// * `bytes` - The byte slice containing the serialized value
///
/// # Returns
///
/// The deserialized value
pub fn deserialize_from_ffi<T: Decode>(bytes: &[u8]) -> Result<T, FfiSerializationError> {
    T::from_ssz_bytes(bytes).map_err(|e| e.into())
}

/// Serialize a value to a hex string for FFI use
///
/// This function serializes a value using SSZ and returns a hex-encoded
/// string that can be passed across FFI boundaries.
///
/// # Arguments
///
/// * `value` - The value to serialize
///
/// # Returns
///
/// A hex-encoded string containing the serialized value
pub fn serialize_to_hex<T: Encode>(value: &T) -> String {
    let bytes = serialize_for_ffi(value);
    hex::encode(bytes)
}

/// Deserialize a value from a hex string
///
/// This function deserializes a value from a hex-encoded string that was
/// passed across FFI boundaries.
///
/// # Arguments
///
/// * `hex_str` - The hex-encoded string containing the serialized value
///
/// # Returns
///
/// The deserialized value
pub fn deserialize_from_hex<T: Decode>(hex_str: &str) -> Result<T, FfiSerializationError> {
    let bytes = hex::decode(hex_str)?;
    deserialize_from_ffi(&bytes)
}

/// Helper function to handle errors in FFI context
///
/// This function is useful for FFI functions that need to return a result
/// without propagating errors.
///
/// # Arguments
///
/// * `result` - The result to handle
/// * `error_handler` - A function to call with the error message if the result is an error
///
/// # Returns
///
/// The unwrapped value if the result is Ok, or the default value if the result is Err
pub fn handle_ffi_result<T, F>(result: Result<T, FfiSerializationError>, error_handler: F, default: T) -> T
where
    F: FnOnce(String),
{
    match result {
        Ok(value) => value,
        Err(err) => {
            error_handler(err.message);
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::SimpleSerialize;

    // Simple test type
    #[derive(Debug, PartialEq, Clone)]
    struct TestType {
        value: u32,
    }

    impl Encode for TestType {
        fn as_ssz_bytes(&self) -> Vec<u8> {
            self.value.as_ssz_bytes()
        }
    }

    impl Decode for TestType {
        fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
            let value = u32::from_ssz_bytes(bytes)?;
            Ok(TestType { value })
        }
    }

    impl SimpleSerialize for TestType {}

    #[test]
    fn test_ffi_serialization() {
        let original = TestType { value: 42 };
        
        // Test binary serialization
        let serialized = serialize_for_ffi(&original);
        let deserialized = deserialize_from_ffi::<TestType>(&serialized).unwrap();
        assert_eq!(deserialized, original);
        
        // Test hex serialization
        let hex = serialize_to_hex(&original);
        let deserialized = deserialize_from_hex::<TestType>(&hex).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_ffi_error_handling() {
        // Test with invalid hex string
        let result = deserialize_from_hex::<TestType>("not a hex string");
        assert!(result.is_err());
        
        // Test error handler
        let error_message = String::new();
        let default = TestType { value: 0 };
        let result = handle_ffi_result(
            Ok(TestType { value: 42 }),
            |_| {},
            default.clone()
        );
        
        assert_eq!(result, TestType { value: 42 });
        assert!(error_message.is_empty());
    }
} 