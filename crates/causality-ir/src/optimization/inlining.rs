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
    graph::edge::EdgeData
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
                    result.push(effect_id);
                }
            }
        }
        
        result
    }
    
    /// Determine if an effect should be inlined
    fn should_inline(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> bool {
        let effect = match teg.get_effect(effect_id) {
            Some(e) => e,
            None => return false,
        };
        
        // Don't inline effects with side effects
        if effect.has_side_effects() {
            return false;
        }
        
        // Don't inline large effects
        let size = self.estimate_effect_size(teg, effect_id);
        if size > 50 {  // Arbitrary threshold
            return false;
        }
        
        // Don't inline effects used multiple times
        let usage_count = teg.count_outgoing_edges(effect_id);
        if usage_count > 1 {
            return false;
        }
        
        true
    }
    
    /// Estimate the size of an effect
    fn estimate_effect_size(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> usize {
        let mut size = 1;  // Start with the effect itself
        
        // Add size of all descendants
        let descendants = teg.find_descendants(effect_id, |_| true);
        size += descendants.len();
        
        size
    }
    
    /// Inline an effect application
    fn inline_effect(&self, teg: &mut TemporalEffectGraph, effect_id: EffectId) -> Result<bool> {
        let effect = match teg.get_effect(effect_id) {
            Some(e) => e.clone(),
            None => return Ok(false),
        };
        
        // Get the effect definition
        let definition_id = match self.get_effect_definition(teg, effect_id) {
            Some(id) => id,
            None => return Ok(false),
        };
        
        // Create a mapping of parameters
        let param_mapping = self.create_parameter_mapping(teg, effect_id, definition_id)?;
        
        // Clone the effect subgraph
        let cloned_nodes = self.clone_effect_subgraph(teg, definition_id, &param_mapping)?;
        
        // Connect the cloned subgraph to the original callers
        self.connect_cloned_subgraph(teg, effect_id, &cloned_nodes)?;
        
        // Remove the original effect application
        teg.remove_effect(effect_id)?;
        
        Ok(true)
    }
    
    /// Get the effect definition for an application
    fn get_effect_definition(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> Option<EffectId> {
        let effect = teg.get_effect(effect_id)?;
        
        // Get the reference to the effect definition
        let params = effect.parameters();
        params.get("effect_id")
            .and_then(|id_str| id_str.parse::<EffectId>().ok())
    }
    
    /// Create a mapping from definition parameters to application arguments
    fn create_parameter_mapping(&self, teg: &TemporalEffectGraph, 
                                application_id: EffectId, 
                                definition_id: EffectId) -> Result<HashMap<String, String>> {
        let mut mapping = HashMap::new();
        
        let application = teg.get_effect(application_id)
            .ok_or_else(|| anyhow!("Application effect not found"))?;
        
        let definition = teg.get_effect(definition_id)
            .ok_or_else(|| anyhow!("Definition effect not found"))?;
        
        // Get parameter definitions
        let params = definition.parameters();
        let args = application.parameters();
        
        // Map each parameter to its argument
        for (name, _) in params {
            if let Some(arg_value) = args.get(name) {
                mapping.insert(name.clone(), arg_value.clone());
            }
        }
        
        Ok(mapping)
    }
    
    /// Clone an effect subgraph with parameter substitution
    fn clone_effect_subgraph(&self, teg: &mut TemporalEffectGraph,
                            root_id: EffectId,
                            param_mapping: &HashMap<String, String>) -> Result<HashMap<EffectId, EffectId>> {
        let mut original_to_clone = HashMap::new();
        let mut visited = HashSet::new();
        
        // Get all effects in the subgraph
        let subgraph_effects = teg.find_descendants(root_id, |_| true);
        subgraph_effects.push(root_id); // Include the root
        
        // Clone each effect
        for &original_id in &subgraph_effects {
            let original = teg.get_effect(original_id)
                .ok_or_else(|| anyhow!("Effect not found"))?
                .clone();
            
            // Create a clone with substituted parameters
            let mut clone = EffectNode::new(
                format!("inline_{}", original.name()),
                original.operation_type().unwrap_or_default(),
            );
            
            // Apply parameter substitutions
            let mut new_params = original.parameters().clone();
            for (param_name, param_value) in &new_params {
                if param_mapping.contains_key(param_value) {
                    new_params.insert(param_name.clone(), param_mapping[param_value].clone());
                }
            }
            clone.set_parameters(new_params);
            
            // Copy metadata
            let mut metadata = original.metadata().clone();
            metadata.insert("inlined_from".to_string(), original.name().to_string());
            clone.set_metadata(metadata);
            
            // Add to graph
            let clone_id = teg.add_effect(clone)?;
            original_to_clone.insert(original_id, clone_id);
        }
        
        // Clone edges between the effects
        for &original_id in &subgraph_effects {
            let outgoing = teg.get_outgoing_edges(original_id);
            
            for (dst, edge_data) in outgoing {
                // Skip edges to effects outside the subgraph
                if !original_to_clone.contains_key(&dst) {
                    continue;
                }
                
                let src_clone = original_to_clone[&original_id];
                let dst_clone = original_to_clone[&dst];
                
                teg.add_edge(src_clone, dst_clone, edge_data.clone())?;
            }
        }
        
        Ok(original_to_clone)
    }
    
    /// Connect the cloned subgraph to the original callers
    fn connect_cloned_subgraph(&self, teg: &mut TemporalEffectGraph,
                              application_id: EffectId,
                              cloned_nodes: &HashMap<EffectId, EffectId>) -> Result<()> {
        // Get incoming edges to the application
        let incoming = teg.get_incoming_edges(application_id);
        
        // Get outgoing edges from the application
        let outgoing = teg.get_outgoing_edges(application_id);
        
        // Connect incoming edges to the entry point of the cloned subgraph
        for (src, edge_data) in incoming {
            for &clone_id in cloned_nodes.values() {
                // Check if this is an entry point (no incoming edges within the subgraph)
                let has_internal_incoming = cloned_nodes.values().any(|&id| {
                    teg.has_edge(id, clone_id)
                });
                
                if !has_internal_incoming {
                    teg.add_edge(src, clone_id, edge_data.clone())?;
                }
            }
        }
        
        // Connect outgoing edges to the exit points of the cloned subgraph
        for (dst, edge_data) in outgoing {
            for &clone_id in cloned_nodes.values() {
                // Check if this is an exit point (no outgoing edges within the subgraph)
                let has_internal_outgoing = cloned_nodes.values().any(|&id| {
                    teg.has_edge(clone_id, id)
                });
                
                if !has_internal_outgoing {
                    teg.add_edge(clone_id, dst, edge_data.clone())?;
                }
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
            if self.inline_effect(teg, effect_id)? {
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
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_effect_inlining() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create an effect definition
        let param = graph_builder.add_effect("param", "param");
        let body = graph_builder.add_effect("body", "compute");
        let result = graph_builder.add_effect("result", "result");
        
        graph_builder.connect_effects(param, body);
        graph_builder.connect_effects(body, result);
        
        // Create an effect application
        let arg = graph_builder.add_effect("arg", "compute");
        let apply = graph_builder.add_effect_with_params(
            "apply", 
            "apply",
            [("effect_id", result.to_string())].iter().cloned().collect()
        );
        let output = graph_builder.add_effect("output", "output");
        
        graph_builder.connect_effects(arg, apply);
        graph_builder.connect_effects(apply, output);
        
        let mut teg = graph_builder.build().unwrap();
        
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
        let mut graph_builder = GraphBuilder::new();
        
        // Create an effect definition with side effects
        let param = graph_builder.add_effect("param", "param");
        let body = graph_builder.add_effect("body", "compute");
        let side_effect = graph_builder.add_effect("side_effect", "io");
        let result = graph_builder.add_effect("result", "result");
        
        graph_builder.connect_effects(param, body);
        graph_builder.connect_effects(body, side_effect);
        graph_builder.connect_effects(side_effect, result);
        
        // Mark as having side effects
        graph_builder.mark_as_side_effect(side_effect);
        
        // Create an effect application
        let arg = graph_builder.add_effect("arg", "compute");
        let apply = graph_builder.add_effect_with_params(
            "apply", 
            "apply",
            [("effect_id", result.to_string())].iter().cloned().collect()
        );
        let output = graph_builder.add_effect("output", "output");
        
        graph_builder.connect_effects(arg, apply);
        graph_builder.connect_effects(apply, output);
        
        let mut teg = graph_builder.build().unwrap();
        
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
