// Causality Patterns
// Common patterns for working with the Causality log system

pub mod integration;
pub mod ast;
pub mod relationship; // Stub module for now - real code moved to backup directory

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 