// Fragment module for the Temporal Effect Graph
// This file defines the TEGFragment struct, which represents a composable fragment
// of a Temporal Effect Graph for use in building and transforming TEGs.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::{
    EffectNode, ResourceNode, EffectId, ResourceId,
    graph::edge::{Edge, EdgeId, Condition, RelationshipType},
    graph::TemporalConstraint,
};

/// A composable fragment of a Temporal Effect Graph
/// This structure allows for building TEGs in a modular way,
/// with clear entry and exit points for composition.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TEGFragment {
    /// Effect nodes in the fragment
    pub effect_nodes: HashMap<EffectId, EffectNode>,
    
    /// Resource nodes in the fragment
    pub resource_nodes: HashMap<ResourceId, ResourceNode>,
    
    /// Edges in the fragment
    pub edges: HashMap<EdgeId, Edge>,
    
    /// Effect dependencies within the fragment
    pub effect_dependencies: HashMap<EffectId, Vec<EffectId>>,
    
    /// Effect continuations within the fragment
    pub effect_continuations: HashMap<EffectId, Vec<(EffectId, Option<Condition>)>>,
    
    /// Resource relationships within the fragment
    pub resource_relationships: HashMap<ResourceId, Vec<(ResourceId, RelationshipType)>>,
    
    /// Temporal constraints within the fragment
    pub temporal_constraints: HashMap<EffectId, Vec<TemporalConstraint>>,
    
    /// Entry points for composition - effects that can be connected to previous fragments
    pub entry_points: Vec<EffectId>,
    
    /// Exit points for composition - effects that can be connected to next fragments
    pub exit_points: Vec<EffectId>,
}

impl TEGFragment {
    /// Create a new empty fragment
    pub fn new() -> Self {
        Self {
            effect_nodes: HashMap::new(),
            resource_nodes: HashMap::new(),
            edges: HashMap::new(),
            effect_dependencies: HashMap::new(),
            effect_continuations: HashMap::new(),
            resource_relationships: HashMap::new(),
            temporal_constraints: HashMap::new(),
            entry_points: Vec::new(),
            exit_points: Vec::new(),
        }
    }
    
    /// Create a fragment from a single effect node
    pub fn from_effect(effect: EffectNode) -> Self {
        let effect_id = effect.id.clone();
        let mut fragment = Self::new();
        
        fragment.effect_nodes.insert(effect_id.clone(), effect);
        fragment.entry_points.push(effect_id.clone());
        fragment.exit_points.push(effect_id);
        
        fragment
    }
    
    /// Add an effect node to the fragment
    pub fn add_effect(&mut self, effect: EffectNode) -> &EffectId {
        let effect_id = effect.id.clone();
        self.effect_nodes.insert(effect_id.clone(), effect);
        &effect_id
    }
    
    /// Add a resource node to the fragment
    pub fn add_resource(&mut self, resource: ResourceNode) -> &ResourceId {
        let resource_id = resource.id.clone();
        self.resource_nodes.insert(resource_id.clone(), resource);
        &resource_id
    }
    
    /// Compose this fragment sequentially with another fragment,
    /// connecting this fragment's exit points to the next fragment's entry points
    pub fn sequence(mut self, next: TEGFragment) -> Self {
        // Create connections between exit points of this fragment and entry points of next fragment
        for (i, exit_id) in self.exit_points.iter().enumerate() {
            if i < next.entry_points.len() {
                let entry_id = &next.entry_points[i];
                
                // Add effect continuation
                if let Some(continuations) = self.effect_continuations.get_mut(exit_id) {
                    continuations.push((entry_id.clone(), Some(Condition::Success)));
                } else {
                    self.effect_continuations.insert(
                        exit_id.clone(),
                        vec![(entry_id.clone(), Some(Condition::Success))],
                    );
                }
                
                // Add effect dependency
                if let Some(dependencies) = self.effect_dependencies.get_mut(entry_id) {
                    dependencies.push(exit_id.clone());
                } else {
                    self.effect_dependencies.insert(
                        entry_id.clone(),
                        vec![exit_id.clone()],
                    );
                }
            }
        }
        
        // Merge nodes and relationships from the next fragment
        for (id, effect) in next.effect_nodes {
            self.effect_nodes.insert(id, effect);
        }
        
        for (id, resource) in next.resource_nodes {
            self.resource_nodes.insert(id, resource);
        }
        
        for (id, edge) in next.edges {
            self.edges.insert(id, edge);
        }
        
        for (id, deps) in next.effect_dependencies {
            if let Some(existing_deps) = self.effect_dependencies.get_mut(&id) {
                existing_deps.extend(deps);
            } else {
                self.effect_dependencies.insert(id, deps);
            }
        }
        
        for (id, conts) in next.effect_continuations {
            if let Some(existing_conts) = self.effect_continuations.get_mut(&id) {
                existing_conts.extend(conts);
            } else {
                self.effect_continuations.insert(id, conts);
            }
        }
        
        for (id, rels) in next.resource_relationships {
            if let Some(existing_rels) = self.resource_relationships.get_mut(&id) {
                existing_rels.extend(rels);
            } else {
                self.resource_relationships.insert(id, rels);
            }
        }
        
        for (id, constraints) in next.temporal_constraints {
            if let Some(existing_constraints) = self.temporal_constraints.get_mut(&id) {
                existing_constraints.extend(constraints);
            } else {
                self.temporal_constraints.insert(id, constraints);
            }
        }
        
        // Update entry and exit points
        // Entry points remain the same (from this fragment)
        // Exit points come from the next fragment
        self.exit_points = next.exit_points;
        
        self
    }
    
    /// Compose this fragment in parallel with another fragment
    pub fn parallel(mut self, other: TEGFragment) -> Self {
        // Merge nodes and relationships from the other fragment
        for (id, effect) in other.effect_nodes {
            self.effect_nodes.insert(id, effect);
        }
        
        for (id, resource) in other.resource_nodes {
            self.resource_nodes.insert(id, resource);
        }
        
        for (id, edge) in other.edges {
            self.edges.insert(id, edge);
        }
        
        for (id, deps) in other.effect_dependencies {
            if let Some(existing_deps) = self.effect_dependencies.get_mut(&id) {
                existing_deps.extend(deps);
            } else {
                self.effect_dependencies.insert(id, deps);
            }
        }
        
        for (id, conts) in other.effect_continuations {
            if let Some(existing_conts) = self.effect_continuations.get_mut(&id) {
                existing_conts.extend(conts);
            } else {
                self.effect_continuations.insert(id, conts);
            }
        }
        
        for (id, rels) in other.resource_relationships {
            if let Some(existing_rels) = self.resource_relationships.get_mut(&id) {
                existing_rels.extend(rels);
            } else {
                self.resource_relationships.insert(id, rels);
            }
        }
        
        for (id, constraints) in other.temporal_constraints {
            if let Some(existing_constraints) = self.temporal_constraints.get_mut(&id) {
                existing_constraints.extend(constraints);
            } else {
                self.temporal_constraints.insert(id, constraints);
            }
        }
        
        // Combine entry and exit points
        self.entry_points.extend(other.entry_points);
        self.exit_points.extend(other.exit_points);
        
        self
    }
    
    /// Apply a conditional branch based on a condition
    pub fn branch(
        mut self,
        condition: Condition,
        then_fragment: TEGFragment,
        else_fragment: Option<TEGFragment>,
    ) -> Self {
        // TODO: Implement branching logic
        // This will require creating control flow nodes and appropriate edges
        
        // For now, we'll just return a placeholder implementation that uses the then_fragment
        if let Some(else_frag) = else_fragment {
            // If we have an else branch, add both branches
            let then_entries = then_fragment.entry_points.clone();
            let else_entries = else_frag.entry_points.clone();
            
            // Add the then fragment
            for (id, effect) in then_fragment.effect_nodes {
                self.effect_nodes.insert(id, effect);
            }
            
            // Add the else fragment
            for (id, effect) in else_frag.effect_nodes {
                self.effect_nodes.insert(id, effect);
            }
            
            // Update exit points to include all exit points from both branches
            self.exit_points = Vec::new();
            self.exit_points.extend(then_fragment.exit_points);
            self.exit_points.extend(else_frag.exit_points);
            
            // Connect entry points based on condition
            // (simplified implementation)
            for exit_id in &self.exit_points {
                // Connect to then branch with the specified condition
                for entry_id in &then_entries {
                    if let Some(continuations) = self.effect_continuations.get_mut(exit_id) {
                        continuations.push((entry_id.clone(), Some(condition.clone())));
                    } else {
                        self.effect_continuations.insert(
                            exit_id.clone(),
                            vec![(entry_id.clone(), Some(condition.clone()))],
                        );
                    }
                }
                
                // Connect to else branch with the inverse condition
                // (simplified - in reality we would create a proper "else" condition)
                for entry_id in &else_entries {
                    if let Some(continuations) = self.effect_continuations.get_mut(exit_id) {
                        continuations.push((entry_id.clone(), None)); // None is a placeholder for "else"
                    } else {
                        self.effect_continuations.insert(
                            exit_id.clone(),
                            vec![(entry_id.clone(), None)],
                        );
                    }
                }
            }
            
        } else {
            // If we have no else branch, just add the then branch
            for (id, effect) in then_fragment.effect_nodes {
                self.effect_nodes.insert(id, effect);
            }
            
            // Update exit points
            self.exit_points = then_fragment.exit_points;
        }
        
        self
    }
}
