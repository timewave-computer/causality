//! Causality Runtime - Core execution engine for the Causality system
//!
//! This crate primarily provides the TEL (Temporal Effect Language) interpreter and its context,
//! along with supporting components for execution, state management, and transformation.

#![forbid(unsafe_code)]
// #![warn(missing_docs)] // Temporarily disabled during heavy development

//-----------------------------------------------------------------------------
// Module export
//-----------------------------------------------------------------------------

pub mod system_coordinator;
pub mod error;
pub mod nullifier;
pub mod state;
pub mod store;
pub mod tel;
pub mod state_manager;
pub mod trace_builder;
pub mod optimization;
pub mod strategies;
pub mod config;

/// Placeholder function to confirm crate linkage.
pub fn placeholder_runtime_function() -> String {
    "causality-runtime placeholder function executed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert!(placeholder_runtime_function().contains("placeholder"));
    }
}
