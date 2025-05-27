//! Simulation History and Execution Trace Management
//!
//! This module provides functionality for tracking and analyzing simulation execution history,
//! including state snapshots, event logging, and trace analysis.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------


use serde::{Serialize, Deserialize};
use causality_types::{
    core::id::EffectId,
    serialization::{Encode, SimpleSerialize},
};
// Removed: use crate::breakpoint::BreakpointInfo;

// Define types locally since engine module is disabled
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakpointInfo {
    pub label: String,
    pub id: String,
}

impl SimpleSerialize for BreakpointInfo {}

impl Encode for BreakpointInfo {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let label_bytes = self.label.as_bytes();
        let id_bytes = self.id.as_bytes();
        
        bytes.extend_from_slice(&(label_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(label_bytes);
        bytes.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(id_bytes);
        
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationOutputInfo {
    pub action: String,
    pub error: Option<String>,
    pub result: Option<String>,
}

impl SimpleSerialize for SimulationOutputInfo {}

impl Encode for SimulationOutputInfo {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let action_bytes = self.action.as_bytes();
        
        bytes.extend_from_slice(&(action_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(action_bytes);
        
        match &self.error {
            Some(error) => {
                bytes.push(1); // Some marker
                let error_bytes = error.as_bytes();
                bytes.extend_from_slice(&(error_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(error_bytes);
            }
            None => {
                bytes.push(0); // None marker
            }
        }
        
        match &self.result {
            Some(result) => {
                bytes.push(1); // Some marker
                let result_bytes = result.as_bytes();
                bytes.extend_from_slice(&(result_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(result_bytes);
            }
            None => {
                bytes.push(0); // None marker
            }
        }
        
        bytes
    }
}

//-----------------------------------------------------------------------------
// Execution Events
//-----------------------------------------------------------------------------

/// Represents discrete events that occur during an effect's execution trace or simulation step.
/// These are more granular than a full SimulationStepOutcome.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectExecutionEvent {
    /// Indicates a specific effect was processed.
    EffectProcessed(EffectId), // ID of the effect processed

    /// A breakpoint was encountered during simulation.
    Breakpoint(BreakpointInfo), // Contains information about the breakpoint

    /// Output or result from a control action or explicit simulation output marker.
    ControlOutput(SimulationOutputInfo),
    // Error(String), // For errors that occur during an effect that don't halt the simulation
    // NoOp, // If a step is recorded but no meaningful simulation event occurred.
}

impl Encode for EffectExecutionEvent {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        // Simple manual implementation
        match self {
            EffectExecutionEvent::EffectProcessed(id) => {
                let mut bytes = vec![0u8]; // variant tag
                bytes.extend(id.as_ssz_bytes());
                bytes
            }
            EffectExecutionEvent::Breakpoint(info) => {
                let mut bytes = vec![1u8]; // variant tag
                bytes.extend(info.as_ssz_bytes());
                bytes
            }
            EffectExecutionEvent::ControlOutput(info) => {
                let mut bytes = vec![2u8]; // variant tag
                bytes.extend(info.as_ssz_bytes());
                bytes
            }
        }
    }
}

impl SimpleSerialize for EffectExecutionEvent {}

//-----------------------------------------------------------------------------
// Simulation Snapshots
//-----------------------------------------------------------------------------

/// Snapshot of the simulation state at a specific step.
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationSnapshot {
    pub step_number: u64,
    // pub context: TelContext, // TODO: This needs to be serializable or represented by serializable state.
    pub event: Option<EffectExecutionEvent>, // Event that led to this state or occurred in this step.
                                             // pub other_simulation_specific_state: ... // e.g., RNG state
}

impl Encode for SimulationSnapshot {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.step_number.to_le_bytes());
        match &self.event {
            Some(event) => {
                bytes.push(1); // Some marker
                bytes.extend(event.as_ssz_bytes());
            }
            None => {
                bytes.push(0); // None marker
            }
        }
        bytes
    }
}

impl SimpleSerialize for SimulationSnapshot {}

impl SimulationSnapshot {
    /// Creates a new snapshot.
    pub fn new(
        step_number: u64,
        // context: TelContext, // TODO: Restore when serializable
        event: Option<EffectExecutionEvent>,
    ) -> Self {
        Self {
            step_number,
            // context, // TODO: Restore
            event,
        }
    }
}

//-----------------------------------------------------------------------------
// History Management
//-----------------------------------------------------------------------------

/// Manages the history of simulation snapshots.
#[derive(Debug, Default, Clone)]
pub struct SimulationHistory {
    pub snapshots: Vec<SimulationSnapshot>,
    current_step_index: usize, // Points to the current snapshot in the `snapshots` vector
}

impl Encode for SimulationHistory {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend((self.snapshots.len() as u32).to_le_bytes());
        for snapshot in &self.snapshots {
            bytes.extend(snapshot.as_ssz_bytes());
        }
        bytes.extend((self.current_step_index as u32).to_le_bytes());
        bytes
    }
}

impl SimpleSerialize for SimulationHistory {}

impl SimulationHistory {
    /// Creates a new history with an initial snapshot.
    pub fn new(initial_snapshot: SimulationSnapshot) -> Self {
        Self {
            snapshots: vec![initial_snapshot],
            current_step_index: 0,
        }
    }

    /// Creates a new empty history.
    pub fn new_empty() -> Self {
        Self {
            snapshots: Vec::new(),
            current_step_index: 0,
        }
    }

    /// Records a new step in the history.
    pub fn record_step(&mut self, snapshot: SimulationSnapshot) {
        self.record_snapshot(snapshot);
    }

    /// Sets the current step index.
    pub fn set_current_step_index(&mut self, index: usize) {
        if index < self.snapshots.len() {
            self.current_step_index = index;
        }
    }

    /// Gets the current step index.
    pub fn get_current_step_index(&self) -> usize {
        self.current_step_index
    }

    /// Adds a new snapshot to the history. If the current step is not the last one (due to time travel),
    /// future snapshots are truncated.
    pub fn record_snapshot(&mut self, snapshot_to_add: SimulationSnapshot) {
        // Ensure step numbers are consistent (or handle out-of-order if allowed)
        if let Some(last_snapshot) = self.snapshots.last() {
            if snapshot_to_add.step_number <= last_snapshot.step_number
                && !self.snapshots.is_empty()
            {
                // This might indicate an issue or a specific time travel scenario not handled by simple append.
                // For now, let's assume snapshots are added in increasing step order or after a jump.
                // If we jumped back and are now moving forward, truncate future states.
                if self.current_step_index < self.snapshots.len() - 1 {
                    self.snapshots.truncate(self.current_step_index + 1);
                }
            }
        }
        // If current_step_index is not at the end of snapshots (i.e., we've jumped back in time)
        // and are now recording a new future, truncate the old future.
        if self.current_step_index < self.snapshots.len() - 1 {
            self.snapshots.truncate(self.current_step_index + 1);
        }

        self.snapshots.push(snapshot_to_add);
        self.current_step_index = self.snapshots.len() - 1;
    }

    /// Gets the current step number.
    pub fn get_current_step_number(&self) -> u64 {
        self.snapshots
            .get(self.current_step_index)
            .map_or(0, |s| s.step_number)
    }

    /// Gets the snapshot for the current step.
    pub fn get_current_snapshot(&self) -> Option<&SimulationSnapshot> {
        self.snapshots.get(self.current_step_index)
    }

    /// Jumps to a specific step number in history.
    /// Returns the snapshot for that step if it exists.
    pub fn jump_to_step(&mut self, step_number: u64) -> Option<&SimulationSnapshot> {
        if let Some(index) = self
            .snapshots
            .iter()
            .position(|s| s.step_number == step_number)
        {
            self.current_step_index = index;
            self.snapshots.get(index)
        } else {
            None
        }
    }

    /// Retrieves a snapshot by its step number.
    pub fn get_snapshot_data(
        &self,
        step_number: u64,
    ) -> Option<&SimulationSnapshot> {
        self.snapshots.iter().find(|s| s.step_number == step_number)
    }

    /// Gets the total number of snapshots (steps) recorded.
    pub fn total_steps(&self) -> usize {
        self.snapshots.len()
    }
}

/// Represents a simulation event that can be recorded. (Different from EffectExecutionEvent)
// This might be higher level, encompassing step outcomes.
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationEvent {
    StepCompleted {
        step_number: u64,
        outcome: String, // Or a more structured outcome type
        duration_ms: u128,
    },
    BreakpointHit {
        step_number: u64,
        breakpoint_info: BreakpointInfo,
    },
    SimulationEnded {
        total_steps: u64,
        reason: String,
    },
    MockRegistered {
        effect_type: String,
        // behavior_description: String, // Could add details about the mock behavior
    },
    ErrorOccurred(String), // General simulation error
}

impl Encode for SimulationEvent {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        // Simple manual implementation for each variant
        match self {
            SimulationEvent::StepCompleted { step_number, outcome, duration_ms } => {
                let mut bytes = vec![0u8]; // variant tag
                bytes.extend(step_number.to_le_bytes());
                bytes.extend(outcome.as_ssz_bytes());
                bytes.extend(duration_ms.to_le_bytes());
                bytes
            }
            SimulationEvent::BreakpointHit { step_number, breakpoint_info } => {
                let mut bytes = vec![1u8]; // variant tag
                bytes.extend(step_number.to_le_bytes());
                bytes.extend(breakpoint_info.as_ssz_bytes());
                bytes
            }
            SimulationEvent::SimulationEnded { total_steps, reason } => {
                let mut bytes = vec![2u8]; // variant tag
                bytes.extend(total_steps.to_le_bytes());
                bytes.extend(reason.as_ssz_bytes());
                bytes
            }
            SimulationEvent::MockRegistered { effect_type } => {
                let mut bytes = vec![3u8]; // variant tag
                bytes.extend(effect_type.as_ssz_bytes());
                bytes
            }
            SimulationEvent::ErrorOccurred(error) => {
                let mut bytes = vec![4u8]; // variant tag
                bytes.extend(error.as_ssz_bytes());
                bytes
            }
        }
    }
}

impl SimpleSerialize for SimulationEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_record_and_get() {
        // Create initial snapshot
        let snapshot = SimulationSnapshot::new(0, None);
        let mut history = SimulationHistory::new(snapshot);

        // Check initial state
        assert_eq!(history.total_steps(), 1);
        assert_eq!(history.get_current_step_number(), 0);
        assert_eq!(history.get_snapshot_data(0).unwrap().step_number, 0);

        // Add a second snapshot
        let snapshot2 = SimulationSnapshot::new(1, None);
        history.record_snapshot(snapshot2);

        assert_eq!(history.total_steps(), 2);
        assert_eq!(history.get_current_step_number(), 1);
        assert_eq!(history.get_current_snapshot().unwrap().step_number, 1);
    }

    #[test]
    fn test_jump_to_step() {
        let mut history = SimulationHistory::new(SimulationSnapshot::new(0, None));

        // Add more snapshots
        history.record_snapshot(SimulationSnapshot::new(1, None));
        history.record_snapshot(SimulationSnapshot::new(2, None));

        // Jump back to step 0
        let result = history.jump_to_step(0);
        assert!(result.is_some());
        assert_eq!(history.get_current_step_number(), 0);

        // Jump to step 2
        let result = history.jump_to_step(2);
        assert!(result.is_some());
        assert_eq!(history.get_current_step_number(), 2);

        // Jump to nonexistent step
        let result = history.jump_to_step(3);
        assert!(result.is_none());
    }

    #[test]
    fn test_history_truncation() {
        let mut history = SimulationHistory::new(SimulationSnapshot::new(0, None));

        // Add two more snapshots
        history.record_snapshot(SimulationSnapshot::new(1, None));
        history.record_snapshot(SimulationSnapshot::new(2, None));

        // Jump back to step 1
        history.jump_to_step(1);

        // Add a new snapshot - should truncate the previous step 2
        history.record_snapshot(SimulationSnapshot::new(2, None));

        // Should only have 3 snapshots total
        assert_eq!(history.total_steps(), 3);
        assert_eq!(history.get_current_step_number(), 2);
    }
}
