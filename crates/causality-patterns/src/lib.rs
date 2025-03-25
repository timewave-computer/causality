// Pattern matching and AST functionality for the Causality system
// Original file: src/ast.rs and related modules

// Re-export modules
pub mod ast;
pub mod capabilities;
pub mod integration;
pub mod relationship {
    pub mod cross_domain_query;
}

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 