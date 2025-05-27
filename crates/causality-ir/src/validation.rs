// Validation module for the Temporal Effect Graph
// This module implements robust validators for ensuring TEG integrity and correctness.

use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use crate::{TemporalEffectGraph, EffectId, ResourceId};
use crate::graph::edge::{TemporalRelation, RelationshipType};

/// Validate a TEG for basic structural integrity
pub fn validate_teg_structure(teg: &TemporalEffectGraph) -> Result<()> {
    // Check that effect nodes exist for all referenced effect IDs
    for (effect_id, _) in &teg.effect_dependencies {
        if !teg.effect_nodes.contains_key(effect_id) {
            return Err(anyhow!("Referenced effect ID {} not found in effect_nodes", effect_id));
        }
    }
    
    // Check that resource nodes exist for all referenced resource IDs
    for (resource_id, _) in &teg.resource_relationships {
        if !teg.resource_nodes.contains_key(resource_id) {
            return Err(anyhow!("Referenced resource ID {} not found in resource_nodes", resource_id));
        }
    }
    
    // Verify all edges reference valid nodes
    for (_, edge) in &teg.edges {
        match &edge.source {
            crate::graph::edge::NodeId::Effect(id) => {
                if !teg.effect_nodes.contains_key(id) {
                    return Err(anyhow!("Edge source effect ID {} not found", id));
                }
            },
            crate::graph::edge::NodeId::Resource(id) => {
                if !teg.resource_nodes.contains_key(id) {
                    return Err(anyhow!("Edge source resource ID {} not found", id));
                }
            },
        }
        
        match &edge.target {
            crate::graph::edge::NodeId::Effect(id) => {
                if !teg.effect_nodes.contains_key(id) {
                    return Err(anyhow!("Edge target effect ID {} not found", id));
                }
            },
            crate::graph::edge::NodeId::Resource(id) => {
                if !teg.resource_nodes.contains_key(id) {
                    return Err(anyhow!("Edge target resource ID {} not found", id));
                }
            },
        }
    }
    
    // Verify content hashes are not empty/default
    for (id, effect) in &teg.effect_nodes {
        if effect.content_hash.as_bytes().is_empty() {
            return Err(anyhow!("Effect node {} has empty content hash", id));
        }
    }
    
    for (id, resource) in &teg.resource_nodes {
        if resource.content_hash.as_bytes().is_empty() {
            return Err(anyhow!("Resource node {} has empty content hash", id));
        }
    }
    
    Ok(())
}

/// Validate that all effect dependencies are satisfied
pub fn validate_effect_dependencies(teg: &TemporalEffectGraph) -> Result<()> {
    for (effect_id, dependencies) in &teg.effect_dependencies {
        // Check each dependency exists
        for dep_id in dependencies {
            if !teg.effect_nodes.contains_key(dep_id) {
                return Err(anyhow!("Effect {} depends on non-existent effect {}", effect_id, dep_id));
            }
        }
        
        // Check for circular dependencies
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        fn has_cycle(
            current: &EffectId, 
            visited: &mut HashSet<EffectId>, 
            stack: &mut Vec<EffectId>,
            dependencies: &HashMap<EffectId, Vec<EffectId>>
        ) -> bool {
            if stack.contains(current) {
                return true;
            }
            
            if visited.contains(current) {
                return false;
            }
            
            visited.insert(current.clone());
            stack.push(current.clone());
            
            if let Some(deps) = dependencies.get(current) {
                for dep in deps {
                    if has_cycle(dep, visited, stack, dependencies) {
                        return true;
                    }
                }
            }
            
            stack.pop();
            false
        }
        
        if has_cycle(effect_id, &mut visited, &mut stack, &teg.effect_dependencies) {
            return Err(anyhow!("Circular dependency detected starting from effect {}", effect_id));
        }
    }
    
    Ok(())
}

/// Validate that all resources are properly referenced
pub fn validate_resource_references(teg: &TemporalEffectGraph) -> Result<()> {
    // Check all accessed resources actually exist
    for (effect_id, effect) in &teg.effect_nodes {
        for resource_id in &effect.resources_accessed {
            if !teg.resource_nodes.contains_key(resource_id) {
                return Err(anyhow!("Effect {} accesses non-existent resource {}", effect_id, resource_id));
            }
        }
    }
    
    // Check all resource relationships reference valid resources
    for (resource_id, relationships) in &teg.resource_relationships {
        for (related_id, _) in relationships {
            if !teg.resource_nodes.contains_key(related_id) {
                return Err(anyhow!("Resource {} has relationship with non-existent resource {}", 
                                  resource_id, related_id));
            }
        }
    }
    
    // Check for contradictory relationships
    for (resource_id, relationships) in &teg.resource_relationships {
        for (related_id, rel_type) in relationships {
            match rel_type {
                RelationshipType::ParentChild => {
                    // If A contains B, B shouldn't contain A
                    if let Some(reverse_rels) = teg.resource_relationships.get(related_id) {
                        for (rev_rel_id, rev_rel_type) in reverse_rels {
                            if rev_rel_id == resource_id && *rev_rel_type == RelationshipType::ParentChild {
                                return Err(anyhow!("Contradictory ParentChild relationship between {} and {}", 
                                                 resource_id, related_id));
                            }
                        }
                    }
                },
                RelationshipType::Dependency => {
                    // Check for circular dependencies (A depends on B, B depends on A)
                    let mut visited = HashSet::new();
                    let mut stack = Vec::new();
                    
                    fn has_cycle(
                        current: &ResourceId,
                        target: &ResourceId,
                        visited: &mut HashSet<ResourceId>,
                        stack: &mut Vec<ResourceId>,
                        relationships: &HashMap<ResourceId, Vec<(ResourceId, RelationshipType)>>
                    ) -> bool {
                        if stack.contains(current) {
                            return false; // Already in process
                        }
                        
                        if visited.contains(current) {
                            return false; // Already checked
                        }
                        
                        visited.insert(current.clone());
                        stack.push(current.clone());
                        
                        if let Some(rels) = relationships.get(current) {
                            for (rel_id, rel_type) in rels {
                                if *rel_type == RelationshipType::Dependency {
                                    if rel_id == target {
                                        return true; // Found cycle
                                    }
                                    
                                    if has_cycle(rel_id, target, visited, stack, relationships) {
                                        return true;
                                    }
                                }
                            }
                        }
                        
                        stack.pop();
                        false
                    }
                    
                    if has_cycle(related_id, resource_id, &mut visited, &mut stack, &teg.resource_relationships) {
                        return Err(anyhow!("Circular dependency detected between resources {} and {}", 
                                         resource_id, related_id));
                    }
                },
                _ => {} // Other relationship types don't have explicit contradictions
            }
        }
    }
    
    Ok(())
}

/// Validate temporal constraints for consistency
pub fn validate_temporal_constraints(teg: &TemporalEffectGraph) -> Result<()> {
    // Check all effects in constraints exist
    for (effect_id, constraints) in &teg.temporal_constraints {
        if !teg.effect_nodes.contains_key(effect_id) {
            return Err(anyhow!("Temporal constraint references non-existent effect {}", effect_id));
        }
        
        for constraint in constraints {
            if !teg.effect_nodes.contains_key(&constraint.source) {
                return Err(anyhow!("Temporal constraint source effect {} doesn't exist", constraint.source));
            }
            
            if !teg.effect_nodes.contains_key(&constraint.target) {
                return Err(anyhow!("Temporal constraint target effect {} doesn't exist", constraint.target));
            }
        }
    }
    
    // Check for contradictory temporal constraints
    let mut all_constraints = Vec::new();
    for constraints in teg.temporal_constraints.values() {
        all_constraints.extend(constraints.iter().map(|c| c.clone()));
    }
    
    for i in 0..all_constraints.len() {
        for j in (i+1)..all_constraints.len() {
            let a = &all_constraints[i];
            let b = &all_constraints[j];
            
            // Check for direct contradictions
            if a.source == b.target && a.target == b.source {
                match (a.relation.clone(), b.relation.clone()) {
                    (TemporalRelation::Before, TemporalRelation::Before) => {
                        return Err(anyhow!("Contradictory Before relations between {} and {}", 
                                        a.source, a.target));
                    },
                    (TemporalRelation::After, TemporalRelation::After) => {
                        return Err(anyhow!("Contradictory After relations between {} and {}", 
                                        a.source, a.target));
                    },
                    _ => {} // Other combinations might be valid
                }
            }
            
            // Check for cycle (A before B, B before C, C before A)
            // This would require more complex cycle detection in the constraint graph
        }
    }
    
    Ok(())
}

/// Validate capability requirements for all effects
pub fn validate_capability_requirements(teg: &TemporalEffectGraph) -> Result<()> {
    // Verify that all capability authorizations reference valid effects
    for (effect_id, _) in &teg.capability_authorizations {
        if !teg.effect_nodes.contains_key(effect_id) {
            return Err(anyhow!("Capability authorization for non-existent effect {}", effect_id));
        }
    }
    
    // Check that all effects with required capabilities have authorizations
    for (effect_id, effect) in &teg.effect_nodes {
        if !effect.required_capabilities.is_empty() {
            if !teg.capability_authorizations.contains_key(effect_id) {
                return Err(anyhow!("Effect {} requires capabilities but has no authorizations", effect_id));
            }
            
            // Check all required capabilities are covered by authorizations
            let empty_vec = Vec::new();
            let authorizations = teg.capability_authorizations.get(effect_id)
                                   .unwrap_or(&empty_vec);
            
            for required in &effect.required_capabilities {
                if !authorizations.contains(required) {
                    return Err(anyhow!("Effect {} requires capability {} but it's not authorized", 
                                     effect_id, required));
                }
            }
        }
    }
    
    Ok(())
}

/// Run all validators on a TEG
pub fn validate_teg(teg: &TemporalEffectGraph) -> Result<()> {
    validate_teg_structure(teg)?;
    validate_effect_dependencies(teg)?;
    validate_resource_references(teg)?;
    validate_temporal_constraints(teg)?;
    validate_capability_requirements(teg)?;
    Ok(())
}
