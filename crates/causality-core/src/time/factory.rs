// Time Effect Factory
//
// This module provides factory functions for creating and configuring the time effect system.

use std::sync::Arc;
use std::collections::HashMap;

use crate::effect::EffectRegistry;
use super::effect_handler::{TimeEffectHandlerImpl, AttestationStore, InMemoryAttestationStore};
use super::provider::{TimeProvider, TimeProviderFactory};
use super::effect::{TimeEffectHandler, BasicTimeEffectHandler};

/// Configure the time effect system with default components
pub fn configure_time_effect_system<T: EffectRegistry>(registry: &mut T) {
    let provider = TimeProviderFactory::create_real_time_provider();
    let attestation_store = Arc::new(InMemoryAttestationStore::new());
    
    let handler = TimeEffectHandlerImpl::new(provider, attestation_store);
    handler.register(registry);
}

/// Configure the time effect system with custom components
pub fn configure_time_effect_system_with_components<T: EffectRegistry>(
    registry: &mut T,
    provider: Arc<dyn TimeProvider>,
    attestation_store: Arc<dyn AttestationStore>,
) {
    let handler = TimeEffectHandlerImpl::new(provider, attestation_store);
    handler.register(registry);
}

/// Create a simulation time system for testing
pub fn create_simulation_time_system<T: EffectRegistry>(
    registry: &mut T,
    initial_time: Option<crate::time::Timestamp>,
    time_scale: Option<f64>,
) {
    let provider = TimeProviderFactory::create_simulation_provider(initial_time, time_scale);
    let attestation_store = Arc::new(InMemoryAttestationStore::new());
    
    let handler = TimeEffectHandlerImpl::new(provider, attestation_store);
    handler.register(registry);
}

/// Create an in-memory time system for testing or simple applications
pub fn create_in_memory_time_system<T: EffectRegistry>(registry: &mut T) {
    configure_time_effect_system(registry);
} 