//-----------------------------------------------------------------------------
// JSON Serialization Utilities
//-----------------------------------------------------------------------------

pub use causality_types::serialization::{Decode, DecodeError, Encode, SimpleSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::io::{Error as IoError, ErrorKind};

/// A wrapper around serde_json::Value that implements SSZ serialization.
/// This allows us to use serde_json::Value with SSZ serialization in a controlled way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SszJsonWrapper(pub JsonValue);

impl Encode for SszJsonWrapper {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        // Convert the JSON value to a string and serialize that string
        match serde_json::to_string(&self.0) {
            Ok(json_string) => json_string.into_bytes(),
            Err(_) => Vec::new(), // Empty vector on error
        }
    }
}

impl Decode for SszJsonWrapper {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        // Convert bytes to string
        let json_str = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return Err(causality_types::serialization::DecodeError {
                message: format!("Invalid UTF-8 in JSON bytes (length: {})", bytes.len())
            }),
        };
        
        // Parse the JSON string
        let json_value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => return Err(causality_types::serialization::DecodeError {
                message: format!("Invalid JSON: {}", e)
            }),
        };
        
        Ok(SszJsonWrapper(json_value))
    }
}

impl SimpleSerialize for SszJsonWrapper {}

// Conversion traits
impl From<JsonValue> for SszJsonWrapper {
    fn from(value: JsonValue) -> Self {
        SszJsonWrapper(value)
    }
}

impl From<SszJsonWrapper> for JsonValue {
    fn from(value: SszJsonWrapper) -> Self {
        value.0
    }
}

// Error conversion helpers
pub fn ssz_error_to_io_error(err: causality_types::serialization::DecodeError) -> IoError {
    IoError::new(ErrorKind::InvalidData, format!("SSZ decode error: {}", err.message))
}

pub fn io_error_to_ssz_error(err: IoError) -> causality_types::serialization::DecodeError {
    causality_types::serialization::DecodeError {
        message: format!("IO Error: {}", err),
    }
}
