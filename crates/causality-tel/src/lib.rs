//! Temporal Effect Language (TEL)
//!
//! TEL is a language for expressing and composing temporal effects.
//! It provides a set of combinators for building and manipulating
//! temporal effects, as well as a query system for retrieving
//! information from temporal effects.

// Import core modules
pub mod ast;
pub mod types;
// Use the file-based parser (not the directory-based one)
#[path = "parser.rs"]
pub mod parser;
pub mod compiler;
pub mod handlers;
pub mod combinators;
pub mod cli;
// Note: The effects module has been moved to causality-engine

// Re-exports of key components
pub use ast::{Program, Flow, Statement, Expression, Literal};
pub use types::{TelType, BaseType, RecordType, TypeEnvironment};
pub use types::row::{RowType, RowError};
// Re-export effect operations and types
pub use types::effect::{EffectRow, EffectError as TelEffectError, TelEffect};
// Re-export effect operations directly from the operations module
pub use types::effect::operations;
pub use compiler::{TelEngineExecutor, TelError};
pub use combinators::Combinator;
// Re-export query system components
pub use combinators::query::{Query, QueryResult, FilterOperator, SortDirection, Projection, AggregationOperation, result_content_id};

// Re-export resource and effect components from causality-core
pub use causality_core::resource::{
    Resource, ResourceId, ResourceTypeId, ResourceState, ResourceInterface, 
    ResourceManager, ResourceError, ResourceResult, ResourceConfig
};

// Re-export effect components from causality-core
pub use causality_core::effect::{
    Effect, EffectId, EffectType, EffectContext, EffectOutcome,
    EffectError as CoreEffectError, EffectRegistry, EffectHandler, HandlerResult,
    DomainEffect, EffectExecutor
};

// Note: The TelEffectAdapter, TelEffectExecutor, and related components are now in causality-engine
// Use causality-engine::effect::tel::{TelEffectAdapter, TelEffectExecutor, TegExecutor} instead 
// of the deprecated TelHandlerAdapter which has been completely removed.

// Make macros available at the root
pub use types::effect::macros;
// Re-export the row and effect macros if they exist
#[cfg(feature = "macros")]
pub use types::row::record;
#[cfg(feature = "macros")]
pub use types::effect::effect;

// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Normalize an effect name to a standard format
pub fn normalize_effect_name(name: &str) -> String {
    name.to_lowercase().replace(" ", "_")
}

#[cfg(test)]
pub mod tests;

// Re-exports for convenience - no duplicates 
pub use combinators::*;
pub use types::*;

// For execution of TEL effects:
// This module provides the type definitions and combinators for TEL,
// but execution requires the causality-engine crate which provides
// TelEffectExecutor and TelEffectAdapter implementations. 

// Re-export TEG conversion functionality
pub use compiler::ToTEG;

// Use Causality IR for TEG integration
use causality_ir;

// Make TEG functionality available in the crate
pub mod teg {
    pub use causality_ir::{TemporalEffectGraph, TEGFragment, EffectNode, ResourceNode};
}

// Re-export ToTEG trait from compiler
pub use compiler::ToTEG; 