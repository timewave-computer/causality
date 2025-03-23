//! Capability-based access control system for resources
//!
//! This module provides a capability-based approach to resource access control,
//! enabling fine-grained delegation and revocation of access rights.

// Re-export from capability submodules
#[cfg(feature = "resource_capability_proof")]
pub mod proof;

#[cfg(feature = "resource_capability_delegation")]
pub mod delegation;

#[cfg(feature = "resource_capability_validation")]
pub mod validation;

#[cfg(feature = "resource_capability_integration")]
pub mod integration;

// Re-export core capability system functionality
pub use crate::resource::capability_system::{
    RigorousCapability,
    CapabilityConstraint,
    AuthenticationFactor,
    CapabilityProof,
    ProofType,
    CapabilityStatus,
    CapabilitySystem,
    UnifiedCapabilitySystem,
    CapabilityValidator,
    AuthorizationService,
}; 