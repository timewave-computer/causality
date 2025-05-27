//! Effect state management
//!
//! This module provides state management for effects, allowing effects to
//! maintain state between executions and across boundaries.

use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::any::Any;

use super::{EffectId, EffectError, EffectResult};

/// Effect state
pub trait EffectState: Debug + Send + Sync {
    /// Get the state as any
    fn as_any(&self) -> &dyn Any;
    
    /// Get a mutable reference to the state as any
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// Clone the state
    fn clone_state(&self) -> Box<dyn EffectState>;
}

/// State manager for effects
pub trait StateManager: Debug + Send + Sync {
    /// Get state for an effect
    fn get_state(&self, effect_id: &EffectId) -> EffectResult<Option<Box<dyn EffectState>>>;
    
    /// Set state for an effect
    fn set_state(&self, effect_id: &EffectId, state: Box<dyn EffectState>) -> EffectResult<()>;
    
    /// Remove state for an effect
    fn remove_state(&self, effect_id: &EffectId) -> EffectResult<()>;
    
    /// Check if state exists for an effect
    fn has_state(&self, effect_id: &EffectId) -> bool;
}

/// Simple in-memory state manager
#[derive(Debug, Default)]
pub struct InMemoryStateManager {
    /// States by effect ID
    states: Mutex<HashMap<EffectId, Box<dyn EffectState>>>,
}

impl InMemoryStateManager {
    /// Create a new in-memory state manager
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }
}

impl StateManager for InMemoryStateManager {
    fn get_state(&self, effect_id: &EffectId) -> EffectResult<Option<Box<dyn EffectState>>> {
        let states = self.states.lock().map_err(|e| {
            EffectError::SystemError(format!("Failed to lock state manager: {}", e))
        })?;
        
        Ok(states.get(effect_id).map(|s| s.clone_state()))
    }
    
    fn set_state(&self, effect_id: &EffectId, state: Box<dyn EffectState>) -> EffectResult<()> {
        let mut states = self.states.lock().map_err(|e| {
            EffectError::SystemError(format!("Failed to lock state manager: {}", e))
        })?;
        
        states.insert(effect_id.clone(), state);
        Ok(())
    }
    
    fn remove_state(&self, effect_id: &EffectId) -> EffectResult<()> {
        let mut states = self.states.lock().map_err(|e| {
            EffectError::SystemError(format!("Failed to lock state manager: {}", e))
        })?;
        
        states.remove(effect_id);
        Ok(())
    }
    
    fn has_state(&self, effect_id: &EffectId) -> bool {
        match self.states.lock() {
            Ok(states) => states.contains_key(effect_id),
            Err(_) => false,
        }
    }
} 