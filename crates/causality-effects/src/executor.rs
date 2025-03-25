// Effect executor implementation
// Original file: src/effect/executor.rs

// Executor module for Content-Addressable Effects
//
// This module provides the executor for content-addressable code with
// comprehensive tracing, sandboxing, and security features.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};
use chrono::Utc;
use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use crate::effect::{Effect, EffectOutcome};
use causality_effects::ContentHash;
use causality_effects::{CodeRepository, CodeEntry};
use causality_crypto::ContentId;
use crate::crypto::content_addressed::{ContentAddressed, ContentId};

/// Value type for the execution environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// List of values
    Array(Vec<Value>),
    /// Map of string keys to values
    Map(HashMap<String, Value>),
    /// Reference to another value by ID
    Ref(String),
    /// Content-addressed code reference
    CodeRef(ContentHash),
}

impl Value {
    /// Convert a value to a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Convert a value to an integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }
    
    /// Convert a value to a float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }
    
    /// Convert a value to a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Convert a value to bytes
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }
    
    /// Convert a value to an array
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }
    
    /// Convert a value to a map
    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }
    
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// Content data for context ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ContextIdContentData {
    /// Creation timestamp
    pub timestamp: u64,
    
    /// Creator information (optional)
    pub creator: Option<String>,
    
    /// Random nonce for uniqueness
    pub nonce: [u8; 16],
}

impl ContentAddressed for ContextIdContentData {
    fn content_hash(&self) -> Result<ContentId> {
        let bytes = self.to_bytes()?;
        Ok(ContentId::from_bytes(&bytes)?)
    }
    
    fn verify(&self, content_id: &ContentId) -> Result<bool> {
        let calculated_id = self.content_hash()?;
        Ok(calculated_id == *content_id)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = borsh::to_vec(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize ContextIdContentData: {}", e)))?;
        Ok(bytes)
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes)
            .map_err(|e| Error::Deserialization(format!("Failed to deserialize ContextIdContentData: {}", e)))
    }
}

/// A unique identifier for an execution context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(pub String);

impl ContextId {
    /// Create a new random context ID
    pub fn new() -> Self {
        // Generate content data for the context ID
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut nonce = [0u8; 16];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = ContextIdContentData {
            timestamp: now,
            creator: None,
            nonce,
        };
        
        let id = content_data.content_hash()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| format!("ctx-error-{}", now));
            
        ContextId(id)
    }
    
    /// Create a context ID from a string
    pub fn from_string(id: &str) -> Self {
        ContextId(id.to_string())
    }
    
    /// Create a context ID with creator information
    pub fn with_creator(creator: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut nonce = [0u8; 16];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = ContextIdContentData {
            timestamp: now,
            creator: Some(creator.to_string()),
            nonce,
        };
        
        let id = content_data.content_hash()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| format!("ctx-error-{}", now));
            
        ContextId(id)
    }
}

/// Execution event emitted during code execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Function call started
    CallStart {
        /// Context ID
        context_id: ContextId,
        /// Function name
        name: Option<String>,
        /// Arguments
        args: Vec<Value>,
        /// Call timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Function call completed
    CallEnd {
        /// Context ID
        context_id: ContextId,
        /// Return value
        result: Value,
        /// Execution time in milliseconds
        duration_ms: u64,
        /// Call timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Variable assignment
    Assignment {
        /// Context ID
        context_id: ContextId,
        /// Variable name
        name: String,
        /// Variable value
        value: Value,
        /// Assignment timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Effect executed
    EffectExecuted {
        /// Context ID
        context_id: ContextId,
        /// Effect name
        name: String,
        /// Effect parameters
        params: HashMap<String, Value>,
        /// Effect result
        result: EffectOutcome,
        /// Execution time in milliseconds
        duration_ms: u64,
        /// Execution timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Error occurred
    Error {
        /// Context ID
        context_id: ContextId,
        /// Error message
        message: String,
        /// Error timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Resource accessed
    ResourceAccess {
        /// Context ID
        context_id: ContextId,
        /// Resource ID
        resource_id: ContentId,
        /// Access type (read/write)
        access_type: String,
        /// Access timestamp
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Security sandbox for code execution
pub trait SecuritySandbox: Send + Sync {
    /// Check if the given operation is allowed
    fn check_operation(&self, operation: &str, args: &[Value]) -> Result<()>;
    
    /// Check if access to a resource is allowed
    fn check_resource_access(&self, resource_id: &ContentId, access_type: &str) -> Result<()>;
    
    /// Get the maximum execution time allowed
    fn max_execution_time(&self) -> Duration;
    
    /// Get the maximum memory allowed (in bytes)
    fn max_memory(&self) -> u64;
}

/// Default security sandbox implementation
pub struct DefaultSecuritySandbox {
    max_execution_time: Duration,
    max_memory: u64,
}

impl DefaultSecuritySandbox {
    /// Create a new default security sandbox
    pub fn new() -> Self {
        Self {
            max_execution_time: Duration::from_secs(5),
            max_memory: 1024 * 1024 * 10, // 10 MB
        }
    }
    
    /// Create a new security sandbox with custom limits
    pub fn with_limits(max_execution_time: Duration, max_memory: u64) -> Self {
        Self {
            max_execution_time,
            max_memory,
        }
    }
}

impl SecuritySandbox for DefaultSecuritySandbox {
    fn check_operation(&self, _operation: &str, _args: &[Value]) -> Result<()> {
        // All operations allowed by default
        Ok(())
    }
    
    fn check_resource_access(&self, _resource_id: &ContentId, _access_type: &str) -> Result<()> {
        // All resource access allowed by default
        Ok(())
    }
    
    fn max_execution_time(&self) -> Duration {
        self.max_execution_time
    }
    
    fn max_memory(&self) -> u64 {
        self.max_memory
    }
}

/// Execution context for content-addressable code
pub struct ExecutionContext {
    /// Unique identifier for this context
    pub id: ContextId,
    /// Variables in the current scope
    variables: HashMap<String, Value>,
    /// Call stack
    call_stack: Vec<CallFrame>,
    /// Start time
    start_time: Instant,
    /// Security sandbox
    security: Arc<dyn SecuritySandbox>,
    /// Event listener
    event_listener: Option<Box<dyn Fn(ExecutionEvent) + Send + Sync>>,
}

/// A single frame in the call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Code hash
    pub code_hash: ContentHash,
    /// Function name
    pub name: Option<String>,
    /// Arguments
    pub args: Vec<Value>,
    /// Start time
    pub start_time: Instant,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(security: Arc<dyn SecuritySandbox>) -> Self {
        Self {
            id: ContextId::new(),
            variables: HashMap::new(),
            call_stack: Vec::new(),
            start_time: Instant::now(),
            security,
            event_listener: None,
        }
    }
    
    /// Set an event listener
    pub fn set_event_listener<F>(&mut self, listener: F)
    where
        F: Fn(ExecutionEvent) + Send + Sync + 'static,
    {
        self.event_listener = Some(Box::new(listener));
    }
    
    /// Emit an execution event
    pub fn emit_event(&self, event: ExecutionEvent) {
        if let Some(listener) = &self.event_listener {
            listener(event);
        }
    }
    
    /// Get a variable from the context
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
    
    /// Set a variable in the context
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value.clone());
        
        // Emit assignment event
        self.emit_event(ExecutionEvent::Assignment {
            context_id: self.id.clone(),
            name: name.to_string(),
            value,
            timestamp: Utc::now(),
        });
    }
    
    /// Push a call frame onto the stack
    pub fn push_call(&mut self, code_hash: ContentHash, name: Option<String>, args: Vec<Value>) {
        let frame = CallFrame {
            code_hash,
            name: name.clone(),
            args: args.clone(),
            start_time: Instant::now(),
        };
        
        self.call_stack.push(frame);
        
        // Emit call start event
        self.emit_event(ExecutionEvent::CallStart {
            context_id: self.id.clone(),
            name,
            args,
            timestamp: Utc::now(),
        });
    }
    
    /// Pop a call frame from the stack and return the elapsed time
    pub fn pop_call(&mut self, result: Value) -> Duration {
        if let Some(frame) = self.call_stack.pop() {
            let duration = frame.start_time.elapsed();
            
            // Emit call end event
            self.emit_event(ExecutionEvent::CallEnd {
                context_id: self.id.clone(),
                result: result.clone(),
                duration_ms: duration.as_millis() as u64,
                timestamp: Utc::now(),
            });
            
            duration
        } else {
            // No call frame to pop
            Duration::from_secs(0)
        }
    }
    
    /// Check if the context has exceeded its execution time limit
    pub fn check_timeout(&self) -> Result<()> {
        let elapsed = self.start_time.elapsed();
        let max_time = self.security.max_execution_time();
        
        if elapsed > max_time {
            Err(Error::ExecutionTimeout)
        } else {
            Ok(())
        }
    }
    
    /// Check if an operation is allowed
    pub fn check_operation(&self, operation: &str, args: &[Value]) -> Result<()> {
        self.security.check_operation(operation, args)
    }
    
    /// Check if resource access is allowed
    pub fn check_resource_access(&self, resource_id: &ContentId, access_type: &str) -> Result<()> {
        self.security.check_resource_access(resource_id, access_type)
    }
}

/// Content-addressable executor for effects
pub struct ContentAddressableExecutor<R: CodeRepository> {
    repository: Arc<R>,
    security: Arc<dyn SecuritySandbox>,
}

impl<R: CodeRepository> ContentAddressableExecutor<R> {
    /// Create a new content-addressable executor
    pub fn new(repository: Arc<R>, security: Arc<dyn SecuritySandbox>) -> Self {
        Self {
            repository,
            security,
        }
    }
    
    /// Execute content-addressed code
    pub async fn execute(&self, code_hash: &ContentHash, args: Vec<Value>) -> Result<Value> {
        // Get the code definition
        let code_entry = self.repository.get_code(code_hash).await?
            .ok_or_else(|| Error::NotFound(format!("Code hash not found: {}", code_hash)))?;
        
        // Create execution context
        let mut context = ExecutionContext::new(self.security.clone());
        
        // Extract the code name if available
        let code_name = code_entry.definition.name.clone();
        
        // Push initial call frame
        context.push_call(code_hash.clone(), code_name, args.clone());
        
        // TODO: Execute the code based on its content type
        // For now, just return a dummy value
        let result = Value::String("Code execution not yet implemented".to_string());
        
        // Pop the call frame
        context.pop_call(result.clone());
        
        Ok(result)
    }
    
    /// Execute an effect with content-addressed code
    pub async fn execute_effect<E: Effect + 'static>(&self, effect: &E) -> Result<EffectOutcome> {
        // Get the start time
        let start_time = Instant::now();
        
        // Execute the effect
        let outcome = effect.execute().await?;
        
        // Calculate the duration
        let duration = start_time.elapsed();
        
        Ok(outcome)
    }
} 
