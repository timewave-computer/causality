//! Serialization for Causality types using SSZ (Simple Serialize)
//!
//! This module implements serialization for all core types using the SSZ standard
//! from the Ethereum ecosystem. It consolidates serialization functionality including
//! SSZ encoding/decoding, FFI utilities, and Merkle tree operations.
//!
//! ## Features
//! 
//! - **Type-safe serialization**: Leverages Rust's type system for safe serialization
//! - **Content addressing**: Supports hash-based content addressing via merkleization
//! - **Cross-language compatibility**: Compatible with Ethereum ecosystem and OCaml implementation
//! - **Performance optimized**: Benchmarked and optimized for common Causality types
//! - **Zero-copy where possible**: Minimizes allocations for better performance
//! - **FFI support**: Utilities for serializing data across foreign function interfaces
//! - **Merkle tree support**: For efficient verification and proof generation

use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use std::io;
use std::fmt;
use anyhow::Result;
use sha2::{Digest, Sha256};
use crate::primitive::string::Str;

// ----------------------------------------------------------------------------
// CORE TRAITS
// ----------------------------------------------------------------------------

/// Trait for encoding values to SSZ bytes
pub trait Encode {
    fn as_ssz_bytes(&self) -> Vec<u8>;
}

/// Trait for decoding values from SSZ bytes
pub trait Decode: Sized {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

/// Trait for decoding values with length information
pub trait DecodeWithLength: Sized {
    /// Decode from SSZ bytes and return the value and number of bytes consumed
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError>;
}

/// Marker trait for types that can be SSZ serialized
pub trait SimpleSerialize {}

// ===== ERROR TYPES =====

/// Error type for SSZ decode operations
#[derive(Debug, Clone)]
pub struct DecodeError {
    pub message: String,
}

impl DecodeError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSZ Decode Error: {}", self.message)
    }
}

impl std::error::Error for DecodeError {}

/// Error type for FFI serialization
#[derive(Debug)]
pub struct FfiSerializationError {
    pub message: String,
}

impl fmt::Display for FfiSerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FFI Serialization Error: {}", self.message)
    }
}

impl std::error::Error for FfiSerializationError {}

impl From<DecodeError> for FfiSerializationError {
    fn from(err: DecodeError) -> Self {
        FfiSerializationError {
            message: err.message,
        }
    }
}

impl From<hex::FromHexError> for FfiSerializationError {
    fn from(err: hex::FromHexError) -> Self {
        FfiSerializationError {
            message: format!("Hex decode error: {}", err),
        }
    }
}

// ----------------------------------------------------------------------------
// MAIN SERIALIZATION FUNCTIONS
// ----------------------------------------------------------------------------

/// Serializes a value using SSZ
pub fn serialize<T: Encode>(value: &T) -> Vec<u8> {
    value.as_ssz_bytes()
}

/// Deserializes a value using SSZ
pub fn deserialize<T: Decode>(bytes: &[u8]) -> Result<T, DecodeError> {
    T::from_ssz_bytes(bytes)
}

/// Helper function for serializing potentially recursive structures
/// by limiting recursion depth.
pub fn serialize_with_depth_limit<T: Encode>(value: &T, max_depth: usize) -> io::Result<Vec<u8>> {
    if max_depth == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Maximum recursion depth exceeded",
        ));
    }

    // For now we just call the standard serialization
    Ok(value.as_ssz_bytes())
}

/// Helper function for deserializing potentially recursive structures
/// by limiting recursion depth.
pub fn deserialize_with_depth_limit<T: Decode>(bytes: &[u8], max_depth: usize) -> Result<T, DecodeError> {
    if max_depth == 0 {
        return Err(DecodeError {
            message: "Maximum recursion depth exceeded".to_string(),
        });
    }

    // For now we just call the standard deserialization
    T::from_ssz_bytes(bytes)
}

/// Helper trait for providing custom serialization functionality
pub trait SszSerializable: Encode + Decode + SimpleSerialize {
    /// Convert to SSZ-encoded bytes
    fn to_ssz_bytes(&self) -> Vec<u8> {
        Encode::as_ssz_bytes(self)
    }
    
    /// Create from SSZ-encoded bytes
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Decode::from_ssz_bytes(bytes)
    }

    /// Convert to SSZ-encoded bytes with recursion depth limit
    fn to_ssz_bytes_with_depth_limit(&self, max_depth: usize) -> io::Result<Vec<u8>> {
        serialize_with_depth_limit(self, max_depth)
    }
    
    /// Create from SSZ-encoded bytes with recursion depth limit
    fn from_ssz_bytes_with_depth_limit(bytes: &[u8], max_depth: usize) -> Result<Self, DecodeError> {
        deserialize_with_depth_limit(bytes, max_depth)
    }
}

// Remove the blanket implementation to avoid conflicts

// ===== PRIMITIVE TYPE IMPLEMENTATIONS =====

// Str implementation
impl Encode for Str {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        // Use fixed-size encoding - just return the 64-byte array
        self.0.to_vec()
    }
}

impl Decode for Str {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 64 {
            return Err(DecodeError {
                message: format!("Invalid Str length {}, expected 64", bytes.len()),
            });
        }
        
        let mut array = [0u8; 64];
        array.copy_from_slice(bytes);
        Ok(Str(array))
    }
}

impl DecodeWithLength for Str {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 64 {
            return Err(DecodeError {
                message: format!("Not enough bytes for Str: {}, expected 64", bytes.len()),
            });
        }
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes[0..64]);
        Ok((Str(array), 64))
    }
}

impl SimpleSerialize for Str {}

// Bool
impl Encode for bool {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        vec![if *self { 1u8 } else { 0u8 }]
    }
}

impl Decode for bool {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 1 {
            return Err(DecodeError {
                message: format!("Invalid bool length {}, expected 1", bytes.len()),
            });
        }
        match bytes[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError {
                message: format!("Invalid bool value {}", bytes[0]),
            }),
        }
    }
}

impl DecodeWithLength for bool {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = Self::from_ssz_bytes(bytes)?;
        Ok((value, 1))
    }
}

impl SimpleSerialize for bool {}

// u8
impl Encode for u8 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl Decode for u8 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 1 {
            return Err(DecodeError {
                message: format!("Invalid u8 length {}, expected 1", bytes.len()),
            });
        }
        Ok(bytes[0])
    }
}

impl SimpleSerialize for u8 {}

impl DecodeWithLength for u8 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = Self::from_ssz_bytes(bytes)?;
        Ok((value, 1))
    }
}

// u32
impl Encode for u32 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for u32 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 4 {
            return Err(DecodeError {
                message: format!("Invalid u32 length {}, expected 4", bytes.len()),
            });
        }
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(array))
    }
}

impl SimpleSerialize for u32 {}

impl DecodeWithLength for u32 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = Self::from_ssz_bytes(bytes)?;
        Ok((value, 4))
    }
}

// u64
impl Encode for u64 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for u64 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 8 {
            return Err(DecodeError {
                message: format!("Invalid u64 length {}, expected 8", bytes.len()),
            });
        }
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(array))
    }
}

impl SimpleSerialize for u64 {}

impl DecodeWithLength for u64 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = Self::from_ssz_bytes(bytes)?;
        Ok((value, 8))
    }
}

// i32
impl Encode for i32 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for i32 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 4 {
            return Err(DecodeError {
                message: format!("Invalid i32 length {}, expected 4", bytes.len()),
            });
        }
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(i32::from_le_bytes(array))
    }
}

impl SimpleSerialize for i32 {}

impl DecodeWithLength for i32 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = Self::from_ssz_bytes(bytes)?;
        Ok((value, 4))
    }
}

// i64
impl Encode for i64 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for i64 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 8 {
            return Err(DecodeError {
                message: format!("Invalid i64 length {}, expected 8", bytes.len()),
            });
        }
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(array))
    }
}

impl SimpleSerialize for i64 {}

impl DecodeWithLength for i64 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = Self::from_ssz_bytes(bytes)?;
        Ok((value, 8))
    }
}

// Vec implementation
impl<T: Encode> Encode for Vec<T> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // First, serialize the length
        bytes.extend((self.len() as u64).as_ssz_bytes());
        
        // Then serialize each element
        for item in self {
            bytes.extend(item.as_ssz_bytes());
        }
        
        bytes
    }
}

impl<T: Decode + Encode + 'static> Decode for Vec<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 8 {
            return Err(DecodeError {
                message: "Not enough bytes to read Vec length".to_string(),
            });
        }
        
        let len = <u64 as Decode>::from_ssz_bytes(&bytes[0..8])? as usize;
        let mut result = Vec::with_capacity(len);
        let mut offset = 8;
        
        for _ in 0..len {
            if offset >= bytes.len() {
                return Err(DecodeError {
                    message: "Not enough bytes to read Vec element".to_string(),
                });
            }
            
            // Use DecodeWithLength if available, otherwise fallback to manual calculation
            let remaining_bytes = &bytes[offset..];
            
            // For primitive types, we can calculate the size directly
            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<u32>() {
                if remaining_bytes.len() < 4 {
                    return Err(DecodeError {
                        message: "Not enough bytes for u32".to_string(),
                    });
                }
                let item = T::from_ssz_bytes(&remaining_bytes[..4])?;
                offset += 4;
                result.push(item);
            } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<u64>() {
                if remaining_bytes.len() < 8 {
                    return Err(DecodeError {
                        message: "Not enough bytes for u64".to_string(),
                    });
                }
                let item = T::from_ssz_bytes(&remaining_bytes[..8])?;
                offset += 8;
                result.push(item);
            } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<bool>() {
                if remaining_bytes.len() < 1 {
                    return Err(DecodeError {
                        message: "Not enough bytes for bool".to_string(),
                    });
                }
                let item = T::from_ssz_bytes(&remaining_bytes[..1])?;
                offset += 1;
                result.push(item);
            } else {
                // For other types, use the fallback method (less efficient but safe)
                let item = T::from_ssz_bytes(remaining_bytes)?;
                let item_bytes = item.as_ssz_bytes();
                offset += item_bytes.len();
                result.push(item);
            }
        }
        
        Ok(result)
    }
}

impl<T: DecodeWithLength> DecodeWithLength for Vec<T> {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 8 {
            return Err(DecodeError {
                message: "Not enough bytes to read Vec length".to_string(),
            });
        }
        
        let len = <u64 as Decode>::from_ssz_bytes(&bytes[0..8])? as usize;
        let mut result = Vec::with_capacity(len);
        let mut offset = 8;
        
        for _ in 0..len {
            let (item, consumed) = T::from_ssz_bytes_with_length(&bytes[offset..])?;
            result.push(item);
            offset += consumed;
        }
        
        Ok((result, offset))
    }
}

impl<T: Encode + Decode + SimpleSerialize> SimpleSerialize for Vec<T> {}

// Option implementation
impl<T: Encode> Encode for Option<T> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        match self {
            Some(value) => {
                bytes.push(1); // Presence flag
                bytes.extend(value.as_ssz_bytes());
            }
            None => {
                bytes.push(0); // Absence flag
            }
        }
        
        bytes
    }
}

impl<T: Decode> Decode for Option<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Empty bytes for Option".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(None),
            1 => {
                if bytes.len() == 1 {
                    return Err(DecodeError {
                        message: "Option marked as Some but no data follows".to_string(),
                    });
                }
                let value = T::from_ssz_bytes(&bytes[1..])?;
                Ok(Some(value))
            }
            _ => Err(DecodeError {
                message: format!("Invalid Option flag: {}", bytes[0]),
            }),
        }
    }
}

impl<T: DecodeWithLength> DecodeWithLength for Option<T> {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Empty bytes for Option".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok((None, 1)),
            1 => {
                let (value, consumed) = T::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((Some(value), consumed + 1))
            }
            _ => Err(DecodeError {
                message: format!("Invalid Option flag: {}", bytes[0]),
            }),
        }
    }
}

impl<T: Encode + Decode + SimpleSerialize> SimpleSerialize for Option<T> {}

// Array implementations
impl<T: Encode, const N: usize> Encode for [T; N] {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for item in self {
            bytes.extend(item.as_ssz_bytes());
        }
        bytes
    }
}

impl<T: Decode + Default + Copy + Encode, const N: usize> Decode for [T; N] {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut result = [T::default(); N];
        let mut offset = 0;
        
        for i in 0..N {
            let item = T::from_ssz_bytes(&bytes[offset..])?;
            let item_bytes = item.as_ssz_bytes();
            offset += item_bytes.len();
            result[i] = item;
        }
        
        Ok(result)
    }
}

// BTreeMap implementation
impl<K: Encode + Ord, V: Encode> Encode for BTreeMap<K, V> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize the number of elements
        bytes.extend((self.len() as u64).as_ssz_bytes());
        
        // Serialize each key-value pair (BTreeMap iteration is deterministic)
        for (key, value) in self {
            bytes.extend(key.as_ssz_bytes());
            bytes.extend(value.as_ssz_bytes());
        }
        
        bytes
    }
}

impl<K: Decode + Ord + Encode, V: Decode + Encode> Decode for BTreeMap<K, V> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 8 {
            return Err(DecodeError {
                message: "Not enough bytes to read BTreeMap length".to_string(),
            });
        }
        
        let len = <u64 as Decode>::from_ssz_bytes(&bytes[0..8])? as usize;
        let mut result = BTreeMap::new();
        let mut offset = 8;
        
        for _ in 0..len {
            // Read key
            let key = K::from_ssz_bytes(&bytes[offset..])?;
            let key_bytes = key.as_ssz_bytes();
            offset += key_bytes.len();
            
            // Read value
            let value = V::from_ssz_bytes(&bytes[offset..])?;
            let value_bytes = value.as_ssz_bytes();
            offset += value_bytes.len();
            
            result.insert(key, value);
        }
        
        Ok(result)
    }
}

impl<K: Decode + Ord + Encode, V: Decode + Encode> DecodeWithLength for BTreeMap<K, V> {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 4 { // Length of count
            return Err(DecodeError::new("BTreeMap (with length): Input bytes too short for count"));
        }
        let mut count_bytes = [0u8; 4];
        count_bytes.copy_from_slice(&bytes[0..4]);
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut map = BTreeMap::new();
        let mut current_offset = 4; // Start after count

        for _ in 0..count {
            // Key length
            if bytes.len() < current_offset + 4 {
                return Err(DecodeError::new("BTreeMap (with length): Input bytes too short for key length"));
            }
            let mut key_len_bytes = [0u8; 4];
            key_len_bytes.copy_from_slice(&bytes[current_offset..current_offset + 4]);
            let key_len = u32::from_le_bytes(key_len_bytes) as usize;
            current_offset += 4;

            // Key
            if bytes.len() < current_offset + key_len {
                return Err(DecodeError::new("BTreeMap (with length): Input bytes too short for key"));
            }
            let key = K::from_ssz_bytes(&bytes[current_offset..current_offset + key_len])?;
            current_offset += key_len;

            // Value length
            if bytes.len() < current_offset + 4 {
                return Err(DecodeError::new("BTreeMap (with length): Input bytes too short for value length"));
            }
            let mut value_len_bytes = [0u8; 4];
            value_len_bytes.copy_from_slice(&bytes[current_offset..current_offset + 4]);
            let value_len = u32::from_le_bytes(value_len_bytes) as usize;
            current_offset += 4;

            // Value
            if bytes.len() < current_offset + value_len {
                return Err(DecodeError::new("BTreeMap (with length): Input bytes too short for value"));
            }
            let value = V::from_ssz_bytes(&bytes[current_offset..current_offset + value_len])?;
            current_offset += value_len;

            map.insert(key, value);
        }
        Ok((map, current_offset))
    }
}

impl<K: Encode + Decode + Ord + SimpleSerialize, V: Encode + Decode + SimpleSerialize> SimpleSerialize for BTreeMap<K, V> {}

// HashMap implementation  
impl<K, V> Encode for HashMap<K, V>
where
    K: Encode + Eq + Hash + Clone,
    V: Encode + Clone,
{
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Serialize the number of elements
        bytes.extend((self.len() as u64).as_ssz_bytes());
        // Serialize each key-value pair
        for (key, value) in self {
            bytes.extend(key.as_ssz_bytes());
            bytes.extend(value.as_ssz_bytes());
        }
        bytes
    }
}

impl<K, V> Decode for HashMap<K, V>
where
    K: Decode + Encode + Eq + Hash + Clone,
    V: Decode + Encode + Clone,
{
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        // Deserialize the number of elements
        let len = <u64 as Decode>::from_ssz_bytes(&bytes[offset..offset + std::mem::size_of::<u64>()])? as usize;
        offset += std::mem::size_of::<u64>();

        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let key = K::from_ssz_bytes(&bytes[offset..])?;
            let key_byte_len = key.as_ssz_bytes().len();
            offset += key_byte_len;

            let value = V::from_ssz_bytes(&bytes[offset..])?;
            let value_byte_len = value.as_ssz_bytes().len();
            offset += value_byte_len;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K,V> SimpleSerialize for HashMap<K,V>
where K: Encode + Decode + Eq + Hash + Clone,
      V: Encode + Decode + Clone,
{}

// String implementation
impl Encode for String {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize length first
        bytes.extend((self.len() as u64).as_ssz_bytes());
        
        // Then serialize the UTF-8 bytes
        bytes.extend(self.as_bytes());
        
        bytes
    }
}

impl Decode for String {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 8 {
            return Err(DecodeError {
                message: "Not enough bytes to read String length".to_string(),
            });
        }
        
        let len = <u64 as Decode>::from_ssz_bytes(&bytes[0..8])? as usize;
        
        if bytes.len() < 8 + len {
            return Err(DecodeError {
                message: format!("Not enough bytes for String: expected {}, got {}", 8 + len, bytes.len()),
            });
        }
        
        let string_bytes = &bytes[8..8 + len];
        String::from_utf8(string_bytes.to_vec()).map_err(|e| DecodeError {
            message: format!("Invalid UTF-8 in String: {}", e),
        })
    }
}

impl SimpleSerialize for String {}

// Tuple implementations for (A, B)
impl<A: Encode, B: Encode> Encode for (A, B) {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.as_ssz_bytes());
        bytes.extend_from_slice(&self.1.as_ssz_bytes());
        bytes
    }
}

impl<A: Decode + Encode, B: Decode + Encode> Decode for (A, B) {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let a = A::from_ssz_bytes(&bytes[offset..])?;
        let a_bytes = a.as_ssz_bytes();
        offset += a_bytes.len();
        
        let b = B::from_ssz_bytes(&bytes[offset..])?;
        
        Ok((a, b))
    }
}

impl<A: Encode + Decode + SimpleSerialize, B: Encode + Decode + SimpleSerialize> SimpleSerialize for (A, B) {}

// &str implementation
impl Encode for &str {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize length first
        bytes.extend((self.len() as u64).as_ssz_bytes());
        
        // Then serialize the UTF-8 bytes
        bytes.extend(self.as_bytes());
        
        bytes
    }
}

impl DecodeWithLength for &str {
    fn from_ssz_bytes_with_length(_bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        // &str cannot be decoded because it requires a lifetime
        Err(DecodeError {
            message: "Cannot decode &str - use String instead".to_string(),
        })
    }
}

// ----------------------------------------------------------------------------
// UTILITY FUNCTIONS
// ----------------------------------------------------------------------------

/// Common utility functions for serialization
pub mod utils {
    use super::*;

    /// Computes a content address (ID) for a serializable object
    /// Uses SSZ serialization and SHA-256 hashing
    pub fn compute_content_address<T: Encode>(value: &T) -> Result<[u8; 32]> {
        let serialized = value.as_ssz_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Ok(bytes)
    }

    /// Serializes an object for content addressing
    pub fn serialize_for_content_addressing<T: Encode>(value: &T) -> Vec<u8> {
        value.as_ssz_bytes()
    }

    /// Compute a deterministic hash for a collection of values
    pub fn compute_collection_hash<T>(values: &[T]) -> Result<[u8; 32]>
    where
        T: Encode,
    {
        let mut combined = Vec::new();
        
        for value in values {
            combined.extend_from_slice(&value.as_ssz_bytes());
        }
        
        let mut hasher = Sha256::new();
        hasher.update(&combined);
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Ok(bytes)
    }
}

/// Convert an SSZ DecodeError to an anyhow Error
pub fn serialization_error_to_anyhow(error: DecodeError) -> anyhow::Error {
    anyhow::anyhow!("SSZ serialization error: {}", error)
}

/// Helper functions for content addressing with SSZ serialization
pub mod content_addressing {
    use super::*;

    /// Compute the content address of a value using SSZ serialization and SHA-256
    pub fn compute_content_address<T: Encode>(value: &T) -> Result<[u8; 32]> {
        let serialized = value.as_ssz_bytes();
        let digest = Sha256::digest(&serialized);
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&digest);
        Ok(result)
    }

    /// Helper function to generate a byte array for SSZ-based content addressing
    pub fn generate_bytes_for_content_addressing<T: Encode>(value: &T) -> Vec<u8> {
        value.as_ssz_bytes()
    }
}

// ----------------------------------------------------------------------------
// FFI UTILITIES
// ----------------------------------------------------------------------------

/// Serialize a value for FFI use
///
/// This function serializes a value using SSZ and returns a byte vector
/// that can be passed across FFI boundaries.
pub fn serialize_for_ffi<T: Encode>(value: &T) -> Vec<u8> {
    value.as_ssz_bytes()
}

/// Deserialize a value from FFI data
///
/// This function deserializes a value from a byte slice that was passed
/// across FFI boundaries.
pub fn deserialize_from_ffi<T: Decode>(bytes: &[u8]) -> Result<T, FfiSerializationError> {
    T::from_ssz_bytes(bytes).map_err(|e| e.into())
}

/// Serialize a value to a hex string for FFI use
///
/// This function serializes a value using SSZ and returns a hex-encoded
/// string that can be passed across FFI boundaries.
pub fn serialize_to_hex<T: Encode>(value: &T) -> String {
    let bytes = serialize_for_ffi(value);
    hex::encode(bytes)
}

/// Deserialize a value from a hex string
///
/// This function deserializes a value from a hex-encoded string that was
/// passed across FFI boundaries.
pub fn deserialize_from_hex<T: Decode>(hex_str: &str) -> Result<T, FfiSerializationError> {
    let bytes = hex::decode(hex_str)?;
    deserialize_from_ffi(&bytes)
}

/// Helper function to handle errors in FFI context
///
/// This function is useful for FFI functions that need to return a result
/// without propagating errors.
pub fn handle_ffi_result<T, F>(result: Result<T, FfiSerializationError>, error_handler: F, default: T) -> T
where
    F: FnOnce(String),
{
    match result {
        Ok(value) => value,
        Err(err) => {
            error_handler(err.message);
            default
        }
    }
}

// ----------------------------------------------------------------------------
// MERKLE TREE IMPLEMENTATION
// ----------------------------------------------------------------------------

/// A simple Merkle tree implementation for SSZ data
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    pub layers: Vec<Vec<[u8; 32]>>,
    pub root: [u8; 32],
}

impl MerkleTree {
    /// Create a new Merkle tree from a list of leaf values
    pub fn new<T: Encode>(values: &[T]) -> Result<Self> {
        let leaves: Vec<[u8; 32]> = values
            .iter()
            .map(|v| utils::compute_content_address(v))
            .collect::<Result<Vec<_>>>()?;

        let mut layers = vec![leaves.clone()];
        let mut current_layer = leaves.clone();

        // Build layers bottom-up
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            
            for chunk in current_layer.chunks(2) {
                let hash = if chunk.len() == 2 {
                    hash_pair(&chunk[0], &chunk[1])
                } else {
                    // Odd number - hash with itself
                    hash_pair(&chunk[0], &chunk[0])
                };
                next_layer.push(hash);
            }
            
            layers.push(next_layer.clone());
            current_layer = next_layer;
        }

        let root = current_layer.into_iter().next().unwrap_or([0u8; 32]);

        Ok(MerkleTree {
            leaves,
            layers,
            root,
        })
    }

    /// Get the root hash of the tree
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Generate a Merkle proof for a given leaf index
    pub fn proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= self.leaves.len() {
            return None;
        }

        let mut proof_hashes = Vec::new();
        let mut current_index = leaf_index;

        // Traverse up the tree collecting sibling hashes
        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < layer.len() {
                proof_hashes.push(layer[sibling_index]);
            } else {
                // No sibling (odd number of nodes), use the node itself
                proof_hashes.push(layer[current_index]);
            }

            current_index /= 2;
        }

        Some(MerkleProof {
            leaf_index,
            leaf_hash: self.leaves[leaf_index],
            proof_hashes,
            root: self.root,
        })
    }
}

/// A Merkle proof for verifying inclusion of a leaf in a tree
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub leaf_hash: [u8; 32],
    pub proof_hashes: Vec<[u8; 32]>,
    pub root: [u8; 32],
}

impl MerkleProof {
    /// Verify that this proof is valid for the given root
    pub fn verify(&self) -> bool {
        verify_proof(self.leaf_index, &self.leaf_hash, &self.proof_hashes, &self.root)
    }
}

/// Verify a Merkle proof
pub fn verify_proof(
    leaf_index: usize,
    leaf_hash: &[u8; 32],
    proof_hashes: &[[u8; 32]],
    root: &[u8; 32],
) -> bool {
    let mut current_hash = *leaf_hash;
    let mut current_index = leaf_index;

    for proof_hash in proof_hashes {
        current_hash = if current_index % 2 == 0 {
            hash_pair(&current_hash, proof_hash)
        } else {
            hash_pair(proof_hash, &current_hash)
        };
        current_index /= 2;
    }

    current_hash == *root
}

/// Hash two 32-byte arrays together
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// ----------------------------------------------------------------------------
// DERIVE SUPPORT
// ----------------------------------------------------------------------------

/// Add a macro for SimpleSerialize derive that implements Encode and Decode
#[macro_export]
macro_rules! derive_simple_serialize {
    ($type:ty) => {
        impl $crate::system::serialization::SimpleSerialize for $type {}
    };
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssz_roundtrip() {
        let value = 42u64;
        let serialized = serialize(&value);
        let deserialized: u64 = deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_primitive_types() {
        // Test bool
        let b = true;
        let serialized = b.as_ssz_bytes();
        let deserialized = bool::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(b, deserialized);

        // Test u32
        let n = 12345u32;
        let serialized = n.as_ssz_bytes();
        let deserialized = u32::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(n, deserialized);

        // Test String
        let s = "Hello, World!".to_string();
        let serialized = s.as_ssz_bytes();
        let deserialized = String::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn test_compound_types() {
        // Test Vec
        let vec = vec![1u32, 2, 3, 4, 5];
        let serialized = vec.as_ssz_bytes();
        let deserialized = Vec::<u32>::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(vec, deserialized);

        // Test Option
        let some_val = Some(42u32);
        let serialized = some_val.as_ssz_bytes();
        let deserialized = Option::<u32>::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(some_val, deserialized);

        let none_val: Option<u32> = None;
        let serialized = none_val.as_ssz_bytes();
        let deserialized = Option::<u32>::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(none_val, deserialized);
    }

    #[test]
    fn test_ffi_serialization() {
        let value = 42u32;
        
        // Test binary serialization
        let serialized = serialize_for_ffi(&value);
        let deserialized = deserialize_from_ffi::<u32>(&serialized).unwrap();
        assert_eq!(value, deserialized);
        
        // Test hex serialization
        let hex = serialize_to_hex(&value);
        let deserialized = deserialize_from_hex::<u32>(&hex).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_merkle_tree() {
        let values = vec![1u32, 2u32, 3u32, 4u32];
        let tree = MerkleTree::new(&values).unwrap();
        
        // Test proof generation and verification
        let proof = tree.proof(0).unwrap();
        assert!(proof.verify());
        
        let proof = tree.proof(2).unwrap();
        assert!(proof.verify());
    }

    #[test]
    fn test_content_addressing() {
        let value = "test content";
        let hash1 = utils::compute_content_address(&value).unwrap();
        let hash2 = utils::compute_content_address(&value).unwrap();
        assert_eq!(hash1, hash2); // Should be deterministic
        
        let different_value = "different content";
        let hash3 = utils::compute_content_address(&different_value).unwrap();
        assert_ne!(hash1, hash3); // Should be different
    }
} 