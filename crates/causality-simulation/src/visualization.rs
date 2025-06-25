//! Visualization and tracing for simulation analysis

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::{
    clock::SimulatedTimestamp,
    snapshot::EffectExecution,
    error::SimulationResult,
    engine::{SessionOperation, SessionParticipantState},
};
use causality_core::lambda::base::SessionType;

/// Enhanced visualization hooks for capturing execution traces including session protocols
#[derive(Debug, Default)]
pub struct VisualizationHooks {
    traces: Vec<ExecutionTrace>,
    graph_visualizer: GraphVisualizer,
    session_visualizer: SessionProtocolVisualizer,
    enabled: bool,
}

/// Session protocol visualizer for session-specific diagrams
#[derive(Debug, Default)]
pub struct SessionProtocolVisualizer {
    /// Current session protocol states
    protocol_states: BTreeMap<String, SessionProtocolState>,
    /// Session flow history
    flow_history: Vec<SessionFlowEvent>,
    /// Complexity metrics for session types
    complexity_metrics: BTreeMap<String, SessionComplexityMetrics>,
}

/// Current state of a session protocol
#[derive(Debug, Clone)]
pub struct SessionProtocolState {
    /// Session identifier
    pub session_id: String,
    /// Current session type
    pub current_session_type: SessionType,
    /// Participants in this session
    pub participants: BTreeMap<String, SessionParticipantState>,
    /// Current protocol step
    pub protocol_step: usize,
    /// Last operation performed
    pub last_operation: Option<SessionOperation>,
    /// Protocol compliance status
    pub is_compliant: bool,
    /// Performance metrics
    pub performance_metrics: SessionPerformanceMetrics,
}

/// Session flow event for protocol visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFlowEvent {
    /// Session identifier
    pub session_id: String,
    /// Participant performing the operation
    pub participant: String,
    /// Operation performed
    pub operation: SessionOperation,
    /// Timestamp of the event
    pub timestamp: SimulatedTimestamp,
    /// Protocol state before the operation
    pub pre_state: String,
    /// Protocol state after the operation
    pub post_state: String,
    /// Whether the operation was successful
    pub success: bool,
}

/// Complexity metrics for session types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionComplexityMetrics {
    /// Communication complexity score
    pub communication_complexity: u32,
    /// Message count estimate
    pub estimated_message_count: u32,
    /// Nesting depth
    pub nesting_depth: u32,
    /// Choice branching factor
    pub branching_factor: u32,
    /// Recursion depth (if recursive)
    pub recursion_depth: u32,
    /// Parallelization potential score
    pub parallelization_score: f64,
}

/// Performance metrics for session execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPerformanceMetrics {
    /// Total operations performed
    pub operations_count: usize,
    /// Total execution time
    pub execution_time_ms: u64,
    /// Average operation time
    pub avg_operation_time_ms: f64,
    /// Gas consumed
    pub gas_consumed: u64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Protocol violations detected
    pub violations_count: usize,
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
    pub metadata: BTreeMap<String, String>,
    /// Session-specific trace information
    pub session_info: Option<SessionTraceInfo>,
}

/// Session-specific trace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTraceInfo {
    pub session_id: String,
    pub participant: String,
    pub protocol_step: usize,
    pub session_type_before: String,
    pub session_type_after: String,
    pub compliance_status: bool,
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
            session_visualizer: SessionProtocolVisualizer::new(),
            enabled: true,
        }
    }
    
    /// Enable or disable visualization
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Start tracing a session operation
    pub fn start_session_trace(
        &mut self, 
        operation_id: String, 
        session_id: String,
        participant: String,
        operation: &SessionOperation,
        timestamp: SimulatedTimestamp
    ) {
        if !self.enabled {
            return;
        }
        
        let session_info = SessionTraceInfo {
            session_id: session_id.clone(),
            participant: participant.clone(),
            protocol_step: 0, // Will be updated
            session_type_before: "Unknown".to_string(), // Will be updated
            session_type_after: "Unknown".to_string(), // Will be updated
            compliance_status: true,
        };
        
        let trace = ExecutionTrace {
            operation_id,
            operation_type: format!("{:?}", operation),
            start_time: timestamp,
            end_time: None,
            inputs: vec![format!("participant: {}", participant)],
            outputs: Vec::new(),
            effects: Vec::new(),
            status: TraceStatus::Started,
            metadata: BTreeMap::new(),
            session_info: Some(session_info),
        };
        
        self.traces.push(trace);
        
        // Record session flow event
        self.session_visualizer.record_flow_event(SessionFlowEvent {
            session_id,
            participant,
            operation: operation.clone(),
            timestamp,
            pre_state: "Starting".to_string(),
            post_state: "In Progress".to_string(),
            success: true,
        });
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
            metadata: BTreeMap::new(),
            session_info: None,
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
            
            // Update session flow if this is a session operation
            if let Some(session_info) = &trace.session_info {
                self.session_visualizer.complete_flow_event(&session_info.session_id, success);
            }
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
    
    /// Update session protocol state
    pub fn update_session_state(
        &mut self,
        session_id: String,
        participants: BTreeMap<String, SessionParticipantState>,
        current_session_type: SessionType,
    ) {
        if !self.enabled {
            return;
        }
        
        self.session_visualizer.update_protocol_state(session_id, participants, current_session_type);
    }
    
    /// Record session protocol violation
    pub fn record_protocol_violation(
        &mut self,
        session_id: &str,
        participant: &str,
        violation_description: String,
        timestamp: SimulatedTimestamp,
    ) {
        if !self.enabled {
            return;
        }
        
        self.session_visualizer.record_violation(session_id, participant, violation_description, timestamp);
    }
    
    /// Get all traces
    pub fn get_traces(&self) -> &[ExecutionTrace] {
        &self.traces
    }
    
    /// Get traces for a specific operation type
    pub fn get_traces_by_type(&self, operation_type: &str) -> Vec<&ExecutionTrace> {
        self.traces.iter().filter(|t| t.operation_type == operation_type).collect()
    }
    
    /// Get session-specific traces
    pub fn get_session_traces(&self, session_id: &str) -> Vec<&ExecutionTrace> {
        self.traces.iter()
            .filter(|t| t.session_info.as_ref().is_some_and(|info| info.session_id == session_id))
            .collect()
    }
    
    /// Clear all traces
    pub fn clear_traces(&mut self) {
        self.traces.clear();
        self.session_visualizer.clear_history();
    }
    
    /// Generate a TEG visualization
    pub fn generate_teg_graph(&mut self, effects: &[EffectExecution]) -> SimulationResult<String> {
        self.graph_visualizer.generate_teg(effects)
    }
    
    /// Generate session protocol flow diagram
    pub fn generate_session_flow_diagram(&self, session_id: &str) -> SimulationResult<String> {
        self.session_visualizer.generate_flow_diagram(session_id)
    }
    
    /// Generate session protocol state diagram
    pub fn generate_session_state_diagram(&self, session_id: &str) -> SimulationResult<String> {
        self.session_visualizer.generate_state_diagram(session_id)
    }
    
    /// Generate session complexity analysis
    pub fn generate_session_complexity_report(&self, session_id: &str) -> SimulationResult<String> {
        self.session_visualizer.generate_complexity_report(session_id)
    }
    
    /// Generate real-time session dashboard
    pub fn generate_session_dashboard(&self) -> SimulationResult<String> {
        self.session_visualizer.generate_dashboard()
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
            
            let session_info = trace.session_info
                .map(|info| format!(" [Session: {}, Participant: {}]", info.session_id, info.participant))
                .unwrap_or_default();
            
            timeline.push_str(&format!(
                "- {} ({}): {} -> {} [{}ms]{}\n",
                trace.operation_id,
                trace.operation_type,
                trace.start_time.as_secs(),
                trace.end_time.map_or("ongoing".to_string(), |t| t.as_secs().to_string()),
                duration.as_millis(),
                session_info
            ));
        }
        
        Ok(timeline)
    }
}

impl SessionProtocolVisualizer {
    /// Create a new session protocol visualizer
    pub fn new() -> Self {
        Self {
            protocol_states: BTreeMap::new(),
            flow_history: Vec::new(),
            complexity_metrics: BTreeMap::new(),
        }
    }
    
    /// Update protocol state for a session
    pub fn update_protocol_state(
        &mut self,
        session_id: String,
        participants: BTreeMap<String, SessionParticipantState>,
        current_session_type: SessionType,
    ) {
        let complexity = self.calculate_session_complexity(&current_session_type);
        self.complexity_metrics.insert(session_id.clone(), complexity);
        
        let performance_metrics = SessionPerformanceMetrics {
            operations_count: participants.values().map(|p| p.protocol_history.len()).sum(),
            execution_time_ms: 0, // Will be calculated from flow history
            avg_operation_time_ms: 0.0,
            gas_consumed: participants.values().map(|p| p.gas).sum(),
            throughput_ops_per_sec: 0.0,
            violations_count: participants.values().map(|p| p.compliance_state.violations.len()).sum(),
        };
        
        let state = SessionProtocolState {
            session_id: session_id.clone(),
            current_session_type,
            participants,
            protocol_step: performance_metrics.operations_count,
            last_operation: None,
            is_compliant: performance_metrics.violations_count == 0,
            performance_metrics,
        };
        
        self.protocol_states.insert(session_id, state);
    }
    
    /// Record a session flow event
    pub fn record_flow_event(&mut self, event: SessionFlowEvent) {
        self.flow_history.push(event);
    }
    
    /// Complete a flow event (update success status)
    pub fn complete_flow_event(&mut self, session_id: &str, success: bool) {
        if let Some(last_event) = self.flow_history.iter_mut()
            .filter(|e| e.session_id == session_id)
            .next_back() {
            last_event.success = success;
            last_event.post_state = if success { "Completed".to_string() } else { "Failed".to_string() };
        }
    }
    
    /// Record a protocol violation
    pub fn record_violation(
        &mut self,
        session_id: &str,
        participant: &str,
        violation_description: String,
        timestamp: SimulatedTimestamp,
    ) {
        // Create a special flow event for violations
        let violation_event = SessionFlowEvent {
            session_id: session_id.to_string(),
            participant: participant.to_string(),
            operation: SessionOperation::End, // Placeholder for violation
            timestamp,
            pre_state: "Valid".to_string(),
            post_state: format!("Violation: {}", violation_description),
            success: false,
        };
        
        self.flow_history.push(violation_event);
        
        // Update protocol state to mark as non-compliant
        if let Some(state) = self.protocol_states.get_mut(session_id) {
            state.is_compliant = false;
            state.performance_metrics.violations_count += 1;
        }
    }
    
    /// Generate flow diagram for a session
    pub fn generate_flow_diagram(&self, session_id: &str) -> SimulationResult<String> {
        let events: Vec<_> = self.flow_history.iter()
            .filter(|e| e.session_id == session_id)
            .collect();
        
        if events.is_empty() {
            return Ok(format!("No flow events found for session: {}", session_id));
        }
        
        let mut diagram = String::new();
        diagram.push_str(&format!("# Session Protocol Flow: {}\n\n", session_id));
        diagram.push_str("```mermaid\n");
        diagram.push_str("graph TD\n");
        
        for (i, event) in events.iter().enumerate() {
            let node_id = format!("N{}", i);
            let next_node_id = format!("N{}", i + 1);
            let operation_desc = format!("{:?}", event.operation);
            let status_icon = if event.success { "✓" } else { "✗" };
            
            diagram.push_str(&format!(
                "    {} --> {}[\"{}\\n{}\\n{} {}\"]\n",
                node_id,
                next_node_id,
                event.participant,
                operation_desc,
                status_icon,
                event.post_state
            ));
        }
        
        diagram.push_str("```\n");
        Ok(diagram)
    }
    
    /// Generate state diagram for a session
    pub fn generate_state_diagram(&self, session_id: &str) -> SimulationResult<String> {
        let state = self.protocol_states.get(session_id)
            .ok_or_else(|| crate::error::SimulationError::VisualizationError(
                format!("Session state not found: {}", session_id)
            ))?;
        
        let mut diagram = String::new();
        diagram.push_str(&format!("# Session Protocol State: {}\n\n", session_id));
        diagram.push_str(&format!("- **Current Session Type**: {:?}\n", state.current_session_type));
        diagram.push_str(&format!("- **Protocol Step**: {}\n", state.protocol_step));
        diagram.push_str(&format!("- **Compliance Status**: {}\n", if state.is_compliant { "✓ Compliant" } else { "✗ Non-compliant" }));
        diagram.push_str(&format!("- **Participants**: {}\n", state.participants.len()));
        diagram.push_str("\n## Performance Metrics\n");
        diagram.push_str(&format!("- **Operations Count**: {}\n", state.performance_metrics.operations_count));
        diagram.push_str(&format!("- **Gas Consumed**: {}\n", state.performance_metrics.gas_consumed));
        diagram.push_str(&format!("- **Violations**: {}\n", state.performance_metrics.violations_count));
        
        Ok(diagram)
    }
    
    /// Generate complexity report for a session
    pub fn generate_complexity_report(&self, session_id: &str) -> SimulationResult<String> {
        let complexity = self.complexity_metrics.get(session_id)
            .ok_or_else(|| crate::error::SimulationError::VisualizationError(
                format!("Complexity metrics not found: {}", session_id)
            ))?;
        
        let mut report = String::new();
        report.push_str(&format!("# Session Complexity Analysis: {}\n\n", session_id));
        report.push_str(&format!("- **Communication Complexity**: {}\n", complexity.communication_complexity));
        report.push_str(&format!("- **Estimated Message Count**: {}\n", complexity.estimated_message_count));
        report.push_str(&format!("- **Nesting Depth**: {}\n", complexity.nesting_depth));
        report.push_str(&format!("- **Branching Factor**: {}\n", complexity.branching_factor));
        report.push_str(&format!("- **Recursion Depth**: {}\n", complexity.recursion_depth));
        report.push_str(&format!("- **Parallelization Score**: {:.2}\n", complexity.parallelization_score));
        
        // Add complexity rating
        let overall_complexity = complexity.communication_complexity + 
                                complexity.nesting_depth + 
                                complexity.branching_factor + 
                                complexity.recursion_depth;
        
        let complexity_rating = match overall_complexity {
            0..=5 => "Low",
            6..=15 => "Medium",
            16..=30 => "High",
            _ => "Very High",
        };
        
        report.push_str(&format!("\n**Overall Complexity**: {} ({})\n", overall_complexity, complexity_rating));
        
        Ok(report)
    }
    
    /// Generate real-time dashboard
    pub fn generate_dashboard(&self) -> SimulationResult<String> {
        let mut dashboard = String::new();
        dashboard.push_str("# Session Protocol Dashboard\n\n");
        
        dashboard.push_str(&format!("## Active Sessions: {}\n\n", self.protocol_states.len()));
        
        for (session_id, state) in &self.protocol_states {
            let compliance_icon = if state.is_compliant { "✅" } else { "❌" };
            dashboard.push_str(&format!(
                "### {} {} {}\n",
                compliance_icon,
                session_id,
                if state.is_compliant { "(Compliant)" } else { "(Non-compliant)" }
            ));
            dashboard.push_str(&format!("- Participants: {}\n", state.participants.len()));
            dashboard.push_str(&format!("- Operations: {}\n", state.performance_metrics.operations_count));
            dashboard.push_str(&format!("- Gas: {}\n", state.performance_metrics.gas_consumed));
            dashboard.push_str(&format!("- Violations: {}\n", state.performance_metrics.violations_count));
            dashboard.push('\n');
        }
        
        // Overall statistics
        let total_operations: usize = self.protocol_states.values()
            .map(|s| s.performance_metrics.operations_count)
            .sum();
        let total_gas: u64 = self.protocol_states.values()
            .map(|s| s.performance_metrics.gas_consumed)
            .sum();
        let total_violations: usize = self.protocol_states.values()
            .map(|s| s.performance_metrics.violations_count)
            .sum();
        let compliant_sessions = self.protocol_states.values()
            .filter(|s| s.is_compliant)
            .count();
        
        dashboard.push_str("## Overall Statistics\n");
        dashboard.push_str(&format!("- **Total Operations**: {}\n", total_operations));
        dashboard.push_str(&format!("- **Total Gas Consumed**: {}\n", total_gas));
        dashboard.push_str(&format!("- **Total Violations**: {}\n", total_violations));
        dashboard.push_str(&format!("- **Compliant Sessions**: {}/{}\n", compliant_sessions, self.protocol_states.len()));
        
        Ok(dashboard)
    }
    
    /// Clear flow history
    pub fn clear_history(&mut self) {
        self.flow_history.clear();
        self.protocol_states.clear();
        self.complexity_metrics.clear();
    }
    
    /// Calculate complexity metrics for a session type
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_session_complexity(&self, session_type: &SessionType) -> SessionComplexityMetrics {
        SessionComplexityMetrics {
            communication_complexity: self.calculate_communication_complexity(session_type),
            estimated_message_count: self.estimate_message_count(session_type),
            nesting_depth: self.calculate_nesting_depth(session_type),
            branching_factor: self.calculate_branching_factor(session_type),
            recursion_depth: self.calculate_recursion_depth(session_type),
            parallelization_score: self.calculate_parallelization_score(session_type),
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_communication_complexity(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                1 + self.calculate_communication_complexity(continuation)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                1 + branches.iter().map(|(_, branch)| self.calculate_communication_complexity(branch)).max().unwrap_or(0)
            }
            SessionType::Recursive(_, body) => 2 + self.calculate_communication_complexity(body),
            SessionType::Variable(_) => 1,
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_message_count(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                1 + self.estimate_message_count(continuation)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                branches.iter().map(|(_, branch)| self.estimate_message_count(branch)).sum::<u32>() / (branches.len() as u32).max(1)
            }
            SessionType::Recursive(_, body) => 2 * self.estimate_message_count(body),
            SessionType::Variable(_) => 0,
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_nesting_depth(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                1 + self.calculate_nesting_depth(continuation)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                1 + branches.iter().map(|(_, branch)| self.calculate_nesting_depth(branch)).max().unwrap_or(0)
            }
            SessionType::Recursive(_, body) => 1 + self.calculate_nesting_depth(body),
            SessionType::Variable(_) => 1,
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_branching_factor(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                self.calculate_branching_factor(continuation)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                branches.len() as u32 + branches.iter().map(|(_, branch)| self.calculate_branching_factor(branch)).sum::<u32>()
            }
            SessionType::Recursive(_, body) => self.calculate_branching_factor(body),
            SessionType::Variable(_) => 0,
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_recursion_depth(&self, session_type: &SessionType) -> u32 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                self.calculate_recursion_depth(continuation)
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
                branches.iter().map(|(_, branch)| self.calculate_recursion_depth(branch)).max().unwrap_or(0)
            }
            SessionType::Recursive(_, body) => 1 + self.calculate_recursion_depth(body),
            SessionType::Variable(_) => 0,
            SessionType::End => 0,
        }
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_parallelization_score(&self, session_type: &SessionType) -> f64 {
        match session_type {
            SessionType::Send(_, continuation) | SessionType::Receive(_, continuation) => {
                self.calculate_parallelization_score(continuation) * 0.8 // Sequential operations reduce parallelization
            }
            SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) if branches.len() > 1 => {
                (branches.len() as f64) * 0.9 + branches.iter().map(|(_, branch)| self.calculate_parallelization_score(branch)).fold(0.0, |acc, x| acc + x) / branches.len() as f64
            }
            SessionType::Recursive(_, body) => self.calculate_parallelization_score(body) * 0.6, // Recursion limits parallelization
            SessionType::Variable(_) => 0.0,
            SessionType::End => 0.0,
            _ => 0.0,
        }
    }
}

/// Graph visualizer for TEG and execution flow
#[derive(Debug)]
pub struct GraphVisualizer {
    nodes: BTreeMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}

/// Node in the execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub metadata: BTreeMap<String, String>,
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
            nodes: BTreeMap::new(),
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
                    let mut meta = BTreeMap::new();
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
            metadata: BTreeMap::new(),
        };
        
        let node2 = GraphNode {
            id: "node2".to_string(),
            label: "Effect 2".to_string(),
            node_type: "effect".to_string(),
            metadata: BTreeMap::new(),
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