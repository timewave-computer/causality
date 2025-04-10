//! Serialization formats for the Temporal Effect Graph (TEG)
//!
//! This module provides serialization capabilities for converting TEGs to
//! various formats suitable for external consumption, including JSON and
//! incremental updates.

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{TemporalEffectGraph, EffectId, ResourceId};
use super::{SerializableTEG, SerializableEffect, SerializableResource};

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializationFormat {
    /// JSON format
    JSON,
    
    /// Binary format (using borsh)
    Binary,
    
    /// GraphML format (for visualization tools)
    GraphML,
    
    /// DOT format (for Graphviz)
    DOT,
}

/// Serialization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationOptions {
    /// Format to use
    pub format: SerializationFormat,
    
    /// Whether to include metadata
    pub include_metadata: bool,
    
    /// Whether to include parameters
    pub include_parameters: bool,
    
    /// Whether to include capabilities
    pub include_capabilities: bool,
    
    /// Whether to pretty-print (for text formats)
    pub pretty: bool,
}

impl Default for SerializationOptions {
    fn default() -> Self {
        Self {
            format: SerializationFormat::JSON,
            include_metadata: true,
            include_parameters: true,
            include_capabilities: true,
            pretty: false,
        }
    }
}

/// Incremental update for a TEG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TEGUpdate {
    /// Base version content hash (or None for initial state)
    pub base_version: Option<String>,
    
    /// New version content hash
    pub new_version: String,
    
    /// Added effect nodes
    pub added_effects: HashMap<EffectId, SerializableEffect>,
    
    /// Updated effect nodes
    pub updated_effects: HashMap<EffectId, SerializableEffect>,
    
    /// Removed effect IDs
    pub removed_effects: Vec<EffectId>,
    
    /// Added resource nodes
    pub added_resources: HashMap<ResourceId, SerializableResource>,
    
    /// Updated resource nodes
    pub updated_resources: HashMap<ResourceId, SerializableResource>,
    
    /// Removed resource IDs
    pub removed_resources: Vec<ResourceId>,
    
    /// Updated dependencies
    pub updated_dependencies: HashMap<EffectId, Vec<EffectId>>,
    
    /// Updated continuations
    pub updated_continuations: HashMap<EffectId, Vec<(EffectId, Option<String>)>>,
    
    /// Updated metadata
    pub updated_metadata: HashMap<String, String>,
}

/// Serializer for TEGs
pub struct TEGSerializer;

impl TEGSerializer {
    /// Serialize a TEG to JSON
    pub fn to_json(teg: &TemporalEffectGraph, options: &SerializationOptions) -> Result<String> {
        let serializable = Self::to_serializable(teg, options);
        
        if options.pretty {
            serde_json::to_string_pretty(&serializable)
                .map_err(|e| anyhow!("Failed to serialize TEG to JSON: {}", e))
        } else {
            serde_json::to_string(&serializable)
                .map_err(|e| anyhow!("Failed to serialize TEG to JSON: {}", e))
        }
    }
    
    /// Serialize a TEG to binary format
    pub fn to_binary(teg: &TemporalEffectGraph, _options: &SerializationOptions) -> Result<Vec<u8>> {
        borsh::to_vec(teg)
            .map_err(|e| anyhow!("Failed to serialize TEG to binary: {}", e))
    }
    
    /// Serialize a TEG to GraphML format
    pub fn to_graphml(teg: &TemporalEffectGraph, _options: &SerializationOptions) -> Result<String> {
        let mut output = String::new();
        
        // GraphML header
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        output.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");
        
        // Define node and edge attributes
        output.push_str("  <key id=\"type\" for=\"node\" attr.name=\"type\" attr.type=\"string\"/>\n");
        output.push_str("  <key id=\"domain\" for=\"node\" attr.name=\"domain\" attr.type=\"string\"/>\n");
        output.push_str("  <key id=\"relationship\" for=\"edge\" attr.name=\"relationship\" attr.type=\"string\"/>\n");
        
        // Start graph
        output.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");
        
        // Add effect nodes
        for (id, effect) in &teg.effect_nodes {
            output.push_str(&format!("    <node id=\"effect_{id}\">\n"));
            output.push_str(&format!("      <data key=\"type\">{}</data>\n", effect.effect_type));
            output.push_str(&format!("      <data key=\"domain\">{}</data>\n", effect.domain_id));
            output.push_str("    </node>\n");
        }
        
        // Add resource nodes
        for (id, resource) in &teg.resource_nodes {
            output.push_str(&format!("    <node id=\"resource_{id}\">\n"));
            output.push_str(&format!("      <data key=\"type\">{}</data>\n", resource.resource_type));
            output.push_str(&format!("      <data key=\"domain\">{}</data>\n", resource.domain_id));
            output.push_str("    </node>\n");
        }
        
        // Add dependency edges
        for (effect_id, deps) in &teg.effect_dependencies {
            for dep_id in deps {
                output.push_str(&format!("    <edge source=\"effect_{dep_id}\" target=\"effect_{effect_id}\">\n"));
                output.push_str("      <data key=\"relationship\">dependency</data>\n");
                output.push_str("    </edge>\n");
            }
        }
        
        // Add continuation edges
        for (effect_id, conts) in &teg.effect_continuations {
            for (cont_id, _cond) in conts {
                output.push_str(&format!("    <edge source=\"effect_{effect_id}\" target=\"effect_{cont_id}\">\n"));
                output.push_str("      <data key=\"relationship\">continuation</data>\n");
                output.push_str("    </edge>\n");
            }
        }
        
        // Add resource access edges
        for (effect_id, effect) in &teg.effect_nodes {
            for resource_id in &effect.resources_accessed {
                output.push_str(&format!("    <edge source=\"effect_{effect_id}\" target=\"resource_{resource_id}\">\n"));
                output.push_str("      <data key=\"relationship\">access</data>\n");
                output.push_str("    </edge>\n");
            }
        }
        
        // Close graph and GraphML
        output.push_str("  </graph>\n");
        output.push_str("</graphml>\n");
        
        Ok(output)
    }
    
    /// Serialize a TEG to DOT format
    pub fn to_dot(teg: &TemporalEffectGraph, _options: &SerializationOptions) -> Result<String> {
        let mut output = String::new();
        
        // DOT header
        output.push_str("digraph TemporalEffectGraph {\n");
        output.push_str("  // Graph settings\n");
        output.push_str("  graph [rankdir=LR, fontname=\"Arial\", fontsize=12];\n");
        output.push_str("  node [shape=box, style=filled, fontname=\"Arial\", fontsize=10];\n");
        output.push_str("  edge [fontname=\"Arial\", fontsize=8];\n\n");
        
        // Add effect nodes
        output.push_str("  // Effect nodes\n");
        for (id, effect) in &teg.effect_nodes {
            output.push_str(&format!("  effect_{id} [label=\"{}: {}\", fillcolor=\"lightblue\"];\n", 
                id, effect.effect_type));
        }
        
        // Add resource nodes
        output.push_str("\n  // Resource nodes\n");
        for (id, resource) in &teg.resource_nodes {
            output.push_str(&format!("  resource_{id} [label=\"{}: {}\", fillcolor=\"lightgreen\", shape=ellipse];\n", 
                id, resource.resource_type));
        }
        
        // Add dependency edges
        output.push_str("\n  // Dependency edges\n");
        for (effect_id, deps) in &teg.effect_dependencies {
            for dep_id in deps {
                output.push_str(&format!("  effect_{dep_id} -> effect_{effect_id} [style=dashed, label=\"depends on\"];\n"));
            }
        }
        
        // Add continuation edges
        output.push_str("\n  // Continuation edges\n");
        for (effect_id, conts) in &teg.effect_continuations {
            for (cont_id, cond) in conts {
                if let Some(condition) = cond {
                    output.push_str(&format!("  effect_{effect_id} -> effect_{cont_id} [label=\"if {}\"];\n", condition));
                } else {
                    output.push_str(&format!("  effect_{effect_id} -> effect_{cont_id};\n"));
                }
            }
        }
        
        // Add resource access edges
        output.push_str("\n  // Resource access edges\n");
        for (effect_id, effect) in &teg.effect_nodes {
            for resource_id in &effect.resources_accessed {
                output.push_str(&format!("  effect_{effect_id} -> resource_{resource_id} [color=green];\n"));
            }
        }
        
        // Close DOT
        output.push_str("}\n");
        
        Ok(output)
    }
    
    /// Serialize a TEG using the specified format
    pub fn serialize(teg: &TemporalEffectGraph, options: &SerializationOptions) -> Result<Vec<u8>> {
        match options.format {
            SerializationFormat::JSON => {
                let json = Self::to_json(teg, options)?;
                Ok(json.into_bytes())
            }
            SerializationFormat::Binary => {
                Self::to_binary(teg, options)
            }
            SerializationFormat::GraphML => {
                let graphml = Self::to_graphml(teg, options)?;
                Ok(graphml.into_bytes())
            }
            SerializationFormat::DOT => {
                let dot = Self::to_dot(teg, options)?;
                Ok(dot.into_bytes())
            }
        }
    }
    
    /// Create a serializable representation of a TEG
    pub fn to_serializable(teg: &TemporalEffectGraph, options: &SerializationOptions) -> SerializableTEG {
        // Create a serializable representation of the TEG
        let mut serializable = SerializableTEG {
            effects: HashMap::new(),
            resources: HashMap::new(),
            dependencies: teg.effect_dependencies.clone(),
            continuations: teg.effect_continuations.iter().map(|(id, conts)| {
                (id.clone(), conts.iter().map(|(next, cond)| {
                    (next.clone(), cond.clone().map(|c| c.to_string()))
                }).collect::<Vec<_>>())
            }).collect(),
            resource_relationships: teg.resource_relationships.iter().map(|(id, rels)| {
                (id.clone(), rels.iter().map(|(rel_id, rel_type)| {
                    (rel_id.clone(), format!("{:?}", rel_type))
                }).collect::<Vec<_>>())
            }).collect(),
        };
        
        // Add effect nodes
        for (id, effect) in &teg.effect_nodes {
            let mut serialized_effect = SerializableEffect {
                id: id.clone(),
                effect_type: effect.effect_type.clone(),
                domain: effect.domain_id.clone(),
                parameters: HashMap::new(),
            };
            
            // Include parameters if requested
            if options.include_parameters {
                serialized_effect.parameters = effect.parameters.clone();
            }
            
            serializable.effects.insert(id.clone(), serialized_effect);
        }
        
        // Add resource nodes
        for (id, resource) in &teg.resource_nodes {
            let serialized_resource = SerializableResource {
                id: id.clone(),
                resource_type: resource.resource_type.clone(),
                domain: resource.domain_id.clone(),
            };
            
            serializable.resources.insert(id.clone(), serialized_resource);
        }
        
        serializable
    }
    
    /// Generate an incremental update between two TEGs
    pub fn generate_update(
        base: &TemporalEffectGraph, 
        current: &TemporalEffectGraph
    ) -> Result<TEGUpdate> {
        let mut update = TEGUpdate {
            base_version: Some(base.content_hash.to_string()),
            new_version: current.content_hash.to_string(),
            added_effects: HashMap::new(),
            updated_effects: HashMap::new(),
            removed_effects: Vec::new(),
            added_resources: HashMap::new(),
            updated_resources: HashMap::new(),
            removed_resources: Vec::new(),
            updated_dependencies: HashMap::new(),
            updated_continuations: HashMap::new(),
            updated_metadata: HashMap::new(),
        };
        
        // Find added and updated effects
        for (id, effect) in &current.effect_nodes {
            if let Some(base_effect) = base.effect_nodes.get(id) {
                // Check if effect was updated
                if effect != base_effect {
                    update.updated_effects.insert(id.clone(), Self::create_serializable_effect(effect));
                }
            } else {
                // Effect was added
                update.added_effects.insert(id.clone(), Self::create_serializable_effect(effect));
            }
        }
        
        // Find removed effects
        for id in base.effect_nodes.keys() {
            if !current.effect_nodes.contains_key(id) {
                update.removed_effects.push(id.clone());
            }
        }
        
        // Find added and updated resources
        for (id, resource) in &current.resource_nodes {
            if let Some(base_resource) = base.resource_nodes.get(id) {
                // Check if resource was updated
                if resource != base_resource {
                    update.updated_resources.insert(
                        id.clone(), 
                        Self::create_serializable_resource(resource)
                    );
                }
            } else {
                // Resource was added
                update.added_resources.insert(
                    id.clone(), 
                    Self::create_serializable_resource(resource)
                );
            }
        }
        
        // Find removed resources
        for id in base.resource_nodes.keys() {
            if !current.resource_nodes.contains_key(id) {
                update.removed_resources.push(id.clone());
            }
        }
        
        // Find updated dependencies
        for (id, deps) in &current.effect_dependencies {
            if let Some(base_deps) = base.effect_dependencies.get(id) {
                if deps != base_deps {
                    update.updated_dependencies.insert(id.clone(), deps.clone());
                }
            } else {
                update.updated_dependencies.insert(id.clone(), deps.clone());
            }
        }
        
        // Find updated continuations
        for (id, conts) in &current.effect_continuations {
            if let Some(base_conts) = base.effect_continuations.get(id) {
                if conts != base_conts {
                    update.updated_continuations.insert(
                        id.clone(),
                        conts.iter().map(|(next, cond)| {
                            (next.clone(), cond.clone().map(|c| c.to_string()))
                        }).collect()
                    );
                }
            } else {
                update.updated_continuations.insert(
                    id.clone(),
                    conts.iter().map(|(next, cond)| {
                        (next.clone(), cond.clone().map(|c| c.to_string()))
                    }).collect()
                );
            }
        }
        
        // Find updated metadata
        for (key, value) in &current.metadata {
            if let Some(base_value) = base.metadata.get(key) {
                if value != base_value {
                    update.updated_metadata.insert(key.clone(), value.clone());
                }
            } else {
                update.updated_metadata.insert(key.clone(), value.clone());
            }
        }
        
        Ok(update)
    }
    
    /// Create a serializable effect from an effect node
    fn create_serializable_effect(effect: &crate::EffectNode) -> SerializableEffect {
        SerializableEffect {
            id: effect.id.clone(),
            effect_type: effect.effect_type.clone(),
            domain: effect.domain_id.clone(),
            parameters: effect.parameters.clone(),
        }
    }
    
    /// Create a serializable resource from a resource node
    fn create_serializable_resource(resource: &crate::ResourceNode) -> SerializableResource {
        SerializableResource {
            id: resource.id.clone(),
            resource_type: resource.resource_type.clone(),
            domain: resource.domain_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_json_serialization() {
        let mut builder = GraphBuilder::new();
        
        builder.add_effect("effect1", "domain1");
        builder.add_resource("resource1", "type1");
        
        let teg = builder.build().unwrap();
        
        let options = SerializationOptions {
            format: SerializationFormat::JSON,
            include_metadata: true,
            include_parameters: true,
            include_capabilities: true,
            pretty: false,
        };
        
        let json = TEGSerializer::to_json(&teg, &options).unwrap();
        
        // Basic validation - check if it contains the effect and resource IDs
        assert!(json.contains("effect1"));
        assert!(json.contains("resource1"));
        
        // Parse the JSON back to ensure it's valid
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }
    
    #[test]
    fn test_graphml_serialization() {
        let mut builder = GraphBuilder::new();
        
        builder.add_effect("effect1", "domain1");
        builder.add_resource("resource1", "type1");
        
        let teg = builder.build().unwrap();
        
        let options = SerializationOptions {
            format: SerializationFormat::GraphML,
            include_metadata: true,
            include_parameters: true,
            include_capabilities: true,
            pretty: false,
        };
        
        let graphml = TEGSerializer::to_graphml(&teg, &options).unwrap();
        
        // Basic validation - check if it contains the necessary XML elements
        assert!(graphml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(graphml.contains("<graphml"));
        assert!(graphml.contains("<node id=\"effect_effect1\""));
        assert!(graphml.contains("<node id=\"resource_resource1\""));
    }
    
    #[test]
    fn test_generate_update() {
        let mut builder1 = GraphBuilder::new();
        let mut builder2 = GraphBuilder::new();
        
        // Create base TEG
        builder1.add_effect("effect1", "domain1");
        builder1.add_effect("effect2", "domain1");
        builder1.add_resource("resource1", "type1");
        
        // Create current TEG with changes
        builder2.add_effect("effect1", "domain1"); // Unchanged
        builder2.add_effect("effect3", "domain2"); // Added
        builder2.add_resource("resource1", "type1"); // Unchanged
        builder2.add_resource("resource2", "type2"); // Added
        
        let base_teg = builder1.build().unwrap();
        let current_teg = builder2.build().unwrap();
        
        let update = TEGSerializer::generate_update(&base_teg, &current_teg).unwrap();
        
        // Verify the update
        assert_eq!(update.added_effects.len(), 1);
        assert!(update.added_effects.contains_key("effect3"));
        
        assert_eq!(update.removed_effects.len(), 1);
        assert!(update.removed_effects.contains(&"effect2".to_string()));
        
        assert_eq!(update.added_resources.len(), 1);
        assert!(update.added_resources.contains_key("resource2"));
    }
} 