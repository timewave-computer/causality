// Log system for recording facts and events
// Original file: src/log/mod.rs

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

// Visualization
pub mod visualization;

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

// Synchronization re-exports
pub use sync::{SyncManager, SyncProtocol, SyncConfig, PeerInfo, HttpSyncProtocol, SyncableStorage};

// Visualization re-exports
pub use visualization::{
    LogVisualizer, VisualizationFilter, VisualizationFormat,
    CausalityGraph, CausalityNode
};

pub use time_map::LogTimeMapIntegration;

pub use replay::{ReplayEngine, ReplayOptions, ReplayResult, ReplayStatus};
pub use replay::{ReplayFilter, ReplayCallback, NoopReplayCallback};

pub use fact_snapshot::{FactSnapshot, FactId};
pub use fact_dependency_validator::FactDependencyValidator;

// Log module for Causality Content-Addressed Code System
//
// This module provides functionality for working with logs, including creating
// and reading log entries, log storage, and log visualization.

pub mod performance;

pub use performance::{
    BatchConfig,
    BatchWriter,
    OptimizedLogStorage,
    compression,
    LogIndex,
};

#[cfg(test)]
pub mod tests; 