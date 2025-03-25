// Capability API for resource access control
// Original file: src/resource/capability_api.rs

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use causality_types::Address;
use crate::resource::{
    ResourceId, ResourceState, ContentId, RegisterState, RegisterContents, RegisterMetadata,
    capability::{
        CapabilityId, Right, Restrictions, CapabilityError, ResourceCapability
    },
    lifecycle_manager::ResourceRegisterLifecycleManager,
    relationship_tracker::RelationshipTracker,
    capability_system::{CapabilityValidator, UnifiedCapabilitySystem}
};

/// Error types for capability-based resource operations
#[derive(Debug, thiserror::Error)]
pub enum ResourceApiError {
    #[error("Capability error: {0}")]
    Capability(#[from] CapabilityError),
    
    #[error("Lifecycle manager error: {0}")]
    LifecycleManager(String),
    
    #[error("Resource not found: {0}")]
    NotFound(ContentId),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for resource API operations
pub type ResourceApiResult<T> = Result<T, ResourceApiError>;

/// A trait defining high-level intent-based operations that can be performed on resources
pub trait ResourceIntent {
    /// The result type of the intent operation
    type Output;
    
    /// Converts the high-level intent into concrete operations
    fn to_operations(&self) -> Vec<ResourceOperation>;
    
    /// Validates whether the intent can be fulfilled
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()>;
    
    /// Executes the intent
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output>;
}

/// Concrete resource operations
#[derive(Debug, Clone)]
pub enum ResourceOperation {
    Read(ContentId),
    Write(ContentId, RegisterContents),
    Update(ContentId, RegisterContents),
    Delete(ContentId),
    Create(ResourceState),
    UpdateMetadata(ContentId, RegisterMetadata),
}

impl ResourceOperation {
    /// Returns the required right for this operation
    pub fn required_right(&self) -> Right {
        match self {
            Self::Read(_) => Right::Read,
            Self::Write(_, _) => Right::Write,
            Self::Update(_, _) => Right::Write,
            Self::Delete(_) => Right::Delete,
            Self::Create(_) => Right::Create,
            Self::UpdateMetadata(_, _) => Right::UpdateMetadata,
        }
    }
    
    /// Returns the register ID involved in this operation, if any
    pub fn register_id(&self) -> Option<&ContentId> {
        match self {
            Self::Read(id) => Some(id),
            Self::Write(id, _) => Some(id),
            Self::Update(id, _) => Some(id),
            Self::Delete(id) => Some(id),
            Self::Create(state) => Some(&state.id),
            Self::UpdateMetadata(id, _) => Some(id),
        }
    }
}

/// A capability-based API for resource operations using the unified architecture
#[derive(Debug, Clone)]
pub struct ResourceAPI {
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
    relationship_tracker: Arc<RelationshipTracker>,
    capability_system: Arc<UnifiedCapabilitySystem>,
}

impl ResourceAPI {
    /// Creates a new resource API using the unified architecture components
    pub fn new(
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
        capability_system: Arc<UnifiedCapabilitySystem>,
    ) -> Self {
        Self {
            lifecycle_manager,
            relationship_tracker,
            capability_system,
        }
    }
    
    /// Returns the capability system
    pub fn capability_system(&self) -> &Arc<UnifiedCapabilitySystem> {
        &self.capability_system
    }
    
    /// Verifies a capability for a specific operation
    pub fn verify_capability(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        operation: &ResourceOperation,
    ) -> ResourceApiResult<()> {
        let right = operation.required_right();
        let register_id = operation.register_id();
        
        self.capability_system
            .validate_capability(capability_id, holder, &right, register_id)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Reads a register using a capability
    pub fn read(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &ContentId,
    ) -> ResourceApiResult<RegisterState> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Read(register_id.clone()),
        )?;
        
        // Perform the read operation
        self.lifecycle_manager
            .get_resource_state(register_id)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
            .and_then(|opt| opt.ok_or_else(|| ResourceApiError::NotFound(register_id.clone())))
    }
    
    /// Reads multiple registers using a capability
    pub fn read_batch(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_ids: &[ContentId],
    ) -> ResourceApiResult<Vec<RegisterState>> {
        let mut results = Vec::with_capacity(register_ids.len());
        
        for id in register_ids {
            results.push(self.read(capability_id, holder, id)?);
        }
        
        Ok(results)
    }
    
    /// Creates a new register using a capability
    pub fn create(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        state: ResourceState,
    ) -> ResourceApiResult<ContentId> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Create(state.clone()),
        )?;
        
        // Perform the create operation
        self.lifecycle_manager
            .create_resource(state)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
    }
    
    /// Updates a register using a capability
    pub fn update(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &ContentId,
        contents: RegisterContents,
    ) -> ResourceApiResult<()> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Update(register_id.clone(), contents.clone()),
        )?;
        
        // Get the current state
        let current_state = self.lifecycle_manager
            .get_resource_state(register_id)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?
            .ok_or_else(|| ResourceApiError::NotFound(register_id.clone()))?;
        
        // Create updated state
        let mut updated_state = current_state.clone();
        updated_state.contents = contents;
        
        // Perform the update operation
        self.lifecycle_manager
            .update_resource_state(register_id, updated_state)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
    }
    
    /// Deletes a register using a capability
    pub fn delete(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &ContentId,
    ) -> ResourceApiResult<()> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Delete(register_id.clone()),
        )?;
        
        // Perform the delete operation
        self.lifecycle_manager
            .delete_resource(register_id)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
    }
    
    /// Updates register metadata using a capability
    pub fn update_metadata(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &ContentId,
        metadata: RegisterMetadata,
    ) -> ResourceApiResult<()> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::UpdateMetadata(register_id.clone(), metadata.clone()),
        )?;
        
        // Get the current state
        let current_state = self.lifecycle_manager
            .get_resource_state(register_id)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?
            .ok_or_else(|| ResourceApiError::NotFound(register_id.clone()))?;
        
        // Create updated state with new metadata
        let mut updated_state = current_state.clone();
        updated_state.metadata = metadata;
        
        // Perform the metadata update operation
        self.lifecycle_manager
            .update_resource_state(register_id, updated_state)
            .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))
    }
    
    /// Delegates a capability using the capability system
    pub fn delegate_capability(
        &self,
        parent_id: &CapabilityId,
        delegator: &Address,
        new_holder: Address,
        rights: HashSet<Right>,
        restrictions: Restrictions,
    ) -> ResourceApiResult<CapabilityId> {
        self.capability_system
            .delegate_capability(parent_id, delegator, new_holder, rights, restrictions)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Creates a new capability by composing existing capabilities
    pub fn compose_capabilities(
        &self,
        composer: &Address,
        capability_ids: &[CapabilityId],
        new_holder: Address,
        restrictions: Restrictions,
    ) -> ResourceApiResult<CapabilityId> {
        self.capability_system
            .compose_capabilities(capability_ids, composer, new_holder, restrictions)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Revokes a capability
    pub fn revoke_capability(
        &self,
        capability_id: &CapabilityId,
        revoker: &Address,
    ) -> ResourceApiResult<()> {
        self.capability_system
            .revoke_capability(capability_id, revoker)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Executes a batch of operations with a single capability check
    pub fn execute_batch(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        operations: &[ResourceOperation],
    ) -> ResourceApiResult<()> {
        // For each operation, verify the capability
        for op in operations {
            self.verify_capability(capability_id, holder, op)?;
        }
        
        // If all verifications pass, execute the operations
        for op in operations {
            match op {
                ResourceOperation::Read(_) => {
                    // Read operations don't modify state, so just verify
                    continue;
                },
                ResourceOperation::Write(id, contents) => {
                    let current_state = self.lifecycle_manager
                        .get_resource_state(id)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?
                        .ok_or_else(|| ResourceApiError::NotFound(id.clone()))?;
                    
                    let mut updated_state = current_state.clone();
                    updated_state.contents = contents.clone();
                    
                    self.lifecycle_manager
                        .update_resource_state(id, updated_state)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
                },
                ResourceOperation::Update(id, contents) => {
                    let current_state = self.lifecycle_manager
                        .get_resource_state(id)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?
                        .ok_or_else(|| ResourceApiError::NotFound(id.clone()))?;
                    
                    let mut updated_state = current_state.clone();
                    updated_state.contents = contents.clone();
                    
                    self.lifecycle_manager
                        .update_resource_state(id, updated_state)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
                },
                ResourceOperation::Delete(id) => {
                    self.lifecycle_manager
                        .delete_resource(id)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
                },
                ResourceOperation::Create(state) => {
                    self.lifecycle_manager
                        .create_resource(state.clone())
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
                },
                ResourceOperation::UpdateMetadata(id, metadata) => {
                    let current_state = self.lifecycle_manager
                        .get_resource_state(id)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?
                        .ok_or_else(|| ResourceApiError::NotFound(id.clone()))?;
                    
                    let mut updated_state = current_state.clone();
                    updated_state.metadata = metadata.clone();
                    
                    self.lifecycle_manager
                        .update_resource_state(id, updated_state)
                        .map_err(|e| ResourceApiError::LifecycleManager(e.to_string()))?;
                },
            }
        }
        
        Ok(())
    }
    
    /// Executes a high-level intent
    pub fn execute_intent<I: ResourceIntent>(
        &self,
        intent: &I,
    ) -> ResourceApiResult<I::Output> {
        // First validate the intent
        intent.validate(self)?;
        
        // Then execute it
        intent.execute(self)
    }
    
    /// Finds related resources using the relationship tracker
    pub fn find_related_resources(
        &self,
        resource_id: &ResourceId,
        relationship_type: Option<&str>,
    ) -> ResourceApiResult<Vec<ResourceId>> {
        self.relationship_tracker
            .find_related_resources(resource_id, relationship_type)
            .map_err(|e| ResourceApiError::Internal(e.to_string()))
    }
    
    /// Checks if two resources have a specific relationship
    pub fn has_relationship(
        &self,
        source: &ResourceId,
        target: &ResourceId,
        relationship_type: &str,
    ) -> ResourceApiResult<bool> {
        self.relationship_tracker
            .has_relationship(source, target, relationship_type)
            .map_err(|e| ResourceApiError::Internal(e.to_string()))
    }
    
    /// Creates a relationship between two resources
    pub fn create_relationship(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        source: &ResourceId,
        target: &ResourceId,
        relationship_type: &str,
    ) -> ResourceApiResult<()> {
        // Verify capability for both resources
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Read(source.clone()),
        )?;
        
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Read(target.clone()),
        )?;
        
        // Create the relationship
        self.relationship_tracker
            .create_relationship(source, target, relationship_type)
            .map_err(|e| ResourceApiError::Internal(e.to_string()))
    }
}

/// A transfer intent for moving resources between addresses
pub struct TransferIntent {
    /// The capability ID authorizing the transfer
    pub capability_id: CapabilityId,
    /// The current holder of the capability
    pub current_holder: Address,
    /// The register to transfer
    pub register_id: ContentId,
    /// The recipient of the transfer
    pub recipient: Address,
}

impl ResourceIntent for TransferIntent {
    type Output = CapabilityId;
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        // Transfer doesn't directly map to a single operation
        // It's a higher-level concept that involves capability operations
        vec![ResourceOperation::Read(self.register_id.clone())]
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Check if the register exists
        let state = api.read(&self.capability_id, &self.current_holder, &self.register_id)?;
        
        // Additional validations could be added here
        // For example, check if the register has already been transferred
        
        // Validate that the capability allows transfer (read + management rights)
        api.verify_capability(
            &self.capability_id,
            &self.current_holder,
            &ResourceOperation::Read(self.register_id.clone()),
        )?;
        
        Ok(())
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        // Read the current register state
        let state = api.read(&self.capability_id, &self.current_holder, &self.register_id)?;
        
        // Create a new capability for the recipient
        let rights = {
            let mut set = HashSet::new();
            set.insert(Right::Read);
            set.insert(Right::Write);
            set.insert(Right::Delete);
            set.insert(Right::UpdateMetadata);
            set
        };
        
        // Create transfer restrictions
        let restrictions = Restrictions::default();
        
        // Delegate a capability to the recipient
        let new_capability_id = api.delegate_capability(
            &self.capability_id,
            &self.current_holder,
            self.recipient.clone(),
            rights,
            restrictions,
        )?;
        
        // Create a relationship documenting the transfer
        api.create_relationship(
            &self.capability_id,
            &self.current_holder,
            &self.register_id,
            &ResourceId::from(new_capability_id.clone()),
            "transferred_to",
        )?;
        
        Ok(new_capability_id)
    }
}

/// A swap intent for exchanging resources between two parties
pub struct SwapIntent {
    /// First party's capability
    pub capability_a: CapabilityId,
    /// First party's address
    pub holder_a: Address,
    /// First party's register to swap
    pub register_a: ContentId,
    /// Second party's capability
    pub capability_b: CapabilityId,
    /// Second party's address
    pub holder_b: Address,
    /// Second party's register to swap
    pub register_b: ContentId,
}

impl ResourceIntent for SwapIntent {
    type Output = (CapabilityId, CapabilityId);
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        // Swap doesn't directly map to resource operations
        // It's a higher-level concept involving transfers
        vec![
            ResourceOperation::Read(self.register_a.clone()),
            ResourceOperation::Read(self.register_b.clone()),
        ]
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Check if both registers exist
        let state_a = api.read(&self.capability_a, &self.holder_a, &self.register_a)?;
        let state_b = api.read(&self.capability_b, &self.holder_b, &self.register_b)?;
        
        // Verify capabilities for both registers
        api.verify_capability(
            &self.capability_a,
            &self.holder_a,
            &ResourceOperation::Read(self.register_a.clone()),
        )?;
        
        api.verify_capability(
            &self.capability_b,
            &self.holder_b,
            &ResourceOperation::Read(self.register_b.clone()),
        )?;
        
        // Additional validations could go here
        // For example, checking compatible register types or values
        
        Ok(())
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        // Implement swap as two transfers
        
        // First transfer: A to B
        let transfer_a_to_b = TransferIntent {
            capability_id: self.capability_a.clone(),
            current_holder: self.holder_a.clone(),
            register_id: self.register_a.clone(),
            recipient: self.holder_b.clone(),
        };
        
        // Second transfer: B to A
        let transfer_b_to_a = TransferIntent {
            capability_id: self.capability_b.clone(),
            current_holder: self.holder_b.clone(),
            register_id: self.register_b.clone(),
            recipient: self.holder_a.clone(),
        };
        
        // Execute both transfers
        let capability_b_to_a = transfer_a_to_b.execute(api)?;
        let capability_a_to_b = transfer_b_to_a.execute(api)?;
        
        // Create relationship to document the swap
        api.create_relationship(
            &self.capability_a,
            &self.holder_a,
            &self.register_a,
            &self.register_b,
            "swapped_with",
        )?;
        
        Ok((capability_b_to_a, capability_a_to_b))
    }
} 
