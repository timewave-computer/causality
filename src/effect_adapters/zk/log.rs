//! Logging and diagnostics for ZK effect adapters
//!
//! This module provides logging utilities and diagnostic tools 
//! for tracking and debugging ZK operations.

use std::time::{Duration, Instant};
use log::{debug, error, info, trace, warn};
use crate::effect_adapters::zk::error::{Result, ZkError};

/// Log levels for ZK operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level (most verbose)
    Trace,
    /// Debug level
    Debug,
    /// Info level (default)
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Operation type for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Proof generation
    ProofGeneration,
    /// Proof verification
    ProofVerification,
    /// Witness generation
    WitnessGeneration,
    /// Program compilation
    ProgramCompilation,
    /// Code generation
    CodeGeneration,
    /// Circuit execution
    CircuitExecution,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::ProofGeneration => write!(f, "Proof Generation"),
            OperationType::ProofVerification => write!(f, "Proof Verification"),
            OperationType::WitnessGeneration => write!(f, "Witness Generation"),
            OperationType::ProgramCompilation => write!(f, "Program Compilation"),
            OperationType::CodeGeneration => write!(f, "Code Generation"),
            OperationType::CircuitExecution => write!(f, "Circuit Execution"),
        }
    }
}

/// Operation statistics for performance tracking
#[derive(Debug, Clone)]
pub struct OperationStats {
    /// Operation type
    pub operation_type: OperationType,
    /// Start time
    pub start_time: Instant,
    /// Duration
    pub duration: Option<Duration>,
    /// Memory usage in bytes
    pub memory_usage: Option<usize>,
    /// Number of constraints
    pub constraint_count: Option<usize>,
    /// Operation status
    pub status: OperationStatus,
    /// Error message if any
    pub error_message: Option<String>,
}

/// Operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation not started
    NotStarted,
    /// Operation in progress
    InProgress,
    /// Operation completed successfully
    Success,
    /// Operation failed
    Failed,
}

impl std::fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationStatus::NotStarted => write!(f, "Not Started"),
            OperationStatus::InProgress => write!(f, "In Progress"),
            OperationStatus::Success => write!(f, "Success"),
            OperationStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl OperationStats {
    /// Create new operation statistics
    pub fn new(operation_type: OperationType) -> Self {
        OperationStats {
            operation_type,
            start_time: Instant::now(),
            duration: None,
            memory_usage: None,
            constraint_count: None,
            status: OperationStatus::NotStarted,
            error_message: None,
        }
    }
    
    /// Start the operation
    pub fn start(&mut self) {
        self.start_time = Instant::now();
        self.status = OperationStatus::InProgress;
        self.log_start();
    }
    
    /// Complete the operation successfully
    pub fn complete(&mut self) {
        self.duration = Some(self.start_time.elapsed());
        self.status = OperationStatus::Success;
        self.log_completion();
    }
    
    /// Mark the operation as failed
    pub fn fail(&mut self, error: &ZkError) {
        self.duration = Some(self.start_time.elapsed());
        self.status = OperationStatus::Failed;
        self.error_message = Some(format!("{}", error));
        self.log_failure();
    }
    
    /// Log the start of an operation
    fn log_start(&self) {
        info!("Starting ZK operation: {}", self.operation_type);
        trace!("ZK operation details: type={:?}, time={:?}", self.operation_type, self.start_time);
    }
    
    /// Log the completion of an operation
    fn log_completion(&self) {
        if let Some(duration) = self.duration {
            info!(
                "Completed ZK operation: {}, duration: {:?}",
                self.operation_type, duration
            );
            
            if let Some(constraint_count) = self.constraint_count {
                debug!("ZK operation constraints: {}", constraint_count);
            }
            
            if let Some(memory_usage) = self.memory_usage {
                debug!("ZK operation memory usage: {} bytes", memory_usage);
            }
        }
    }
    
    /// Log a failed operation
    fn log_failure(&self) {
        if let Some(duration) = self.duration {
            error!(
                "Failed ZK operation: {}, duration: {:?}, error: {}",
                self.operation_type,
                duration,
                self.error_message.as_deref().unwrap_or("Unknown error")
            );
        }
    }
}

/// Operation logger for ZK operations
#[derive(Debug)]
pub struct ZkLogger {
    /// Default log level
    pub level: LogLevel,
    /// Operation statistics history
    pub history: Vec<OperationStats>,
    /// Maximum history size
    pub max_history: usize,
    /// Debug mode
    pub debug_mode: bool,
}

impl Default for ZkLogger {
    fn default() -> Self {
        ZkLogger {
            level: LogLevel::Info,
            history: Vec::new(),
            max_history: 100,
            debug_mode: false,
        }
    }
}

impl ZkLogger {
    /// Create a new ZK logger
    pub fn new() -> Self {
        ZkLogger::default()
    }
    
    /// Create a logger with debug mode enabled
    pub fn with_debug() -> Self {
        ZkLogger {
            level: LogLevel::Debug,
            history: Vec::new(),
            max_history: 100,
            debug_mode: true,
        }
    }
    
    /// Set the log level
    pub fn set_level(&mut self, level: LogLevel) -> &mut Self {
        self.level = level;
        self
    }
    
    /// Enable debug mode
    pub fn enable_debug(&mut self) -> &mut Self {
        self.debug_mode = true;
        if self.level == LogLevel::Info {
            self.level = LogLevel::Debug;
        }
        self
    }
    
    /// Disable debug mode
    pub fn disable_debug(&mut self) -> &mut Self {
        self.debug_mode = false;
        self
    }
    
    /// Log a message at the specified level
    pub fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Trace => trace!("{}", message),
            LogLevel::Debug => debug!("{}", message),
            LogLevel::Info => info!("{}", message),
            LogLevel::Warn => warn!("{}", message),
            LogLevel::Error => error!("{}", message),
        }
    }
    
    /// Start tracking an operation
    pub fn start_operation(&mut self, operation_type: OperationType) -> usize {
        let mut stats = OperationStats::new(operation_type);
        stats.start();
        
        self.history.push(stats);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        
        self.history.len() - 1
    }
    
    /// Complete an operation
    pub fn complete_operation(&mut self, index: usize) -> Result<()> {
        if let Some(stats) = self.history.get_mut(index) {
            stats.complete();
            Ok(())
        } else {
            Err(ZkError::UnexpectedError(format!(
                "Operation index out of bounds: {}",
                index
            )))
        }
    }
    
    /// Mark an operation as failed
    pub fn fail_operation(&mut self, index: usize, error: &ZkError) -> Result<()> {
        if let Some(stats) = self.history.get_mut(index) {
            stats.fail(error);
            Ok(())
        } else {
            Err(ZkError::UnexpectedError(format!(
                "Operation index out of bounds: {}",
                index
            )))
        }
    }
    
    /// Update operation statistics
    pub fn update_stats(
        &mut self,
        index: usize,
        memory_usage: Option<usize>,
        constraint_count: Option<usize>,
    ) -> Result<()> {
        if let Some(stats) = self.history.get_mut(index) {
            if let Some(memory) = memory_usage {
                stats.memory_usage = Some(memory);
            }
            
            if let Some(constraints) = constraint_count {
                stats.constraint_count = Some(constraints);
            }
            
            Ok(())
        } else {
            Err(ZkError::UnexpectedError(format!(
                "Operation index out of bounds: {}",
                index
            )))
        }
    }
    
    /// Get operation statistics by index
    pub fn get_stats(&self, index: usize) -> Option<&OperationStats> {
        self.history.get(index)
    }
    
    /// Get the most recent operation statistics
    pub fn last_stats(&self) -> Option<&OperationStats> {
        self.history.last()
    }
    
    /// Clear operation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// Generate a report of all operations
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("==== ZK Operations Report ====\n");
        
        for (i, stats) in self.history.iter().enumerate() {
            report.push_str(&format!(
                "{}. {} - Status: {}, Duration: {:?}\n",
                i + 1,
                stats.operation_type,
                stats.status,
                stats.duration.unwrap_or_else(|| Duration::from_secs(0))
            ));
            
            if let Some(error) = &stats.error_message {
                report.push_str(&format!("   Error: {}\n", error));
            }
            
            if let Some(constraints) = stats.constraint_count {
                report.push_str(&format!("   Constraints: {}\n", constraints));
            }
            
            if let Some(memory) = stats.memory_usage {
                report.push_str(&format!("   Memory: {} bytes\n", memory));
            }
        }
        
        report.push_str("============================\n");
        report
    }
}

/// Helper struct for timing operations
pub struct OperationTimer {
    /// Operation type
    pub operation_type: OperationType,
    /// Operation index in the logger
    pub index: usize,
    /// Logger instance
    pub logger: ZkLogger,
}

impl OperationTimer {
    /// Create a new operation timer
    pub fn new(operation_type: OperationType, logger: ZkLogger) -> Self {
        let index = logger.start_operation(operation_type);
        OperationTimer {
            operation_type,
            index,
            logger,
        }
    }
    
    /// Update operation statistics
    pub fn update_stats(
        &mut self,
        memory_usage: Option<usize>,
        constraint_count: Option<usize>,
    ) -> Result<()> {
        self.logger.update_stats(self.index, memory_usage, constraint_count)
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        // Mark the operation as completed when the timer is dropped
        let _ = self.logger.complete_operation(self.index);
    }
} 