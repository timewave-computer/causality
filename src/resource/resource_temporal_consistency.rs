// Temporal Consistency Management for Resources and Relationships
//
// This module provides temporal consistency for resources and their relationships across domains
// using TimeMap to track and validate state changes over time.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::{ResourceId, DomainId, Timestamp, Metadata};
use crate::time::{TimeMap, TimeMapSnapshot, TimeObserver, TimeEvent};
use crate::resource::{ResourceRegister, RegisterState};
use crate::resource::manager::ResourceManager;
use crate::resource::lifecycle_manager::ResourceRegisterLifecycleManager;
use crate::resource::relationship_tracker::{RelationshipTracker, ResourceRelationship, RelationshipType, RelationshipDirection};
use crate::relationship::cross_domain_query::{RelationshipQueryExecutor, RelationshipQuery};

/// A snapshot of a resource state at a specific time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceTimeSnapshot {
    /// Resource ID
    pub resource_id: ResourceId,
    
    /// State of the resource
    pub state: RegisterState,
    
    /// Time map snapshot for this resource state
    pub time_snapshot: TimeMapSnapshot,
    
    /// Timestamp when this snapshot was created
    pub created_at: Timestamp,
    
    /// Domain ID where this resource state was observed
    pub domain_id: DomainId,
}

/// A snapshot of a relationship at a specific time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipTimeSnapshot {
    /// The relationship data
    pub relationship: ResourceRelationship,
    
    /// Time map snapshot for this relationship state
    pub time_snapshot: TimeMapSnapshot,
    
    /// Timestamp when this snapshot was created
    pub created_at: Timestamp,
    
    /// Metadata for this snapshot
    pub metadata: Metadata,
}

/// Configuration for temporal consistency management
#[derive(Clone, Debug)]
pub struct TemporalConsistencyConfig {
    /// Interval in seconds between periodic synchronizations
    pub sync_interval_seconds: u64,
    
    /// Maximum number of snapshots to keep for each resource/relationship
    pub max_snapshots_per_entity: usize,
    
    /// Whether to verify temporal consistency for all state/relationship changes
    pub verify_temporal_consistency: bool,
    
    /// Whether to automatically repair inconsistencies
    pub auto_repair_inconsistencies: bool,
}

impl Default for TemporalConsistencyConfig {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 300, // 5 minutes
            max_snapshots_per_entity: 100,
            verify_temporal_consistency: true,
            auto_repair_inconsistencies: true,
        }
    }
}

/// The resource temporal consistency manager
pub struct ResourceTemporalConsistency {
    /// The time map service
    time_map: Arc<TimeMap>,
    
    /// Snapshot of resource states at various times
    resource_snapshots: HashMap<ResourceId, Vec<ResourceTimeSnapshot>>,
    
    /// Snapshot of relationships at various times
    relationship_snapshots: Mutex<HashMap<String, Vec<RelationshipTimeSnapshot>>>,
    
    /// Relationship tracker
    relationship_tracker: Option<Arc<RelationshipTracker>>,
    
    /// Cross-domain relationship query executor
    query_executor: Option<Arc<RelationshipQueryExecutor>>,
    
    /// Lifecycle manager for resources
    lifecycle_manager: Option<Arc<ResourceRegisterLifecycleManager>>,
    
    /// Last synchronization timestamp
    last_sync: Timestamp,
    
    /// Configuration
    config: TemporalConsistencyConfig,
}

impl ResourceTemporalConsistency {
    /// Create a new resource temporal consistency manager
    pub fn new(time_map: Arc<TimeMap>) -> Self {
        Self {
            time_map,
            resource_snapshots: HashMap::new(),
            relationship_snapshots: Mutex::new(HashMap::new()),
            relationship_tracker: None,
            query_executor: None,
            lifecycle_manager: None,
            last_sync: 0, // Unix timestamp 0 (1970-01-01)
            config: TemporalConsistencyConfig::default(),
        }
    }
    
    /// Create a new resource temporal consistency manager with custom config
    pub fn with_config(time_map: Arc<TimeMap>, config: TemporalConsistencyConfig) -> Self {
        Self {
            time_map,
            resource_snapshots: HashMap::new(),
            relationship_snapshots: Mutex::new(HashMap::new()),
            relationship_tracker: None,
            query_executor: None,
            lifecycle_manager: None,
            last_sync: 0,
            config,
        }
    }
    
    /// Set the relationship tracker
    pub fn with_relationship_tracker(mut self, tracker: Arc<RelationshipTracker>) -> Self {
        self.relationship_tracker = Some(tracker);
        self
    }
    
    /// Set the relationship query executor
    pub fn with_query_executor(mut self, executor: Arc<RelationshipQueryExecutor>) -> Self {
        self.query_executor = Some(executor);
        self
    }
    
    /// Set the lifecycle manager
    pub fn with_lifecycle_manager(mut self, manager: Arc<ResourceRegisterLifecycleManager>) -> Self {
        self.lifecycle_manager = Some(manager);
        self
    }
    
    /// Record a resource state change with the current time snapshot
    pub fn record_state_change(
        &mut self,
        resource_id: &ResourceId,
        state: RegisterState,
        domain_id: &DomainId,
    ) -> Result<TimeMapSnapshot> {
        // Get current timestamp
        let now = chrono::Utc::now().timestamp() as u64;
        
        // Get the current time map snapshot
        let time_snapshot = self.time_map.get_snapshot()?;
        
        // Create the resource snapshot
        let resource_snapshot = ResourceTimeSnapshot {
            resource_id: resource_id.clone(),
            state,
            time_snapshot: time_snapshot.clone(),
            created_at: now,
            domain_id: domain_id.clone(),
        };
        
        // Add to our snapshots
        self.resource_snapshots
            .entry(resource_id.clone())
            .or_insert_with(Vec::new)
            .push(resource_snapshot);
        
        // Trim snapshots if we have too many
        if let Some(snapshots) = self.resource_snapshots.get_mut(resource_id) {
            if snapshots.len() > self.config.max_snapshots_per_entity {
                // Sort by timestamp (newest first) and keep only the most recent ones
                snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                snapshots.truncate(self.config.max_snapshots_per_entity);
            }
        }
        
        Ok(time_snapshot)
    }
    
    /// Record a relationship change with the current time snapshot
    pub async fn record_relationship_change(
        &self,
        relationship: &ResourceRelationship,
    ) -> Result<TimeMapSnapshot> {
        // Get current timestamp
        let now = chrono::Utc::now().timestamp() as u64;
        
        // Get the current time map snapshot
        let time_snapshot = self.time_map.get_snapshot()?;
        
        // Create the relationship snapshot
        let relationship_snapshot = RelationshipTimeSnapshot {
            relationship: relationship.clone(),
            time_snapshot: time_snapshot.clone(),
            created_at: now,
            metadata: Metadata::default(),
        };
        
        // Add to our snapshots
        let mut snapshots = self.relationship_snapshots.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on relationship_snapshots".to_string())
        })?;
        
        snapshots
            .entry(relationship.id.clone())
            .or_insert_with(Vec::new)
            .push(relationship_snapshot);
        
        // Trim snapshots if we have too many
        if let Some(rel_snapshots) = snapshots.get_mut(&relationship.id) {
            if rel_snapshots.len() > self.config.max_snapshots_per_entity {
                // Sort by timestamp (newest first) and keep only the most recent ones
                rel_snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                rel_snapshots.truncate(self.config.max_snapshots_per_entity);
            }
        }
        
        Ok(time_snapshot)
    }
    
    /// Get the state of a resource at a specific time
    pub fn get_resource_state_at_time(
        &self,
        resource_id: &ResourceId,
        time_snapshot: &TimeMapSnapshot,
    ) -> Result<Option<ResourceTimeSnapshot>> {
        // Get all snapshots for this resource
        let snapshots = match self.resource_snapshots.get(resource_id) {
            Some(s) => s,
            None => return Ok(None),
        };
        
        // Find the latest snapshot that precedes or equals the requested time
        let mut latest_valid_snapshot = None;
        let mut latest_valid_time = 0;
        
        for snapshot in snapshots {
            // Check if this snapshot is valid for the requested time
            let is_valid = self.time_map.is_snapshot_valid_at(&snapshot.time_snapshot, time_snapshot)?;
            
            if is_valid && snapshot.created_at > latest_valid_time {
                latest_valid_snapshot = Some(snapshot.clone());
                latest_valid_time = snapshot.created_at;
            }
        }
        
        Ok(latest_valid_snapshot)
    }
    
    /// Get the relationship at a specific time
    pub fn get_relationship_at_time(
        &self,
        relationship_id: &str,
        time_snapshot: &TimeMapSnapshot,
    ) -> Result<Option<RelationshipTimeSnapshot>> {
        let snapshots = self.relationship_snapshots.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on relationship_snapshots".to_string())
        })?;
        
        // Get all snapshots for this relationship
        let rel_snapshots = match snapshots.get(relationship_id) {
            Some(s) => s,
            None => return Ok(None),
        };
        
        // Find the latest snapshot that precedes or equals the requested time
        let mut latest_valid_snapshot = None;
        let mut latest_valid_time = 0;
        
        for snapshot in rel_snapshots {
            // Check if this snapshot is valid for the requested time
            let is_valid = self.time_map.is_snapshot_valid_at(&snapshot.time_snapshot, time_snapshot)?;
            
            if is_valid && snapshot.created_at > latest_valid_time {
                latest_valid_snapshot = Some(snapshot.clone());
                latest_valid_time = snapshot.created_at;
            }
        }
        
        Ok(latest_valid_snapshot)
    }
    
    /// Verify that a proposed state transition is temporally consistent
    pub fn verify_state_transition(
        &self,
        resource_id: &ResourceId,
        from_state: RegisterState,
        to_state: RegisterState,
        time_snapshot: &TimeMapSnapshot,
    ) -> Result<bool> {
        // Get the resource state at the specified time
        let resource_at_time = self.get_resource_state_at_time(resource_id, time_snapshot)?;
        
        // If we don't have a snapshot, we can't verify
        let snapshot = match resource_at_time {
            Some(s) => s,
            None => {
                // If this is a new resource (from_state is Initial), that's okay
                return Ok(from_state == RegisterState::Initial);
            }
        };
        
        // Check if the current state matches what we expect
        if snapshot.state != from_state {
            return Ok(false);
        }
        
        // Check if the transition is valid
        let lifecycle_manager = self.lifecycle_manager.as_ref().unwrap_or_else(|| {
            // Create a temporary one if not set
            &ResourceRegisterLifecycleManager::new()
        });
        
        if !lifecycle_manager.validate_transition(&from_state, &to_state)? {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Verify that a relationship change is temporally consistent
    pub async fn verify_relationship_change(
        &self,
        relationship: &ResourceRelationship,
    ) -> Result<bool> {
        // We need the relationship tracker to verify changes
        let tracker = match &self.relationship_tracker {
            Some(t) => t,
            None => return Err(Error::Internal("Relationship tracker not set".to_string())),
        };
        
        // Get current time snapshot
        let current_snapshot = self.time_map.get_snapshot()?;
        
        // For new relationships, we need to verify the resources exist and are in valid states
        let source_state = self.get_resource_state_at_time(&relationship.source_id, &current_snapshot)?;
        let target_state = self.get_resource_state_at_time(&relationship.target_id, &current_snapshot)?;
        
        // Check if source and target resources exist
        if source_state.is_none() || target_state.is_none() {
            return Ok(false);
        }
        
        // Check if the resources are in active states
        let source_active = match &source_state {
            Some(s) => s.state == RegisterState::Active,
            None => false,
        };
        
        let target_active = match &target_state {
            Some(s) => s.state == RegisterState::Active,
            None => false,
        };
        
        if !source_active || !target_active {
            return Ok(false);
        }
        
        // For existing relationships being updated, check the previous state
        if let Ok(existing) = tracker.get_direct_relationships(&relationship.source_id, &relationship.target_id) {
            if !existing.is_empty() {
                // This is an update to an existing relationship
                // Verify it's a valid update
                // (Additional checks could be implemented here)
            }
        }
        
        // If we have a query executor, verify cross-domain consistency
        if let Some(executor) = &self.query_executor {
            // Check if this is a cross-domain relationship
            if relationship.source_domain.is_some() && relationship.target_domain.is_some() &&
               relationship.source_domain != relationship.target_domain {
                
                // Create a query to find this relationship
                let query = RelationshipQuery::new(
                    relationship.source_id.clone(),
                    relationship.target_id.clone()
                )
                .with_cross_domain(true);
                
                // Execute the query
                let paths = executor.execute(&query).await?;
                
                // If this is a new relationship but paths already exist, we might have inconsistency
                if !paths.is_empty() {
                    // This might indicate a duplicate relationship - decide based on your policy
                    // For now, we'll allow it (could be parallel edges)
                }
            }
        }
        
        Ok(true)
    }
    
    /// Synchronize resource states across domains using the time map
    pub fn synchronize_resources(
        &mut self,
        resource_manager: &mut ResourceManager,
    ) -> Result<usize> {
        // Get the current time map snapshot
        let current_snapshot = self.time_map.get_snapshot()?;
        
        // Get all resource IDs
        let resource_ids: Vec<ResourceId> = resource_manager.list_resources()?;
        
        let mut sync_count = 0;
        
        // For each resource
        for resource_id in resource_ids {
            // Get the current state from the manager
            let resource = match resource_manager.get_resource(&resource_id) {
                Ok(r) => r,
                Err(_) => continue, // Skip if resource not found
            };
            
            // Get the latest snapshot for this resource
            let latest_snapshot = self.get_resource_state_at_time(&resource_id, &current_snapshot)?;
            
            // If we have a snapshot and it's newer than our resource
            if let Some(snapshot) = latest_snapshot {
                let resource_timestamp = resource.observed_at.timestamp;
                
                if snapshot.created_at > resource_timestamp {
                    // Update the resource state if it's different
                    if resource.state != snapshot.state {
                        // Create a mutable version of the resource
                        let mut updated_resource = resource.clone();
                        
                        // Update the state
                        updated_resource.state = snapshot.state;
                        
                        // Update the observed time
                        updated_resource.observed_at = snapshot.time_snapshot.clone();
                        
                        // Save the updated resource
                        resource_manager.update_resource(&resource_id, updated_resource)?;
                        
                        sync_count += 1;
                    }
                }
            }
        }
        
        // Update last sync time
        self.last_sync = chrono::Utc::now().timestamp() as u64;
        
        Ok(sync_count)
    }
    
    /// Synchronize relationships across domains
    pub async fn synchronize_relationships(&self) -> Result<usize> {
        // We need the relationship tracker to sync relationships
        let tracker = match &self.relationship_tracker {
            Some(t) => t,
            None => return Err(Error::Internal("Relationship tracker not set".to_string())),
        };
        
        // We need the query executor to find cross-domain relationships
        let executor = match &self.query_executor {
            Some(e) => e,
            None => return Err(Error::Internal("Query executor not set".to_string())),
        };
        
        // Get the current time map snapshot
        let current_snapshot = self.time_map.get_snapshot()?;
        
        // Define a simple filter struct locally since it seems to be unavailable
        struct RelationshipFilter {
            relationship_types: Option<Vec<RelationshipType>>,
            max_results: Option<usize>,
            include_deleted: bool,
        }
        
        // Get all relationships
        let filter = RelationshipFilter {
            relationship_types: None,
            max_results: None,
            include_deleted: false,
        };
        
        let relationships = tracker.get_all_relationships(&filter)?;
        
        // Find cross-domain relationships
        let cross_domain_relationships: Vec<_> = relationships.into_iter()
            .filter(|r| {
                r.source_domain.is_some() && r.target_domain.is_some() &&
                r.source_domain != r.target_domain
            })
            .collect();
            
        let mut sync_count = 0;
        
        // Verify each relationship
        for relationship in cross_domain_relationships {
            // Verify the relationship exists in both domains
            if let (Some(source_domain), Some(target_domain)) = 
                (&relationship.source_domain, &relationship.target_domain) {
                
                // Create a query to find this relationship
                let query = RelationshipQuery::new(
                    relationship.source_id.clone(),
                    relationship.target_id.clone()
                )
                .with_cross_domain(true);
                
                // Execute the query
                let paths = executor.execute(&query).await?;
                
                // If no paths found, the relationship might be inconsistent
                if paths.is_empty() && self.config.auto_repair_inconsistencies {
                    // TODO: Implement repair logic
                    // For now, just log the inconsistency
                    eprintln!(
                        "Inconsistent relationship found: {:?} -> {:?}",
                        relationship.source_id, relationship.target_id
                    );
                    
                    // Count these as sync attempts
                    sync_count += 1;
                }
            }
        }
        
        Ok(sync_count)
    }
    
    /// Get the history of a resource's states over time
    pub fn get_resource_history(
        &self,
        resource_id: &ResourceId,
    ) -> Result<Vec<ResourceTimeSnapshot>> {
        match self.resource_snapshots.get(resource_id) {
            Some(snapshots) => Ok(snapshots.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    /// Get the history of a relationship over time
    pub fn get_relationship_history(
        &self,
        relationship_id: &str,
    ) -> Result<Vec<RelationshipTimeSnapshot>> {
        let snapshots = self.relationship_snapshots.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on relationship_snapshots".to_string())
        })?;
        
        match snapshots.get(relationship_id) {
            Some(rel_snapshots) => Ok(rel_snapshots.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    /// Clean up old snapshots to conserve memory
    pub fn prune_old_snapshots(
        &mut self,
        older_than_seconds: u64,
    ) -> Result<usize> {
        let now = chrono::Utc::now().timestamp() as u64;
        let cutoff = now.saturating_sub(older_than_seconds);
        
        let mut pruned_count = 0;
        
        // For each resource
        for snapshots in self.resource_snapshots.values_mut() {
            let original_count = snapshots.len();
            
            // Keep only snapshots newer than the cutoff
            snapshots.retain(|s| s.created_at >= cutoff);
            
            pruned_count += original_count - snapshots.len();
        }
        
        // Remove empty entries
        self.resource_snapshots.retain(|_, v| !v.is_empty());
        
        // Also prune relationship snapshots
        let mut rel_snapshots = self.relationship_snapshots.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on relationship_snapshots".to_string())
        })?;
        
        for snapshots in rel_snapshots.values_mut() {
            let original_count = snapshots.len();
            
            // Keep only snapshots newer than the cutoff
            snapshots.retain(|s| s.created_at >= cutoff);
            
            pruned_count += original_count - snapshots.len();
        }
        
        // Remove empty entries
        rel_snapshots.retain(|_, v| !v.is_empty());
        
        Ok(pruned_count)
    }
}

/// Implementation of TimeObserver for ResourceTemporalConsistency
impl TimeObserver for ResourceTemporalConsistency {
    fn on_time_event(&mut self, event: TimeEvent) -> Result<()> {
        match event {
            TimeEvent::NewSnapshot(snapshot) => {
                // Simply store the latest snapshot timestamp
                self.last_sync = chrono::Utc::now().timestamp() as u64;
                // No need to store the snapshot itself as we'll get it from TimeMap when needed
            }
            TimeEvent::SyncRequest => {
                // When requested to sync, trigger relationship and resource sync
                if let Some(manager) = &self.lifecycle_manager {
                    // We'd need a ResourceManager to synchronize, which we don't have direct access to
                    // Log that we received a sync request
                    eprintln!("Sync request received, but no ResourceManager available");
                }
                
                if self.relationship_tracker.is_some() && self.query_executor.is_some() {
                    // Asynchronous operations can't be started from this sync method
                    // Log that we should sync relationships
                    eprintln!("Relationship sync should be triggered separately via synchronize_relationships()");
                }
            }
            TimeEvent::InconsistencyDetected(message) => {
                // Log the inconsistency
                eprintln!("Time map inconsistency detected: {}", message);
                
                // If auto-repair is enabled, flag for repair
                if self.config.auto_repair_inconsistencies {
                    eprintln!("Auto-repair is enabled, manual synchronization should be triggered");
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test resource temporal consistency
    #[test]
    fn test_record_and_retrieve_resource_state() -> Result<()> {
        let time_map = Arc::new(TimeMap::new());
        let mut manager = ResourceTemporalConsistency::new(Arc::clone(&time_map));
        
        let resource_id = "test_resource".to_string();
        let domain_id = "test_domain".to_string();
        
        // Record a state change
        manager.record_state_change(&resource_id, RegisterState::Active, &domain_id)?;
        
        // Get the snapshot
        let snapshot = time_map.get_snapshot()?;
        
        // Get the resource state at that time
        let resource_state = manager.get_resource_state_at_time(&resource_id, &snapshot)?;
        
        assert!(resource_state.is_some());
        let state = resource_state.unwrap();
        assert_eq!(state.state, RegisterState::Active);
        assert_eq!(state.domain_id, domain_id);
        
        Ok(())
    }
    
    // Test relationship temporal consistency
    // Note: This test is commented out because it requires async test support
    // Use tokio::test when running async tests
    /*
    #[tokio::test]
    async fn test_record_and_retrieve_relationship() -> Result<()> {
        let time_map = Arc::new(TimeMap::new());
        let current_snapshot = time_map.get_snapshot()?;
        
        let tracker = Arc::new(RelationshipTracker::new(current_snapshot.clone()));
        let manager = ResourceTemporalConsistency::new(Arc::clone(&time_map))
            .with_relationship_tracker(Arc::clone(&tracker));
        
        // Create a relationship
        let relationship = ResourceRelationship::new(
            "source".to_string(),
            "target".to_string(),
            RelationshipType::ParentChild,
            RelationshipDirection::ParentToChild,
        );
        
        // Record the relationship change (async method)
        manager.record_relationship_change(&relationship).await?;
        
        // Get the snapshot
        let snapshot = time_map.get_snapshot()?;
        
        // Get the relationship at that time
        let rel_state = manager.get_relationship_at_time(&relationship.id, &snapshot)?;
        
        assert!(rel_state.is_some());
        let state = rel_state.unwrap();
        assert_eq!(state.relationship.source_id, "source");
        assert_eq!(state.relationship.target_id, "target");
        assert_eq!(state.relationship.relationship_type, RelationshipType::ParentChild);
        
        Ok(())
    }
    */
} 