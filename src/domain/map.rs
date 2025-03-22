// Map of time module for Causality
//
// This module provides functionality for map of time synchronization, correlation,
// and causal consistency tracking across domains to ensure causal consistency 
// between different domains.

pub mod map;
pub mod sync;
pub mod types;

// Re-export key types and components
pub use map::{TimeMap, TimeMapEntry, TimeMapHistory, TimeMapNotifier, SharedTimeMap};
pub use types::{TimePoint, TimeRange};
pub use sync::{TimeSyncConfig, SyncStatus, SyncResult, TimeSource, SyncStrategy, VerificationStatus, TimeCommitment};

// Export additional components from sync
pub use sync::{TimeSyncManager, TimeVerificationService, ConsensusVerificationManager}; 