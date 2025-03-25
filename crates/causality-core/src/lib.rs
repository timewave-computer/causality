// Core abstractions and system components for the Causality system
// Original file: src/core/mod.rs

// Re-export modules
pub mod actor;
pub mod committee;
pub mod concurrency;
pub mod time;
pub mod verification;
pub mod zk;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 