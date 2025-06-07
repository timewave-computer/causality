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

//-----------------------------------------------------------------------------
// ZK Message Types for Neutron Integration
//-----------------------------------------------------------------------------

/// ZK message for Valence authorization contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMessage {
    /// Message identifier
    pub id: String,
    
    /// Circuit identifier this proof was generated with
    pub circuit_id: String,
    
    /// ZK proof data
    pub proof: ZkProofData,
    
    /// Public inputs for verification
    pub public_inputs: Vec<ZkInput>,
    
    /// Authorization context
    pub auth_context: AuthorizationContext,
    
    /// Message timestamp
    pub timestamp: u64,
    
    /// Message signature
    pub signature: Option<String>,
}

/// ZK proof data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofData {
    /// Proof bytes (encoded)
    pub proof_bytes: String,
    
    /// Proof format/encoding
    pub encoding: ProofEncoding,
    
    /// Verification key ID
    pub verification_key_id: String,
    
    /// Proof generation metadata
    pub metadata: ProofMetadata,
}

/// Proof encoding formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofEncoding {
    /// Base64 encoded
    Base64,
    /// Hex encoded
    Hex,
    /// Binary (raw bytes)
    Binary,
}

/// Proof generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Proof generation timestamp
    pub generated_at: u64,
    
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    
    /// Prover service URL
    pub prover_service: Option<String>,
    
    /// Circuit constraints satisfied
    pub constraints_satisfied: bool,
    
    /// Additional metadata
    pub extra: HashMap<String, serde_json::Value>,
}

/// ZK input for public verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkInput {
    /// Input name/label
    pub name: String,
    
    /// Input value
    pub value: ZkInputValue,
    
    /// Input type for verification
    pub input_type: ZkInputType,
}

/// ZK input value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZkInputValue {
    /// Field element
    Field(String),
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<ZkInputValue>),
    /// Hash value
    Hash([u8; 32]),
}

/// ZK input types for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZkInputType {
    /// Block hash
    BlockHash,
    /// Transaction hash
    TxHash,
    /// Account address
    Address,
    /// Storage key
    StorageKey,
    /// Storage value
    StorageValue,
    /// Merkle root
    MerkleRoot,
    /// Custom type
    Custom(String),
}

/// Authorization context for ZK messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationContext {
    /// Authorizing address
    pub authorizer: String,
    
    /// Target contract address
    pub target_contract: String,
    
    /// Authorization action
    pub action: AuthorizationAction,
    
    /// Authorized amount (if applicable)
    pub amount: Option<AuthorizedAmount>,
    
    /// Authorization expiry
    pub expires_at: Option<u64>,
    
    /// Authorization nonce
    pub nonce: u64,
    
    /// Additional context data
    pub context_data: HashMap<String, serde_json::Value>,
}

/// Authorization actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationAction {
    /// Transfer authorization
    Transfer,
    /// Contract execution authorization
    Execute,
    /// Delegation authorization
    Delegate,
    /// Administrative action authorization
    Admin,
    /// Custom action
    Custom(String),
}

/// Authorized amount with denomination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedAmount {
    /// Amount value
    pub amount: String,
    
    /// Token denomination
    pub denom: String,
}

/// ZK message submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMessageSubmissionRequest {
    /// ZK message to submit
    pub message: ZkMessage,
    
    /// Target contract address
    pub contract_address: String,
    
    /// Gas configuration
    pub gas_config: Option<GasConfiguration>,
    
    /// Transaction memo
    pub memo: Option<String>,
    
    /// Whether to wait for confirmation
    pub wait_for_confirmation: bool,
}

/// Gas configuration for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfiguration {
    /// Gas limit
    pub gas_limit: u64,
    
    /// Gas price
    pub gas_price: u64,
    
    /// Gas adjustment factor
    pub gas_adjustment: f64,
}

/// ZK message submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMessageSubmissionResponse {
    /// Transaction hash
    pub tx_hash: String,
    
    /// Transaction status
    pub status: TransactionStatus,
    
    /// Block height
    pub block_height: Option<u64>,
    
    /// Gas used
    pub gas_used: u64,
    
    /// Submission timestamp
    pub submitted_at: u64,
    
    /// Confirmation details (if confirmed)
    pub confirmation: Option<TransactionConfirmation>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Pending in mempool
    Pending,
    /// Included in block
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction was rejected
    Rejected,
}

/// Transaction confirmation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfirmation {
    /// Confirmation timestamp
    pub confirmed_at: u64,
    
    /// Number of confirmations
    pub confirmations: u64,
    
    /// Block hash
    pub block_hash: String,
    
    /// Transaction index in block
    pub tx_index: u32,
}

/// Batch ZK message submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchZkMessageSubmissionRequest {
    /// Multiple ZK messages to submit
    pub messages: Vec<ZkMessage>,
    
    /// Target contract address
    pub contract_address: String,
    
    /// Gas configuration
    pub gas_config: Option<GasConfiguration>,
    
    /// Transaction memo
    pub memo: Option<String>,
    
    /// Whether to wait for all confirmations
    pub wait_for_confirmations: bool,
    
    /// Maximum parallel submissions
    pub max_parallel: Option<u8>,
}

/// Batch ZK message submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchZkMessageSubmissionResponse {
    /// Individual submission results
    pub results: Vec<ZkMessageSubmissionResult>,
    
    /// Overall batch status
    pub batch_status: BatchStatus,
    
    /// Total gas used
    pub total_gas_used: u64,
    
    /// Batch submission timestamp
    pub submitted_at: u64,
    
    /// Number of successful submissions
    pub successful_count: usize,
    
    /// Number of failed submissions
    pub failed_count: usize,
}

/// Individual message submission result in batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMessageSubmissionResult {
    /// Message ID
    pub message_id: String,
    
    /// Submission response (if successful)
    pub response: Option<ZkMessageSubmissionResponse>,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Result status
    pub status: SubmissionResultStatus,
}

/// Submission result status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmissionResultStatus {
    /// Submission successful
    Success,
    /// Submission failed
    Failed,
    /// Submission skipped
    Skipped,
}

/// Batch submission status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    /// All submissions successful
    AllSuccessful,
    /// Some submissions failed
    PartialSuccess,
    /// All submissions failed
    AllFailed,
    /// Batch cancelled
    Cancelled,
} 