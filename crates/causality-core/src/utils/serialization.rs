// Purpose: Serialization utilities for SSZ encoding/decoding and SMT-based collections.

use crate::smt::{MemoryBackend, TegMultiDomainSmt};
use causality_types::{
    core::id::DomainId,
    serialization::{Decode, Encode},
};
use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind},
    sync::Arc,
};

//-----------------------------------------------------------------------------
// Basic SSZ Serialization Utilities
//-----------------------------------------------------------------------------

/// Serialize a value to a vector of bytes using SSZ
pub fn to_vec<T: Encode>(value: &T) -> Result<Vec<u8>, std::io::Error> {
    Ok(value.as_ssz_bytes())
}

/// Deserialize a value from a slice of bytes using SSZ
pub fn from_slice<T: Decode>(bytes: &[u8]) -> Result<T, std::io::Error> {
    T::from_ssz_bytes(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))
}

//-----------------------------------------------------------------------------
// Collection Serialization Utilities
//-----------------------------------------------------------------------------

/// Helper to serialize a vector of values
pub fn serialize_vector<T: Encode>(vec: &[T]) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(vec.len() as u32).to_le_bytes());
    for item in vec {
        let item_bytes = item.as_ssz_bytes();
        bytes.extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&item_bytes);
    }
    Ok(bytes)
}

/// Helper to deserialize a vector of values
pub fn deserialize_vector<T: Decode + std::fmt::Debug>(
    bytes: &[u8],
) -> Result<Vec<T>, std::io::Error> {
    let mut offset = 0;

    if offset + 4 > bytes.len() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Truncated vector length",
        ));
    }

    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&bytes[offset..offset + 4]);
    let count = u32::from_le_bytes(len_bytes) as usize;
    offset += 4;

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if offset + 4 > bytes.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Truncated item length"));
        }

        len_bytes.copy_from_slice(&bytes[offset..offset + 4]);
        let item_len = u32::from_le_bytes(len_bytes) as usize;
        offset += 4;

        if offset + item_len > bytes.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Truncated item data"));
        }

        let item = T::from_ssz_bytes(&bytes[offset..offset + item_len])
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.message))?;
        result.push(item);
        offset += item_len;
    }

    Ok(result)
}

/// Helper to serialize a BTreeMap of key-value pairs
/// Note: SSZ doesn't directly support maps, so we serialize as a vector of key-value pairs
pub fn serialize_map<K: Encode + Ord + Clone, V: Encode + Clone>(
    map: &BTreeMap<K, V>,
) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(map.len() as u32).to_le_bytes());

    for (k, v) in map.iter() {
        let k_bytes = k.as_ssz_bytes();
        let v_bytes = v.as_ssz_bytes();

        bytes.extend_from_slice(&(k_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&k_bytes);
        bytes.extend_from_slice(&(v_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&v_bytes);
    }

    Ok(bytes)
}

/// Helper to deserialize a BTreeMap of key-value pairs
/// Note: SSZ doesn't directly support maps, so we deserialize from a vector of key-value pairs
pub fn deserialize_map<K: Decode + Ord + Clone, V: Decode + Clone>(
    bytes: &[u8],
) -> Result<BTreeMap<K, V>, std::io::Error> {
    let mut offset = 0;

    if offset + 4 > bytes.len() {
        return Err(Error::new(ErrorKind::InvalidData, "Truncated map length"));
    }

    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&bytes[offset..offset + 4]);
    let count = u32::from_le_bytes(len_bytes) as usize;
    offset += 4;

    let mut map = BTreeMap::new();
    for _ in 0..count {
        // Deserialize key
        if offset + 4 > bytes.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Truncated key length"));
        }
        len_bytes.copy_from_slice(&bytes[offset..offset + 4]);
        let k_len = u32::from_le_bytes(len_bytes) as usize;
        offset += 4;

        if offset + k_len > bytes.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Truncated key data"));
        }
        let key = K::from_ssz_bytes(&bytes[offset..offset + k_len])
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.message))?;
        offset += k_len;

        // Deserialize value
        if offset + 4 > bytes.len() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Truncated value length",
            ));
        }
        len_bytes.copy_from_slice(&bytes[offset..offset + 4]);
        let v_len = u32::from_le_bytes(len_bytes) as usize;
        offset += 4;

        if offset + v_len > bytes.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Truncated value data"));
        }
        let value = V::from_ssz_bytes(&bytes[offset..offset + v_len])
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.message))?;
        offset += v_len;

        map.insert(key, value);
    }

    Ok(map)
}

//-----------------------------------------------------------------------------
// SMT-Based Serialization Utilities
//-----------------------------------------------------------------------------

/// SMT-based collection for type-safe storage
#[derive(Debug)]
pub struct SmtCollection<K, V>
where
    K: Encode + Decode + Clone + std::fmt::Display,
    V: Encode + Decode + Clone,
{
    smt: Arc<parking_lot::Mutex<TegMultiDomainSmt<MemoryBackend>>>,
    domain_id: DomainId,
    collection_prefix: String,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> SmtCollection<K, V>
where
    K: Encode + Decode + Clone + std::fmt::Display,
    V: Encode + Decode + Clone,
{
    /// Create a new SMT-based collection
    pub fn new(domain_id: DomainId, collection_prefix: impl Into<String>) -> Self {
        let backend = MemoryBackend::new();
        let smt = TegMultiDomainSmt::new(backend);

        Self {
            smt: Arc::new(parking_lot::Mutex::new(smt)),
            domain_id,
            collection_prefix: collection_prefix.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new SMT-based collection with shared SMT instance
    pub fn with_smt(
        smt: Arc<parking_lot::Mutex<TegMultiDomainSmt<MemoryBackend>>>,
        domain_id: DomainId,
        collection_prefix: impl Into<String>,
    ) -> Self {
        Self {
            smt,
            domain_id,
            collection_prefix: collection_prefix.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Generate SMT key for a given collection key
    fn make_smt_key(&self, key: &K) -> String {
        format!(
            "{}-{}-{}",
            self.domain_id.namespace_prefix(),
            self.collection_prefix,
            key
        )
    }

    /// Store a key-value pair in the SMT
    pub fn insert(&self, key: K, value: V) -> Result<(), std::io::Error> {
        let smt_key = self.make_smt_key(&key);
        let value_bytes = value.as_ssz_bytes();
        let mut smt = self.smt.lock();
        smt.store_data(&smt_key, &value_bytes)
            .map_err(Error::other)
    }

    /// Retrieve a value by key from the SMT
    pub fn get(&self, key: &K) -> Result<Option<V>, std::io::Error> {
        let smt_key = self.make_smt_key(key);
        let smt = self.smt.lock();
        match smt.get_data(&smt_key) {
            Ok(Some(bytes)) => {
                let value = V::from_ssz_bytes(&bytes)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e.message))?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Error::other(e)),
        }
    }

    /// Remove a key-value pair from the SMT
    pub fn remove(&self, key: &K) -> Result<Option<V>, std::io::Error> {
        let old_value = self.get(key)?;
        if old_value.is_some() {
            let smt_key = self.make_smt_key(key);
            let mut smt = self.smt.lock();
            smt.store_data(&smt_key, &[])
                .map_err(Error::other)?;
        }
        Ok(old_value)
    }

    /// Check if a key exists in the SMT
    pub fn contains_key(&self, key: &K) -> Result<bool, std::io::Error> {
        let smt_key = self.make_smt_key(key);
        let smt = self.smt.lock();
        Ok(smt.has_data(&smt_key))
    }

    /// Get all keys in the collection (note: this is not efficient for large collections)
    pub fn keys(&self) -> Result<Vec<K>, std::io::Error> {
        // This is a simplified implementation - in practice, you'd want a more efficient approach
        // that doesn't require scanning all keys
        unimplemented!("keys() requires implementing key scanning in SMT")
    }

    /// Convert the SMT collection to a BTreeMap
    pub fn to_btreemap(&self) -> Result<BTreeMap<K, V>, std::io::Error>
    where
        K: Ord,
    {
        unimplemented!("to_btreemap() requires implementing key scanning in SMT")
    }
}

/// Convert a BTreeMap to an SMT collection
pub fn btreemap_to_smt<K, V>(
    map: &BTreeMap<K, V>,
    domain_id: DomainId,
    collection_prefix: impl Into<String>,
) -> Result<SmtCollection<K, V>, std::io::Error>
where
    K: Encode + Decode + Clone + std::fmt::Display,
    V: Encode + Decode + Clone,
{
    let collection = SmtCollection::new(domain_id, collection_prefix);
    for (k, v) in map.iter() {
        collection.insert(k.clone(), v.clone())?;
    }
    Ok(collection)
}

/// Convert an SMT collection to a BTreeMap
pub fn smt_to_btreemap<K, V>(
    collection: &SmtCollection<K, V>,
) -> Result<BTreeMap<K, V>, std::io::Error>
where
    K: Encode + Decode + Clone + std::fmt::Display + Ord,
    V: Encode + Decode + Clone,
{
    collection.to_btreemap()
}

//-----------------------------------------------------------------------------
// Utility Functions for Common Patterns
//-----------------------------------------------------------------------------

/// Serialize any SSZ-encodable type to hex string
pub fn to_hex_string<T: Encode>(value: &T) -> String {
    let bytes = value.as_ssz_bytes();
    hex::encode(bytes)
}

/// Deserialize any SSZ-decodable type from hex string
pub fn from_hex_string<T: Decode>(hex_str: &str) -> Result<T, std::io::Error> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Invalid hex: {}", e)))?;
    from_slice(&bytes)
}

/// Compute the size in bytes of an SSZ-encodable value
pub fn size_of<T: Encode>(value: &T) -> usize {
    value.as_ssz_bytes().len()
} 