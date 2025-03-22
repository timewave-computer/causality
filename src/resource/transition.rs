// Register Transition System
//
// This module implements the register transition system for the one-time use register model,
// including consumption, archival, summarization, and transition observers.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::resource::register::{RegisterId, RegisterContents, Register, BlockHeight};
use crate::resource::lifecycle::{RegisterState, TransitionReason, StateTransition};
use crate::resource::nullifier::{RegisterNullifier, NullifierRegistry, SharedNullifierRegistry};
use crate::types::Domain;

/// Type of register transition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    /// Register is consumed (one-time use)
    Consumption {
        /// Transaction ID that consumed the register
        transaction_id: String,
        
        /// Successors to this register (e.g., for split or merge operations)
        successors: Vec<RegisterId>,
        
        /// Nullifier created for this consumption
        nullifier: Option<RegisterNullifier>,
    },
    
    /// Register is archived for storage efficiency
    Archival {
        /// Reference to the archived data
        archive_reference: String,
        
        /// Reason for archival
        reason: String,
    },
    
    /// Register is converted to a summary of other registers
    Summarization {
        /// List of register IDs that this summarizes
        summarized_registers: Vec<RegisterId>,
        
        /// Summary generation method
        summary_method: String,
    },
    
    /// Register state change (e.g., active to locked)
    StateChange {
        /// Previous state
        from_state: RegisterState,
        
        /// New state
        to_state: RegisterState,
        
        /// Reason for state change
        reason: TransitionReason,
    },
    
    /// Register is migrated (e.g., for version changes)
    Migration {
        /// Source register
        source_register_id: RegisterId,
        
        /// Destination register
        destination_register_id: RegisterId,
        
        /// Migration method
        migration_method: String,
    },
}

impl fmt::Display for TransitionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Consumption { transaction_id, .. } => {
                write!(f, "Consumption (tx: {})", transaction_id)
            }
            Self::Archival { archive_reference, reason } => {
                write!(f, "Archival (ref: {}, reason: {})", archive_reference, reason)
            }
            Self::Summarization { summarized_registers, summary_method } => {
                write!(
                    f,
                    "Summarization ({} registers, method: {})",
                    summarized_registers.len(),
                    summary_method
                )
            }
            Self::StateChange { from_state, to_state, .. } => {
                write!(f, "StateChange ({} → {})", from_state, to_state)
            }
            Self::Migration { source_register_id, destination_register_id, .. } => {
                write!(
                    f,
                    "Migration ({} → {})",
                    source_register_id, destination_register_id
                )
            }
        }
    }
}

/// A record of a register transition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterTransition {
    /// Register ID that was transitioned
    pub register_id: RegisterId,
    
    /// Type of transition
    pub transition_type: TransitionType,
    
    /// Block height when the transition occurred
    pub block_height: BlockHeight,
    
    /// Timestamp of the transition
    pub timestamp: u64,
    
    /// Domain of the register
    pub domain: Domain,
}

impl RegisterTransition {
    /// Create a new register transition
    pub fn new(
        register_id: RegisterId,
        transition_type: TransitionType,
        block_height: BlockHeight,
        domain: Domain,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            register_id,
            transition_type,
            block_height,
            timestamp,
            domain,
        }
    }
}

/// Handler for register transition events
pub trait TransitionObserver: Send + Sync {
    /// Called when a register transition occurs
    fn on_transition(&self, transition: &RegisterTransition) -> Result<()>;
    
    /// Name of the observer for logging
    fn name(&self) -> &str;
}

/// System for managing register transitions
pub struct TransitionSystem {
    /// Registry of nullifiers
    nullifier_registry: SharedNullifierRegistry,
    
    /// Observers for transitions
    observers: Vec<Arc<dyn TransitionObserver>>,
    
    /// Current block height
    current_block_height: BlockHeight,
    
    /// History of transitions
    transition_history: HashMap<RegisterId, Vec<RegisterTransition>>,
}

impl TransitionSystem {
    /// Create a new transition system
    pub fn new(
        nullifier_registry: SharedNullifierRegistry,
        current_block_height: BlockHeight,
    ) -> Self {
        Self {
            nullifier_registry,
            observers: Vec::new(),
            current_block_height,
            transition_history: HashMap::new(),
        }
    }
    
    /// Update the current block height
    pub fn update_block_height(&mut self, block_height: BlockHeight) -> Result<()> {
        self.current_block_height = block_height;
        self.nullifier_registry.update_block_height(block_height)?;
        Ok(())
    }
    
    /// Add a transition observer
    pub fn add_observer(&mut self, observer: Arc<dyn TransitionObserver>) {
        self.observers.push(observer);
    }
    
    /// Remove a transition observer by name
    pub fn remove_observer(&mut self, name: &str) {
        self.observers.retain(|o| o.name() != name);
    }
    
    /// Apply a transition to a register
    pub fn apply_transition(
        &mut self,
        register: &mut Register,
        transition_type: TransitionType,
    ) -> Result<RegisterTransition> {
        // Validate that the transition is allowed
        self.validate_transition(register, &transition_type)?;
        
        // Create the transition record
        let transition = RegisterTransition::new(
            register.register_id.clone(),
            transition_type.clone(),
            self.current_block_height,
            register.domain.clone(),
        );
        
        // Update the register based on the transition type
        match &transition_type {
            TransitionType::Consumption { transaction_id, successors, .. } => {
                // Generate and record nullifier
                let nullifier = self.nullifier_registry.register_nullifier(
                    register.register_id.clone(),
                    transaction_id.clone(),
                )?;
                
                // Mark register as consumed
                register.state = RegisterState::Consumed;
                register.consumed_by_tx = Some(transaction_id.clone());
                register.successors = successors.clone();
                
                // Update the transition record with the nullifier
                let mut updated_transition = transition.clone();
                if let TransitionType::Consumption { transaction_id, successors, .. } = &mut updated_transition.transition_type {
                    *nullifier = Some(nullifier);
                }
                
                // Store the updated transition
                self.store_transition(updated_transition.clone())?;
                
                // Notify observers
                self.notify_observers(&updated_transition)?;
                
                Ok(updated_transition)
            }
            
            TransitionType::Archival { archive_reference, .. } => {
                // Mark register as archived
                register.state = RegisterState::Archived;
                register.archive_reference = Some(archive_reference.clone());
                
                // Store the transition
                self.store_transition(transition.clone())?;
                
                // Notify observers
                self.notify_observers(&transition)?;
                
                Ok(transition)
            }
            
            TransitionType::Summarization { summarized_registers, .. } => {
                // Mark register as a summary
                register.state = RegisterState::Summary;
                register.summarizes = Some(summarized_registers.clone());
                
                // Store the transition
                self.store_transition(transition.clone())?;
                
                // Notify observers
                self.notify_observers(&transition)?;
                
                Ok(transition)
            }
            
            TransitionType::StateChange { from_state, to_state, reason } => {
                // Check that the current state matches the expected from_state
                if register.state != *from_state {
                    return Err(Error::InvalidStateTransition(
                        format!("Register is in state {:?}, not {:?}", register.state, from_state)
                    ));
                }
                
                // Update register state
                register.state = *to_state;
                
                // Store the transition
                self.store_transition(transition.clone())?;
                
                // Notify observers
                self.notify_observers(&transition)?;
                
                Ok(transition)
            }
            
            TransitionType::Migration { destination_register_id, .. } => {
                // Store source register ID in some lookup table
                // This would be implemented based on the specific migration requirements
                
                // Store the transition
                self.store_transition(transition.clone())?;
                
                // Notify observers
                self.notify_observers(&transition)?;
                
                Ok(transition)
            }
        }
    }
    
    /// Validate that a transition is allowed for a register
    fn validate_transition(&self, register: &Register, transition_type: &TransitionType) -> Result<()> {
        match transition_type {
            TransitionType::Consumption { .. } => {
                // Can only consume active registers
                if !register.state.is_active() && !register.state.is_pending_consumption() {
                    return Err(Error::InvalidStateTransition(
                        format!("Cannot consume register in state {:?}", register.state)
                    ));
                }
                
                // Check if there's already a nullifier for this register
                if self.nullifier_registry.has_nullifier(&register.register_id)? {
                    return Err(Error::AlreadyExists(
                        format!("Nullifier already exists for register {}", register.register_id)
                    ));
                }
            }
            
            TransitionType::Archival { .. } => {
                // Can only archive active or summary registers
                if !register.state.is_active() && !register.state.is_summary() {
                    return Err(Error::InvalidStateTransition(
                        format!("Cannot archive register in state {:?}", register.state)
                    ));
                }
            }
            
            TransitionType::Summarization { summarized_registers, .. } => {
                // Can only summarize active registers
                if !register.state.is_active() {
                    return Err(Error::InvalidStateTransition(
                        format!("Cannot convert register to summary in state {:?}", register.state)
                    ));
                }
                
                // Must summarize at least one register
                if summarized_registers.is_empty() {
                    return Err(Error::InvalidOperation(
                        "Must summarize at least one register".to_string()
                    ));
                }
            }
            
            TransitionType::StateChange { from_state, to_state, .. } => {
                // Check that the current state matches the expected from_state
                if register.state != *from_state {
                    return Err(Error::InvalidStateTransition(
                        format!("Register is in state {:?}, not {:?}", register.state, from_state)
                    ));
                }
                
                // Check that the transition is valid according to state rules
                if !from_state.can_transition_to(*to_state) {
                    return Err(Error::InvalidStateTransition(
                        format!("Cannot transition from {:?} to {:?}", from_state, to_state)
                    ));
                }
            }
            
            TransitionType::Migration { .. } => {
                // Can only migrate active registers
                if !register.state.is_active() {
                    return Err(Error::InvalidStateTransition(
                        format!("Cannot migrate register in state {:?}", register.state)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Store a transition in the history
    fn store_transition(&mut self, transition: RegisterTransition) -> Result<()> {
        self.transition_history
            .entry(transition.register_id.clone())
            .or_insert_with(Vec::new)
            .push(transition);
        
        Ok(())
    }
    
    /// Notify all observers of a transition
    fn notify_observers(&self, transition: &RegisterTransition) -> Result<()> {
        for observer in &self.observers {
            if let Err(e) = observer.on_transition(transition) {
                // Log error but continue notifying other observers
                eprintln!(
                    "Error notifying observer {}: {}",
                    observer.name(),
                    e
                );
            }
        }
        
        Ok(())
    }
    
    /// Get the transition history for a register
    pub fn get_transition_history(&self, register_id: &RegisterId) -> Option<&Vec<RegisterTransition>> {
        self.transition_history.get(register_id)
    }
    
    /// Consume a register (one-time use)
    pub fn consume_register(
        &mut self,
        register: &mut Register,
        transaction_id: &str,
        successors: Vec<RegisterId>,
    ) -> Result<RegisterTransition> {
        let transition_type = TransitionType::Consumption {
            transaction_id: transaction_id.to_string(),
            successors,
            nullifier: None, // Will be set during apply_transition
        };
        
        self.apply_transition(register, transition_type)
    }
    
    /// Archive a register
    pub fn archive_register(
        &mut self,
        register: &mut Register,
        archive_reference: &str,
        reason: &str,
    ) -> Result<RegisterTransition> {
        let transition_type = TransitionType::Archival {
            archive_reference: archive_reference.to_string(),
            reason: reason.to_string(),
        };
        
        self.apply_transition(register, transition_type)
    }
    
    /// Convert a register to a summary
    pub fn summarize_registers(
        &mut self,
        register: &mut Register,
        summarized_registers: Vec<RegisterId>,
        summary_method: &str,
    ) -> Result<RegisterTransition> {
        let transition_type = TransitionType::Summarization {
            summarized_registers,
            summary_method: summary_method.to_string(),
        };
        
        self.apply_transition(register, transition_type)
    }
    
    /// Change the state of a register
    pub fn change_register_state(
        &mut self,
        register: &mut Register,
        to_state: RegisterState,
        reason: TransitionReason,
    ) -> Result<RegisterTransition> {
        let transition_type = TransitionType::StateChange {
            from_state: register.state,
            to_state,
            reason,
        };
        
        self.apply_transition(register, transition_type)
    }
    
    /// Migrate a register to a new register
    pub fn migrate_register(
        &mut self,
        register: &mut Register,
        destination_register_id: RegisterId,
        migration_method: &str,
    ) -> Result<RegisterTransition> {
        let transition_type = TransitionType::Migration {
            source_register_id: register.register_id.clone(),
            destination_register_id,
            migration_method: migration_method.to_string(),
        };
        
        self.apply_transition(register, transition_type)
    }
}

/// Thread-safe wrapper for the TransitionSystem
pub struct SharedTransitionSystem {
    inner: Arc<Mutex<TransitionSystem>>,
}

impl SharedTransitionSystem {
    /// Create a new shared transition system
    pub fn new(
        nullifier_registry: SharedNullifierRegistry,
        current_block_height: BlockHeight,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TransitionSystem::new(
                nullifier_registry,
                current_block_height,
            ))),
        }
    }
    
    /// Update the current block height
    pub fn update_block_height(&self, block_height: BlockHeight) -> Result<()> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.update_block_height(block_height)
    }
    
    /// Add a transition observer
    pub fn add_observer(&self, observer: Arc<dyn TransitionObserver>) -> Result<()> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.add_observer(observer);
        Ok(())
    }
    
    /// Apply a transition to a register
    pub fn apply_transition(
        &self,
        register: &mut Register,
        transition_type: TransitionType,
    ) -> Result<RegisterTransition> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.apply_transition(register, transition_type)
    }
    
    /// Consume a register (one-time use)
    pub fn consume_register(
        &self,
        register: &mut Register,
        transaction_id: &str,
        successors: Vec<RegisterId>,
    ) -> Result<RegisterTransition> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.consume_register(register, transaction_id, successors)
    }
    
    /// Archive a register
    pub fn archive_register(
        &self,
        register: &mut Register,
        archive_reference: &str,
        reason: &str,
    ) -> Result<RegisterTransition> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.archive_register(register, archive_reference, reason)
    }
    
    /// Convert a register to a summary
    pub fn summarize_registers(
        &self,
        register: &mut Register,
        summarized_registers: Vec<RegisterId>,
        summary_method: &str,
    ) -> Result<RegisterTransition> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.summarize_registers(register, summarized_registers, summary_method)
    }
    
    /// Change the state of a register
    pub fn change_register_state(
        &self,
        register: &mut Register,
        to_state: RegisterState,
        reason: TransitionReason,
    ) -> Result<RegisterTransition> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.change_register_state(register, to_state, reason)
    }
    
    /// Migrate a register to a new register
    pub fn migrate_register(
        &self,
        register: &mut Register,
        destination_register_id: RegisterId,
        migration_method: &str,
    ) -> Result<RegisterTransition> {
        let mut system = self.inner.lock().map_err(|_| Error::LockError)?;
        system.migrate_register(register, destination_register_id, migration_method)
    }
    
    /// Get the transition history for a register
    pub fn get_transition_history(&self, register_id: &RegisterId) -> Result<Option<Vec<RegisterTransition>>> {
        let system = self.inner.lock().map_err(|_| Error::LockError)?;
        Ok(system.get_transition_history(register_id).cloned())
    }
}

/// Implementation of TransitionObserver that logs transitions
pub struct LoggingTransitionObserver {
    name: String,
}

impl LoggingTransitionObserver {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl TransitionObserver for LoggingTransitionObserver {
    fn on_transition(&self, transition: &RegisterTransition) -> Result<()> {
        println!(
            "RegisterTransition: {} (register: {}, domain: {}, block: {})",
            transition.transition_type,
            transition.register_id,
            transition.domain,
            transition.block_height
        );
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// A transition observer that emits register transition events (e.g., to a message bus)
pub struct EventEmittingTransitionObserver<F>
where
    F: Fn(&RegisterTransition) -> Result<()> + Send + Sync,
{
    name: String,
    event_handler: F,
}

impl<F> EventEmittingTransitionObserver<F>
where
    F: Fn(&RegisterTransition) -> Result<()> + Send + Sync,
{
    pub fn new(name: String, event_handler: F) -> Self {
        Self { name, event_handler }
    }
}

impl<F> TransitionObserver for EventEmittingTransitionObserver<F>
where
    F: Fn(&RegisterTransition) -> Result<()> + Send + Sync,
{
    fn on_transition(&self, transition: &RegisterTransition) -> Result<()> {
        (self.event_handler)(transition)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::lifecycle::TransitionReason;
    use crate::resource::register::{RegisterId, Register, RegisterContents};
    use crate::types::{Domain, Address};
    use std::sync::Arc;
    
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
    fn test_register_consumption() {
        let nullifier_registry = SharedNullifierRegistry::new(1, 10);
        let mut transition_system = TransitionSystem::new(nullifier_registry, 1);
        let mut register = create_test_register();
        
        // Test consuming a register
        let successor_id = RegisterId::new_unique();
        let transition = transition_system.consume_register(
            &mut register,
            "test-tx-consume",
            vec![successor_id.clone()],
        ).unwrap();
        
        if let TransitionType::Consumption { transaction_id, successors, nullifier } = &transition.transition_type {
            assert_eq!(transaction_id, "test-tx-consume");
            assert_eq!(successors, &vec![successor_id]);
            assert!(nullifier.is_some());
        } else {
            panic!("Expected Consumption transition type");
        }
        
        assert_eq!(register.state, RegisterState::Consumed);
        assert_eq!(register.consumed_by_tx, Some("test-tx-consume".to_string()));
        assert_eq!(register.successors, vec![successor_id]);
        
        // Get transition history
        let history = transition_system.get_transition_history(&register.register_id).unwrap();
        assert_eq!(history.len(), 1);
    }
    
    #[test]
    fn test_register_archival() {
        let nullifier_registry = SharedNullifierRegistry::new(1, 10);
        let mut transition_system = TransitionSystem::new(nullifier_registry, 1);
        let mut register = create_test_register();
        
        // Test archiving a register
        let transition = transition_system.archive_register(
            &mut register,
            "archive-123",
            "Old data archival",
        ).unwrap();
        
        if let TransitionType::Archival { archive_reference, reason } = &transition.transition_type {
            assert_eq!(archive_reference, "archive-123");
            assert_eq!(reason, "Old data archival");
        } else {
            panic!("Expected Archival transition type");
        }
        
        assert_eq!(register.state, RegisterState::Archived);
        assert_eq!(register.archive_reference, Some("archive-123".to_string()));
        
        // Get transition history
        let history = transition_system.get_transition_history(&register.register_id).unwrap();
        assert_eq!(history.len(), 1);
    }
    
    #[test]
    fn test_register_summarization() {
        let nullifier_registry = SharedNullifierRegistry::new(1, 10);
        let mut transition_system = TransitionSystem::new(nullifier_registry, 1);
        let mut register = create_test_register();
        
        // Create some registers to summarize
        let summarized_id1 = RegisterId::new_unique();
        let summarized_id2 = RegisterId::new_unique();
        let summarized_registers = vec![summarized_id1, summarized_id2];
        
        // Test summarizing registers
        let transition = transition_system.summarize_registers(
            &mut register,
            summarized_registers.clone(),
            "merkle-tree",
        ).unwrap();
        
        if let TransitionType::Summarization { summarized_registers: summary_regs, summary_method } = &transition.transition_type {
            assert_eq!(summary_regs, &summarized_registers);
            assert_eq!(summary_method, "merkle-tree");
        } else {
            panic!("Expected Summarization transition type");
        }
        
        assert_eq!(register.state, RegisterState::Summary);
        assert_eq!(register.summarizes, Some(summarized_registers));
        
        // Get transition history
        let history = transition_system.get_transition_history(&register.register_id).unwrap();
        assert_eq!(history.len(), 1);
    }
    
    #[test]
    fn test_register_state_change() {
        let nullifier_registry = SharedNullifierRegistry::new(1, 10);
        let mut transition_system = TransitionSystem::new(nullifier_registry, 1);
        let mut register = create_test_register();
        
        // Test changing register state
        let transition = transition_system.change_register_state(
            &mut register,
            RegisterState::Locked,
            TransitionReason::UserAction("Lock for maintenance".to_string()),
        ).unwrap();
        
        if let TransitionType::StateChange { from_state, to_state, reason } = &transition.transition_type {
            assert_eq!(*from_state, RegisterState::Active);
            assert_eq!(*to_state, RegisterState::Locked);
            if let TransitionReason::UserAction(action) = reason {
                assert_eq!(action, "Lock for maintenance");
            } else {
                panic!("Expected UserAction reason");
            }
        } else {
            panic!("Expected StateChange transition type");
        }
        
        assert_eq!(register.state, RegisterState::Locked);
        
        // Get transition history
        let history = transition_system.get_transition_history(&register.register_id).unwrap();
        assert_eq!(history.len(), 1);
    }
    
    #[test]
    fn test_observer_notification() {
        let nullifier_registry = SharedNullifierRegistry::new(1, 10);
        let mut transition_system = TransitionSystem::new(nullifier_registry, 1);
        
        // Create a test observer
        struct TestObserver {
            called: std::sync::atomic::AtomicBool,
        }
        
        impl TransitionObserver for TestObserver {
            fn on_transition(&self, _transition: &RegisterTransition) -> Result<()> {
                self.called.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            
            fn name(&self) -> &str {
                "test-observer"
            }
        }
        
        let observer = Arc::new(TestObserver {
            called: std::sync::atomic::AtomicBool::new(false),
        });
        
        // Add the observer
        transition_system.add_observer(observer.clone());
        
        // Perform a transition
        let mut register = create_test_register();
        transition_system.change_register_state(
            &mut register,
            RegisterState::Locked,
            TransitionReason::UserAction("Test".to_string()),
        ).unwrap();
        
        // Verify the observer was called
        assert!(observer.called.load(std::sync::atomic::Ordering::SeqCst));
    }
} 