//! Causality Concurrency Utilities
//!
//! This crate provides reusable concurrency primitives and patterns.
// TODO: Review if some basic traits/primitives should be in causality-core instead.
// TODO: Update dependencies (likely needs causality-core for error types, maybe causality-types).
// TODO: Create a Cargo.toml for this crate.

pub mod atomic;
pub mod error;
pub mod fork;
pub mod lock;
pub mod mod;
// pub mod resource_manager; // Moved to causality-runtime
pub mod patterns;
pub mod pool;
pub mod primitives;
pub mod race;
pub mod scheduler;
pub mod task_id;
pub mod task_scheduler;
pub mod timeout;
pub mod wait_queue;

// TODO: Add re-exports for commonly used items. 