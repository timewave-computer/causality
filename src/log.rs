// Unified Log System for Causality
//
// This module provides the core log system used for recording all
// effects, facts, and events in the Causality system.

// Core log components
pub mod entry;
pub mod storage;
pub mod segment;
pub mod segment_manager;

// Replay mechanisms
pub mod replay;

// Synchronization
pub mod sync;

// Time integration
pub mod time_map;

// Facts and dependencies
pub mod fact;
pub mod fact_types;
pub mod fact_snapshot;
pub mod fact_replay;
pub mod fact_dependency_validator;
pub mod fact_effect_tracker;
pub mod fact_simulator;

// Supporting components
pub mod effect;
pub mod event;
pub mod test_utils;

// Re-export main types
pub use entry::{LogEntry, EntryType, EntryData};
pub use entry::{EventEntry, EventSeverity};
pub use entry::{FactEntry, EffectEntry};

pub use storage::{LogStorage, StorageConfig};
pub use storage::memory_storage::MemoryLogStorage;
pub use storage::file_storage::FileLogStorage;

pub use segment::{LogSegment, SegmentInfo};
pub use segment_manager::{LogSegmentManager, SegmentManagerOptions, RotationCriteria};

pub use sync::{SyncManager, SyncProtocol, SyncConfig, PeerInfo, HttpSyncProtocol, SyncableStorage};

pub use time_map::LogTimeMapIntegration;

pub use replay::{ReplayEngine, ReplayOptions, ReplayResult, ReplayStatus};
pub use replay::{ReplayFilter, ReplayCallback, NoopReplayCallback};

pub use fact_snapshot::{FactSnapshot, FactId, FactDependency, FactDependencyType};
pub use fact_dependency_validator::FactDependencyValidator;
pub use fact_effect_tracker::{FactEffectTracker, FactEffectRelation};

#[cfg(test)]
pub mod tests;

/// General module for the Causality Unified Log System.
pub struct LogSystem;

impl LogSystem {
    /// The Causality Unified Log System provides a comprehensive approach to recording,
    /// storing, and replaying all system activities.
    ///
    /// The system defines three primary types of log entries:
    ///
    /// 1. **Effect Entries**: Record state changes and side effects in the system
    /// 2. **Fact Entries**: Document observed truths or assertions about the system state
    /// 3. **Event Entries**: Capture significant occurrences that may not directly change state
    ///
    /// See the documentation for more details on how to use the log system.
    pub fn description() -> &'static str {
        "Causality Unified Log System for recording, storing, and replaying all system activities."
    }
} 