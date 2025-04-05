// Effect system logging
//
// This module provides logging utilities for the effect system.

use tracing::{debug, error, info, trace, warn};

/// Log effect execution
pub fn log_effect_execution(effect_type: &str, description: &str) {
    debug!("Executing effect: {effect_type} - {description}");
}

/// Log effect success
pub fn log_effect_success(effect_type: &str, description: &str) {
    info!("Effect executed successfully: {effect_type} - {description}");
}

/// Log effect failure
pub fn log_effect_failure(effect_type: &str, description: &str, error: &str) {
    error!("Effect execution failed: {effect_type} - {description} - Error: {error}");
}

/// Log effect registry operation
pub fn log_registry_operation(operation: &str, details: &str) {
    trace!("Registry operation: {operation} - {details}");
}

/// Log effect handler registration
pub fn log_handler_registration(handler_type: &str) {
    debug!("Registered effect handler: {handler_type}");
}

/// Log effect context creation
pub fn log_context_creation(context_id: &str) {
    trace!("Created effect context: {context_id}");
}

/// Log effect warning
pub fn log_effect_warning(warning: &str) {
    warn!("Effect warning: {warning}");
} 