//! Service Provider Interfaces (SPI)
//!
//! Consolidates all Service Provider Interface (SPI) trait definitions for the Causality framework.
//! These interfaces establish the contract between the core system and extension components,
//! enabling flexible dependency injection and component substitution.

//-----------------------------------------------------------------------------
// Module Exports
//-----------------------------------------------------------------------------

// SPIs related to core runtime behaviors and contexts.
// pub mod capability; // Removed module declaration
pub mod context;
pub mod domain_scoped;
pub mod handler;
// pub mod expr_store; // This trait is in causality-runtime, not types. Will be refactored later.
pub mod messenger;
pub mod registry;
pub mod request_dispatcher;
pub mod store;

// Re-export all SPI traits for easier access from other crates.
// pub use capability::AsCapabilityManager; // Removed re-export
pub use context::{
    AsExecutionContext, AsExprContext, AsRuntimeContext, TelContextInterface,
};
pub use domain_scoped::AsDomainScoped;
pub use handler::ErasedEffectHandler;
// pub use expr_store::ExprStore as OldExprStore; // This trait is in causality-runtime.
pub use messenger::AsMessenger;
pub use registry::AsRegistry;
pub use request_dispatcher::AsRequestDispatcher;
pub use store::AsKeyValueStore;
