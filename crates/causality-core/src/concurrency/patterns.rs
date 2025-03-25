// Common concurrency patterns
// Original file: src/concurrency/patterns.rs

// Concurrency patterns
//
// This module provides higher-level concurrency patterns that build on the
// primitives provided by the primitives module.

// Import and re-export submodules
mod fork;
pub use fork::{fork, fork_join, fork_try_join, fork_each};

mod race;
pub use race::{race, race_ok, race_result, race_until};

mod barrier;
pub use barrier::{Barrier, barrier, resource_barrier, timeout_barrier, wait_for_resources};

mod timeout;
pub use timeout::{timeout, timeout_result, with_timeout, WithTimeout, timeout_with_retry}; 