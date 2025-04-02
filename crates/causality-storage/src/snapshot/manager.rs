// Snapshot management system
// Original file: src/snapshot/manager.rs

// Snapshot management module for Causality Content-Addressed Code System
//
// This module provides functionality for creating, managing, and restoring
// execution snapshots, enabling persistence and time-travel debugging.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use causality_types::{Error, Result};
use causality_crypto::ContentHash;
use causality_engine::ExecutionContext;
use causality_engine::{CallFrame, ContextId};
use causality_core::resource::types::{ResourceUsage, GrantId};
use crate::effect::{EffectContext, random::{RandomEffectFactory, RandomType}};

/// A unique identifier for a snapshot
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(String);

impl SnapshotId {
    /// Create a new snapshot ID
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Generate a simple ID based on current timestamp and a random component
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        // Use RandomEffect for generating the random component
        let context = EffectContext::default();
        let random_effect = RandomEffectFactory::create_effect(RandomType::Standard);
        
        // Generate random u64 - in synchronous context so we block_on
        let random_component = std::future::block_on(random_effect.gen_u64(&context))
            .unwrap_or_else(|_| timestamp as u64);
            
        let id = format!("snapshot-{}-{}", timestamp, random_component);
        
        SnapshotId(id)
    }
    
    /// Create a snapshot ID from a string
    pub fn from_string(id: String) -> Self {
        SnapshotId(id)
    }
    
    /// Get the string representation of this ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A snapshot of an execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSnapshot {
    /// Unique identifier for this snapshot
    pub snapshot_id: SnapshotId,
    /// The context ID this snapshot belongs to
    pub context_id: ContextId,
    /// The timestamp when this snapshot was created
    pub created_at: SystemTime,
    /// The execution position (event index) in the trace
    pub execution_position: usize,
    /// Variable bindings at the time of snapshot
    pub variables: HashMap<String, serde_json::Value>,
    /// Call stack at the time of snapshot
    pub call_stack: Vec<CallFrame>,
    /// Resource usage at the time of snapshot
    pub resource_usage: ResourceUsage,
    /// Active resource grants
    pub active_grants: Vec<GrantId>,
    /// Parent snapshot ID, if this is an incremental snapshot
    pub parent_snapshot_id: Option<SnapshotId>,
}

impl ExecutionSnapshot {
    /// Create a new snapshot from an execution context
    pub fn new(
        context: &ExecutionContext,
        execution_position: usize,
        parent_snapshot_id: Option<SnapshotId>,
    ) -> Result<Self> {
        Ok(ExecutionSnapshot {
            snapshot_id: SnapshotId::new(),
            context_id: context.context_id().clone(),
            created_at: SystemTime::now(),
            execution_position,
            variables: context.get_all_variables()?,
            call_stack: context.get_call_stack()?,
            resource_usage: context.get_resource_usage()?,
            active_grants: context.get_active_grants()?,
            parent_snapshot_id,
        })
    }
}

/// An error that can occur during snapshot operations
#[derive(Debug)]
pub enum SnapshotError {
    /// The snapshot was not found
    NotFound(String),
    /// The snapshot could not be created
    CreationFailed(String),
    /// The snapshot could not be restored
    RestoreFailed(String),
    /// The snapshot could not be serialized or deserialized
    SerializationError(String),
    /// The snapshot is invalid or corrupt
    InvalidSnapshot(String),
    /// The storage backend failed
    StorageError(String),
    /// A generic error occurred
    Other(String),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotError::NotFound(msg) => write!(f, "Snapshot not found: {}", msg),
            SnapshotError::CreationFailed(msg) => write!(f, "Failed to create snapshot: {}", msg),
            SnapshotError::RestoreFailed(msg) => write!(f, "Failed to restore snapshot: {}", msg),
            SnapshotError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SnapshotError::InvalidSnapshot(msg) => write!(f, "Invalid snapshot: {}", msg),
            SnapshotError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            SnapshotError::Other(msg) => write!(f, "Snapshot error: {}", msg),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl From<SnapshotError> for Error {
    fn from(err: SnapshotError) -> Self {
        Error::SnapshotError(err.to_string())
    }
}

/// Interface for snapshot management
pub trait SnapshotManager: Send + Sync {
    /// Create a snapshot of the current execution context
    fn create_snapshot(
        &self,
        context: &ExecutionContext,
    ) -> std::result::Result<SnapshotId, SnapshotError>;
    
    /// Restore execution from a snapshot
    fn restore_snapshot(
        &self,
        snapshot_id: &SnapshotId,
    ) -> std::result::Result<ExecutionContext, SnapshotError>;
    
    /// List available snapshots for a context
    fn list_snapshots(
        &self,
        context_id: &ContextId,
    ) -> std::result::Result<Vec<ExecutionSnapshot>, SnapshotError>;
    
    /// Delete a snapshot
    fn delete_snapshot(
        &self,
        snapshot_id: &SnapshotId,
    ) -> std::result::Result<(), SnapshotError>;
    
    /// Get a specific snapshot by ID
    fn get_snapshot(
        &self,
        snapshot_id: &SnapshotId,
    ) -> std::result::Result<ExecutionSnapshot, SnapshotError>;
    
    /// Create an incremental snapshot
    fn create_incremental_snapshot(
        &self,
        context: &ExecutionContext,
        parent_snapshot_id: &SnapshotId,
    ) -> std::result::Result<SnapshotId, SnapshotError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_id_creation() {
        let id1 = SnapshotId::new();
        let id2 = SnapshotId::new();
        
        // Ensure IDs are different
        assert_ne!(id1, id2);
        
        // Test string conversion
        let id_str = id1.to_string();
        let id_from_str = SnapshotId::from_string(id_str.clone());
        assert_eq!(id1, id_from_str);
        assert_eq!(id_from_str.as_str(), id_str);
    }
} 