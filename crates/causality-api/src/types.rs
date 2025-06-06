//! Type definitions for Causality API
//!
//! Request and response types for the REST API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Request Types
//-----------------------------------------------------------------------------

/// Request to compile source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileRequest {
    /// Source code to compile
    pub source: String,
    
    /// Optional session ID for stateful compilation
    pub session_id: Option<String>,
    
    /// Compilation options
    pub options: Option<CompileOptions>,
}

/// Compilation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOptions {
    /// Enable optimizations
    pub optimize: Option<bool>,
    
    /// Show compilation stages
    pub show_stages: Option<bool>,
    
    /// Target platform
    pub target: Option<String>,
}

/// Request to execute source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    /// Source code to execute
    pub source: String,
    
    /// Optional session ID for stateful execution
    pub session_id: Option<String>,
    
    /// Execution options
    pub options: Option<ExecuteOptions>,
}

/// Execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOptions {
    /// Maximum execution steps
    pub max_steps: Option<u64>,
    
    /// Enable execution trace
    pub trace: Option<bool>,
    
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Request to create a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// Optional session name
    pub name: Option<String>,
    
    /// Optional tags for the session
    pub tags: Option<HashMap<String, String>>,
}

//-----------------------------------------------------------------------------
// Response Types
//-----------------------------------------------------------------------------

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status
    pub status: String,
    
    /// Response data
    pub data: T,
    
    /// Optional error message
    pub error: Option<String>,
    
    /// Request timestamp
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data,
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Create an error response
    pub fn error(error: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            status: "error".to_string(),
            data: (),
            error: Some(error.into()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    /// Compilation time in milliseconds
    pub compilation_time_ms: u64,
    
    /// Number of generated instructions
    pub instruction_count: usize,
    
    /// Generated instructions (if requested)
    pub instructions: Option<Vec<InstructionInfo>>,
    
    /// Compilation warnings
    pub warnings: Vec<String>,
}

/// Instruction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionInfo {
    /// Instruction index
    pub index: usize,
    
    /// Instruction description
    pub instruction: String,
    
    /// Optional source location
    pub source_location: Option<SourceLocation>,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    
    /// Column number (1-based)
    pub column: usize,
    
    /// Length in characters
    pub length: Option<usize>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Execution result value
    pub result: String,
    
    /// Number of instructions executed
    pub instruction_count: usize,
    
    /// Execution trace (if requested)
    pub trace: Option<Vec<ExecutionStep>>,
    
    /// Final machine state (if requested)
    pub machine_state: Option<MachineStateInfo>,
}

/// Execution step information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number
    pub step: usize,
    
    /// Instruction executed
    pub instruction: String,
    
    /// Register states before execution
    pub registers_before: HashMap<String, String>,
    
    /// Register states after execution
    pub registers_after: HashMap<String, String>,
}

/// Machine state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineStateInfo {
    /// Register states
    pub registers: HashMap<String, String>,
    
    /// Current program counter
    pub program_counter: usize,
    
    /// Execution statistics
    pub stats: ExecutionStatsInfo,
}

/// Execution statistics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatsInfo {
    /// Steps executed
    pub steps_executed: u64,
    
    /// Memory usage in bytes
    pub memory_usage: u64,
    
    /// CPU time in microseconds
    pub cpu_time_us: u64,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: String,
    
    /// Session name
    pub name: Option<String>,
    
    /// Creation timestamp
    pub created_at: String,
    
    /// Last accessed timestamp
    pub last_accessed: String,
    
    /// Session tags
    pub tags: HashMap<String, String>,
    
    /// Session statistics
    pub stats: SessionStatsInfo,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatsInfo {
    /// Number of compilations performed
    pub compilations: u64,
    
    /// Number of executions performed
    pub executions: u64,
    
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    
    /// Number of errors encountered
    pub errors: u64,
    
    /// Number of warnings encountered
    pub warnings: u64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Service status
    pub status: String,
    
    /// Service name
    pub service: String,
    
    /// Service version
    pub version: String,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
    
    /// System information
    pub system: SystemInfo,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Available memory in bytes
    pub available_memory: u64,
    
    /// CPU usage percentage
    pub cpu_usage: f64,
    
    /// Active sessions count
    pub active_sessions: usize,
    
    /// Total requests served
    pub total_requests: u64,
} 