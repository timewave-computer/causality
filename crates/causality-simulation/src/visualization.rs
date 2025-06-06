//! Visualization and tracing for simulation analysis

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{
    clock::SimulatedTimestamp,
    snapshot::EffectExecution,
    error::SimulationResult,
};

/// Visualization hooks for capturing execution traces
#[derive(Debug, Default)]
pub struct VisualizationHooks {
    traces: Vec<ExecutionTrace>,
    graph_visualizer: GraphVisualizer,
    enabled: bool,
}

/// Execution trace for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub operation_id: String,
    pub operation_type: String,
    pub start_time: SimulatedTimestamp,
    pub end_time: Option<SimulatedTimestamp>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub effects: Vec<String>,
    pub status: TraceStatus,
    pub metadata: HashMap<String, String>,
}

/// Status of an execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    Started,
    InProgress,
    Completed,
    Failed { error: String },
    Cancelled,
}

impl VisualizationHooks {
    /// Create new visualization hooks
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            graph_visualizer: GraphVisualizer::new(),
            enabled: true,
        }
    }
    
    /// Enable or disable visualization
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Start tracing an operation
    pub fn start_trace(&mut self, operation_id: String, operation_type: String, timestamp: SimulatedTimestamp) {
        if !self.enabled {
            return;
        }
        
        let trace = ExecutionTrace {
            operation_id,
            operation_type,
            start_time: timestamp,
            end_time: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            effects: Vec::new(),
            status: TraceStatus::Started,
            metadata: HashMap::new(),
        };
        
        self.traces.push(trace);
    }
    
    /// Update a trace with inputs
    pub fn add_trace_inputs(&mut self, operation_id: &str, inputs: Vec<String>) {
        if !self.enabled {
            return;
        }
        
        if let Some(trace) = self.traces.iter_mut().find(|t| t.operation_id == operation_id) {
            trace.inputs.extend(inputs);
            trace.status = TraceStatus::InProgress;
        }
    }
    
    /// Update a trace with outputs
    pub fn add_trace_outputs(&mut self, operation_id: &str, outputs: Vec<String>) {
        if !self.enabled {
            return;
        }
        
        if let Some(trace) = self.traces.iter_mut().find(|t| t.operation_id == operation_id) {
            trace.outputs.extend(outputs);
        }
    }
    
    /// Complete a trace
    pub fn complete_trace(&mut self, operation_id: &str, timestamp: SimulatedTimestamp, success: bool, error: Option<String>) {
        if !self.enabled {
            return;
        }
        
        if let Some(trace) = self.traces.iter_mut().find(|t| t.operation_id == operation_id) {
            trace.end_time = Some(timestamp);
            trace.status = if success {
                TraceStatus::Completed
            } else {
                TraceStatus::Failed {
                    error: error.unwrap_or_else(|| "Unknown error".to_string())
                }
            };
        }
    }
    
    /// Add metadata to a trace
    pub fn add_trace_metadata(&mut self, operation_id: &str, key: String, value: String) {
        if !self.enabled {
            return;
        }
        
        if let Some(trace) = self.traces.iter_mut().find(|t| t.operation_id == operation_id) {
            trace.metadata.insert(key, value);
        }
    }
    
    /// Get all traces
    pub fn get_traces(&self) -> &[ExecutionTrace] {
        &self.traces
    }
    
    /// Get traces for a specific operation type
    pub fn get_traces_by_type(&self, operation_type: &str) -> Vec<&ExecutionTrace> {
        self.traces.iter().filter(|t| t.operation_type == operation_type).collect()
    }
    
    /// Clear all traces
    pub fn clear_traces(&mut self) {
        self.traces.clear();
    }
    
    /// Generate a TEG visualization
    pub fn generate_teg_graph(&mut self, effects: &[EffectExecution]) -> SimulationResult<String> {
        self.graph_visualizer.generate_teg(effects)
    }
    
    /// Generate execution timeline
    pub fn generate_timeline(&self) -> SimulationResult<String> {
        let mut timeline = String::new();
        timeline.push_str("# Execution Timeline\n\n");
        
        let mut sorted_traces = self.traces.clone();
        sorted_traces.sort_by_key(|t| t.start_time);
        
        for trace in sorted_traces {
            let duration = trace.end_time
                .map(|end| end.duration_since(trace.start_time))
                .unwrap_or_else(|| std::time::Duration::ZERO);
            
            timeline.push_str(&format!(
                "- {} ({}): {} -> {} [{}ms]\n",
                trace.operation_id,
                trace.operation_type,
                trace.start_time.as_secs(),
                trace.end_time.map_or("ongoing".to_string(), |t| t.as_secs().to_string()),
                duration.as_millis()
            ));
        }
        
        Ok(timeline)
    }
}

/// Graph visualizer for TEG and execution flow
#[derive(Debug)]
pub struct GraphVisualizer {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}

/// Node in the execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub metadata: HashMap<String, String>,
}

/// Edge in the execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub edge_type: String,
}

impl GraphVisualizer {
    /// Create a new graph visualizer
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
    
    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.id.clone(), node);
    }
    
    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }
    
    /// Generate a TEG visualization from effect executions
    pub fn generate_teg(&mut self, effects: &[EffectExecution]) -> SimulationResult<String> {
        self.nodes.clear();
        self.edges.clear();
        
        // Create nodes for each effect
        for effect in effects {
            let node = GraphNode {
                id: effect.effect_id.clone(),
                label: effect.effect_id.clone(),
                node_type: "effect".to_string(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("start_time".to_string(), effect.start_time.as_secs().to_string());
                    if let Some(end_time) = effect.end_time {
                        meta.insert("end_time".to_string(), end_time.as_secs().to_string());
                    }
                    meta.insert("result".to_string(), format!("{:?}", effect.result));
                    meta
                },
            };
            self.add_node(node);
        }
        
        // Create edges based on resource dependencies
        for effect in effects {
            for consumed in &effect.resources_consumed {
                // Find effects that produced this resource
                for producer in effects {
                    if producer.resources_produced.contains(consumed) && producer.effect_id != effect.effect_id {
                        let edge = GraphEdge {
                            from: producer.effect_id.clone(),
                            to: effect.effect_id.clone(),
                            label: Some(consumed.clone()),
                            edge_type: "resource_dependency".to_string(),
                        };
                        self.add_edge(edge);
                    }
                }
            }
        }
        
        self.to_mermaid()
    }
    
    /// Convert the graph to Mermaid format
    pub fn to_mermaid(&self) -> SimulationResult<String> {
        let mut mermaid = String::new();
        mermaid.push_str("graph TD\n");
        
        // Add nodes
        for node in self.nodes.values() {
            mermaid.push_str(&format!("    {}[{}]\n", node.id, node.label));
        }
        
        // Add edges
        for edge in &self.edges {
            let label = edge.label.as_deref().unwrap_or("");
            mermaid.push_str(&format!("    {} --> |{}| {}\n", edge.from, label, edge.to));
        }
        
        Ok(mermaid)
    }
    
    /// Convert the graph to DOT format for Graphviz
    pub fn to_dot(&self) -> SimulationResult<String> {
        let mut dot = String::new();
        dot.push_str("digraph TEG {\n");
        dot.push_str("    rankdir=LR;\n");
        
        // Add nodes
        for node in self.nodes.values() {
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\" shape=box];\n", 
                node.id, 
                node.label
            ));
        }
        
        // Add edges
        for edge in &self.edges {
            let label = edge.label.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\"];\n", 
                edge.from, 
                edge.to, 
                label
            ));
        }
        
        dot.push_str("}\n");
        Ok(dot)
    }
}

impl Default for GraphVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_visualization_hooks() {
        let mut hooks = VisualizationHooks::new();
        let timestamp = SimulatedTimestamp::from_secs(1000);
        
        hooks.start_trace("op1".to_string(), "effect".to_string(), timestamp);
        hooks.add_trace_inputs("op1", vec!["input1".to_string()]);
        hooks.add_trace_outputs("op1", vec!["output1".to_string()]);
        hooks.complete_trace("op1", timestamp, true, None);
        
        let traces = hooks.get_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].operation_id, "op1");
        assert!(matches!(traces[0].status, TraceStatus::Completed));
    }
    
    #[test]
    fn test_graph_visualizer() {
        let mut visualizer = GraphVisualizer::new();
        
        let node1 = GraphNode {
            id: "node1".to_string(),
            label: "Effect 1".to_string(),
            node_type: "effect".to_string(),
            metadata: HashMap::new(),
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            label: "Effect 2".to_string(),
            node_type: "effect".to_string(),
            metadata: HashMap::new(),
        };
        
        visualizer.add_node(node1);
        visualizer.add_node(node2);
        
        let edge = GraphEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            label: Some("resource".to_string()),
            edge_type: "dependency".to_string(),
        };
        
        visualizer.add_edge(edge);
        
        let mermaid = visualizer.to_mermaid().unwrap();
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("node1[Effect 1]"));
        assert!(mermaid.contains("node1 --> |resource| node2"));
    }
} 