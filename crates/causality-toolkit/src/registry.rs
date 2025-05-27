//! Effect registry
//!
//! This module provides a registry for standard effects that can be used in Causality applications.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use causality_types::effects_core::{Effect, EffectInput, EffectOutput};
use causality_types::expr::TypeExpr;
use anyhow::{anyhow, Result};

// Debug-friendly wrapper for effect handlers
#[derive(Debug, Default)]
struct HandlerInfo {
    name: String,
    type_info: String,
}

/// Trait for handlers that can handle specific effect types
pub trait Handles<E: Effect> {
    /// Handle the effect
    fn handle_effect(&self, effect: &E) -> Result<()>;
}

/// Trait for converting effects to Any for type erasure
pub trait EffectAny: Send + Sync + 'static {
    /// Convert to Any for type checking
    fn as_any(&self) -> &dyn Any;
    
    /// Get the effect type name
    fn typ(&self) -> &'static str;
    
    /// Get input schema
    fn input_schema(&self) -> TypeExpr;
    
    /// Get output schema  
    fn output_schema(&self) -> TypeExpr;
}

// Blanket implementation for all Effect types
impl<T: Effect> EffectAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn typ(&self) -> &'static str {
        T::EFFECT_TYPE
    }
    
    fn input_schema(&self) -> TypeExpr {
        T::Input::schema()
    }
    
    fn output_schema(&self) -> TypeExpr {
        T::Output::schema()
    }
}

/// Simple handler trait for type-erased effects
pub trait SimpleEffectHandler: Send + Sync + 'static {
    /// Handle an effect
    fn handle_any(&self, effect: &dyn EffectAny) -> Result<()>;
}

/// Registry for standard effects and their handlers
pub struct EffectRegistry {
    // Map from effect type name to handler
    handlers_by_name: RwLock<HashMap<&'static str, Arc<dyn SimpleEffectHandler>>>,

    // Map from effect TypeId to handler  
    handlers_by_type: RwLock<HashMap<TypeId, Arc<dyn SimpleEffectHandler>>>,

    // Debug-friendly version of the handlers for Debug impl
    #[allow(dead_code)]
    handler_info: RwLock<Vec<HandlerInfo>>,
}

// Manual implementation of Debug for EffectRegistry
impl std::fmt::Debug for EffectRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectRegistry")
            .field("handlers", &self.handler_info)
            .finish()
    }
}

// Manual implementation of Default for EffectRegistry
impl Default for EffectRegistry {
    fn default() -> Self {
        Self {
            handlers_by_name: RwLock::new(HashMap::new()),
            handlers_by_type: RwLock::new(HashMap::new()),
            handler_info: RwLock::new(Vec::new()),
        }
    }
}

impl EffectRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler for an effect type
    pub fn register<E, H>(&self, handler: H)
    where
        E: Effect + 'static,
        H: Handles<E> + SimpleEffectHandler + 'static,
    {
        // Create a placeholder effect to get its type name
        let type_name = std::any::type_name::<E>();
        let type_id = TypeId::of::<E>();

        // Create debug info before moving handler into Arc
        let info = HandlerInfo {
            name: type_name.to_string(),
            type_info: std::any::type_name::<H>().to_string(),
        };

        let handler = Arc::new(handler);

        // Register by name
        {
            let mut handlers = self.handlers_by_name.write().unwrap();
            handlers.insert(E::EFFECT_TYPE, handler.clone());
        }

        // Register by TypeId
        {
            let mut handlers = self.handlers_by_type.write().unwrap();
            handlers.insert(type_id, handler);
        }

        // Store debug info
        {
            let mut info_vec = self.handler_info.write().unwrap();
            info_vec.push(info);
        }
    }

    /// Get a handler for a specific effect
    pub fn get_handler(
        &self,
        effect: &dyn EffectAny,
    ) -> Option<Arc<dyn SimpleEffectHandler>> {
        // Try to find by type name first
        {
            let handlers = self.handlers_by_name.read().unwrap();
            if let Some(handler) = handlers.get(effect.typ()) {
                return Some(handler.clone());
            }
        }

        // Try to find by TypeId if possible
        let type_id = effect.as_any().type_id();
        let handlers = self.handlers_by_type.read().unwrap();
        if let Some(handler) = handlers.get(&type_id) {
            return Some(handler.clone());
        }

        None
    }

    /// Handle an effect using the registered handlers
    pub fn handle(&self, effect: &dyn EffectAny) -> Result<()> {
        if let Some(handler) = self.get_handler(effect) {
            handler.handle_any(effect)
        } else {
            Err(anyhow!("Unhandled effect"))
        }
    }

    /// Register a composed handler that wraps another handler
    pub fn register_composed<F>(&self, wrapper: F)
    where
        F: Fn(Arc<dyn SimpleEffectHandler>) -> Arc<dyn SimpleEffectHandler>,
    {
        // Get all existing handlers
        let name_handlers = {
            let handlers = self.handlers_by_name.read().unwrap();
            handlers
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<Vec<_>>()
        };

        // Wrap each handler and re-register
        for (name, handler) in name_handlers {
            let wrapped = wrapper(handler);
            let mut handlers = self.handlers_by_name.write().unwrap();
            handlers.insert(name, wrapped);
        }

        // Do the same for type-based handlers
        let type_handlers = {
            let handlers = self.handlers_by_type.read().unwrap();
            handlers
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<Vec<_>>()
        };

        for (type_id, handler) in type_handlers {
            let wrapped = wrapper(handler);
            let mut handlers = self.handlers_by_type.write().unwrap();
            handlers.insert(type_id, wrapped);
        }
    }
}
