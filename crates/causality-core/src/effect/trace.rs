//! Execution tracing infrastructure
//!
//! This module provides types and utilities for tracking the execution
//! of effects and computations through the Causality system.

use crate::system::content_addressing::{EntityId, Timestamp};
use serde::{Serialize, Deserialize};

/// Execution trace for tracking effect execution through the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Unique identifier for this trace
    pub id: EntityId,
    
    /// When the execution started
    pub start_time: Timestamp,
    
    /// When the execution completed (if finished)
    pub end_time: Option<Timestamp>,
    
    /// Effects that were executed
    pub effects: Vec<EffectStep>,
    
    /// Resources consumed during execution
    pub resources_consumed: Vec<EntityId>,
    
    /// Resources created during execution  
    pub resources_created: Vec<EntityId>,
    
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Error message if execution failed
    pub error: Option<String>,
}

/// A single step in the execution trace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectStep {
    /// Effect identifier
    pub effect_id: EntityId,
    
    /// When this step started
    pub start_time: Timestamp,
    
    /// When this step completed
    pub end_time: Option<Timestamp>,
    
    /// Step status
    pub status: StepStatus,
    
    /// Input parameters 
    pub inputs: Vec<u8>,
    
    /// Output results
    pub outputs: Option<Vec<u8>>,
    
    /// Error if step failed
    pub error: Option<String>,
}

/// Status of overall execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution is still running
    Running,
    
    /// Execution completed successfully  
    Completed,
    
    /// Execution failed with error
    Failed,
    
    /// Execution was cancelled
    Cancelled,
}

/// Status of individual execution step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] 
pub enum StepStatus {
    /// Step is pending execution
    Pending,
    
    /// Step is currently executing
    Running,
    
    /// Step completed successfully
    Completed,
    
    /// Step failed with error
    Failed,
    
    /// Step was skipped
    Skipped,
}

impl ExecutionTrace {
    /// Create a new execution trace
    pub fn new() -> Self {
        let start_time = Timestamp::now();
        let id = EntityId::from_content(&start_time);
        
        Self {
            id,
            start_time,
            end_time: None,
            effects: Vec::new(),
            resources_consumed: Vec::new(),
            resources_created: Vec::new(),
            status: ExecutionStatus::Running,
            error: None,
        }
    }
    
    /// Add an effect step to the trace
    pub fn add_effect_step(&mut self, effect_id: EntityId) -> &mut EffectStep {
        let step = EffectStep {
            effect_id,
            start_time: Timestamp::now(),
            end_time: None,
            status: StepStatus::Running,
            inputs: Vec::new(),
            outputs: None,
            error: None,
        };
        
        self.effects.push(step);
        self.effects.last_mut().unwrap()
    }
    
    /// Mark execution as completed
    pub fn complete(&mut self) {
        self.status = ExecutionStatus::Completed;
        self.end_time = Some(Timestamp::now());
    }
    
    /// Mark execution as failed
    pub fn fail(&mut self, error: String) {
        self.status = ExecutionStatus::Failed;
        self.end_time = Some(Timestamp::now());
        self.error = Some(error);
    }
    
    /// Get the duration of execution
    pub fn duration(&self) -> Option<u64> {
        self.end_time.map(|end| end.millis - self.start_time.millis)
    }
}

impl Default for ExecutionTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectStep {
    /// Mark the step as completed
    pub fn complete(&mut self, outputs: Vec<u8>) {
        self.status = StepStatus::Completed;
        self.end_time = Some(Timestamp::now());
        self.outputs = Some(outputs);
    }
    
    /// Mark the step as failed
    pub fn fail(&mut self, error: String) {
        self.status = StepStatus::Failed;
        self.end_time = Some(Timestamp::now());
        self.error = Some(error);
    }
} 