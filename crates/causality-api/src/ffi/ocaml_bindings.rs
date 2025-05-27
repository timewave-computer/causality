//! OCaml FFI Bindings
//!
//! This module provides C-compatible functions for FFI with OCaml,
//! using SSZ serialization to ensure consistent data representation.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar};
use std::slice;

use causality_types::{
    expr::value::ValueExpr,
    resource::Resource,
    tel::{
        Handler, Effect, Intent, Edge, EdgeKind, EffectGraph,
    },
};

use super::ocaml_adapter;

// Helper function to convert a C string to a Rust String
unsafe fn c_str_to_string(c_str: *const c_char) -> String {
    CStr::from_ptr(c_str).to_string_lossy().into_owned()
}

// Helper function to convert a Rust String to a C string
fn string_to_c_str(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// Helper function to convert a C byte array to a Rust Vec<u8>
unsafe fn c_bytes_to_vec(bytes: *const c_uchar, len: usize) -> Vec<u8> {
    slice::from_raw_parts(bytes, len).to_vec()
}

// Helper function to convert a Rust Vec<u8> to a C byte array
fn vec_to_c_bytes(vec: Vec<u8>) -> (*mut c_uchar, usize) {
    let len = vec.len();
    let mut vec = vec;
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec);
    (ptr, len)
}

/// Free a C string allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Free a byte array allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_bytes(ptr: *mut c_uchar, len: usize) {
    if !ptr.is_null() {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}

//-----------------------------------------------------------------------------
// ValueExpr FFI
//-----------------------------------------------------------------------------

/// Deserialize a ValueExpr from OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn value_expr_from_ocaml(
    bytes: *const c_uchar,
    len: usize,
    error_msg: *mut *mut c_char,
) -> *mut ValueExpr {
    let bytes_vec = c_bytes_to_vec(bytes, len);
    
    match ocaml_adapter::value_expr_from_ocaml(&bytes_vec) {
        Ok(value) => Box::into_raw(Box::new(value)),
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Serialize a ValueExpr to OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn value_expr_to_ocaml(
    value_ptr: *const ValueExpr,
    out_len: *mut usize,
) -> *mut c_uchar {
    if value_ptr.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    
    let value = &*value_ptr;
    let bytes = ocaml_adapter::value_expr_to_ocaml(value);
    let (ptr, len) = vec_to_c_bytes(bytes);
    *out_len = len;
    ptr
}

/// Free a ValueExpr allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_value_expr(ptr: *mut ValueExpr) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

//-----------------------------------------------------------------------------
// Resource FFI
//-----------------------------------------------------------------------------

/// Deserialize a Resource from OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn resource_from_ocaml(
    bytes: *const c_uchar,
    len: usize,
    error_msg: *mut *mut c_char,
) -> *mut Resource {
    let bytes_vec = c_bytes_to_vec(bytes, len);
    
    match ocaml_adapter::resource_from_ocaml(&bytes_vec) {
        Ok(resource) => Box::into_raw(Box::new(resource)),
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Serialize a Resource to OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn resource_to_ocaml(
    resource_ptr: *const Resource,
    out_len: *mut usize,
) -> *mut c_uchar {
    if resource_ptr.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    
    let resource = &*resource_ptr;
    let bytes = ocaml_adapter::resource_to_ocaml(resource);
    let (ptr, len) = vec_to_c_bytes(bytes);
    *out_len = len;
    ptr
}

/// Free a Resource allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_resource(ptr: *mut Resource) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

//-----------------------------------------------------------------------------
// Handler FFI
//-----------------------------------------------------------------------------

/// Deserialize a Handler from OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn handler_from_ocaml(
    bytes: *const c_uchar,
    len: usize,
    error_msg: *mut *mut c_char,
) -> *mut Handler {
    let bytes_vec = c_bytes_to_vec(bytes, len);
    
    match ocaml_adapter::handler_from_ocaml(&bytes_vec) {
        Ok(handler) => Box::into_raw(Box::new(handler)),
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Serialize a Handler to OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn handler_to_ocaml(
    handler_ptr: *const Handler,
    out_len: *mut usize,
) -> *mut c_uchar {
    if handler_ptr.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    
    let handler = &*handler_ptr;
    let bytes = ocaml_adapter::handler_to_ocaml(handler);
    let (ptr, len) = vec_to_c_bytes(bytes);
    *out_len = len;
    ptr
}

/// Free a Handler allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_handler(ptr: *mut Handler) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

//-----------------------------------------------------------------------------
// Effect FFI
//-----------------------------------------------------------------------------

/// Deserialize an Effect from OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn effect_from_ocaml(
    bytes: *const c_uchar,
    len: usize,
    error_msg: *mut *mut c_char,
) -> *mut Effect {
    let bytes_vec = c_bytes_to_vec(bytes, len);
    
    match ocaml_adapter::effect_from_ocaml(&bytes_vec) {
        Ok(effect) => Box::into_raw(Box::new(effect)),
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Serialize an Effect to OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn effect_to_ocaml(
    effect_ptr: *const Effect,
    out_len: *mut usize,
) -> *mut c_uchar {
    if effect_ptr.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    
    let effect = &*effect_ptr;
    let bytes = ocaml_adapter::effect_to_ocaml(effect);
    let (ptr, len) = vec_to_c_bytes(bytes);
    *out_len = len;
    ptr
}

/// Free an Effect allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_effect(ptr: *mut Effect) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

//-----------------------------------------------------------------------------
// Intent FFI
//-----------------------------------------------------------------------------

/// Deserialize an Intent from OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn intent_from_ocaml(
    bytes: *const c_uchar,
    len: usize,
    error_msg: *mut *mut c_char,
) -> *mut Intent {
    let bytes_vec = c_bytes_to_vec(bytes, len);
    
    match ocaml_adapter::intent_from_ocaml(&bytes_vec) {
        Ok(intent) => Box::into_raw(Box::new(intent)),
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Serialize an Intent to OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn intent_to_ocaml(
    intent_ptr: *const Intent,
    out_len: *mut usize,
) -> *mut c_uchar {
    if intent_ptr.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    
    let intent = &*intent_ptr;
    let bytes = ocaml_adapter::intent_to_ocaml(intent);
    let (ptr, len) = vec_to_c_bytes(bytes);
    *out_len = len;
    ptr
}

/// Free an Intent allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_intent(ptr: *mut Intent) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

//-----------------------------------------------------------------------------
// Edge FFI
//-----------------------------------------------------------------------------

/// Deserialize an Edge from OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn edge_from_ocaml(
    bytes: *const c_uchar,
    len: usize,
    error_msg: *mut *mut c_char,
) -> *mut Edge {
    let bytes_vec = c_bytes_to_vec(bytes, len);
    
    match ocaml_adapter::edge_from_ocaml(&bytes_vec) {
        Ok(edge) => Box::into_raw(Box::new(edge)),
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Serialize an Edge to OCaml bytes
#[no_mangle]
pub unsafe extern "C" fn edge_to_ocaml(
    edge_ptr: *const Edge,
    out_len: *mut usize,
) -> *mut c_uchar {
    if edge_ptr.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }
    
    let edge = &*edge_ptr;
    let bytes = ocaml_adapter::edge_to_ocaml(edge);
    let (ptr, len) = vec_to_c_bytes(bytes);
    *out_len = len;
    ptr
}

/// Free an Edge allocated by Rust
#[no_mangle]
pub unsafe extern "C" fn free_edge(ptr: *mut Edge) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}

//-----------------------------------------------------------------------------
// Hex Utilities FFI
//-----------------------------------------------------------------------------

/// Convert a ValueExpr from hex string (OCaml)
#[no_mangle]
pub unsafe extern "C" fn value_expr_from_hex(
    hex_ptr: *const c_char,
    error_msg: *mut *mut c_char,
) -> *mut ValueExpr {
    if hex_ptr.is_null() {
        if !error_msg.is_null() {
            *error_msg = string_to_c_str("Null hex string pointer".to_string());
        }
        return std::ptr::null_mut();
    }
    
    let hex_str = c_str_to_string(hex_ptr);
    
    match ocaml_adapter::hex_from_ocaml(&hex_str) {
        Ok(bytes) => {
            match ocaml_adapter::value_expr_from_ocaml(&bytes) {
                Ok(value) => Box::into_raw(Box::new(value)),
                Err(e) => {
                    if !error_msg.is_null() {
                        *error_msg = string_to_c_str(format!("ValueExpr decode error: {}", e));
                    }
                    std::ptr::null_mut()
                }
            }
        },
        Err(e) => {
            if !error_msg.is_null() {
                *error_msg = string_to_c_str(format!("Hex decode error: {}", e));
            }
            std::ptr::null_mut()
        }
    }
}

/// Convert a ValueExpr to hex string (OCaml)
#[no_mangle]
pub unsafe extern "C" fn value_expr_to_hex(value_ptr: *const ValueExpr) -> *mut c_char {
    if value_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    let value = &*value_ptr;
    let bytes = ocaml_adapter::value_expr_to_ocaml(value);
    let hex_str = ocaml_adapter::hex_to_ocaml(&bytes);
    string_to_c_str(hex_str)
} 