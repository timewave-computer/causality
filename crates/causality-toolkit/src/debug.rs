//! Debug utilities for Causality development

/// Simple logging utility
pub fn debug_log(message: &str) {
    eprintln!("[DEBUG] {}", message);
}

/// Pretty print any serializable structure
pub fn pretty_print<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
} 