//! System-level utilities and infrastructure
//!
//! This module provides cross-cutting concerns like errors, serialization,
//! content addressing, causality tracking, domain management, and nullifier-based
//! resource consumption tracking.

pub mod error;
pub mod errors;
pub mod serialization;
pub mod content_addressing;
pub mod causality;
pub mod deterministic;
pub mod domain;
pub mod utils;
pub mod storage;

// Re-export common types
pub use error::{Error, Result, ErrorKind, ResultExt};
pub use content_addressing::{
    EntityId, ResourceId, ExprId, RowTypeId, HandlerId, TransactionId, IntentId, NullifierId,
    ContentAddressable, Timestamp, Str,
};
pub use serialization::{
    encode_fixed_bytes, decode_fixed_bytes, DecodeWithRemainder,
    encode_with_length, decode_with_length, encode_enum_variant, decode_enum_variant
};
pub use causality::CausalProof;
pub use domain::{Domain, UnifiedRouter, RoutingInfo, RoutingPath, RoutingStrategy, RoutingStats};
pub use utils::{get_current_time_ms, SszDuration};
pub use deterministic::{
    DeterministicSystem, DeterministicFloat, deterministic_system_time,
    deterministic_instant, deterministic_duration_millis, deterministic_lamport_time,
};
pub use storage::{
    StorageCommitment, StorageKeyDerivation, StorageKeyComponent, 
    StorageAddressable, StorageCommitmentBatch
};

pub use content_addressing::*;
 