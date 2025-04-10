// Serialization module for the Temporal Effect Graph
// This file provides implementations for serializing and deserializing TEG structures.

use borsh::{BorshSerialize, BorshDeserialize};
use anyhow::Result;
use serde_json;

use crate::{TemporalEffectGraph, EffectNode, ResourceNode, TEGFragment};

/// Serialize a TEG to Borsh format
pub fn serialize_teg(teg: &TemporalEffectGraph) -> Result<Vec<u8>> {
    let bytes = teg.try_to_vec()?;
    Ok(bytes)
}

/// Deserialize a TEG from Borsh format
pub fn deserialize_teg(bytes: &[u8]) -> Result<TemporalEffectGraph> {
    let teg = TemporalEffectGraph::try_from_slice(bytes)?;
    Ok(teg)
}

/// Serialize a TEG to JSON format
pub fn teg_to_json(teg: &TemporalEffectGraph) -> Result<String> {
    let json = serde_json::to_string_pretty(teg)?;
    Ok(json)
}

/// Deserialize a TEG from JSON format
pub fn teg_from_json(json: &str) -> Result<TemporalEffectGraph> {
    let teg = serde_json::from_str(json)?;
    Ok(teg)
}

/// Serialize a TEG fragment to Borsh format
pub fn serialize_fragment(fragment: &TEGFragment) -> Result<Vec<u8>> {
    let bytes = fragment.try_to_vec()?;
    Ok(bytes)
}

/// Deserialize a TEG fragment from Borsh format
pub fn deserialize_fragment(bytes: &[u8]) -> Result<TEGFragment> {
    let fragment = TEGFragment::try_from_slice(bytes)?;
    Ok(fragment)
}

/// Serialize a TEG fragment to JSON format
pub fn fragment_to_json(fragment: &TEGFragment) -> Result<String> {
    let json = serde_json::to_string_pretty(fragment)?;
    Ok(json)
}

/// Deserialize a TEG fragment from JSON format
pub fn fragment_from_json(json: &str) -> Result<TEGFragment> {
    let fragment = serde_json::from_str(json)?;
    Ok(fragment)
}

/// Serialize an effect node to Borsh format
pub fn serialize_effect_node(effect: &EffectNode) -> Result<Vec<u8>> {
    let bytes = effect.try_to_vec()?;
    Ok(bytes)
}

/// Deserialize an effect node from Borsh format
pub fn deserialize_effect_node(bytes: &[u8]) -> Result<EffectNode> {
    let effect = EffectNode::try_from_slice(bytes)?;
    Ok(effect)
}

/// Serialize a resource node to Borsh format
pub fn serialize_resource_node(resource: &ResourceNode) -> Result<Vec<u8>> {
    let bytes = resource.try_to_vec()?;
    Ok(bytes)
}

/// Deserialize a resource node from Borsh format
pub fn deserialize_resource_node(bytes: &[u8]) -> Result<ResourceNode> {
    let resource = ResourceNode::try_from_slice(bytes)?;
    Ok(resource)
}

/// Verify that a serialized TEG matches its expected content hash
pub fn verify_teg_hash(teg: &TemporalEffectGraph) -> Result<bool> {
    let result = teg.verify()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectId;
    
    #[test]
    fn test_teg_serialization_roundtrip() {
        let teg = TemporalEffectGraph::new();
        let bytes = serialize_teg(&teg).expect("Failed to serialize TEG");
        let deserialized = deserialize_teg(&bytes).expect("Failed to deserialize TEG");
        
        // Basic check to make sure the structure is the same
        assert_eq!(teg.effect_nodes.len(), deserialized.effect_nodes.len());
        assert_eq!(teg.resource_nodes.len(), deserialized.resource_nodes.len());
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
}
