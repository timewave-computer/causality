//! Core type definitions and trait interfaces for the Causality framework.
//!
//! This crate contains only type definitions and trait interfaces, with zero implementation code.
//! It serves as the foundation for the Causality system, providing the minimal set of types
//! needed by other crates while supporting both std and no_std environments.
//!
//! ## Module Organization
//!
//! The crate is organized into six main modules:
//!
//! - **`primitive/`** - Basic primitive types (IDs, strings, numbers, time, errors)
//! - **`resource/`** - Resource types and resource-related functionality
//! - **`expression/`** - Expression system (AST, values, types, results)
//! - **`effect/`** - Effect system and execution model
//! - **`graph/`** - Graph structures and Temporal Effect Language (TEL)
//! - **`system/`** - System-level concerns (serialization, patterns, providers, config)

// #![cfg_attr(not(feature = "std"), no_std)] // Temporarily removed to ensure std for ssz derive
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(missing_docs)]
#![recursion_limit = "256"]

pub use anyhow;
pub use async_trait;
// SSZ serialization system is used for all types

//-----------------------------------------------------------------------------
// Core Modules
//-----------------------------------------------------------------------------

/// Primitive types module - Basic types like IDs, strings, numbers, time, errors
pub mod primitive;

/// Resource types and resource-related functionality
pub mod resource;

/// Expression system types and AST
pub mod expression;

/// Effect system and execution model
pub mod effect;

/// Graph data structures and operations
pub mod graph;

/// System-level concerns - serialization, patterns, providers, configuration
pub mod system;

//-----------------------------------------------------------------------------
// Re-exports for Backward Compatibility
//-----------------------------------------------------------------------------

/// Core primitive type re-exports
pub mod core {
    pub use crate::primitive::*;
    pub use crate::resource::*;
    pub use crate::effect::*;
}

//-----------------------------------------------------------------------------
// Primary Type Re-exports
//-----------------------------------------------------------------------------

// Primitive types
pub use primitive::{
    ids::*,
    string::Str,
    number::Number,
    time::{Timestamp, AsClock, AsTimestampGenerator},
    error::{CausalError, ErrorCategory, ResourceError},
    trait_::*,
};

// Resource types
pub use resource::{
    Resource, ResourceFlow, ResourcePattern, ResourceType, Nullifier,
    conversion::{AsResourceData, ToValueExpr, FromValueExpr, ConversionError},
    state::ResourceState,
};

// Expression types
pub use expression::{
    ast::{AsExpr, Expr},
    r#type::{AsSchema, TypeExpr, TypeExprBox, TypeExprMap, TypeExprVec},
    value::{AsValueExpr, ValueExpr},
    result::{ExprError, ExprResult},
    helper::*,
};

// Effect types
pub use effect::{
    EffectStruct as Effect, Intent, Handler, Transaction, Domain,
    core::{EffectHandler, EffectInput, EffectOutput, HandlerError},
    trace::{ExecutionTrace, ZkExecutionMetadata, ZkExecutionTrace},
};

// Graph types
pub use graph::{
    tel::{EffectGraph, Edge, EdgeKind, ResourceRef},
    element::{Node, Edge as GraphEdge, TypeId},
    subgraph::Subgraph,
    r#trait::{AsEdge, AsNode},
    registry::GraphError,
    execution::{ExecutionContext, ExecutionMode},
    optimization::{OptimizationStrategy, TypedDomain},
    dataflow::{DomainAwareNode, DataflowPort},
};

// System types
pub use system::{
    // Serialization
    Encode, Decode, SimpleSerialize, DecodeError,
    serialize, deserialize, serialize_for_ffi, deserialize_from_ffi,
    MerkleTree, MerkleProof,
    
    // Patterns
    Message, message_schema,
    
    // Provider interfaces
    AsExprContext, AsExecutionContext, AsRuntimeContext,
    TelContextInterface, AsyncTelContextInterface,
    AsDomainScoped, ErasedEffectHandler, AsMessenger,
    AsRegistry, AsRequestDispatcher, AsKeyValueStore, AsMutableKeyValueStore,
    
    // Configuration
    LispContextConfig, LispEvaluationError, LispEvaluator,
    RuntimeConfig, DomainConfig, SystemConfig,
    
    // Compiler output
    CompiledSubgraph, CompiledTeg, CompiledTegMetadata, CompiledTegBuilder,
    
    // Utilities
    AsIdentifiable, AsResolvable, TransformFn,
    get_current_time_ms, SszDuration, SimpleRegistry,
};

//-----------------------------------------------------------------------------
// Legacy Module Re-exports for Backward Compatibility
//-----------------------------------------------------------------------------

/// Legacy utils module - points to system::util
pub mod utils {
    pub use crate::system::util::*;
}

/// Legacy provider module - points to system::provider
pub mod provider {
    pub use crate::system::provider::*;
}



/// Legacy tel module - points to graph::tel
pub mod tel {
    pub use crate::graph::tel::*;
}

/// Legacy serialization module - points to system::serialization
pub mod serialization {
    pub use crate::system::serialization::*;
}

/// Legacy expr module - points to expression
pub mod expr {
    pub use crate::expression::*;
    
    // Specific legacy re-exports
    pub mod ast {
        pub use crate::expression::ast::*;
    }
    
    pub mod value {
        pub use crate::expression::value::*;
    }
    
    pub mod expr_type {
        pub use crate::expression::r#type::*;
    }
    
    pub mod result {
        pub use crate::expression::result::*;
    }
}

//-----------------------------------------------------------------------------
// Feature-specific Re-exports
//-----------------------------------------------------------------------------

// Re-export all core SSZ serialization functionality
pub use system::serialization::*;

// Re-export derive macros when available (TODO: implement derive feature)
// #[cfg(feature = "derive")]
// pub use causality_derive::*;
