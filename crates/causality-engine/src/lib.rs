// Core engine functionality for the Causality system
// Original file: src/engine/mod.rs
// Last updated: 23.03.27

// Re-export modules
pub mod execution;
pub mod invocation;
pub mod log;
pub mod operation;
pub mod effect;

// Re-export the effect runtime
pub use effect::runtime::{
    EngineEffectRuntime,
    EngineCapabilityVerifier,
    EngineEffectRuntimeFactory,
    get_effect_runtime,
};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 