// Time Module
//
// This module provides core time definitions for the causality system.
// Implementations and runtime logic are moved elsewhere.

use std::fmt::Debug;
use std::time::Duration as StdDuration; // Alias to avoid conflict if we define our own Duration
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Core time types and definitions
pub mod timestamp;
pub mod duration;
pub mod clock;
// REMOVED: mod watch;
pub mod map; // Contains TimeMap trait/type?
pub mod types;
// REMOVED: mod effect;
// REMOVED: mod effect_handler;
// REMOVED: mod service;
// REMOVED: mod adapter;
// REMOVED: mod observer;
pub mod utils; // Check if core utils or impl utils
pub mod view; // Check if core view trait/type or impl
pub mod event; // Check if core event type or impl
// REMOVED: mod provider;
// Potential core concepts - check contents later
pub mod temporal;
pub mod physical;
pub mod interval;
pub mod error;
pub mod proof;
// REMOVED: mod implementations;
// REMOVED: mod factory;
// REMOVED: mod integration;
// REMOVED: mod handler;

#[cfg(test)]
mod tests;

// Re-exports from modules that likely REMAIN in core
pub use duration::TimeDelta; // We'll refer to it as TimeDelta to avoid conflicts
pub use timestamp::Timestamp;
pub use clock::{Clock, FixedClock, MonotonicClock, ThreadLocalClock, PhysicalClock, default_clock, fixed_clock, monotonic_clock, thread_local_clock}; // Export Clock trait and implementations
pub use map::{TimeMap, TimeMapSnapshot}; // Assuming these are core types/traits
pub use utils::{parse_duration, format_duration}; // Assuming these are core utils
pub use view::{TimeView, TimeViewSnapshot}; // Assuming these are core types/traits
pub use event::Timer; // Assuming Timer is a core type/trait
pub use error::TimeError; // Export the new TimeError
pub use physical::{TimeUtils, TimestampFormat}; // Export TimeUtils for formatting and timestamp conversion

// --- TimeMetrics Removed ---
// The TimeMetrics struct and its implementation have been removed
// as they represent concrete implementation/utility logic, not core definitions. 