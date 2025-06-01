//! System-level utilities and core types
//!
//! This module contains fundamental types and utilities used throughout the
//! Causality framework, including:
//! - Content addressing and entity identification
//! - Common error types
//! - Serialization utilities

/// Content addressing system
pub mod content_addressing;

/// Common error types (simple enum-based)
pub mod errors;

/// Unified error handling (thiserror-based)
pub mod error;

/// Serialization utilities
pub mod serialization;

// Re-export commonly used types
pub use content_addressing::{
    EntityId, ResourceId, ValueExprId, ExprId, RowTypeId, HandlerId,
    TransactionId, IntentId, DomainId, NullifierId, Timestamp, Str,
    ContentAddressable,
};

// Re-export both error systems
pub use errors::{CausalityError, Result as CausalityResult};
pub use error::{Error, Result, ErrorKind, TypeError, MachineError, ReductionError, LinearityError, ResultExt};

pub use serialization::{
    ToBytes, FromBytes, hash_encode, encode_tuple, encode_list,
    check_serialized_size, MAX_SERIALIZED_SIZE,
    encode_fixed_bytes, decode_fixed_bytes, encode_enum_variant, decode_enum_variant,
    DecodeWithRemainder, encode_with_length, decode_with_length,
}; 