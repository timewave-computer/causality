//! Effect system and execution model
//!
//! Core effect types, effects, intents, handlers, transactions,
//! domains, and execution traces.

pub mod core;
pub mod types;
pub mod intent;
pub mod handler;
pub mod transaction;
pub mod domain;
pub mod trace;

// Re-exports for convenience - explicit to avoid ambiguity
pub use core::{
    ConversionError, HandlerError, EffectInput, EffectOutput, 
    Effect as EffectTrait, EffectHandler, HandlerRegistrationInfo, inventory
};

// Use explicit imports to avoid name conflicts
pub use types::Effect;
pub use intent::*;
pub use handler::*;
pub use transaction::*;
pub use domain::*;
pub use trace::*; 