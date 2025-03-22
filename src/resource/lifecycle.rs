// Register Lifecycle and State Management
//
// This module implements the register lifecycle stages, state transitions,
// and validation for the one-time use register system as described in
// ADR-006: ZK-Based Register System for Domain Adapters.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

use crate::error::{Error, Result};
use crate::types::{ResourceId, Domain, Address};
use crate::resource::register::{
    RegisterId, RegisterContents, Register, BlockHeight, RegisterOperation, OperationType
};

/// A more comprehensive state enum for registers in the one-time use model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterState {
    /// Register is active and can be used
    Active,
    
    /// Register is locked and cannot be modified temporarily
    Locked,
    
    /// Register is frozen and cannot be used until unfrozen
    Frozen,
    
    /// Register has been consumed and cannot be used again (one-time use)
    Consumed,
    
    /// Register is pending consumption, waiting for nullifier to be verified
    PendingConsumption,
    
    /// Register is archived after garbage collection
    Archived,
    
    /// Register contains a summary of other registers
    Summary,
    
    /// Register is pending deletion
    PendingDeletion,
    
    /// Register has been deleted but is kept as a tombstone
    Tombstone,
    
    /// Register is in an error state
    Error,
}

impl fmt::Display for RegisterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Locked => write!(f, "Locked"),
            Self::Frozen => write!(f, "Frozen"),
            Self::Consumed => write!(f, "Consumed"),
            Self::PendingConsumption => write!(f, "PendingConsumption"),
            Self::Archived => write!(f, "Archived"),
            Self::Summary => write!(f, "Summary"),
            Self::PendingDeletion => write!(f, "PendingDeletion"),
            Self::Tombstone => write!(f, "Tombstone"),
            Self::Error => write!(f, "Error"),
        }
    }
}

impl RegisterState {
    /// Check if the register is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if the register is locked
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Locked)
    }
    
    /// Check if the register is frozen
    pub fn is_frozen(&self) -> bool {
        matches!(self, Self::Frozen)
    }
    
    /// Check if the register is consumed
    pub fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed)
    }
    
    /// Check if the register is pending consumption
    pub fn is_pending_consumption(&self) -> bool {
        matches!(self, Self::PendingConsumption)
    }
    
    /// Check if the register is archived
    pub fn is_archived(&self) -> bool {
        matches!(self, Self::Archived)
    }
    
    /// Check if the register is a summary
    pub fn is_summary(&self) -> bool {
        matches!(self, Self::Summary)
    }
    
    /// Check if the register is pending deletion
    pub fn is_pending_deletion(&self) -> bool {
        matches!(self, Self::PendingDeletion)
    }
    
    /// Check if the register is a tombstone
    pub fn is_tombstone(&self) -> bool {
        matches!(self, Self::Tombstone)
    }
    
    /// Check if the register is in an error state
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
    
    /// Check if the register can be used (i.e., it's in a state where operations are permitted)
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if the register is in a terminal state (cannot transition to another state)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Consumed | Self::Tombstone | Self::Error)
    }
    
    /// Get all valid next states from the current state
    pub fn valid_next_states(&self) -> Vec<RegisterState> {
        match self {
            Self::Active => vec![
                RegisterState::Locked,
                RegisterState::Frozen,
                RegisterState::PendingConsumption,
                RegisterState::PendingDeletion,
            ],
            Self::Locked => vec![
                RegisterState::Active,
                RegisterState::Frozen,
                RegisterState::PendingConsumption,
            ],
            Self::Frozen => vec![
                RegisterState::Active,
                RegisterState::PendingConsumption,
            ],
            Self::PendingConsumption => vec![
                RegisterState::Consumed,
                RegisterState::Active, // If consumption fails
            ],
            Self::PendingDeletion => vec![
                RegisterState::Tombstone,
                RegisterState::Active, // If deletion is canceled
            ],
            Self::Summary => vec![
                RegisterState::Archived,
                RegisterState::PendingDeletion,
            ],
            Self::Archived => vec![
                RegisterState::PendingDeletion,
            ],
            // Terminal states have no valid next states
            Self::Consumed | Self::Tombstone | Self::Error => vec![],
        }
    }
    
    /// Check if a state transition is valid
    pub fn can_transition_to(&self, next_state: RegisterState) -> bool {
        self.valid_next_states().contains(&next_state)
    }
}

/// Reason for a state transition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionReason {
    /// User-initiated action
    UserAction(String),
    
    /// Automatic action by the system
    SystemAction(String),
    
    /// Operation result
    OperationResult(String),
    
    /// Error condition
    Error(String),
    
    /// Lifecycle management
    LifecycleManagement(String),
}

impl fmt::Display for TransitionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserAction(action) => write!(f, "User action: {}", action),
            Self::SystemAction(action) => write!(f, "System action: {}", action),
            Self::OperationResult(result) => write!(f, "Operation result: {}", result),
            Self::Error(error) => write!(f, "Error: {}", error),
            Self::LifecycleManagement(action) => write!(f, "Lifecycle management: {}", action),
        }
    }
}

/// A state transition record for a register
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransition {
    /// From state
    pub from_state: RegisterState,
    
    /// To state
    pub to_state: RegisterState,
    
    /// Reason for the transition
    pub reason: TransitionReason,
    
    /// Block height when the transition occurred
    pub block_height: BlockHeight,
    
    /// Transaction ID that caused the transition
    pub transaction_id: String,
    
    /// Timestamp of the transition
    pub timestamp: u64,
}

impl StateTransition {
    /// Create a new state transition
    pub fn new(
        from_state: RegisterState,
        to_state: RegisterState,
        reason: TransitionReason,
        block_height: BlockHeight,
        transaction_id: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            from_state,
            to_state,
            reason,
            block_height,
            transaction_id,
            timestamp,
        }
    }
}

/// Register lifecycle manager that handles state transitions and validation
pub struct RegisterLifecycleManager {
    /// Transition history for registers
    transition_history: HashMap<RegisterId, Vec<StateTransition>>,
    
    /// Current block height
    current_block_height: BlockHeight,
}

impl RegisterLifecycleManager {
    /// Create a new register lifecycle manager
    pub fn new(current_block_height: BlockHeight) -> Self {
        Self {
            transition_history: HashMap::new(),
            current_block_height,
        }
    }
    
    /// Update the current block height
    pub fn update_block_height(&mut self, block_height: BlockHeight) {
        self.current_block_height = block_height;
    }
    
    /// Transition a register to a new state
    pub fn transition_state(
        &mut self,
        register: &mut Register,
        new_state: RegisterState,
        reason: TransitionReason,
        transaction_id: String,
    ) -> Result<()> {
        let register_id = register.register_id.clone();
        let current_state = register.state;
        
        // Check if the transition is valid
        if !current_state.can_transition_to(new_state) {
            return Err(Error::InvalidStateTransition(
                format!("Cannot transition from {:?} to {:?}", current_state, new_state)
            ));
        }
        
        // Create the transition record
        let transition = StateTransition::new(
            current_state,
            new_state,
            reason,
            self.current_block_height,
            transaction_id,
        );
        
        // Update the register state
        register.state = new_state;
        
        // Record the transition
        self.transition_history
            .entry(register_id)
            .or_insert_with(Vec::new)
            .push(transition);
        
        Ok(())
    }
    
    /// Get the transition history for a register
    pub fn get_transition_history(&self, register_id: &RegisterId) -> Option<&Vec<StateTransition>> {
        self.transition_history.get(register_id)
    }
    
    /// Validate that an operation is permitted for a register's current state
    pub fn validate_operation_for_state(&self, register: &Register, op_type: &OperationType) -> Result<()> {
        match (register.state, op_type) {
            // Active registers can have any operation
            (RegisterState::Active, _) => Ok(()),
            
            // Locked registers can only be unlocked or viewed
            (RegisterState::Locked, OperationType::UpdateRegister) => {
                // Allow only unlock operations on locked registers
                // In a real implementation, we would check if this is an unlock operation
                Err(Error::InvalidOperation("Register is locked".to_string()))
            }
            
            // Frozen registers cannot be modified
            (RegisterState::Frozen, OperationType::UpdateRegister | OperationType::DeleteRegister) => {
                Err(Error::InvalidOperation("Register is frozen".to_string()))
            }
            
            // Consumed registers cannot be used
            (RegisterState::Consumed, _) => {
                Err(Error::InvalidOperation("Register has been consumed".to_string()))
            }
            
            // PendingConsumption registers can only be fully consumed or reverted to active
            (RegisterState::PendingConsumption, op) => {
                match op {
                    // Only allow specific consumption operations
                    _ => Err(Error::InvalidOperation(
                        "Register is pending consumption".to_string()
                    ))
                }
            }
            
            // Archived registers can only be viewed or deleted
            (RegisterState::Archived, OperationType::UpdateRegister) => {
                Err(Error::InvalidOperation("Register is archived".to_string()))
            }
            
            // Summary registers can only be viewed or archived
            (RegisterState::Summary, OperationType::UpdateRegister) => {
                Err(Error::InvalidOperation("Register is a summary".to_string()))
            }
            
            // PendingDeletion registers can only be fully deleted or undeleted
            (RegisterState::PendingDeletion, op) => {
                match op {
                    OperationType::DeleteRegister => Ok(()),
                    _ => Err(Error::InvalidOperation(
                        "Register is pending deletion".to_string()
                    ))
                }
            }
            
            // Tombstone registers cannot be modified
            (RegisterState::Tombstone, _) => {
                Err(Error::InvalidOperation("Register is a tombstone".to_string()))
            }
            
            // Error state registers cannot be used
            (RegisterState::Error, _) => {
                Err(Error::InvalidOperation("Register is in an error state".to_string()))
            }
            
            // Default case for other combinations
            _ => Ok(()),
        }
    }
    
    /// Apply a register operation with state validation
    pub fn apply_operation_with_validation(
        &mut self,
        register: &mut Register,
        operation: &RegisterOperation,
        transaction_id: &str,
    ) -> Result<()> {
        // Validate that the operation is allowed for the current state
        self.validate_operation_for_state(register, &operation.op_type)?;
        
        // Apply state transitions based on operation type
        match operation.op_type {
            OperationType::UpdateRegister => {
                // For update operations, just update the contents
                if let Some(ref new_contents) = operation.new_contents {
                    register.contents = new_contents.clone();
                    register.last_updated = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    register.last_updated_height = self.current_block_height;
                } else {
                    return Err(Error::InvalidOperation(
                        "Update operation requires new contents".to_string()
                    ));
                }
            }
            
            OperationType::DeleteRegister => {
                // For delete operations, transition to tombstone
                self.transition_state(
                    register,
                    RegisterState::Tombstone,
                    TransitionReason::UserAction("Delete register".to_string()),
                    transaction_id.to_string(),
                )?;
            }
            
            OperationType::TransferOwnership(new_owner) => {
                // For transfer operations, update the owner
                register.owner = new_owner.clone();
            }
            
            OperationType::CreateRegister => {
                // Should not happen as the register already exists
                return Err(Error::InvalidOperation(
                    "Cannot create a register that already exists".to_string()
                ));
            }
            
            OperationType::MergeRegisters => {
                // For merge operations, consume the register
                self.transition_state(
                    register,
                    RegisterState::PendingConsumption,
                    TransitionReason::OperationResult("Merged into another register".to_string()),
                    transaction_id.to_string(),
                )?;
            }
            
            OperationType::SplitRegister => {
                // For split operations, consume the register
                self.transition_state(
                    register,
                    RegisterState::PendingConsumption,
                    TransitionReason::OperationResult("Split into multiple registers".to_string()),
                    transaction_id.to_string(),
                )?;
            }
            
            OperationType::CompositeOperation(ref ops) => {
                // For composite operations, process each sub-operation
                for op in ops {
                    // Create a new operation with the same fields but different op_type
                    let sub_operation = RegisterOperation {
                        op_type: op.clone(),
                        registers: operation.registers.clone(),
                        new_contents: operation.new_contents.clone(),
                        authorization: operation.authorization.clone(),
                        proof: operation.proof.clone(),
                        resource_delta: operation.resource_delta.clone(),
                        ast_context: operation.ast_context.clone(),
                    };
                    
                    // Recursive call to process the sub-operation
                    self.apply_operation_with_validation(register, &sub_operation, transaction_id)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Mark a register as consumed (one-time use)
    pub fn consume_register(
        &mut self,
        register: &mut Register,
        transaction_id: &str,
        successors: Vec<RegisterId>,
    ) -> Result<()> {
        // Ensure the register is in a state that can be consumed
        if !matches!(register.state, RegisterState::Active | RegisterState::PendingConsumption) {
            return Err(Error::InvalidStateTransition(
                format!("Cannot consume register in state {:?}", register.state)
            ));
        }
        
        // Set the consume-related fields
        register.consumed_by_tx = Some(transaction_id.to_string());
        register.successors = successors;
        
        // Transition to Consumed state
        self.transition_state(
            register,
            RegisterState::Consumed,
            TransitionReason::OperationResult("Register consumed".to_string()),
            transaction_id.to_string(),
        )
    }
    
    /// Mark a register as pending consumption
    pub fn mark_pending_consumption(
        &mut self,
        register: &mut Register,
        transaction_id: &str,
    ) -> Result<()> {
        // Transition to PendingConsumption state
        self.transition_state(
            register,
            RegisterState::PendingConsumption,
            TransitionReason::SystemAction("Register pending consumption".to_string()),
            transaction_id.to_string(),
        )
    }
    
    /// Archive a register
    pub fn archive_register(
        &mut self,
        register: &mut Register,
        archive_reference: &str,
        transaction_id: &str,
    ) -> Result<()> {
        // Set the archive reference
        register.archive_reference = Some(archive_reference.to_string());
        
        // Transition to Archived state
        self.transition_state(
            register,
            RegisterState::Archived,
            TransitionReason::LifecycleManagement("Register archived".to_string()),
            transaction_id.to_string(),
        )
    }
    
    /// Convert a register to a summary
    pub fn convert_to_summary(
        &mut self,
        register: &mut Register,
        summarizes: Vec<RegisterId>,
        transaction_id: &str,
    ) -> Result<()> {
        // Set the summarizes field
        register.summarizes = Some(summarizes);
        
        // Transition to Summary state
        self.transition_state(
            register,
            RegisterState::Summary,
            TransitionReason::LifecycleManagement("Register converted to summary".to_string()),
            transaction_id.to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Domain;
    
    fn create_test_register() -> Register {
        Register {
            register_id: RegisterId::new_unique(),
            owner: Address::new("owner"),
            domain: Domain::new("test-domain"),
            contents: RegisterContents::empty(),
            state: RegisterState::Active,
            created_at: 0,
            last_updated: 0,
            last_updated_height: 0,
            validity: crate::resource::register::TimeRange::new(0, None),
            epoch: 0,
            created_by_tx: "test-tx".to_string(),
            consumed_by_tx: None,
            successors: Vec::new(),
            summarizes: None,
            archive_reference: None,
            metadata: Default::default(),
        }
    }
    
    #[test]
    fn test_register_state_transitions() {
        let mut lifecycle_manager = RegisterLifecycleManager::new(1);
        let mut register = create_test_register();
        
        // Test valid transition: Active -> Locked
        assert!(lifecycle_manager.transition_state(
            &mut register,
            RegisterState::Locked,
            TransitionReason::UserAction("Lock register".to_string()),
            "test-tx-1".to_string()
        ).is_ok());
        assert_eq!(register.state, RegisterState::Locked);
        
        // Test valid transition: Locked -> Active
        assert!(lifecycle_manager.transition_state(
            &mut register,
            RegisterState::Active,
            TransitionReason::UserAction("Unlock register".to_string()),
            "test-tx-2".to_string()
        ).is_ok());
        assert_eq!(register.state, RegisterState::Active);
        
        // Test valid transition: Active -> PendingConsumption
        assert!(lifecycle_manager.transition_state(
            &mut register,
            RegisterState::PendingConsumption,
            TransitionReason::UserAction("Prepare for consumption".to_string()),
            "test-tx-3".to_string()
        ).is_ok());
        assert_eq!(register.state, RegisterState::PendingConsumption);
        
        // Test valid transition: PendingConsumption -> Consumed
        assert!(lifecycle_manager.transition_state(
            &mut register,
            RegisterState::Consumed,
            TransitionReason::UserAction("Consume register".to_string()),
            "test-tx-4".to_string()
        ).is_ok());
        assert_eq!(register.state, RegisterState::Consumed);
        
        // Test invalid transition: Consumed -> Active (already in terminal state)
        assert!(lifecycle_manager.transition_state(
            &mut register,
            RegisterState::Active,
            TransitionReason::UserAction("Reactivate register".to_string()),
            "test-tx-5".to_string()
        ).is_err());
        assert_eq!(register.state, RegisterState::Consumed);
    }
    
    #[test]
    fn test_operation_validation() {
        let lifecycle_manager = RegisterLifecycleManager::new(1);
        let mut register = create_test_register();
        
        // Active register should allow updates
        assert!(lifecycle_manager.validate_operation_for_state(
            &register,
            &OperationType::UpdateRegister
        ).is_ok());
        
        // Transition to Locked state
        register.state = RegisterState::Locked;
        
        // Locked register should not allow updates
        assert!(lifecycle_manager.validate_operation_for_state(
            &register,
            &OperationType::UpdateRegister
        ).is_err());
        
        // Transition to Consumed state
        register.state = RegisterState::Consumed;
        
        // Consumed register should not allow any operations
        assert!(lifecycle_manager.validate_operation_for_state(
            &register,
            &OperationType::UpdateRegister
        ).is_err());
        assert!(lifecycle_manager.validate_operation_for_state(
            &register,
            &OperationType::DeleteRegister
        ).is_err());
    }
    
    #[test]
    fn test_consume_register() {
        let mut lifecycle_manager = RegisterLifecycleManager::new(1);
        let mut register = create_test_register();
        
        // Test consuming a register
        let successor_id = RegisterId::new_unique();
        assert!(lifecycle_manager.consume_register(
            &mut register,
            "test-tx-consume",
            vec![successor_id.clone()]
        ).is_ok());
        
        assert_eq!(register.state, RegisterState::Consumed);
        assert_eq!(register.consumed_by_tx, Some("test-tx-consume".to_string()));
        assert_eq!(register.successors, vec![successor_id]);
        
        // Get transition history
        let history = lifecycle_manager.get_transition_history(&register.register_id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].from_state, RegisterState::Active);
        assert_eq!(history[0].to_state, RegisterState::Consumed);
    }
} 