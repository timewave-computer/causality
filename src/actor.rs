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
pub use types::{GenericActorId, UuidActorId};
pub use role::ActorRole;

use std::fmt::{Debug, Display};
use std::hash::Hash;

/// Type for actor IDs
///
/// This is a base trait for all actor ID types in the system.
pub trait ActorId: Debug + Display + Clone + PartialEq + Eq + Hash + Send + Sync {} 