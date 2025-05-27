//-----------------------------------------------------------------------------
// SP1 Format Utilities
//-----------------------------------------------------------------------------
//
// This module provides simplified formatting utilities for the SP1 environment
// that doesn't have std::fmt available.

use alloc::{string::String, vec::Vec, vec};

//-----------------------------------------------------------------------------
// SP1-Compatible String Conversions
//-----------------------------------------------------------------------------

/// Extension trait to convert primitive numbers to strings in SP1 environment
pub trait ToStringInSp1 {
    /// Convert value to string
    fn to_string(&self) -> String;
}

impl ToStringInSp1 for usize {
    fn to_string(&self) -> String {
        // Simple implementation for usize to string conversion
        if *self == 0 {
            return String::from("0");
        }
        
        let mut num = *self;
        let mut digits = Vec::new();
        
        while num > 0 {
            let digit = (num % 10) as u8;
            digits.push(b'0' + digit);
            num /= 10;
        }
        
        // Reverse the digits
        digits.reverse();
        
        // Convert to String
        String::from_utf8(digits).unwrap_or(String::from("ERROR"))
    }
}

//-----------------------------------------------------------------------------
// SP1-Compatible Format Functions
//-----------------------------------------------------------------------------

/// Format a message with the number of constraints
pub fn format_constraint_count(count: usize) -> String {
    // Create a basic string with the count
    let mut result = String::from("All ");
    
    // Use our SP1-compatible to_string implementation
    let count_str = ToStringInSp1::to_string(&count);
    result.push_str(&count_str);
    
    // Add the rest of the message
    result.push_str(" constraints satisfied");
    
    result
}

/// Format an error message for failed constraints
pub fn format_constraint_failure(failed_count: usize, total_count: usize) -> String {
    // Basic implementation without detailed error info
    let mut result = String::from("Constraint verification failed: ");
    
    // Add counts using our SP1-compatible to_string
    result.push_str(&ToStringInSp1::to_string(&failed_count));
    result.push_str(" of ");
    result.push_str(&ToStringInSp1::to_string(&total_count));
    result.push_str(" constraints not satisfied");
    
    result
}

/// Format bytes for error messages in a simplified way
pub trait IntoBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

impl IntoBytes for String {
    fn as_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}
