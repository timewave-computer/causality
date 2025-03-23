// Actor system for Causality
//
// This module provides actor framework functionality for managing and
// communicating between different participants in the system.

// Module declarations
pub mod identity;
pub mod role;
pub mod user;
pub mod operator;
pub mod committee;
pub mod registry;
pub mod communication;
pub mod messaging;
pub mod types;

// Re-exports of core types
pub use types::{GenericActorId, UuidActorId, ActorIdBox};
pub use role::ActorRole; 