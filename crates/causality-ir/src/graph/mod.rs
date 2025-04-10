// Graph module for the Temporal Effect Graph (TEG)
// This module defines the main graph structure and operations.

pub mod effect;
pub mod resource;
pub mod edge;

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use causality_types::{ContentHash, ContentAddressed, ContentHashError};

use crate::{EffectNode, ResourceNode, EffectId, ResourceId, CapabilityId, DomainId, FactId};
use self::edge::{Edge, EdgeId, EdgeType, NodeId, Condition, TemporalRelation, RelationshipType, AccessMode};

/// Temporal constraint between effects
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TemporalConstraint {
    /// Source effect
    pub source: EffectId,
    
    /// Target effect
    pub target: EffectId,
    
    /// Type of temporal relation
    pub relation: TemporalRelation,
}

/// The main Temporal Effect Graph structure
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TemporalEffectGraph {
    /// Effect nodes in the graph
    pub effect_nodes: HashMap<EffectId, EffectNode>,
    
    /// Resource nodes in the graph
    pub resource_nodes: HashMap<ResourceId, ResourceNode>,
    
    /// Edges in the graph
    pub edges: HashMap<EdgeId, Edge>,
    
    /// Effect dependencies (effect_id -> [dependent_effect_ids])
    pub effect_dependencies: HashMap<EffectId, Vec<EffectId>>,
    
    /// Effect continuations (effect_id -> [(next_effect_id, condition)])
    pub effect_continuations: HashMap<EffectId, Vec<(EffectId, Option<Condition>)>>,
    
    /// Resource relationships (resource_id -> [(related_resource_id, relationship_type)])
    pub resource_relationships: HashMap<ResourceId, Vec<(ResourceId, RelationshipType)>>,
    
    /// Temporal constraints (effect_id -> [temporal_constraints])
    pub temporal_constraints: HashMap<EffectId, Vec<TemporalConstraint>>,
    
    /// Capability authorizations (effect_id -> [capability_ids])
    pub capability_authorizations: HashMap<EffectId, Vec<CapabilityId>>,
    
    /// Set of domains in the graph
    pub domains: HashSet<DomainId>,
    
    /// Additional metadata about the graph
    pub metadata: HashMap<String, String>,
    
    /// Content hash for this graph
    pub content_hash: ContentHash,
}

impl TemporalEffectGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            effect_nodes: HashMap::new(),
            resource_nodes: HashMap::new(),
            edges: HashMap::new(),
            effect_dependencies: HashMap::new(),
            effect_continuations: HashMap::new(),
            resource_relationships: HashMap::new(),
            temporal_constraints: HashMap::new(),
            capability_authorizations: HashMap::new(),
            domains: HashSet::new(),
            metadata: HashMap::new(),
            content_hash: ContentHash::default(), // Placeholder
        }
    }
    
    /// Add an effect node to the graph
    pub fn add_effect_node(&mut self, effect: EffectNode) -> EffectId {
        let effect_id = effect.id.clone();
        self.domains.insert(effect.domain_id.clone());
        self.effect_nodes.insert(effect_id.clone(), effect);
        effect_id
    }
    
    /// Add a resource node to the graph
    pub fn add_resource_node(&mut self, resource: ResourceNode) -> ResourceId {
        let resource_id = resource.id.clone();
        self.domains.insert(resource.domain_id.clone());
        self.resource_nodes.insert(resource_id.clone(), resource);
        resource_id
    }
    
    /// Incorporate a TEG fragment into this graph
    ///
    /// This method takes a TEGFragment and adds all of its nodes and relationships
    /// to this graph, optionally with a namespace to avoid ID collisions.
    ///
    /// # Arguments
    /// * `fragment` - The fragment to incorporate
    /// * `namespace` - Optional namespace to prefix all IDs in the fragment
    ///
    /// # Returns
    /// Result with a mapping from original IDs to new IDs, or an error if the operation failed
    pub fn incorporate_fragment(
        &mut self, 
        fragment: crate::fragment::TEGFragment, 
        namespace: Option<String>
    ) -> anyhow::Result<HashMap<String, String>> {
        // Create a mapping from original IDs to new IDs
        let mut id_mapping = HashMap::new();
        
        // Use namespace if provided
        let prefix = namespace.unwrap_or_default();
        
        // Handle effect nodes
        for (id, mut effect) in fragment.effect_nodes {
            let new_id = if prefix.is_empty() {
                id.clone()
            } else {
                format!("{}_{}", prefix, id)
            };
            
            // Update ID in the node
            effect.id = new_id.clone();
            
            // Store mapping
            id_mapping.insert(id, new_id.clone());
            
            // Add to graph
            self.add_effect_node(effect);
        }
        
        // Handle resource nodes
        for (id, mut resource) in fragment.resource_nodes {
            let new_id = if prefix.is_empty() {
                id.clone()
            } else {
                format!("{}_{}", prefix, id)
            };
            
            // Update ID in the node
            resource.id = new_id.clone();
            
            // Store mapping
            id_mapping.insert(id, new_id.clone());
            
            // Add to graph
            self.add_resource_node(resource);
        }
        
        // Update effect dependencies
        for (orig_effect_id, deps) in fragment.effect_dependencies {
            if let Some(new_effect_id) = id_mapping.get(&orig_effect_id) {
                let mapped_deps = deps.iter()
                    .filter_map(|dep_id| id_mapping.get(dep_id).cloned())
                    .collect();
                
                self.effect_dependencies.insert(new_effect_id.clone(), mapped_deps);
            }
        }
        
        // Update effect continuations
        for (orig_effect_id, conts) in fragment.effect_continuations {
            if let Some(new_effect_id) = id_mapping.get(&orig_effect_id) {
                let mapped_conts = conts.iter()
                    .filter_map(|(cont_id, cond)| {
                        id_mapping.get(cont_id).map(|new_cont_id| (new_cont_id.clone(), cond.clone()))
                    })
                    .collect();
                
                self.effect_continuations.insert(new_effect_id.clone(), mapped_conts);
            }
        }
        
        // Update resource relationships
        for (orig_resource_id, rels) in fragment.resource_relationships {
            if let Some(new_resource_id) = id_mapping.get(&orig_resource_id) {
                let mapped_rels = rels.iter()
                    .filter_map(|(rel_id, rel_type)| {
                        id_mapping.get(rel_id).map(|new_rel_id| (new_rel_id.clone(), rel_type.clone()))
                    })
                    .collect();
                
                self.resource_relationships.insert(new_resource_id.clone(), mapped_rels);
            }
        }
        
        Ok(id_mapping)
    }
    
    /// Add metadata for an effect type to the graph
    ///
    /// # Arguments
    /// * `effect_name` - The name of the effect
    /// * `metadata` - The metadata to add
    ///
    /// # Returns
    /// Result<()> indicating success or failure
    pub fn add_effect_metadata(
        &mut self,
        effect_name: String,
        metadata: HashMap<String, String>
    ) -> anyhow::Result<()> {
        // Store the metadata with a prefixed key to indicate it's for an effect
        for (key, value) in metadata {
            let prefixed_key = format!("effect_meta_{}_{}", effect_name, key);
            self.metadata.insert(prefixed_key, value);
        }
        
        Ok(())
    }
    
    /// Get an iterator over all effect nodes in the graph
    pub fn effect_nodes(&self) -> impl Iterator<Item = &EffectNode> {
        self.effect_nodes.values()
    }
    
    /// Get an iterator over all resource nodes in the graph
    pub fn resource_nodes(&self) -> impl Iterator<Item = &ResourceNode> {
        self.resource_nodes.values()
    }
    
    /// Get a specific effect node by ID
    pub fn get_effect(&self, id: &EffectId) -> Option<&EffectNode> {
        self.effect_nodes.get(id)
    }
    
    /// Get a specific resource node by ID
    pub fn get_resource(&self, id: &ResourceId) -> Option<&ResourceNode> {
        self.resource_nodes.get(id)
    }
    
    /// Get the output effects (effects without continuations)
    pub fn get_output_effects(&self) -> Vec<EffectId> {
        let mut output_effects = Vec::new();
        for (effect_id, _) in &self.effect_nodes {
            if !self.effect_continuations.contains_key(effect_id) || 
               self.effect_continuations.get(effect_id).map_or(true, |v| v.is_empty()) {
                output_effects.push(effect_id.clone());
            }
        }
        output_effects
    }
}

impl ContentAddressed for TemporalEffectGraph {
    fn content_hash(&self) -> Result<ContentHash, ContentHashError> {
        // For now, we'll return the precalculated hash
        // In a full implementation, we would compute the hash here
        Ok(self.content_hash.clone())
    }
    
    fn verify(&self) -> Result<bool, ContentHashError> {
        let computed_hash = self.content_hash()?;
        Ok(computed_hash == self.content_hash)
    }
}
