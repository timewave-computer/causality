//! Value handling utilities for FFI

#[cfg(feature = "c-ffi")]
/// Value manipulation utilities
pub use crate::c_interface::{
    causality_value_unit, causality_value_bool, causality_value_int, 
    causality_value_string, causality_value_symbol, causality_value_free,
    causality_value_type, causality_value_as_bool, causality_value_as_int, 
    causality_value_as_string, ValueType
};

/// Placeholder for value handling utilities
pub fn value_module_placeholder() {
    // This module will contain value conversion utilities
} 