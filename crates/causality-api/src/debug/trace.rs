//! Execution Trace Debugging
//!
//! This module provides interfaces and implementations for collecting, analyzing,
//! and visualizing execution traces to help with debugging and performance analysis.
//! All implementations maintain ZK compatibility through bounded sizes and fixed data structures.

use async_trait::async_trait;
// Serialization imports removed as we don't use manual SSZ implementations here
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;

use causality_types::core::{
    AsErrorContext, ContextualError, ErrorCategory, ErrorMetadata,
};

/// Maximum number of trace entries to store
pub const MAX_TRACE_ENTRIES: usize = 1000;

/// Maximum size of a trace entry (in bytes)
pub const MAX_TRACE_ENTRY_SIZE: usize = 1024 * 1024; // 1 MB

//-----------------------------------------------------------------------------
// Core Trace Definition
//-----------------------------------------------------------------------------

/// Trace entry type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEntryType {
    /// Function entry
    FunctionEntry,

    /// Function exit
    FunctionExit,

    /// Variable assignment
    VariableAssignment,

    /// Expression evaluation
    ExpressionEvaluation,

    /// Effect application
    EffectApplication,

    /// State transition
    StateTransition,

    /// External interaction
    ExternalInteraction,

    /// Assertion check
    Assertion,

    /// Custom event
    Custom(String),
}

/// A single entry in an execution trace
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Unique identifier for this trace entry
    pub id: [u8; 16],

    /// Entry type
    pub entry_type: TraceEntryType,

    /// Location in code (file:line:column)
    pub location: String,

    /// Timestamp (nanoseconds since epoch)
    pub timestamp: u64,

    /// Entry data serialized as bytes
    pub data: Vec<u8>,

    /// Parent entry ID (if any)
    pub parent_id: Option<[u8; 16]>,

    /// Related expression ID (if any)
    pub expression_id: Option<[u8; 32]>,
}

/// Execution trace identifier
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct TraceId([u8; 32]);

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceId {
    /// Create a new random trace ID
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        Self(bytes)
    }

    /// Convert to hexadecimal string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Try to parse from hexadecimal string
    pub fn from_hex(hex_str: &str) -> Result<Self, ContextualError> {
        let bytes = hex::decode(hex_str).map_err(|_| {
            ContextualError::new(
                "Invalid hex string for trace ID",
                ErrorMetadata::new(ErrorCategory::Validation),
            )
        })?;

        if bytes.len() != 32 {
            return Err(ContextualError::new(
                "Trace ID must be 32 bytes",
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        let mut id = [0u8; 32];
        id.copy_from_slice(&bytes);
        Ok(Self(id))
    }
}

/// Full execution trace containing all entries
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    /// Unique trace identifier
    pub id: TraceId,

    /// Trace name
    pub name: String,

    /// Trace entries
    pub entries: VecDeque<TraceEntry>,

    /// Creation timestamp
    pub created_at: u64,

    /// Last updated timestamp
    pub updated_at: u64,

    /// Metadata (key-value pairs)
    pub metadata: std::collections::HashMap<String, String>,
}

//-----------------------------------------------------------------------------
// Trace Collection Interface
//-----------------------------------------------------------------------------

/// Interface for collecting and analyzing execution traces
#[async_trait]
pub trait TraceCollector: Send + Sync {
    /// Create a new trace
    async fn create_trace(&self, name: &str) -> Result<TraceId, ContextualError>;

    /// Add an entry to an existing trace
    async fn add_entry(
        &self,
        trace_id: &TraceId,
        entry: TraceEntry,
    ) -> Result<(), ContextualError>;

    /// Get a trace by ID
    async fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> Result<ExecutionTrace, ContextualError>;

    /// List all available traces
    async fn list_traces(&self) -> Result<Vec<(TraceId, String)>, ContextualError>;

    /// Find traces matching a pattern
    async fn find_traces(
        &self,
        pattern: &str,
    ) -> Result<Vec<(TraceId, String)>, ContextualError>;

    /// Delete a trace
    async fn delete_trace(&self, trace_id: &TraceId) -> Result<(), ContextualError>;
}

//-----------------------------------------------------------------------------
// Trace Analysis Interface
//-----------------------------------------------------------------------------

/// Interface for trace analysis
#[async_trait]
pub trait TraceAnalyzer: Send + Sync {
    /// Analyze an execution trace
    async fn analyze_trace(
        &self,
        trace_id: &TraceId,
    ) -> Result<TraceAnalysis, ContextualError>;

    /// Find performance bottlenecks in a trace
    async fn find_bottlenecks(
        &self,
        trace_id: &TraceId,
    ) -> Result<Vec<Bottleneck>, ContextualError>;

    /// Find potential bugs or issues in a trace
    async fn find_issues(
        &self,
        trace_id: &TraceId,
    ) -> Result<Vec<TraceIssue>, ContextualError>;

    /// Compare two traces
    async fn compare_traces(
        &self,
        trace_id1: &TraceId,
        trace_id2: &TraceId,
    ) -> Result<TraceComparison, ContextualError>;
}

/// Result of a trace analysis
#[derive(Debug, Clone)]
pub struct TraceAnalysis {
    /// Trace ID
    pub trace_id: TraceId,

    /// Execution time statistics (in nanoseconds)
    pub execution_time: ExecutionTimeStats,

    /// Function call statistics
    pub function_calls: Vec<FunctionCallStats>,

    /// State transition statistics
    pub state_transitions: Vec<StateTransitionStats>,

    /// External interaction statistics
    pub external_interactions: Vec<ExternalInteractionStats>,

    /// Detected bottlenecks
    pub bottlenecks: Vec<Bottleneck>,

    /// Detected issues
    pub issues: Vec<TraceIssue>,
}

/// Execution time statistics
#[derive(Debug, Clone)]
pub struct ExecutionTimeStats {
    /// Total execution time (nanoseconds)
    pub total_time: u64,

    /// Average time per entry (nanoseconds)
    pub average_time_per_entry: u64,

    /// Maximum time between entries (nanoseconds)
    pub max_time_between_entries: u64,
}

/// Function call statistics
#[derive(Debug, Clone)]
pub struct FunctionCallStats {
    /// Function name
    pub function_name: String,

    /// Number of calls
    pub call_count: u32,

    /// Total execution time (nanoseconds)
    pub total_time: u64,

    /// Average execution time (nanoseconds)
    pub average_time: u64,

    /// Maximum execution time (nanoseconds)
    pub max_time: u64,
}

/// State transition statistics
#[derive(Debug, Clone)]
pub struct StateTransitionStats {
    /// State type
    pub state_type: String,

    /// Number of transitions
    pub transition_count: u32,

    /// Average transition time (nanoseconds)
    pub average_time: u64,
}

/// External interaction statistics
#[derive(Debug, Clone)]
pub struct ExternalInteractionStats {
    /// Interaction type
    pub interaction_type: String,

    /// Number of interactions
    pub interaction_count: u32,

    /// Average interaction time (nanoseconds)
    pub average_time: u64,

    /// Maximum interaction time (nanoseconds)
    pub max_time: u64,
}

/// A detected bottleneck in the execution
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// Bottleneck type
    pub bottleneck_type: String,

    /// Location in code
    pub location: String,

    /// Description
    pub description: String,

    /// Impact level (0-100)
    pub impact: u8,

    /// Potential fix suggestion
    pub suggestion: String,
}

/// A detected issue in the execution trace
#[derive(Debug, Clone)]
pub struct TraceIssue {
    /// Issue type
    pub issue_type: String,

    /// Location in code
    pub location: String,

    /// Description
    pub description: String,

    /// Severity (0-100)
    pub severity: u8,

    /// Potential fix suggestion
    pub suggestion: String,
}

/// Comparison between two traces
#[derive(Debug, Clone)]
pub struct TraceComparison {
    /// First trace ID
    pub trace_id1: TraceId,

    /// Second trace ID
    pub trace_id2: TraceId,

    /// Execution time difference (percentage)
    pub execution_time_diff: f32,

    /// Function call differences
    pub function_call_diffs: Vec<FunctionCallDiff>,

    /// State transition differences
    pub state_transition_diffs: Vec<StateTransitionDiff>,

    /// External interaction differences
    pub external_interaction_diffs: Vec<ExternalInteractionDiff>,

    /// Significant differences
    pub significant_differences: Vec<SignificantDifference>,
}

/// Function call differences
#[derive(Debug, Clone)]
pub struct FunctionCallDiff {
    /// Function name
    pub function_name: String,

    /// Call count difference
    pub call_count_diff: i32,

    /// Average execution time difference (percentage)
    pub average_time_diff: f32,
}

/// State transition differences
#[derive(Debug, Clone)]
pub struct StateTransitionDiff {
    /// State type
    pub state_type: String,

    /// Transition count difference
    pub transition_count_diff: i32,

    /// Average time difference (percentage)
    pub average_time_diff: f32,
}

/// External interaction differences
#[derive(Debug, Clone)]
pub struct ExternalInteractionDiff {
    /// Interaction type
    pub interaction_type: String,

    /// Interaction count difference
    pub interaction_count_diff: i32,

    /// Average time difference (percentage)
    pub average_time_diff: f32,
}

/// A significant difference between two traces
#[derive(Debug, Clone)]
pub struct SignificantDifference {
    /// Difference type
    pub difference_type: String,

    /// Description
    pub description: String,

    /// Impact level (0-100)
    pub impact: u8,
}

//-----------------------------------------------------------------------------
// In-Memory Implementations
//-----------------------------------------------------------------------------

/// In-memory implementation of trace collector
pub struct InMemoryTraceCollector {
    /// Map of trace ID to execution trace
    traces: tokio::sync::Mutex<std::collections::HashMap<[u8; 32], ExecutionTrace>>,

    /// Error context
    error_context: Arc<dyn AsErrorContext>,
}

impl InMemoryTraceCollector {
    /// Create a new in-memory trace collector
    pub fn new(error_context: Arc<dyn AsErrorContext>) -> Self {
        Self {
            traces: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            error_context,
        }
    }

    /// Get the current timestamp in nanoseconds
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }
}

#[async_trait]
impl TraceCollector for InMemoryTraceCollector {
    async fn create_trace(&self, name: &str) -> Result<TraceId, ContextualError> {
        let trace_id = TraceId::new();
        let now = self.current_timestamp();

        let trace = ExecutionTrace {
            id: trace_id,
            name: name.to_string(),
            entries: VecDeque::with_capacity(MAX_TRACE_ENTRIES),
            created_at: now,
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        };

        let mut traces = self.traces.lock().await;
        traces.insert(trace_id.0, trace);

        Ok(trace_id)
    }

    async fn add_entry(
        &self,
        trace_id: &TraceId,
        entry: TraceEntry,
    ) -> Result<(), ContextualError> {
        // Validate entry size
        if entry.data.len() > MAX_TRACE_ENTRY_SIZE {
            return Err(self.error_context.create_error(
                format!(
                    "Trace entry data exceeds maximum size: {} > {}",
                    entry.data.len(),
                    MAX_TRACE_ENTRY_SIZE
                ),
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        let mut traces = self.traces.lock().await;

        let trace = traces.get_mut(&trace_id.0).ok_or_else(|| {
            self.error_context.create_error(
                format!("Trace not found: {}", hex::encode(trace_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })?;

        // Add the entry
        trace.entries.push_back(entry);

        // Ensure we don't exceed the maximum number of entries
        while trace.entries.len() > MAX_TRACE_ENTRIES {
            trace.entries.pop_front();
        }

        // Update the timestamp
        trace.updated_at = self.current_timestamp();

        Ok(())
    }

    async fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> Result<ExecutionTrace, ContextualError> {
        let traces = self.traces.lock().await;

        traces.get(&trace_id.0).cloned().ok_or_else(|| {
            self.error_context.create_error(
                format!("Trace not found: {}", hex::encode(trace_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })
    }

    async fn list_traces(&self) -> Result<Vec<(TraceId, String)>, ContextualError> {
        let traces = self.traces.lock().await;

        let result = traces
            .iter()
            .map(|(id, trace)| (TraceId(*id), trace.name.clone()))
            .collect();

        Ok(result)
    }

    async fn find_traces(
        &self,
        pattern: &str,
    ) -> Result<Vec<(TraceId, String)>, ContextualError> {
        let traces = self.traces.lock().await;

        let pattern = pattern.to_lowercase();
        let result = traces
            .iter()
            .filter(|(_, trace)| trace.name.to_lowercase().contains(&pattern))
            .map(|(id, trace)| (TraceId(*id), trace.name.clone()))
            .collect();

        Ok(result)
    }

    async fn delete_trace(&self, trace_id: &TraceId) -> Result<(), ContextualError> {
        let mut traces = self.traces.lock().await;

        if traces.remove(&trace_id.0).is_none() {
            return Err(self.error_context.create_error(
                format!("Trace not found: {}", hex::encode(trace_id.0)),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            ));
        }

        Ok(())
    }
}
