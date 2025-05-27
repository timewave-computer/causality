// Purpose: Provides graph analysis functions, e.g., cycle detection.
// This file was moved from causality-types/src/graph/analysis.rs

use crate::graph_registry::EdgeRegistry; // Changed from crate::graph::registry::EdgeRegistry
use causality_types::primitive::ids::NodeId;
use causality_types::tel::Edge as TelEdge; // Changed from crate::tel::Edge
use causality_types::tel::TelEdgeTypes; // Changed from crate::tel::TelEdgeTypes
use std::collections::HashSet;

/// Checks for cycles in a subgraph defined by a set of nodes and an edge registry.
/// This is a basic DFS-based cycle detection.
pub fn tel_graph_has_cycles(
    nodes_in_subgraph: &HashSet<NodeId>,
    edge_registry: &EdgeRegistry<TelEdgeTypes>,
) -> bool {
    let mut visited_global = HashSet::new(); // Global tracker for visited nodes across all DFS calls

    for node_id in nodes_in_subgraph {
        if !visited_global.contains(node_id) {
            let mut recursion_stack_path = HashSet::new(); // Tracks nodes in current DFS path
            if dfs_cycle_check(
                *node_id,
                &mut visited_global,
                &mut recursion_stack_path,
                nodes_in_subgraph,
                edge_registry,
            ) {
                return true;
            }
        }
    }
    false
}

// Helper DFS function for cycle detection
fn dfs_cycle_check(
    current_node_id: NodeId,
    visited_global: &mut HashSet<NodeId>,
    recursion_stack_path: &mut HashSet<NodeId>,
    nodes_in_subgraph: &HashSet<NodeId>,
    edge_registry: &EdgeRegistry<TelEdgeTypes>,
) -> bool {
    if !nodes_in_subgraph.contains(&current_node_id) {
        return false; // Not part of the subgraph we are checking
    }

    visited_global.insert(current_node_id);
    recursion_stack_path.insert(current_node_id);

    for edge_id in edge_registry.get_outgoing_edges(current_node_id) {
        // Assuming get_edge<TelEdge> works correctly after TelEdge type is resolved
        if let Some(edge) = edge_registry.get_edge::<TelEdge>(edge_id) {
            let neighbor_node_id = edge.target; // Assuming TelEdge has a target field

            if !nodes_in_subgraph.contains(&neighbor_node_id) {
                continue; // Edge leads out of the subgraph, ignore for cycle within subgraph
            }

            if recursion_stack_path.contains(&neighbor_node_id) {
                return true; // Cycle detected
            }

            if !visited_global.contains(&neighbor_node_id) && dfs_cycle_check(
                neighbor_node_id,
                visited_global,
                recursion_stack_path,
                nodes_in_subgraph,
                edge_registry,
            ) {
                return true;
            }
        }
    }

    recursion_stack_path.remove(&current_node_id); // Backtrack
    false
}
