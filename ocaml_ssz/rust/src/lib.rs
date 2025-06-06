//! Rust FFI bindings for  OCaml library
//! 
//! This module provides Rust implementations of SSZ serialization functions
//! that can be called from OCaml code, enabling interoperability testing.

use ocaml::{ToValue, FromValue, Value};

// Import ocaml macros
ocaml::export! {
    // Boolean serialization/deserialization
    fn rust_serialize_bool(value: bool) -> String {
        if value {
            String::from("\u{01}")
        } else {
            String::from("\u{00}")
        }
    }

    fn rust_deserialize_bool(data: String) -> bool {
        if data.is_empty() {
            false
        } else {
            data.as_bytes()[0] == 1
        }
    }

    // u32 serialization/deserialization
    fn rust_serialize_u32(value: u32) -> String {
        let mut result = String::with_capacity(4);
        result.push(((value & 0xFF) as u8) as char);
        result.push((((value >> 8) & 0xFF) as u8) as char);
        result.push((((value >> 16) & 0xFF) as u8) as char);
        result.push((((value >> 24) & 0xFF) as u8) as char);
        result
    }

    fn rust_deserialize_u32(data: String) -> u32 {
        if data.len() < 4 {
            return 0;
        }
        
        let bytes = data.as_bytes();
        let mut result: u32 = 0;
        
        result |= bytes[0] as u32;
        result |= (bytes[1] as u32) << 8;
        result |= (bytes[2] as u32) << 16;
        result |= (bytes[3] as u32) << 24;
        
        result
    }

    // String serialization/deserialization with length prefix
    fn rust_serialize_string(value: String) -> String {
        let len = value.len() as u32;
        let len_bytes = rust_serialize_u32(len);
        len_bytes + &value
    }

    fn rust_deserialize_string(data: String) -> String {
        if data.len() < 4 {
            return String::new();
        }
        
        let len_bytes = data[0..4].to_string();
        let len = rust_deserialize_u32(len_bytes) as usize;
        
        if data.len() < 4 + len {
            return String::new();
        }
        
        data[4..4+len].to_string()
    }

    // Simple hash function for hash tree root testing
    fn rust_simple_hash(data: String) -> String {
        let mut hash: u32 = 0;
        for byte in data.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        
        let mut result = String::with_capacity(32);
        for i in 0..8 {
            let value = (hash >> (i * 4)) & 0xF;
            for _ in 0..4 {
                result.push(value as u8 as char);
            }
        }
        
        // Pad to 32 bytes
        while result.len() < 32 {
            result.push('\0');
        }
        
        result
    }

    // Roundtrip test helper function
    fn rust_roundtrip_bool(value: bool) -> bool {
        let serialized = rust_serialize_bool(value);
        rust_deserialize_bool(serialized)
    }

    fn rust_roundtrip_u32(value: u32) -> u32 {
        let serialized = rust_serialize_u32(value);
        rust_deserialize_u32(serialized)
    }

    fn rust_roundtrip_string(value: String) -> String {
        let serialized = rust_serialize_string(value);
        rust_deserialize_string(serialized)
    }
} 