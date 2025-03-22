use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::SystemTime;

use crate::address::Address;
use crate::resource::{
    capability::{CapabilityId, ResourceCapability, CapabilityRegistry, 
                Right, Restrictions, CapabilityError},
    capability_api::{ResourceOperation, ResourceIntent, ResourceAPI, ResourceApiResult, ResourceApiError},
    RegisterId, RegisterContents, RegisterMetadata
};

/// A chain of capabilities from a root to a leaf
#[derive(Debug, Clone)]
pub struct CapabilityChain {
    /// The leaf capability at the end of the chain
    leaf: CapabilityId,
    /// The ordered sequence of capabilities from root to leaf
    chain: Vec<CapabilityId>,
    /// The registry containing the capabilities
    registry: Arc<CapabilityRegistry>,
}

impl CapabilityChain {
    /// Creates a new capability chain from a leaf capability
    pub fn from_leaf(
        leaf_id: CapabilityId,
        registry: Arc<CapabilityRegistry>,
    ) -> Result<Self, CapabilityError> {
        let mut chain = Vec::new();
        let mut current_id = leaf_id.clone();
        
        // Build the chain from leaf to root
        loop {
            let capability = registry.get(&current_id)?;
            chain.push(current_id.clone());
            
            if let Some(parent_id) = capability.parent() {
                current_id = parent_id.clone();
            } else {
                break;
            }
        }
        
        // Reverse to get root to leaf order
        chain.reverse();
        
        Ok(Self {
            leaf: leaf_id,
            chain,
            registry,
        })
    }
    
    /// Returns the leaf capability ID
    pub fn leaf_id(&self) -> &CapabilityId {
        &self.leaf
    }
    
    /// Returns the root capability ID
    pub fn root_id(&self) -> Option<&CapabilityId> {
        self.chain.first()
    }
    
    /// Returns all capability IDs in the chain
    pub fn capability_ids(&self) -> &[CapabilityId] {
        &self.chain
    }
    
    /// Returns the length of the chain
    pub fn len(&self) -> usize {
        self.chain.len()
    }
    
    /// Checks if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
    
    /// Verifies the integrity of the capability chain
    pub fn verify(&self) -> Result<(), CapabilityError> {
        if self.chain.is_empty() {
            return Err(CapabilityError::VerificationFailed);
        }
        
        // Get all capabilities in the chain
        let mut capabilities = Vec::with_capacity(self.chain.len());
        for id in &self.chain {
            capabilities.push(self.registry.get(id)?);
        }
        
        // Verify parent-child relationships
        for i in 1..capabilities.len() {
            let parent = &capabilities[i-1];
            let child = &capabilities[i];
            
            // Verify parent ID matches
            if child.parent() != Some(parent.id()) {
                return Err(CapabilityError::VerificationFailed);
            }
            
            // Verify child's issuer is parent's holder
            if child.issuer() != parent.holder() {
                return Err(CapabilityError::VerificationFailed);
            }
            
            // Verify child's rights are a subset of parent's
            if !child.rights().is_subset(parent.rights()) {
                return Err(CapabilityError::RightsEscalation);
            }
            
            // Verify parent allows delegation
            if !parent.restrictions().allow_delegation {
                return Err(CapabilityError::DelegationNotAllowed);
            }
        }
        
        // Verify none of the capabilities are revoked
        for capability in &capabilities {
            if capability.revoked() {
                return Err(CapabilityError::Revoked);
            }
        }
        
        Ok(())
    }
    
    /// Checks if the chain is valid for a specific operation
    pub fn is_valid_for(
        &self,
        holder: &Address,
        right: &Right,
        resource_id: Option<&RegisterId>,
    ) -> Result<(), CapabilityError> {
        // First verify the chain integrity
        self.verify()?;
        
        // Get the leaf capability
        let leaf = self.registry.get(&self.leaf)?;
        
        // Verify the holder
        if leaf.holder() != holder {
            return Err(CapabilityError::UnauthorizedHolder);
        }
        
        // Check if the leaf has the required right
        if !leaf.has_right(right) {
            return Err(CapabilityError::MissingRight(right.clone()));
        }
        
        // Check if any capability in the chain has an expiration
        let now = SystemTime::now();
        for id in &self.chain {
            let capability = self.registry.get(id)?;
            if let Some(expiration) = capability.restrictions().expiration {
                if now > expiration {
                    return Err(CapabilityError::Expired);
                }
            }
        }
        
        // Check if the leaf has resource scope restrictions
        if let (Some(resource_id), Some(ref scope)) = (resource_id, leaf.restrictions().resource_scope) {
            if !scope.contains(resource_id) {
                return Err(CapabilityError::OutOfResourceScope);
            }
        }
        
        Ok(())
    }
}

/// Extension to ResourceCapability to access parent
pub trait CapabilityExt {
    fn parent(&self) -> Option<&CapabilityId>;
    fn revoked(&self) -> bool;
}

impl CapabilityExt for ResourceCapability {
    fn parent(&self) -> Option<&CapabilityId> {
        self.parent.as_ref()
    }
    
    fn revoked(&self) -> bool {
        self.revoked
    }
}

/// A composed intent that combines multiple intents into one atomic operation
#[derive(Debug)]
pub struct ComposedIntent<T> {
    /// The individual intents to compose
    intents: Vec<T>,
    /// Whether to abort the entire operation if any intent fails
    atomic: bool,
}

impl<T: ResourceIntent + Clone> ComposedIntent<T> {
    /// Creates a new composed intent
    pub fn new(intents: Vec<T>, atomic: bool) -> Self {
        Self { intents, atomic }
    }
    
    /// Adds an intent to the composition
    pub fn add_intent(&mut self, intent: T) {
        self.intents.push(intent);
    }
    
    /// Sets whether the composition should be atomic
    pub fn set_atomic(&mut self, atomic: bool) {
        self.atomic = atomic;
    }
    
    /// Returns the intents in this composition
    pub fn intents(&self) -> &[T] {
        &self.intents
    }
    
    /// Returns whether the composition is atomic
    pub fn is_atomic(&self) -> bool {
        self.atomic
    }
}

impl<T: ResourceIntent + Clone> ResourceIntent for ComposedIntent<T> {
    type Output = Vec<Result<T::Output, ResourceApiError>>;
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        let mut operations = Vec::new();
        for intent in &self.intents {
            operations.extend(intent.to_operations());
        }
        operations
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Validate each intent
        for intent in &self.intents {
            if let Err(e) = intent.validate(api) {
                if self.atomic {
                    return Err(e);
                }
                // If not atomic, continue validating other intents
            }
        }
        
        Ok(())
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        let mut results = Vec::with_capacity(self.intents.len());
        
        for intent in &self.intents {
            let result = intent.execute(api);
            
            if let Err(ref e) = result {
                if self.atomic {
                    return Err(e.clone());
                }
            }
            
            results.push(result);
        }
        
        Ok(results)
    }
}

/// A chained intent where each step depends on the previous one
#[derive(Debug)]
pub struct ChainedIntent<I, F, O>
where
    I: ResourceIntent,
    F: Fn(I::Output) -> O,
{
    /// The first intent in the chain
    first: I,
    /// Function to create the next intent from the output of the previous one
    next_generator: F,
}

impl<I, F, O> ChainedIntent<I, F, O>
where
    I: ResourceIntent,
    F: Fn(I::Output) -> O,
{
    /// Creates a new chained intent
    pub fn new(first: I, next_generator: F) -> Self {
        Self { first, next_generator }
    }
}

impl<I, F, O> ResourceIntent for ChainedIntent<I, F, O>
where
    I: ResourceIntent,
    F: Fn(I::Output) -> O,
    O: ResourceIntent,
{
    type Output = (I::Output, O::Output);
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        // Can only determine the first intent's operations initially
        self.first.to_operations()
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Can only validate the first intent
        self.first.validate(api)
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        // Execute the first intent
        let first_output = self.first.execute(api)?;
        
        // Generate and execute the second intent
        let second_intent = (self.next_generator)(first_output.clone());
        let second_output = second_intent.execute(api)?;
        
        Ok((first_output, second_output))
    }
}

/// A conditional intent that executes one of two intents based on a condition
#[derive(Debug)]
pub struct ConditionalIntent<C, T, F>
where
    C: Fn(&ResourceAPI) -> ResourceApiResult<bool>,
    T: ResourceIntent,
    F: ResourceIntent,
{
    /// The condition to evaluate
    condition: C,
    /// The intent to execute if the condition is true
    true_intent: T,
    /// The intent to execute if the condition is false
    false_intent: F,
}

impl<C, T, F> ConditionalIntent<C, T, F>
where
    C: Fn(&ResourceAPI) -> ResourceApiResult<bool>,
    T: ResourceIntent,
    F: ResourceIntent,
{
    /// Creates a new conditional intent
    pub fn new(condition: C, true_intent: T, false_intent: F) -> Self {
        Self {
            condition,
            true_intent,
            false_intent,
        }
    }
}

impl<C, T, F> ResourceIntent for ConditionalIntent<C, T, F>
where
    C: Fn(&ResourceAPI) -> ResourceApiResult<bool>,
    T: ResourceIntent,
    F: ResourceIntent,
{
    type Output = Result<T::Output, F::Output>;
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        // Combine operations from both branches
        let mut operations = self.true_intent.to_operations();
        operations.extend(self.false_intent.to_operations());
        operations
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Can't validate until we know which branch will be taken
        Ok(())
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        // Evaluate the condition
        let condition_result = (self.condition)(api)?;
        
        if condition_result {
            // Execute the true branch
            Ok(Ok(self.true_intent.execute(api)?))
        } else {
            // Execute the false branch
            Ok(Err(self.false_intent.execute(api)?))
        }
    }
}

/// A multi-party transfer intent where resources move through multiple owners
#[derive(Debug, Clone)]
pub struct MultiTransferIntent {
    /// The initial transfer
    first_transfer: crate::resource::capability_api::TransferIntent,
    /// The subsequent transfers
    subsequent_transfers: Vec<(Address, RegisterId, Address)>,
}

impl MultiTransferIntent {
    /// Creates a new multi-party transfer intent
    pub fn new(
        first_transfer: crate::resource::capability_api::TransferIntent,
        subsequent_transfers: Vec<(Address, RegisterId, Address)>,
    ) -> Self {
        Self {
            first_transfer,
            subsequent_transfers,
        }
    }
}

impl ResourceIntent for MultiTransferIntent {
    type Output = Vec<CapabilityId>;
    
    fn to_operations(&self) -> Vec<ResourceOperation> {
        let mut operations = self.first_transfer.to_operations();
        
        // Add operations for subsequent transfers
        for (_, register_id, _) in &self.subsequent_transfers {
            operations.push(ResourceOperation::Read(register_id.clone()));
            operations.push(ResourceOperation::UpdateMetadata(register_id.clone(), RegisterMetadata::default()));
        }
        
        operations
    }
    
    fn validate(&self, api: &ResourceAPI) -> ResourceApiResult<()> {
        // Validate the first transfer
        self.first_transfer.validate(api)?;
        
        // For subsequent transfers, we can't validate yet because the capabilities
        // for them will be created during execution
        
        Ok(())
    }
    
    fn execute(&self, api: &ResourceAPI) -> ResourceApiResult<Self::Output> {
        let mut capability_ids = Vec::with_capacity(self.subsequent_transfers.len() + 1);
        
        // Execute the first transfer
        let first_capability = self.first_transfer.execute(api)?;
        capability_ids.push(first_capability.clone());
        
        let mut current_holder = self.first_transfer.recipient.clone();
        let mut current_capability = first_capability;
        
        // Execute subsequent transfers
        for (expected_holder, register_id, recipient) in &self.subsequent_transfers {
            // Verify current holder matches expected holder
            if &current_holder != expected_holder {
                return Err(ResourceApiError::AccessDenied(
                    format!("Expected holder {:?} but got {:?}", expected_holder, current_holder)
                ));
            }
            
            // Create transfer intent for this step
            let transfer = crate::resource::capability_api::TransferIntent {
                capability_id: current_capability,
                current_holder: current_holder.clone(),
                register_id: register_id.clone(),
                recipient: recipient.clone(),
            };
            
            // Execute the transfer
            current_capability = transfer.execute(api)?;
            capability_ids.push(current_capability.clone());
            
            // Update current holder for next iteration
            current_holder = recipient.clone();
        }
        
        Ok(capability_ids)
    }
} 