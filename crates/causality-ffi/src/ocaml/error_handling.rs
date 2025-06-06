//! Error handling utilities for OCaml FFI

#[cfg(feature = "ocaml-ffi")]
use ocaml::Value;

/// Standard error conversion for OCaml FFI
#[cfg(feature = "ocaml-ffi")]
pub fn ffi_error_to_string(error: impl std::fmt::Display) -> String {
    format!("FFI Error: {}", error)
}

/// Convert Rust error to OCaml exception
#[cfg(feature = "ocaml-ffi")]
pub fn raise_ffi_error(message: &str) -> Value {
    unsafe {
        let error_message = Value::string(message);
        Value::exception(&error_message)
    }
} 