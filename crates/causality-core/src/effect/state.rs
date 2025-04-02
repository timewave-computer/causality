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

/// Effect state extension trait
pub trait EffectStateExt {
    /// Get state for an effect
    fn get_state<T: 'static>(&self, effect_id: &EffectId) -> EffectResult<Option<T>>
    where
        T: Clone;
    
    /// Set state for an effect
    fn set_state<T: 'static + Debug + Send + Sync>(&self, effect_id: &EffectId, state: T) -> EffectResult<()>;
}

/// Blanket implementation for any state manager
impl<S: StateManager> EffectStateExt for S {
    fn get_state<T: 'static>(&self, effect_id: &EffectId) -> EffectResult<Option<T>>
    where
        T: Clone,
    {
        if let Some(state) = self.get_state(effect_id)? {
            if let Some(typed_state) = state.as_any().downcast_ref::<T>() {
                return Ok(Some(typed_state.clone()));
            }
        }
        
        Ok(None)
    }
    
    fn set_state<T: 'static + Debug + Send + Sync>(&self, effect_id: &EffectId, state: T) -> EffectResult<()> {
        struct TypedState<T: Debug + Send + Sync>(T);
        
        impl<T: Debug + Send + Sync> Debug for TypedState<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "TypedState({:?})", self.0)
            }
        }
        
        impl<T: Debug + Send + Sync + 'static> EffectState for TypedState<T> {
            fn as_any(&self) -> &dyn Any {
                &self.0
            }
            
            fn as_any_mut(&mut self) -> &mut dyn Any {
                &mut self.0
            }
            
            fn clone_state(&self) -> Box<dyn EffectState> {
                Box::new(TypedState(self.0.clone()))
            }
        }
        
        impl<T: Clone> Clone for TypedState<T> 
        where
            T: Debug + Send + Sync
        {
            fn clone(&self) -> Self {
                TypedState(self.0.clone())
            }
        }
        
        self.set_state(effect_id, Box::new(TypedState(state)))
    }
} 