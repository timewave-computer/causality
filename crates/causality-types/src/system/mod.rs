//! System Module
//!
//! This module consolidates system-level concerns including serialization,
//! patterns, provider interfaces, configuration, compiler output, and utilities.

pub mod serialization;
pub mod pattern;
pub mod provider;
pub mod config;
pub mod compiler;
pub mod util;

// Re-export commonly used items
pub use serialization::{
    Encode, Decode, SimpleSerialize, DecodeError,
    serialize, deserialize, serialize_for_ffi, deserialize_from_ffi,
    serialize_to_hex, deserialize_from_hex,
    MerkleTree, MerkleProof, verify_proof,
};

pub use pattern::{
    Message, message_schema,
    matching::{MessagePattern, DomainPattern, ResourcePattern},
    capability::{Capability, Permission},
    communication::{RequestResponse, PubSub},
};

pub use provider::{
    AsExprContext, AsExecutionContext, AsRuntimeContext,
    StaticExprContext, TelContextInterface, AsyncTelContextInterface,
    AsDomainScoped, ErasedEffectHandler, AsMessenger,
    AsRegistry, AsRequestDispatcher, AsKeyValueStore, AsMutableKeyValueStore,
    MemoryRegistry, MemoryStore,
};

pub use config::{
    HostFunction, LispContextConfig, LispEvaluationError, LispEvaluator,
    RuntimeConfig, DomainConfig, SystemConfig,
};

pub use compiler::{
    CompiledSubgraph, CompiledTeg, CompiledTegMetadata, CompiledTegBuilder,
    CompilerValidationError,
};

pub use util::{
    AsIdentifiable, AsResolvable, TransformFn,
    constant_transform, identity_transform, mapping_transform, validated_transform,
    get_current_time_ms, get_current_time_us, get_current_time_ns,
    duration_to_ms, ms_to_duration, measure_time,
    SszDuration, CustomDuration, SimpleRegistry,
}; 