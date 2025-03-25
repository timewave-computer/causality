// Resource graph AST implementation
// Original file: src/ast/resource_graph.rs

// AST and Resource Graph Correspondence Module
//
// This module implements the bidirectional mapping between
// Abstract Syntax Tree nodes and resource allocations

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use crate::resource::{ResourceGrant, GrantId, ResourceRequest};
use causality_types::SourceLocation;

/// A unique identifier for an AST node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AstNodeId(String);

impl AstNodeId {
    /// Create a new AST node ID from a string
    pub fn new(id: String) -> Self {
        AstNodeId(id)
    }
    
    /// Get the string representation of this AST node ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AstNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// AST node type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNodeType {
    /// Program root node
    Program,
    /// Function definition
    Function,
    /// Control flow: Sequence
    Sequence,
    /// Control flow: Parallel
    Parallel,
    /// Control flow: Conditional
    Conditional,
    /// Control flow: Loop
    Loop,
    /// Variable definition
    VariableDefinition,
    /// Variable reference
    VariableReference,
    /// Effect application
    Effect,
    /// Resource allocation
    ResourceAllocation,
    /// Resource consumption
    ResourceConsumption,
    /// Domain-specific construct
    DomainExtension(String),
    /// Other node types
    Other(String),
}

/// Context for AST-based resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstContext {
    /// AST node ID responsible for the allocation
    pub ast_node_id: AstNodeId,
    /// Source code location
    pub source_location: Option<SourceLocation>,
    /// Controller label for cross-domain resources (if applicable)
    pub controller_label: Option<String>,
}

impl AstContext {
    /// Create a new AST context
    pub fn new(ast_node_id: AstNodeId) -> Self {
        AstContext {
            ast_node_id,
            source_location: None,
            controller_label: None,
        }
    }
    
    /// Add source location information
    pub fn with_source_location(mut self, location: SourceLocation) -> Self {
        self.source_location = Some(location);
        self
    }
    
    /// Add controller label for cross-domain resources
    pub fn with_controller(mut self, controller: String) -> Self {
        self.controller_label = Some(controller);
        self
    }
}

/// Resource delta for tracking resource creation and consumption
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    /// Memory delta in bytes
    pub memory_bytes: i64,
    /// CPU time delta in milliseconds
    pub cpu_millis: i64,
    /// IO operations delta
    pub io_operations: i64,
    /// Effect count delta
    pub effect_count: i64,
}

impl Delta {
    /// Create a new zero delta
    pub fn zero() -> Self {
        Delta {
            memory_bytes: 0,
            cpu_millis: 0,
            io_operations: 0,
            effect_count: 0,
        }
    }
    
    /// Create a delta from resource grant (positive delta)
    pub fn from_grant(grant: &ResourceGrant) -> Self {
        Delta {
            memory_bytes: grant.memory_bytes as i64,
            cpu_millis: grant.cpu_millis as i64,
            io_operations: grant.io_operations as i64,
            effect_count: grant.effect_count as i64,
        }
    }
    
    /// Create a negative delta from resource grant (consumption)
    pub fn consumption_from_grant(grant: &ResourceGrant) -> Self {
        Delta {
            memory_bytes: -(grant.memory_bytes as i64),
            cpu_millis: -(grant.cpu_millis as i64),
            io_operations: -(grant.io_operations as i64),
            effect_count: -(grant.effect_count as i64),
        }
    }
    
    /// Add another delta to this one
    pub fn add(&self, other: &Delta) -> Delta {
        Delta {
            memory_bytes: self.memory_bytes + other.memory_bytes,
            cpu_millis: self.cpu_millis + other.cpu_millis,
            io_operations: self.io_operations + other.io_operations,
            effect_count: self.effect_count + other.effect_count,
        }
    }
    
    /// Check if delta is zero (resources are balanced)
    pub fn is_zero(&self) -> bool {
        self.memory_bytes == 0 && 
        self.cpu_millis == 0 && 
        self.io_operations == 0 && 
        self.effect_count == 0
    }
}

/// Types of divergence between AST and resource graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivergenceType {
    /// A loop in AST becomes multiple allocations
    LoopUnrolling,
    /// A single node forks into parallel branches
    ConcurrentExecution,
    /// Function passed to another context
    HigherOrderDivergence,
    /// Effect handler causes non-local execution
    EffectHandlerJump,
    /// Resources reallocated to different AST nodes
    ResourceResharing,
    /// Resource moves between controllers
    ControllerTransition,
}

/// Point where AST and resource graphs diverge significantly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergencePoint {
    /// AST node where divergence occurs
    pub ast_node: AstNodeId,
    /// Resource grant IDs involved
    pub resource_ids: Vec<GrantId>,
    /// Type of divergence
    pub divergence_type: DivergenceType,
    /// How significant the divergence is (0.0-1.0)
    pub magnitude: f32,
    /// Resource delta imbalance if any
    pub delta_imbalance: Option<Delta>,
}

/// Transition of a resource between controllers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerTransition {
    /// Resource grant ID
    pub resource_id: GrantId,
    /// Source controller
    pub source_controller: String,
    /// Target controller
    pub target_controller: String,
    /// AST node responsible for transition
    pub ast_node: AstNodeId,
}

/// Correlation between AST and resource graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphCorrelation {
    /// Mapping from AST nodes to resources
    pub ast_to_resources: HashMap<AstNodeId, Vec<GrantId>>,
    /// Mapping from resources to AST nodes
    pub resource_to_ast: HashMap<GrantId, AstNodeId>,
    /// Points of significant divergence
    pub divergence_points: Vec<DivergencePoint>,
    /// Resource deltas by AST node
    pub resource_deltas: HashMap<AstNodeId, Delta>,
    /// Controller transitions
    pub controller_transitions: Vec<ControllerTransition>,
}

impl GraphCorrelation {
    /// Create a new empty graph correlation
    pub fn new() -> Self {
        GraphCorrelation {
            ast_to_resources: HashMap::new(),
            resource_to_ast: HashMap::new(),
            divergence_points: Vec::new(),
            resource_deltas: HashMap::new(),
            controller_transitions: Vec::new(),
        }
    }
    
    /// Record allocation of a resource by an AST node
    pub fn record_allocation(&mut self, ast_node_id: AstNodeId, grant_id: GrantId, grant: &ResourceGrant) -> Result<()> {
        // Add to AST -> Resource mapping
        self.ast_to_resources
            .entry(ast_node_id.clone())
            .or_insert_with(Vec::new)
            .push(grant_id.clone());
        
        // Add to Resource -> AST mapping
        self.resource_to_ast.insert(grant_id, ast_node_id.clone());
        
        // Update resource delta
        let delta = Delta::from_grant(grant);
        let node_delta = self.resource_deltas
            .entry(ast_node_id)
            .or_insert_with(Delta::zero);
        
        *node_delta = node_delta.add(&delta);
        
        Ok(())
    }
    
    /// Record consumption of a resource
    pub fn record_consumption(&mut self, ast_node_id: AstNodeId, grant_id: &GrantId, grant: &ResourceGrant) -> Result<()> {
        // Update resource delta
        let delta = Delta::consumption_from_grant(grant);
        let node_delta = self.resource_deltas
            .entry(ast_node_id)
            .or_insert_with(Delta::zero);
        
        *node_delta = node_delta.add(&delta);
        
        Ok(())
    }
    
    /// Find all resources allocated by a given AST node
    pub fn resources_for_ast_node(&self, ast_node_id: &AstNodeId) -> Vec<GrantId> {
        self.ast_to_resources
            .get(ast_node_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Find the AST node responsible for a resource allocation
    pub fn ast_node_for_resource(&self, grant_id: &GrantId) -> Option<AstNodeId> {
        self.resource_to_ast.get(grant_id).cloned()
    }
    
    /// Record controller transition
    pub fn record_controller_transition(
        &mut self,
        resource_id: GrantId,
        source_controller: String,
        target_controller: String,
        ast_node: AstNodeId,
    ) -> Result<()> {
        let transition = ControllerTransition {
            resource_id,
            source_controller,
            target_controller,
            ast_node,
        };
        
        self.controller_transitions.push(transition);
        
        Ok(())
    }
    
    /// Add a divergence point
    pub fn add_divergence_point(&mut self, divergence: DivergencePoint) {
        self.divergence_points.push(divergence);
    }
    
    /// Calculate total delta for a subtree
    pub fn subtree_delta(&self, ast_node_id: &AstNodeId, ast_tree: &HashMap<AstNodeId, Vec<AstNodeId>>) -> Delta {
        let mut total = self.resource_deltas
            .get(ast_node_id)
            .cloned()
            .unwrap_or_else(Delta::zero);
            
        // Add deltas from all children
        if let Some(children) = ast_tree.get(ast_node_id) {
            for child in children {
                let child_delta = self.subtree_delta(child, ast_tree);
                total = total.add(&child_delta);
            }
        }
        
        total
    }
    
    /// Validate resource conservation for a subtree
    pub fn validate_subtree_delta(&self, ast_node_id: &AstNodeId, ast_tree: &HashMap<AstNodeId, Vec<AstNodeId>>) -> Result<()> {
        let delta = self.subtree_delta(ast_node_id, ast_tree);
        
        if !delta.is_zero() {
            return Err(Error::ResourceImbalance(format!(
                "Resource imbalance detected in subtree starting at node {}: {:?}",
                ast_node_id, delta
            )));
        }
        
        Ok(())
    }
}

/// Tracker for AST-resource correlation
#[derive(Debug, Clone)]
pub struct CorrelationTracker {
    /// The correlation data
    correlation: Arc<RwLock<GraphCorrelation>>,
    /// AST structure (node ID to children mapping)
    ast_tree: Arc<RwLock<HashMap<AstNodeId, Vec<AstNodeId>>>>,
}

impl CorrelationTracker {
    /// Create a new correlation tracker
    pub fn new() -> Self {
        CorrelationTracker {
            correlation: Arc::new(RwLock::new(GraphCorrelation::new())),
            ast_tree: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register AST node structure
    pub fn register_ast_node(&self, node_id: AstNodeId, children: Vec<AstNodeId>) -> Result<()> {
        let mut tree = self.ast_tree.write().map_err(|_| Error::ConcurrencyError("Failed to acquire write lock".to_string()))?;
        tree.insert(node_id, children);
        Ok(())
    }
    
    /// Record resource allocation
    pub fn record_allocation(&self, ast_node_id: AstNodeId, grant_id: GrantId, grant: &ResourceGrant) -> Result<()> {
        let mut correlation = self.correlation.write().map_err(|_| Error::ConcurrencyError("Failed to acquire write lock".to_string()))?;
        correlation.record_allocation(ast_node_id, grant_id, grant)
    }
    
    /// Record resource consumption
    pub fn record_consumption(&self, ast_node_id: AstNodeId, grant_id: &GrantId, grant: &ResourceGrant) -> Result<()> {
        let mut correlation = self.correlation.write().map_err(|_| Error::ConcurrencyError("Failed to acquire write lock".to_string()))?;
        correlation.record_consumption(ast_node_id, grant_id, grant)
    }
    
    /// Record controller transition
    pub fn record_controller_transition(
        &self,
        resource_id: GrantId,
        source_controller: String,
        target_controller: String,
        ast_node: AstNodeId,
    ) -> Result<()> {
        let mut correlation = self.correlation.write().map_err(|_| Error::ConcurrencyError("Failed to acquire write lock".to_string()))?;
        correlation.record_controller_transition(resource_id, source_controller, target_controller, ast_node)
    }
    
    /// Get snapshot of current correlation
    pub fn get_correlation(&self) -> Result<GraphCorrelation> {
        let correlation = self.correlation.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        Ok(correlation.clone())
    }
    
    /// Find resources allocated by an AST node
    pub fn resources_for_ast_node(&self, ast_node_id: &AstNodeId) -> Result<Vec<GrantId>> {
        let correlation = self.correlation.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        Ok(correlation.resources_for_ast_node(ast_node_id))
    }
    
    /// Find AST node for a resource
    pub fn ast_node_for_resource(&self, grant_id: &GrantId) -> Result<Option<AstNodeId>> {
        let correlation = self.correlation.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        Ok(correlation.ast_node_for_resource(grant_id))
    }
    
    /// Validate resource conservation for a subtree
    pub fn validate_subtree_delta(&self, ast_node_id: &AstNodeId) -> Result<()> {
        let correlation = self.correlation.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        let tree = self.ast_tree.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        correlation.validate_subtree_delta(ast_node_id, &tree)
    }
    
    /// Find divergence points between AST and resource graph
    pub fn find_divergence_points(&self) -> Result<Vec<DivergencePoint>> {
        let correlation = self.correlation.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        Ok(correlation.divergence_points.clone())
    }
    
    /// Find controller transitions
    pub fn find_controller_transitions(&self) -> Result<Vec<ControllerTransition>> {
        let correlation = self.correlation.read().map_err(|_| Error::ConcurrencyError("Failed to acquire read lock".to_string()))?;
        Ok(correlation.controller_transitions.clone())
    }
}

/// Extension to ResourceGrant for AST attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributedResourceGrant {
    /// The base resource grant
    pub grant: ResourceGrant,
    /// AST node ID responsible for the allocation
    pub source_ast_node_id: Option<AstNodeId>,
    /// Source code location
    pub source_location: Option<SourceLocation>,
    /// Resource consumption delta
    pub consumption_delta: Delta,
    /// Controller label for cross-domain resources
    pub controller_label: Option<String>,
}

impl AttributedResourceGrant {
    /// Create a new attributed resource grant
    pub fn new(grant: ResourceGrant) -> Self {
        AttributedResourceGrant {
            grant,
            source_ast_node_id: None,
            source_location: None,
            consumption_delta: Delta::zero(),
            controller_label: None,
        }
    }
    
    /// Add AST context information
    pub fn with_ast_context(mut self, context: &AstContext) -> Self {
        self.source_ast_node_id = Some(context.ast_node_id.clone());
        self.source_location = context.source_location.clone();
        self.controller_label = context.controller_label.clone();
        self
    }
    
    /// Set consumption delta
    pub fn with_delta(mut self, delta: Delta) -> Self {
        self.consumption_delta = delta;
        self
    }
}

// Helper for visualization in DOT format
pub fn graph_to_dot(correlation: &GraphCorrelation) -> String {
    let mut result = String::new();
    
    // Start digraph
    result.push_str("digraph ast_resource_correlation {\n");
    result.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
    result.push_str("  rankdir=LR;\n\n");
    
    // Create subgraph for AST nodes
    result.push_str("  subgraph cluster_ast {\n");
    result.push_str("    label=\"Abstract Syntax Tree\";\n");
    result.push_str("    bgcolor=\"#EEEEEE\";\n");
    
    // Add AST nodes
    for (ast_id, resources) in &correlation.ast_to_resources {
        let delta = correlation.resource_deltas.get(ast_id).unwrap_or(&Delta::zero());
        let delta_str = if delta.is_zero() {
            "balanced".to_string()
        } else {
            format!("delta={:?}", delta)
        };
        
        result.push_str(&format!("    \"ast_{}\" [label=\"{}\\n{}\", fillcolor=\"#AACCFF\"];\n", 
            ast_id, ast_id, delta_str));
    }
    
    // End AST subgraph
    result.push_str("  }\n\n");
    
    // Create subgraph for resources
    result.push_str("  subgraph cluster_resources {\n");
    result.push_str("    label=\"Resource Graph\";\n");
    result.push_str("    bgcolor=\"#EEFFEE\";\n");
    
    // Add resource nodes
    for (grant_id, ast_id) in &correlation.resource_to_ast {
        result.push_str(&format!("    \"res_{}\" [label=\"{}\", fillcolor=\"#AAFFAA\"];\n", 
            grant_id, grant_id));
    }
    
    // End resource subgraph
    result.push_str("  }\n\n");
    
    // Add connections between AST and resources
    for (ast_id, resources) in &correlation.ast_to_resources {
        for resource_id in resources {
            result.push_str(&format!("  \"ast_{}\" -> \"res_{}\" [color=\"blue\"];\n", 
                ast_id, resource_id));
        }
    }
    
    // Highlight divergence points
    for divergence in &correlation.divergence_points {
        result.push_str(&format!("  \"ast_{}\" [fillcolor=\"#FFAAAA\", penwidth=2];\n", 
            divergence.ast_node));
        
        for resource_id in &divergence.resource_ids {
            result.push_str(&format!("  \"res_{}\" [fillcolor=\"#FFAAAA\", penwidth=2];\n", 
                resource_id));
        }
    }
    
    // Add controller transitions
    for transition in &correlation.controller_transitions {
        result.push_str(&format!("  \"res_{}\" [label=\"{}\\nTransition: {} -> {}\", fillcolor=\"#FFFFAA\"];\n", 
            transition.resource_id, transition.resource_id, 
            transition.source_controller, transition.target_controller));
    }
    
    // End digraph
    result.push_str("}\n");
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ast_node_id() {
        let id = AstNodeId::new("test-123".to_string());
        assert_eq!(id.as_str(), "test-123");
        assert_eq!(id.to_string(), "test-123");
    }
    
    #[test]
    fn test_delta_operations() {
        let delta1 = Delta {
            memory_bytes: 100,
            cpu_millis: 50,
            io_operations: 10,
            effect_count: 5,
        };
        
        let delta2 = Delta {
            memory_bytes: -30,
            cpu_millis: -20,
            io_operations: -5,
            effect_count: -2,
        };
        
        let sum = delta1.add(&delta2);
        assert_eq!(sum.memory_bytes, 70);
        assert_eq!(sum.cpu_millis, 30);
        assert_eq!(sum.io_operations, 5);
        assert_eq!(sum.effect_count, 3);
        
        assert!(!sum.is_zero());
        
        let zero = Delta::zero();
        assert!(zero.is_zero());
    }
    
    #[test]
    fn test_correlation_tracker() {
        let tracker = CorrelationTracker::new();
        
        // Register AST nodes
        let root = AstNodeId::new("root".to_string());
        let child1 = AstNodeId::new("child1".to_string());
        let child2 = AstNodeId::new("child2".to_string());
        
        tracker.register_ast_node(root.clone(), vec![child1.clone(), child2.clone()]).unwrap();
        tracker.register_ast_node(child1.clone(), vec![]).unwrap();
        tracker.register_ast_node(child2.clone(), vec![]).unwrap();
        
        // Create resource grants
        let grant1 = ResourceGrant::new(
            GrantId::from_string("grant1".to_string()),
            100, 50, 10, 5
        );
        
        let grant2 = ResourceGrant::new(
            GrantId::from_string("grant2".to_string()),
            200, 100, 20, 10
        );
        
        // Record allocations
        tracker.record_allocation(
            child1.clone(), 
            grant1.grant_id.clone(), 
            &grant1
        ).unwrap();
        
        tracker.record_allocation(
            child2.clone(), 
            grant2.grant_id.clone(), 
            &grant2
        ).unwrap();
        
        // Record consumptions
        tracker.record_consumption(
            root.clone(),
            &grant1.grant_id,
            &grant1
        ).unwrap();
        
        tracker.record_consumption(
            root.clone(),
            &grant2.grant_id,
            &grant2
        ).unwrap();
        
        // Validate conservation
        tracker.validate_subtree_delta(&root).unwrap();
        
        // Check resource lookup
        let resources = tracker.resources_for_ast_node(&child1).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0], grant1.grant_id);
        
        let ast_node = tracker.ast_node_for_resource(&grant2.grant_id).unwrap();
        assert!(ast_node.is_some());
        assert_eq!(ast_node.unwrap(), child2);
    }
} 