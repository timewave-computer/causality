// Copyright 2024 Timewave Labs. All rights reserved.
// Use of this source code is governed by a MIT-style license that can be
// found in the LICENSE file.

//! # Causality Core
//!
//! This crate defines the core traits, types, and interfaces for the
//! Causality system. It forms the foundation upon which other Causality
//! crates are built.
//!
//! ## Key Concepts
//!
//! *   **Effects:** Represent units of computation or interaction.
//! *   **Resources:** Represent state or entities that effects interact with.
//! *   **Capabilities:** Define permissions for effects to interact with resources.
//! *   **Domains:** Represent execution environments (e.g., blockchains, APIs).
//! *   **Time:** Concepts related to temporal ordering and constraints.

// Public API
pub mod capability;
pub mod effect;
pub mod error;
pub mod id_utils;
pub mod identity;
// Temporarily comment out observation module due to unresolved imports
// pub mod observation;
pub mod resource;
pub mod time;
pub mod utils;
pub mod verification;

// Re-export key types and traits for easier access
pub use capability::Capability;
pub use capability::CapabilityError;
pub use capability::CapabilityGrants;

pub use effect::{Effect, EffectContext, EffectHandler, EffectOutcome, EffectId, EffectTypeId, EffectError};
pub use error::{Error, ResourceError, CoreTimeError};

pub use id_utils::generate_content_id;

// Temporarily comment out observation re-exports
// pub use observation::ObservationProxy;

pub use resource::{ResourceId, ResourceState, ResourceTypeRegistry, ResourceStateStore};
pub use resource::storage::types::ResourceStateStorage;

pub use time::Timestamp;

pub use verification::{Verifiable, VerificationError, Attestation};

// From causality_types
pub use causality_types::ContentId;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: Review serialization and utils modules to see if they should remain in core or move.
// TODO: Ensure all traits needed for external integration (storage, crypto, db) are defined here.
