// Time Map Integration with Resource Lifecycle Management
//
// This module provides the integration between the Time Map and Resource Lifecycle systems,
// ensuring temporal consistency for resources across domains.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::{ResourceId, DomainId, Timestamp};
use crate::time::{TimeMap, TimeMapSnapshot, TimeObserver, TimeEvent};
use crate::resource::{ResourceRegister, RegisterState, ResourceManager};
use crate::resource::lifecycle_manager::ResourceRegisterLifecycleManager;

/// The resource time map integration service
pub struct ResourceTimeMapIntegration {
    /// The time map service
    time_map: Arc<TimeMap>,
    
    /// Snapshot of resource states at various times
    resource_snapshots: HashMap<ResourceId, Vec<ResourceTimeSnapshot>>,
    
    /// Last synchronization timestamp
    last_sync: Timestamp,
}

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

impl ResourceTimeMapIntegration {
    /// Create a new resource time map integration
    pub fn new(time_map: Arc<TimeMap>) -> Self {
        Self {
            time_map,
            resource_snapshots: HashMap::new(),
            last_sync: 0, // Unix timestamp 0 (1970-01-01)
        }
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
        let lifecycle_manager = ResourceRegisterLifecycleManager::new();
        if !lifecycle_manager.is_valid_transition(&from_state, &to_state) {
            return Ok(false);
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
        
        Ok(pruned_count)
    }
}

/// Implementation of TimeObserver for ResourceTimeMapIntegration
impl TimeObserver for ResourceTimeMapIntegration {
    fn on_time_event(&mut self, event: TimeEvent) -> Result<()> {
        match event {
            TimeEvent::NewSnapshot(snapshot) => {
                // Could trigger a scan of resources for potential updates
                log::info!("New time map snapshot available: {:?}", snapshot.timestamp);
            },
            TimeEvent::SyncRequest => {
                // We'll handle this in a separate synchronization call
                log::info!("Time map sync requested");
            },
            TimeEvent::InconsistencyDetected(error) => {
                log::warn!("Time map inconsistency detected: {}", error);
                // Could trigger a conflict resolution process
            },
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_and_retrieve_state() -> Result<()> {
        let time_map = Arc::new(TimeMap::new());
        let mut integration = ResourceTimeMapIntegration::new(time_map);
        
        let resource_id = ResourceId::new();
        let domain_id = DomainId::new();
        
        // Record an initial state
        let snapshot1 = integration.record_state_change(
            &resource_id, 
            RegisterState::Initial,
            &domain_id,
        )?;
        
        // Get the state at that time
        let state1 = integration.get_resource_state_at_time(
            &resource_id,
            &snapshot1,
        )?;
        
        assert!(state1.is_some());
        assert_eq!(state1.unwrap().state, RegisterState::Initial);
        
        // Record a new state
        let snapshot2 = integration.record_state_change(
            &resource_id, 
            RegisterState::Active,
            &domain_id,
        )?;
        
        // Get the state at the new time
        let state2 = integration.get_resource_state_at_time(
            &resource_id,
            &snapshot2,
        )?;
        
        assert!(state2.is_some());
        assert_eq!(state2.unwrap().state, RegisterState::Active);
        
        // The original state should still be available when requesting the first snapshot
        let original_state = integration.get_resource_state_at_time(
            &resource_id,
            &snapshot1,
        )?;
        
        assert!(original_state.is_some());
        assert_eq!(original_state.unwrap().state, RegisterState::Initial);
        
        Ok(())
    }
    
    #[test]
    fn test_verify_state_transition() -> Result<()> {
        let time_map = Arc::new(TimeMap::new());
        let mut integration = ResourceTimeMapIntegration::new(time_map);
        
        let resource_id = ResourceId::new();
        let domain_id = DomainId::new();
        
        // Record an initial state
        let snapshot = integration.record_state_change(
            &resource_id, 
            RegisterState::Initial,
            &domain_id,
        )?;
        
        // Verify a valid transition
        let is_valid1 = integration.verify_state_transition(
            &resource_id,
            RegisterState::Initial,
            RegisterState::Active,
            &snapshot,
        )?;
        
        assert!(is_valid1);
        
        // Verify an invalid transition (Initial -> Consumed)
        let is_valid2 = integration.verify_state_transition(
            &resource_id,
            RegisterState::Initial,
            RegisterState::Consumed,
            &snapshot,
        )?;
        
        assert!(!is_valid2);
        
        // Verify with incorrect from_state
        let is_valid3 = integration.verify_state_transition(
            &resource_id,
            RegisterState::Active, // This doesn't match our recorded Initial state
            RegisterState::Locked,
            &snapshot,
        )?;
        
        assert!(!is_valid3);
        
        Ok(())
    }
} 