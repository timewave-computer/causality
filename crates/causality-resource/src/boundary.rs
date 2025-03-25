// Resource boundary management
// Original file: src/resource/boundary_manager.rs

// Boundary-Aware Resource Management
//
// This module implements boundary awareness for resource management,
// allowing resources to be safely operated on across different system boundaries.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::boundary::{BoundaryType, CrossingType, BoundarySafe};
use crate::resource::{ContentId};
use causality_resource_manager::ResourceRegisterLifecycleManager;
use causality_resource::{RegisterState, StorageStrategy, ResourceRegister};
use causality_types::{Error, Result};

/// Represents a resource crossing operation across boundaries
#[derive(Debug, Clone)]
pub struct ResourceBoundaryCrossing {
    /// The resource ID being crossed
    pub resource_id: ContentId,
    
    /// Source boundary
    pub source_boundary: BoundaryType,
    
    /// Target boundary
    pub target_boundary: BoundaryType,
    
    /// Crossing type
    pub crossing_type: CrossingType,
    
    /// Strategy for handling the resource in the target boundary
    pub crossing_strategy: ResourceCrossingStrategy,
}

/// Strategies for handling resources when crossing boundaries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCrossingStrategy {
    /// Full resource is copied across the boundary
    FullCopy,
    
    /// Only a commitment to the resource crosses the boundary
    CommitmentOnly,
    
    /// Only specific fields cross the boundary
    SelectedFields,
    
    /// The resource is locked in source boundary while being used in target boundary
    LockAndReference,
}

/// Manager for boundary-aware resource operations
pub struct BoundaryAwareResourceManager {
    /// Core lifecycle manager
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
    
    /// Resources that have crossed boundaries
    crossed_resources: RwLock<HashMap<ContentId, HashSet<BoundaryType>>>,
    
    /// Default crossing strategies by boundary type
    default_strategies: RwLock<HashMap<(BoundaryType, BoundaryType), ResourceCrossingStrategy>>,
}

impl BoundaryAwareResourceManager {
    /// Create a new boundary-aware resource manager
    pub fn new(lifecycle_manager: Arc<ResourceRegisterLifecycleManager>) -> Self {
        let mut default_strategies = HashMap::new();
        
        // Set up default crossing strategies
        default_strategies.insert(
            (BoundaryType::InsideSystem, BoundaryType::OutsideSystem),
            ResourceCrossingStrategy::CommitmentOnly
        );
        
        default_strategies.insert(
            (BoundaryType::OutsideSystem, BoundaryType::InsideSystem),
            ResourceCrossingStrategy::FullCopy
        );
        
        default_strategies.insert(
            (BoundaryType::InsideSystem, BoundaryType::OnChain),
            ResourceCrossingStrategy::CommitmentOnly
        );
        
        default_strategies.insert(
            (BoundaryType::OnChain, BoundaryType::OffChain),
            ResourceCrossingStrategy::SelectedFields
        );
        
        Self {
            lifecycle_manager,
            crossed_resources: RwLock::new(HashMap::new()),
            default_strategies: RwLock::new(default_strategies),
        }
    }
    
    /// Set a default strategy for a boundary crossing
    pub fn set_default_strategy(
        &self,
        source: BoundaryType,
        target: BoundaryType,
        strategy: ResourceCrossingStrategy
    ) {
        let mut strategies = self.default_strategies.write().unwrap();
        strategies.insert((source, target), strategy);
    }
    
    /// Get the default strategy for a boundary crossing
    pub fn default_strategy(
        &self,
        source: BoundaryType,
        target: BoundaryType
    ) -> ResourceCrossingStrategy {
        let strategies = self.default_strategies.read().unwrap();
        *strategies.get(&(source, target))
            .unwrap_or(&ResourceCrossingStrategy::CommitmentOnly)
    }
    
    /// Check if a resource can cross a boundary
    pub fn can_cross_boundary(
        &self,
        resource_id: &ContentId,
        source: BoundaryType,
        target: BoundaryType
    ) -> Result<bool> {
        // Get the resource
        let resource = self.lifecycle_manager.get_resource(resource_id)
            .ok_or_else(|| Error::ResourceNotFound(resource_id.clone()))?;
        
        // Check resource state
        if !resource.is_active() {
            return Ok(false);
        }
        
        // Check if a resource with on-chain commitment can be sent to on-chain
        if resource.storage_strategy == StorageStrategy::CommitmentBased { 
            if matches!(target, BoundaryType::OnChain | BoundaryType::EVM | BoundaryType::CosmWasm) {
                // Already has a commitment format suitable for on-chain storage
                return Ok(true);
            }
        }
        
        // Certain storage strategies might not be allowed to cross specific boundaries
        match resource.storage_strategy {
            StorageStrategy::FullyOnChain { .. } => {
                // On-chain resources can generally cross to any boundary
                Ok(true)
            },
            StorageStrategy::Hybrid { .. } => {
                // Hybrid resources are flexible and can cross most boundaries
                Ok(true)
            },
            StorageStrategy::CommitmentBased { .. } => {
                // Commitment-based resources can cross any boundary as their commitment
                Ok(true)
            }
        }
    }
    
    /// Prepare a resource for crossing a boundary
    pub fn prepare_for_crossing(
        &self,
        resource_id: &ContentId,
        source: BoundaryType,
        target: BoundaryType
    ) -> Result<ResourceBoundaryCrossing> {
        // Check if the resource can cross this boundary
        if !self.can_cross_boundary(resource_id, source, target)? {
            return Err(Error::InvalidOperation(
                format!("Resource {} cannot cross from {:?} to {:?}", resource_id, source, target)
            ));
        }
        
        // Determine crossing type
        let crossing_type = match (source, target) {
            (BoundaryType::InsideSystem, BoundaryType::OutsideSystem) => CrossingType::InsideToOutside,
            (BoundaryType::OutsideSystem, BoundaryType::InsideSystem) => CrossingType::OutsideToInside,
            (BoundaryType::OffChain, BoundaryType::OnChain) => CrossingType::OffChainToOnChain,
            (BoundaryType::OnChain, BoundaryType::OffChain) => CrossingType::OnChainToOffChain,
            _ => CrossingType::Custom(format!("{:?}_to_{:?}", source, target))
        };
        
        // Get the crossing strategy
        let strategy = self.default_strategy(source, target);
        
        // Create the crossing
        let crossing = ResourceBoundaryCrossing {
            resource_id: resource_id.clone(),
            source_boundary: source,
            target_boundary: target,
            crossing_type,
            crossing_strategy: strategy,
        };
        
        // If resource needs to be locked during crossing, lock it
        if strategy == ResourceCrossingStrategy::LockAndReference {
            self.lifecycle_manager.transition_state(resource_id, RegisterState::Locked)?;
        }
        
        // Track that this resource has crossed into the target boundary
        {
            let mut crossed = self.crossed_resources.write().unwrap();
            crossed.entry(resource_id.clone())
                .or_insert_with(HashSet::new)
                .insert(target);
        }
        
        Ok(crossing)
    }
    
    /// Complete a resource crossing
    pub fn complete_crossing(&self, crossing: &ResourceBoundaryCrossing) -> Result<()> {
        // If resource was locked during crossing, unlock it
        if crossing.crossing_strategy == ResourceCrossingStrategy::LockAndReference {
            self.lifecycle_manager.transition_state(&crossing.resource_id, RegisterState::Active)?;
        }
        
        Ok(())
    }
    
    /// Get all boundaries that a resource exists in
    pub fn resource_boundaries(&self, resource_id: &ContentId) -> HashSet<BoundaryType> {
        let crossed = self.crossed_resources.read().unwrap();
        crossed.get(resource_id)
            .cloned()
            .unwrap_or_else(|| {
                // If resource hasn't crossed any boundaries, it exists only in the inside system
                let mut set = HashSet::new();
                set.insert(BoundaryType::InsideSystem);
                set
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boundary_crossing_strategies() {
        // This test would verify that the different crossing strategies work correctly
    }
    
    #[test]
    fn test_resource_boundaries_tracking() {
        // This test would verify that we correctly track which boundaries a resource exists in
    }
} 
