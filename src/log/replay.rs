// Replay engine for Causality Unified Log System
//
// This module provides functionality for replaying log entries to
// reconstruct system state.

mod engine;
mod filter;
mod callback;
mod state;

pub use engine::ReplayEngine;
pub use filter::ReplayFilter;
pub use callback::{ReplayCallback, NoopReplayCallback};
pub use state::{ReplayState, ResourceState, DomainState};

use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// The status of a replay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayStatus {
    /// The replay is pending
    Pending,
    /// The replay is in progress
    InProgress,
    /// The replay is complete
    Complete,
    /// The replay failed
    Failed,
}

impl fmt::Display for ReplayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayStatus::Pending => write!(f, "Pending"),
            ReplayStatus::InProgress => write!(f, "In Progress"),
            ReplayStatus::Complete => write!(f, "Complete"),
            ReplayStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// The result of a replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// The status of the replay
    pub status: ReplayStatus,
    /// The number of entries processed
    pub processed_entries: usize,
    /// The time when the replay started
    pub start_time: DateTime<Utc>,
    /// The time when the replay ended
    pub end_time: Option<DateTime<Utc>>,
    /// The error message, if any
    pub error: Option<String>,
    /// The reconstructed state
    pub state: Option<ReplayState>,
}

/// Replay options for configuring replay behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOptions {
    /// The start time for replay
    pub start_time: Option<DateTime<Utc>>,
    /// The end time for replay
    pub end_time: Option<DateTime<Utc>>,
    /// The trace ID to filter by
    pub trace_id: Option<String>,
    /// The resources to include
    pub resources: Option<std::collections::HashSet<crate::types::ContentId>>,
    /// The domains to include
    pub domains: Option<std::collections::HashSet<crate::types::DomainId>>,
    /// The entry types to include
    pub entry_types: Option<std::collections::HashSet<crate::log::entry::EntryType>>,
    /// The maximum number of entries to process
    pub max_entries: Option<usize>,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            start_time: None,
            end_time: None,
            trace_id: None,
            resources: None,
            domains: None,
            entry_types: None,
            max_entries: None,
        }
    }
}

impl ReplayOptions {
    /// Create a new replay options with default values
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Set the start time for replay
    pub fn with_start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }
    
    /// Set the end time for replay
    pub fn with_end_time(mut self, time: DateTime<Utc>) -> Self {
        self.end_time = Some(time);
        self
    }
    
    /// Set the trace ID to filter by
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Set the resources to include
    pub fn with_resources(mut self, resources: std::collections::HashSet<crate::types::ContentId>) -> Self {
        self.resources = Some(resources);
        self
    }
    
    /// Set the domains to include
    pub fn with_domains(mut self, domains: std::collections::HashSet<crate::types::DomainId>) -> Self {
        self.domains = Some(domains);
        self
    }
    
    /// Set the entry types to include
    pub fn with_entry_types(mut self, entry_types: std::collections::HashSet<crate::log::entry::EntryType>) -> Self {
        self.entry_types = Some(entry_types);
        self
    }
    
    /// Set the maximum number of entries to process
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = Some(max);
        self
    }
} 
