//! Dead Code Elimination Optimization for TEG
//!
//! This module provides optimization passes that eliminate unused
//! or unreachable code in the Temporal Effect Graph.

use anyhow::Result;
use std::collections::{HashSet, VecDeque};

use crate::{TemporalEffectGraph, EffectId, ResourceId};
use super::{Optimization, OptimizationConfig};

/// Performs dead code elimination on the TEG
///
/// This optimization identifies and removes:
/// 1. Effects that don't contribute to output results
/// 2. Unreachable code paths
/// 3. Unused resource declarations
#[derive(Debug)]
pub struct DeadCodeElimination {
    /// Metadata describing the optimization
    name: String,
    description: String,
}

impl DeadCodeElimination {
    /// Create a new DeadCodeElimination optimization
    pub fn new() -> Self {
        Self {
            name: "dead_code_elimination".to_string(),
            description: "Eliminates unused or unreachable code".to_string(),
        }
    }
    
    /// Find live effect nodes by marking from outputs and side effects
    fn find_live_effects(&self, teg: &TemporalEffectGraph) -> HashSet<EffectId> {
        let mut live_effects = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with all output nodes and nodes with side effects
        for (effect_id, effect) in teg.effects() {
            // Include output nodes
            if teg.is_output(effect_id) {
                live_effects.insert(effect_id);
                queue.push_back(effect_id);
            }
            
            // Include nodes with side effects that can't be eliminated
            if effect.has_side_effects() && !effect.is_pure() {
                live_effects.insert(effect_id);
                queue.push_back(effect_id);
            }
        }
        
        // Traverse backwards to mark all nodes that contribute to live nodes
        while let Some(effect_id) = queue.pop_front() {
            // Get all predecessors
            let predecessors = teg.get_incoming_edges(effect_id);
            
            for (pred_id, _) in predecessors {
                // If we haven't seen this node yet, mark it and add to queue
                if !live_effects.contains(&pred_id) {
                    live_effects.insert(pred_id);
                    queue.push_back(pred_id);
                }
            }
        }
        
        live_effects
    }
    
    /// Find resources used by live effects
    fn find_live_resources(&self, teg: &TemporalEffectGraph, live_effects: &HashSet<EffectId>) -> HashSet<ResourceId> {
        let mut live_resources = HashSet::new();
        
        // Check each live effect for resource usage
        for &effect_id in live_effects {
            if let Some(effect) = teg.get_effect(effect_id) {
                for (resource_id, _) in effect.resource_edges() {
                    live_resources.insert(*resource_id);
                }
            }
        }
        
        live_resources
    }
}

impl Optimization for DeadCodeElimination {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
    
    fn apply(&self, teg: &mut TemporalEffectGraph, config: &OptimizationConfig) -> Result<bool> {
        let mut changed = false;
        
        // Skip if optimization level is too low
        if config.level < 1 {
            return Ok(false);
        }
        
        // Find all live effects
        let live_effects = self.find_live_effects(teg);
        
        // Remove dead effects
        let all_effects: Vec<EffectId> = teg.effects().keys().copied().collect();
        for effect_id in all_effects {
            if !live_effects.contains(&effect_id) {
                teg.remove_effect(effect_id)?;
                changed = true;
            }
        }
        
        // Find live resources
        let live_resources = self.find_live_resources(teg, &live_effects);
        
        // Remove unused resources
        let all_resources: Vec<ResourceId> = teg.resources().keys().copied().collect();
        for resource_id in all_resources {
            if !live_resources.contains(&resource_id) {
                teg.remove_resource(resource_id)?;
                changed = true;
            }
        }
        
        Ok(changed)
    }

    fn preserves_adjunction(&self) -> bool {
        // Dead code elimination preserves semantics as it only removes
        // code that doesn't affect the program's output
        true
    }

    fn preserves_resource_structure(&self) -> bool {
        // Dead code elimination preserves the resource structure of live code
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_dead_code_elimination() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create resources
        let resource1 = graph_builder.add_resource("resource1", "test_type");
        let resource2 = graph_builder.add_resource("resource2", "test_type"); // Will be unused
        
        // Create effects
        let input = graph_builder.add_effect("input", "input");
        let process1 = graph_builder.add_effect("process1", "process");
        let process2 = graph_builder.add_effect("process2", "process"); // Will be unused
        let output = graph_builder.add_effect("output", "output");
        
        // Connect effects
        graph_builder.connect_effects(input, process1);
        graph_builder.connect_effects(process1, output);
        
        // Connect process2 but not to any output
        graph_builder.connect_effects(input, process2);
        
        // Connect resources
        graph_builder.connect_effect_to_resource(process1, resource1, crate::graph::edge::AccessMode::Read);
        
        // Mark output as an output node
        graph_builder.mark_as_output(output);
        
        let mut teg = graph_builder.build().unwrap();
        
        // Apply optimization
        let opt = DeadCodeElimination::new();
        let config = OptimizationConfig {
            level: 1,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that dead code was eliminated
        assert!(result);
        
        // Check that unused effects were removed
        assert!(teg.get_effect(input).is_some());
        assert!(teg.get_effect(process1).is_some());
        assert!(teg.get_effect(process2).is_none()); // Should be removed
        assert!(teg.get_effect(output).is_some());
        
        // Check that unused resources were removed
        assert!(teg.get_resource(resource1).is_some());
        assert!(teg.get_resource(resource2).is_none()); // Should be removed
    }
    
    #[test]
    fn test_side_effect_preservation() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create effects
        let input = graph_builder.add_effect("input", "input");
        
        // Create a process with side effects
        let mut side_effect = graph_builder.add_effect("side_effect", "process");
        graph_builder.mark_as_side_effect(side_effect);
        
        // Create another process that doesn't lead to output or side effects
        let unused = graph_builder.add_effect("unused", "process");
        
        // Connect effects
        graph_builder.connect_effects(input, side_effect);
        graph_builder.connect_effects(input, unused);
        
        let mut teg = graph_builder.build().unwrap();
        
        // Apply optimization
        let opt = DeadCodeElimination::new();
        let config = OptimizationConfig {
            level: 1,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that dead code was eliminated
        assert!(result);
        
        // The process with side effects should be preserved
        assert!(teg.get_effect(input).is_some());
        assert!(teg.get_effect(side_effect).is_some());
        assert!(teg.get_effect(unused).is_none()); // Should be removed
    }
}
