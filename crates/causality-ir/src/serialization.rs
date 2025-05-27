// Serialization module for the Temporal Effect Graph
// This module provides implementations for serializing and deserializing TEG structures.

use anyhow::Result;
use serde_json;
use causality_types::ContentAddressed;

use crate::{TemporalEffectGraph, EffectNode, ResourceNode, TEGFragment};

/// Serialize a TEG to JSON format
pub fn serialize_teg(teg: &TemporalEffectGraph) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(teg)?;
    Ok(json)
}

/// Deserialize a TEG from JSON format
pub fn deserialize_teg(bytes: &[u8]) -> Result<TemporalEffectGraph> {
    let teg = serde_json::from_slice(bytes)?;
    Ok(teg)
}

/// Serialize a TEG to JSON string format (pretty printed)
pub fn teg_to_json(teg: &TemporalEffectGraph) -> Result<String> {
    let json = serde_json::to_string_pretty(teg)?;
    Ok(json)
}

/// Deserialize a TEG from JSON string format
pub fn teg_from_json(json: &str) -> Result<TemporalEffectGraph> {
    let teg = serde_json::from_str(json)?;
    Ok(teg)
}

/// Serialize a TEG fragment to JSON format
pub fn serialize_fragment(fragment: &TEGFragment) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(fragment)?;
    Ok(json)
}

/// Deserialize a TEG fragment from JSON format
pub fn deserialize_fragment(bytes: &[u8]) -> Result<TEGFragment> {
    let fragment = serde_json::from_slice(bytes)?;
    Ok(fragment)
}

/// Serialize a TEG fragment to JSON string format (pretty printed)
pub fn fragment_to_json(fragment: &TEGFragment) -> Result<String> {
    let json = serde_json::to_string_pretty(fragment)?;
    Ok(json)
}

/// Deserialize a TEG fragment from JSON string format
pub fn fragment_from_json(json: &str) -> Result<TEGFragment> {
    let fragment = serde_json::from_str(json)?;
    Ok(fragment)
}

/// Serialize an effect node to JSON format
pub fn serialize_effect_node(effect: &EffectNode) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(effect)?;
    Ok(json)
}

/// Deserialize an effect node from JSON format
pub fn deserialize_effect_node(bytes: &[u8]) -> Result<EffectNode> {
    let effect = serde_json::from_slice(bytes)?;
    Ok(effect)
}

/// Serialize a resource node to JSON format
pub fn serialize_resource_node(resource: &ResourceNode) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(resource)?;
    Ok(json)
}

/// Deserialize a resource node from JSON format
pub fn deserialize_resource_node(bytes: &[u8]) -> Result<ResourceNode> {
    let resource = serde_json::from_slice(bytes)?;
    Ok(resource)
}

/// Verify that a serialized TEG matches its expected content hash
pub fn verify_teg_hash(teg: &TemporalEffectGraph) -> Result<bool> {
    // We need to compute the actual hash and compare with the stored hash
    let actual_hash = teg.content_hash()?;
    let expected_hash = teg.content_hash.to_hash_output()?;
    Ok(actual_hash == expected_hash)
}

/// Update the content hash in a TemporalEffectGraph
pub fn update_teg_hash(teg: &mut TemporalEffectGraph) -> Result<()> {
    let hash = teg.content_hash()?;
    teg.content_hash = causality_types::crypto_primitives::ContentHash::from_hash_output(&hash);
    Ok(())
}

/// Create a new TEG with properly computed content hash
pub fn create_teg_with_hash() -> Result<TemporalEffectGraph> {
    let mut teg = TemporalEffectGraph::new();
    update_teg_hash(&mut teg)?;
    Ok(teg)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_teg_serialization_roundtrip() {
        let teg = TemporalEffectGraph::new();
        let bytes = serialize_teg(&teg).expect("Failed to serialize TEG");
        let deserialized = deserialize_teg(&bytes).expect("Failed to deserialize TEG");
        
        // Basic check to make sure the structure is the same
        assert_eq!(teg.effect_nodes.len(), deserialized.effect_nodes.len());
        assert_eq!(teg.resource_nodes.len(), deserialized.resource_nodes.len());
        assert_eq!(teg.edges.len(), deserialized.edges.len());
    }
    
    #[test]
    fn test_fragment_serialization_roundtrip() {
        let fragment = TEGFragment::new();
        let bytes = serialize_fragment(&fragment).expect("Failed to serialize fragment");
        let deserialized = deserialize_fragment(&bytes).expect("Failed to deserialize fragment");
        
        // Basic check to make sure the structure is the same
        assert_eq!(fragment.effect_nodes.len(), deserialized.effect_nodes.len());
        assert_eq!(fragment.resource_nodes.len(), deserialized.resource_nodes.len());
    }
    
    #[test]
    fn test_teg_content_hash() {
        let mut teg = TemporalEffectGraph::new();
        update_teg_hash(&mut teg).expect("Failed to update TEG hash");
        
        // After updating the hash, verification should pass
        let result = verify_teg_hash(&teg).expect("Failed to verify TEG hash");
        assert!(result, "TEG hash verification failed");
        
        // Modify the TEG and the hash should no longer match
        teg.metadata.insert("test".to_string(), "value".to_string());
        let result = verify_teg_hash(&teg).expect("Failed to verify TEG hash");
        assert!(!result, "TEG hash verification unexpectedly passed after modification");
        
        // Update the hash and verification should pass again
        update_teg_hash(&mut teg).expect("Failed to update TEG hash");
        let result = verify_teg_hash(&teg).expect("Failed to verify TEG hash");
        assert!(result, "TEG hash verification failed after updating hash");
    }
}
