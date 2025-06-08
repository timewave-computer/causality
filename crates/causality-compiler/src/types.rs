//! Types module for compiler results and responses.

use serde::{Serialize, Deserialize};

/// Result of a compilation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompileResult {
    /// Successful compilation with instruction count and metadata
    Success {
        instruction_count: usize,
        program_size: usize,
        compilation_time_ms: u64,
        optimizations_applied: Vec<String>,
        warnings: Vec<String>,
    },
    /// Failed compilation with error message
    Error {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
        suggestions: Vec<String>,
    },
}

impl CompileResult {
    /// Create a successful compilation result
    pub fn success(instruction_count: usize, program_size: usize, compilation_time_ms: u64) -> Self {
        Self::Success {
            instruction_count,
            program_size,
            compilation_time_ms,
            optimizations_applied: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Create an error compilation result
    pub fn error(message: String) -> Self {
        Self::Error {
            message,
            line: None,
            column: None,
            suggestions: Vec::new(),
        }
    }
} 