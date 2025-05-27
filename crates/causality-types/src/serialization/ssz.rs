//! SSZ serialization implementation
//!
//! This module implements a basic version of the SSZ serialization format for the Causality framework.

// Import required types
use std::collections::BTreeMap;
use std::io::{self};
use crate::primitive::string::Str;
use crate::serialization::{Decode, DecodeError, Encode, SimpleSerialize};

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

// Implement SszSerializable for common types
impl<T> SszSerializable for T where T: Encode + Decode + SimpleSerialize {}

// Custom Str implementation
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

impl SimpleSerialize for Str {}

// Implementations for primitive types
impl Encode for bool {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
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
                message: format!("Invalid bool value: {}", bytes[0]),
            }),
        }
    }
}

impl SimpleSerialize for bool {}

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

impl Encode for u16 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for u16 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 2 {
            return Err(DecodeError {
                message: format!("Invalid u16 length {}, expected 2", bytes.len()),
            });
        }
        let mut arr = [0u8; 2];
        arr.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(arr))
    }
}

impl SimpleSerialize for u16 {}

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
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(arr))
    }
}

impl SimpleSerialize for u32 {}

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
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(arr))
    }
}

impl SimpleSerialize for u64 {}

impl Encode for i8 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for i8 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 1 {
            return Err(DecodeError {
                message: format!("Invalid i8 length {}, expected 1", bytes.len()),
            });
        }
        let mut arr = [0u8; 1];
        arr.copy_from_slice(bytes);
        Ok(i8::from_le_bytes(arr))
    }
}

impl SimpleSerialize for i8 {}

impl Encode for i16 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for i16 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 2 {
            return Err(DecodeError {
                message: format!("Invalid i16 length {}, expected 2", bytes.len()),
            });
        }
        let mut arr = [0u8; 2];
        arr.copy_from_slice(bytes);
        Ok(i16::from_le_bytes(arr))
    }
}

impl SimpleSerialize for i16 {}

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
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Ok(i32::from_le_bytes(arr))
    }
}

impl SimpleSerialize for i32 {}

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
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(arr))
    }
}

impl SimpleSerialize for i64 {}

impl Encode for u128 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for u128 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 16 {
            return Err(DecodeError {
                message: format!("Invalid u128 length {}, expected 16", bytes.len()),
            });
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        Ok(u128::from_le_bytes(arr))
    }
}

impl SimpleSerialize for u128 {}

impl Encode for i128 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for i128 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 16 {
            return Err(DecodeError {
                message: format!("Invalid i128 length {}, expected 16", bytes.len()),
            });
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        Ok(i128::from_le_bytes(arr))
    }
}

impl SimpleSerialize for i128 {}

impl Encode for f32 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for f32 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 4 {
            return Err(DecodeError {
                message: format!("Invalid f32 length {}, expected 4", bytes.len()),
            });
        }
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Ok(f32::from_le_bytes(arr))
    }
}

impl SimpleSerialize for f32 {}

impl Encode for f64 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Decode for f64 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 8 {
            return Err(DecodeError {
                message: format!("Invalid f64 length {}, expected 8", bytes.len()),
            });
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(f64::from_le_bytes(arr))
    }
}

impl SimpleSerialize for f64 {}

// Vec implementation
impl<T: Encode> Encode for Vec<T> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Write length prefix
        let len = self.len() as u32;
        result.extend_from_slice(&len.to_le_bytes());
        
        // Write elements
        for item in self {
            let bytes = item.as_ssz_bytes();
            result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(&bytes);
        }
        
        result
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError {
                message: format!("Invalid Vec length {}, expected at least 4", bytes.len()),
            });
        }
        
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&bytes[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;
        
        let mut result = Vec::with_capacity(len);
        let mut offset = 4;
        
        // Read elements
        for _ in 0..len {
            if offset + 4 > bytes.len() {
                return Err(DecodeError {
                    message: "Invalid Vec: incomplete element size".to_string(),
                });
            }
            
            // Read element size
            let mut size_bytes = [0u8; 4];
            size_bytes.copy_from_slice(&bytes[offset..offset + 4]);
            let size = u32::from_le_bytes(size_bytes) as usize;
            offset += 4;
            
            if offset + size > bytes.len() {
                return Err(DecodeError {
                    message: "Invalid Vec: incomplete element data".to_string(),
                });
            }
            
            // Read element data
            let element = T::from_ssz_bytes(&bytes[offset..offset + size])?;
            result.push(element);
            offset += size;
        }
        
        Ok(result)
    }
}

impl<T: Encode + Decode + SimpleSerialize> SimpleSerialize for Vec<T> {}

// Option implementation
impl<T: Encode> Encode for Option<T> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            None => vec![0],
            Some(value) => {
                let value_bytes = value.as_ssz_bytes();
                let mut result = Vec::with_capacity(1 + value_bytes.len());
                result.push(1);
                result.extend_from_slice(&value_bytes);
                result
            }
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Invalid Option: empty bytes".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(None),
            1 => {
                if bytes.len() == 1 {
                    return Err(DecodeError {
                        message: "Invalid Option: missing value".to_string(),
                    });
                }
                let value = T::from_ssz_bytes(&bytes[1..])?;
                Ok(Some(value))
            }
            _ => Err(DecodeError {
                message: format!("Invalid Option tag: {}", bytes[0]),
            }),
        }
    }
}

impl<T: Encode + Decode + SimpleSerialize> SimpleSerialize for Option<T> {}

// Fixed-size array implementations (examples for common sizes)
impl<T: Encode, const N: usize> Encode for [T; N] {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for item in self {
            let item_bytes = item.as_ssz_bytes();
            result.extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(&item_bytes);
        }
        result
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut result = Vec::with_capacity(N);
        let mut offset = 0;
        
        for _ in 0..N {
            if offset + 4 > bytes.len() {
                return Err(DecodeError {
                    message: "Array deserialization error: not enough bytes for item size".to_string(),
                });
            }
            
            // Read item size
            let mut size_bytes = [0u8; 4];
            size_bytes.copy_from_slice(&bytes[offset..offset + 4]);
            let item_size = u32::from_le_bytes(size_bytes) as usize;
            offset += 4;
            
            if offset + item_size > bytes.len() {
                return Err(DecodeError {
                    message: "Array deserialization error: not enough bytes for item data".to_string(),
                });
            }
            
            // Read item data
            let item = T::from_ssz_bytes(&bytes[offset..offset + item_size])?;
            result.push(item);
            offset += item_size;
        }
        
        // Convert Vec<T> to [T; N]
        match result.try_into() {
            Ok(arr) => Ok(arr),
            Err(_) => Err(DecodeError {
                message: format!("Failed to convert vector to array of size {}", N),
            }),
        }
    }
}

// Implement SimpleSerialize for arrays
impl<T: Encode + Decode + SimpleSerialize, const N: usize> SimpleSerialize for [T; N] {}

// BTreeMap implementation for common structures
impl<K: Encode + Ord, V: Encode> Encode for BTreeMap<K, V> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Write length prefix
        let len = self.len() as u32;
        result.extend_from_slice(&len.to_le_bytes());
        
        // Write key-value pairs
        for (key, value) in self {
            let key_bytes = key.as_ssz_bytes();
            let value_bytes = value.as_ssz_bytes();
            
            // Write key size and key
            result.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(&key_bytes);
            
            // Write value size and value
            result.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(&value_bytes);
        }
        
        result
    }
}

impl<K: Decode + Ord, V: Decode> Decode for BTreeMap<K, V> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError {
                message: format!("Invalid BTreeMap length {}, expected at least 4", bytes.len()),
            });
        }
        
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&bytes[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;
        
        let mut result = BTreeMap::new();
        let mut offset = 4;
        
        // Read key-value pairs
        for _ in 0..len {
            if offset + 4 > bytes.len() {
                return Err(DecodeError {
                    message: "Invalid BTreeMap: incomplete key size".to_string(),
                });
            }
            
            // Read key size
            let mut key_size_bytes = [0u8; 4];
            key_size_bytes.copy_from_slice(&bytes[offset..offset + 4]);
            let key_size = u32::from_le_bytes(key_size_bytes) as usize;
            offset += 4;
            
            if offset + key_size > bytes.len() {
                return Err(DecodeError {
                    message: "Invalid BTreeMap: incomplete key data".to_string(),
                });
            }
            
            // Read key data
            let key = K::from_ssz_bytes(&bytes[offset..offset + key_size])?;
            offset += key_size;
            
            if offset + 4 > bytes.len() {
                return Err(DecodeError {
                    message: "Invalid BTreeMap: incomplete value size".to_string(),
                });
            }
            
            // Read value size
            let mut value_size_bytes = [0u8; 4];
            value_size_bytes.copy_from_slice(&bytes[offset..offset + 4]);
            let value_size = u32::from_le_bytes(value_size_bytes) as usize;
            offset += 4;
            
            if offset + value_size > bytes.len() {
                return Err(DecodeError {
                    message: "Invalid BTreeMap: incomplete value data".to_string(),
                });
            }
            
            // Read value data
            let value = V::from_ssz_bytes(&bytes[offset..offset + value_size])?;
            offset += value_size;
            
            // Insert key-value pair
            result.insert(key, value);
        }
        
        Ok(result)
    }
}

impl<K: Encode + Decode + Ord + SimpleSerialize, V: Encode + Decode + SimpleSerialize> SimpleSerialize for BTreeMap<K, V> {}

// Implementation for unit type
impl Encode for () {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        Vec::new() // Unit type is empty
    }
}

impl Decode for () {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if !bytes.is_empty() {
            return Err(DecodeError {
                message: format!("Invalid unit type length {}, expected 0", bytes.len()),
            });
        }
        Ok(())
    }
}

impl SimpleSerialize for () {}

// String implementation
impl Encode for String {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(4 + self.len());
        
        // Write length prefix (4 bytes, little-endian)
        let len = self.len() as u32;
        result.extend_from_slice(&len.to_le_bytes());
        
        // Write string data
        result.extend_from_slice(self.as_bytes());
        
        result
    }
}

impl Decode for String {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError {
                message: format!("Invalid string length {}, expected at least 4", bytes.len()),
            });
        }
        
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&bytes[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;
        
        if bytes.len() < 4 + len {
            return Err(DecodeError {
                message: format!("Invalid string data length, expected {} got {}", len, bytes.len() - 4),
            });
        }
        
        // Read string data
        let string_data = &bytes[4..4+len];
        match std::str::from_utf8(string_data) {
            Ok(s) => Ok(s.to_owned()),
            Err(e) => Err(DecodeError {
                message: format!("Invalid UTF-8 string: {}", e),
            }),
        }
    }
}

impl SimpleSerialize for String {}

// Implementations for Tuples (up to a certain arity, e.g., 2 for now)
impl<T1, T2> Encode for (T1, T2)
where
    T1: Encode,
    T2: Encode,
{
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.0.as_ssz_bytes());
        bytes.extend(self.1.as_ssz_bytes());
        bytes
    }
}

impl<T1, T2> Decode for (T1, T2)
where
    T1: Decode + Encode, // Encode needed to determine byte length if variable
    T2: Decode,
{
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        let t1 = T1::from_ssz_bytes(&bytes[offset..])?;
        // Determine how many bytes t1 took. This is a simplification and might need adjustment
        // for complex, variable-length types. Ideally, T1::from_ssz_bytes would return (Self, usize_bytes_read)
        // or T1::as_ssz_bytes().len() is used if T1 also implements Encode.
        let t1_len = t1.as_ssz_bytes().len(); // Requires T1: Encode
        offset += t1_len;

        let t2 = T2::from_ssz_bytes(&bytes[offset..])?;
        Ok((t1, t2))
    }
}

impl<T1, T2> SimpleSerialize for (T1, T2)
where
    T1: Encode + Decode + SimpleSerialize,
    T2: Encode + Decode + SimpleSerialize,
{}

/// Trait for SSZ decoding that returns both the decoded value and bytes consumed
pub trait DecodeWithLength: Sized {
    /// Decode from SSZ bytes and return the value and number of bytes consumed
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError>;
}

// Implement DecodeWithLength for basic types
impl DecodeWithLength for bool {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = <bool as Decode>::from_ssz_bytes(bytes)?;
        Ok((value, 1))
    }
}

impl DecodeWithLength for u8 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = <u8 as Decode>::from_ssz_bytes(bytes)?;
        Ok((value, 1))
    }
}

impl DecodeWithLength for u32 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = <u32 as Decode>::from_ssz_bytes(bytes)?;
        Ok((value, 4))
    }
}

impl DecodeWithLength for u64 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = <u64 as Decode>::from_ssz_bytes(bytes)?;
        Ok((value, 8))
    }
}

impl DecodeWithLength for i64 {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let value = <i64 as Decode>::from_ssz_bytes(bytes)?;
        Ok((value, 8))
    }
}

impl DecodeWithLength for Str {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 64 {
            return Err(DecodeError {
                message: format!("Invalid Str length {}, expected at least 64", bytes.len()),
            });
        }
        
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes[..64]);
        Ok((Str(array), 64))
    }
}

impl<T: DecodeWithLength> DecodeWithLength for Vec<T> {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 4 {
            return Err(DecodeError {
                message: format!("Invalid byte length {}, expected at least 4", bytes.len()),
            });
        }
        
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&bytes[0..4]);
        let count = u32::from_le_bytes(len_bytes) as usize;
        
        let mut result = Vec::with_capacity(count);
        let mut offset = 4;
        
        for _ in 0..count {
            let (item, consumed) = T::from_ssz_bytes_with_length(&bytes[offset..])?;
            result.push(item);
            offset += consumed;
        }
        
        Ok((result, offset))
    }
}

impl DecodeWithLength for [u8; 32] {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.len() < 32 {
            return Err(DecodeError {
                message: format!("Invalid byte length {}, expected at least 32", bytes.len()),
            });
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes[0..32]);
        Ok((array, 32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::string::Str;
    
    #[test]
    fn test_ssz_roundtrip() {
        let original = Str::from("test string");
        let serialized = serialize(&original);
        let deserialized = deserialize::<Str>(&serialized).unwrap();
        
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_recursive_serialization_depth_limit() {
        let map = BTreeMap::<Str, Str>::from([
            (Str::from("key1"), Str::from("value1")),
            (Str::from("key2"), Str::from("value2")),
        ]);

        // Should succeed with reasonable depth
        let result = serialize_with_depth_limit(&map, 10);
        assert!(result.is_ok());

        // Should fail with depth 0
        let result = serialize_with_depth_limit(&map, 0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_primitive_types() {
        // Test boolean
        let original_bool = true;
        let serialized = serialize(&original_bool);
        let deserialized = deserialize::<bool>(&serialized).unwrap();
        assert_eq!(original_bool, deserialized);
        
        // Test u8
        let original_u8 = 42u8;
        let serialized = serialize(&original_u8);
        let deserialized = deserialize::<u8>(&serialized).unwrap();
        assert_eq!(original_u8, deserialized);
        
        // Test u16
        let original_u16 = 12345u16;
        let serialized = serialize(&original_u16);
        let deserialized = deserialize::<u16>(&serialized).unwrap();
        assert_eq!(original_u16, deserialized);
        
        // Test u32
        let original_u32 = 123456789u32;
        let serialized = serialize(&original_u32);
        let deserialized = deserialize::<u32>(&serialized).unwrap();
        assert_eq!(original_u32, deserialized);
        
        // Test u64
        let original_u64 = 1234567890123456789u64;
        let serialized = serialize(&original_u64);
        let deserialized = deserialize::<u64>(&serialized).unwrap();
        assert_eq!(original_u64, deserialized);
        
        // Test i8
        let original_i8 = -42i8;
        let serialized = serialize(&original_i8);
        let deserialized = deserialize::<i8>(&serialized).unwrap();
        assert_eq!(original_i8, deserialized);
        
        // Test i16
        let original_i16 = -12345i16;
        let serialized = serialize(&original_i16);
        let deserialized = deserialize::<i16>(&serialized).unwrap();
        assert_eq!(original_i16, deserialized);
        
        // Test i32
        let original_i32 = -123456789i32;
        let serialized = serialize(&original_i32);
        let deserialized = deserialize::<i32>(&serialized).unwrap();
        assert_eq!(original_i32, deserialized);
        
        // Test i64
        let original_i64 = -1234567890123456789i64;
        let serialized = serialize(&original_i64);
        let deserialized = deserialize::<i64>(&serialized).unwrap();
        assert_eq!(original_i64, deserialized);
    }
    
    #[test]
    fn test_compound_types() {
        // Test Vec
        let original_vec = vec![1i32, 2, 3, 4, 5];
        let serialized = serialize(&original_vec);
        let deserialized = deserialize::<Vec<i32>>(&serialized).unwrap();
        assert_eq!(original_vec, deserialized);
        
        // Test Option (Some)
        let original_option = Some("test string".to_string());
        let serialized = serialize(&original_option);
        let deserialized = deserialize::<Option<String>>(&serialized).unwrap();
        assert_eq!(original_option, deserialized);
        
        // Test Option (None)
        let original_option: Option<String> = None;
        let serialized = serialize(&original_option);
        let deserialized = deserialize::<Option<String>>(&serialized).unwrap();
        assert_eq!(original_option, deserialized);
        
        // Test fixed-size array
        let original_array = [1u32, 2, 3, 4, 5];
        let serialized = serialize(&original_array);
        let deserialized = deserialize::<[u32; 5]>(&serialized).unwrap();
        assert_eq!(original_array, deserialized);
        
        // Test BTreeMap
        let mut original_map = BTreeMap::new();
        original_map.insert("key1".to_string(), 1i32);
        original_map.insert("key2".to_string(), 2);
        original_map.insert("key3".to_string(), 3);
        let serialized = serialize(&original_map);
        let deserialized = deserialize::<BTreeMap<String, i32>>(&serialized).unwrap();
        assert_eq!(original_map, deserialized);
    }
    
    #[test]
    fn test_nested_types() {
        // Test nested Option<Vec<String>>
        let original = Some(vec!["test1".to_string(), "test2".to_string()]);
        let serialized = serialize(&original);
        let deserialized = deserialize::<Option<Vec<String>>>(&serialized).unwrap();
        assert_eq!(original, deserialized);
        
        // Test Vec<Option<i32>>
        let original = vec![Some(1), None, Some(3)];
        let serialized = serialize(&original);
        let deserialized = deserialize::<Vec<Option<i32>>>(&serialized).unwrap();
        assert_eq!(original, deserialized);
        
        // Test BTreeMap with complex values
        let mut original_map = BTreeMap::new();
        original_map.insert("key1".to_string(), vec![1, 2, 3]);
        original_map.insert("key2".to_string(), vec![4, 5]);
        let serialized = serialize(&original_map);
        let deserialized = deserialize::<BTreeMap<String, Vec<i32>>>(&serialized).unwrap();
        assert_eq!(original_map, deserialized);
    }
    
    #[test]
    fn test_error_cases() {
        // Test deserializing with insufficient bytes
        let result = deserialize::<u32>(&[1, 2]);
        assert!(result.is_err());
        
        // Test deserializing with invalid data
        let result = deserialize::<bool>(&[2]);
        assert!(result.is_err());
        
        // Test Vec with invalid length prefix
        let result = deserialize::<Vec<u32>>(&[255, 255, 255, 255]);
        assert!(result.is_err());
    }
} 