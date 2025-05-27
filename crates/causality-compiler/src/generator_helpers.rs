//-----------------------------------------------------------------------------
// Generator helper functions for the new collection-based approach
//-----------------------------------------------------------------------------

use causality_types::primitive::ids::{DomainId, EdgeId, NodeId};
use std::collections::{HashMap, HashSet};
use crate::project::ProgramProject;

/// Finds which domain a node belongs to using SMT storage
pub fn get_node_domain_from_collections(
    node_id: NodeId,
    project: &ProgramProject,
) -> Option<DomainId> {
    // With SMT storage, we need to query specific nodes rather than iterate
    // Try to get the node from SMT storage
    if let Ok(Some(tel_node)) = project.storage.get_node(&node_id) {
        match tel_node {
            causality_core::smt_collections::TelNode::Effect(effect) => {
                return Some(effect.domain_id);
            }
            causality_core::smt_collections::TelNode::Handler(handler) => {
                return Some(handler.domain_id);
            }
            causality_core::smt_collections::TelNode::Intent(intent) => {
                return Some(intent.domain_id);
            }
            causality_core::smt_collections::TelNode::Resource(_resource_ref) => {
                // ResourceRefs don't store domains. Return project domain or default
                return Some(project.domain_id());
            }
        }
    }
    
    None
}

/// Determines if a graph has cycles using the simplified edge collection
pub fn tel_graph_has_cycles(
    nodes: &HashSet<NodeId>,
    edges: &HashMap<EdgeId, (NodeId, NodeId, String)>, // source, target, kind
) -> bool {
    // Implementation of cycle detection algorithm using DFS
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    
    for &node_id in nodes {
        if is_cyclic_util(node_id, edges, &mut visited, &mut recursion_stack, nodes) {
            return true;
        }
    }
    
    false
}

/// Helper function for cycle detection
fn is_cyclic_util(
    node_id: NodeId,
    edges: &HashMap<EdgeId, (NodeId, NodeId, String)>,
    visited: &mut HashSet<NodeId>,
    recursion_stack: &mut HashSet<NodeId>,
    nodes_in_subgraph: &HashSet<NodeId>,
) -> bool {
    // Mark current node as visited and add to recursion stack
    visited.insert(node_id);
    recursion_stack.insert(node_id);
    
    // Check all outgoing edges from this node
    for (source, target, _) in edges.values() {
        if *source == node_id && nodes_in_subgraph.contains(target) {
            // If the target is not visited, then check if its subtree has a cycle
            if !visited.contains(target) {
                if is_cyclic_util(*target, edges, visited, recursion_stack, nodes_in_subgraph) {
                    return true;
                }
            } else if recursion_stack.contains(target) {
                // If the target is in the recursion stack, then there is a cycle
                return true;
            }
        }
    }
    
    // Remove the current node from recursion stack
    recursion_stack.remove(&node_id);
    false
}
