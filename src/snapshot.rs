// Snapshot and Time-Travel System for Causality
//
// This module provides functionality for:
// 1. Creating, managing, and restoring execution snapshots
// 2. Time-travel debugging with execution history navigation
// 3. State inspection and comparison across execution points
// 4. Execution forking for "what-if" scenarios

// Snapshot submodules
pub mod manager;
pub mod storage;
pub mod incremental;
pub mod checkpointing;

// Time-travel submodules (moved from timetravel module)
pub mod navigator;
pub mod inspector;
pub mod diff;
pub mod fork;

// Re-export snapshot core types
pub use manager::{ExecutionSnapshot, SnapshotId, SnapshotManager, SnapshotError};
pub use storage::FileSystemSnapshotManager;
pub use incremental::{
    IncrementalSnapshotManager, 
    SnapshotDiff, 
    CallStackChanges, 
    ModifiedFrame, 
    ResourceUsageDiff
};
pub use checkpointing::{CheckpointManager, CheckpointConfig}; 

// Re-export time-travel core types
pub use navigator::{TimeTravel, TimeTravelNavigator, DebugError};
pub use inspector::{StateInspector, ContextStateInspector, VariableState, PositionInfo};
pub use diff::{StateDiffer, StateComparer, VariableChange, StateDiff};
pub use fork::{ExecutionFork, ForkManager, ForkId, ForkInfo, ForkModification, ForkError}; 