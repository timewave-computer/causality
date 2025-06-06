//! C-compatible FFI interface for Causality framework
//!
//! This module provides extern "C" functions for interoperability with C and other
//! languages that can call C functions, including Python, JavaScript (via Node.js), etc.

use crate::FfiErrorCode;
use causality_core::lambda::Value;
use causality_core::system::serialization::{SszEncode, SszDecode};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};

//-----------------------------------------------------------------------------
// Core Value FFI Interface
//-----------------------------------------------------------------------------

/// Opaque handle to a Causality Value
#[repr(C)]
pub struct CausalityValue {
    _private: [u8; 0],
}

/// Create a unit value
#[no_mangle]
pub extern "C" fn causality_value_unit() -> *mut CausalityValue {
    let value = Value::Unit;
    Box::into_raw(Box::new(value)) as *mut CausalityValue
}

/// Create a boolean value
#[no_mangle]
pub extern "C" fn causality_value_bool(b: c_int) -> *mut CausalityValue {
    let value = Value::Bool(b != 0);
    Box::into_raw(Box::new(value)) as *mut CausalityValue
}

/// Create an integer value
#[no_mangle]
pub extern "C" fn causality_value_int(i: c_uint) -> *mut CausalityValue {
    let value = Value::Int(i);
    Box::into_raw(Box::new(value)) as *mut CausalityValue
}

/// Create a string value
#[no_mangle]
pub extern "C" fn causality_value_string(s: *const c_char) -> *mut CausalityValue {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str = unsafe { CStr::from_ptr(s) };
    match c_str.to_str() {
        Ok(rust_str) => {
            let value = Value::String(causality_core::system::Str::new(rust_str));
            Box::into_raw(Box::new(value)) as *mut CausalityValue
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Create a symbol value
#[no_mangle]
pub extern "C" fn causality_value_symbol(s: *const c_char) -> *mut CausalityValue {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str = unsafe { CStr::from_ptr(s) };
    match c_str.to_str() {
        Ok(rust_str) => {
            let value = Value::Symbol(causality_core::system::Str::new(rust_str));
            Box::into_raw(Box::new(value)) as *mut CausalityValue
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a Causality value
#[no_mangle]
pub extern "C" fn causality_value_free(value: *mut CausalityValue) {
    if !value.is_null() {
        unsafe {
            let _ = Box::from_raw(value as *mut Value);
        }
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization FFI Interface
//-----------------------------------------------------------------------------

/// Serialization result
#[repr(C)]
pub struct SerializationResult {
    pub data: *mut u8,
    pub length: usize,
    pub error_code: FfiErrorCode,
    pub error_message: *mut c_char,
}

impl SerializationResult {
    fn success(data: Vec<u8>) -> Self {
        let length = data.len();
        let data_ptr = data.as_ptr() as *mut u8;
        std::mem::forget(data); // Prevent deallocation - caller must free
        
        Self {
            data: data_ptr,
            length,
            error_code: FfiErrorCode::Success,
            error_message: std::ptr::null_mut(),
        }
    }
    
    fn error(code: FfiErrorCode, message: &str) -> Self {
        let c_message = CString::new(message).unwrap_or_else(|_| {
            CString::new("Failed to create error message").unwrap()
        });
        
        Self {
            data: std::ptr::null_mut(),
            length: 0,
            error_code: code,
            error_message: c_message.into_raw(),
        }
    }
}

/// Serialize a Causality value to SSZ bytes
#[no_mangle]
pub extern "C" fn causality_value_serialize(value: *const CausalityValue) -> SerializationResult {
    if value.is_null() {
        return SerializationResult::error(
            FfiErrorCode::InvalidInput, 
            "Null value pointer"
        );
    }

    let rust_value = unsafe { &*(value as *const Value) };
    
    let len = rust_value.ssz_bytes_len();
    let mut bytes = Vec::with_capacity(len);
    rust_value.ssz_append(&mut bytes);
    
    SerializationResult::success(bytes)
}

/// Deserialize SSZ bytes to a Causality value
#[no_mangle]
pub extern "C" fn causality_value_deserialize(
    data: *const u8,
    length: usize,
) -> *mut CausalityValue {
    if data.is_null() || length == 0 {
        return std::ptr::null_mut();
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, length) };
    
    match Value::from_ssz_bytes(bytes) {
        Ok(value) => Box::into_raw(Box::new(value)) as *mut CausalityValue,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free serialized data
#[no_mangle]
pub extern "C" fn causality_free_serialized_data(data: *mut u8, length: usize) {
    if !data.is_null() && length > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(data, length, length);
        }
    }
}

/// Free error message
#[no_mangle]
pub extern "C" fn causality_free_error_message(message: *mut c_char) {
    if !message.is_null() {
        unsafe {
            let _ = CString::from_raw(message);
        }
    }
}

//-----------------------------------------------------------------------------
// Value Inspection
//-----------------------------------------------------------------------------

/// Value type enumeration for C interface
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Unit = 0,
    Bool = 1,
    Int = 2,
    Symbol = 3,
    String = 4,
    Product = 5,
    Sum = 6,
    Record = 7,
}

/// Get the type of a Causality value
#[no_mangle]
pub extern "C" fn causality_value_type(value: *const CausalityValue) -> ValueType {
    if value.is_null() {
        return ValueType::Unit;
    }
    
    let rust_value = unsafe { &*(value as *const Value) };
    match rust_value {
        Value::Unit => ValueType::Unit,
        Value::Bool(_) => ValueType::Bool,
        Value::Int(_) => ValueType::Int,
        Value::Symbol(_) => ValueType::Symbol,
        Value::String(_) => ValueType::String,
        Value::Product(_, _) => ValueType::Product,
        Value::Sum { tag: _, value: _ } => ValueType::Sum,
        Value::Record { fields: _ } => ValueType::Record,
    }
}

/// Extract boolean value (returns 0 for false, 1 for true, -1 for error)
#[no_mangle]
pub extern "C" fn causality_value_as_bool(value: *const CausalityValue) -> c_int {
    if value.is_null() {
        return -1;
    }
    
    let rust_value = unsafe { &*(value as *const Value) };
    match rust_value {
        Value::Bool(b) => if *b { 1 } else { 0 },
        _ => -1,
    }
}

/// Extract integer value (returns 0 for error cases)
#[no_mangle]
pub extern "C" fn causality_value_as_int(value: *const CausalityValue) -> c_uint {
    if value.is_null() {
        return 0;
    }
    
    let rust_value = unsafe { &*(value as *const Value) };
    match rust_value {
        Value::Int(i) => *i,
        _ => 0,
    }
}

/// Extract string value (caller must free with causality_free_string)
#[no_mangle]
pub extern "C" fn causality_value_as_string(value: *const CausalityValue) -> *mut c_char {
    if value.is_null() {
        return std::ptr::null_mut();
    }
    
    let rust_value = unsafe { &*(value as *const Value) };
    match rust_value {
        Value::String(s) => {
            match CString::new(s.value.as_str()) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Value::Symbol(s) => {
            match CString::new(s.value.as_str()) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        _ => std::ptr::null_mut(),
    }
}

/// Free a string returned by causality_value_as_string
#[no_mangle]
pub extern "C" fn causality_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

//-----------------------------------------------------------------------------
// Round-trip Testing
//-----------------------------------------------------------------------------

/// Test round-trip serialization for a value (returns 1 for success, 0 for failure)
#[no_mangle]
pub extern "C" fn causality_test_roundtrip(value: *const CausalityValue) -> c_int {
    if value.is_null() {
        return 0;
    }

    let original = unsafe { &*(value as *const Value) };
    
    // Serialize
    let len = original.ssz_bytes_len();
    let mut bytes = Vec::with_capacity(len);
    original.ssz_append(&mut bytes);
    
    // Deserialize
    match Value::from_ssz_bytes(&bytes) {
        Ok(deserialized) => {
            if *original == deserialized { 1 } else { 0 }
        }
        Err(_) => 0,
    }
}

/// Test all round-trip serializations for basic types
#[no_mangle]
pub extern "C" fn causality_test_all_roundtrips() -> c_int {
    let test_values = vec![
        Value::Unit,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(42),
        Value::String(causality_core::system::Str::new("test")),
        Value::Symbol(causality_core::system::Str::new("symbol")),
    ];
    
    for value in &test_values {
        let len = value.ssz_bytes_len();
        let mut bytes = Vec::with_capacity(len);
        value.ssz_append(&mut bytes);
        
        match Value::from_ssz_bytes(&bytes) {
            Ok(deserialized) => {
                if *value != deserialized {
                    return 0; // Round-trip failed
                }
            }
            Err(_) => return 0, // Deserialization failed
        }
    }
    
    1 // All tests passed
}

//-----------------------------------------------------------------------------
// Utility Functions
//-----------------------------------------------------------------------------

/// Get FFI version string (caller must free with causality_free_string)
#[no_mangle]
pub extern "C" fn causality_ffi_version() -> *mut c_char {
    let version = format!("Causality FFI v{}", env!("CARGO_PKG_VERSION"));
    match CString::new(version) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get debug information for a value (caller must free with causality_free_string)
#[no_mangle]
pub extern "C" fn causality_value_debug_info(value: *const CausalityValue) -> *mut c_char {
    if value.is_null() {
        return std::ptr::null_mut();
    }
    
    let rust_value = unsafe { &*(value as *const Value) };
    let debug_info = format!("{:?}", rust_value);
    match CString::new(debug_info) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
} 