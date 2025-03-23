// Effect Templates for Common Operations
//
// This module provides templates for commonly used effect patterns,
// making it easier to create common operations without having to manually
// compose multiple effects.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::address::Address;
use crate::domain::DomainType;
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    create_composite_effect, ExecutionBoundary,
};
use crate::effect::storage::{
    create_store_on_chain_effect,
    create_read_from_chain_effect,
    create_store_commitment_effect,
    create_store_nullifier_effect,
};
use crate::error::{Error, Result};
use crate::resource::{
    ResourceId, 
    ResourceRegister, 
    RegisterState,
    RegisterOperationType,
    ResourceRegisterLifecycleManager,
    RelationshipTracker,
    RelationshipType,
};
use crate::types::{DomainId, Metadata};
use crate::time::TimeMapSnapshot;

/// Template for creating a new resource
pub fn create_resource_effect(
    resource: &ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Default fields to store for a new resource
    let fields = HashSet::from([
        "id".to_string(),
        "owner".to_string(),
        "state".to_string(),
        "metadata".to_string(),
    ]);
    
    // Create a storage effect based on the resource's storage strategy
    let storage_effect = match resource.storage_strategy {
        crate::resource::StorageStrategy::OnChain => {
            create_store_on_chain_effect(
                resource.id.clone(),
                fields,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Commitment => {
            // Generate a commitment for the resource
            let commitment = resource.generate_commitment()?;
            
            create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Nullifier => {
            // Generate a nullifier for the resource
            let nullifier = resource.generate_nullifier()?;
            
            create_store_nullifier_effect(
                resource.id.clone(),
                nullifier,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Hybrid => {
            // For hybrid storage, store both on-chain and a commitment
            let on_chain_effect = create_store_on_chain_effect(
                resource.id.clone(),
                fields.clone(),
                domain_id.clone(),
                invoker,
            );
            
            let commitment = resource.generate_commitment()?;
            let commitment_effect = create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            );
            
            // Combine the effects
            create_composite_effect(
                vec![on_chain_effect, commitment_effect],
                "Hybrid Storage".to_string(),
            )
        },
    };
    
    Ok(storage_effect)
}

/// Template for updating a resource's metadata
pub fn update_resource_effect(
    resource: &mut ResourceRegister,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Ensure resource is in a valid state for updates
    if resource.state != RegisterState::Active && 
       resource.state != RegisterState::Pending {
        return Err(Error::InvalidOperation(
            format!("Resource {} cannot be updated in state {:?}", 
                resource.id, resource.state)
        ));
    }
    
    // Create a storage effect based on the resource's storage strategy
    let storage_effect = match resource.storage_strategy {
        crate::resource::StorageStrategy::OnChain => {
            create_store_on_chain_effect(
                resource.id.clone(),
                fields,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Commitment => {
            // Generate a new commitment for the updated resource
            let commitment = resource.generate_commitment()?;
            
            create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Nullifier => {
            // For nullifier-based resources, updates create a new nullifier
            let nullifier = resource.generate_nullifier()?;
            
            create_store_nullifier_effect(
                resource.id.clone(),
                nullifier,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Hybrid => {
            // For hybrid storage, update both on-chain and commitment
            let on_chain_effect = create_store_on_chain_effect(
                resource.id.clone(),
                fields.clone(),
                domain_id.clone(),
                invoker,
            );
            
            let commitment = resource.generate_commitment()?;
            let commitment_effect = create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            );
            
            // Combine the effects
            create_composite_effect(
                vec![on_chain_effect, commitment_effect],
                "Hybrid Update".to_string(),
            )
        },
    };
    
    Ok(storage_effect)
}

/// Template for locking a resource
pub fn lock_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Lock the resource in the lifecycle manager
    resource.lifecycle_manager.lock(&resource.id, Some(&invoker))?;
    
    // If a relationship tracker is provided and we have a locker,
    // create a lock relationship between the resources
    if let Some(tracker) = resource.relationship_tracker.as_mut() {
        tracker.add_relationship(
            invoker.clone(),
            resource.id.clone(),
            RelationshipType::Lock,
            None,
        )?;
    }
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for unlocking a resource
pub fn unlock_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Unlock the resource in the lifecycle manager
    resource.lifecycle_manager.unlock(&resource.id, Some(&invoker))?;
    
    // If a relationship tracker is provided and we have an unlocker,
    // remove the lock relationship between the resources
    if let Some(tracker) = resource.relationship_tracker.as_mut() {
        tracker.remove_relationship_between(
            &invoker,
            &resource.id,
            Some(RelationshipType::Lock),
        )?;
    }
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for resource consumption
pub fn consume_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
    relationship_tracker: &mut RelationshipTracker,
) -> Result<Arc<dyn Effect>> {
    // Consume the resource in the lifecycle manager
    resource.lifecycle_manager.consume(&resource.id)?;
    
    // If a relationship tracker is provided and we have a consumer,
    // create a consumption relationship between the resources
    if let Some(consumer) = resource.relationship_tracker.as_mut() {
        consumer.add_relationship(
            invoker.clone(),
            resource.id.clone(),
            RelationshipType::Consumption,
            None,
        )?;
    }
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for transferring resource ownership
pub fn transfer_resource_effect(
    resource: &mut ResourceRegister,
    from: Address,
    to: Address,
    domain_id: DomainId,
) -> Result<Arc<dyn Effect>> {
    // Ensure resource is in a valid state for transfer
    if resource.state != RegisterState::Active {
        return Err(Error::InvalidOperation(
            format!("Resource {} cannot be transferred in state {:?}", 
                resource.id, resource.state)
        ));
    }
    
    // Update the owner
    resource.update_owner(to.clone())?;
    
    // Create a fields set for updating
    let fields = HashSet::from([
        "owner".to_string(),
        "metadata".to_string(), // Include metadata for transfer record
    ]);
    
    // Create a storage effect based on the resource's storage strategy
    let storage_effect = match resource.storage_strategy {
        crate::resource::StorageStrategy::OnChain => {
            create_store_on_chain_effect(
                resource.id.clone(),
                fields,
                domain_id,
                from,
            )
        },
        crate::resource::StorageStrategy::Commitment => {
            // Generate a new commitment for the transferred resource
            let commitment = resource.generate_commitment()?;
            
            create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                from,
            )
        },
        crate::resource::StorageStrategy::Nullifier => {
            // For nullifier-based resources, transfers create a new nullifier
            let nullifier = resource.generate_nullifier()?;
            
            create_store_nullifier_effect(
                resource.id.clone(),
                nullifier,
                domain_id,
                from,
            )
        },
        crate::resource::StorageStrategy::Hybrid => {
            // For hybrid storage, update both on-chain and commitment
            let on_chain_effect = create_store_on_chain_effect(
                resource.id.clone(),
                fields.clone(),
                domain_id.clone(),
                from.clone(),
            );
            
            let commitment = resource.generate_commitment()?;
            let commitment_effect = create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                from,
            );
            
            // Combine the effects
            create_composite_effect(
                vec![on_chain_effect, commitment_effect],
                "Hybrid Transfer".to_string(),
            )
        },
    };
    
    Ok(storage_effect)
}

/// Template for creating a parent-child relationship between resources
pub fn create_parent_child_effect(
    parent_id: ResourceId,
    child_id: ResourceId,
    relationship_tracker: &mut RelationshipTracker,
    metadata: Option<Metadata>,
) -> Result<Arc<dyn Effect>> {
    // Add a parent-child relationship
    relationship_tracker.add_parent_child_relationship(
        parent_id,
        child_id,
        metadata,
    )?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for creating a dependency relationship between resources
pub fn create_dependency_effect(
    dependent_id: ResourceId,
    dependency_id: ResourceId,
    relationship_tracker: &mut RelationshipTracker,
    metadata: Option<Metadata>,
) -> Result<Arc<dyn Effect>> {
    // Add a dependency relationship
    relationship_tracker.add_dependency_relationship(
        dependent_id,
        dependency_id,
        metadata,
    )?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for freezing a resource
pub fn freeze_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Freeze the resource in the lifecycle manager
    resource.lifecycle_manager.freeze(&resource.id)?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for unfreezing a resource
pub fn unfreeze_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Unfreeze the resource in the lifecycle manager
    resource.lifecycle_manager.unfreeze(&resource.id)?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for archiving a resource
pub fn archive_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Archive the resource in the lifecycle manager
    resource.lifecycle_manager.archive(&resource.id)?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for creating a resource with boundary awareness
pub fn create_resource_with_boundary_effect(
    resource: &ResourceRegister,
    boundary: ExecutionBoundary,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Create a basic resource effect
    let base_effect = create_resource_effect(resource, domain_id, invoker)?;
    
    // Wrap the effect with boundary information
    let boundary_effect = BoundaryAwareEffect::new(
        base_effect,
        boundary,
        format!("Create resource {} with boundary awareness", resource.id),
    );
    
    Ok(Arc::new(boundary_effect))
}

/// Template for cross-domain resource operations
pub fn cross_domain_resource_effect(
    resource: &ResourceRegister,
    source_domain: DomainId,
    target_domain: DomainId,
    invoker: Address,
    operation_type: RegisterOperationType,
) -> Result<Arc<dyn Effect>> {
    // Create appropriate base effect based on operation type
    let base_effect = match operation_type {
        RegisterOperationType::Create => {
            create_resource_effect(resource, target_domain.clone(), invoker.clone())?
        },
        RegisterOperationType::Update => {
            update_resource_effect(
                &mut resource.clone(), 
                resource.all_fields(), 
                target_domain.clone(), 
                invoker.clone()
            )?
        },
        RegisterOperationType::Lock => {
            lock_resource_effect(&mut resource.clone(), target_domain.clone(), invoker.clone())?
        },
        RegisterOperationType::Unlock => {
            unlock_resource_effect(&mut resource.clone(), target_domain.clone(), invoker.clone())?
        },
        RegisterOperationType::Consume => {
            // For consumption, we need a relationship tracker
            let mut tracker = RelationshipTracker::new();
            consume_resource_effect(
                &mut resource.clone(), 
                target_domain.clone(), 
                invoker.clone(),
                &mut tracker
            )?
        },
        _ => {
            return Err(Error::InvalidOperation(
                format!("Unsupported operation type {:?} for cross-domain effect", operation_type)
            ));
        }
    };
    
    // Create a domain transition effect that wraps the base effect
    let domain_transition_effect = DomainTransitionEffect::new(
        base_effect,
        source_domain,
        target_domain,
        format!("{:?} resource {} across domains", operation_type, resource.id),
    );
    
    Ok(Arc::new(domain_transition_effect))
}

/// Template for resource operation with capability validation
pub fn resource_operation_with_capability_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
    operation_type: RegisterOperationType,
    capability_ids: Vec<String>,
) -> Result<Arc<dyn Effect>> {
    // Validate the capability first
    let capability_validator = CapabilityValidationEffect::new(
        resource.id.clone(),
        operation_type,
        capability_ids,
        domain_id.clone(),
        invoker.clone(),
    );
    
    // Create the appropriate operation effect based on type
    let operation_effect = match operation_type {
        RegisterOperationType::Create => {
            create_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Update => {
            update_resource_effect(resource, resource.all_fields(), domain_id, invoker)?
        },
        RegisterOperationType::Lock => {
            lock_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Unlock => {
            unlock_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Freeze => {
            freeze_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Unfreeze => {
            unfreeze_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Consume => {
            let mut tracker = RelationshipTracker::new();
            consume_resource_effect(resource, domain_id, invoker, &mut tracker)?
        },
        RegisterOperationType::Archive => {
            archive_resource_effect(resource, domain_id, invoker)?
        },
        _ => {
            return Err(Error::InvalidOperation(
                format!("Unsupported operation type {:?} for capability effect", operation_type)
            ));
        }
    };
    
    // Chain the capability validator with the operation effect
    let combined_effect = create_composite_effect(
        vec![Arc::new(capability_validator), operation_effect],
        format!("Capability-validated {:?} operation", operation_type),
    );
    
    Ok(combined_effect)
}

/// Template for resource operation with time map validation
pub fn resource_operation_with_timemap_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
    operation_type: RegisterOperationType,
    time_map_snapshot: TimeMapSnapshot,
) -> Result<Arc<dyn Effect>> {
    // Create the time map validation effect
    let time_validation_effect = TimeMapValidationEffect::new(
        resource.id.clone(),
        time_map_snapshot.clone(),
        domain_id.clone(),
        invoker.clone(),
    );
    
    // Create the appropriate operation effect based on type
    let operation_effect = match operation_type {
        RegisterOperationType::Create => {
            create_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Update => {
            update_resource_effect(resource, resource.all_fields(), domain_id, invoker)?
        },
        RegisterOperationType::Lock => {
            lock_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Unlock => {
            unlock_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Freeze => {
            freeze_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Unfreeze => {
            unfreeze_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Consume => {
            let mut tracker = RelationshipTracker::new();
            consume_resource_effect(resource, domain_id, invoker, &mut tracker)?
        },
        RegisterOperationType::Archive => {
            archive_resource_effect(resource, domain_id, invoker)?
        },
        _ => {
            return Err(Error::InvalidOperation(
                format!("Unsupported operation type {:?} for time map effect", operation_type)
            ));
        }
    };
    
    // Update the resource's time map snapshot
    resource.update_timeframe(time_map_snapshot);
    
    // Chain the time validation with the operation effect
    let combined_effect = create_composite_effect(
        vec![Arc::new(time_validation_effect), operation_effect],
        format!("Time-map validated {:?} operation", operation_type),
    );
    
    Ok(combined_effect)
}

/// Template for resource operation with on-chain commitment
pub fn resource_operation_with_commitment_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
    operation_type: RegisterOperationType,
) -> Result<Arc<dyn Effect>> {
    // Generate commitment for the resource state
    let commitment = resource.generate_commitment()?;
    
    // Create commitment storage effect
    let commitment_effect = create_store_commitment_effect(
        resource.id.clone(),
        commitment,
        domain_id.clone(),
        invoker.clone(),
    );
    
    // Create the appropriate operation effect based on type
    let operation_effect = match operation_type {
        RegisterOperationType::Create => {
            create_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Update => {
            update_resource_effect(resource, resource.all_fields(), domain_id, invoker)?
        },
        RegisterOperationType::Lock => {
            lock_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Unlock => {
            unlock_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Freeze => {
            freeze_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Unfreeze => {
            unfreeze_resource_effect(resource, domain_id, invoker)?
        },
        RegisterOperationType::Consume => {
            let mut tracker = RelationshipTracker::new();
            consume_resource_effect(resource, domain_id, invoker, &mut tracker)?
        },
        RegisterOperationType::Archive => {
            archive_resource_effect(resource, domain_id, invoker)?
        },
        _ => {
            return Err(Error::InvalidOperation(
                format!("Unsupported operation type {:?} for commitment effect", operation_type)
            ));
        }
    };
    
    // Chain the commitment storage with the operation effect
    let combined_effect = create_composite_effect(
        vec![commitment_effect, operation_effect],
        format!("Commitment-backed {:?} operation", operation_type),
    );
    
    Ok(combined_effect)
}

/// Effect implementation for capability validation
struct CapabilityValidationEffect {
    id: EffectId,
    resource_id: ResourceId,
    operation_type: RegisterOperationType,
    capability_ids: Vec<String>,
    domain_id: DomainId,
    invoker: Address,
}

impl CapabilityValidationEffect {
    pub fn new(
        resource_id: ResourceId,
        operation_type: RegisterOperationType,
        capability_ids: Vec<String>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource_id,
            operation_type,
            capability_ids,
            domain_id,
            invoker,
        }
    }
}

#[async_trait]
impl Effect for CapabilityValidationEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn display_name(&self) -> String {
        format!("Capability Validation for {:?}", self.operation_type)
    }
    
    fn description(&self) -> String {
        format!(
            "Validates that capabilities permit {:?} operation on resource {}",
            self.operation_type,
            self.resource_id
        )
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Create authorization service
        let auth_service = context.get_authorization_service()?;
        
        // Convert capability IDs to the right format
        let capability_ids = self.capability_ids.iter()
            .map(|id| CapabilityId::from(id.clone()))
            .collect::<Vec<_>>();
        
        // Check if operation is allowed
        let is_allowed = auth_service.check_operation_allowed(
            &self.resource_id,
            self.operation_type,
            &capability_ids,
        )?;
        
        if !is_allowed {
            return Err(EffectError::ValidationFailed(format!(
                "Operation {:?} not allowed on resource {} with the provided capabilities",
                self.operation_type,
                self.resource_id
            )));
        }
        
        // If allowed, return success
        Ok(EffectOutcome {
            id: self.id.clone(),
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: context.execution_id,
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}

/// Effect implementation for time map validation
struct TimeMapValidationEffect {
    id: EffectId,
    resource_id: ResourceId,
    time_map_snapshot: TimeMapSnapshot,
    domain_id: DomainId,
    invoker: Address,
}

impl TimeMapValidationEffect {
    pub fn new(
        resource_id: ResourceId,
        time_map_snapshot: TimeMapSnapshot,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource_id,
            time_map_snapshot,
            domain_id,
            invoker,
        }
    }
}

#[async_trait]
impl Effect for TimeMapValidationEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn display_name(&self) -> String {
        format!("Time Map Validation")
    }
    
    fn description(&self) -> String {
        format!(
            "Validates that resource {} operation is temporally consistent",
            self.resource_id
        )
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get time map service from context
        let time_service = context.get_time_service()?;
        
        // Validate the time map snapshot
        let is_valid = time_service.validate_snapshot(&self.time_map_snapshot)?;
        
        if !is_valid {
            return Err(EffectError::ValidationFailed(format!(
                "Time map snapshot validation failed for resource {}",
                self.resource_id
            )));
        }
        
        // If valid, return success
        Ok(EffectOutcome {
            id: self.id.clone(),
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: context.execution_id,
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}

/// Effect implementation for domain transitions
struct DomainTransitionEffect {
    id: EffectId,
    inner_effect: Arc<dyn Effect>,
    source_domain: DomainId,
    target_domain: DomainId,
    description: String,
}

impl DomainTransitionEffect {
    pub fn new(
        inner_effect: Arc<dyn Effect>,
        source_domain: DomainId,
        target_domain: DomainId,
        description: String,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            inner_effect,
            source_domain,
            target_domain,
            description,
        }
    }
}

#[async_trait]
impl Effect for DomainTransitionEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn display_name(&self) -> String {
        format!("Domain Transition Effect")
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get domain service from context
        let domain_service = context.get_domain_service()?;
        
        // Check if domains are compatible for this operation
        let can_transition = domain_service.can_transition_between(
            &self.source_domain,
            &self.target_domain,
        )?;
        
        if !can_transition {
            return Err(EffectError::ValidationFailed(format!(
                "Cannot transition between domains {} and {}",
                self.source_domain,
                self.target_domain
            )));
        }
        
        // If domains are compatible, execute the inner effect
        let outcome = self.inner_effect.execute_async(context).await?;
        
        // Add domain transition metadata
        let mut updated_metadata = outcome.metadata.clone();
        updated_metadata.insert(
            "source_domain".to_string(), 
            serde_json::to_value(&self.source_domain).unwrap()
        );
        updated_metadata.insert(
            "target_domain".to_string(), 
            serde_json::to_value(&self.target_domain).unwrap()
        );
        
        // Return success with updated metadata
        Ok(EffectOutcome {
            id: self.id.clone(),
            success: outcome.success,
            data: outcome.data,
            error: outcome.error,
            execution_id: context.execution_id,
            resource_changes: outcome.resource_changes,
            metadata: updated_metadata,
        })
    }
}

/// Effect implementation for boundary awareness
struct BoundaryAwareEffect {
    id: EffectId,
    inner_effect: Arc<dyn Effect>,
    boundary: ExecutionBoundary,
    description: String,
}

impl BoundaryAwareEffect {
    pub fn new(
        inner_effect: Arc<dyn Effect>,
        boundary: ExecutionBoundary,
        description: String,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            inner_effect,
            boundary,
            description,
        }
    }
}

#[async_trait]
impl Effect for BoundaryAwareEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn display_name(&self) -> String {
        format!("Boundary Aware Effect")
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get boundary manager from context
        let boundary_manager = context.get_boundary_manager()?;
        
        // Check if operation is allowed across the boundary
        let can_cross = boundary_manager.can_cross_boundary(
            &self.boundary,
            context.invoker.as_ref(),
        )?;
        
        if !can_cross {
            return Err(EffectError::ValidationFailed(format!(
                "Cannot cross boundary {:?}",
                self.boundary
            )));
        }
        
        // If boundary crossing is allowed, execute the inner effect
        let outcome = self.inner_effect.execute_async(context).await?;
        
        // Add boundary metadata
        let mut updated_metadata = outcome.metadata.clone();
        updated_metadata.insert(
            "boundary".to_string(), 
            serde_json::to_value(&self.boundary).unwrap()
        );
        
        // Return success with updated metadata
        Ok(EffectOutcome {
            id: self.id.clone(),
            success: outcome.success,
            data: outcome.data,
            error: outcome.error,
            execution_id: context.execution_id,
            resource_changes: outcome.resource_changes,
            metadata: updated_metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;
    
    // Helper function to create a test resource
    fn create_test_resource(id: &str) -> ResourceRegister {
        let mut metadata = Metadata::default();
        metadata.insert("test".to_string(), "value".to_string());
        
        ResourceRegister {
            id: id.to_string(),
            owner: Address::random(),
            state: RegisterState::Initial,
            metadata,
            storage_strategy: crate::resource::StorageStrategy::OnChain,
            // ... other fields would be initialized here
        }
    }
    
    #[test]
    fn test_lock_unlock_resource_effect() {
        let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
        let mut relationship_tracker = RelationshipTracker::new();
        
        let resource_id = "resource1".to_string();
        let locker_id = "locker1".to_string();
        
        // Register and activate resources
        lifecycle_manager.register_resource(resource_id.clone()).unwrap();
        lifecycle_manager.activate(&resource_id).unwrap();
        
        lifecycle_manager.register_resource(locker_id.clone()).unwrap();
        lifecycle_manager.activate(&locker_id).unwrap();
        
        // Test lock effect
        lock_resource_effect(
            &mut create_test_resource("resource1"),
            DomainId::new(0),
            Address::random(),
        ).unwrap();
        
        // Verify resource is locked
        assert_eq!(
            lifecycle_manager.get_state(&resource_id).unwrap(),
            RegisterState::Locked
        );
        
        // Verify relationship is created
        let relationships = relationship_tracker.get_relationships_between(
            &locker_id,
            &resource_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::Lock);
        
        // Test unlock effect
        unlock_resource_effect(
            &mut create_test_resource("resource1"),
            DomainId::new(0),
            Address::random(),
        ).unwrap();
        
        // Verify resource is unlocked
        assert_eq!(
            lifecycle_manager.get_state(&resource_id).unwrap(),
            RegisterState::Active
        );
        
        // Verify relationship is removed
        let relationships = relationship_tracker.get_relationships_between(
            &locker_id,
            &resource_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 0);
    }
    
    #[test]
    fn test_parent_child_relationship() {
        let mut relationship_tracker = RelationshipTracker::new();
        
        let parent_id = "parent1".to_string();
        let child_id = "child1".to_string();
        
        // Test creating parent-child relationship
        create_parent_child_effect(
            parent_id.clone(),
            child_id.clone(),
            &mut relationship_tracker,
            None,
        ).unwrap();
        
        // Verify relationship is created
        let relationships = relationship_tracker.get_relationships_between(
            &parent_id,
            &child_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::ParentChild);
        
        // Test getting children
        let children = relationship_tracker.get_child_resources(&parent_id).unwrap();
        assert_eq!(children.len(), 1);
        assert!(children.contains(&child_id));
        
        // Test getting parent
        let parents = relationship_tracker.get_parent_resources(&child_id).unwrap();
        assert_eq!(parents.len(), 1);
        assert!(parents.contains(&parent_id));
    }
    
    #[test]
    fn test_dependency_relationship() {
        let mut relationship_tracker = RelationshipTracker::new();
        
        let dependent_id = "dependent1".to_string();
        let dependency_id = "dependency1".to_string();
        
        // Test creating dependency relationship
        create_dependency_effect(
            dependent_id.clone(),
            dependency_id.clone(),
            &mut relationship_tracker,
            None,
        ).unwrap();
        
        // Verify relationship is created
        let relationships = relationship_tracker.get_relationships_between(
            &dependent_id,
            &dependency_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::Dependency);
        
        // Test getting dependencies
        let dependencies = relationship_tracker.get_dependencies(&dependent_id).unwrap();
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&dependency_id));
        
        // Test getting dependents
        let dependents = relationship_tracker.get_dependents(&dependency_id).unwrap();
        assert_eq!(dependents.len(), 1);
        assert!(dependents.contains(&dependent_id));
    }
} 