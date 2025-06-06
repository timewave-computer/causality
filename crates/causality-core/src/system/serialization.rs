//! Serialization utilities for the Causality framework
//!
//! This module provides helpers for SSZ serialization and other
//! serialization-related functionality.

use ssz::{Encode, Decode};
use super::errors::{CausalityError, Result};
use crate::system::content_addressing::{ContentAddressable, EntityId};

// Re-export SSZ traits and types for external use
pub use ssz::{Encode as SszEncode, Decode as SszDecode, DecodeError};

/// SimpleSerialize trait alias for compatibility
pub trait SimpleSerialize: SszEncode + SszDecode + Clone + PartialEq {}

// Blanket implementation for all types that implement the required traits
impl<T> SimpleSerialize for T where T: SszEncode + SszDecode + Clone + PartialEq {}

//-----------------------------------------------------------------------------
// Trait Definitions
//-----------------------------------------------------------------------------

/// Trait for types that can be serialized to bytes
pub trait ToBytes {
    /// Serialize to bytes
    fn to_bytes(&self) -> Vec<u8>;
}

/// Trait for types that can be deserialized from bytes
pub trait FromBytes: Sized {
    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

//-----------------------------------------------------------------------------
// Trait Implementations
//-----------------------------------------------------------------------------

/// Blanket implementation for SSZ types
impl<T: Encode> ToBytes for T {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
}

/// Blanket implementation for SSZ decodable types
impl<T: Decode> FromBytes for T {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        T::from_ssz_bytes(bytes)
            .map_err(|e| CausalityError::SerializationError(format!("SSZ decode error: {:?}", e)))
    }
}

//-----------------------------------------------------------------------------
// Core Helper Functions
//-----------------------------------------------------------------------------

/// Helper to encode a value and compute its hash
pub fn hash_encode<T: Encode>(value: &T) -> [u8; 32] {
    use crate::{Sha256Hasher, Hasher};
    let encoded = value.as_ssz_bytes();
    let hash = Sha256Hasher::hash(&encoded);
    hash
}

/// Helper to encode multiple values together
pub fn encode_tuple<A: Encode, B: Encode>(a: &A, b: &B) -> Vec<u8> {
    let mut bytes = Vec::new();
    a.ssz_append(&mut bytes);
    b.ssz_append(&mut bytes);
    bytes
}

/// Helper to encode a list of items
pub fn encode_list<T: Encode>(items: &[T]) -> Vec<u8> {
    let mut bytes = Vec::new();
    // Encode length as u32
    let len = items.len() as u32;
    len.ssz_append(&mut bytes);
    // Encode each item
    for item in items {
        item.ssz_append(&mut bytes);
    }
    bytes
}

//-----------------------------------------------------------------------------
// Constants and Configuration
//-----------------------------------------------------------------------------

/// Maximum size for serialized data (for safety)
pub const MAX_SERIALIZED_SIZE: usize = 16 * 1024 * 1024; // 16MB

/// Check if a serialized size is within bounds
pub fn check_serialized_size(size: usize) -> Result<()> {
    if size > MAX_SERIALIZED_SIZE {
        Err(CausalityError::SerializationError(
            format!("Serialized size {} exceeds maximum {}", size, MAX_SERIALIZED_SIZE)
        ))
    } else {
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Common SSZ Patterns
//-----------------------------------------------------------------------------

/// Helper for encoding fixed-size byte arrays
pub fn encode_fixed_bytes<const N: usize>(bytes: &[u8; N], buf: &mut Vec<u8>) {
    buf.extend_from_slice(bytes);
}

/// Helper for decoding fixed-size byte arrays
pub fn decode_fixed_bytes<const N: usize>(bytes: &[u8]) -> std::result::Result<[u8; N], ssz::DecodeError> {
    if bytes.len() != N {
        return Err(ssz::DecodeError::InvalidByteLength {
            len: bytes.len(),
            expected: N,
        });
    }
    let mut array = [0u8; N];
    array.copy_from_slice(bytes);
    Ok(array)
}

/// Helper for encoding enum variants with a discriminator byte
pub fn encode_enum_variant(variant: u8, buf: &mut Vec<u8>) {
    buf.push(variant);
}

/// Helper for decoding enum variants
pub fn decode_enum_variant(bytes: &[u8]) -> std::result::Result<(u8, &[u8]), ssz::DecodeError> {
    if bytes.is_empty() {
        return Err(ssz::DecodeError::InvalidByteLength {
            len: 0,
            expected: 1,
        });
    }
    Ok((bytes[0], &bytes[1..]))
}

/// Helper trait for types that can be decoded with remainder
pub trait DecodeWithRemainder: Sized {
    /// Decode from bytes and return the remaining bytes
    fn decode_with_remainder(bytes: &[u8]) -> std::result::Result<(Self, &[u8]), ssz::DecodeError>;
}

/// Helper for encoding variable-length data with a length prefix
pub fn encode_with_length(data: &[u8], buf: &mut Vec<u8>) {
    let len = data.len() as u32;
    len.ssz_append(buf);
    buf.extend_from_slice(data);
}

/// Helper for decoding variable-length data with a length prefix
pub fn decode_with_length(bytes: &[u8]) -> std::result::Result<(&[u8], &[u8]), ssz::DecodeError> {
    if bytes.len() < 4 {
        return Err(ssz::DecodeError::InvalidByteLength {
            len: bytes.len(),
            expected: 4,
        });
    }
    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if bytes.len() < 4 + len {
        return Err(ssz::DecodeError::InvalidByteLength {
            len: bytes.len(),
            expected: 4 + len,
        });
    }
    Ok((&bytes[4..4 + len], &bytes[4 + len..]))
}

/// Macro for implementing SSZ for simple enum types with unit variants
#[macro_export]
macro_rules! impl_ssz_for_unit_enum {
    ($enum_type:ty, $($variant:ident => $value:expr),+ $(,)?) => {
        impl ssz::Encode for $enum_type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                1
            }

            fn ssz_bytes_len(&self) -> usize {
                1
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let byte = match self {
                    $(<$enum_type>::$variant => $value,)+
                };
                buf.push(byte);
            }
        }

        impl ssz::Decode for $enum_type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                1
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
                if bytes.len() != 1 {
                    return Err(ssz::DecodeError::InvalidByteLength {
                        len: bytes.len(),
                        expected: 1,
                    });
                }
                
                match bytes[0] {
                    $($value => Ok(<$enum_type>::$variant),)+
                    _ => Err(ssz::DecodeError::BytesInvalid(
                        format!("Invalid {} variant: {}", stringify!($enum_type), bytes[0]).into()
                    )),
                }
            }
        }
    };
}

/// Macro for implementing SSZ for types that delegate to an inner field
#[macro_export]
macro_rules! impl_ssz_delegate {
    ($type:ty, $inner_field:ident) => {
        impl ssz::Encode for $type {
            fn is_ssz_fixed_len() -> bool {
                <_ as ssz::Encode>::is_ssz_fixed_len(&self.$inner_field)
            }

            fn ssz_bytes_len(&self) -> usize {
                self.$inner_field.ssz_bytes_len()
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                self.$inner_field.ssz_append(buf);
            }
        }

        impl ssz::Decode for $type {
            fn is_ssz_fixed_len() -> bool {
                <_ as ssz::Decode>::is_ssz_fixed_len::<Self>()
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
                Ok(Self {
                    $inner_field: <_>::from_ssz_bytes(bytes)?,
                    ..Default::default()
                })
            }
        }
    };
}

impl<T: Encode> ContentAddressable for T {
    fn content_id(&self) -> EntityId {
        use crate::{Sha256Hasher, Hasher};
        let encoded = self.as_ssz_bytes();
        let hash = Sha256Hasher::hash(&encoded);
        EntityId::from_bytes(hash)
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_encode() {
        let value = vec![1u8, 2, 3, 4];
        let hash1 = hash_encode(&value);
        let hash2 = hash_encode(&value);
        assert_eq!(hash1, hash2); // Deterministic
    }
    
    #[test]
    fn test_encode_tuple() {
        let a = 42u32;
        let b = vec![1u8, 2, 3];
        let encoded = encode_tuple(&a, &b);
        assert!(!encoded.is_empty());
    }
    
    #[test]
    fn test_encode_list() {
        let items = vec![1u32, 2, 3, 4, 5];
        let encoded = encode_list(&items);
        assert!(!encoded.is_empty());
        // Should start with length (5 as u32 = 4 bytes)
        assert_eq!(&encoded[0..4], &5u32.to_le_bytes());
    }
    
    #[test]
    fn test_fixed_bytes() {
        let bytes = [1u8, 2, 3, 4];
        let mut buf = Vec::new();
        encode_fixed_bytes(&bytes, &mut buf);
        assert_eq!(buf, vec![1, 2, 3, 4]);
        
        let decoded: [u8; 4] = decode_fixed_bytes(&buf).unwrap();
        assert_eq!(decoded, bytes);
    }
    
    #[test]
    fn test_enum_variant() {
        let mut buf = Vec::new();
        encode_enum_variant(42, &mut buf);
        assert_eq!(buf, vec![42]);
        
        let (variant, remaining) = decode_enum_variant(&buf).unwrap();
        assert_eq!(variant, 42);
        assert!(remaining.is_empty());
    }
    
    #[test]
    fn test_with_length() {
        let data = b"hello world";
        let mut buf = Vec::new();
        encode_with_length(data, &mut buf);
        
        let (decoded, remaining) = decode_with_length(&buf).unwrap();
        assert_eq!(decoded, data);
        assert!(remaining.is_empty());
    }
} 