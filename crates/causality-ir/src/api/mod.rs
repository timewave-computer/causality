//! API for Temporal Effect Graph (TEG)
//! 
//! This module provides a public API for interacting with TEGs, including:
//! 1. Creation and modification of TEGs
//! 2. Querying and analyzing TEGs
//! 3. Serializing TEGs to various formats
//! 4. Converting TEGs to and from other representations

pub mod analysis;
pub mod serialization;
pub mod manipulation;
pub mod query;
pub mod export;

// Re-export the most commonly used types
pub use analysis::TEGAnalyzer;
pub use serialization::{TEGSerializer, SerializationFormat, SerializationOptions};
pub use manipulation::{TEGTransaction, TEGDiff, TEGManipulator};
pub use query::TEGQuery;
pub use export::TEGExporter;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{TemporalEffectGraph, EffectId, ResourceId, DomainId};

/// Public API for interacting with a TEG
#[derive(Debug)]
pub struct TEGApi {
    /// The underlying TEG
    teg: TemporalEffectGraph,
}

impl TEGApi {
    /// Create a new TEG API from an existing graph
    pub fn new(teg: TemporalEffectGraph) -> Self {
        Self { teg }
    }
    
    /// Create a new empty TEG API
    pub fn empty() -> Self {
        Self { teg: TemporalEffectGraph::new() }
    }
    
    /// Get the underlying TEG
    pub fn teg(&self) -> &TemporalEffectGraph {
        &self.teg
    }
    
    /// Get a mutable reference to the underlying TEG
    pub fn teg_mut(&mut self) -> &mut TemporalEffectGraph {
        &mut self.teg
    }
    
    /// Get a summary of the TEG
    pub fn summary(&self) -> TEGSummary {
        TEGSummary {
            effect_count: self.teg.effect_nodes.len(),
            resource_count: self.teg.resource_nodes.len(),
            domain_count: self.teg.domains.len(),
            entry_points: self.get_entry_points().len(),
            exit_points: self.get_exit_points().len(),
        }
    }
    
    /// Get all entry points (effects with no dependencies)
    pub fn get_entry_points(&self) -> Vec<EffectId> {
        self.teg.effect_nodes.keys()
            .filter(|id| {
                !self.teg.effect_dependencies.values().any(|deps| deps.contains(id))
            })
            .cloned()
            .collect()
    }
    
    /// Get all exit points (effects with no continuations)
    pub fn get_exit_points(&self) -> Vec<EffectId> {
        self.teg.effect_nodes.keys()
            .filter(|id| {
                !self.teg.effect_continuations.contains_key(*id) || 
                self.teg.effect_continuations.get(*id).map_or(true, |conts| conts.is_empty())
            })
            .cloned()
            .collect()
    }
    
    /// Get effects by domain
    pub fn get_effects_by_domain(&self, domain_id: &DomainId) -> Vec<EffectId> {
        self.teg.effect_nodes.iter()
            .filter_map(|(id, effect)| {
                if &effect.domain_id == domain_id {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get resources by domain
    pub fn get_resources_by_domain(&self, domain_id: &DomainId) -> Vec<ResourceId> {
        self.teg.resource_nodes.iter()
            .filter_map(|(id, resource)| {
                if &resource.domain_id == domain_id {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all resources accessed by an effect
    pub fn get_resources_accessed_by_effect(&self, effect_id: &EffectId) -> Vec<ResourceId> {
        if let Some(effect) = self.teg.effect_nodes.get(effect_id) {
            effect.resources_accessed.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Get all effects that access a resource
    pub fn get_effects_accessing_resource(&self, resource_id: &ResourceId) -> Vec<EffectId> {
        self.teg.effect_nodes.iter()
            .filter_map(|(id, effect)| {
                if effect.resources_accessed.contains(resource_id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Find a path between two effects
    pub fn find_path(&self, from: &EffectId, to: &EffectId) -> Option<Vec<EffectId>> {
        // Simplified BFS implementation to find a path
        use std::collections::{HashSet, VecDeque};
        
        if from == to {
            return Some(vec![from.clone()]);
        }
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessor = HashMap::new();
        
        queue.push_back(from.clone());
        visited.insert(from.clone());
        
        while let Some(current) = queue.pop_front() {
            // Check continuations
            if let Some(continuations) = self.teg.effect_continuations.get(&current) {
                for (next, _) in continuations {
                    if !visited.contains(next) {
                        visited.insert(next.clone());
                        predecessor.insert(next.clone(), current.clone());
                        queue.push_back(next.clone());
                        
                        if next == to {
                            // Found the target, reconstruct the path
                            let mut path = Vec::new();
                            let mut current = next.clone();
                            
                            while current != *from {
                                path.push(current.clone());
                                current = predecessor[&current].clone();
                            }
                            
                            path.push(from.clone());
                            path.reverse();
                            return Some(path);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Convert the TEG to a serializable format
    pub fn to_serializable(&self) -> SerializableTEG {
        // Create a serializable representation of the TEG
        SerializableTEG {
            effects: self.teg.effect_nodes.iter().map(|(id, effect)| {
                (id.clone(), SerializableEffect {
                    id: id.clone(),
                    effect_type: effect.effect_type.clone(),
                    domain: effect.domain_id.clone(),
                    parameters: effect.parameters.clone(),
                })
            }).collect(),
            
            resources: self.teg.resource_nodes.iter().map(|(id, resource)| {
                (id.clone(), SerializableResource {
                    id: id.clone(),
                    resource_type: resource.resource_type.clone(),
                    domain: resource.domain_id.clone(),
                })
            }).collect(),
            
            dependencies: self.teg.effect_dependencies.clone(),
            continuations: self.teg.effect_continuations.iter().map(|(id, conts)| {
                (id.clone(), conts.iter().map(|(next, cond)| {
                    (next.clone(), cond.clone().map(|c| c.to_string()))
                }).collect::<Vec<_>>())
            }).collect(),
            
            resource_relationships: self.teg.resource_relationships.iter().map(|(id, rels)| {
                (id.clone(), rels.iter().map(|(rel_id, rel_type)| {
                    (rel_id.clone(), format!("{:?}", rel_type))
                }).collect::<Vec<_>>())
            }).collect(),
        }
    }
}

/// Summary of a TEG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TEGSummary {
    /// Number of effect nodes
    pub effect_count: usize,
    
    /// Number of resource nodes
    pub resource_count: usize,
    
    /// Number of domains
    pub domain_count: usize,
    
    /// Number of entry points
    pub entry_points: usize,
    
    /// Number of exit points
    pub exit_points: usize,
}

/// Serializable representation of a TEG for external consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTEG {
    /// Effect nodes
    pub effects: HashMap<EffectId, SerializableEffect>,
    
    /// Resource nodes
    pub resources: HashMap<ResourceId, SerializableResource>,
    
    /// Effect dependencies
    pub dependencies: HashMap<EffectId, Vec<EffectId>>,
    
    /// Effect continuations with string-based conditions
    pub continuations: HashMap<EffectId, Vec<(EffectId, Option<String>)>>,
    
    /// Resource relationships with string-based types
    pub resource_relationships: HashMap<ResourceId, Vec<(ResourceId, String)>>,
}

/// Serializable representation of an effect node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEffect {
    /// Unique identifier
    pub id: EffectId,
    
    /// Type of effect
    pub effect_type: String,
    
    /// Domain ID
    pub domain: DomainId,
    
    /// Effect parameters
    pub parameters: HashMap<String, crate::effect_node::ParameterValue>,
}

/// Serializable representation of a resource node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableResource {
    /// Unique identifier
    pub id: ResourceId,
    
    /// Type of resource
    pub resource_type: String,
    
    /// Domain ID
    pub domain: DomainId,
}

/// Create a new TEG API from an existing graph
pub fn create_api(teg: TemporalEffectGraph) -> TEGApi {
    TEGApi::new(teg)
}

/// Create a new empty TEG API
pub fn create_empty_api() -> TEGApi {
    TEGApi::empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_create_api() {
        let api = create_empty_api();
        assert_eq!(api.summary().effect_count, 0);
        assert_eq!(api.summary().resource_count, 0);
    }
    
    #[test]
    fn test_entry_exit_points() {
        let mut builder = GraphBuilder::new();
        
        // Create a simple linear graph
        let effect1 = builder.add_effect("effect1", "test");
        let effect2 = builder.add_effect("effect2", "test");
        let effect3 = builder.add_effect("effect3", "test");
        
        builder.connect_effects(effect1, effect2);
        builder.connect_effects(effect2, effect3);
        
        let teg = builder.build().unwrap();
        let api = TEGApi::new(teg);
        
        let entry_points = api.get_entry_points();
        let exit_points = api.get_exit_points();
        
        assert_eq!(entry_points.len(), 1);
        assert_eq!(exit_points.len(), 1);
        assert_eq!(entry_points[0], effect1);
        assert_eq!(exit_points[0], effect3);
    }
    
    #[test]
    fn test_find_path() {
        let mut builder = GraphBuilder::new();
        
        // Create a simple graph with multiple paths
        let effect1 = builder.add_effect("effect1", "test");
        let effect2 = builder.add_effect("effect2", "test");
        let effect3 = builder.add_effect("effect3", "test");
        let effect4 = builder.add_effect("effect4", "test");
        
        builder.connect_effects(effect1, effect2);
        builder.connect_effects(effect1, effect3);
        builder.connect_effects(effect2, effect4);
        builder.connect_effects(effect3, effect4);
        
        let teg = builder.build().unwrap();
        let api = TEGApi::new(teg);
        
        let path = api.find_path(&effect1, &effect4);
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert!(path.contains(&effect1));
        assert!(path.contains(&effect4));
        assert!(path.len() == 3); // Should be either [effect1, effect2, effect4] or [effect1, effect3, effect4]
    }
} 