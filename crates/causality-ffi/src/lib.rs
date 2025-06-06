//! Unified FFI bindings for Causality framework
//!
//! This crate provides both C-compatible FFI interfaces and OCaml-specific bindings
//! for core Causality types, enabling seamless integration with multiple languages
//! through SSZ serialization and native bindings.

#![warn(missing_docs)]
// FFI operations require unsafe code for pointer manipulation
#![allow(unsafe_code)]

// Common modules
pub mod types;
pub mod value;
pub mod serialization;
pub mod error;

// C FFI modules (enabled by default)
#[cfg(feature = "c-ffi")]
pub mod c_interface;

// OCaml FFI modules (optional)
#[cfg(feature = "ocaml-ffi")]
pub mod ocaml;

// Re-exports for specific features
#[cfg(feature = "c-ffi")]
pub use c_interface::*;

#[cfg(feature = "ocaml-ffi")]
pub use ocaml::*;

// Common exports
pub use error::*;

use std::ffi::CString;
use std::os::raw::c_char;

//-----------------------------------------------------------------------------
// Error Handling
//-----------------------------------------------------------------------------

/// FFI result type
pub type FfiResult<T> = Result<T, FfiError>;

/// FFI error codes for external bindings
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FfiErrorCode {
    /// Operation succeeded
    Success = 0,
    /// Invalid input parameter
    InvalidInput = 1,
    /// Serialization failed
    SerializationError = 2,
    /// Deserialization failed
    DeserializationError = 3,
    /// Memory allocation/deallocation error
    MemoryError = 4,
    /// Internal system error
    InternalError = 5,
}

/// FFI error type
#[repr(C)]
pub struct FfiError {
    /// Error code
    pub code: FfiErrorCode,
    /// Error message (caller must free with causality_free_string)
    pub message: *mut c_char,
}

impl FfiError {
    #[allow(dead_code)]
    fn new(code: FfiErrorCode, message: &str) -> Self {
        let c_message = match CString::new(message) {
            Ok(s) => s.into_raw(),
            Err(_) => std::ptr::null_mut(),
        };
        
        Self {
            code,
            message: c_message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    
    #[cfg(feature = "c-ffi")]
    #[test]
    fn test_ffi_value_creation() {
        use crate::c_interface::*;
        
        // Test unit value
        let unit_ptr = causality_value_unit();
        assert!(!unit_ptr.is_null());
        causality_value_free(unit_ptr);
        
        // Test boolean value
        let bool_ptr = causality_value_bool(1);
        assert!(!bool_ptr.is_null());
        let bool_type = causality_value_type(bool_ptr);
        assert_eq!(bool_type, ValueType::Bool);
        let bool_val = causality_value_as_bool(bool_ptr);
        assert_eq!(bool_val, 1);
        causality_value_free(bool_ptr);
        
        // Test integer value
        let int_ptr = causality_value_int(42);
        assert!(!int_ptr.is_null());
        let int_type = causality_value_type(int_ptr);
        assert_eq!(int_type, ValueType::Int);
        let int_val = causality_value_as_int(int_ptr);
        assert_eq!(int_val, 42);
        causality_value_free(int_ptr);
    }
    
    #[cfg(feature = "c-ffi")]
    #[test]
    fn test_ffi_string_handling() {
        use crate::c_interface::*;
        use std::ffi::CString;
        
        let test_string = CString::new("Hello, World!").unwrap();
        let string_ptr = causality_value_string(test_string.as_ptr());
        assert!(!string_ptr.is_null());
        
        let string_type = causality_value_type(string_ptr);
        assert_eq!(string_type, ValueType::String);
        
        let extracted_string = causality_value_as_string(string_ptr);
        assert!(!extracted_string.is_null());
        
        let extracted_cstr = unsafe { CStr::from_ptr(extracted_string) };
        assert_eq!(extracted_cstr.to_str().unwrap(), "Hello, World!");
        
        causality_free_string(extracted_string);
        causality_value_free(string_ptr);
    }
    
    #[cfg(feature = "c-ffi")]
    #[test]
    fn test_ffi_serialization() {
        use crate::c_interface::*;
        
        let value_ptr = causality_value_int(12345);
        assert!(!value_ptr.is_null());
        
        // Test serialization
        let serialization_result = causality_value_serialize(value_ptr);
        assert_eq!(serialization_result.error_code, FfiErrorCode::Success);
        assert!(!serialization_result.data.is_null());
        assert!(serialization_result.length > 0);
        
        // Test deserialization
        let deserialized_ptr = causality_value_deserialize(
            serialization_result.data,
            serialization_result.length,
        );
        assert!(!deserialized_ptr.is_null());
        
        // Check values are equal
        let original_val = causality_value_as_int(value_ptr);
        let deserialized_val = causality_value_as_int(deserialized_ptr);
        assert_eq!(original_val, deserialized_val);
        
        // Cleanup
        causality_free_serialized_data(
            serialization_result.data,
            serialization_result.length,
        );
        causality_value_free(value_ptr);
        causality_value_free(deserialized_ptr);
    }
    
    #[cfg(feature = "c-ffi")]
    #[test]
    fn test_ffi_roundtrip() {
        use crate::c_interface::*;
        
        let test_values = vec![
            (causality_value_unit(), "unit"),
            (causality_value_bool(1), "bool true"),
            (causality_value_bool(0), "bool false"),
            (causality_value_int(0), "int 0"),
            (causality_value_int(42), "int 42"),
            (causality_value_int(u32::MAX), "int max"),
        ];
        
        for (value_ptr, description) in test_values {
            assert!(!value_ptr.is_null(), "Failed to create {}", description);
            
            let roundtrip_result = causality_test_roundtrip(value_ptr);
            assert_eq!(roundtrip_result, 1, "Roundtrip failed for {}", description);
            
            causality_value_free(value_ptr);
        }
    }
    
    #[cfg(feature = "c-ffi")]
    #[test]
    fn test_comprehensive_roundtrip() {
        use crate::c_interface::*;
        
        let result = causality_test_all_roundtrips();
        assert_eq!(result, 1, "Comprehensive roundtrip test failed");
    }
} 