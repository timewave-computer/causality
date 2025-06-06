//! Session management for the Causality API
//!
//! This module provides session state management for maintaining
//! execution context across multiple API calls.

use std::collections::HashMap;

/// Execution session state
#[derive(Debug)]
pub struct ExecutionSession {
    /// Session ID
    pub id: String,
    
    /// Current executor state
    pub executor: causality_runtime::Executor,
    
    /// Session metadata
    pub metadata: SessionMetadata,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last accessed timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

impl ExecutionSession {
    /// Create a new execution session
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        
        Self {
            id,
            executor: causality_runtime::Executor::new(),
            metadata: SessionMetadata::default(),
            created_at: now,
            last_accessed: now,
        }
    }
    
    /// Touch the session to update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
    }
}

/// Session metadata
#[derive(Debug, Default)]
pub struct SessionMetadata {
    /// User-defined tags
    pub tags: HashMap<String, String>,
    
    /// Execution statistics
    pub stats: ExecutionStats,
}

/// Execution statistics
#[derive(Debug, Default)]
pub struct ExecutionStats {
    /// Number of compilations performed
    pub compilations: u64,
    
    /// Number of executions performed
    pub executions: u64,
    
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    
    /// Number of errors encountered
    pub errors: u64,
} 