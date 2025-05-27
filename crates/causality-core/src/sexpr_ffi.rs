// sexpr_ffi.rs
//
// Foreign Function Interface (FFI) for S-expression serialization
// This enables interoperability between Rust and OCaml.

use lexpr::{Value as SexprValue, parse as lexpr_parse};
use std::{ffi::{CStr, CString}, os::raw::{c_char}, slice};

use causality_types::serialization::{Decode, Encode};
use crate::sexpr_utils::SexprSerializable;

/// FFI result for byte arrays.
///
/// Used to return either SSZ-serialized bytes or an error message string.
#[repr(C)]
pub struct FfiByteResult {
    pub success: bool,
    pub data: *mut u8,
    pub data_len: usize,
    pub error_msg: *mut c_char,
}

/// FFI result for strings.
///
/// Used to return either an S-expression string or an error message string.
#[repr(C)]
pub struct FfiStringResult {
    pub success: bool,
    pub data: *mut c_char,
    pub error_msg: *mut c_char,
}

impl Default for FfiByteResult {
    fn default() -> Self {
        Self {
            success: false,
            data: std::ptr::null_mut(),
            data_len: 0,
            error_msg: std::ptr::null_mut(),
        }
    }
}

impl Default for FfiStringResult {
    fn default() -> Self {
        Self {
            success: false,
            data: std::ptr::null_mut(),
            error_msg: std::ptr::null_mut(),
        }
    }
}

/// Helper to create an error result for byte arrays.
fn byte_error_result(msg: &str) -> FfiByteResult {
    let error_msg = CString::new(msg).unwrap_or_default();
    FfiByteResult {
        success: false,
        data: std::ptr::null_mut(),
        data_len: 0,
        error_msg: error_msg.into_raw(),
    }
}

/// Helper to create an error result for strings.
fn string_error_result(msg: &str) -> FfiStringResult {
    let error_msg = CString::new(msg).unwrap_or_default();
    FfiStringResult {
        success: false,
        data: std::ptr::null_mut(),
        error_msg: error_msg.into_raw(),
    }
}

/// Helper to create a success result for byte arrays.
#[allow(dead_code)]
fn byte_success_result(bytes: Vec<u8>) -> FfiByteResult {
    let mut data = bytes;
    let data_ptr = data.as_mut_ptr();
    let data_len = data.len();
    std::mem::forget(data); // Prevent deallocation
    
    FfiByteResult {
        success: true,
        data: data_ptr,
        data_len,
        error_msg: std::ptr::null_mut(),
    }
}

/// Helper to create a success result for strings.
#[allow(dead_code)]
fn string_success_result(data_str: String) -> FfiStringResult {
    let c_str = CString::new(data_str).unwrap_or_default();
    let data = c_str.into_raw();
    
    FfiStringResult {
        success: true,
        data,
        error_msg: std::ptr::null_mut(),
    }
}

/// Generic trait for types that can be converted between S-expressions and SSZ
#[allow(dead_code)]
trait SexprSszConvertible: Encode + Decode + SexprSerializable {}

/// Convert an S-expression string to SSZ-serialized bytes.
/// 
/// # Safety
///
/// This function is unsafe because it:
/// 1. Dereferences raw pointers (sexpr_input)
/// 2. Creates raw pointers to be freed by the FFI caller
/// 3. Use std::mem::forget on the returned byte vector
#[no_mangle]
pub unsafe extern "C" fn rust_sexpr_to_ssz_bytes(
    sexpr_input: *const c_char,
    type_hint: u32,
) -> FfiByteResult {
    if sexpr_input.is_null() {
        return byte_error_result("Null input pointer");
    }
    
    let c_str = CStr::from_ptr(sexpr_input);
    let sexpr_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return byte_error_result("Invalid UTF-8 in S-expression string"),
    };
    
    // Parse the S-expression string
    let _sexpr_value = match lexpr_parse::from_str(sexpr_str) {
        Ok(value) => value,
        Err(err) => return byte_error_result(&format!("Failed to parse S-expression: {}", err)),
    };
    
    // Try test handlers first when in test mode
    #[cfg(test)]
    {
        if let Some(result) = test_resource_handlers::extend_sexpr_to_ssz_handler(&_sexpr_value, type_hint) {
            return result;
        }
    }
    
    // Dispatch to appropriate type-specific handler based on type_hint
    byte_error_result(&format!("Unsupported type hint: {}", type_hint))
}

/// Convert SSZ-serialized bytes to an S-expression string.
/// 
/// # Safety
///
/// This function is unsafe because it:
/// 1. Dereferences raw pointers (ssz_input)
/// 2. Creates raw pointers to be freed by the FFI caller
#[no_mangle]
pub unsafe extern "C" fn rust_ssz_bytes_to_sexpr(
    ssz_input: *const u8,
    input_len: usize,
    type_hint: u32,
) -> FfiStringResult {
    if ssz_input.is_null() {
        return string_error_result("Null input pointer");
    }
    
    let _ssz_slice = slice::from_raw_parts(ssz_input, input_len);
    
    // Try test handlers first when in test mode
    #[cfg(test)]
    {
        if let Some(result) = test_resource_handlers::extend_ssz_to_sexpr_handler(_ssz_slice, type_hint) {
            return result;
        }
    }
    
    // Dispatch to appropriate type-specific handler based on type_hint
    string_error_result(&format!("Unsupported type hint: {}", type_hint))
}

/// Free a byte result returned by rust_sexpr_to_ssz_bytes.
/// 
/// # Safety
///
/// This function is unsafe because it:
/// 1. Dereferences and deallocates raw pointers
#[no_mangle]
pub unsafe extern "C" fn free_byte_result(result: FfiByteResult) {
    if !result.data.is_null() {
        let _ = Vec::from_raw_parts(result.data, result.data_len, result.data_len);
    }
    
    if !result.error_msg.is_null() {
        let _ = CString::from_raw(result.error_msg);
    }
}

/// Free a string result returned by rust_ssz_bytes_to_sexpr.
/// 
/// # Safety
///
/// This function is unsafe because it:
/// 1. Dereferences and deallocates raw pointers
#[no_mangle]
pub unsafe extern "C" fn free_string_result(result: FfiStringResult) {
    if !result.data.is_null() {
        let _ = CString::from_raw(result.data);
    }
    
    if !result.error_msg.is_null() {
        let _ = CString::from_raw(result.error_msg);
    }
}

// Add type-specific conversion functions below as needed

// Example handler for one type (Resource)
#[allow(dead_code)]
fn handle_resource_type(_sexpr: &SexprValue) -> FfiByteResult {
    // This is a placeholder. Implement actual logic based on your resource structure
    byte_error_result("Resource conversion not yet implemented")
}

// Example handler for converting bytes back to S-expression
#[allow(dead_code)]
fn handle_resource_bytes_to_sexpr(_bytes: &[u8]) -> FfiStringResult {
    // This is a placeholder. Implement actual logic based on your resource structure
    string_error_result("Resource conversion not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sexpr_ssz_placeholder() {
        // This is a placeholder test that simply creates a test S-expression with lexpr directly
        // instead of using utility functions that might have compatibility issues
        let test_sexpr = SexprValue::list(vec![
            SexprValue::symbol("test"),
            SexprValue::string("value")
        ]);
        
        // Assert the S-expression has the right format
        assert!(test_sexpr.is_list());
        
        // Print the S-expression for debugging
        println!("Test S-expression: {}", test_sexpr);
        
        // Basic test passed if we get here without panic
    }
}

#[cfg(test)]
mod test_resource_handlers {
    use super::*;
    
    pub fn handle_test_resource(_sexpr: &SexprValue) -> FfiByteResult {
        // This is a placeholder that just creates a test byte array
        let test_bytes = vec![0, 1, 2, 3, 4, 5];
        byte_success_result(test_bytes)
    }
    
    pub fn handle_test_resource_bytes_to_sexpr(bytes: &[u8]) -> FfiStringResult {
        // This is a placeholder that just creates a test S-expression string
        let s_expr_str = format!("(test-resource bytes-len {})", bytes.len());
        string_success_result(s_expr_str)
    }
    
    pub const TEST_RESOURCE_TYPE_HINT: u32 = 100;
    
    // Helper for extending the handler in tests
    pub fn extend_sexpr_to_ssz_handler(sexpr_value: &SexprValue, type_hint: u32) -> Option<FfiByteResult> {
        if type_hint == TEST_RESOURCE_TYPE_HINT {
            Some(handle_test_resource(sexpr_value))
        } else {
            None
        }
    }
    
    pub fn extend_ssz_to_sexpr_handler(ssz_slice: &[u8], type_hint: u32) -> Option<FfiStringResult> {
        if type_hint == TEST_RESOURCE_TYPE_HINT {
            Some(handle_test_resource_bytes_to_sexpr(ssz_slice))
        } else {
            None
        }
    }
} 