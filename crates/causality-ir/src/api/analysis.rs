//! Analysis module for the Temporal Effect Graph (TEG) API
//! 
//! This module provides advanced analysis capabilities for TEGs, including:
//! 1. Graph metrics and statistics
//! 2. Cycle detection and analysis
//! 3. Critical path analysis
//! 4. Resource flow analysis
//! 5. Domain boundary crossing analysis

use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{Result, anyhow};

use crate::{
    TemporalEffectGraph, EffectId, ResourceId, DomainId,
    graph::edge::Condition,
};

/// The TEG Analyzer provides analysis capabilities for Temporal Effect Graphs
#[derive(Debug)]
pub struct TEGAnalyzer<'a> {
    /// Reference to the TEG being analyzed
    teg: &'a TemporalEffectGraph,
    
    /// Cache for analysis results
    cache: HashMap<String, AnalysisResult>,
}

/// Result of an analysis operation
#[derive(Debug, Clone)]
pub enum AnalysisResult {
    /// Simple numeric metric
    Metric(f64),
    
    /// List of effect nodes
    EffectList(Vec<EffectId>),
    
    /// List of resource nodes
    ResourceList(Vec<ResourceId>),
    
    /// Path through the graph (sequence of effect nodes)
    Path(Vec<EffectId>),
    
    /// Set of paths
    Paths(Vec<Vec<EffectId>>),
    
    /// Cycles in the graph
    Cycles(Vec<Vec<EffectId>>),
    
    /// Domain boundary crossings
    DomainCrossings(Vec<(EffectId, EffectId, DomainId, DomainId)>),
    
    /// Complex analysis with structured data
    Complex(serde_json::Value),
}

impl<'a> TEGAnalyzer<'a> {
    /// Create a new TEG analyzer
    pub fn new(teg: &'a TemporalEffectGraph) -> Self {
        Self {
            teg,
            cache: HashMap::new(),
        }
    }
    
    /// Get basic graph statistics
    pub fn basic_stats(&mut self) -> Result<HashMap<String, f64>> {
        // Use cache if available
        if let Some(AnalysisResult::Complex(value)) = self.cache.get("basic_stats") {
            if let Some(map) = value.as_object() {
                let mut result = HashMap::new();
                for (k, v) in map {
                    if let Some(num) = v.as_f64() {
                        result.insert(k.clone(), num);
                    }
                }
                return Ok(result);
            }
        }
        
        // Calculate statistics
        let effect_count = self.teg.effect_nodes.len() as f64;
        let resource_count = self.teg.resource_nodes.len() as f64;
        let domain_count = self.teg.domains.len() as f64;
        
        let mut dependency_counts = Vec::new();
        for deps in self.teg.effect_dependencies.values() {
            dependency_counts.push(deps.len() as f64);
        }
        
        let avg_dependencies = if !dependency_counts.is_empty() {
            dependency_counts.iter().sum::<f64>() / dependency_counts.len() as f64
        } else {
            0.0
        };
        
        let mut continuation_counts = Vec::new();
        for conts in self.teg.effect_continuations.values() {
            continuation_counts.push(conts.len() as f64);
        }
        
        let avg_continuations = if !continuation_counts.is_empty() {
            continuation_counts.iter().sum::<f64>() / continuation_counts.len() as f64
        } else {
            0.0
        };
        
        // Compute graph density (ratio of actual to possible edges)
        let possible_edges = effect_count * (effect_count - 1.0);
        let actual_edges = self.teg.effect_dependencies.values()
            .map(|deps| deps.len() as f64).sum::<f64>();
        
        let density = if possible_edges > 0.0 {
            actual_edges / possible_edges
        } else {
            0.0
        };
        
        // Build result
        let mut result = HashMap::new();
        result.insert("effect_count".to_string(), effect_count);
        result.insert("resource_count".to_string(), resource_count);
        result.insert("domain_count".to_string(), domain_count);
        result.insert("avg_dependencies".to_string(), avg_dependencies);
        result.insert("avg_continuations".to_string(), avg_continuations);
        result.insert("graph_density".to_string(), density);
        
        // Cache the result
        let json_value = serde_json::to_value(&result)?;
        self.cache.insert("basic_stats".to_string(), AnalysisResult::Complex(json_value));
        
        Ok(result)
    }
    
    /// Detect cycles in the graph
    pub fn detect_cycles(&mut self) -> Result<Vec<Vec<EffectId>>> {
        // Use cache if available
        if let Some(AnalysisResult::Cycles(cycles)) = self.cache.get("cycles") {
            return Ok(cycles.clone());
        }
        
        // Use Tarjan's algorithm to find strongly connected components
        let mut cycles = Vec::new();
        let mut index_counter = 0;
        let mut indices = HashMap::new();
        let mut lowlinks = HashMap::new();
        let mut onstack = HashSet::new();
        let mut stack = Vec::new();
        
        // Helper function to run Tarjan's algorithm
        fn strongconnect(
            node: EffectId,
            teg: &TemporalEffectGraph,
            index_counter: &mut usize,
            indices: &mut HashMap<EffectId, usize>,
            lowlinks: &mut HashMap<EffectId, usize>,
            onstack: &mut HashSet<EffectId>,
            stack: &mut Vec<EffectId>,
            cycles: &mut Vec<Vec<EffectId>>,
        ) {
            // Set the depth index for node
            indices.insert(node.clone(), *index_counter);
            lowlinks.insert(node.clone(), *index_counter);
            *index_counter += 1;
            stack.push(node.clone());
            onstack.insert(node.clone());
            
            // Consider successors
            if let Some(continuations) = teg.effect_continuations.get(&node) {
                for (next_node, _) in continuations {
                    if !indices.contains_key(next_node) {
                        // Successor has not yet been visited; recurse on it
                        strongconnect(
                            next_node.clone(),
                            teg,
                            index_counter,
                            indices,
                            lowlinks,
                            onstack,
                            stack,
                            cycles,
                        );
                        
                        // Check if we can reach earlier nodes through the successor
                        if let (Some(&lowlink), Some(&our_lowlink)) = (
                            lowlinks.get(next_node),
                            lowlinks.get(&node),
                        ) {
                            lowlinks.insert(node.clone(), lowlink.min(our_lowlink));
                        }
                    } else if onstack.contains(next_node) {
                        // Successor is in stack and hence in the current SCC
                        if let (Some(&index), Some(&our_lowlink)) = (
                            indices.get(next_node),
                            lowlinks.get(&node),
                        ) {
                            lowlinks.insert(node.clone(), index.min(our_lowlink));
                        }
                    }
                }
            }
            
            // If we're at the root of an SCC, pop the stack and generate an SCC
            if let (Some(&index), Some(&lowlink)) = (indices.get(&node), lowlinks.get(&node)) {
                if index == lowlink {
                    // Start a new strongly connected component
                    let mut component = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        onstack.remove(&w);
                        component.push(w.clone());
                        if w == node {
                            break;
                        }
                    }
                    
                    // Only include components with more than one node (cycles)
                    if component.len() > 1 {
                        cycles.push(component);
                    }
                }
            }
        }
        
        // Run Tarjan's algorithm on each unvisited node
        for node in self.teg.effect_nodes.keys() {
            if !indices.contains_key(node) {
                strongconnect(
                    node.clone(),
                    self.teg,
                    &mut index_counter,
                    &mut indices,
                    &mut lowlinks,
                    &mut onstack,
                    &mut stack,
                    &mut cycles,
                );
            }
        }
        
        // Cache the result
        self.cache.insert("cycles".to_string(), AnalysisResult::Cycles(cycles.clone()));
        
        Ok(cycles)
    }
    
    /// Find the critical path through the graph
    pub fn critical_path(&mut self) -> Result<Vec<EffectId>> {
        // Use cache if available
        if let Some(AnalysisResult::Path(path)) = self.cache.get("critical_path") {
            return Ok(path.clone());
        }
        
        // Find entry points (effects with no dependencies)
        let entry_points: Vec<EffectId> = self.teg.effect_nodes.keys()
            .filter(|id| {
                !self.teg.effect_dependencies.values().any(|deps| deps.contains(id))
            })
            .cloned()
            .collect();
        
        if entry_points.is_empty() {
            return Err(anyhow!("No entry points found in the graph"));
        }
        
        // Find exit points (effects with no continuations)
        let exit_points: Vec<EffectId> = self.teg.effect_nodes.keys()
            .filter(|id| {
                !self.teg.effect_continuations.contains_key(*id) || 
                self.teg.effect_continuations.get(*id).map_or(true, |conts| conts.is_empty())
            })
            .cloned()
            .collect();
        
        if exit_points.is_empty() {
            return Err(anyhow!("No exit points found in the graph"));
        }
        
        // Use a modified longest path algorithm
        let mut distances: HashMap<EffectId, usize> = HashMap::new();
        let mut predecessors: HashMap<EffectId, EffectId> = HashMap::new();
        
        // Initialize distances
        for id in self.teg.effect_nodes.keys() {
            distances.insert(id.clone(), 0);
        }
        
        // Process nodes in topological order using a BFS-like approach
        let mut queue = VecDeque::new();
        for entry in &entry_points {
            queue.push_back(entry.clone());
            distances.insert(entry.clone(), 1);
        }
        
        while let Some(current) = queue.pop_front() {
            let current_dist = *distances.get(&current).unwrap_or(&0);
            
            // Process continuations
            if let Some(continuations) = self.teg.effect_continuations.get(&current) {
                for (next, _) in continuations {
                    let next_dist = *distances.get(next).unwrap_or(&0);
                    
                    // If we found a longer path, update
                    if current_dist + 1 > next_dist {
                        distances.insert(next.clone(), current_dist + 1);
                        predecessors.insert(next.clone(), current.clone());
                        queue.push_back(next.clone());
                    }
                }
            }
        }
        
        // Find the exit point with the longest path
        let mut max_exit_dist = 0;
        let mut max_exit = None;
        
        for exit in &exit_points {
            let dist = *distances.get(exit).unwrap_or(&0);
            if dist > max_exit_dist {
                max_exit_dist = dist;
                max_exit = Some(exit.clone());
            }
        }
        
        let max_exit = max_exit.ok_or_else(|| anyhow!("No valid exit point found"))?;
        
        // Reconstruct the critical path
        let mut path = Vec::new();
        let mut current = max_exit;
        
        while let Some(prev) = predecessors.get(&current) {
            path.push(current.clone());
            current = prev.clone();
        }
        
        path.push(current);
        path.reverse();
        
        // Cache the result
        self.cache.insert("critical_path".to_string(), AnalysisResult::Path(path.clone()));
        
        Ok(path)
    }
    
    /// Analyze resource flow in the graph
    pub fn analyze_resource_flow(&mut self, resource_id: &ResourceId) -> Result<Vec<EffectId>> {
        let cache_key = format!("resource_flow_{}", resource_id);
        
        // Use cache if available
        if let Some(AnalysisResult::EffectList(effects)) = self.cache.get(&cache_key) {
            return Ok(effects.clone());
        }
        
        // Find all effects that access this resource
        let effects: Vec<EffectId> = self.teg.effect_nodes.iter()
            .filter_map(|(id, effect)| {
                if effect.resources_accessed.contains(resource_id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Order effects based on dependencies and continuations
        let mut ordered_effects = Vec::new();
        let mut visited = HashSet::new();
        
        // Helper function for topological sort
        fn visit(
            node: &EffectId,
            teg: &TemporalEffectGraph,
            visited: &mut HashSet<EffectId>,
            ordered: &mut Vec<EffectId>,
            resource_id: &ResourceId,
        ) {
            if visited.contains(node) {
                return;
            }
            
            visited.insert(node.clone());
            
            // Visit dependencies first
            if let Some(deps) = teg.effect_dependencies.get(node) {
                for dep in deps {
                    if let Some(effect) = teg.effect_nodes.get(dep) {
                        if effect.resources_accessed.contains(resource_id) {
                            visit(dep, teg, visited, ordered, resource_id);
                        }
                    }
                }
            }
            
            // Add this node
            if let Some(effect) = teg.effect_nodes.get(node) {
                if effect.resources_accessed.contains(resource_id) {
                    ordered.push(node.clone());
                }
            }
            
            // Visit continuations
            if let Some(conts) = teg.effect_continuations.get(node) {
                for (cont, _) in conts {
                    if let Some(effect) = teg.effect_nodes.get(cont) {
                        if effect.resources_accessed.contains(resource_id) {
                            visit(cont, teg, visited, ordered, resource_id);
                        }
                    }
                }
            }
        }
        
        // Find starting points (effects with no dependencies)
        let start_points: Vec<EffectId> = effects.iter()
            .filter(|id| {
                !self.teg.effect_dependencies.values()
                    .any(|deps| deps.contains(id))
            })
            .cloned()
            .collect();
        
        // Run topological sort from each starting point
        for start in start_points {
            visit(&start, self.teg, &mut visited, &mut ordered_effects, resource_id);
        }
        
        // For any remaining effects, just add them to the end
        for effect_id in &effects {
            if !ordered_effects.contains(effect_id) {
                ordered_effects.push(effect_id.clone());
            }
        }
        
        // Cache the result
        self.cache.insert(cache_key, AnalysisResult::EffectList(ordered_effects.clone()));
        
        Ok(ordered_effects)
    }
    
    /// Identify domain boundary crossings
    pub fn domain_boundary_crossings(&mut self) -> Result<Vec<(EffectId, EffectId, DomainId, DomainId)>> {
        // Use cache if available
        if let Some(AnalysisResult::DomainCrossings(crossings)) = self.cache.get("domain_crossings") {
            return Ok(crossings.clone());
        }
        
        let mut crossings = Vec::new();
        
        // Check all continuations for domain changes
        for (from_id, continuations) in &self.teg.effect_continuations {
            if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                let from_domain = &from_effect.domain_id;
                
                for (to_id, _) in continuations {
                    if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                        let to_domain = &to_effect.domain_id;
                        
                        if from_domain != to_domain {
                            crossings.push((
                                from_id.clone(),
                                to_id.clone(),
                                from_domain.clone(),
                                to_domain.clone(),
                            ));
                        }
                    }
                }
            }
        }
        
        // Check dependencies for domain changes
        for (to_id, dependencies) in &self.teg.effect_dependencies {
            if let Some(to_effect) = self.teg.effect_nodes.get(to_id) {
                let to_domain = &to_effect.domain_id;
                
                for from_id in dependencies {
                    if let Some(from_effect) = self.teg.effect_nodes.get(from_id) {
                        let from_domain = &from_effect.domain_id;
                        
                        if from_domain != to_domain {
                            // Only add if not already in continuations
                            let already_in_conts = self.teg.effect_continuations.get(from_id)
                                .map(|conts| conts.iter().any(|(id, _)| id == to_id))
                                .unwrap_or(false);
                                
                            if !already_in_conts {
                                crossings.push((
                                    from_id.clone(),
                                    to_id.clone(),
                                    from_domain.clone(),
                                    to_domain.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        // Cache the result
        self.cache.insert("domain_crossings".to_string(), AnalysisResult::DomainCrossings(crossings.clone()));
        
        Ok(crossings)
    }
    
    /// Calculate graph complexity metrics
    pub fn complexity_metrics(&mut self) -> Result<HashMap<String, f64>> {
        // Use cache if available
        if let Some(AnalysisResult::Complex(value)) = self.cache.get("complexity_metrics") {
            if let Some(map) = value.as_object() {
                let mut result = HashMap::new();
                for (k, v) in map {
                    if let Some(num) = v.as_f64() {
                        result.insert(k.clone(), num);
                    }
                }
                return Ok(result);
            }
        }
        
        let mut metrics = HashMap::new();
        
        // Count nodes and edges
        let node_count = self.teg.effect_nodes.len() as f64;
        let edge_count = self.teg.effect_dependencies.values()
            .map(|deps| deps.len() as f64).sum::<f64>();
        
        // Basic metrics
        metrics.insert("node_count".to_string(), node_count);
        metrics.insert("edge_count".to_string(), edge_count);
        
        // Cyclomatic complexity (edges - nodes + 2)
        let cyclomatic = edge_count - node_count + 2.0;
        metrics.insert("cyclomatic_complexity".to_string(), cyclomatic);
        
        // Detect cycles
        let cycles = self.detect_cycles()?;
        metrics.insert("cycle_count".to_string(), cycles.len() as f64);
        
        // Domain complexity - number of domain crossings
        let crossings = self.domain_boundary_crossings()?;
        metrics.insert("domain_crossing_count".to_string(), crossings.len() as f64);
        
        // Average connectivity
        let avg_connectivity = if node_count > 0.0 {
            edge_count / node_count
        } else {
            0.0
        };
        metrics.insert("avg_connectivity".to_string(), avg_connectivity);
        
        // Cache the result
        let json_value = serde_json::to_value(&metrics)?;
        self.cache.insert("complexity_metrics".to_string(), AnalysisResult::Complex(json_value));
        
        Ok(metrics)
    }
    
    /// Find all paths between two effect nodes
    pub fn find_all_paths(&mut self, from: &EffectId, to: &EffectId) -> Result<Vec<Vec<EffectId>>> {
        let cache_key = format!("paths_{}_{}", from, to);
        
        // Use cache if available
        if let Some(AnalysisResult::Paths(paths)) = self.cache.get(&cache_key) {
            return Ok(paths.clone());
        }
        
        // DFS to find all paths
        let mut all_paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();
        
        // Helper function for DFS
        fn dfs(
            current: &EffectId,
            target: &EffectId,
            teg: &TemporalEffectGraph,
            visited: &mut HashSet<EffectId>,
            current_path: &mut Vec<EffectId>,
            all_paths: &mut Vec<Vec<EffectId>>,
        ) {
            // Mark node as visited and add to current path
            visited.insert(current.clone());
            current_path.push(current.clone());
            
            // Check if we've reached the target
            if current == target {
                all_paths.push(current_path.clone());
            } else {
                // Explore continuations
                if let Some(continuations) = teg.effect_continuations.get(current) {
                    for (next, _) in continuations {
                        if !visited.contains(next) {
                            dfs(next, target, teg, visited, current_path, all_paths);
                        }
                    }
                }
            }
            
            // Remove current node from path and mark as not visited
            current_path.pop();
            visited.remove(current);
        }
        
        // Start DFS
        dfs(from, to, self.teg, &mut visited, &mut current_path, &mut all_paths);
        
        // Cache the result
        self.cache.insert(cache_key, AnalysisResult::Paths(all_paths.clone()));
        
        Ok(all_paths)
    }
    
    /// Clear analysis cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Create a new TEG analyzer
pub fn create_analyzer(teg: &TemporalEffectGraph) -> TEGAnalyzer {
    TEGAnalyzer::new(teg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectNode;
    
    #[test]
    fn test_basic_stats() {
        let mut teg = TemporalEffectGraph::new();
        // Add some nodes and relationships
        
        let mut analyzer = TEGAnalyzer::new(&teg);
        let stats = analyzer.basic_stats().unwrap();
        
        assert!(stats.contains_key("effect_count"));
        assert!(stats.contains_key("resource_count"));
        assert!(stats.contains_key("domain_count"));
    }
    
    #[test]
    fn test_detect_cycles() {
        let mut teg = TemporalEffectGraph::new();
        // Create a graph with cycles
        
        let mut analyzer = TEGAnalyzer::new(&teg);
        let cycles = analyzer.detect_cycles().unwrap();
        
        // Verify cycles are detected correctly
    }
} 