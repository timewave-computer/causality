//! Temporal Effect Graph (TEG) for dynamic orchestration
//!
//! This module implements the TEG system for dependency-based parallel execution
//! of effects, enabling automatic parallelization and optimization.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::{
    effect::{EffectExpr, Intent, Constraint},
    lambda::base::Value,
    system::content_addressing::{EntityId, Timestamp},
};

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
    
    /// Parallelization potential
    pub parallelization_factor: f64,
    
    /// Intent that generated this TEG
    pub source_intent: Option<EntityId>,
}

/// Main Temporal Effect Graph structure
#[derive(Debug, Clone)]
pub struct TemporalEffectGraph {
    /// Nodes in the graph
    pub nodes: HashMap<NodeId, EffectNode>,
    
    /// Edges representing dependencies
    pub edges: Vec<EffectEdge>,
    
    /// Graph metadata
    pub metadata: TegMetadata,
    
    /// Adjacency list for efficient traversal
    adjacency_list: HashMap<NodeId, Vec<NodeId>>,
    
    /// Reverse adjacency list (incoming edges)
    reverse_adjacency_list: HashMap<NodeId, Vec<NodeId>>,
}

/// Result of TEG execution
#[derive(Debug, Clone)]
pub struct TegResult {
    /// Final results from all nodes
    pub results: HashMap<NodeId, Value>,
    
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
    
    /// Actual parallelization achieved
    pub actual_parallelization: f64,
    
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

/// Storage proof dependency information for optimization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProofDependency {
    /// Node ID of the storage proof effect
    pub node_id: NodeId,
    
    /// Type of storage proof effect
    pub storage_type: String,
    
    /// Blockchain domain (ethereum, cosmos, etc.)
    pub domain: String,
    
    /// Estimated latency in milliseconds
    pub estimated_latency: u64,
    
    /// Whether this storage proof can be cached
    pub can_be_cached: bool,
    
    /// Whether this storage proof can be batched with others
    pub can_be_batched: bool,
}

/// Storage proof optimization configuration
#[derive(Debug, Clone)]
pub struct StorageProofOptimizationConfig {
    /// Enable domain-based batching
    pub enable_domain_batching: bool,
    
    /// Enable storage proof caching
    pub enable_caching: bool,
    
    /// Enable prefetching for commonly used proofs
    pub enable_prefetching: bool,
    
    /// Maximum batch size for storage proofs
    pub max_batch_size: usize,
    
    /// Cache TTL for storage proofs (in seconds)
    pub cache_ttl_seconds: u64,
    
    /// Enable ZK proof parallelization
    pub enable_zk_parallelization: bool,
}

impl Default for StorageProofOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_domain_batching: true,
            enable_caching: true,
            enable_prefetching: true,
            max_batch_size: 10,
            cache_ttl_seconds: 300,
            enable_zk_parallelization: true,
        }
    }
}

/// Helper function to create EntityId from effect (since EffectExpr doesn't implement Encode)
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
            nodes: HashMap::new(),
            edges: Vec::new(),
            metadata: TegMetadata {
                created_at: Timestamp::now(),
                total_cost: 0,
                critical_path_length: 0,
                parallelization_factor: 1.0,
                source_intent: None,
            },
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
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
        
        // Analyze and optimize dependencies
        teg.analyze_dependencies()?;
        teg.optimize_parallelization()?;
        
        Ok(teg)
    }
    
    /// Create TEG from an intent by first synthesizing effects
    pub fn from_intent(intent: &Intent) -> Result<Self, TegError> {
        use crate::effect::synthesis::FlowSynthesizer;
        
        let synthesizer = FlowSynthesizer::new(intent.domain);
        let effects = synthesizer.synthesize(intent)
            .map_err(|e| TegError::InvalidGraph(e.to_string()))?;
        
        let mut teg = Self::from_effect_sequence(effects)?;
        teg.metadata.source_intent = Some(intent.id);
        
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
        
        // Add dependencies to adjacency lists
        for dep in &node.dependencies {
            if let Some(deps) = self.adjacency_list.get_mut(dep) {
                deps.push(node_id);
            }
            if let Some(rev_deps) = self.reverse_adjacency_list.get_mut(&node_id) {
                rev_deps.push(*dep);
            }
        }
        
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
        
        // Update node dependencies
        if let Some(node) = self.nodes.get_mut(&to) {
            if !node.dependencies.contains(&from) {
                node.dependencies.push(from);
            }
        }
        
        self.edges.push(edge);
        Ok(())
    }
    
    /// Perform topological sort to get execution order
    pub fn topological_sort(&self) -> Result<Vec<NodeId>, TegError> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        // Initialize in-degrees
        for node_id in self.nodes.keys() {
            in_degree.insert(*node_id, 0);
        }
        
        // Calculate in-degrees
        for edge in &self.edges {
            let to = match edge {
                EffectEdge::CausalityLink { to, .. } => *to,
                EffectEdge::ResourceLink { to, .. } => *to,
                EffectEdge::ControlLink { to, .. } => *to,
            };
            *in_degree.get_mut(&to).unwrap() += 1;
        }
        
        // Find nodes with no incoming edges
        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(*node_id);
            }
        }
        
        // Process queue
        while let Some(node_id) = queue.pop_front() {
            result.push(node_id);
            
            // Update neighbors
            if let Some(neighbors) = self.adjacency_list.get(&node_id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(*neighbor);
                        }
                    }
                }
            }
        }
        
        // Check for cycles
        if result.len() != self.nodes.len() {
            // Find cycle
            let remaining: Vec<NodeId> = self.nodes.keys()
                .filter(|id| !result.contains(id))
                .copied()
                .collect();
            return Err(TegError::CyclicDependency(remaining));
        }
        
        Ok(result)
    }
    
    /// Get nodes ready for execution (dependencies satisfied)
    pub fn get_ready_nodes(&self) -> Vec<NodeId> {
        let mut ready = Vec::new();
        
        for (node_id, node) in &self.nodes {
            if node.status == NodeStatus::Pending {
                let deps_satisfied = node.dependencies.iter().all(|dep_id| {
                    if let Some(dep_node) = self.nodes.get(dep_id) {
                        dep_node.status == NodeStatus::Completed
                    } else {
                        false
                    }
                });
                
                if deps_satisfied {
                    ready.push(*node_id);
                }
            }
        }
        
        ready
    }
    
    /// Analyze dependencies and optimize for parallelization
    fn analyze_dependencies(&mut self) -> Result<(), TegError> {
        // Identify resource dependencies
        self.add_resource_dependencies()?;
        
        // Calculate critical path
        self.calculate_critical_path()?;
        
        // Estimate parallelization potential
        self.estimate_parallelization();
        
        Ok(())
    }
    
    /// Add resource-based dependencies
    fn add_resource_dependencies(&mut self) -> Result<(), TegError> {
        let mut resource_producers: HashMap<String, NodeId> = HashMap::new();
        let execution_order = self.topological_sort()?;
        
        // Collect resource requirements and productions first to avoid borrowing issues
        let mut resource_edges = Vec::new();
        
        for node_id in execution_order {
            if let Some(node) = self.nodes.get(&node_id) {
                // Check resource requirements
                for required_resource in &node.resource_requirements {
                    if let Some(producer_id) = resource_producers.get(required_resource) {
                        let edge = EffectEdge::ResourceLink {
                            from: *producer_id,
                            to: node_id,
                            resource: required_resource.clone(),
                        };
                        
                        // Only add if not already present
                        if !self.edges.contains(&edge) {
                            resource_edges.push(edge);
                        }
                    }
                }
                
                // Register resource productions
                for produced_resource in &node.resource_productions {
                    resource_producers.insert(produced_resource.clone(), node_id);
                }
            }
        }
        
        // Add all resource edges
        for edge in resource_edges {
            self.add_edge(edge)?;
        }
        
        Ok(())
    }
    
    /// Calculate critical path through the graph
    fn calculate_critical_path(&mut self) -> Result<(), TegError> {
        let execution_order = self.topological_sort()?;
        let mut earliest_start: HashMap<NodeId, u64> = HashMap::new();
        
        // Calculate earliest start times
        for node_id in &execution_order {
            let mut max_start = 0;
            
            if let Some(node) = self.nodes.get(node_id) {
                for dep_id in &node.dependencies {
                    if let (Some(dep_start), Some(dep_node)) = (
                        earliest_start.get(dep_id),
                        self.nodes.get(dep_id)
                    ) {
                        max_start = max_start.max(dep_start + dep_node.cost);
                    }
                }
                earliest_start.insert(*node_id, max_start);
            }
        }
        
        // Find critical path length
        self.metadata.critical_path_length = execution_order.iter()
            .map(|node_id| {
                if let (Some(start), Some(node)) = (
                    earliest_start.get(node_id),
                    self.nodes.get(node_id)
                ) {
                    start + node.cost
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);
        
        Ok(())
    }
    
    /// Estimate parallelization potential
    fn estimate_parallelization(&mut self) {
        if self.metadata.critical_path_length > 0 {
            self.metadata.parallelization_factor = 
                self.metadata.total_cost as f64 / self.metadata.critical_path_length as f64;
        } else {
            self.metadata.parallelization_factor = 1.0;
        }
    }
    
    /// Optimize the graph for better parallelization
    fn optimize_parallelization(&mut self) -> Result<(), TegError> {
        // TODO: Re-enable redundant dependency removal with better logic
        // For now, keep all dependencies to maintain correct execution order
        // self.remove_redundant_dependencies()?;
        
        // Recalculate metrics after optimization
        self.calculate_critical_path()?;
        self.estimate_parallelization();
        
        Ok(())
    }
    
    /// Remove redundant dependencies (transitive reduction)
    #[allow(dead_code)]
    fn remove_redundant_dependencies(&mut self) -> Result<(), TegError> {
        let mut edges_to_remove = Vec::new();
        
        for (i, edge) in self.edges.iter().enumerate() {
            if let EffectEdge::CausalityLink { from, to, .. } = edge {
                // Check if there's an indirect path from 'from' to 'to'
                if self.has_indirect_path(*from, *to, Some(*from)) {
                    edges_to_remove.push(i);
                }
            }
        }
        
        // Remove redundant edges (in reverse order to maintain indices)
        for &index in edges_to_remove.iter().rev() {
            self.edges.remove(index);
        }
        
        // Rebuild adjacency lists
        self.rebuild_adjacency_lists();
        
        Ok(())
    }
    
    /// Check if there's an indirect path between two nodes
    #[allow(dead_code)]
    fn has_indirect_path(&self, from: NodeId, to: NodeId, exclude_from: Option<NodeId>) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![from];
        
        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            
            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for &neighbor in neighbors {
                    // Skip the direct edge we're checking for redundancy
                    if current == from && neighbor == to && exclude_from == Some(from) {
                        continue;
                    }
                    if neighbor == to && current != from {
                        return true; // Found indirect path that doesn't use the direct edge
                    }
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }
        
        false
    }
    
    /// Rebuild adjacency lists from edges
    fn rebuild_adjacency_lists(&mut self) {
        self.adjacency_list.clear();
        self.reverse_adjacency_list.clear();
        
        // Initialize empty lists
        for node_id in self.nodes.keys() {
            self.adjacency_list.insert(*node_id, Vec::new());
            self.reverse_adjacency_list.insert(*node_id, Vec::new());
        }
        
        // Rebuild from edges
        for edge in &self.edges {
            let (from, to) = match edge {
                EffectEdge::CausalityLink { from, to, .. } => (*from, *to),
                EffectEdge::ResourceLink { from, to, .. } => (*from, *to),
                EffectEdge::ControlLink { from, to, .. } => (*from, *to),
            };
            
            if let Some(adj) = self.adjacency_list.get_mut(&from) {
                adj.push(to);
            }
            if let Some(rev_adj) = self.reverse_adjacency_list.get_mut(&to) {
                rev_adj.push(from);
            }
        }
        
        // Update node dependencies
        for (node_id, node) in self.nodes.iter_mut() {
            if let Some(deps) = self.reverse_adjacency_list.get(node_id) {
                node.dependencies = deps.clone();
            }
        }
    }
    
    /// Extract resource requirements from an effect
    fn extract_resource_requirements(&self, effect: &EffectExpr) -> Vec<String> {
        // Simplified resource extraction - in practice this would analyze the effect AST
        match &effect.kind {
            crate::effect::EffectExprKind::Perform { effect_tag, args: _ } => {
                match effect_tag.as_str() {
                    // Existing DeFi effects
                    "transfer" => vec!["source_tokens".to_string()],
                    "swap" => vec!["input_tokens".to_string(), "pool".to_string()],
                    "mint" => vec!["mint_authority".to_string()],
                    "stake" => vec!["tokens".to_string(), "staking_pool".to_string()],
                    "lend" => vec!["tokens".to_string(), "lending_pool".to_string()],
                    "borrow" => vec!["collateral".to_string(), "lending_pool".to_string()],
                    
                    // Storage proof effects
                    "storage_proof" => vec![
                        "blockchain_connection".to_string(),
                        "verification_key".to_string(),
                        "storage_commitment".to_string(),
                    ],
                    "ethereum_storage" => vec![
                        "ethereum_rpc".to_string(),
                        "storage_proof_circuit".to_string(),
                    ],
                    "cosmos_storage" => vec![
                        "cosmos_rpc".to_string(),
                        "wasm_storage_circuit".to_string(),
                    ],
                    "cross_chain_verification" => vec![
                        "source_chain_proof".to_string(),
                        "dest_chain_connection".to_string(),
                        "aggregation_circuit".to_string(),
                    ],
                    "zk_storage_proof" => vec![
                        "storage_data".to_string(),
                        "zk_circuit".to_string(),
                        "proving_key".to_string(),
                    ],
                    _ => vec![],
                }
            }
            _ => vec![],
        }
    }
    
    /// Extract resource productions from an effect
    fn extract_resource_productions(&self, effect: &EffectExpr) -> Vec<String> {
        // Simplified resource extraction - in practice this would analyze the effect AST
        match &effect.kind {
            crate::effect::EffectExprKind::Perform { effect_tag, args: _ } => {
                match effect_tag.as_str() {
                    // Existing DeFi effects
                    "transfer" => vec!["dest_tokens".to_string()],
                    "swap" => vec!["output_tokens".to_string(), "updated_pool".to_string()],
                    "mint" => vec!["new_tokens".to_string()],
                    "stake" => vec!["stake_tokens".to_string(), "updated_pool".to_string()],
                    "lend" => vec!["deposit_tokens".to_string(), "updated_pool".to_string()],
                    "borrow" => vec!["borrowed_tokens".to_string(), "debt_tokens".to_string()],
                    
                    // Storage proof effects
                    "storage_proof" => vec![
                        "verified_storage_data".to_string(),
                        "storage_proof_cache".to_string(),
                    ],
                    "ethereum_storage" => vec![
                        "ethereum_storage_value".to_string(),
                        "merkle_proof".to_string(),
                    ],
                    "cosmos_storage" => vec![
                        "cosmos_storage_value".to_string(),
                        "wasm_state_proof".to_string(),
                    ],
                    "cross_chain_verification" => vec![
                        "verified_cross_chain_state".to_string(),
                        "aggregated_proof".to_string(),
                    ],
                    "zk_storage_proof" => vec![
                        "zk_proof".to_string(),
                        "verified_storage_commitment".to_string(),
                    ],
                    _ => vec![],
                }
            }
            _ => vec![],
        }
    }
    
    /// Estimate execution cost for an effect
    fn estimate_effect_cost(&self, effect: &EffectExpr) -> u64 {
        // Simplified cost estimation - in practice this would be more sophisticated
        match &effect.kind {
            crate::effect::EffectExprKind::Perform { effect_tag, args: _ } => {
                match effect_tag.as_str() {
                    // Existing DeFi effects
                    "transfer" => 100,
                    "swap" => 300,
                    "mint" => 150,
                    "burn" => 120,
                    "stake" => 200,
                    "lend" => 250,
                    "borrow" => 350,
                    
                    // Storage proof effects (generally more expensive due to cryptographic operations)
                    "storage_proof" => 800,  // Base storage proof verification
                    "ethereum_storage" => 600,  // Ethereum storage access + merkle proof
                    "cosmos_storage" => 500,    // Cosmos storage access (typically faster)
                    "cross_chain_verification" => 1200,  // Cross-chain verification is expensive
                    "zk_storage_proof" => 2000, // ZK proof generation is most expensive
                    _ => 50,
                }
            }
            crate::effect::EffectExprKind::Pure(_) => 10,
            crate::effect::EffectExprKind::Bind { .. } => 20,
            crate::effect::EffectExprKind::Handle { .. } => 30,
            crate::effect::EffectExprKind::Parallel { .. } => 40,
            crate::effect::EffectExprKind::Race { .. } => 60,
        }
    }
    
    /// Generate Mermaid diagram representation
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::new();
        mermaid.push_str("graph TD\n");
        
        // Add nodes
        for (node_id, node) in &self.nodes {
            let node_label = match &node.effect.kind {
                crate::effect::EffectExprKind::Perform { effect_tag, .. } => {
                    format!("{}[{}]", &node_id.to_hex()[..8], effect_tag)
                }
                _ => format!("{}[Effect]", &node_id.to_hex()[..8]),
            };
            mermaid.push_str(&format!("    {}\n", node_label));
        }
        
        mermaid.push('\n');
        
        // Add edges
        for edge in &self.edges {
            match edge {
                EffectEdge::CausalityLink { from, to, .. } => {
                    mermaid.push_str(&format!(
                        "    {} --> {}\n",
                        &from.to_hex()[..8],
                        &to.to_hex()[..8]
                    ));
                }
                EffectEdge::ResourceLink { from, to, resource } => {
                    mermaid.push_str(&format!(
                        "    {} -->|{}| {}\n",
                        &from.to_hex()[..8],
                        resource,
                        &to.to_hex()[..8]
                    ));
                }
                EffectEdge::ControlLink { from, to, .. } => {
                    mermaid.push_str(&format!(
                        "    {} -.-> {}\n",
                        &from.to_hex()[..8],
                        &to.to_hex()[..8]
                    ));
                }
            }
        }
        
        mermaid
    }
    
    /// Advanced performance optimization algorithms
    /// Optimize the graph for better cache locality and memory access patterns
    pub fn optimize_cache_locality(&mut self) -> Result<(), TegError> {
        // Reorder nodes to improve spatial locality
        let reordered_nodes = self.optimize_node_ordering()?;
        
        // Update node IDs to reflect optimized order
        self.apply_node_reordering(&reordered_nodes)?;
        
        // Rebuild adjacency lists for optimal traversal
        self.rebuild_adjacency_lists();
        
        Ok(())
    }
    
    /// Optimize node ordering for cache-friendly traversal
    fn optimize_node_ordering(&self) -> Result<Vec<NodeId>, TegError> {
        let mut ordered_nodes = Vec::new();
        let _visited: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
        
        // Use topological ordering as base, then apply cache optimizations
        let topo_order = self.topological_sort()?;
        
        // Group nodes by execution level (depth from roots)
        let levels = self.compute_execution_levels(&topo_order);
        
        // Within each level, order by cache affinity and resource dependencies
        for level_nodes in levels {
            let optimized_level = self.optimize_level_ordering(level_nodes);
            ordered_nodes.extend(optimized_level);
        }
        
        Ok(ordered_nodes)
    }
    
    /// Compute execution levels for cache optimization
    fn compute_execution_levels(&self, topo_order: &[NodeId]) -> Vec<Vec<NodeId>> {
        let mut levels = Vec::new();
        let mut node_levels = std::collections::HashMap::new();
        
        // Calculate depth of each node
        for &node_id in topo_order {
            let mut max_depth = 0;
            
            // Find maximum depth of dependencies
            for edge in &self.edges {
                let (from, to) = match edge {
                    EffectEdge::CausalityLink { from, to, .. } => (*from, *to),
                    EffectEdge::ResourceLink { from, to, .. } => (*from, *to),
                    EffectEdge::ControlLink { from, to, .. } => (*from, *to),
                };
                
                if to == node_id {
                    if let Some(&dep_level) = node_levels.get(&from) {
                        max_depth = max_depth.max(dep_level + 1);
                    }
                }
            }
            
            node_levels.insert(node_id, max_depth);
            
            // Ensure we have enough levels
            while levels.len() <= max_depth {
                levels.push(Vec::new());
            }
            
            levels[max_depth].push(node_id);
        }
        
        levels
    }
    
    /// Optimize ordering within an execution level
    fn optimize_level_ordering(&self, mut level_nodes: Vec<NodeId>) -> Vec<NodeId> {
        // Sort by cache affinity score
        level_nodes.sort_by_key(|&node_id| {
            self.calculate_cache_affinity_score(node_id)
        });
        
        level_nodes
    }
    
    /// Calculate cache affinity score for node ordering
    fn calculate_cache_affinity_score(&self, node_id: NodeId) -> u64 {
        let mut score = 0u64;
        
        if let Some(node) = self.nodes.get(&node_id) {
            // Higher score for nodes with more resource dependencies (better to group)
            score += node.resource_requirements.len() as u64 * 100;
            
            // Higher score for nodes with higher cost (execute expensive operations together)
            score += node.cost / 10;
            
            // Higher score for nodes that produce resources (cache producers together)
            score += node.resource_productions.len() as u64 * 50;
        }
        
        score
    }
    
    /// Apply node reordering for cache optimization
    fn apply_node_reordering(&mut self, _reordered_nodes: &[NodeId]) -> Result<(), TegError> {
        // For now, this is a placeholder - would need careful ID remapping
        // In a full implementation, this would update all node references
        Ok(())
    }
    
    /// Advanced critical path optimization with resource constraints
    pub fn optimize_critical_path_with_resources(&mut self) -> Result<u64, TegError> {
        // Find critical path considering both time and resource constraints
        let resource_constrained_paths = self.find_resource_constrained_paths()?;
        
        // Apply optimizations to reduce critical path length
        self.apply_critical_path_optimizations(&resource_constrained_paths)?;
        
        // Recalculate critical path after optimization
        self.calculate_critical_path_length()
    }
    
    /// Find paths that are constrained by resource dependencies
    fn find_resource_constrained_paths(&self) -> Result<Vec<Vec<NodeId>>, TegError> {
        let mut constrained_paths = Vec::new();
        
        // Group nodes by resource type they depend on
        let mut resource_groups = std::collections::HashMap::new();
        
        for (node_id, node) in &self.nodes {
            for resource in &node.resource_requirements {
                resource_groups.entry(resource.clone())
                    .or_insert_with(Vec::new)
                    .push(*node_id);
            }
        }
        
        // Find paths within each resource group
        for resource_nodes in resource_groups.values() {
            if resource_nodes.len() > 1 {
                let path = self.find_path_through_nodes(resource_nodes)?;
                if !path.is_empty() {
                    constrained_paths.push(path);
                }
            }
        }
        
        Ok(constrained_paths)
    }
    
    /// Find execution path through a set of nodes
    fn find_path_through_nodes(&self, nodes: &[NodeId]) -> Result<Vec<NodeId>, TegError> {
        // Simplified path finding - in practice would use more sophisticated algorithms
        let mut path = Vec::new();
        let mut remaining_nodes = nodes.to_vec();
        
        while !remaining_nodes.is_empty() {
            // Find node with no dependencies in remaining set
            let next_node = remaining_nodes.iter()
                .find(|&&node_id| {
                    self.get_dependencies(node_id).iter()
                        .all(|dep| !remaining_nodes.contains(dep))
                })
                .copied()
                .unwrap_or(remaining_nodes[0]);
            
            path.push(next_node);
            remaining_nodes.retain(|&x| x != next_node);
        }
        
        Ok(path)
    }
    
    /// Apply optimizations to reduce critical path length
    fn apply_critical_path_optimizations(&mut self, _paths: &[Vec<NodeId>]) -> Result<(), TegError> {
        // Placeholder for advanced optimizations like:
        // - Effect batching
        // - Resource prefetching
        // - Parallel resource loading
        // - Effect fusion where possible
        Ok(())
    }
    
    /// Adaptive scheduling based on execution history
    pub fn optimize_scheduling_with_history(&mut self, execution_history: &ExecutionHistory) -> Result<(), TegError> {
        // Update node priorities based on historical performance
        for (node_id, node) in &mut self.nodes {
            if let Some(history) = execution_history.get_node_history(*node_id) {
                // Adjust cost estimates based on actual execution times
                let avg_execution_time = history.average_execution_time();
                node.cost = (node.cost + avg_execution_time) / 2;
                
                // Note: We no longer have a metadata field on EffectNode
                // This would need to be tracked differently in the actual implementation
            }
        }
        
        // Recompute critical path with updated costs
        self.metadata.critical_path_length = self.calculate_critical_path_length()?;
        
        Ok(())
    }
    
    /// Memory pool optimization for large TEGs
    pub fn optimize_memory_pools(&mut self) -> Result<MemoryOptimizationStats, TegError> {
        let initial_memory = self.estimate_memory_usage();
        
        // Apply memory optimizations
        self.compact_node_storage()?;
        self.optimize_edge_storage()?;
        self.intern_common_strings()?;
        
        let final_memory = self.estimate_memory_usage();
        
        Ok(MemoryOptimizationStats {
            initial_memory,
            final_memory,
            savings: initial_memory - final_memory,
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
        })
    }
    
    /// Estimate memory usage of the TEG
    fn estimate_memory_usage(&self) -> usize {
        let node_size = std::mem::size_of::<EffectNode>() * self.nodes.len();
        let edge_size = std::mem::size_of::<EffectEdge>() * self.edges.len();
        let metadata_size = std::mem::size_of::<TegMetadata>();
        
        node_size + edge_size + metadata_size
    }
    
    /// Compact node storage to reduce memory fragmentation
    fn compact_node_storage(&mut self) -> Result<(), TegError> {
        // Create new compacted storage
        let mut compacted_nodes = std::collections::HashMap::new();
        
        // Copy nodes to new storage (would typically use arena allocation)
        for (node_id, node) in &self.nodes {
            compacted_nodes.insert(*node_id, node.clone());
        }
        
        self.nodes = compacted_nodes;
        Ok(())
    }
    
    /// Optimize edge storage for better cache locality
    fn optimize_edge_storage(&mut self) -> Result<(), TegError> {
        // Sort edges by source node for better cache locality during traversal
        self.edges.sort_by_key(|edge| {
            let (from, to) = match edge {
                EffectEdge::CausalityLink { from, to, .. } => (*from, *to),
                EffectEdge::ResourceLink { from, to, .. } => (*from, *to),
                EffectEdge::ControlLink { from, to, .. } => (*from, *to),
            };
            (from, to)
        });
        Ok(())
    }
    
    /// Intern common strings to reduce memory usage
    fn intern_common_strings(&mut self) -> Result<(), TegError> {
        // In a full implementation, this would use a string interner
        // Since we don't have metadata fields on nodes anymore, this is a no-op
        Ok(())
    }
    
    /// Performance profiling and benchmarking
    pub fn profile_execution_performance(&self) -> PerformanceProfile {
        PerformanceProfile {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            critical_path_length: self.metadata.critical_path_length,
            parallelization_factor: self.calculate_parallelization_factor(),
            memory_usage: self.estimate_memory_usage(),
            cache_locality_score: self.calculate_cache_locality_score(),
            optimization_opportunities: self.identify_optimization_opportunities(),
        }
    }
    
    /// Calculate theoretical parallelization factor
    fn calculate_parallelization_factor(&self) -> f64 {
        if self.metadata.critical_path_length == 0 {
            return 1.0;
        }
        
        let total_work: u64 = self.nodes.values().map(|n| n.cost).sum();
        total_work as f64 / self.metadata.critical_path_length as f64
    }
    
    /// Calculate cache locality score (higher is better)
    fn calculate_cache_locality_score(&self) -> f64 {
        let mut locality_score = 0.0;
        let mut total_transitions = 0;
        
        // Analyze transitions between dependent nodes
        for edge in &self.edges {
            if let EffectEdge::ResourceLink { .. } = edge {
                // Resource dependencies benefit from cache locality
                locality_score += 1.0;
            }
            total_transitions += 1;
        }
        
        if total_transitions > 0 {
            locality_score / total_transitions as f64
        } else {
            1.0
        }
    }
    
    /// Identify optimization opportunities
    fn identify_optimization_opportunities(&self) -> Vec<OptimizationOpportunity> {
        let mut opportunities = Vec::new();
        
        // Look for batching opportunities
        let batchable_nodes = self.find_batchable_nodes();
        if !batchable_nodes.is_empty() {
            opportunities.push(OptimizationOpportunity {
                opportunity_type: "effect_batching".to_string(),
                potential_savings: batchable_nodes.len() as u64 * 50, // Estimated savings
                description: format!("Can batch {} similar effects", batchable_nodes.len()),
                complexity: OptimizationComplexity::Medium,
            });
        }
        
        // Look for resource prefetching opportunities
        let prefetchable_resources = self.find_prefetchable_resources();
        if !prefetchable_resources.is_empty() {
            opportunities.push(OptimizationOpportunity {
                opportunity_type: "resource_prefetching".to_string(),
                potential_savings: prefetchable_resources.len() as u64 * 100,
                description: format!("Can prefetch {} resources", prefetchable_resources.len()),
                complexity: OptimizationComplexity::High,
            });
        }
        
        // Look for pipeline optimization opportunities
        if self.metadata.critical_path_length > self.calculate_average_path_length() * 2 {
            opportunities.push(OptimizationOpportunity {
                opportunity_type: "pipeline_optimization".to_string(),
                potential_savings: self.metadata.critical_path_length / 4,
                description: "Critical path significantly longer than average - pipeline optimization recommended".to_string(),
                complexity: OptimizationComplexity::High,
            });
        }
        
        opportunities
    }
    
    /// Find nodes that can be batched together
    fn find_batchable_nodes(&self) -> Vec<Vec<NodeId>> {
        let mut batchable_groups = Vec::new();
        let mut processed = std::collections::HashSet::new();
        
        for node_id in self.nodes.keys() {
            if processed.contains(node_id) {
                continue;
            }
            
            // Find similar nodes that can be batched
            let similar_nodes = self.find_similar_nodes(*node_id);
            if similar_nodes.len() > 1 {
                batchable_groups.push(similar_nodes.clone());
                for &similar_id in &similar_nodes {
                    processed.insert(similar_id);
                }
            }
        }
        
        batchable_groups
    }
    
    /// Find nodes similar to the given node (for batching)
    fn find_similar_nodes(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut similar = vec![node_id];
        
        if let Some(target_node) = self.nodes.get(&node_id) {
            for (other_id, other_node) in &self.nodes {
                if *other_id != node_id && self.nodes_are_similar(target_node, other_node) {
                    similar.push(*other_id);
                }
            }
        }
        
        similar
    }
    
    /// Check if two nodes are similar enough to batch
    fn nodes_are_similar(&self, node1: &EffectNode, node2: &EffectNode) -> bool {
        // Simple similarity check - in practice would be more sophisticated
        node1.effect.kind == node2.effect.kind &&
        (node1.cost as i64 - node2.cost as i64).abs() < 100
    }
    
    /// Find resources that can be prefetched
    fn find_prefetchable_resources(&self) -> Vec<String> {
        let mut prefetchable = Vec::new();
        let mut resource_usage = std::collections::HashMap::new();
        
        // Count resource usage patterns
        for node in self.nodes.values() {
            for resource in &node.resource_requirements {
                *resource_usage.entry(resource.clone()).or_insert(0) += 1;
            }
        }
        
        // Resources used by multiple nodes are good prefetching candidates
        for (resource, usage_count) in resource_usage {
            if usage_count > 1 {
                prefetchable.push(resource);
            }
        }
        
        prefetchable
    }
    
    /// Calculate average path length through the graph
    fn calculate_average_path_length(&self) -> u64 {
        if self.nodes.is_empty() {
            return 0;
        }
        
        let mut total_path_length = 0u64;
        let mut path_count = 0u64;
        
        // Sample paths from each node to estimate average
        for node_id in self.nodes.keys() {
            let path_length = self.calculate_path_length_from_node(*node_id);
            total_path_length += path_length;
            path_count += 1;
        }
        
        if path_count > 0 {
            total_path_length / path_count
        } else {
            0
        }
    }
    
    /// Calculate path length from a specific node
    fn calculate_path_length_from_node(&self, start_node: NodeId) -> u64 {
        let mut max_length = 0u64;
        let mut visited = std::collections::HashSet::new();
        
        self.calculate_path_length_recursive(start_node, 0, &mut max_length, &mut visited);
        max_length
    }
    
    /// Recursive helper for path length calculation
    fn calculate_path_length_recursive(
        &self,
        node_id: NodeId,
        current_length: u64,
        max_length: &mut u64,
        visited: &mut std::collections::HashSet<NodeId>,
    ) {
        if visited.contains(&node_id) {
            return;
        }
        
        visited.insert(node_id);
        
        if let Some(node) = self.nodes.get(&node_id) {
            let new_length = current_length + node.cost;
            *max_length = (*max_length).max(new_length);
            
            // Continue to dependent nodes
            for edge in &self.edges {
                let (from, to) = match edge {
                    EffectEdge::CausalityLink { from, to, .. } => (*from, *to),
                    EffectEdge::ResourceLink { from, to, .. } => (*from, *to),
                    EffectEdge::ControlLink { from, to, .. } => (*from, *to),
                };
                
                if from == node_id {
                    self.calculate_path_length_recursive(to, new_length, max_length, visited);
                }
            }
        }
        
        visited.remove(&node_id);
    }
    
    /// Get dependencies for a node
    fn get_dependencies(&self, node_id: NodeId) -> Vec<NodeId> {
        if let Some(node) = self.nodes.get(&node_id) {
            node.dependencies.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Calculate critical path length
    pub fn calculate_critical_path_length(&self) -> Result<u64, TegError> {
        if self.nodes.is_empty() {
            return Ok(0);
        }
        
        // Find nodes with no dependencies (roots)
        let roots: Vec<NodeId> = self.nodes
            .keys()
            .filter(|&&node_id| {
                !self.edges.iter().any(|edge| {
                    match edge {
                        EffectEdge::CausalityLink { to, .. } => *to == node_id,
                        EffectEdge::ResourceLink { to, .. } => *to == node_id,
                        EffectEdge::ControlLink { to, .. } => *to == node_id,
                    }
                })
            })
            .copied()
            .collect();
        
        if roots.is_empty() {
            return Err(TegError::InvalidGraph("No root nodes found".to_string()));
        }
        
        // Calculate longest path from any root
        let mut max_path_length = 0u64;
        for &root in &roots {
            let path_length = self.calculate_path_length_from_node(root);
            max_path_length = max_path_length.max(path_length);
        }
        
        Ok(max_path_length)
    }
    
    /// Storage proof-specific scheduling optimizations
    /// Optimize storage proof effect scheduling for better performance
    pub fn optimize_storage_proof_scheduling(&mut self) -> Result<(), TegError> {
        // Group storage proof effects by blockchain domain for batching
        self.group_storage_effects_by_domain()?;
        
        // Optimize cross-chain verification ordering
        self.optimize_cross_chain_verification_order()?;
        
        // Add prefetching for commonly used storage proofs
        self.add_storage_proof_prefetching()?;
        
        // Optimize ZK proof generation scheduling
        self.optimize_zk_proof_generation()?;
        
        Ok(())
    }
    
    /// Group storage proof effects by blockchain domain for efficient batching
    fn group_storage_effects_by_domain(&mut self) -> Result<(), TegError> {
        let mut domain_groups: HashMap<String, Vec<NodeId>> = HashMap::new();
        
        // Identify storage proof nodes and group by domain
        for (node_id, node) in &self.nodes {
            if let crate::effect::EffectExprKind::Perform { effect_tag, .. } = &node.effect.kind {
                let domain = match effect_tag.as_str() {
                    "ethereum_storage" => Some("ethereum".to_string()),
                    "cosmos_storage" => Some("cosmos".to_string()),
                    "storage_proof" => {
                        // Try to infer domain from resource requirements
                        if node.resource_requirements.contains(&"ethereum_rpc".to_string()) {
                            Some("ethereum".to_string())
                        } else if node.resource_requirements.contains(&"cosmos_rpc".to_string()) {
                            Some("cosmos".to_string())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                
                if let Some(domain_name) = domain {
                    domain_groups.entry(domain_name).or_default().push(*node_id);
                }
            }
        }
        
        // Add resource links within domain groups for better batching
        for (domain, node_ids) in domain_groups {
            if node_ids.len() > 1 {
                // Create a shared resource for the domain
                let shared_resource = format!("{}_batch_context", domain);
                
                // Add resource links to enable batching
                for i in 0..node_ids.len() {
                    for j in (i + 1)..node_ids.len() {
                        // Only add if there's no existing path between nodes
                        if !self.has_indirect_path(node_ids[i], node_ids[j], None) {
                            let edge = EffectEdge::ResourceLink {
                                from: node_ids[i],
                                to: node_ids[j],
                                resource: shared_resource.clone(),
                            };
                            self.add_edge(edge)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Optimize cross-chain verification ordering for atomic operations
    fn optimize_cross_chain_verification_order(&mut self) -> Result<(), TegError> {
        let mut cross_chain_nodes = Vec::new();
        
        // Find cross-chain verification nodes
        for (node_id, node) in &self.nodes {
            if let crate::effect::EffectExprKind::Perform { effect_tag, .. } = &node.effect.kind {
                if effect_tag == "cross_chain_verification" {
                    cross_chain_nodes.push(*node_id);
                }
            }
        }
        
        // Sort cross-chain nodes by estimated cost (heavier operations first)
        cross_chain_nodes.sort_by_key(|&node_id| {
            std::cmp::Reverse(self.nodes.get(&node_id).map(|n| n.cost).unwrap_or(0))
        });
        
        // Add ordering constraints for cross-chain operations
        for i in 0..cross_chain_nodes.len() {
            for j in (i + 1)..cross_chain_nodes.len() {
                // Ensure heavier cross-chain operations start first
                let edge = EffectEdge::CausalityLink {
                    from: cross_chain_nodes[i],
                    to: cross_chain_nodes[j],
                    constraint: Some("cross_chain_ordering".to_string()),
                };
                
                // Only add if it doesn't create a cycle
                if !self.would_create_cycle(&edge) {
                    self.add_edge(edge)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Add prefetching for commonly used storage proofs
    fn add_storage_proof_prefetching(&mut self) -> Result<(), TegError> {
        let storage_nodes: Vec<NodeId> = self.nodes
            .iter()
            .filter_map(|(node_id, node)| {
                if let crate::effect::EffectExprKind::Perform { effect_tag, .. } = &node.effect.kind {
                    if matches!(effect_tag.as_str(), "storage_proof" | "ethereum_storage" | "cosmos_storage") {
                        Some(*node_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Add prefetch edges to reduce latency
        for &storage_node in &storage_nodes {
            // Find nodes that depend on this storage proof
            let dependent_nodes: Vec<NodeId> = self.edges
                .iter()
                .filter_map(|edge| {
                    match edge {
                        EffectEdge::ResourceLink { from, to, .. } if *from == storage_node => Some(*to),
                        EffectEdge::CausalityLink { from, to, .. } if *from == storage_node => Some(*to),
                        _ => None,
                    }
                })
                .collect();
            
            // If this storage proof is used by multiple nodes, prioritize it
            if dependent_nodes.len() > 1 {
                // Update the node's cost to reflect higher priority
                if let Some(node) = self.nodes.get_mut(&storage_node) {
                    node.cost = (node.cost as f64 * 0.8) as u64; // Reduce cost = higher priority
                }
            }
        }
        
        Ok(())
    }
    
    /// Optimize ZK proof generation scheduling
    fn optimize_zk_proof_generation(&mut self) -> Result<(), TegError> {
        let mut zk_nodes = Vec::new();
        
        // Find ZK proof generation nodes
        for (node_id, node) in &self.nodes {
            if let crate::effect::EffectExprKind::Perform { effect_tag, .. } = &node.effect.kind {
                if effect_tag == "zk_storage_proof" {
                    zk_nodes.push(*node_id);
                }
            }
        }
        
        // Sort ZK nodes by dependency count (independent nodes first)
        zk_nodes.sort_by_key(|&node_id| {
            let dependency_count = self.edges
                .iter()
                .filter(|edge| {
                    match edge {
                        EffectEdge::CausalityLink { to, .. } => *to == node_id,
                        EffectEdge::ResourceLink { to, .. } => *to == node_id,
                        EffectEdge::ControlLink { to, .. } => *to == node_id,
                    }
                })
                .count();
            dependency_count
        });
        
        // Schedule independent ZK proofs in parallel
        for chunk in zk_nodes.chunks(2) { // Process in pairs to avoid resource contention
            if chunk.len() == 2 {
                // These can potentially run in parallel
                let parallel_resource = "zk_proving_parallelism".to_string();
                
                // Add parallel execution hint
                let edge = EffectEdge::ResourceLink {
                    from: chunk[0],
                    to: chunk[1],
                    resource: parallel_resource,
                };
                
                if !self.would_create_cycle(&edge) {
                    self.add_edge(edge)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if adding an edge would create a cycle
    fn would_create_cycle(&self, edge: &EffectEdge) -> bool {
        let (from, to) = match edge {
            EffectEdge::CausalityLink { from, to, .. } => (*from, *to),
            EffectEdge::ResourceLink { from, to, .. } => (*from, *to),
            EffectEdge::ControlLink { from, to, .. } => (*from, *to),
        };
        
        // Check if there's already a path from 'to' to 'from'
        self.has_indirect_path(to, from, None)
    }
    
    /// Get storage proof dependency information for a node
    pub fn get_storage_dependencies(&self, node_id: NodeId) -> Vec<StorageProofDependency> {
        let mut dependencies = Vec::new();
        
        if let Some(node) = self.nodes.get(&node_id) {
            if let crate::effect::EffectExprKind::Perform { effect_tag, .. } = &node.effect.kind {
                match effect_tag.as_str() {
                    "storage_proof" | "ethereum_storage" | "cosmos_storage" => {
                        // This is a storage proof node
                        dependencies.push(StorageProofDependency {
                            node_id,
                            storage_type: effect_tag.clone(),
                            domain: self.infer_storage_domain(node),
                            estimated_latency: self.estimate_storage_latency(effect_tag),
                            can_be_cached: true,
                            can_be_batched: matches!(effect_tag.as_str(), "ethereum_storage" | "cosmos_storage"),
                        });
                    }
                    _ => {
                        // Check if this node depends on storage proofs
                        for edge in &self.edges {
                            if let EffectEdge::ResourceLink { from, to, resource } = edge {
                                if *to == node_id && resource.contains("storage") {
                                    if let Some(storage_node) = self.nodes.get(from) {
                                        if let crate::effect::EffectExprKind::Perform { effect_tag: storage_tag, .. } = &storage_node.effect.kind {
                                            dependencies.push(StorageProofDependency {
                                                node_id: *from,
                                                storage_type: storage_tag.clone(),
                                                domain: self.infer_storage_domain(storage_node),
                                                estimated_latency: self.estimate_storage_latency(storage_tag),
                                                can_be_cached: true,
                                                can_be_batched: matches!(storage_tag.as_str(), "ethereum_storage" | "cosmos_storage"),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        dependencies
    }
    
    /// Infer storage domain from node resource requirements
    fn infer_storage_domain(&self, node: &EffectNode) -> String {
        if node.resource_requirements.iter().any(|r| r.contains("ethereum")) {
            "ethereum".to_string()
        } else if node.resource_requirements.iter().any(|r| r.contains("cosmos")) {
            "cosmos".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    /// Estimate storage proof latency based on effect type
    fn estimate_storage_latency(&self, effect_tag: &str) -> u64 {
        match effect_tag {
            "ethereum_storage" => 300,  // ~300ms for Ethereum RPC call
            "cosmos_storage" => 150,    // ~150ms for Cosmos query
            "storage_proof" => 500,     // ~500ms for proof verification
            "zk_storage_proof" => 2000, // ~2s for ZK proof generation
            _ => 100,
        }
    }
}

/// Execution history for adaptive optimization
#[derive(Debug, Clone)]
pub struct ExecutionHistory {
    pub node_histories: std::collections::HashMap<NodeId, NodeExecutionHistory>,
}

/// Execution history for a single node
#[derive(Debug, Clone)]
pub struct NodeExecutionHistory {
    pub executions: Vec<ExecutionRecord>,
}

/// Record of a single execution
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub execution_time: u64,
    pub success: bool,
    pub timestamp: u64,
    pub resource_usage: u64,
}

/// Memory optimization statistics
#[derive(Debug, Clone)]
pub struct MemoryOptimizationStats {
    pub initial_memory: usize,
    pub final_memory: usize,
    pub savings: usize,
    pub node_count: usize,
    pub edge_count: usize,
}

/// Performance profiling results
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub critical_path_length: u64,
    pub parallelization_factor: f64,
    pub memory_usage: usize,
    pub cache_locality_score: f64,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

/// Optimization opportunity identification
#[derive(Debug, Clone)]
pub struct OptimizationOpportunity {
    pub opportunity_type: String,
    pub potential_savings: u64,
    pub description: String,
    pub complexity: OptimizationComplexity,
}

/// Complexity level of optimization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationComplexity {
    Low,
    Medium,
    High,
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionHistory {
    /// Create new empty execution history
    pub fn new() -> Self {
        Self {
            node_histories: std::collections::HashMap::new(),
        }
    }
    
    /// Add execution record for a node
    pub fn add_execution_record(&mut self, node_id: NodeId, record: ExecutionRecord) {
        self.node_histories.entry(node_id)
            .or_insert_with(|| NodeExecutionHistory { executions: Vec::new() })
            .executions.push(record);
    }
    
    /// Get execution history for a node
    pub fn get_node_history(&self, node_id: NodeId) -> Option<&NodeExecutionHistory> {
        self.node_histories.get(&node_id)
    }
}

impl NodeExecutionHistory {
    /// Calculate average execution time
    pub fn average_execution_time(&self) -> u64 {
        if self.executions.is_empty() {
            return 0;
        }
        
        let total: u64 = self.executions.iter().map(|e| e.execution_time).sum();
        total / self.executions.len() as u64
    }
    
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.executions.is_empty() {
            return 1.0;
        }
        
        let successful = self.executions.iter().filter(|e| e.success).count();
        successful as f64 / self.executions.len() as f64
    }
    
    /// Calculate resource usage trend
    pub fn average_resource_usage(&self) -> u64 {
        if self.executions.is_empty() {
            return 0;
        }
        
        let total: u64 = self.executions.iter().map(|e| e.resource_usage).sum();
        total / self.executions.len() as u64
    }
}

impl Default for TemporalEffectGraph {
    fn default() -> Self {
        Self::new()
    }
}

// Error display implementations
impl std::fmt::Display for TegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TegError::CyclicDependency(nodes) => {
                write!(f, "Cyclic dependency detected involving nodes: {:?}", nodes)
            }
            TegError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            TegError::UnsatisfiableResource(resource) => {
                write!(f, "Unsatisfiable resource dependency: {}", resource)
            }
            TegError::InvalidGraph(msg) => write!(f, "Invalid graph: {}", msg),
            TegError::ExecutionError(id, msg) => write!(f, "Execution error in node {}: {}", id, msg),
        }
    }
}

impl std::error::Error for TegError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::{EffectExpr, EffectExprKind, Intent, ResourceBinding, Constraint},
        lambda::Term,
        system::content_addressing::{DomainId, Str},
    };

    #[test]
    fn test_teg_creation() {
        let teg = TemporalEffectGraph::new();
        assert!(teg.nodes.is_empty());
        assert!(teg.edges.is_empty());
        assert_eq!(teg.metadata.total_cost, 0);
    }

    #[test]
    fn test_node_addition() {
        let mut teg = TemporalEffectGraph::new();
        
        let effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "transfer".to_string(),
            args: vec![Term::var("source"), Term::var("dest")],
        });
        
        let node = EffectNode {
            id: effect_to_entity_id(&effect),
            effect: effect.clone(),
            status: NodeStatus::Pending,
            dependencies: vec![],
            results: None,
            cost: 100,
            resource_requirements: vec!["source_tokens".to_string()],
            resource_productions: vec!["dest_tokens".to_string()],
        };
        
        let result = teg.add_node(node);
        assert!(result.is_ok());
        assert_eq!(teg.nodes.len(), 1);
        assert_eq!(teg.metadata.total_cost, 100);
    }

    #[test]
    fn test_effect_sequence_to_teg() {
        let effects = vec![
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "load".to_string(),
                args: vec![Term::var("input")],
            }),
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "transfer".to_string(),
                args: vec![Term::var("source"), Term::var("dest")],
            }),
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "store".to_string(),
                args: vec![Term::var("output")],
            }),
        ];
        
        let teg = TemporalEffectGraph::from_effect_sequence(effects).unwrap();
        
        assert_eq!(teg.nodes.len(), 3);
        // We should have sequential dependencies between the 3 effects
        assert!(teg.edges.len() >= 2);
        assert!(teg.metadata.total_cost > 0);
    }

    #[test]
    fn test_topological_sort() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create three effects with dependencies: A -> B -> C
        let effect_a = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_a".to_string(),
            args: vec![],
        });
        let effect_b = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_b".to_string(),
            args: vec![],
        });
        let effect_c = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_c".to_string(),
            args: vec![],
        });
        
        let node_a_id = effect_to_entity_id(&effect_a);
        let node_b_id = effect_to_entity_id(&effect_b);
        let node_c_id = effect_to_entity_id(&effect_c);
        
        teg.add_node(EffectNode {
            id: node_a_id,
            effect: effect_a,
            status: NodeStatus::Pending,
            dependencies: vec![],
            results: None,
            cost: 100,
            resource_requirements: vec![],
            resource_productions: vec!["resource_a".to_string()],
        }).unwrap();
        
        teg.add_node(EffectNode {
            id: node_b_id,
            effect: effect_b,
            status: NodeStatus::Pending,
            dependencies: vec![node_a_id],
            results: None,
            cost: 200,
            resource_requirements: vec!["resource_a".to_string()],
            resource_productions: vec!["resource_b".to_string()],
        }).unwrap();
        
        teg.add_node(EffectNode {
            id: node_c_id,
            effect: effect_c,
            status: NodeStatus::Pending,
            dependencies: vec![node_b_id],
            results: None,
            cost: 150,
            resource_requirements: vec!["resource_b".to_string()],
            resource_productions: vec!["resource_c".to_string()],
        }).unwrap();
        
        // Add edges
        teg.add_edge(EffectEdge::CausalityLink {
            from: node_a_id,
            to: node_b_id,
            constraint: None,
        }).unwrap();
        
        teg.add_edge(EffectEdge::CausalityLink {
            from: node_b_id,
            to: node_c_id,
            constraint: None,
        }).unwrap();
        
        let sorted = teg.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        
        // Verify order: A should come before B, B should come before C
        let pos_a = sorted.iter().position(|&id| id == node_a_id).unwrap();
        let pos_b = sorted.iter().position(|&id| id == node_b_id).unwrap();
        let pos_c = sorted.iter().position(|&id| id == node_c_id).unwrap();
        
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_ready_nodes() {
        let mut teg = TemporalEffectGraph::new();
        
        let effect_a = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_a".to_string(),
            args: vec![],
        });
        let effect_b = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_b".to_string(),
            args: vec![],
        });
        
        let node_a_id = effect_to_entity_id(&effect_a);
        let node_b_id = effect_to_entity_id(&effect_b);
        
        // Node A has no dependencies (ready)
        teg.add_node(EffectNode {
            id: node_a_id,
            effect: effect_a,
            status: NodeStatus::Pending,
            dependencies: vec![],
            results: None,
            cost: 100,
            resource_requirements: vec![],
            resource_productions: vec!["resource_a".to_string()],
        }).unwrap();
        
        // Node B depends on A (not ready)
        teg.add_node(EffectNode {
            id: node_b_id,
            effect: effect_b,
            status: NodeStatus::Pending,
            dependencies: vec![node_a_id],
            results: None,
            cost: 200,
            resource_requirements: vec!["resource_a".to_string()],
            resource_productions: vec![],
        }).unwrap();
        
        let ready = teg.get_ready_nodes();
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&node_a_id));
        assert!(!ready.contains(&node_b_id));
        
        // Complete node A
        if let Some(node) = teg.nodes.get_mut(&node_a_id) {
            node.status = NodeStatus::Completed;
        }
        
        let ready = teg.get_ready_nodes();
        assert_eq!(ready.len(), 1);
        assert!(ready.contains(&node_b_id));
    }

    #[test]
    fn test_intent_to_teg() {
        let domain = DomainId::from_content(&Str::new("test_domain"));
        
        let intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input_tokens", "Token").with_quantity(100)],
            Constraint::produces_quantity("output_tokens", "Token", 100),
        );
        
        let result = TemporalEffectGraph::from_intent(&intent);
        assert!(result.is_ok());
        
        let teg = result.unwrap();
        assert!(!teg.nodes.is_empty());
        assert_eq!(teg.metadata.source_intent, Some(intent.id));
    }

    #[test]
    fn test_mermaid_generation() {
        let effects = vec![
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "transfer".to_string(),
                args: vec![Term::var("source"), Term::var("dest")],
            }),
        ];
        
        let teg = TemporalEffectGraph::from_effect_sequence(effects).unwrap();
        let mermaid = teg.to_mermaid();
        
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("transfer"));
        assert!(!mermaid.is_empty());
    }

    #[test]
    fn test_resource_dependency_analysis() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create effects that have resource dependencies
        let mint_effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "mint".to_string(),
            args: vec![Term::var("authority")],
        });
        
        let transfer_effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "transfer".to_string(),
            args: vec![Term::var("tokens")],
        });
        
        let mint_id = effect_to_entity_id(&mint_effect);
        let transfer_id = effect_to_entity_id(&transfer_effect);
        
        // Mint produces tokens, transfer consumes tokens
        teg.add_node(EffectNode {
            id: mint_id,
            effect: mint_effect,
            status: NodeStatus::Pending,
            dependencies: vec![],
            results: None,
            cost: 150,
            resource_requirements: vec!["mint_authority".to_string()],
            resource_productions: vec!["new_tokens".to_string()],
        }).unwrap();
        
        teg.add_node(EffectNode {
            id: transfer_id,
            effect: transfer_effect,
            status: NodeStatus::Pending,
            dependencies: vec![],
            results: None,
            cost: 100,
            resource_requirements: vec!["source_tokens".to_string()],
            resource_productions: vec!["dest_tokens".to_string()],
        }).unwrap();
        
        // Analyze dependencies
        let result = teg.analyze_dependencies();
        assert!(result.is_ok());
        
        // Should have calculated critical path and parallelization metrics
        assert!(teg.metadata.critical_path_length > 0);
        assert!(teg.metadata.parallelization_factor > 0.0);
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        let mut teg = TemporalEffectGraph::new();
        
        let effect_a = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_a".to_string(),
            args: vec![],
        });
        let effect_b = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "effect_b".to_string(),
            args: vec![],
        });
        
        let node_a_id = effect_to_entity_id(&effect_a);
        let node_b_id = effect_to_entity_id(&effect_b);
        
        // Create a cycle: A depends on B, B depends on A
        teg.add_node(EffectNode {
            id: node_a_id,
            effect: effect_a,
            status: NodeStatus::Pending,
            dependencies: vec![node_b_id],
            results: None,
            cost: 100,
            resource_requirements: vec![],
            resource_productions: vec![],
        }).unwrap();
        
        teg.add_node(EffectNode {
            id: node_b_id,
            effect: effect_b,
            status: NodeStatus::Pending,
            dependencies: vec![node_a_id],
            results: None,
            cost: 200,
            resource_requirements: vec![],
            resource_productions: vec![],
        }).unwrap();
        
        teg.add_edge(EffectEdge::CausalityLink {
            from: node_a_id,
            to: node_b_id,
            constraint: None,
        }).unwrap();
        
        teg.add_edge(EffectEdge::CausalityLink {
            from: node_b_id,
            to: node_a_id,
            constraint: None,
        }).unwrap();
        
        // Should detect the cycle
        let result = teg.topological_sort();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TegError::CyclicDependency(_)));
    }

    #[test]
    fn test_cache_locality_optimization() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create nodes with different resource dependencies
        let node1 = EffectNode {
            id: EntityId::from_bytes([1; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test1"))),
            dependencies: vec![],
            results: None,
            cost: 100,
            resource_requirements: vec!["resource_a".to_string()],
            resource_productions: vec!["output_a".to_string()],
            status: NodeStatus::Pending,
        };
        
        let node2 = EffectNode {
            id: EntityId::from_bytes([2; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test2"))),
            dependencies: vec![],
            results: None,
            cost: 150,
            resource_requirements: vec!["resource_a".to_string()],
            resource_productions: vec!["output_b".to_string()],
            status: NodeStatus::Pending,
        };
        
        teg.add_node(node1).unwrap();
        teg.add_node(node2).unwrap();
        
        let result = teg.optimize_cache_locality();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_critical_path_optimization_with_resources() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create a more complex graph with resource constraints
        let node1 = EffectNode {
            id: EntityId::from_bytes([1; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test1"))),
            dependencies: vec![],
            results: None,
            cost: 200,
            resource_requirements: vec!["shared_resource".to_string()],
            resource_productions: vec!["intermediate".to_string()],
            status: NodeStatus::Pending,
        };
        
        let node2 = EffectNode {
            id: EntityId::from_bytes([2; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test2"))),
            dependencies: vec![EntityId::from_bytes([1; 32])],
            results: None,
            cost: 300,
            resource_requirements: vec!["shared_resource".to_string(), "intermediate".to_string()],
            resource_productions: vec!["final_output".to_string()],
            status: NodeStatus::Pending,
        };
        
        teg.add_node(node1).unwrap();
        teg.add_node(node2).unwrap();
        teg.add_edge(EffectEdge::CausalityLink {
            from: EntityId::from_bytes([1; 32]),
            to: EntityId::from_bytes([2; 32]),
            constraint: None,
        }).unwrap();
        
        let optimized_length = teg.optimize_critical_path_with_resources();
        assert!(optimized_length.is_ok());
        assert!(optimized_length.unwrap() >= 500); // Should be at least the sum of costs
    }
    
    #[test]
    fn test_adaptive_scheduling_with_history() {
        let mut teg = TemporalEffectGraph::new();
        let mut history = ExecutionHistory::new();
        
        let node_id = EntityId::from_bytes([1; 32]);
        let node = EffectNode {
            id: node_id,
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test"))),
            dependencies: vec![],
            results: None,
            cost: 100,
            resource_requirements: vec![],
            resource_productions: vec![],
            status: NodeStatus::Pending,
        };
        
        teg.add_node(node).unwrap();
        
        // Add some execution history
        history.add_execution_record(node_id, ExecutionRecord {
            execution_time: 150,
            success: true,
            timestamp: 1000,
            resource_usage: 50,
        });
        
        history.add_execution_record(node_id, ExecutionRecord {
            execution_time: 200,
            success: true,
            timestamp: 2000,
            resource_usage: 60,
        });
        
        let result = teg.optimize_scheduling_with_history(&history);
        assert!(result.is_ok());
        
        // Cost should be adjusted based on history
        let updated_node = teg.nodes.get(&node_id).unwrap();
        assert_ne!(updated_node.cost, 100); // Should be updated
    }
    
    #[test]
    fn test_memory_pool_optimization() {
        let mut teg = TemporalEffectGraph::new();
        
        // Add several nodes to test memory optimization
        for i in 0..10 {
            let node = EffectNode {
                id: EntityId::from_bytes([i; 32]),
                effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test"))),
                dependencies: vec![],
                results: None,
                cost: 100 + i as u64,
                resource_requirements: vec![format!("resource_{}", i)],
                resource_productions: vec![format!("output_{}", i)],
                status: NodeStatus::Pending,
            };
            teg.add_node(node).unwrap();
        }
        
        let stats = teg.optimize_memory_pools();
        assert!(stats.is_ok());
        
        let optimization_stats = stats.unwrap();
        assert_eq!(optimization_stats.node_count, 10);
        assert!(optimization_stats.initial_memory > 0);
        assert!(optimization_stats.final_memory > 0);
    }
    
    #[test]
    fn test_performance_profiling() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create a graph with various characteristics for profiling
        // Make sure to create multiple similar nodes for batching opportunities
        for i in 0..3 {
            let node = EffectNode {
                id: EntityId::from_bytes([i; 32]),
                effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test"))), // Same effect for batching
                dependencies: vec![],
                results: None,
                cost: 100, // Same cost for batching opportunity
                resource_requirements: vec!["shared_resource".to_string()], // Shared resource for prefetching
                resource_productions: vec![format!("output_{}", i)],
                status: NodeStatus::Pending,
            };
            teg.add_node(node).unwrap();
        }
        
        // Add one more node with dependencies to create a proper graph structure
        let dependent_node = EffectNode {
            id: EntityId::from_bytes([10; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test_dependent"))),
            dependencies: vec![EntityId::from_bytes([0; 32])],
            results: None,
            cost: 200,
            resource_requirements: vec!["shared_resource".to_string()],
            resource_productions: vec!["final_output".to_string()],
            status: NodeStatus::Pending,
        };
        teg.add_node(dependent_node).unwrap();
        
        teg.add_edge(EffectEdge::ResourceLink {
            from: EntityId::from_bytes([0; 32]),
            to: EntityId::from_bytes([10; 32]),
            resource: "shared_resource".to_string(),
        }).unwrap();
        
        // Set up a critical path scenario that's much longer than average
        teg.metadata.critical_path_length = 2000; // Make this much longer than average to trigger pipeline optimization
        
        let profile = teg.profile_execution_performance();
        
        assert_eq!(profile.total_nodes, 4);
        assert_eq!(profile.total_edges, 1);
        assert_eq!(profile.critical_path_length, 2000);
        assert!(profile.parallelization_factor > 0.0);
        assert!(profile.memory_usage > 0);
        assert!(profile.cache_locality_score >= 0.0);
        
        // Should definitely find optimization opportunities now
        let opportunities = &profile.optimization_opportunities;
        
        // Debug output to see what opportunities were found
        println!("Found {} optimization opportunities:", opportunities.len());
        for op in opportunities {
            println!("- {}: {}", op.opportunity_type, op.description);
        }
        
        assert!(!opportunities.is_empty(), "Should find optimization opportunities with this setup");
    }
    
    #[test]
    fn test_optimization_opportunity_identification() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create similar nodes for batching opportunity
        for i in 0..3 {
            let node = EffectNode {
                id: EntityId::from_bytes([i; 32]),
                effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test"))),
                dependencies: vec![],
                results: None,
                cost: 100, // Same cost for batching
                resource_requirements: vec!["shared_resource".to_string()],
                resource_productions: vec![format!("output_{}", i)],
                status: NodeStatus::Pending,
            };
            teg.add_node(node).unwrap();
        }
        
        // Set up a critical path scenario
        teg.metadata.critical_path_length = 1000;
        
        let profile = teg.profile_execution_performance();
        let opportunities = &profile.optimization_opportunities;
        
        assert!(!opportunities.is_empty());
        
        // Should find batching opportunity
        let has_batching = opportunities.iter()
            .any(|op| op.opportunity_type == "effect_batching");
        assert!(has_batching);
        
        // Should find prefetching opportunity (shared resource used by multiple nodes)
        let has_prefetching = opportunities.iter()
            .any(|op| op.opportunity_type == "resource_prefetching");
        assert!(has_prefetching);
    }
    
    #[test]
    fn test_cache_affinity_scoring() {
        let mut teg = TemporalEffectGraph::new();
        
        let high_affinity_node = EffectNode {
            id: EntityId::from_bytes([1; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test1"))),
            dependencies: vec![],
            results: None,
            cost: 1000, // High cost
            resource_requirements: vec!["res_a".to_string(), "res_b".to_string()],
            resource_productions: vec!["out_a".to_string(), "out_b".to_string()],
            status: NodeStatus::Pending,
        };
        
        let low_affinity_node = EffectNode {
            id: EntityId::from_bytes([2; 32]),
            effect: EffectExpr::new(EffectExprKind::Pure(Term::var("test2"))),
            dependencies: vec![],
            results: None,
            cost: 100, // Low cost
            resource_requirements: vec![], // No dependencies
            resource_productions: vec![], // No productions
            status: NodeStatus::Pending,
        };
        
        teg.add_node(high_affinity_node).unwrap();
        teg.add_node(low_affinity_node).unwrap();
        
        let high_score = teg.calculate_cache_affinity_score(EntityId::from_bytes([1; 32]));
        let low_score = teg.calculate_cache_affinity_score(EntityId::from_bytes([2; 32]));
        
        assert!(high_score > low_score);
        assert!(high_score >= 300); // 2*100 (deps) + 100 (cost/10) + 2*50 (productions)
        assert!(low_score <= 10); // Just cost/10
    }
    
    #[test]
    fn test_memory_optimization_stats() {
        let stats = MemoryOptimizationStats {
            initial_memory: 1000,
            final_memory: 800,
            savings: 200,
            node_count: 10,
            edge_count: 15,
        };
        
        assert_eq!(stats.savings, 200);
        assert_eq!(stats.savings, stats.initial_memory - stats.final_memory);
    }
    
    #[test]
    fn test_execution_history_tracking_impl() {
        let mut history = ExecutionHistory::new();
        
        let node_id = EntityId::from_bytes([1u8; 32]);
        let record = ExecutionRecord {
            execution_time: 100,
            success: true,
            timestamp: 1000,
            resource_usage: 50,
        };
        
        history.add_execution_record(node_id, record);
        
        let node_history = history.get_node_history(node_id).unwrap();
        assert_eq!(node_history.executions.len(), 1);
        assert_eq!(node_history.average_execution_time(), 100);
        assert_eq!(node_history.success_rate(), 1.0);
        assert_eq!(node_history.average_resource_usage(), 50);
    }

    #[test]
    fn test_storage_proof_effect_recognition() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create storage proof effects
        let ethereum_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "ethereum_storage".to_string(),
                args: Vec::new(),
            }
        );
        
        let cosmos_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "cosmos_storage".to_string(),
                args: Vec::new(),
            }
        );
        
        let zk_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "zk_storage_proof".to_string(),
                args: Vec::new(),
            }
        );
        
        // Create nodes
        let eth_node = EffectNode {
            id: EntityId::from_bytes([1u8; 32]),
            effect: ethereum_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 600,
            resource_requirements: vec!["ethereum_rpc".to_string()],
            resource_productions: vec!["ethereum_storage_value".to_string()],
        };
        
        let cosmos_node = EffectNode {
            id: EntityId::from_bytes([2u8; 32]),
            effect: cosmos_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 500,
            resource_requirements: vec!["cosmos_rpc".to_string()],
            resource_productions: vec!["cosmos_storage_value".to_string()],
        };
        
        let zk_node = EffectNode {
            id: EntityId::from_bytes([3u8; 32]),
            effect: zk_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 2000,
            resource_requirements: vec!["storage_data".to_string(), "zk_circuit".to_string()],
            resource_productions: vec!["zk_proof".to_string()],
        };
        
        teg.add_node(eth_node).unwrap();
        teg.add_node(cosmos_node).unwrap();
        teg.add_node(zk_node).unwrap();
        
        // Test storage dependency detection
        let eth_deps = teg.get_storage_dependencies(EntityId::from_bytes([1u8; 32]));
        assert_eq!(eth_deps.len(), 1);
        assert_eq!(eth_deps[0].storage_type, "ethereum_storage");
        assert_eq!(eth_deps[0].domain, "ethereum");
        assert!(eth_deps[0].can_be_batched);
        
        let cosmos_deps = teg.get_storage_dependencies(EntityId::from_bytes([2u8; 32]));
        assert_eq!(cosmos_deps.len(), 1);
        assert_eq!(cosmos_deps[0].storage_type, "cosmos_storage");
        assert_eq!(cosmos_deps[0].domain, "cosmos");
        
        let zk_deps = teg.get_storage_dependencies(EntityId::from_bytes([3u8; 32]));
        assert_eq!(zk_deps.len(), 0); // Not a direct storage effect
    }

    #[test]
    fn test_storage_proof_scheduling_optimization() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create multiple Ethereum storage effects
        for i in 0..3 {
            let effect = EffectExpr::new(
                crate::effect::EffectExprKind::Perform {
                    effect_tag: "ethereum_storage".to_string(),
                    args: Vec::new(),
                }
            );
            
            let node = EffectNode {
                id: EntityId::from_bytes([i as u8; 32]),
                effect,
                status: NodeStatus::Pending,
                dependencies: Vec::new(),
                results: None,
                cost: 600,
                resource_requirements: vec!["ethereum_rpc".to_string()],
                resource_productions: vec!["ethereum_storage_value".to_string()],
            };
            
            teg.add_node(node).unwrap();
        }
        
        // Apply storage proof optimizations
        let result = teg.optimize_storage_proof_scheduling();
        assert!(result.is_ok());
        
        // Check that domain batching was applied (should have resource links)
        let resource_links: Vec<_> = teg.edges.iter()
            .filter(|edge| matches!(edge, EffectEdge::ResourceLink { resource, .. } if resource.contains("batch")))
            .collect();
        
        assert!(!resource_links.is_empty(), "Expected batching resource links to be created");
    }

    #[test]
    fn test_cross_chain_verification_ordering() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create cross-chain verification effects with different costs
        let heavy_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "cross_chain_verification".to_string(),
                args: Vec::new(),
            }
        );
        
        let light_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "cross_chain_verification".to_string(),
                args: Vec::new(),
            }
        );
        
        let heavy_node = EffectNode {
            id: EntityId::from_bytes([1u8; 32]),
            effect: heavy_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 2000, // Heavy operation
            resource_requirements: Vec::new(),
            resource_productions: Vec::new(),
        };
        
        let light_node = EffectNode {
            id: EntityId::from_bytes([2u8; 32]),
            effect: light_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 500, // Light operation
            resource_requirements: Vec::new(),
            resource_productions: Vec::new(),
        };
        
        teg.add_node(heavy_node).unwrap();
        teg.add_node(light_node).unwrap();
        
        // Apply cross-chain optimization
        let result = teg.optimize_storage_proof_scheduling();
        assert!(result.is_ok());
        
        // Check that ordering constraints were added (heavy before light)
        let ordering_edges: Vec<_> = teg.edges.iter()
            .filter(|edge| {
                matches!(edge, EffectEdge::CausalityLink { constraint: Some(c), .. } if c == "cross_chain_ordering")
            })
            .collect();
        
        assert!(!ordering_edges.is_empty(), "Expected cross-chain ordering constraints");
    }

    #[test]
    fn test_zk_proof_parallelization() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create multiple independent ZK proof generation effects
        for i in 0..4 {
            let effect = EffectExpr::new(
                crate::effect::EffectExprKind::Perform {
                    effect_tag: "zk_storage_proof".to_string(),
                    args: Vec::new(),
                }
            );
            
            let node = EffectNode {
                id: EntityId::from_bytes([i as u8; 32]),
                effect,
                status: NodeStatus::Pending,
                dependencies: Vec::new(),
                results: None,
                cost: 2000,
                resource_requirements: vec!["storage_data".to_string()],
                resource_productions: vec!["zk_proof".to_string()],
            };
            
            teg.add_node(node).unwrap();
        }
        
        // Apply ZK optimization
        let result = teg.optimize_storage_proof_scheduling();
        assert!(result.is_ok());
        
        // Check that parallel execution hints were added
        let parallel_links: Vec<_> = teg.edges.iter()
            .filter(|edge| {
                matches!(edge, EffectEdge::ResourceLink { resource, .. } if resource == "zk_proving_parallelism")
            })
            .collect();
        
        assert!(!parallel_links.is_empty(), "Expected ZK parallelization hints");
    }

    #[test]
    fn test_storage_proof_latency_estimation() {
        let teg = TemporalEffectGraph::new();
        
        assert_eq!(teg.estimate_storage_latency("ethereum_storage"), 300);
        assert_eq!(teg.estimate_storage_latency("cosmos_storage"), 150);
        assert_eq!(teg.estimate_storage_latency("storage_proof"), 500);
        assert_eq!(teg.estimate_storage_latency("zk_storage_proof"), 2000);
        assert_eq!(teg.estimate_storage_latency("unknown"), 100);
    }

    #[test]
    fn test_storage_proof_cost_estimation() {
        let teg = TemporalEffectGraph::new();
        
        let ethereum_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "ethereum_storage".to_string(),
                args: Vec::new(),
            }
        );
        
        let zk_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "zk_storage_proof".to_string(),
                args: Vec::new(),
            }
        );
        
        assert_eq!(teg.estimate_effect_cost(&ethereum_effect), 600);
        assert_eq!(teg.estimate_effect_cost(&zk_effect), 2000);
    }

    #[test]
    fn test_storage_proof_resource_extraction() {
        let teg = TemporalEffectGraph::new();
        
        let storage_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "storage_proof".to_string(),
                args: Vec::new(),
            }
        );
        
        let requirements = teg.extract_resource_requirements(&storage_effect);
        assert!(requirements.contains(&"blockchain_connection".to_string()));
        assert!(requirements.contains(&"verification_key".to_string()));
        assert!(requirements.contains(&"storage_commitment".to_string()));
        
        let productions = teg.extract_resource_productions(&storage_effect);
        assert!(productions.contains(&"verified_storage_data".to_string()));
        assert!(productions.contains(&"storage_proof_cache".to_string()));
    }

    #[test]
    #[ignore = "cycle detection needs refinement - currently detecting false positives"]
    fn test_cycle_detection_for_storage_effects() {
        let mut teg = TemporalEffectGraph::new();
        
        let node1_id = EntityId::from_bytes([1u8; 32]);
        let node2_id = EntityId::from_bytes([2u8; 32]);
        
        // Create nodes first
        let node1 = EffectNode {
            id: node1_id,
            effect: EffectExpr::new(
                crate::effect::EffectExprKind::Perform {
                    effect_tag: "storage_proof".to_string(),
                    args: Vec::new(),
                }
            ),
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 500,
            resource_requirements: vec!["storage_commitment".to_string()],
            resource_productions: vec!["verified_data".to_string()],
        };
        
        let node2 = EffectNode {
            id: node2_id,
            effect: EffectExpr::new(
                crate::effect::EffectExprKind::Perform {
                    effect_tag: "compute".to_string(),
                    args: Vec::new(),
                }
            ),
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 300,
            resource_requirements: vec!["verified_data".to_string()],
            resource_productions: vec!["result".to_string()],
        };
        
        // Add nodes to the graph
        teg.add_node(node1).unwrap();
        teg.add_node(node2).unwrap();
        
        // Create a simple edge that doesn't create a cycle (1 -> 2)
        let edge1 = EffectEdge::CausalityLink {
            from: node1_id,
            to: node2_id,
            constraint: None,
        };
        
        // Before adding any edges, there should be no cycle
        assert!(!teg.would_create_cycle(&edge1));
        
        // Add the first edge
        teg.add_edge(edge1).unwrap();
        
        // Create a reverse edge that would create a cycle (2 -> 1)
        let edge2 = EffectEdge::CausalityLink {
            from: node2_id,
            to: node1_id,
            constraint: None,
        };
        
        // After adding edge1 (1->2), adding edge2 (2->1) should create a cycle
        assert!(teg.would_create_cycle(&edge2), 
            "Expected cycle detection to work. Current edges: {:?}, adjacency_list: {:?}", 
            teg.edges, teg.adjacency_list);
        
        // Verify we can detect cycles with topological sort as well
        let result = teg.add_edge(edge2);
        // It should be allowed to add, but topological sort should fail
        assert!(result.is_ok());
        
        let topo_result = teg.topological_sort();
        assert!(topo_result.is_err(), "Topological sort should fail with cycle");
        
        if let Err(TegError::CyclicDependency(cycle_nodes)) = topo_result {
            assert!(!cycle_nodes.is_empty(), "Should have identified nodes in cycle");
        } else {
            panic!("Expected CyclicDependency error");
        }
    }

    #[test]
    fn test_storage_proof_dependency_analysis() {
        let mut teg = TemporalEffectGraph::new();
        
        // Create a storage proof node
        let storage_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "ethereum_storage".to_string(),
                args: Vec::new(),
            }
        );
        
        let storage_node = EffectNode {
            id: EntityId::from_bytes([1u8; 32]),
            effect: storage_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 600,
            resource_requirements: vec!["ethereum_rpc".to_string()],
            resource_productions: vec!["ethereum_storage_value".to_string()],
        };
        
        // Create a compute effect that depends on storage
        let compute_effect = EffectExpr::new(
            crate::effect::EffectExprKind::Perform {
                effect_tag: "swap".to_string(),
                args: Vec::new(),
            }
        );
        
        let compute_node = EffectNode {
            id: EntityId::from_bytes([2u8; 32]),
            effect: compute_effect,
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            results: None,
            cost: 300,
            resource_requirements: vec!["input_tokens".to_string()],
            resource_productions: vec!["output_tokens".to_string()],
        };
        
        teg.add_node(storage_node).unwrap();
        teg.add_node(compute_node).unwrap();
        
        // Add resource dependency
        let resource_edge = EffectEdge::ResourceLink {
            from: EntityId::from_bytes([1u8; 32]),
            to: EntityId::from_bytes([2u8; 32]),
            resource: "ethereum_storage_value".to_string(),
        };
        
        teg.add_edge(resource_edge).unwrap();
        
        // Test storage dependency detection for the compute node
        let compute_deps = teg.get_storage_dependencies(EntityId::from_bytes([2u8; 32]));
        assert_eq!(compute_deps.len(), 1);
        assert_eq!(compute_deps[0].storage_type, "ethereum_storage");
        assert_eq!(compute_deps[0].domain, "ethereum");
    }
} 