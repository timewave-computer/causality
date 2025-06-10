//! Error handling for OCaml FFI
//!
//! This module provides utilities for converting Rust errors to OCaml exceptions
//! and handling error propagation across the FFI boundary.

use ocaml::Value;

/// Convert an error message to an OCaml exception
pub fn raise_ffi_error(message: &str) -> Value {
    unsafe {
        let error_message = message.to_string();
        // Return a simple value that represents an error
        // In a real implementation, you would construct a proper OCaml exception
        Value::int(0) // Placeholder error representation
    }
}

/// Convert a Result to an OCaml value (Success or Error)
pub fn result_to_ocaml<T>(result: Result<T, String>) -> Value 
where
    T: ocaml::ToValue,
{
    match result {
        Ok(value) => {
            // For now, just return the value directly
            // In a full implementation, you would wrap this in a Result variant
            unsafe { Value::int(1) } // Placeholder success
        },
        Err(error) => {
            raise_ffi_error(&error)
        }
    }
}

/// Standard error conversion for OCaml FFI
#[cfg(feature = "ocaml-ffi")]
pub fn ffi_error_to_string(error: impl std::fmt::Display) -> String {
    format!("FFI Error: {}", error)
} 