//! Effect execution tracking for simulation

use std::time::SystemTime;
use causality_core::lambda::base::Value;

/// Represents the execution of an effect during simulation
#[derive(Debug, Clone)]
pub struct EffectExecution {
    /// Unique identifier for this execution
    pub id: String,
    
    /// The effect that was executed
    pub effect_name: String,
    
    /// Input parameters to the effect
    pub inputs: Vec<Value>,
    
    /// Output result from the effect
    pub output: Option<Value>,
    
    /// Timestamp when execution started
    pub started_at: SystemTime,
    
    /// Timestamp when execution completed
    pub completed_at: Option<SystemTime>,
    
    /// Whether the execution was successful
    pub success: bool,
    
    /// Error message if execution failed
    pub error_message: Option<String>,
    
    /// Gas consumed during execution
    pub gas_consumed: u64,
} 