//! Constant Folding Optimization for TEG
//!
//! This module provides optimization passes that fold constant values
//! and simplify expressions in the Temporal Effect Graph.

use anyhow::Result;
use std::collections::HashMap;

use crate::{TemporalEffectGraph, EffectId};
use crate::effect_node::{EffectNode, ParameterValue};
use crate::builder::GraphBuilder;
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
            if effect.is_constant() || self.can_be_evaluated_at_compile_time(teg, effect_id.clone()) {
                constants.push(effect_id.clone());
            }
        }
        
        constants
    }
    
    /// Check if an effect can be evaluated at compile time
    fn can_be_evaluated_at_compile_time(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> bool {
        let effect = match teg.get_effect(&effect_id) {
            Some(e) => e,
            None => return false,
        };
        
        // Check if the effect is a pure computation
        if !effect.is_pure() {
            return false;
        }
        
        // Check if all inputs are constants
        let predecessors = teg.get_incoming_edges(&effect_id);
        for (pred_id, _) in predecessors {
            let pred = match teg.get_effect(&pred_id) {
                Some(e) => e,
                None => return false,
            };
            
            if !pred.is_constant() && !self.can_be_evaluated_at_compile_time(teg, pred_id.clone()) {
                return false;
            }
        }
        
        true
    }
    
    /// Evaluate a constant expression
    fn evaluate_constant(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> Option<String> {
        let effect = teg.get_effect(&effect_id)?;
        
        // If it's already a constant, return its value
        if effect.is_constant() {
            return effect.constant_value().and_then(|v| {
                match v {
                    ParameterValue::String(s) => Some(s.clone()),
                    ParameterValue::Integer(i) => Some(i.to_string()),
                    ParameterValue::Boolean(b) => Some(b.to_string()),
                    ParameterValue::Float(f) => Some(f.to_string()),
                    _ => Some("".to_string())
                }
            });
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
        for (pred_id, _) in teg.get_incoming_edges(&effect_id) {
            input_ids.push(pred_id.clone());
        }
        
        // Sort by edge order if available
        // This is important for operations where order matters
        input_ids.sort_by(|a, b| {
            let edge_a = teg.get_edge(a.clone(), effect_id.clone()).unwrap();
            let edge_b = teg.get_edge(b.clone(), effect_id.clone()).unwrap();
            
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
            if let Some(value) = self.evaluate_constant(teg, effect_id.clone()) {
                // Create a new constant node
                let mut constant_node = EffectNode::new(
                    format!("const_{}", effect_id),
                    "constant".to_string(),
                    "constant_domain".to_string(), // Default domain ID for constant nodes
                );
                
                // Set the constant value
                let value_param = ParameterValue::String(value);
                constant_node.parameters.insert("constant_value".to_string(), value_param.clone());
                constant_node.parameters.insert("constant".to_string(), ParameterValue::Boolean(true));
                
                // Get the original effect for metadata
                let original = teg.get_effect(&effect_id).unwrap().clone();
                
                // Copy metadata
                let mut metadata = original.metadata().clone();
                metadata.insert(
                    "folded_from".to_string(), 
                    ParameterValue::String(original.name().to_string())
                );
                constant_node.metadata = metadata;
                
                // Add the constant node to the graph
                let const_id = teg.add_effect(constant_node)?;
                
                // Redirect all outgoing edges from the original effect to the constant
                let outgoing = teg.get_outgoing_edges(&effect_id);
                for (dst, edge_data) in outgoing {
                    teg.add_edge(&const_id, &dst, edge_data.clone())?;
                }
                
                // Remove the original effect
                teg.remove_effect(&effect_id)?;
                
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
    fn test_arithmetic() {
        // Create a new graph
        let mut builder = GraphBuilder::new();
        
        // Add two number constants
        let const1 = builder.add_effect(
            EffectNode::new(
                "const1".to_string(),
                "constant".to_string(),
                "constant_domain".to_string(),
            )
        ).unwrap();
        
        // Set constant parameters
        let mut const1_node = builder.get_effect_mut(&const1).unwrap();
        const1_node.parameters.insert("constant".to_string(), ParameterValue::Boolean(true));
        const1_node.parameters.insert("constant_value".to_string(), ParameterValue::Number(5.0));
        
        let const2 = builder.add_effect(
            EffectNode::new(
                "const2".to_string(),
                "constant".to_string(),
                "constant_domain".to_string(),
            )
        ).unwrap();
        
        // Set constant parameters
        let mut const2_node = builder.get_effect_mut(&const2).unwrap();
        const2_node.parameters.insert("constant".to_string(), ParameterValue::Boolean(true));
        const2_node.parameters.insert("constant_value".to_string(), ParameterValue::Number(3.0));
        
        // Create an add operation
        let add = builder.add_effect(
            EffectNode::new(
                "add".to_string(),
                "add".to_string(),
                "arithmetic_domain".to_string(),
            )
        ).unwrap();
        
        // Connect the constants to the add operation
        builder.connect_effects(&const1, &add).unwrap();
        builder.connect_effects(&const2, &add).unwrap();
        
        // Build the graph
        let mut teg = builder.build().unwrap();
        
        // Apply constant folding
        let optimization = ConstantFolding::new();
        let changed = optimization.apply(&mut teg, &OptimizationConfig::default()).unwrap();
        
        // Verify that the graph changed
        assert!(changed);
        
        // Verify that the add operation was replaced with a constant
        let mut found_const = false;
        for (_, effect) in teg.effect_nodes.iter() {
            if effect.effect_type() == "constant" {
                if let Some(ParameterValue::Number(value)) = effect.parameters.get("constant_value") {
                    if *value == 8.0 {
                        found_const = true;
                    }
                }
            }
        }
        
        assert!(found_const);
    }

    #[test]
    fn test_string_operations() {
        // Create a new graph
        let mut builder = GraphBuilder::new();
        
        // Add two string constants
        let const1 = builder.add_effect(
            EffectNode::new(
                "const1".to_string(),
                "constant".to_string(),
                "constant_domain".to_string(),
            )
        ).unwrap();
        
        // Set constant parameters
        let mut const1_node = builder.get_effect_mut(&const1).unwrap();
        const1_node.parameters.insert("constant".to_string(), ParameterValue::Boolean(true));
        const1_node.parameters.insert("constant_value".to_string(), ParameterValue::String("Hello, ".to_string()));
        
        let const2 = builder.add_effect(
            EffectNode::new(
                "const2".to_string(),
                "constant".to_string(),
                "constant_domain".to_string(),
            )
        ).unwrap();
        
        // Set constant parameters
        let mut const2_node = builder.get_effect_mut(&const2).unwrap();
        const2_node.parameters.insert("constant".to_string(), ParameterValue::Boolean(true));
        const2_node.parameters.insert("constant_value".to_string(), ParameterValue::String("world!".to_string()));
        
        // Create a concat operation
        let concat = builder.add_effect(
            EffectNode::new(
                "concat".to_string(),
                "concat".to_string(),
                "string_domain".to_string(),
            )
        ).unwrap();
        
        // Connect the constants to the concat operation
        builder.connect_effects(&const1, &concat).unwrap();
        builder.connect_effects(&const2, &concat).unwrap();
        
        // Build the graph
        let mut teg = builder.build().unwrap();
        
        // Apply constant folding
        let optimization = ConstantFolding::new();
        let changed = optimization.apply(&mut teg, &OptimizationConfig::default()).unwrap();
        
        // Verify that the graph changed
        assert!(changed);
        
        // Verify that the concat operation was replaced with a constant
        let mut found_const = false;
        for (_, effect) in teg.effect_nodes.iter() {
            if effect.effect_type() == "constant" {
                if let Some(ParameterValue::String(value)) = effect.parameters.get("constant_value") {
                    if value == "Hello, world!" {
                        found_const = true;
                    }
                }
            }
        }
        
        assert!(found_const);
    }
}
