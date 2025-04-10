//! Manipulation operations for the Temporal Effect Graph (TEG)
//!
//! This module provides operations for modifying TEGs, including graph
//! transformations, merging, and diff operations.

use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};

use crate::{TemporalEffectGraph, EffectId, ResourceId, EffectNode, ResourceNode, DomainId};
use crate::graph::edge::{RelationshipType, Condition};

/// A modification transaction for the TEG
pub struct TEGTransaction {
    /// Original TEG (unmodified)
    original: TemporalEffectGraph,
    
    /// Working copy of the TEG
    working_copy: TemporalEffectGraph,
    
    /// Whether the transaction has been committed
    committed: bool,
}

impl TEGTransaction {
    /// Create a new transaction
    pub fn new(teg: TemporalEffectGraph) -> Self {
        Self {
            original: teg.clone(),
            working_copy: teg,
            committed: false,
        }
    }
    
    /// Add an effect node
    pub fn add_effect(&mut self, effect: EffectNode) -> Result<&EffectId> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        Ok(self.working_copy.add_effect_node(effect))
    }
    
    /// Add a resource node
    pub fn add_resource(&mut self, resource: ResourceNode) -> Result<&ResourceId> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        Ok(self.working_copy.add_resource_node(resource))
    }
    
    /// Remove an effect node
    pub fn remove_effect(&mut self, effect_id: &EffectId) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        if !self.working_copy.effect_nodes.contains_key(effect_id) {
            return Err(anyhow!("Effect not found: {}", effect_id));
        }
        
        // Remove effect node
        self.working_copy.effect_nodes.remove(effect_id);
        
        // Remove from dependencies
        self.working_copy.effect_dependencies.remove(effect_id);
        for deps in self.working_copy.effect_dependencies.values_mut() {
            deps.retain(|id| id != effect_id);
        }
        
        // Remove from continuations
        self.working_copy.effect_continuations.remove(effect_id);
        for conts in self.working_copy.effect_continuations.values_mut() {
            conts.retain(|(id, _)| id != effect_id);
        }
        
        // Remove from temporal constraints
        self.working_copy.temporal_constraints.remove(effect_id);
        for constraints in self.working_copy.temporal_constraints.values_mut() {
            constraints.retain(|c| &c.source != effect_id && &c.target != effect_id);
        }
        
        // Remove from capability authorizations
        self.working_copy.capability_authorizations.remove(effect_id);
        
        Ok(())
    }
    
    /// Remove a resource node
    pub fn remove_resource(&mut self, resource_id: &ResourceId) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        if !self.working_copy.resource_nodes.contains_key(resource_id) {
            return Err(anyhow!("Resource not found: {}", resource_id));
        }
        
        // Remove resource node
        self.working_copy.resource_nodes.remove(resource_id);
        
        // Remove from resource relationships
        self.working_copy.resource_relationships.remove(resource_id);
        for rels in self.working_copy.resource_relationships.values_mut() {
            rels.retain(|(id, _)| id != resource_id);
        }
        
        // Remove from effect resource accesses
        for effect in self.working_copy.effect_nodes.values_mut() {
            effect.resources_accessed.retain(|id| id != resource_id);
        }
        
        Ok(())
    }
    
    /// Add a dependency between effects
    pub fn add_dependency(&mut self, from: &EffectId, to: &EffectId) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        if !self.working_copy.effect_nodes.contains_key(from) {
            return Err(anyhow!("Source effect not found: {}", from));
        }
        
        if !self.working_copy.effect_nodes.contains_key(to) {
            return Err(anyhow!("Target effect not found: {}", to));
        }
        
        // Add the dependency
        self.working_copy.effect_dependencies
            .entry(to.clone())
            .or_insert_with(Vec::new)
            .push(from.clone());
        
        Ok(())
    }
    
    /// Add a continuation between effects
    pub fn add_continuation(
        &mut self, 
        from: &EffectId, 
        to: &EffectId, 
        condition: Option<Condition>
    ) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        if !self.working_copy.effect_nodes.contains_key(from) {
            return Err(anyhow!("Source effect not found: {}", from));
        }
        
        if !self.working_copy.effect_nodes.contains_key(to) {
            return Err(anyhow!("Target effect not found: {}", to));
        }
        
        // Add the continuation
        self.working_copy.effect_continuations
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push((to.clone(), condition));
        
        Ok(())
    }
    
    /// Add a relationship between resources
    pub fn add_resource_relationship(
        &mut self, 
        from: &ResourceId, 
        to: &ResourceId, 
        relationship: RelationshipType
    ) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        if !self.working_copy.resource_nodes.contains_key(from) {
            return Err(anyhow!("Source resource not found: {}", from));
        }
        
        if !self.working_copy.resource_nodes.contains_key(to) {
            return Err(anyhow!("Target resource not found: {}", to));
        }
        
        // Add the relationship
        self.working_copy.resource_relationships
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push((to.clone(), relationship));
        
        Ok(())
    }
    
    /// Update an effect's properties
    pub fn update_effect(
        &mut self,
        effect_id: &EffectId,
        updates: HashMap<String, String>,
    ) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        let effect = self.working_copy.effect_nodes.get_mut(effect_id)
            .ok_or_else(|| anyhow!("Effect not found: {}", effect_id))?;
        
        // Apply updates
        for (key, value) in updates {
            match key.as_str() {
                "effect_type" => {
                    effect.effect_type = value;
                }
                _ => {
                    // Add as a parameter
                    effect.parameters.insert(key, value.into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Update a resource's properties
    pub fn update_resource(
        &mut self,
        resource_id: &ResourceId,
        updates: HashMap<String, String>,
    ) -> Result<()> {
        if self.committed {
            return Err(anyhow!("Transaction already committed"));
        }
        
        let resource = self.working_copy.resource_nodes.get_mut(resource_id)
            .ok_or_else(|| anyhow!("Resource not found: {}", resource_id))?;
        
        // Apply updates
        for (key, value) in updates {
            match key.as_str() {
                "resource_type" => {
                    resource.resource_type = value;
                }
                _ => {
                    // Add as metadata
                    resource.metadata.insert(key, value.into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Commit the transaction and return the modified TEG
    pub fn commit(mut self) -> TemporalEffectGraph {
        self.committed = true;
        self.working_copy
    }
    
    /// Rollback the transaction and return the original TEG
    pub fn rollback(mut self) -> TemporalEffectGraph {
        self.committed = true;
        self.original
    }
    
    /// Check if the transaction has been committed
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

/// A diff between two TEGs
#[derive(Debug)]
pub struct TEGDiff {
    /// Added effect nodes
    pub added_effects: HashSet<EffectId>,
    
    /// Removed effect nodes
    pub removed_effects: HashSet<EffectId>,
    
    /// Modified effect nodes
    pub modified_effects: HashSet<EffectId>,
    
    /// Added resource nodes
    pub added_resources: HashSet<ResourceId>,
    
    /// Removed resource nodes
    pub removed_resources: HashSet<ResourceId>,
    
    /// Modified resource nodes
    pub modified_resources: HashSet<ResourceId>,
    
    /// Added dependencies
    pub added_dependencies: Vec<(EffectId, EffectId)>,
    
    /// Removed dependencies
    pub removed_dependencies: Vec<(EffectId, EffectId)>,
    
    /// Added continuations
    pub added_continuations: Vec<(EffectId, EffectId, Option<Condition>)>,
    
    /// Removed continuations
    pub removed_continuations: Vec<(EffectId, EffectId)>,
}

/// TEG manipulation utilities
pub struct TEGManipulator;

impl TEGManipulator {
    /// Create a transaction for modifying a TEG
    pub fn create_transaction(teg: TemporalEffectGraph) -> TEGTransaction {
        TEGTransaction::new(teg)
    }
    
    /// Compute the difference between two TEGs
    pub fn diff(
        original: &TemporalEffectGraph, 
        current: &TemporalEffectGraph
    ) -> TEGDiff {
        let mut diff = TEGDiff {
            added_effects: HashSet::new(),
            removed_effects: HashSet::new(),
            modified_effects: HashSet::new(),
            added_resources: HashSet::new(),
            removed_resources: HashSet::new(),
            modified_resources: HashSet::new(),
            added_dependencies: Vec::new(),
            removed_dependencies: Vec::new(),
            added_continuations: Vec::new(),
            removed_continuations: Vec::new(),
        };
        
        // Find added and modified effects
        for (id, effect) in &current.effect_nodes {
            if let Some(orig_effect) = original.effect_nodes.get(id) {
                if effect != orig_effect {
                    diff.modified_effects.insert(id.clone());
                }
            } else {
                diff.added_effects.insert(id.clone());
            }
        }
        
        // Find removed effects
        for id in original.effect_nodes.keys() {
            if !current.effect_nodes.contains_key(id) {
                diff.removed_effects.insert(id.clone());
            }
        }
        
        // Find added and modified resources
        for (id, resource) in &current.resource_nodes {
            if let Some(orig_resource) = original.resource_nodes.get(id) {
                if resource != orig_resource {
                    diff.modified_resources.insert(id.clone());
                }
            } else {
                diff.added_resources.insert(id.clone());
            }
        }
        
        // Find removed resources
        for id in original.resource_nodes.keys() {
            if !current.resource_nodes.contains_key(id) {
                diff.removed_resources.insert(id.clone());
            }
        }
        
        // Find added and removed dependencies
        for (effect_id, deps) in &current.effect_dependencies {
            let orig_deps = original.effect_dependencies.get(effect_id).cloned()
                .unwrap_or_default();
            
            for dep_id in deps {
                if !orig_deps.contains(dep_id) {
                    diff.added_dependencies.push((dep_id.clone(), effect_id.clone()));
                }
            }
            
            for dep_id in &orig_deps {
                if !deps.contains(dep_id) {
                    diff.removed_dependencies.push((dep_id.clone(), effect_id.clone()));
                }
            }
        }
        
        // Check for removed effects in dependencies
        for (effect_id, deps) in &original.effect_dependencies {
            if !current.effect_dependencies.contains_key(effect_id) {
                for dep_id in deps {
                    diff.removed_dependencies.push((dep_id.clone(), effect_id.clone()));
                }
            }
        }
        
        // Find added and removed continuations
        for (effect_id, conts) in &current.effect_continuations {
            let orig_conts = original.effect_continuations.get(effect_id).cloned()
                .unwrap_or_default();
            
            for (cont_id, cond) in conts {
                if !orig_conts.iter().any(|(id, _)| id == cont_id) {
                    diff.added_continuations.push((
                        effect_id.clone(), 
                        cont_id.clone(), 
                        cond.clone()
                    ));
                }
            }
            
            for (cont_id, _) in &orig_conts {
                if !conts.iter().any(|(id, _)| id == cont_id) {
                    diff.removed_continuations.push((effect_id.clone(), cont_id.clone()));
                }
            }
        }
        
        // Check for removed effects in continuations
        for (effect_id, conts) in &original.effect_continuations {
            if !current.effect_continuations.contains_key(effect_id) {
                for (cont_id, _) in conts {
                    diff.removed_continuations.push((effect_id.clone(), cont_id.clone()));
                }
            }
        }
        
        diff
    }
    
    /// Merge two TEGs
    pub fn merge(base: &TemporalEffectGraph, other: &TemporalEffectGraph) -> Result<TemporalEffectGraph> {
        let mut merged = base.clone();
        
        // Merge effect nodes
        for (id, effect) in &other.effect_nodes {
            merged.effect_nodes.insert(id.clone(), effect.clone());
        }
        
        // Merge resource nodes
        for (id, resource) in &other.resource_nodes {
            merged.resource_nodes.insert(id.clone(), resource.clone());
        }
        
        // Merge dependencies
        for (effect_id, deps) in &other.effect_dependencies {
            let merged_deps = merged.effect_dependencies
                .entry(effect_id.clone())
                .or_insert_with(Vec::new);
            
            for dep_id in deps {
                if !merged_deps.contains(dep_id) {
                    merged_deps.push(dep_id.clone());
                }
            }
        }
        
        // Merge continuations
        for (effect_id, conts) in &other.effect_continuations {
            let merged_conts = merged.effect_continuations
                .entry(effect_id.clone())
                .or_insert_with(Vec::new);
            
            for (cont_id, cond) in conts {
                if !merged_conts.iter().any(|(id, _)| id == cont_id) {
                    merged_conts.push((cont_id.clone(), cond.clone()));
                }
            }
        }
        
        // Merge resource relationships
        for (resource_id, rels) in &other.resource_relationships {
            let merged_rels = merged.resource_relationships
                .entry(resource_id.clone())
                .or_insert_with(Vec::new);
            
            for (rel_id, rel_type) in rels {
                if !merged_rels.iter().any(|(id, _)| id == rel_id) {
                    merged_rels.push((rel_id.clone(), rel_type.clone()));
                }
            }
        }
        
        // Merge temporal constraints
        for (effect_id, constraints) in &other.temporal_constraints {
            let merged_constraints = merged.temporal_constraints
                .entry(effect_id.clone())
                .or_insert_with(Vec::new);
            
            for constraint in constraints {
                if !merged_constraints.contains(constraint) {
                    merged_constraints.push(constraint.clone());
                }
            }
        }
        
        // Merge capability authorizations
        for (effect_id, caps) in &other.capability_authorizations {
            let merged_caps = merged.capability_authorizations
                .entry(effect_id.clone())
                .or_insert_with(Vec::new);
            
            for cap in caps {
                if !merged_caps.contains(cap) {
                    merged_caps.push(cap.clone());
                }
            }
        }
        
        // Merge domains
        for domain in &other.domains {
            merged.domains.insert(domain.clone());
        }
        
        // Merge metadata
        for (key, value) in &other.metadata {
            merged.metadata.insert(key.clone(), value.clone());
        }
        
        Ok(merged)
    }
    
    /// Extract a subgraph from a TEG
    pub fn extract_subgraph(
        teg: &TemporalEffectGraph,
        effect_ids: &[EffectId],
        include_dependencies: bool,
        include_continuations: bool,
        include_resources: bool,
    ) -> TemporalEffectGraph {
        let mut subgraph = TemporalEffectGraph::new();
        let mut included_effects = HashSet::new();
        let mut included_resources = HashSet::new();
        
        // Add the specified effects
        for id in effect_ids {
            if let Some(effect) = teg.effect_nodes.get(id) {
                included_effects.insert(id.clone());
                subgraph.add_effect_node(effect.clone());
                
                // Add domain
                subgraph.domains.insert(effect.domain_id.clone());
                
                // Add accessed resources
                if include_resources {
                    for resource_id in &effect.resources_accessed {
                        if let Some(resource) = teg.resource_nodes.get(resource_id) {
                            included_resources.insert(resource_id.clone());
                            subgraph.add_resource_node(resource.clone());
                            
                            // Add domain
                            subgraph.domains.insert(resource.domain_id.clone());
                        }
                    }
                }
            }
        }
        
        // Add dependencies if requested
        if include_dependencies {
            for effect_id in &included_effects {
                if let Some(deps) = teg.effect_dependencies.get(effect_id) {
                    let mut subgraph_deps = Vec::new();
                    
                    for dep_id in deps {
                        if included_effects.contains(dep_id) {
                            subgraph_deps.push(dep_id.clone());
                        }
                    }
                    
                    if !subgraph_deps.is_empty() {
                        subgraph.effect_dependencies.insert(effect_id.clone(), subgraph_deps);
                    }
                }
            }
        }
        
        // Add continuations if requested
        if include_continuations {
            for effect_id in &included_effects {
                if let Some(conts) = teg.effect_continuations.get(effect_id) {
                    let mut subgraph_conts = Vec::new();
                    
                    for (cont_id, cond) in conts {
                        if included_effects.contains(cont_id) {
                            subgraph_conts.push((cont_id.clone(), cond.clone()));
                        }
                    }
                    
                    if !subgraph_conts.is_empty() {
                        subgraph.effect_continuations.insert(effect_id.clone(), subgraph_conts);
                    }
                }
            }
        }
        
        // Add resource relationships if requested
        if include_resources {
            for resource_id in &included_resources {
                if let Some(rels) = teg.resource_relationships.get(resource_id) {
                    let mut subgraph_rels = Vec::new();
                    
                    for (rel_id, rel_type) in rels {
                        if included_resources.contains(rel_id) {
                            subgraph_rels.push((rel_id.clone(), rel_type.clone()));
                        }
                    }
                    
                    if !subgraph_rels.is_empty() {
                        subgraph.resource_relationships.insert(resource_id.clone(), subgraph_rels);
                    }
                }
            }
        }
        
        subgraph
    }
    
    /// Filter a TEG by domain
    pub fn filter_by_domain(teg: &TemporalEffectGraph, domain_id: &DomainId) -> TemporalEffectGraph {
        let mut filtered = TemporalEffectGraph::new();
        
        // Add effects from the specified domain
        for (id, effect) in &teg.effect_nodes {
            if &effect.domain_id == domain_id {
                filtered.add_effect_node(effect.clone());
            }
        }
        
        // Add resources from the specified domain
        for (id, resource) in &teg.resource_nodes {
            if &resource.domain_id == domain_id {
                filtered.add_resource_node(resource.clone());
            }
        }
        
        // Add dependencies between effects in the domain
        for (effect_id, deps) in &teg.effect_dependencies {
            if filtered.effect_nodes.contains_key(effect_id) {
                let filtered_deps: Vec<EffectId> = deps.iter()
                    .filter(|dep_id| filtered.effect_nodes.contains_key(*dep_id))
                    .cloned()
                    .collect();
                
                if !filtered_deps.is_empty() {
                    filtered.effect_dependencies.insert(effect_id.clone(), filtered_deps);
                }
            }
        }
        
        // Add continuations between effects in the domain
        for (effect_id, conts) in &teg.effect_continuations {
            if filtered.effect_nodes.contains_key(effect_id) {
                let filtered_conts: Vec<(EffectId, Option<Condition>)> = conts.iter()
                    .filter(|(cont_id, _)| filtered.effect_nodes.contains_key(cont_id))
                    .cloned()
                    .collect();
                
                if !filtered_conts.is_empty() {
                    filtered.effect_continuations.insert(effect_id.clone(), filtered_conts);
                }
            }
        }
        
        // Add resource relationships between resources in the domain
        for (resource_id, rels) in &teg.resource_relationships {
            if filtered.resource_nodes.contains_key(resource_id) {
                let filtered_rels: Vec<(ResourceId, RelationshipType)> = rels.iter()
                    .filter(|(rel_id, _)| filtered.resource_nodes.contains_key(rel_id))
                    .cloned()
                    .collect();
                
                if !filtered_rels.is_empty() {
                    filtered.resource_relationships.insert(resource_id.clone(), filtered_rels);
                }
            }
        }
        
        // Add the domain
        filtered.domains.insert(domain_id.clone());
        
        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_transaction_add_effect() {
        let mut builder = GraphBuilder::new();
        
        // Create a simple TEG
        builder.add_effect("effect1", "domain1");
        
        let teg = builder.build().unwrap();
        let mut transaction = TEGTransaction::new(teg);
        
        // Create a new effect
        let effect = EffectNode::builder()
            .id("effect2")
            .effect_type("test")
            .domain("domain1")
            .build()
            .unwrap();
        
        // Add the effect
        let result = transaction.add_effect(effect);
        assert!(result.is_ok());
        
        // Commit the transaction
        let modified_teg = transaction.commit();
        
        // Verify the effect was added
        assert!(modified_teg.effect_nodes.contains_key("effect2"));
        assert_eq!(modified_teg.effect_nodes.len(), 2);
    }
    
    #[test]
    fn test_diff() {
        let mut builder1 = GraphBuilder::new();
        let mut builder2 = GraphBuilder::new();
        
        // Create original TEG
        builder1.add_effect("effect1", "domain1");
        builder1.add_effect("effect2", "domain1");
        builder1.add_resource("resource1", "type1");
        
        // Create modified TEG
        builder2.add_effect("effect1", "domain1");
        builder2.add_effect("effect3", "domain2");
        builder2.add_resource("resource1", "type1");
        builder2.add_resource("resource2", "type2");
        
        let original = builder1.build().unwrap();
        let modified = builder2.build().unwrap();
        
        let diff = TEGManipulator::diff(&original, &modified);
        
        // Verify diff
        assert_eq!(diff.added_effects.len(), 1);
        assert!(diff.added_effects.contains("effect3"));
        
        assert_eq!(diff.removed_effects.len(), 1);
        assert!(diff.removed_effects.contains("effect2"));
        
        assert_eq!(diff.added_resources.len(), 1);
        assert!(diff.added_resources.contains("resource2"));
    }
    
    #[test]
    fn test_merge() {
        let mut builder1 = GraphBuilder::new();
        let mut builder2 = GraphBuilder::new();
        
        // Create base TEG
        builder1.add_effect("effect1", "domain1");
        builder1.add_resource("resource1", "type1");
        
        // Create other TEG
        builder2.add_effect("effect2", "domain2");
        builder2.add_resource("resource2", "type2");
        
        let base = builder1.build().unwrap();
        let other = builder2.build().unwrap();
        
        let result = TEGManipulator::merge(&base, &other);
        assert!(result.is_ok());
        
        let merged = result.unwrap();
        
        // Verify merged TEG
        assert_eq!(merged.effect_nodes.len(), 2);
        assert!(merged.effect_nodes.contains_key("effect1"));
        assert!(merged.effect_nodes.contains_key("effect2"));
        
        assert_eq!(merged.resource_nodes.len(), 2);
        assert!(merged.resource_nodes.contains_key("resource1"));
        assert!(merged.resource_nodes.contains_key("resource2"));
        
        assert_eq!(merged.domains.len(), 2);
        assert!(merged.domains.contains("domain1"));
        assert!(merged.domains.contains("domain2"));
    }
    
    #[test]
    fn test_extract_subgraph() {
        let mut builder = GraphBuilder::new();
        
        // Create a more complex graph
        let effect1 = builder.add_effect("effect1", "domain1");
        let effect2 = builder.add_effect("effect2", "domain1");
        let effect3 = builder.add_effect("effect3", "domain2");
        
        let resource1 = builder.add_resource("resource1", "type1");
        
        builder.connect_effects(effect1, effect2);
        builder.connect_effects(effect2, effect3);
        
        builder.connect_effect_to_resource(effect2, resource1, crate::graph::edge::AccessMode::Read);
        
        let teg = builder.build().unwrap();
        
        // Extract a subgraph with just effect1 and effect2
        let subgraph = TEGManipulator::extract_subgraph(
            &teg,
            &[effect1.to_string(), effect2.to_string()],
            true,
            true,
            true,
        );
        
        // Verify the subgraph
        assert_eq!(subgraph.effect_nodes.len(), 2);
        assert!(subgraph.effect_nodes.contains_key(effect1));
        assert!(subgraph.effect_nodes.contains_key(effect2));
        assert!(!subgraph.effect_nodes.contains_key(effect3));
        
        assert_eq!(subgraph.resource_nodes.len(), 1);
        assert!(subgraph.resource_nodes.contains_key(resource1));
        
        assert!(subgraph.effect_continuations.contains_key(effect1));
        assert_eq!(subgraph.effect_continuations[effect1].len(), 1);
        assert_eq!(subgraph.effect_continuations[effect1][0].0, effect2);
    }
} 