// Resource model for TEL
// Original file: src/tel/resource/model/mod.rs

// Resource model module for TEL
//
// This module provides the register-based resource model
// as defined in ADR 003.

pub mod register;
pub mod manager;
pub mod guard;

// Re-export core components
pub use register::{
    Register,
    RegisterId,
    RegisterContents,
    RegisterState,
    Resource,
    ResourceLogic,
    ResourceLogicType,
    ControllerLabel,
    ResourceTimeData,
    TimeRange,
};

pub use manager::{
    ResourceManager,
    GarbageCollectionConfig,
    CollectionStats,
};

pub use guard::{
    ResourceGuard,
    ResourceAccessControl,
    SharedResourceManager,
    AccessMode,
}; 
