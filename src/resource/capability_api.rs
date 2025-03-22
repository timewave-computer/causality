use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::address::Address;
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterMetadata,
    RegisterService, ResourceManager, capability::{
        CapabilityId, Right, Restrictions, CapabilityError, CapabilityRegistry, 
        ResourceCapability
    }
};

/// Error types for capability-based resource operations
#[derive(Debug, thiserror::Error)]
pub enum ResourceApiError {
    #[error("Capability error: {0}")]
    Capability(#[from] CapabilityError),
    
    #[error("Register service error: {0}")]
    RegisterService(String),
    
    #[error("Resource not found: {0}")]
    NotFound(RegisterId),
    
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
    Read(RegisterId),
    Write(RegisterId, RegisterContents),
    Update(RegisterId, RegisterContents),
    Delete(RegisterId),
    Create(Register),
    UpdateMetadata(RegisterId, RegisterMetadata),
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
    pub fn register_id(&self) -> Option<&RegisterId> {
        match self {
            Self::Read(id) => Some(id),
            Self::Write(id, _) => Some(id),
            Self::Update(id, _) => Some(id),
            Self::Delete(id) => Some(id),
            Self::Create(register) => Some(register.id()),
            Self::UpdateMetadata(id, _) => Some(id),
        }
    }
}

/// A capability-based API for resource operations
#[derive(Debug, Clone)]
pub struct ResourceAPI {
    register_service: Arc<dyn RegisterService>,
    capability_registry: Arc<CapabilityRegistry>,
}

impl ResourceAPI {
    /// Creates a new resource API
    pub fn new(
        register_service: Arc<dyn RegisterService>,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Self {
        Self {
            register_service,
            capability_registry,
        }
    }
    
    /// Returns the capability registry
    pub fn capability_registry(&self) -> &Arc<CapabilityRegistry> {
        &self.capability_registry
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
        
        self.capability_registry
            .verify(capability_id, holder, &right, register_id, None)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Reads a register using a capability
    pub fn read(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &RegisterId,
    ) -> ResourceApiResult<Register> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Read(register_id.clone()),
        )?;
        
        // Perform the read operation
        self.register_service
            .get_register(register_id)
            .map_err(|e| ResourceApiError::RegisterService(e.to_string()))
            .and_then(|opt| opt.ok_or_else(|| ResourceApiError::NotFound(register_id.clone())))
    }
    
    /// Reads multiple registers using a capability
    pub fn read_batch(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_ids: &[RegisterId],
    ) -> ResourceApiResult<Vec<Register>> {
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
        register: Register,
    ) -> ResourceApiResult<RegisterId> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Create(register.clone()),
        )?;
        
        // Perform the create operation
        self.register_service
            .create_register(register)
            .map_err(|e| ResourceApiError::RegisterService(e.to_string()))
    }
    
    /// Updates a register using a capability
    pub fn update(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &RegisterId,
        contents: RegisterContents,
    ) -> ResourceApiResult<()> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Update(register_id.clone(), contents.clone()),
        )?;
        
        // Perform the update operation
        self.register_service
            .update_register(register_id, contents)
            .map_err(|e| ResourceApiError::RegisterService(e.to_string()))?
            .ok_or_else(|| ResourceApiError::NotFound(register_id.clone()))
    }
    
    /// Deletes a register using a capability
    pub fn delete(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &RegisterId,
    ) -> ResourceApiResult<()> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::Delete(register_id.clone()),
        )?;
        
        // Perform the delete operation
        self.register_service
            .delete_register(register_id)
            .map_err(|e| ResourceApiError::RegisterService(e.to_string()))?
            .ok_or_else(|| ResourceApiError::NotFound(register_id.clone()))
    }
    
    /// Updates register metadata using a capability
    pub fn update_metadata(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        register_id: &RegisterId,
        metadata: RegisterMetadata,
    ) -> ResourceApiResult<()> {
        // Verify the capability for this operation
        self.verify_capability(
            capability_id,
            holder,
            &ResourceOperation::UpdateMetadata(register_id.clone(), metadata.clone()),
        )?;
        
        // Perform the metadata update operation
        self.register_service
            .update_metadata(register_id, metadata)
            .map_err(|e| ResourceApiError::RegisterService(e.to_string()))?
            .ok_or_else(|| ResourceApiError::NotFound(register_id.clone()))
    }
    
    /// Delegates a capability to another principal
    pub fn delegate_capability(
        &self,
        parent_id: &CapabilityId,
        delegator: &Address,
        new_holder: Address,
        rights: HashSet<Right>,
        restrictions: Restrictions,
    ) -> ResourceApiResult<CapabilityId> {
        self.capability_registry
            .delegate(parent_id, delegator, new_holder, rights, restrictions)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Composes multiple capabilities into a new one
    pub fn compose_capabilities(
        &self,
        composer: &Address,
        capability_ids: &[CapabilityId],
        new_holder: Address,
        restrictions: Restrictions,
    ) -> ResourceApiResult<CapabilityId> {
        self.capability_registry
            .compose(composer, capability_ids, new_holder, restrictions)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Revokes a capability
    pub fn revoke_capability(
        &self,
        capability_id: &CapabilityId,
        revoker: &Address,
    ) -> ResourceApiResult<()> {
        self.capability_registry
            .revoke(capability_id, revoker)
            .map_err(ResourceApiError::Capability)
    }
    
    /// Executes a batch of operations using a capability
    pub fn execute_batch(
        &self,
        capability_id: &CapabilityId,
        holder: &Address,
        operations: &[ResourceOperation],
    ) -> ResourceApiResult<()> {
        // First verify all operations can be performed with this capability
        for op in operations {
            self.verify_capability(capability_id, holder, op)?;
        }
        
        // Then execute each operation
        for op in operations {
            match op {
                ResourceOperation::Read(id) => {
                    self.read(capability_id, holder, id)?;
                }
                ResourceOperation::Write(id, contents) => {
                    self.update(capability_id, holder, id, contents.clone())?;
                }
                ResourceOperation::Update(id, contents) => {
                    self.update(capability_id, holder, id, contents.clone())?;
                }
                ResourceOperation::Delete(id) => {
                    self.delete(capability_id, holder, id)?;
                }
                ResourceOperation::Create(register) => {
                    self.create(capability_id, holder, register.clone())?;
                }
                ResourceOperation::UpdateMetadata(id, metadata) => {
                    self.update_metadata(capability_id, holder, id, metadata.clone())?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Executes a high-level intent
    pub fn execute_intent<I: ResourceIntent>(
        &self,
        intent: &I,
    ) -> ResourceApiResult<I::Output> {
        // Validate the intent first
        intent.validate(self)?;
        
        // Execute the intent
        intent.execute(self)
    }
}

/// A transfer intent to move ownership of a resource
#[derive(Debug, Clone)]
pub struct TransferIntent {
    /// The capability ID authorizing the transfer
    pub capability_id: CapabilityId,
    /// The current holder of the capability
    pub current_holder: Address,
    /// The register to transfer
    pub register_id: RegisterId,
    /// The recipient of the transfer
    pub recipient: Address,
}

impl ResourceIntent for TransferIntent {
    type Output = CapabilityId;
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        vec![
            ResourceOperation::Read(self.register_id.clone()),
            ResourceOperation::UpdateMetadata(self.register_id.clone(), RegisterMetadata::default()),
        ]
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Check if the register exists
        let _register = api.read(&self.capability_id, &self.current_holder, &self.register_id)?;
        
        // Check that the holder has delegate rights
        api.capability_registry()
            .verify(
                &self.capability_id,
                &self.current_holder,
                &Right::Delegate,
                Some(&self.register_id),
                None,
            )
            .map_err(ResourceApiError::Capability)
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        // Read the register
        let register = api.read(&self.capability_id, &self.current_holder, &self.register_id)?;
        
        // Create a new capability for the recipient with all rights
        let mut rights = HashSet::new();
        rights.insert(Right::Read);
        rights.insert(Right::Write);
        rights.insert(Right::Delete);
        rights.insert(Right::UpdateMetadata);
        rights.insert(Right::Delegate);
        
        let restrictions = Restrictions {
            resource_scope: {
                let mut scope = HashSet::new();
                scope.insert(self.register_id.clone());
                Some(scope)
            },
            ..Restrictions::default()
        };
        
        // Delegate to create a new capability for the recipient
        api.delegate_capability(
            &self.capability_id,
            &self.current_holder,
            self.recipient.clone(),
            rights,
            restrictions,
        )
    }
}

/// A swap intent to exchange two resources between parties
#[derive(Debug, Clone)]
pub struct SwapIntent {
    /// First party's capability
    pub capability_a: CapabilityId,
    /// First party's address
    pub holder_a: Address,
    /// First party's register to swap
    pub register_a: RegisterId,
    /// Second party's capability
    pub capability_b: CapabilityId,
    /// Second party's address
    pub holder_b: Address,
    /// Second party's register to swap
    pub register_b: RegisterId,
}

impl ResourceIntent for SwapIntent {
    type Output = (CapabilityId, CapabilityId);
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        vec![
            ResourceOperation::Read(self.register_a.clone()),
            ResourceOperation::Read(self.register_b.clone()),
            ResourceOperation::UpdateMetadata(self.register_a.clone(), RegisterMetadata::default()),
            ResourceOperation::UpdateMetadata(self.register_b.clone(), RegisterMetadata::default()),
        ]
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Check if both registers exist
        let _register_a = api.read(&self.capability_a, &self.holder_a, &self.register_a)?;
        let _register_b = api.read(&self.capability_b, &self.holder_b, &self.register_b)?;
        
        // Check that both holders have delegate rights
        api.capability_registry()
            .verify(
                &self.capability_a,
                &self.holder_a,
                &Right::Delegate,
                Some(&self.register_a),
                None,
            )
            .map_err(ResourceApiError::Capability)?;
            
        api.capability_registry()
            .verify(
                &self.capability_b,
                &self.holder_b,
                &Right::Delegate,
                Some(&self.register_b),
                None,
            )
            .map_err(ResourceApiError::Capability)
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        // Read both registers
        let _register_a = api.read(&self.capability_a, &self.holder_a, &self.register_a)?;
        let _register_b = api.read(&self.capability_b, &self.holder_b, &self.register_b)?;
        
        // Create capabilities for both parties with all rights
        let mut rights = HashSet::new();
        rights.insert(Right::Read);
        rights.insert(Right::Write);
        rights.insert(Right::Delete);
        rights.insert(Right::UpdateMetadata);
        rights.insert(Right::Delegate);
        
        // Create restrictions for register A
        let restrictions_a = Restrictions {
            resource_scope: {
                let mut scope = HashSet::new();
                scope.insert(self.register_a.clone());
                Some(scope)
            },
            ..Restrictions::default()
        };
        
        // Create restrictions for register B
        let restrictions_b = Restrictions {
            resource_scope: {
                let mut scope = HashSet::new();
                scope.insert(self.register_b.clone());
                Some(scope)
            },
            ..Restrictions::default()
        };
        
        // Delegate capability for holder B to access register A
        let cap_a_for_b = api.delegate_capability(
            &self.capability_a,
            &self.holder_a,
            self.holder_b.clone(),
            rights.clone(),
            restrictions_a,
        )?;
        
        // Delegate capability for holder A to access register B
        let cap_b_for_a = api.delegate_capability(
            &self.capability_b,
            &self.holder_b,
            self.holder_a.clone(),
            rights,
            restrictions_b,
        )?;
        
        Ok((cap_b_for_a, cap_a_for_b))
    }
} 