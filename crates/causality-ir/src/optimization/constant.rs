//! Constant Folding Optimization for TEG
//!
//! This module provides optimization passes that fold constant values
//! and simplify expressions in the Temporal Effect Graph.

use anyhow::Result;
use std::collections::HashMap;

use crate::{TemporalEffectGraph, EffectNode, EffectId};
use super::{Optimization, OptimizationConfig};

/// Performs constant folding optimizations on the TEG
///
/// This optimization identifies constant expressions and evaluates them at compile time:
/// 1. Replaces constant expressions with their computed values
/// 2. Simplifies expressions with constant operands
/// 3. Propagates constants through the graph
#[derive(Debug)]
pub struct ConstantFolding {
    /// Metadata describing the optimization
    name: String,
    description: String,
}

impl ConstantFolding {
    /// Create a new ConstantFolding optimization
    pub fn new() -> Self {
        Self {
            name: "constant_folding".to_string(),
            description: "Evaluates constant expressions at compile time".to_string(),
        }
    }
    
    /// Find constant effect nodes in the TEG
    fn find_constant_effects(&self, teg: &TemporalEffectGraph) -> Vec<EffectId> {
        let mut constants = Vec::new();
        
        for (effect_id, effect) in teg.effects() {
            // Check if this is a literal or constant computation
            if effect.is_constant() || self.can_be_evaluated_at_compile_time(teg, effect_id) {
                constants.push(effect_id);
            }
        }
        
        constants
    }
    
    /// Check if an effect can be evaluated at compile time
    fn can_be_evaluated_at_compile_time(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> bool {
        let effect = match teg.get_effect(effect_id) {
            Some(e) => e,
            None => return false,
        };
        
        // Check if the effect is a pure computation
        if !effect.is_pure() {
            return false;
        }
        
        // Check if all inputs are constants
        let predecessors = teg.get_incoming_edges(effect_id);
        for (pred_id, _) in predecessors {
            let pred = match teg.get_effect(pred_id) {
                Some(e) => e,
                None => return false,
            };
            
            if !pred.is_constant() && !self.can_be_evaluated_at_compile_time(teg, pred_id) {
                return false;
            }
        }
        
        true
    }
    
    /// Evaluate a constant expression
    fn evaluate_constant(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> Option<String> {
        let effect = teg.get_effect(effect_id)?;
        
        // If it's already a constant, return its value
        if effect.is_constant() {
            return Some(effect.constant_value().unwrap_or_default());
        }
        
        // Otherwise, evaluate based on operation type
        let op_type = effect.operation_type()?;
        
        match op_type {
            "add" | "subtract" | "multiply" | "divide" => {
                self.evaluate_arithmetic(teg, effect_id, op_type)
            },
            "concat" => self.evaluate_string_operation(teg, effect_id, op_type),
            "and" | "or" | "not" => self.evaluate_boolean_operation(teg, effect_id, op_type),
            _ => None,
        }
    }
    
    /// Evaluate arithmetic operations
    fn evaluate_arithmetic(&self, teg: &TemporalEffectGraph, effect_id: EffectId, op_type: &str) -> Option<String> {
        // Get input values
        let inputs = self.get_input_values(teg, effect_id)?;
        if inputs.len() < 2 {
            return None;
        }
        
        // Parse input values as numbers
        let a = inputs[0].parse::<f64>().ok()?;
        let b = inputs[1].parse::<f64>().ok()?;
        
        // Perform operation
        let result = match op_type {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return None; // Avoid division by zero
                }
                a / b
            },
            _ => return None,
        };
        
        Some(result.to_string())
    }
    
    /// Evaluate string operations
    fn evaluate_string_operation(&self, teg: &TemporalEffectGraph, effect_id: EffectId, op_type: &str) -> Option<String> {
        // Get input values
        let inputs = self.get_input_values(teg, effect_id)?;
        
        match op_type {
            "concat" => {
                // Join all input strings
                Some(inputs.join(""))
            },
            _ => None,
        }
    }
    
    /// Evaluate boolean operations
    fn evaluate_boolean_operation(&self, teg: &TemporalEffectGraph, effect_id: EffectId, op_type: &str) -> Option<String> {
        // Get input values
        let inputs = self.get_input_values(teg, effect_id)?;
        
        match op_type {
            "and" => {
                // Check if all inputs are true
                let all_true = inputs.iter().all(|v| v == "true");
                Some(all_true.to_string())
            },
            "or" => {
                // Check if any input is true
                let any_true = inputs.iter().any(|v| v == "true");
                Some(any_true.to_string())
            },
            "not" => {
                // Negate the input
                if inputs.len() != 1 {
                    return None;
                }
                let value = inputs[0] == "true";
                Some((!value).to_string())
            },
            _ => None,
        }
    }
    
    /// Get all input values for an effect
    fn get_input_values(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> Option<Vec<String>> {
        let mut inputs = Vec::new();
        let mut input_ids = Vec::new();
        
        // Get all predecessors
        for (pred_id, _) in teg.get_incoming_edges(effect_id) {
            input_ids.push(pred_id);
        }
        
        // Sort by edge order if available
        // This is important for operations where order matters
        input_ids.sort_by(|a, b| {
            let edge_a = teg.get_edge(*a, effect_id).unwrap();
            let edge_b = teg.get_edge(*b, effect_id).unwrap();
            
            edge_a.order().cmp(&edge_b.order())
        });
        
        // Evaluate each input
        for pred_id in input_ids {
            let value = self.evaluate_constant(teg, pred_id)?;
            inputs.push(value);
        }
        
        Some(inputs)
    }
}

impl Optimization for ConstantFolding {
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
        
        // Find all constant effects
        let constant_effects = self.find_constant_effects(teg);
        
        // Replace each constant with its evaluated value
        for effect_id in constant_effects {
            if let Some(value) = self.evaluate_constant(teg, effect_id) {
                // Create a new constant node
                let mut constant_node = EffectNode::new(
                    format!("const_{}", effect_id),
                    "constant".to_string(),
                );
                
                // Set the constant value
                constant_node.set_constant_value(value);
                
                // Get the original effect for metadata
                let original = teg.get_effect(effect_id).unwrap().clone();
                
                // Copy metadata
                let mut metadata = original.metadata().clone();
                metadata.insert("folded_from".to_string(), original.name().to_string());
                constant_node.set_metadata(metadata);
                
                // Add the constant node to the graph
                let const_id = teg.add_effect(constant_node)?;
                
                // Redirect all outgoing edges from the original effect to the constant
                let outgoing = teg.get_outgoing_edges(effect_id);
                for (dst, edge_data) in outgoing {
                    teg.add_edge(const_id, dst, edge_data.clone())?;
                }
                
                // Remove the original effect
                teg.remove_effect(effect_id)?;
                
                changed = true;
            }
        }
        
        Ok(changed)
    }

    fn preserves_adjunction(&self) -> bool {
        // Constant folding preserves semantics as it only replaces computations
        // with their results, which doesn't change program behavior
        true
    }

    fn preserves_resource_structure(&self) -> bool {
        // Constant folding doesn't affect resource structure
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_constant_folding_arithmetic() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create a simple graph with constant expressions
        let const_a = graph_builder.add_constant("const_a", 5);
        let const_b = graph_builder.add_constant("const_b", 3);
        
        // Add operation: a + b
        let add = graph_builder.add_effect("add", "add");
        graph_builder.connect_effects(const_a, add);
        graph_builder.connect_effects(const_b, add);
        
        // Add operation: a * b
        let multiply = graph_builder.add_effect("multiply", "multiply");
        graph_builder.connect_effects(const_a, multiply);
        graph_builder.connect_effects(const_b, multiply);
        
        let mut teg = graph_builder.build().unwrap();
        
        // Apply optimization
        let opt = ConstantFolding::new();
        let config = OptimizationConfig {
            level: 1,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that constants were folded
        assert!(result);
        
        // Original operations should be removed
        assert!(teg.get_effect(add).is_none());
        assert!(teg.get_effect(multiply).is_none());
        
        // New constants should be present
        let folded_effects: Vec<_> = teg.effects()
            .iter()
            .filter(|(_, e)| e.name().starts_with("const_"))
            .collect();
            
        // We should have the original two constants plus two new folded constants
        assert_eq!(folded_effects.len(), 4);
        
        // Check for the folded values
        let has_add_result = folded_effects.iter().any(|(_, e)| {
            e.is_constant() && e.constant_value() == Some("8".to_string())
        });
        
        let has_multiply_result = folded_effects.iter().any(|(_, e)| {
            e.is_constant() && e.constant_value() == Some("15".to_string())
        });
        
        assert!(has_add_result);
        assert!(has_multiply_result);
    }
    
    #[test]
    fn test_constant_folding_string() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create string constants
        let str_a = graph_builder.add_string_constant("str_a", "Hello, ");
        let str_b = graph_builder.add_string_constant("str_b", "World!");
        
        // Add operation: concat strings
        let concat = graph_builder.add_effect("concat", "concat");
        graph_builder.connect_effects(str_a, concat);
        graph_builder.connect_effects(str_b, concat);
        
        let mut teg = graph_builder.build().unwrap();
        
        // Apply optimization
        let opt = ConstantFolding::new();
        let config = OptimizationConfig {
            level: 1,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that constants were folded
        assert!(result);
        assert!(teg.get_effect(concat).is_none());
        
        // Check for the folded value
        let folded_effects: Vec<_> = teg.effects()
            .iter()
            .filter(|(_, e)| e.is_constant())
            .collect();
            
        let has_concat_result = folded_effects.iter().any(|(_, e)| {
            e.constant_value() == Some("Hello, World!".to_string())
        });
        
        assert!(has_concat_result);
    }
}
