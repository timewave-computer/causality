// Resource Integration Module for TEL
//
// This module integrates Temporal Effect Language effects with
// the resource management system, based on the register-based
// resource model defined in ADR 003.

// Core resource submodules
pub mod model;
pub mod operations;
pub mod tracking;
pub mod verify;
pub mod vm;
pub mod snapshot;
pub mod version;

#[cfg(test)]
pub mod tests;

// Re-export core components
pub use model::{Register, RegisterId, RegisterState, RegisterContents, Resource, ResourceGuard, ResourceManager, AccessMode};
pub use operations::{ResourceOperation, ResourceOperationType};
pub use crate::tel::types::OperationId;
pub use tracking::{ResourceTracker, ResourceState, ResourceStatus};
pub use verify::{ZkVerifier, VerifierConfig, VerificationResult};
pub use vm::{ResourceVmIntegration, ExecutionContext, VmIntegrationConfig, VmRegId, MemoryManager, AccessIntent};
pub use snapshot::{SnapshotManager, SnapshotId, SnapshotStorage, FileSnapshotStorage, SnapshotScheduleConfig, RestoreMode, RestoreOptions, RestoreResult};
pub use version::{VersionManager, VersionId, ResourceChange, ChangeType, VersioningConfig, VersionDiff, VersionTree}; 
