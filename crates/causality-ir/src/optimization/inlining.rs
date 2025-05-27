//! Effect Inlining Optimization for TEG
//!
//! This module provides optimization passes that inline effect
//! applications to reduce overhead and enable further optimizations.

use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};

use crate::{
    TemporalEffectGraph, 
    EffectNode, 
    EffectId,
    effect_node::ParameterValue,
    graph::edge::{Edge, EdgeId, NodeId}
};
use super::{Optimization, OptimizationConfig};

/// Performs effect inlining on the TEG
///
/// This optimization inlines effect applications to:
/// 1. Reduce call overhead
/// 2. Enable more aggressive optimizations
/// 3. Improve performance by eliminating indirection
#[derive(Debug)]
pub struct EffectInlining {
    /// Metadata describing the optimization
    name: String,
    description: String,
    /// Maximum depth for recursive inlining
    max_depth: u32,
}

impl EffectInlining {
    /// Create a new EffectInlining optimization
    pub fn new() -> Self {
        Self {
            name: "effect_inlining".to_string(),
            description: "Inlines effect applications to reduce overhead".to_string(),
            max_depth: 3,
        }
    }
    
    /// Set the maximum inlining depth
    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }
    
    /// Find effect applications that can be inlined
    fn find_inlinable_effects(&self, teg: &TemporalEffectGraph) -> Vec<EffectId> {
        let mut result = Vec::new();
        
        for (effect_id, effect) in teg.effects() {
            // Check if this is an effect application
            if effect.operation_type() == Some("apply") {
                // Only inline if the effect is small enough
                if self.should_inline(teg, effect_id) {
                    result.push(effect_id.clone());
                }
            }
        }
        
        result
    }
    
    /// Determine if an effect should be inlined
    fn should_inline(&self, teg: &TemporalEffectGraph, effect_id: &EffectId) -> bool {
        let effect = match teg.get_effect(effect_id) {
            Some(e) => e,
            None => return false,
        };
        
        // Don't inline effects with side effects
        // TODO: Implement has_side_effects
        // if effect.has_side_effects() {
        //     return false;
        // }
        
        // Don't inline large effects
        // TODO: Implement estimate_effect_size or remove check
        // let size = self.estimate_effect_size(teg, effect_id);
        // if size > 50 {  // Arbitrary threshold
        //     return false;
        // }
        
        // Don't inline effects used multiple times
        // TODO: Implement count_outgoing_edges or remove check
        // let usage_count = teg.count_outgoing_edges(effect_id);
        // if usage_count > 1 {
        //     return false;
        // }
        
        true // Placeholder: Assume true for now
    }
    
    /// Estimate the size of an effect
    // TODO: Implement estimate_effect_size or remove
    /*
    fn estimate_effect_size(&self, teg: &TemporalEffectGraph, effect_id: &EffectId) -> usize {
        let mut size = 1;  // Start with the effect itself
        
        // Add size of all descendants
        // let descendants = teg.find_descendants(effect_id, |_| true); // Needs implementation
        // size += descendants.len();
        
        size
    }
    */
    
    /// Inline an effect application
    fn inline_effect(&self, teg: &mut TemporalEffectGraph, effect_id: &EffectId) -> Result<bool> {
        let _effect = match teg.get_effect(effect_id) {
            Some(e) => e.clone(),
            None => return Ok(false),
        };
        
        // Get the effect definition
        let definition_id = match self.get_effect_definition(teg, effect_id) {
            Some(id) => id,
            None => {
                // Effect application doesn't specify a definition ID via parameters
                return Ok(false);
            }
        };
        
        // Create a mapping of parameters
        let param_mapping = self.create_parameter_mapping(teg, effect_id, &definition_id)?;
        
        // Clone the effect subgraph
        let cloned_nodes = self.clone_effect_subgraph(teg, &definition_id, &param_mapping)?;
        
        // Connect the cloned subgraph to the original callers
        self.connect_cloned_subgraph(teg, effect_id, &cloned_nodes)?;
        
        // Remove the original effect application
        // TODO: Implement remove_effect
        // teg.remove_effect(effect_id)?;
        
        Ok(true)
    }
    
    /// Get the effect definition for an application
    fn get_effect_definition(&self, teg: &TemporalEffectGraph, effect_id: &EffectId) -> Option<EffectId> {
        let effect = teg.get_effect(effect_id)?;
        
        // Get the reference to the effect definition
        let params = effect.parameters();
        params.get("effect_id")
            .and_then(|param_value| match param_value {
                ParameterValue::String(s) => Some(s.clone()),
                _ => None,
            })
    }
    
    /// Create a mapping from definition parameters to application arguments
    fn create_parameter_mapping(&self, teg: &TemporalEffectGraph, 
                                application_id: &EffectId,
                                definition_id: &EffectId) -> Result<HashMap<String, ParameterValue>> {
        let mut mapping = HashMap::new();
        
        let application = teg.get_effect(application_id)
            .ok_or_else(|| anyhow!("Application effect not found"))?;
        
        let definition = teg.get_effect(definition_id)
            .ok_or_else(|| anyhow!("Definition effect not found"))?;
        
        // Get parameter definitions
        let params = definition.parameters();
        let args = application.parameters();
        
        // Map each parameter to its argument (cloning the ParameterValue)
        for (name, _) in params {
            if let Some(arg_value) = args.get(name) {
                mapping.insert(name.clone(), arg_value.clone());
            }
        }
        
        Ok(mapping)
    }
    
    /// Clone an effect subgraph with parameter substitution
    fn clone_effect_subgraph(&self, teg: &mut TemporalEffectGraph,
                            root_id: &EffectId,
                            param_mapping: &HashMap<String, ParameterValue>) -> Result<HashMap<EffectId, EffectId>> {
        let mut original_to_clone = HashMap::new();
        
        // TODO: Implement find_descendants or equivalent traversal
        // let subgraph_effects = teg.find_descendants(root_id, |_| true); // Needs implementation
        // let mut subgraph_effects_owned = subgraph_effects.into_iter().cloned().collect::<Vec<_>>(); // If find_descendants returns Vec<&EffectId>
        // subgraph_effects_owned.push(root_id.clone()); // Include the root
        let subgraph_effects_owned = vec![root_id.clone()]; // Placeholder: just the root for now

        // Clone each effect
        for original_id in &subgraph_effects_owned {
            let original = teg.get_effect(original_id)
                .ok_or_else(|| anyhow!("Effect not found in TEG during clone"))?
                .clone();
            
            // Create a clone using the builder
            let mut builder = EffectNode::builder()
                .effect_type(original.effect_type())
                .domain(original.domain_id().clone());

            // Apply parameter substitutions and add to builder
            for (param_name, param_value) in original.parameters() { 
                let substituted_value = match param_value {
                     ParameterValue::String(s) => {
                         param_mapping.get(s).cloned().unwrap_or_else(|| param_value.clone())
                     },
                     _ => param_value.clone(),
                };
                // Add parameter individually
                builder = builder.parameter(param_name.clone(), substituted_value);
            }
            
            // Copy metadata, adding inlined_from, and add to builder
            let mut metadata_map = original.metadata().clone(); 
            metadata_map.insert("inlined_from".to_string(), ParameterValue::String(original.name().to_string()));
            for (key, value) in metadata_map {
                // Add metadata individually
                builder = builder.metadata(key, value);
            }

            // Copy other fields
            for cap in &original.required_capabilities {
                 builder = builder.requires_capability(cap.clone());
            }
            for res in &original.resources_accessed {
                 builder = builder.accesses_resource(res.clone());
            }
             for fact in &original.fact_dependencies {
                 builder = builder.depends_on_fact(fact.clone());
             }
             
             // Build the cloned node
             let clone = builder.build().map_err(|e| anyhow!("Failed to build cloned effect node: {}", e))?; 
            
            // Add to graph using add_effect_node
            let clone_id = teg.add_effect_node(clone); 
            original_to_clone.insert(original_id.clone(), clone_id);
        }
        
        // Clone edges between the effects
        // TODO: Implement get_outgoing_edges and add_edge
        /*
        for original_id in &subgraph_effects_owned {
            if let Some(outgoing_edges) = teg.get_outgoing_edges(original_id) { // Needs implementation
                for edge in outgoing_edges {
                    // Skip edges to effects outside the subgraph
                    if !original_to_clone.contains_key(&edge.target_id) {
                        continue;
                    }
                    
                    let src_clone = original_to_clone[original_id].clone();
                    let dst_clone = original_to_clone[&edge.target_id].clone();
                    
                    teg.add_edge(src_clone, dst_clone, edge.clone())?; // Needs implementation
                }
            }
        }
        */
        
        Ok(original_to_clone)
    }
    
    /// Connect the cloned subgraph to the original callers
    fn connect_cloned_subgraph(&self, teg: &mut TemporalEffectGraph,
                              application_id: &EffectId,
                              cloned_nodes: &HashMap<EffectId, EffectId>) -> Result<()> {
        // Get incoming edges to the application
        // TODO: Implement get_incoming_edges
        // let incoming = teg.get_incoming_edges(application_id);
        let incoming: Vec<(EffectId, Edge)> = Vec::new(); // Placeholder
        
        // Get outgoing edges from the application
        // TODO: Implement get_outgoing_edges
        // let outgoing = teg.get_outgoing_edges(application_id);
        let outgoing: Vec<(EffectId, Edge)> = Vec::new(); // Placeholder
        
        // Connect incoming edges to the entry point(s) of the cloned subgraph
        // Find entry points (cloned nodes with no incoming edges *from other cloned nodes*)
        let entry_points: Vec<EffectId> = cloned_nodes.values()
            .filter(|&clone_id| {
                // TODO: Implement check for incoming edges from other cloned nodes
                // !cloned_nodes.values().any(|&other_clone_id| teg.has_edge(other_clone_id, clone_id))
                true // Placeholder: connect all incoming to all cloned nodes for now
            })
            .cloned()
            .collect();

        for (src, edge_data) in incoming {
            for entry_point_id in &entry_points {
                // TODO: Implement add_edge
                // teg.add_edge(src.clone(), entry_point_id.clone(), edge_data.clone())?;
            }
        }

        // Connect exit points of the cloned subgraph to the original outgoing targets
        // Find exit points (cloned nodes with no outgoing edges *to other cloned nodes*)
        let exit_points: Vec<EffectId> = cloned_nodes.values()
            .filter(|&clone_id| {
                // TODO: Implement check for outgoing edges to other cloned nodes
                // !cloned_nodes.values().any(|&other_clone_id| teg.has_edge(clone_id, other_clone_id))
                true // Placeholder: connect all exit points to all original outgoing targets
            })
            .cloned()
            .collect();

        for (dst, edge_data) in outgoing {
            for exit_point_id in &exit_points {
                 // TODO: Implement add_edge
                // teg.add_edge(exit_point_id.clone(), dst.clone(), edge_data.clone())?;
            }
        }

        Ok(())
    }
}

impl Optimization for EffectInlining {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
    
    fn apply(&self, teg: &mut TemporalEffectGraph, config: &OptimizationConfig) -> Result<bool> {
        let mut changed = false;
        
        // Skip if optimization level is too low
        if config.level < 2 {
            return Ok(false);
        }
        
        // Find effects that can be inlined
        let inlinable_effects = self.find_inlinable_effects(teg);
        
        // Inline each effect
        for effect_id in inlinable_effects {
            if self.inline_effect(teg, &effect_id)? {
                changed = true;
            }
        }
        
        Ok(changed)
    }

    fn preserves_adjunction(&self) -> bool {
        // Inlining preserves semantics as it substitutes
        // the effect definition for its application
        true
    }

    fn preserves_resource_structure(&self) -> bool {
        // Inlining preserves resource structure as it doesn't
        // change resource access patterns
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_inlining() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create an effect definition
        let param = teg.add_effect("param", "param");
        let body = teg.add_effect("body", "compute");
        let result = teg.add_effect("result", "result");
        
        teg.connect_effects(param, body);
        teg.connect_effects(body, result);
        
        // Create an effect application
        let arg = teg.add_effect("arg", "compute");
        let apply = teg.add_effect_with_params(
            "apply", 
            "apply",
            [("effect_id", result.to_string())].iter().cloned().collect()
        );
        let output = teg.add_effect("output", "output");
        
        teg.connect_effects(arg, apply);
        teg.connect_effects(apply, output);
        
        // Apply optimization
        let opt = EffectInlining::new();
        let config = OptimizationConfig {
            level: 2,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that inlining occurred
        assert!(result);
        
        // The application should be gone
        assert!(teg.get_effect(apply).is_none());
        
        // There should be new inlined nodes
        let inlined_effects: Vec<_> = teg.effects()
            .iter()
            .filter(|(_, e)| e.name().starts_with("inline_"))
            .collect();
            
        assert!(!inlined_effects.is_empty());
        
        // The arg should now connect to the inlined body
        let path_exists = inlined_effects.iter().any(|(id, _)| {
            teg.has_path(arg, *id)
        });
        
        assert!(path_exists);
        
        // The inlined effect should connect to the output
        let path_to_output = inlined_effects.iter().any(|(id, _)| {
            teg.has_path(*id, output)
        });
        
        assert!(path_to_output);
    }
    
    #[test]
    fn test_no_inline_side_effects() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create an effect definition with side effects
        let param = teg.add_effect("param", "param");
        let body = teg.add_effect("body", "compute");
        let side_effect = teg.add_effect("side_effect", "io");
        let result = teg.add_effect("result", "result");
        
        teg.connect_effects(param, body);
        teg.connect_effects(body, side_effect);
        teg.connect_effects(side_effect, result);
        
        // Mark as having side effects
        teg.mark_as_side_effect(side_effect);
        
        // Create an effect application
        let arg = teg.add_effect("arg", "compute");
        let apply = teg.add_effect_with_params(
            "apply", 
            "apply",
            [("effect_id", result.to_string())].iter().cloned().collect()
        );
        let output = teg.add_effect("output", "output");
        
        teg.connect_effects(arg, apply);
        teg.connect_effects(apply, output);
        
        // Apply optimization
        let opt = EffectInlining::new();
        let config = OptimizationConfig {
            level: 2,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Effect should not be inlined due to side effects
        assert!(!result);
        assert!(teg.get_effect(apply).is_some());
    }
}
