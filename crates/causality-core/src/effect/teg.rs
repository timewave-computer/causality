//! Temporal Effect Graph (TEG) for dynamic orchestration
//!
//! This module implements the Temporal Effect Graph system for handling
//! dynamic effect orchestration and execution.

use super::{
    core::{EffectExpr, EffectExprKind},
    intent::{Intent, Constraint},
};
use crate::{
    lambda::base::{Value, SessionType},
    system::{
        content_addressing::{EntityId, Timestamp},
        deterministic::DeterministicFloat,
    },
};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use ssz::{Encode, Decode};

/// Unique identifier for nodes in the TEG
pub type NodeId = EntityId;

/// Status of a node in the TEG
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is waiting for dependencies
    Pending,
    
    /// Node is ready to execute (dependencies satisfied)
    Ready,
    
    /// Node is currently executing
    Executing,
    
    /// Node has completed successfully
    Completed,
    
    /// Node execution failed
    Failed(String),
    
    /// Node was cancelled
    Cancelled,
}

/// Node in the Temporal Effect Graph
#[derive(Debug, Clone)]
pub struct EffectNode {
    /// Unique identifier for this node
    pub id: NodeId,
    
    /// The effect to execute
    pub effect: EffectExpr,
    
    /// Current execution status
    pub status: NodeStatus,
    
    /// Direct dependencies (must complete before this node)
    pub dependencies: Vec<NodeId>,
    
    /// Execution results (if completed)
    pub results: Option<Value>,
    
    /// Estimated execution cost
    pub cost: u64,
    
    /// Resource requirements
    pub resource_requirements: Vec<String>,
    
    /// Resource productions
    pub resource_productions: Vec<String>,
}

/// Types of edges in the TEG
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectEdge {
    /// Causal dependency: from must complete before to
    CausalityLink { 
        from: NodeId, 
        to: NodeId,
        constraint: Option<String>,
    },
    
    /// Resource dependency: from produces resource needed by to
    ResourceLink { 
        from: NodeId, 
        to: NodeId, 
        resource: String,
    },
    
    /// Control flow dependency: conditional execution
    ControlLink { 
        from: NodeId, 
        to: NodeId, 
        condition: Constraint,
    },
}

/// Metadata for the entire TEG
#[derive(Debug, Clone)]
pub struct TegMetadata {
    /// Creation timestamp
    pub created_at: Timestamp,
    
    /// Total estimated cost
    pub total_cost: u64,
    
    /// Critical path length
    pub critical_path_length: u64,
    
    /// Parallelization potential (scaled by 1000 for precision)
    pub parallelization_factor: u64,
    
    /// Intent that generated this TEG
    pub source_intent: Option<EntityId>,
}

/// Main Temporal Effect Graph structure
#[derive(Debug, Clone)]
pub struct TemporalEffectGraph {
    /// Nodes in the graph
    pub nodes: BTreeMap<NodeId, EffectNode>,
    
    /// Edges representing dependencies
    pub edges: Vec<EffectEdge>,
    
    /// Graph metadata
    pub metadata: TegMetadata,
    
    /// Adjacency list for efficient traversal
    adjacency_list: BTreeMap<NodeId, Vec<NodeId>>,
    
    /// Reverse adjacency list (incoming edges)
    reverse_adjacency_list: BTreeMap<NodeId, Vec<NodeId>>,
}

/// Result of TEG execution
#[derive(Debug, Clone)]
pub struct TegResult {
    /// Final results from all nodes
    pub results: BTreeMap<NodeId, Value>,
    
    /// Execution statistics
    pub stats: ExecutionStats,
    
    /// Any errors that occurred
    pub errors: Vec<(NodeId, String)>,
}

/// Execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total execution time
    pub total_time_ms: u64,
    
    /// Number of nodes executed in parallel
    pub parallel_nodes: u64,
    
    /// Actual parallelization achieved (using deterministic float)
    pub actual_parallelization: DeterministicFloat,
    
    /// Critical path execution time
    pub critical_path_time_ms: u64,
}

/// Error types for TEG operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TegError {
    /// Circular dependency detected
    CyclicDependency(Vec<NodeId>),
    
    /// Node not found
    NodeNotFound(NodeId),
    
    /// Resource dependency cannot be satisfied
    UnsatisfiableResource(String),
    
    /// Invalid graph structure
    InvalidGraph(String),
    
    /// Execution error
    ExecutionError(NodeId, String),
}

/// Helper function to create EntityId from effect
fn effect_to_entity_id(effect: &EffectExpr) -> EntityId {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Use debug representation hash as a temporary solution
    let debug_str = format!("{:?}", effect);
    let mut hasher = DefaultHasher::new();
    debug_str.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Convert hash to 32-byte array
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&hash.to_le_bytes());
    
    EntityId::from_bytes(bytes)
}

impl TemporalEffectGraph {
    /// Create a new empty TEG
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
            metadata: TegMetadata {
                created_at: Timestamp::now(),
                total_cost: 0,
                critical_path_length: 0,
                parallelization_factor: 1000,
                source_intent: None,
            },
            adjacency_list: BTreeMap::new(),
            reverse_adjacency_list: BTreeMap::new(),
        }
    }
    
    /// Create TEG from an effect sequence
    pub fn from_effect_sequence(effects: Vec<EffectExpr>) -> Result<Self, TegError> {
        let mut teg = Self::new();
        
        // Create nodes for each effect
        let mut previous_node: Option<NodeId> = None;
        
        for effect in effects {
            let node_id = effect_to_entity_id(&effect);
            let node = EffectNode {
                id: node_id,
                effect: effect.clone(),
                status: NodeStatus::Pending,
                dependencies: if let Some(prev) = previous_node {
                    vec![prev]
                } else {
                    vec![]
                },
                results: None,
                cost: teg.estimate_effect_cost(&effect),
                resource_requirements: teg.extract_resource_requirements(&effect),
                resource_productions: teg.extract_resource_productions(&effect),
            };
            
            teg.add_node(node)?;
            
            // Add sequential dependency edge
            if let Some(prev) = previous_node {
                teg.add_edge(EffectEdge::CausalityLink {
                    from: prev,
                    to: node_id,
                    constraint: None,
                })?;
            }
            
            previous_node = Some(node_id);
        }
        
        Ok(teg)
    }
    
    /// Add a node to the TEG
    pub fn add_node(&mut self, node: EffectNode) -> Result<(), TegError> {
        let node_id = node.id;
        
        if self.nodes.contains_key(&node_id) {
            return Err(TegError::InvalidGraph("Duplicate node ID".to_string()));
        }
        
        // Update metadata
        self.metadata.total_cost += node.cost;
        
        // Add to adjacency lists
        self.adjacency_list.insert(node_id, Vec::new());
        self.reverse_adjacency_list.insert(node_id, Vec::new());
        
        self.nodes.insert(node_id, node);
        Ok(())
    }
    
    /// Add an edge to the TEG
    pub fn add_edge(&mut self, edge: EffectEdge) -> Result<(), TegError> {
        let (from, to) = match &edge {
            EffectEdge::CausalityLink { from, to, .. } => (*from, *to),
            EffectEdge::ResourceLink { from, to, .. } => (*from, *to),
            EffectEdge::ControlLink { from, to, .. } => (*from, *to),
        };
        
        // Verify nodes exist
        if !self.nodes.contains_key(&from) {
            return Err(TegError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(TegError::NodeNotFound(to));
        }
        
        // Update adjacency lists
        if let Some(adj) = self.adjacency_list.get_mut(&from) {
            if !adj.contains(&to) {
                adj.push(to);
            }
        }
        if let Some(rev_adj) = self.reverse_adjacency_list.get_mut(&to) {
            if !rev_adj.contains(&from) {
                rev_adj.push(from);
            }
        }
        
        self.edges.push(edge);
        Ok(())
    }
    
    /// Get nodes that are ready to execute
    pub fn get_ready_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|(id, node)| {
                if node.status == NodeStatus::Pending {
                    // Check if all dependencies are completed
                    let all_deps_completed = node.dependencies.iter().all(|dep_id| {
                        self.nodes.get(dep_id)
                            .map(|dep| dep.status == NodeStatus::Completed)
                            .unwrap_or(false)
                    });
                    
                    if all_deps_completed {
                        Some(*id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Extract resource requirements from an effect
    fn extract_resource_requirements(&self, effect: &EffectExpr) -> Vec<String> {
        match &effect.kind {
            EffectExprKind::Perform { effect_tag, .. } => {
                // Simple heuristic based on effect tag
                if effect_tag.contains("read") || effect_tag.contains("access") {
                    vec![format!("resource_{}", effect_tag)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
    
    /// Extract resource productions from an effect
    fn extract_resource_productions(&self, effect: &EffectExpr) -> Vec<String> {
        match &effect.kind {
            EffectExprKind::Perform { effect_tag, .. } => {
                // Simple heuristic based on effect tag
                if effect_tag.contains("create") || effect_tag.contains("produce") {
                    vec![format!("resource_{}", effect_tag)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
    
    /// Estimate the cost of executing an effect
    fn estimate_effect_cost(&self, effect: &EffectExpr) -> u64 {
        match &effect.kind {
            EffectExprKind::Pure(_) => 1,
            EffectExprKind::Bind { .. } => 2,
            EffectExprKind::Perform { .. } => 10,
            EffectExprKind::Handle { .. } => 5,
            _ => 1,
        }
    }
    
    /// Generate Mermaid diagram representation
    pub fn to_mermaid(&self) -> String {
        let mut result = String::from("graph TD\n");
        
        // Add nodes
        for (id, node) in &self.nodes {
            let node_label = format!("{}[{}]", 
                hex::encode(&id.bytes[0..4]), 
                node.effect.kind.to_string()
            );
            result.push_str(&format!("    {}\n", node_label));
        }
        
        // Add edges
        for edge in &self.edges {
            let (from, to) = match edge {
                EffectEdge::CausalityLink { from, to, .. } => (from, to),
                EffectEdge::ResourceLink { from, to, .. } => (from, to),
                EffectEdge::ControlLink { from, to, .. } => (from, to),
            };
            
            result.push_str(&format!(
                "    {} --> {}\n",
                hex::encode(&from.bytes[0..4]),
                hex::encode(&to.bytes[0..4])
            ));
        }
        
        result
    }
}

impl Default for TemporalEffectGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TegError::CyclicDependency(nodes) => 
                write!(f, "Cyclic dependency detected: {:?}", nodes),
            TegError::NodeNotFound(id) => 
                write!(f, "Node not found: {:?}", id),
            TegError::UnsatisfiableResource(resource) => 
                write!(f, "Unsatisfiable resource: {}", resource),
            TegError::InvalidGraph(msg) => 
                write!(f, "Invalid graph: {}", msg),
            TegError::ExecutionError(id, msg) => 
                write!(f, "Execution error for node {:?}: {}", id, msg),
        }
    }
}

impl std::error::Error for TegError {}

impl std::fmt::Display for EffectExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectExprKind::Pure(_) => write!(f, "Pure"),
            EffectExprKind::Bind { .. } => write!(f, "Bind"),
            EffectExprKind::Perform { effect_tag, .. } => write!(f, "Perform({})", effect_tag),
            EffectExprKind::Handle { .. } => write!(f, "Handle"),
            _ => write!(f, "Effect"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{Term, TermKind};
    
    #[test]
    fn test_teg_creation() {
        let teg = TemporalEffectGraph::new();
        assert!(teg.nodes.is_empty());
        assert!(teg.edges.is_empty());
        assert_eq!(teg.metadata.total_cost, 0);
    }
    
    #[test]
    fn test_effect_sequence_to_teg() {
        let effects = vec![
            EffectExpr::new(EffectExprKind::Pure(Term::new(TermKind::Unit))),
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "test".to_string(),
                args: vec![],
            }),
        ];
        
        let teg = TemporalEffectGraph::from_effect_sequence(effects).unwrap();
        assert_eq!(teg.nodes.len(), 2);
        assert_eq!(teg.edges.len(), 1);
    }
    
    #[test]
    fn test_ready_nodes() {
        let mut teg = TemporalEffectGraph::new();
        
        let effect1 = EffectExpr::new(EffectExprKind::Pure(Term::new(TermKind::Unit)));
        let effect2 = EffectExpr::new(EffectExprKind::Pure(Term::new(TermKind::Unit)));
        
        let node1_id = effect_to_entity_id(&effect1);
        let node2_id = effect_to_entity_id(&effect2);
        
        let node1 = EffectNode {
            id: node1_id,
            effect: effect1,
            status: NodeStatus::Pending,
            dependencies: vec![],
            results: None,
            cost: 1,
            resource_requirements: vec![],
            resource_productions: vec![],
        };
        
        let node2 = EffectNode {
            id: node2_id,
            effect: effect2,
            status: NodeStatus::Pending,
            dependencies: vec![node1_id],
            results: None,
            cost: 1,
            resource_requirements: vec![],
            resource_productions: vec![],
        };
        
        teg.add_node(node1).unwrap();
        teg.add_node(node2).unwrap();
        
        let ready = teg.get_ready_nodes();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], node1_id);
    }
} 