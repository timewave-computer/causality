// Core engine functionality for the Causality system
// Original file: src/engine/mod.rs

// Re-export modules
pub mod execution;
pub mod invocation;
pub mod log;
pub mod operation;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 