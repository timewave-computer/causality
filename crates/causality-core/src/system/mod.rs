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
pub mod domain;
pub mod utils;

// Re-export common types
pub use error::{Error, Result, ErrorKind, ResultExt};
pub use content_addressing::{
    EntityId, ResourceId, ExprId, RowTypeId, HandlerId, TransactionId, IntentId, DomainId, NullifierId,
    Timestamp, Str, ContentAddressable
};
pub use serialization::{
    encode_fixed_bytes, decode_fixed_bytes, DecodeWithRemainder,
    encode_with_length, decode_with_length, encode_enum_variant, decode_enum_variant
};
pub use causality::CausalProof;
pub use domain::Domain;
pub use utils::{get_current_time_ms, SszDuration};

pub use content_addressing::*;
 