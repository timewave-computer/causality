//! Primitive types and utilities
//!
//! Core primitive types, identifiers, strings, numbers, time, errors, traits,
//! content addressing, and logging functionality.

pub mod ids;
pub mod string;
pub mod number;
pub mod time;
pub mod error;
pub mod trait_;
pub mod content;
pub mod logging;
pub mod mock_logger;

// Re-exports
pub use ids::*;
pub use string::*;
pub use number::*;
pub use time::*;
pub use error::{ErrorCategory, ErrorContext, CausalError, MAX_CONTEXT_ENTRIES as ERROR_MAX_CONTEXT_ENTRIES};
pub use trait_::*;
pub use content::*;
pub use logging::{LogLevel, LogEntry, AsLogger, MAX_CONTEXT_ENTRIES as LOGGING_MAX_CONTEXT_ENTRIES};
pub use mock_logger::*; 