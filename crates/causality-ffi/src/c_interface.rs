//! C-compatible FFI interface for Causality values
//!
//! This module provides a C-compatible interface for creating, manipulating,
//! and serializing Causality values for cross-language integration.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use causality_core::lambda::base::Value;
use causality_core::system::serialization::{SszEncode, SszDecode};
use causality_core::{EntityId, ExprId, Hasher};
use causality_core::machine::Resource;
use crate::{FfiErrorCode};

// Re-export common types for convenience
pub use causality_core::lambda::base::Value as CausalityValueRust;

//-----------------------------------------------------------------------------
// Core C Interface Types
//-----------------------------------------------------------------------------

/// Opaque handle to a Causality value for C interface
#[repr(C)]
pub struct CausalityValue {
    _private: [u8; 0],
}

/// Opaque handle to a Causality resource for C interface
#[repr(C)]
pub struct CausalityResource {
    _private: [u8; 0],
}

/// Opaque handle to a Causality expression for C interface
#[repr(C)]
pub struct CausalityExpr {
    _private: [u8; 0],
}

//-----------------------------------------------------------------------------
// Value Creation and Management
//-----------------------------------------------------------------------------

/// Create a unit value
#[no_mangle]
pub extern "C" fn causality_value_unit() -> *mut CausalityValue {
    let value = Box::new(Value::Unit);
    Box::into_raw(value) as *mut CausalityValue
}

/// Create a boolean value
#[no_mangle]
pub extern "C" fn causality_value_bool(b: c_int) -> *mut CausalityValue {
    let value = Box::new(Value::Bool(b != 0));
    Box::into_raw(value) as *mut CausalityValue
}

/// Create an integer value
#[no_mangle]
pub extern "C" fn causality_value_int(i: c_uint) -> *mut CausalityValue {
    let value = Box::new(Value::Int(i));
    Box::into_raw(value) as *mut CausalityValue
}

/// Create a string value
#[no_mangle]
pub extern "C" fn causality_value_string(s: *const c_char) -> *mut CausalityValue {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str = unsafe { CStr::from_ptr(s) };
    match c_str.to_str() {
        Ok(str_slice) => {
            let value = Box::new(Value::String(causality_core::system::Str::new(str_slice)));
            Box::into_raw(value) as *mut CausalityValue
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
        Ok(str_slice) => {
            let value = Box::new(Value::Symbol(causality_core::system::Str::new(str_slice)));
            Box::into_raw(value) as *mut CausalityValue
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
// Resource Management Extensions
//-----------------------------------------------------------------------------

/// Simple resource wrapper with consumption state for FFI
#[derive(Debug, Clone)]
struct ResourceWrapper {
    resource: Resource,
    is_consumed: bool,
}

/// Create a resource
#[no_mangle]
pub extern "C" fn causality_create_resource(
    resource_type: *const c_char,
    domain_id: *const u8,
    quantity: u64,
) -> *mut CausalityResource {
    if resource_type.is_null() || domain_id.is_null() {
        return std::ptr::null_mut();
    }

    let resource_type_str = unsafe { CStr::from_ptr(resource_type) };
    let resource_type_string = match resource_type_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Convert domain_id bytes to EntityId
    let domain_bytes = unsafe { std::slice::from_raw_parts(domain_id, 32) };
    let mut domain_array = [0u8; 32];
    domain_array.copy_from_slice(domain_bytes);
    let domain_entity_id = EntityId::from_bytes(domain_array);

    // Create resource using the simple constructor
    let machine_value = causality_core::machine::MachineValue::Unit; // Simple unit value
    let type_inner = causality_core::lambda::TypeInner::Base(causality_core::lambda::BaseType::Unit);
    
    let mut resource = Resource::simple(machine_value, type_inner, domain_entity_id);
    
    // Update quantity field
    resource.quantity = quantity;
    
    // Update label (resource type)
    resource.label = causality_core::system::Str::new(resource_type_string);

    let resource_wrapper = ResourceWrapper {
        resource,
        is_consumed: false,
    };

    let boxed_resource = Box::new(resource_wrapper);
    Box::into_raw(boxed_resource) as *mut CausalityResource
}

/// Consume a resource
#[no_mangle]
pub extern "C" fn causality_consume_resource(resource: *mut CausalityResource) -> c_int {
    if resource.is_null() {
        return 0;
    }

    let resource_ref = unsafe { &mut *(resource as *mut ResourceWrapper) };
    if resource_ref.is_consumed {
        return 0; // Already consumed
    }

    resource_ref.is_consumed = true;
    1 // Success
}

/// Check if a resource is valid
#[no_mangle]
pub extern "C" fn causality_is_resource_valid(resource: *const CausalityResource) -> c_int {
    if resource.is_null() {
        return 0;
    }

    let resource_ref = unsafe { &*(resource as *const ResourceWrapper) };
    if resource_ref.is_consumed {
        0 // Consumed
    } else {
        1 // Valid
    }
}

/// Free a resource
#[no_mangle]
pub extern "C" fn causality_resource_free(resource: *mut CausalityResource) {
    if !resource.is_null() {
        unsafe {
            let _ = Box::from_raw(resource as *mut ResourceWrapper);
        }
    }
}

/// Get resource ID as bytes
#[no_mangle]
pub extern "C" fn causality_resource_id(resource: *const CausalityResource) -> *const u8 {
    if resource.is_null() {
        return std::ptr::null();
    }

    let resource_ref = unsafe { &*(resource as *const ResourceWrapper) };
    resource_ref.resource.id.as_bytes().as_ptr()
}

//-----------------------------------------------------------------------------
// Expression Management Extensions
//-----------------------------------------------------------------------------

/// Compile an expression from string
#[no_mangle]
pub extern "C" fn causality_compile_expr(expr_string: *const c_char) -> *mut CausalityExpr {
    if expr_string.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(expr_string) };
    let expr_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Create content-addressed expression ID
    let expr_bytes = causality_core::Sha256Hasher::hash(expr_str.as_bytes());
    let expr_id = ExprId::from_bytes(expr_bytes);

    // Create simple expression container
    let expr_container = ExprContainer {
        id: expr_id,
        _expression: causality_core::system::Str::new(expr_str),
    };

    let boxed_expr = Box::new(expr_container);
    Box::into_raw(boxed_expr) as *mut CausalityExpr
}

/// Simple expression container
#[derive(Debug, Clone)]
struct ExprContainer {
    id: ExprId,
    _expression: causality_core::system::Str,
}

/// Get expression ID as bytes
#[no_mangle]
pub extern "C" fn causality_expr_id(expr: *const CausalityExpr) -> *const u8 {
    if expr.is_null() {
        return std::ptr::null();
    }

    let expr_ref = unsafe { &*(expr as *const ExprContainer) };
    expr_ref.id.as_bytes().as_ptr()
}

/// Free an expression
#[no_mangle]
pub extern "C" fn causality_expr_free(expr: *mut CausalityExpr) {
    if !expr.is_null() {
        unsafe {
            let _ = Box::from_raw(expr as *mut ExprContainer);
        }
    }
}

/// Submit an intent
#[no_mangle]
pub extern "C" fn causality_submit_intent(
    name: *const c_char,
    domain_id: *const u8,
    expr_string: *const c_char,
) -> c_int {
    if name.is_null() || domain_id.is_null() || expr_string.is_null() {
        return 0;
    }

    let name_str = unsafe { CStr::from_ptr(name) };
    let expr_str = unsafe { CStr::from_ptr(expr_string) };

    if name_str.to_str().is_err() || expr_str.to_str().is_err() {
        return 0;
    }

    // Mock implementation - in production this would submit to the intent system
    1 // Success
}

/// Get system metrics
#[no_mangle]
pub extern "C" fn causality_get_system_metrics() -> *mut c_char {
    let metrics = "Resources: active, Expressions: compiled, FFI: operational";
    match CString::new(metrics) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

//-----------------------------------------------------------------------------
// Serialization Interface
//-----------------------------------------------------------------------------

/// Result of serialization operation
#[repr(C)]
pub struct SerializationResult {
    /// Serialized data (caller must free with causality_free_serialized_data)
    pub data: *mut u8,
    /// Length of serialized data
    pub length: usize,
    /// Error code
    pub error_code: FfiErrorCode,
    /// Error message (caller must free with causality_free_error_message)
    pub error_message: *mut c_char,
}

impl SerializationResult {
    fn success(data: Vec<u8>) -> Self {
        let boxed_data = data.into_boxed_slice();
        let length = boxed_data.len();
        let raw_data = Box::into_raw(boxed_data) as *mut u8;
        
        Self {
            data: raw_data,
            length,
            error_code: FfiErrorCode::Success,
            error_message: std::ptr::null_mut(),
        }
    }
    
    fn error(code: FfiErrorCode, message: &str) -> Self {
        let error_message = match CString::new(message) {
            Ok(s) => s.into_raw(),
            Err(_) => std::ptr::null_mut(),
        };
        
        Self {
            data: std::ptr::null_mut(),
            length: 0,
            error_code: code,
            error_message,
        }
    }
}

/// Serialize a Causality value to bytes
#[no_mangle]
pub extern "C" fn causality_value_serialize(value: *const CausalityValue) -> SerializationResult {
    if value.is_null() {
        return SerializationResult::error(FfiErrorCode::InvalidInput, "Value pointer is null");
    }
    
    let rust_value = unsafe { &*(value as *const Value) };
    
    let len = rust_value.ssz_bytes_len();
    let mut bytes = Vec::with_capacity(len);
    rust_value.ssz_append(&mut bytes);
    SerializationResult::success(bytes)
}

/// Deserialize bytes to a Causality value
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
        Ok(value) => {
            let boxed_value = Box::new(value);
            Box::into_raw(boxed_value) as *mut CausalityValue
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free serialized data
#[no_mangle]
pub extern "C" fn causality_free_serialized_data(data: *mut u8, length: usize) {
    if !data.is_null() && length > 0 {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(data, length));
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
// Value Type Inspection
//-----------------------------------------------------------------------------

/// Enumeration of Causality value types
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// Unit type
    Unit = 0,
    /// Boolean type
    Bool = 1,
    /// Integer type
    Int = 2,
    /// Symbol type
    Symbol = 3,
    /// String type
    String = 4,
    /// Product type
    Product = 5,
    /// Sum type
    Sum = 6,
    /// Record type
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

//-----------------------------------------------------------------------------
// Configuration Types
//-----------------------------------------------------------------------------

/// Memory management mode for FFI
#[derive(Debug, Clone)]
pub enum MemoryMode {
    /// Automatic memory management
    Automatic,
    /// Manual memory management
    Manual,
    /// Shared memory mode
    Shared,
}

/// Configuration for FFI operations
#[derive(Debug, Clone)]
pub struct FfiConfig {
    /// Enable debug mode
    pub debug: bool,
    /// Maximum string length
    pub max_string_length: usize,
    /// Memory management mode
    pub memory_mode: MemoryMode,
}

impl Default for FfiConfig {
    fn default() -> Self {
        Self {
            debug: false,
            max_string_length: 1024 * 1024, // 1MB default limit
            memory_mode: MemoryMode::Automatic,
        }
    }
}

/// High-level FFI interface for Causality operations
#[derive(Debug)]
pub struct CausalityFfi {
    /// FFI configuration
    config: FfiConfig,
}

impl CausalityFfi {
    /// Create a new FFI interface with default configuration
    pub fn new() -> Self {
        Self::with_config(FfiConfig::default())
    }
    
    /// Create a new FFI interface with custom configuration
    pub fn with_config(config: FfiConfig) -> Self {
        Self { config }
    }
    
    /// Get the configuration
    pub fn config(&self) -> &FfiConfig {
        &self.config
    }
}

impl Default for CausalityFfi {
    fn default() -> Self {
        Self::new()
    }
} 