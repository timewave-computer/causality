//! Simulation Breakpoints
//!
//! Defines structures related to simulation breakpoints for debugging
//! and controlling execution flow.

//-----------------------------------------------------------------------------
// Breakpoint Structures
//-----------------------------------------------------------------------------

use crate::SimulationError;
use causality_types::serialization::{SimpleSerialize};
use serde::{Deserialize, Serialize};
use ethereum_ssz_derive::{Encode, Decode};

/// Information about a breakpoint that was hit during simulation.
/// 
/// Uses derive macros for automatic SSZ serialization instead of manual implementation
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct BreakpointInfo {
    /// The label of the breakpoint, as defined in the TEL effect.
    pub label: String,
    /// The unique ID of the breakpoint instance, as defined in the TEL effect.
    pub id: String,
}

impl SimpleSerialize for BreakpointInfo {}

/// Action to take when a breakpoint is hit
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum BreakpointAction {
    /// Stop execution
    Stop,
    /// Log information and continue
    Log,
    /// Custom action with description
    Custom(String),
}

impl SimpleSerialize for BreakpointAction {}

/// A breakpoint in the simulation
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Breakpoint {
    /// Unique identifier for this breakpoint
    pub id: String,
    
    /// Whether the breakpoint is currently enabled
    pub enabled: bool,
    
    /// Description of what triggers this breakpoint
    pub condition: String,
    
    /// Action to take when the breakpoint is hit
    pub action: BreakpointAction,
}

impl SimpleSerialize for Breakpoint {} 