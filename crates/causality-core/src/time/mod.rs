// Time Module
//
// This module provides time capabilities for the causality system.

use std::fmt::Debug;
use std::time::Duration as StdDuration; // Alias to avoid conflict if we define our own Duration
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Internal time types and models
pub mod timestamp;
pub mod duration;
pub mod clock;
pub mod watch;
pub mod provider;
pub mod map;
pub mod types;
pub mod effect;
pub mod effect_handler;
pub mod service;
pub mod adapter;
pub mod observer;
pub mod utils;
pub mod view;
pub mod event; // Make sure event module is included

// Re-exports from modules
pub use duration::TimeDelta; // We'll refer to it as TimeDelta to avoid conflicts
pub use effect::{
    TimeEffect, TimeEffectHandler, BasicTimeEffectHandler, SimpleTimeEffectHandler,
    TimeEffectType, TimeAttestation, AttestationSource, TimeError,
    CausalTimeEffect, ClockTimeEffect,
};
pub use timestamp::Timestamp;
pub use clock::ClockTime;
pub use map::{TimeMap, TimeMapSnapshot};
pub use service::TimeService;
pub use adapter::TimeSystemAdapter;
pub use observer::TimeObserver;
pub use provider::TimeProvider;
pub use utils::{parse_duration, format_duration};
pub use view::{TimeView, TimeViewSnapshot};
pub use event::Timer; // Export Timer from event module

/// TimeMetrics tracks timing information for various operations
/// (This is separate from the domain-based TimeMap, to avoid confusion)
#[derive(Debug, Default, Clone)]
pub struct TimeMetrics {
    /// Internal storage for timing data
    times: Arc<Mutex<HashMap<String, StdDuration>>>,
    /// Start times for ongoing operations
    start_times: Arc<Mutex<HashMap<String, Instant>>>,
}

impl TimeMetrics {
    /// Create a new empty TimeMetrics
    pub fn new() -> Self {
        Self {
            times: Arc::new(Mutex::new(HashMap::new())),
            start_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start timing an operation
    pub fn start(&self, key: impl Into<String>) {
        let key = key.into();
        if let Ok(mut start_times) = self.start_times.lock() {
            start_times.insert(key, Instant::now());
        }
    }

    /// End timing an operation and record the duration
    pub fn end(&self, key: impl Into<String>) {
        let key = key.into();
        let start = {
            if let Ok(mut start_times) = self.start_times.lock() {
                start_times.remove(&key)
            } else {
                None
            }
        };

        if let Some(start) = start {
            let duration = start.elapsed();
            if let Ok(mut times) = self.times.lock() {
                times.insert(key, duration);
            }
        }
    }

    /// Record a specific duration for an operation
    pub fn record(&self, key: impl Into<String>, duration: StdDuration) {
        let key = key.into();
        if let Ok(mut times) = self.times.lock() {
            times.insert(key, duration);
        }
    }

    /// Get the duration for a specific operation
    pub fn get(&self, key: &str) -> Option<StdDuration> {
        if let Ok(times) = self.times.lock() {
            times.get(key).cloned()
        } else {
            None
        }
    }

    /// Get all recorded timings
    pub fn get_all(&self) -> HashMap<String, StdDuration> {
        if let Ok(times) = self.times.lock() {
            times.clone()
        } else {
            HashMap::new()
        }
    }

    /// Merge another TimeMetrics into this one
    pub fn merge(&self, other: &TimeMetrics) {
        if let (Ok(mut times), Ok(other_times)) = (self.times.lock(), other.times.lock()) {
            for (key, duration) in other_times.iter() {
                times.insert(key.clone(), *duration);
            }
        }
    }
} 