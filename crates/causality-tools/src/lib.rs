// CLI tools and utilities for the Causality system
// Original file: src/main.rs

// Re-export modules
pub mod cli;

// Re-export main
pub use crate::main::main;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 