// In-memory register service implementation
//
// This module provides a simple in-memory implementation of the RegisterService
// trait for testing and development purposes.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;

use crate::tel::{Address, Domain};
use crate::error::{Error, Result};
use crate::ast::AstContext;
use crate::types::TraceId;
use crate::domain::fact::register_observer::RegisterFactObserver;
use crate::resource::register::{
    Register, RegisterId, RegisterContents, RegisterService,
    AuthorizationMethod, RegisterOperation, OperationType, BlockHeight
};

/// In-memory register service for testing and development
pub struct InMemoryRegisterService {
    /// Map of register IDs to registers
    registers: Arc<RwLock<HashMap<RegisterId, Register>>>,
    /// Map of AstContext to register IDs
    ast_context_map: Arc<RwLock<HashMap<String, Vec<RegisterId>>>>,
    /// Map of owners to register IDs
    owner_map: Arc<RwLock<HashMap<Address, Vec<RegisterId>>>>,
    /// Current epoch
    epoch: Arc<RwLock<u64>>,
    /// Register fact observer for logging facts
    fact_observer: Option<Arc<RegisterFactObserver>>,
}

impl InMemoryRegisterService {
    /// Create a new in-memory register service
    pub fn new() -> Self {
        Self {
            registers: Arc::new(RwLock::new(HashMap::new())),
            ast_context_map: Arc::new(RwLock::new(HashMap::new())),
            owner_map: Arc::new(RwLock::new(HashMap::new())),
            epoch: Arc::new(RwLock::new(1)), // Start at epoch 1
            fact_observer: None,
        }
    }
    
    /// Create a new in-memory register service with a fact observer
    pub fn with_fact_observer(fact_observer: Arc<RegisterFactObserver>) -> Self {
        Self {
            registers: Arc::new(RwLock::new(HashMap::new())),
            ast_context_map: Arc::new(RwLock::new(HashMap::new())),
            owner_map: Arc::new(RwLock::new(HashMap::new())),
            epoch: Arc::new(RwLock::new(1)), // Start at epoch 1
            fact_observer: Some(fact_observer),
        }
    }
    
    /// Get the current epoch
    pub fn current_epoch(&self) -> u64 {
        *self.epoch.read().unwrap()
    }
    
    /// Increment the epoch
    pub fn increment_epoch(&self) -> u64 {
        let mut epoch = self.epoch.write().unwrap();
        *epoch += 1;
        *epoch
    }
    
    /// Add a register to the internal maps
    fn add_to_maps(&self, register: &Register, ast_context: Option<&AstContext>) -> Result<()> {
        // Add to owner map
        let mut owner_map = self.owner_map.write().unwrap();
        owner_map
            .entry(register.owner.clone())
            .or_insert_with(Vec::new)
            .push(register.register_id);
            
        // Add to AST context map if provided
        if let Some(ctx) = ast_context {
            let ctx_str = ctx.to_string();
            let mut ast_map = self.ast_context_map.write().unwrap();
            ast_map
                .entry(ctx_str)
                .or_insert_with(Vec::new)
                .push(register.register_id);
        }
        
        Ok(())
    }
    
    /// Remove a register from the internal maps
    fn remove_from_maps(&self, register_id: &RegisterId, owner: &Address) -> Result<()> {
        // Remove from owner map
        let mut owner_map = self.owner_map.write().unwrap();
        if let Some(ids) = owner_map.get_mut(owner) {
            ids.retain(|id| id != register_id);
        }
        
        // Remove from AST context map (all contexts)
        let mut ast_map = self.ast_context_map.write().unwrap();
        for (_ctx, ids) in ast_map.iter_mut() {
            ids.retain(|id| id != register_id);
        }
        
        Ok(())
    }
    
    /// Verify an authorization
    async fn verify_authorization(
        &self,
        register_id: Option<&RegisterId>,
        auth_method: &AuthorizationMethod,
    ) -> Result<bool> {
        // In a real implementation, this would verify the authorization
        // based on the method type. For this minimal implementation, 
        // we'll simply approve all authorizations.
        
        // We could add mock verification for different authorization types:
        match auth_method {
            AuthorizationMethod::ZKProofAuthorization { .. } => {
                // TODO: Implement ZK proof verification
                Ok(true)
            }
            AuthorizationMethod::TokenOwnershipAuthorization { .. } => {
                // TODO: Implement token ownership verification
                Ok(true)
            }
            _ => Ok(true),
        }
    }
    
    /// Generate a trace ID for fact observation
    fn generate_trace_id(&self) -> TraceId {
        // Simple implementation just generates a random trace ID
        TraceId(uuid::Uuid::new_v4().to_string())
    }
    
    /// Get the current block height (mock implementation)
    fn current_block_height(&self) -> BlockHeight {
        // In a real implementation, this would fetch the current block height
        // For now, just use the epoch as a mock block height
        self.current_epoch()
    }
}

impl Default for InMemoryRegisterService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegisterService for InMemoryRegisterService {
    /// Create a new register
    async fn create_register(
        &self,
        owner: Address,
        domain: Domain,
        contents: RegisterContents,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<RegisterId> {
        // Verify authorization
        if !self.verify_authorization(None, &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to create register".to_string()));
        }
        
        let register_id = RegisterId::new();
        let epoch = self.current_epoch();
        let tx_id = format!("tx-{}", uuid::Uuid::new_v4()); // Mock transaction ID
        
        // Create the register
        let register = Register::new(
            register_id,
            owner.clone(),
            domain.clone(),
            contents.clone(),
            epoch,
            tx_id.clone(),
        );
        
        // Add to register map
        let mut registers = self.registers.write().unwrap();
        registers.insert(register_id, register.clone());
        
        // Add to other maps
        self.add_to_maps(&register, ast_context.as_ref())?;
        
        // Emit a fact about the register creation if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            observer.observe_register_creation(
                trace_id,
                register_id,
                &contents,
                &owner,
                &domain,
            )?;
        }
        
        Ok(register_id)
    }
    
    /// Get a register by ID
    async fn get_register(&self, register_id: &RegisterId) -> Result<Register> {
        let registers = self.registers.read().unwrap();
        registers
            .get(register_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))
    }
    
    /// Update a register
    async fn update_register(
        &self,
        register_id: &RegisterId,
        new_contents: RegisterContents,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<()> {
        // Verify authorization
        if !self.verify_authorization(Some(register_id), &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to update register".to_string()));
        }
        
        // Get the register
        let mut registers = self.registers.write().unwrap();
        let register = registers
            .get_mut(register_id)
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Store previous version hash (for fact logging)
        let previous_version = format!("hash-{}", register.contents.len());
        
        // Update the register contents
        register.update_contents(new_contents.clone())?;
        
        // Add to AST context map if provided
        if let Some(ctx) = ast_context {
            let ctx_str = ctx.to_string();
            let mut ast_map = self.ast_context_map.write().unwrap();
            ast_map
                .entry(ctx_str)
                .or_insert_with(Vec::new)
                .push(*register_id);
        }
        
        // Emit a fact about the register update if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            observer.observe_register_update(
                trace_id,
                *register_id,
                &new_contents,
                &previous_version,
            )?;
        }
        
        Ok(())
    }
    
    /// Transfer register ownership
    async fn transfer_register(
        &self,
        register_id: &RegisterId,
        new_owner: Address,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<()> {
        // Verify authorization
        if !self.verify_authorization(Some(register_id), &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to transfer register".to_string()));
        }
        
        // Get the register
        let mut registers = self.registers.write().unwrap();
        let register = registers
            .get_mut(register_id)
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        if !register.state.is_active() {
            return Err(Error::InvalidOperation(format!(
                "Cannot transfer register {}: not in active state", register_id
            )));
        }
        
        // Store previous owner for fact logging
        let previous_owner = register.owner.clone();
        
        // Update maps
        self.remove_from_maps(register_id, &register.owner)?;
        
        // Update owner
        register.owner = new_owner.clone();
        register.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Add to new owner map
        let mut owner_map = self.owner_map.write().unwrap();
        owner_map
            .entry(new_owner.clone())
            .or_insert_with(Vec::new)
            .push(*register_id);
        
        // Add to AST context map if provided
        if let Some(ctx) = ast_context {
            let ctx_str = ctx.to_string();
            let mut ast_map = self.ast_context_map.write().unwrap();
            ast_map
                .entry(ctx_str)
                .or_insert_with(Vec::new)
                .push(*register_id);
        }
        
        // Emit a fact about the ownership transfer if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            observer.observe_register_ownership_transfer(
                trace_id,
                *register_id,
                &previous_owner,
                &new_owner,
            )?;
        }
        
        Ok(())
    }
    
    /// Delete a register
    async fn delete_register(
        &self,
        register_id: &RegisterId,
        authorization: AuthorizationMethod,
        ast_context: Option<AstContext>,
    ) -> Result<()> {
        // Verify authorization
        if !self.verify_authorization(Some(register_id), &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to delete register".to_string()));
        }
        
        // Get the register
        let mut registers = self.registers.write().unwrap();
        let register = registers
            .get_mut(register_id)
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        if !register.state.is_active() {
            return Err(Error::InvalidOperation(format!(
                "Cannot delete register {}: not in active state", register_id
            )));
        }
        
        // Store previous state for fact logging
        let previous_state = format!("{:?}", register.state);
        
        // Mark as tombstone
        register.state = crate::resource::register::RegisterState::Tombstone;
        register.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Remove from maps
        self.remove_from_maps(register_id, &register.owner)?;
        
        // Emit a fact about the state change if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            observer.observe_register_state_change(
                trace_id,
                *register_id,
                &previous_state,
                "Tombstone",
                "register_deleted",
            )?;
        }
        
        Ok(())
    }
    
    /// Apply a register operation
    async fn apply_operation(
        &self,
        operation: RegisterOperation,
    ) -> Result<Vec<RegisterId>> {
        match operation.op_type {
            OperationType::CreateRegister => {
                // Verify authorization
                if !self.verify_authorization(None, &operation.authorization).await? {
                    return Err(Error::Unauthorized("Not authorized to create register".to_string()));
                }
                
                // Operation should have new contents
                let contents = operation.new_contents.ok_or_else(|| 
                    Error::InvalidArgument("CreateRegister operation requires new_contents".to_string()))?;
                
                // Mock domain and owner
                let domain = "default".to_string(); // Mock domain
                let owner = "owner123".to_string(); // Mock owner
                
                // Create the register
                let register_id = self.create_register(
                    owner, 
                    domain, 
                    contents, 
                    operation.authorization, 
                    operation.ast_context
                ).await?;
                
                Ok(vec![register_id])
            },
            OperationType::UpdateRegister => {
                // Operation should have a register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("UpdateRegister operation requires register ID".to_string()))?;
                
                // Operation should have new contents
                let contents = operation.new_contents.ok_or_else(|| 
                    Error::InvalidArgument("UpdateRegister operation requires new_contents".to_string()))?;
                
                // Update the register
                self.update_register(
                    register_id, 
                    contents, 
                    operation.authorization, 
                    operation.ast_context
                ).await?;
                
                Ok(vec![*register_id])
            },
            OperationType::DeleteRegister => {
                // Operation should have a register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("DeleteRegister operation requires register ID".to_string()))?;
                
                // Delete the register
                self.delete_register(
                    register_id, 
                    operation.authorization, 
                    operation.ast_context
                ).await?;
                
                Ok(vec![])
            },
            OperationType::TransferOwnership(new_owner) => {
                // Operation should have a register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("TransferOwnership operation requires register ID".to_string()))?;
                
                // Transfer the register
                self.transfer_register(
                    register_id, 
                    new_owner, 
                    operation.authorization, 
                    operation.ast_context
                ).await?;
                
                Ok(vec![*register_id])
            },
            OperationType::MergeRegisters => {
                // This is a more complex operation that would create a new register
                // from the contents of multiple registers.
                // For now, return not implemented
                
                // If we implement it, we would need to emit a register merge fact
                if let Some(observer) = &self.fact_observer {
                    let trace_id = self.generate_trace_id();
                    let result_register = RegisterId::new(); // Generate a new ID for result
                    observer.observe_register_merge(
                        trace_id,
                        &operation.registers,
                        result_register,
                    )?;
                }
                
                Err(Error::NotImplemented("MergeRegisters operation not implemented".to_string()))
            },
            OperationType::SplitRegister => {
                // This is a more complex operation that would create multiple new registers
                // from the contents of a single register.
                // For now, return not implemented
                
                // If we implement it, we would need to emit a register split fact
                if let Some(observer) = &self.fact_observer && operation.registers.len() > 0 {
                    let trace_id = self.generate_trace_id();
                    let source_register = operation.registers[0];
                    let result_registers = vec![RegisterId::new(), RegisterId::new()]; // Generate new IDs
                    observer.observe_register_split(
                        trace_id,
                        source_register,
                        &result_registers,
                    )?;
                }
                
                Err(Error::NotImplemented("SplitRegister operation not implemented".to_string()))
            },
            OperationType::CompositeOperation(ops) => {
                // This would apply multiple operations in sequence
                // For now, return not implemented
                Err(Error::NotImplemented("CompositeOperation not implemented".to_string()))
            },
        }
    }
    
    /// Verify a register operation
    async fn verify_operation(
        &self,
        operation: &RegisterOperation,
    ) -> Result<bool> {
        // In a real implementation, this would verify the operation
        // against the register state and the provided authorization.
        // For this minimal implementation, we'll simply verify the authorization.
        
        match operation.op_type {
            OperationType::CreateRegister => {
                self.verify_authorization(None, &operation.authorization).await
            },
            _ => {
                // Get the first register ID
                let register_id = operation.registers.first().ok_or_else(|| 
                    Error::InvalidArgument("Operation requires at least one register ID".to_string()))?;
                    
                self.verify_authorization(Some(register_id), &operation.authorization).await
            }
        }
    }
    
    /// Get registers associated with an AST context
    async fn get_registers_by_ast_context(
        &self,
        ast_context: &AstContext,
    ) -> Result<Vec<Register>> {
        let ctx_str = ast_context.to_string();
        let ast_map = self.ast_context_map.read().unwrap();
        let register_ids = ast_map.get(&ctx_str).cloned().unwrap_or_default();
        
        let registers = self.registers.read().unwrap();
        let result = register_ids
            .iter()
            .filter_map(|id| registers.get(id).cloned())
            .collect();
            
        Ok(result)
    }
    
    /// Get all registers owned by an address
    async fn get_registers_by_owner(
        &self,
        owner: &Address,
    ) -> Result<Vec<Register>> {
        let owner_map = self.owner_map.read().unwrap();
        let register_ids = owner_map.get(owner).cloned().unwrap_or_default();
        
        let registers = self.registers.read().unwrap();
        let result = register_ids
            .iter()
            .filter_map(|id| registers.get(id).cloned())
            .collect();
            
        Ok(result)
    }
    
    /// Consume a register (mark it as one-time use)
    async fn consume_register(
        &self,
        register_id: &RegisterId,
        transaction_id: &str,
        successors: &[RegisterId],
        authorization: AuthorizationMethod,
    ) -> Result<()> {
        // Verify authorization
        if !self.verify_authorization(Some(register_id), &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to consume register".to_string()));
        }
        
        // Get the register
        let mut registers = self.registers.write().unwrap();
        let register = registers
            .get_mut(register_id)
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        if !register.state.is_active() {
            return Err(Error::InvalidOperation(format!(
                "Cannot consume register {}: not in active state", register_id
            )));
        }
        
        // Store previous state for fact logging
        let previous_state = format!("{:?}", register.state);
        
        // Mark as consumed
        register.state = crate::resource::register::RegisterState::Consumed;
        register.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Generate a nullifier (mock implementation)
        let nullifier = format!("nullifier-{}-{}", register_id, transaction_id);
        let block_height = self.current_block_height();
        
        // Emit a fact about the consumption if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            
            // Log the consumption
            observer.observe_register_consumption(
                trace_id.clone(),
                *register_id,
                transaction_id,
                &nullifier,
                successors,
                block_height,
            )?;
            
            // Also log the state change
            observer.observe_register_state_change(
                trace_id,
                *register_id,
                &previous_state,
                "Consumed",
                &format!("consumed_by_tx_{}", transaction_id),
            )?;
        }
        
        Ok(())
    }
    
    /// Lock a register
    async fn lock_register(
        &self,
        register_id: &RegisterId,
        reason: &str,
        authorization: AuthorizationMethod,
    ) -> Result<()> {
        // Verify authorization
        if !self.verify_authorization(Some(register_id), &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to lock register".to_string()));
        }
        
        // Get the register
        let mut registers = self.registers.write().unwrap();
        let register = registers
            .get_mut(register_id)
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        if !register.state.is_active() {
            return Err(Error::InvalidOperation(format!(
                "Cannot lock register {}: not in active state", register_id
            )));
        }
        
        // Store previous state for fact logging
        let previous_state = format!("{:?}", register.state);
        
        // Mark as locked
        register.state = crate::resource::register::RegisterState::Locked;
        register.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Emit a fact about the lock if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            
            // Log the lock operation
            observer.observe_register_lock(
                trace_id.clone(),
                *register_id,
                reason,
            )?;
            
            // Also log the state change
            observer.observe_register_state_change(
                trace_id,
                *register_id,
                &previous_state,
                "Locked",
                reason,
            )?;
        }
        
        Ok(())
    }
    
    /// Unlock a register
    async fn unlock_register(
        &self,
        register_id: &RegisterId,
        reason: &str,
        authorization: AuthorizationMethod,
    ) -> Result<()> {
        // Verify authorization
        if !self.verify_authorization(Some(register_id), &authorization).await? {
            return Err(Error::Unauthorized("Not authorized to unlock register".to_string()));
        }
        
        // Get the register
        let mut registers = self.registers.write().unwrap();
        let register = registers
            .get_mut(register_id)
            .ok_or_else(|| Error::NotFound(format!("Register {} not found", register_id)))?;
        
        // Check if the register is locked
        if !matches!(register.state, crate::resource::register::RegisterState::Locked) {
            return Err(Error::InvalidOperation(format!(
                "Cannot unlock register {}: not in locked state", register_id
            )));
        }
        
        // Store previous state for fact logging
        let previous_state = format!("{:?}", register.state);
        
        // Mark as active
        register.state = crate::resource::register::RegisterState::Active;
        register.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Emit a fact about the unlock if we have an observer
        if let Some(observer) = &self.fact_observer {
            let trace_id = self.generate_trace_id();
            
            // Log the unlock operation
            observer.observe_register_unlock(
                trace_id.clone(),
                *register_id,
                reason,
            )?;
            
            // Also log the state change
            observer.observe_register_state_change(
                trace_id,
                *register_id,
                &previous_state,
                "Active",
                reason,
            )?;
        }
        
        Ok(())
    }
} 