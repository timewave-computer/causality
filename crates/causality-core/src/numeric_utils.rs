//! Utilities for working with numeric types in the Causality framework.
//!
//! This module provides helper functions for converting between different
//! numeric representations while maintaining determinism for ZK compatibility.

use causality_types::primitive::number::Number;

//-----------------------------------------------------------------------------
// Number Conversion Utilities
//-----------------------------------------------------------------------------

/// Convert an i64 to a Number type
pub fn as_number(value: i64) -> Number {
    Number::Integer(value)
}

/// Convert an i32 to a Number type
pub fn i32_as_number(value: i32) -> Number {
    Number::Integer(value as i64)
}

/// Convert a u64 to a Number type
pub fn u64_as_number(value: u64) -> Number {
    // Safely convert u64 to i64, capping at i64::MAX for values that would overflow
    let i64_value = if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    };
    Number::Integer(i64_value)
}
