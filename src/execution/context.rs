// Execution context module for Causality Content-Addressed Code System
//
// This module provides the execution context for content-addressed code,
// including variable bindings, call stack management, and execution trace recording.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::effect::EffectType;
use crate::effect_adapters::repository;

/// A unique identifier for an execution context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(String);

impl ContextId {
    /// Create a new random context ID
    pub fn new() -> Self {
        ContextId(Uuid::new_v4().to_string())
    }
    
    /// Create a context ID from a string
    pub fn from_string(id: String) -> Self {
        ContextId(id)
    }
    
    /// Get the string representation of this context ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A value that can be stored in the execution context
#[derive(Clone, Serialize, Deserialize)]
pub enum Value {
    /// A null value
    Null,
    /// A boolean value
    Bool(bool),
    /// An integer value
    Int(i64),
    /// A floating point value
    Float(f64),
    /// A string value
    String(String),
    /// A binary value
    Bytes(Vec<u8>),
    /// An array of values
    Array(Vec<Value>),
    /// A dictionary of values
    Map(HashMap<String, Value>),
    /// A reference to code by hash
    CodeRef(ContentHash),
    /// A reference to a resource
    ResourceRef(String),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Int(i) => write!(f, "Int({})", i),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::String(s) => write!(f, "String(\"{}\")", s),
            Value::Bytes(b) => write!(f, "Bytes({} bytes)", b.len()),
            Value::Array(a) => write!(f, "Array({} items)", a.len()),
            Value::Map(m) => write!(f, "Map({} entries)", m.len()),
            Value::CodeRef(c) => write!(f, "CodeRef({})", c),
            Value::ResourceRef(r) => write!(f, "ResourceRef({})", r),
        }
    }
}

/// An error that can occur during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    /// Invalid argument
    InvalidArgument(String),
    /// Type error
    TypeError(String),
    /// Runtime error
    RuntimeError(String),
    /// Effect error
    EffectError(String),
    /// Resource error
    ResourceError(String),
    /// Security error
    SecurityError(String),
    /// Timeout error
    TimeoutError,
    /// Out of memory error
    OutOfMemory,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            ExecutionError::TypeError(msg) => write!(f, "Type error: {}", msg),
            ExecutionError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            ExecutionError::EffectError(msg) => write!(f, "Effect error: {}", msg),
            ExecutionError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            ExecutionError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            ExecutionError::TimeoutError => write!(f, "Execution timed out"),
            ExecutionError::OutOfMemory => write!(f, "Out of memory"),
        }
    }
}

/// A single frame in the call stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    /// The hash of the code being executed
    pub code_hash: ContentHash,
    /// The name of the function, if known
    pub name: Option<String>,
    /// Arguments to the function
    pub arguments: Vec<Value>,
    /// The start time of this call frame
    pub start_time: SystemTime,
}

impl CallFrame {
    /// Create a new call frame
    pub fn new(code_hash: ContentHash, name: Option<String>, arguments: Vec<Value>) -> Self {
        CallFrame {
            code_hash,
            name,
            arguments,
            start_time: SystemTime::now(),
        }
    }
    
    /// Get the elapsed time for this call frame
    pub fn elapsed(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.start_time).unwrap_or_default()
    }
}

/// Events recorded during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// A function was called
    FunctionCall {
        /// The hash of the function
        hash: ContentHash,
        /// The name of the function, if known
        name: Option<String>,
        /// Arguments to the function
        arguments: Vec<Value>,
        /// The timestamp when this event occurred
        timestamp: u64,
    },
    /// A function returned
    FunctionReturn {
        /// The hash of the function
        hash: ContentHash,
        /// The returned value
        result: Value,
        /// The timestamp when this event occurred
        timestamp: u64,
    },
    /// An effect was applied
    EffectApplied {
        /// The type of effect
        effect_type: EffectType,
        /// Parameters for the effect
        parameters: HashMap<String, Value>,
        /// The result of the effect
        result: Value,
        /// The timestamp when this event occurred
        timestamp: u64,
    },
    /// An error occurred
    Error(ExecutionError),
}

impl ExecutionEvent {
    /// Create a function call event
    pub fn function_call(hash: ContentHash, name: Option<String>, arguments: Vec<Value>) -> Self {
        ExecutionEvent::FunctionCall {
            hash,
            name,
            arguments,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// Create a function return event
    pub fn function_return(hash: ContentHash, result: Value) -> Self {
        ExecutionEvent::FunctionReturn {
            hash,
            result,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// Create an effect applied event
    pub fn effect_applied(
        effect_type: EffectType,
        parameters: HashMap<String, Value>,
        result: Value,
    ) -> Self {
        ExecutionEvent::EffectApplied {
            effect_type,
            parameters,
            result,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// Create an error event
    pub fn error(error: ExecutionError) -> Self {
        ExecutionEvent::Error(error)
    }
    
    /// Get the timestamp for this event
    pub fn timestamp(&self) -> u64 {
        match self {
            ExecutionEvent::FunctionCall { timestamp, .. } => *timestamp,
            ExecutionEvent::FunctionReturn { timestamp, .. } => *timestamp,
            ExecutionEvent::EffectApplied { timestamp, .. } => *timestamp,
            ExecutionEvent::Error(_) => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

/// A context for code execution
pub struct ExecutionContext {
    /// Unique identifier for this context
    pub context_id: ContextId,
    /// Parent context, if any
    pub parent: Option<Arc<ExecutionContext>>,
    /// The code repository to use
    pub repository: Arc<dyn repository::CodeRepository>,
    /// Variable bindings in this context
    variables: RwLock<HashMap<String, Value>>,
    /// Current call stack
    call_stack: RwLock<Vec<CallFrame>>,
    /// Execution trace
    execution_trace: RwLock<Vec<ExecutionEvent>>,
    /// Resource allocator
    resource_allocator: Arc<dyn crate::resource::ResourceAllocator>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        context_id: ContextId,
        repository: Arc<dyn repository::CodeRepository>,
        resource_allocator: Arc<dyn crate::resource::ResourceAllocator>,
        parent: Option<Arc<ExecutionContext>>,
    ) -> Self {
        ExecutionContext {
            context_id,
            parent,
            repository,
            variables: RwLock::new(HashMap::new()),
            call_stack: RwLock::new(Vec::new()),
            execution_trace: RwLock::new(Vec::new()),
            resource_allocator,
        }
    }
    
    /// Create a new execution context with a random ID
    pub fn new_with_random_id(
        repository: Arc<dyn repository::CodeRepository>,
        resource_allocator: Arc<dyn crate::resource::ResourceAllocator>,
        parent: Option<Arc<ExecutionContext>>,
    ) -> Self {
        Self::new(ContextId::new(), repository, resource_allocator, parent)
    }
    
    /// Get the context ID
    pub fn id(&self) -> &ContextId {
        &self.context_id
    }
    
    /// Check if this context has a parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    
    /// Get the resource allocator
    pub fn resource_allocator(&self) -> Arc<dyn crate::resource::ResourceAllocator> {
        self.resource_allocator.clone()
    }
    
    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Result<Option<Value>> {
        // Check in this context first
        {
            let variables = self.variables.read().map_err(|_| Error::LockError)?;
            if let Some(value) = variables.get(name) {
                return Ok(Some(value.clone()));
            }
        }
        
        // Check parent context if available
        if let Some(parent) = &self.parent {
            return parent.get_variable(name);
        }
        
        Ok(None)
    }
    
    /// Set a variable value
    pub fn set_variable(&self, name: String, value: Value) -> Result<()> {
        let mut variables = self.variables.write().map_err(|_| Error::LockError)?;
        variables.insert(name, value);
        Ok(())
    }
    
    /// Push a call frame onto the stack
    pub fn push_call_frame(&self, frame: CallFrame) -> Result<()> {
        // Record the function call event
        self.record_event(ExecutionEvent::function_call(
            frame.code_hash.clone(),
            frame.name.clone(),
            frame.arguments.clone(),
        ))?;
        
        // Push the frame
        let mut call_stack = self.call_stack.write().map_err(|_| Error::LockError)?;
        call_stack.push(frame);
        Ok(())
    }
    
    /// Pop a call frame from the stack
    pub fn pop_call_frame(&self) -> Result<Option<CallFrame>> {
        let mut call_stack = self.call_stack.write().map_err(|_| Error::LockError)?;
        let frame = call_stack.pop();
        Ok(frame)
    }
    
    /// Record an execution event
    pub fn record_event(&self, event: ExecutionEvent) -> Result<()> {
        let mut trace = self.execution_trace.write().map_err(|_| Error::LockError)?;
        trace.push(event);
        Ok(())
    }
    
    /// Get the current execution trace
    pub fn execution_trace(&self) -> Result<Vec<ExecutionEvent>> {
        let trace = self.execution_trace.read().map_err(|_| Error::LockError)?;
        Ok(trace.clone())
    }
    
    /// Get the current call stack
    pub fn call_stack(&self) -> Result<Vec<CallFrame>> {
        let stack = self.call_stack.read().map_err(|_| Error::LockError)?;
        Ok(stack.clone())
    }
    
    /// Record a function return event and pop the call stack
    pub fn record_return(&self, result: Value) -> Result<()> {
        // Pop the call frame
        let frame = self.pop_call_frame()?
            .ok_or_else(|| Error::RuntimeError("Call stack underflow".to_string()))?;
        
        // Record the return event
        self.record_event(ExecutionEvent::function_return(
            frame.code_hash,
            result,
        ))?;
        
        Ok(())
    }
    
    /// Create a child context
    pub fn create_child(&self) -> Self {
        ExecutionContext::new_with_random_id(
            self.repository.clone(),
            self.resource_allocator.clone(),
            Some(Arc::new(self.clone())),
        )
    }
}

impl Clone for ExecutionContext {
    fn clone(&self) -> Self {
        let variables = self.variables.read().unwrap_or_default().clone();
        let call_stack = self.call_stack.read().unwrap_or_default().clone();
        let execution_trace = self.execution_trace.read().unwrap_or_default().clone();
        
        ExecutionContext {
            context_id: self.context_id.clone(),
            parent: self.parent.clone(),
            repository: self.repository.clone(),
            variables: RwLock::new(variables),
            call_stack: RwLock::new(call_stack),
            execution_trace: RwLock::new(execution_trace),
            resource_allocator: self.resource_allocator.clone(),
        }
    }
}

impl fmt::Debug for ExecutionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionContext")
            .field("context_id", &self.context_id)
            .field("has_parent", &self.has_parent())
            .field("call_stack_depth", &self.call_stack.read().map(|s| s.len()).unwrap_or(0))
            .field("trace_events", &self.execution_trace.read().map(|t| t.len()).unwrap_or(0))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    // Mock implementations for testing
    struct MockCodeRepository;
    impl repository::CodeRepository for MockCodeRepository {
        // Implement required methods here
    }
    
    struct MockResourceAllocator;
    impl crate::resource::ResourceAllocator for MockResourceAllocator {
        // Implement required methods here
    }
    
    #[test]
    fn test_context_creation() {
        let repo = Arc::new(MockCodeRepository);
        let allocator = Arc::new(MockResourceAllocator);
        let context = ExecutionContext::new_with_random_id(repo, allocator, None);
        
        assert!(!context.has_parent());
        // Further assertions...
    }
    
    #[test]
    fn test_variable_bindings() {
        // Test variable setting and getting
    }
    
    #[test]
    fn test_call_stack() {
        // Test call stack management
    }
    
    #[test]
    fn test_execution_trace() {
        // Test execution trace recording
    }
    
    #[test]
    fn test_parent_child_relationship() {
        // Test parent-child context relationship
    }
} 