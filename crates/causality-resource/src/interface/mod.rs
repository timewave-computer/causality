// Resource interfaces module
//
// This module defines the core interfaces for the resource management system.

mod access;
mod lifecycle;
mod locking;
mod dependency;
mod context;
mod state;
mod resource_trait;
mod capability;
mod adapter;
mod deprecation;

// Export the main interfaces
pub use access::*;
pub use lifecycle::*;
pub use locking::*;
pub use dependency::*;
pub use context::*;
pub use state::*;
pub use resource_trait::*;
pub use capability::*;
pub use adapter::*;
pub use deprecation::*;

// Re-export the deprecation macros at the interface level
pub use crate::deprecated_warning;
pub use crate::deprecated_error; 