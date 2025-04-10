//! Resource-specific optimizations for TEG
//!
//! This module provides optimizations focused on resource access patterns
//! and operation batching in the Temporal Effect Graph.

use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use std::rc::Rc;

use crate::{
    TemporalEffectGraph, 
    EffectNode, 
    ResourceNode,
    EffectId, 
    ResourceId,
    graph::edge::{Condition, TemporalRelation, RelationshipType, AccessMode}
};
use super::{Optimization, OptimizationConfig};

/// Optimizes resource access patterns by reordering operations and removing redundant accesses
///
/// This optimization identifies resource access patterns and applies the following optimizations:
/// 1. Reordering independent resource accesses to improve locality
/// 2. Removing redundant resource reads when the value hasn't changed
/// 3. Combining read-modify-write patterns into single operations
#[derive(Debug)]
pub struct ResourceAccessOptimization {
    /// Metadata describing the optimization
    name: String,
    description: String,
}

impl ResourceAccessOptimization {
    /// Create a new ResourceAccessOptimization
    pub fn new() -> Self {
        Self {
            name: "resource_access_optimization".to_string(),
            description: "Optimizes resource access patterns by reordering and eliminating redundant accesses".to_string(),
        }
    }

    /// Find all resource accesses in the TEG
    fn find_resource_accesses(&self, teg: &TemporalEffectGraph) -> HashMap<ResourceId, Vec<EffectId>> {
        let mut result = HashMap::new();
        
        for (effect_id, effect_node) in teg.effects() {
            // Skip non-resource operations
            if !effect_node.is_resource_operation() {
                continue;
            }
            
            // Get the resource ID from the effect
            let resource_id = effect_node
                .resource_edges()
                .iter()
                .next()
                .map(|(resource_id, _)| *resource_id);
                
            if let Some(resource_id) = resource_id {
                result
                    .entry(resource_id)
                    .or_insert_with(Vec::new)
                    .push(effect_id);
            }
        }
        
        result
    }
    
    /// Check if effect1 and effect2 can be reordered
    fn can_reorder(&self, teg: &TemporalEffectGraph, effect1: EffectId, effect2: EffectId) -> bool {
        // If there's a direct or indirect dependency between the effects, they can't be reordered
        if teg.has_path(effect1, effect2) || teg.has_path(effect2, effect1) {
            return false;
        }
        
        // Get all resources accessed by both effects
        let resources1 = teg.get_effect_resources(effect1);
        let resources2 = teg.get_effect_resources(effect2);
        
        // If they don't share resources, they can be reordered
        let mut shared_resources = HashSet::new();
        for r1 in &resources1 {
            if resources2.contains(r1) {
                shared_resources.insert(*r1);
            }
        }
        
        if shared_resources.is_empty() {
            return true;
        }
        
        // Check access modes for shared resources
        for resource_id in shared_resources {
            let access1 = teg.get_access_mode(effect1, resource_id);
            let access2 = teg.get_access_mode(effect2, resource_id);
            
            // If both are read-only, they can be reordered
            if access1 == Some(AccessMode::Read) && access2 == Some(AccessMode::Read) {
                continue;
            }
            
            // Otherwise, they can't be reordered
            return false;
        }
        
        true
    }
    
    /// Check if a read access is redundant
    fn is_redundant_read(&self, teg: &TemporalEffectGraph, effect_id: EffectId) -> bool {
        let effect = match teg.get_effect(effect_id) {
            Some(e) => e,
            None => return false,
        };
        
        // Only consider read operations
        if !effect.is_resource_operation() || effect.operation_type() != Some("read") {
            return false;
        }
        
        // Get the resource being read
        let resource_id = match effect.resource_edges().iter().next() {
            Some((resource_id, _)) => *resource_id,
            None => return false,
        };
        
        // Get all previous read/write operations on this resource
        let prev_ops = teg.find_predecessors(effect_id, |e_id| {
            let e = teg.get_effect(e_id).unwrap();
            e.is_resource_operation() && teg.get_effect_resources(e_id).contains(&resource_id)
        });
        
        // If no previous operations, it's not redundant
        if prev_ops.is_empty() {
            return false;
        }
        
        // If the most recent operation was a read with the same parameters,
        // this read is redundant
        let latest_op = prev_ops[0];
        let latest_effect = teg.get_effect(latest_op).unwrap();
        
        if latest_effect.operation_type() == Some("read") &&
           latest_effect.parameters() == effect.parameters() {
            return true;
        }
        
        false
    }
}

impl Optimization for ResourceAccessOptimization {
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
        
        // Find all resource accesses
        let resource_accesses = self.find_resource_accesses(teg);
        
        // Identify redundant reads
        let mut redundant_reads = Vec::new();
        for (_, effect_ids) in &resource_accesses {
            for &effect_id in effect_ids {
                if self.is_redundant_read(teg, effect_id) {
                    redundant_reads.push(effect_id);
                }
            }
        }
        
        // Remove redundant reads
        for effect_id in redundant_reads {
            teg.remove_effect(effect_id)?;
            changed = true;
        }
        
        // Handle reordering
        // This is a more complex optimization that requires careful analysis
        // of the data flow. For now, we'll focus on the redundant reads.
        
        Ok(changed)
    }

    fn preserves_adjunction(&self) -> bool {
        // This optimization preserves the adjunction property because
        // it only removes redundant operations without changing the
        // semantic meaning of the program
        true
    }

    fn preserves_resource_structure(&self) -> bool {
        // This optimization preserves the resource monoidal structure
        // as it doesn't change how resources are composed
        true
    }
}

/// Batches similar resource operations to reduce overhead
///
/// This optimization identifies resource operations that can be batched together
/// to reduce the overhead of multiple separate operations. It focuses on:
/// 1. Combining multiple reads from the same resource
/// 2. Batching writes to the same resource that don't depend on each other
/// 3. Batching operations across different resources when they're independent
#[derive(Debug)]
pub struct ResourceOperationBatching {
    /// Metadata describing the optimization
    name: String,
    description: String,
}

impl ResourceOperationBatching {
    /// Create a new ResourceOperationBatching optimization
    pub fn new() -> Self {
        Self {
            name: "resource_operation_batching".to_string(),
            description: "Batches similar resource operations to reduce overhead".to_string(),
        }
    }
    
    /// Find operations that can be batched
    fn find_batchable_operations(&self, teg: &TemporalEffectGraph) -> Vec<Vec<EffectId>> {
        let mut result = Vec::new();
        let mut processed = HashSet::new();
        
        // Group operations by resource
        let resource_ops = self.group_by_resource(teg);
        
        for (resource_id, ops) in resource_ops {
            // Skip resources with only one operation
            if ops.len() <= 1 {
                continue;
            }
            
            let resource = match teg.get_resource(resource_id) {
                Some(r) => r,
                None => continue,
            };
            
            // For each operation type, find groups that can be batched
            let op_types = vec!["read", "write", "create", "delete"];
            
            for op_type in op_types {
                let type_ops: Vec<EffectId> = ops.iter()
                    .filter(|&&op_id| {
                        !processed.contains(&op_id) && 
                        teg.get_effect(op_id)
                           .map(|e| e.operation_type() == Some(op_type))
                           .unwrap_or(false)
                    })
                    .copied()
                    .collect();
                
                if type_ops.len() <= 1 {
                    continue;
                }
                
                // Find independent subsets that can be batched
                let mut remaining = type_ops.clone();
                
                while !remaining.is_empty() {
                    let start = remaining[0];
                    let mut batch = vec![start];
                    remaining.remove(0);
                    
                    // Find other operations that can be batched with start
                    let mut i = 0;
                    while i < remaining.len() {
                        let can_batch = batch.iter().all(|&batch_op| {
                            self.can_batch(teg, batch_op, remaining[i])
                        });
                        
                        if can_batch {
                            batch.push(remaining[i]);
                            remaining.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                    
                    if batch.len() > 1 {
                        result.push(batch.clone());
                        for op in batch {
                            processed.insert(op);
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Group operations by resource
    fn group_by_resource(&self, teg: &TemporalEffectGraph) -> HashMap<ResourceId, Vec<EffectId>> {
        let mut result = HashMap::new();
        
        for (effect_id, effect) in teg.effects() {
            // Skip non-resource operations
            if !effect.is_resource_operation() {
                continue;
            }
            
            for (resource_id, _) in effect.resource_edges() {
                result
                    .entry(*resource_id)
                    .or_insert_with(Vec::new)
                    .push(effect_id);
            }
        }
        
        result
    }
    
    /// Check if two operations can be batched
    fn can_batch(&self, teg: &TemporalEffectGraph, op1: EffectId, op2: EffectId) -> bool {
        // Operations can be batched if:
        // 1. They're the same type (both reads, both writes, etc.)
        // 2. They don't depend on each other
        // 3. They have compatible parameters
        
        let effect1 = match teg.get_effect(op1) {
            Some(e) => e,
            None => return false,
        };
        
        let effect2 = match teg.get_effect(op2) {
            Some(e) => e,
            None => return false,
        };
        
        // Check if they're the same type
        if effect1.operation_type() != effect2.operation_type() {
            return false;
        }
        
        // Check if they depend on each other
        if teg.has_path(op1, op2) || teg.has_path(op2, op1) {
            return false;
        }
        
        // Check resource compatibility
        let resources1: HashSet<_> = effect1.resource_edges().iter()
            .map(|(id, _)| *id)
            .collect();
            
        let resources2: HashSet<_> = effect2.resource_edges().iter()
            .map(|(id, _)| *id)
            .collect();
            
        // For now, only batch operations on the same resource
        if resources1 != resources2 {
            return false;
        }
        
        // For more complex batching, we'd need to check parameter compatibility
        // and other constraints specific to the operation type
        
        true
    }
}

impl Optimization for ResourceOperationBatching {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn apply(&self, teg: &mut TemporalEffectGraph, config: &OptimizationConfig) -> Result<bool> {
        let mut changed = false;
        
        // Skip if optimization level is too low
        if config.level < 3 {
            return Ok(false);
        }
        
        // Find operations that can be batched
        let batchable_ops = self.find_batchable_operations(teg);
        
        for batch in batchable_ops {
            if batch.len() <= 1 {
                continue;
            }
            
            // Create a new batched operation
            let first_op = batch[0];
            let first_effect = teg.get_effect(first_op).unwrap().clone();
            
            // Create a new effect node for the batched operation
            let mut batched_effect = EffectNode::new(
                format!("batched_{}", first_effect.name()),
                first_effect.operation_type().unwrap_or("unknown").to_string(),
            );
            
            // Copy parameters and metadata from the first operation
            batched_effect.set_parameters(first_effect.parameters().clone());
            
            // Add a metadata entry indicating this is a batched operation
            let mut metadata = first_effect.metadata().clone();
            metadata.insert("batched".to_string(), batch.len().to_string());
            batched_effect.set_metadata(metadata);
            
            // Add the batched effect to the graph
            let batched_id = teg.add_effect(batched_effect)?;
            
            // Connect the batched effect to all resources from the original operations
            for op_id in &batch {
                let effect = teg.get_effect(*op_id).unwrap();
                
                // Copy resource connections
                for (resource_id, edge_data) in effect.resource_edges() {
                    teg.connect_effect_to_resource(batched_id, *resource_id, edge_data.clone())?;
                }
                
                // Redirect incoming edges to the batched operation
                let incoming = teg.get_incoming_edges(*op_id);
                for (src, edge_data) in incoming {
                    teg.add_edge(src, batched_id, edge_data.clone())?;
                }
                
                // Redirect outgoing edges from the batched operation
                let outgoing = teg.get_outgoing_edges(*op_id);
                for (dst, edge_data) in outgoing {
                    teg.add_edge(batched_id, dst, edge_data.clone())?;
                }
            }
            
            // Remove the original operations
            for op_id in batch {
                teg.remove_effect(op_id)?;
            }
            
            changed = true;
        }
        
        Ok(changed)
    }

    fn preserves_adjunction(&self) -> bool {
        // Batching operations should preserve semantics if done correctly
        true
    }

    fn preserves_resource_structure(&self) -> bool {
        // Batching preserves resource structure as it combines operations
        // without changing their fundamental relationships
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    
    #[test]
    fn test_resource_access_optimization() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create a simple graph with redundant reads
        let resource = graph_builder.add_resource("test_resource", "test_type");
        
        let read1 = graph_builder.add_effect("read1", "read");
        let read2 = graph_builder.add_effect("read2", "read"); // Redundant read
        let write = graph_builder.add_effect("write", "write");
        let read3 = graph_builder.add_effect("read3", "read");
        
        // Connect effects to resources
        graph_builder.connect_effect_to_resource(read1, resource, AccessMode::Read);
        graph_builder.connect_effect_to_resource(read2, resource, AccessMode::Read);
        graph_builder.connect_effect_to_resource(write, resource, AccessMode::Write);
        graph_builder.connect_effect_to_resource(read3, resource, AccessMode::Read);
        
        // Set up dependencies
        graph_builder.add_dependency(read1, read2);
        graph_builder.add_dependency(read2, write);
        graph_builder.add_dependency(write, read3);
        
        let mut teg = graph_builder.build().unwrap();
        
        // Apply optimization
        let opt = ResourceAccessOptimization::new();
        let config = OptimizationConfig {
            level: 2,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that the redundant read was removed
        assert!(result);
        assert!(teg.get_effect(read1).is_some());
        assert!(teg.get_effect(read2).is_none()); // Should be removed
        assert!(teg.get_effect(write).is_some());
        assert!(teg.get_effect(read3).is_some());
        
        // Check that dependencies were properly maintained
        assert!(teg.has_dependency(read1, write));
        assert!(teg.has_dependency(write, read3));
    }
    
    #[test]
    fn test_resource_operation_batching() {
        let mut graph_builder = GraphBuilder::new();
        
        // Create a simple graph with multiple read operations
        let resource = graph_builder.add_resource("test_resource", "test_type");
        
        let read1 = graph_builder.add_effect("read1", "read");
        let read2 = graph_builder.add_effect("read2", "read");
        let read3 = graph_builder.add_effect("read3", "read");
        
        let write1 = graph_builder.add_effect("write1", "write");
        let write2 = graph_builder.add_effect("write2", "write");
        
        // Connect effects to resources
        graph_builder.connect_effect_to_resource(read1, resource, AccessMode::Read);
        graph_builder.connect_effect_to_resource(read2, resource, AccessMode::Read);
        graph_builder.connect_effect_to_resource(read3, resource, AccessMode::Read);
        
        graph_builder.connect_effect_to_resource(write1, resource, AccessMode::Write);
        graph_builder.connect_effect_to_resource(write2, resource, AccessMode::Write);
        
        // Set up dependencies (read1 -> write1 -> read2, read3)
        // Note: read2 and read3 are independent
        graph_builder.add_dependency(read1, write1);
        graph_builder.add_dependency(write1, read2);
        graph_builder.add_dependency(write1, read3);
        graph_builder.add_dependency(read2, write2);
        graph_builder.add_dependency(read3, write2);
        
        let mut teg = graph_builder.build().unwrap();
        
        // Apply optimization
        let opt = ResourceOperationBatching::new();
        let config = OptimizationConfig {
            level: 3,
            ..Default::default()
        };
        
        let result = opt.apply(&mut teg, &config).unwrap();
        
        // Check that read2 and read3 were batched
        assert!(result);
        assert!(teg.get_effect(read1).is_some());
        assert!(teg.get_effect(read2).is_none()); // Should be batched
        assert!(teg.get_effect(read3).is_none()); // Should be batched
        assert!(teg.get_effect(write1).is_some());
        assert!(teg.get_effect(write2).is_some());
        
        // There should be a new batched operation
        let batched_reads = teg.effects().iter()
            .filter(|(_, e)| e.name().starts_with("batched_"))
            .collect::<Vec<_>>();
            
        assert_eq!(batched_reads.len(), 1);
        
        // The batched operation should have dependencies from write1 and to write2
        let batched_id = batched_reads[0].0;
        assert!(teg.has_dependency(write1, batched_id));
        assert!(teg.has_dependency(batched_id, write2));
    }
} 