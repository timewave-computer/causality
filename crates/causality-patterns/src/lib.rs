// Causality Patterns Crate
// Defines reusable, high-level computational patterns as TEG structures.

// TODO: Decide on module structure (e.g., mod retry; mod fanout; etc.)
pub mod integration;
pub mod capabilities; // TODO: Does this belong here or in core/ir?
pub mod relationship; // TODO: Does this belong here or in core/domain?
// pub mod ast; // TODO: Remove if not needed for patterns themselves

// Re-export key pattern generation functions
pub use integration::{create_retry_teg, create_fan_out_in_teg, PatternError};
// TODO: Add exports for other pattern functions once defined

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 