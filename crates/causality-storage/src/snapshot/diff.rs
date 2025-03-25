// Snapshot diffing utilities
// Original file: src/snapshot/diff.rs

// State comparison for Causality Content-Addressed Code System
//
// This module provides functionality for comparing execution state at
// different points in time, enabling powerful debugging capabilities.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::fmt;

use causality_engine::{ExecutionContext, Value, ExecutionEvent};
use causality_storage::navigator::{TimeTravel, DebugError};
use causality_storage::inspector::{StateInspector, VariableState, PositionInfo};

/// Represents a change in a variable's value
#[derive(Debug, Clone)]
pub struct VariableChange {
    /// Name of the variable
    pub name: String,
    /// Value before the change (None if the variable was created)
    pub old_value: Option<Value>,
    /// Value after the change (None if the variable was deleted)
    pub new_value: Option<Value>,
    /// Type before the change
    pub old_type: Option<String>,
    /// Type after the change
    pub new_type: Option<String>,
}

impl VariableChange {
    /// Check if this is a new variable (didn't exist before)
    pub fn is_new(&self) -> bool {
        self.old_value.is_none() && self.new_value.is_some()
    }
    
    /// Check if this variable was deleted
    pub fn is_deleted(&self) -> bool {
        self.old_value.is_some() && self.new_value.is_none()
    }
    
    /// Check if the value changed (but variable existed before and after)
    pub fn is_modified(&self) -> bool {
        self.old_value.is_some() && self.new_value.is_some() && self.old_value != self.new_value
    }
    
    /// Check if the type changed
    pub fn type_changed(&self) -> bool {
        self.old_type.is_some() && self.new_type.is_some() && self.old_type != self.new_type
    }
}

impl fmt::Display for VariableChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_new() {
            write!(f, "CREATED {}: {:?} ({})", 
                   self.name, 
                   self.new_value.as_ref().unwrap(),
                   self.new_type.as_deref().unwrap_or("unknown"))
        } else if self.is_deleted() {
            write!(f, "DELETED {}: was {:?} ({})", 
                   self.name, 
                   self.old_value.as_ref().unwrap(),
                   self.old_type.as_deref().unwrap_or("unknown"))
        } else if self.is_modified() {
            if self.type_changed() {
                write!(f, "CHANGED {}: {:?} ({}) → {:?} ({})", 
                       self.name, 
                       self.old_value.as_ref().unwrap(),
                       self.old_type.as_deref().unwrap_or("unknown"),
                       self.new_value.as_ref().unwrap(),
                       self.new_type.as_deref().unwrap_or("unknown"))
            } else {
                write!(f, "CHANGED {}: {:?} → {:?}", 
                       self.name, 
                       self.old_value.as_ref().unwrap(),
                       self.new_value.as_ref().unwrap())
            }
        } else {
            write!(f, "UNCHANGED {}", self.name)
        }
    }
}

/// Represents a comparison between two execution states
#[derive(Debug, Clone)]
pub struct StateDiff {
    /// Source position information
    pub source_position: PositionInfo,
    /// Target position information
    pub target_position: PositionInfo,
    /// Variable changes
    pub variable_changes: Vec<VariableChange>,
    /// Events between the two positions
    pub events_between: Vec<ExecutionEvent>,
}

impl StateDiff {
    /// Get only variables that were created
    pub fn created_variables(&self) -> Vec<&VariableChange> {
        self.variable_changes.iter()
            .filter(|change| change.is_new())
            .collect()
    }
    
    /// Get only variables that were deleted
    pub fn deleted_variables(&self) -> Vec<&VariableChange> {
        self.variable_changes.iter()
            .filter(|change| change.is_deleted())
            .collect()
    }
    
    /// Get only variables that were modified
    pub fn modified_variables(&self) -> Vec<&VariableChange> {
        self.variable_changes.iter()
            .filter(|change| change.is_modified())
            .collect()
    }
    
    /// Count how many changes of each type
    pub fn change_counts(&self) -> (usize, usize, usize) {
        let created = self.created_variables().len();
        let deleted = self.deleted_variables().len();
        let modified = self.modified_variables().len();
        (created, deleted, modified)
    }
}

impl fmt::Display for StateDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (created, deleted, modified) = self.change_counts();
        writeln!(f, "State diff between positions {} and {}", 
               self.source_position.position, 
               self.target_position.position)?;
        
        writeln!(f, "Summary: {} created, {} deleted, {} modified variables", 
               created, deleted, modified)?;
        
        writeln!(f, "\nCreated variables:")?;
        for change in self.created_variables() {
            writeln!(f, "  {}", change)?;
        }
        
        writeln!(f, "\nDeleted variables:")?;
        for change in self.deleted_variables() {
            writeln!(f, "  {}", change)?;
        }
        
        writeln!(f, "\nModified variables:")?;
        for change in self.modified_variables() {
            writeln!(f, "  {}", change)?;
        }
        
        writeln!(f, "\nEvents between positions ({} events):", self.events_between.len())?;
        for (i, event) in self.events_between.iter().enumerate() {
            writeln!(f, "  {}. {:?}", i + 1, event)?;
        }
        
        Ok(())
    }
}

/// Interface for state comparison functionality
pub trait StateDiffer: Send + Sync {
    /// Compare state between two positions
    fn compare_positions(
        &self,
        context: &ExecutionContext,
        position1: usize,
        position2: usize,
    ) -> std::result::Result<StateDiff, DebugError>;
    
    /// Compare state before and after a specific event
    fn compare_around_event(
        &self,
        context: &ExecutionContext,
        position: usize,
    ) -> std::result::Result<StateDiff, DebugError>;
    
    /// Compare current state with a previous position
    fn compare_with_current(
        &self,
        context: &ExecutionContext,
        position: usize,
    ) -> std::result::Result<StateDiff, DebugError>;
    
    /// Find the position where a specific variable changed
    fn find_variable_change(
        &self,
        context: &ExecutionContext,
        variable_name: &str,
        start_position: usize,
        end_position: usize,
    ) -> std::result::Result<Option<usize>, DebugError>;
}

/// Implementation of state comparison functionality
pub struct StateComparer {
    /// Time-travel navigation
    time_travel: Arc<dyn TimeTravel>,
    /// State inspector
    inspector: Arc<dyn StateInspector>,
}

impl StateComparer {
    /// Create a new state comparer
    pub fn new(
        time_travel: Arc<dyn TimeTravel>,
        inspector: Arc<dyn StateInspector>,
    ) -> Self {
        StateComparer {
            time_travel,
            inspector,
        }
    }
    
    /// Extract value changes between two state maps
    fn extract_changes(
        &self,
        before_state: Vec<VariableState>,
        after_state: Vec<VariableState>,
    ) -> Vec<VariableChange> {
        // Create maps for easier lookup
        let before_map: HashMap<String, VariableState> = before_state.into_iter()
            .map(|vs| (vs.name.clone(), vs))
            .collect();
        
        let after_map: HashMap<String, VariableState> = after_state.into_iter()
            .map(|vs| (vs.name.clone(), vs))
            .collect();
        
        // Get all variable names from both states
        let mut all_names = HashSet::new();
        all_names.extend(before_map.keys().cloned());
        all_names.extend(after_map.keys().cloned());
        
        // Create changes list
        let mut changes = Vec::new();
        for name in all_names {
            let before = before_map.get(&name);
            let after = after_map.get(&name);
            
            // Skip if value is the same
            if before.is_some() && after.is_some() {
                let before_val = &before.unwrap().value;
                let after_val = &after.unwrap().value;
                
                if before_val == after_val {
                    continue;
                }
            }
            
            // Create the change record
            let change = VariableChange {
                name,
                old_value: before.map(|vs| vs.value.clone()),
                new_value: after.map(|vs| vs.value.clone()),
                old_type: before.and_then(|vs| vs.var_type.clone()),
                new_type: after.and_then(|vs| vs.var_type.clone()),
            };
            
            changes.push(change);
        }
        
        // Sort changes by name for consistency
        changes.sort_by(|a, b| a.name.cmp(&b.name));
        
        changes
    }
    
    /// Get events between two positions
    fn get_events_between(
        &self,
        context: &ExecutionContext,
        start: usize,
        end: usize,
    ) -> std::result::Result<Vec<ExecutionEvent>, DebugError> {
        let events = context.execution_trace()
            .map_err(|e| DebugError::InspectionError(format!("Failed to get execution trace: {:?}", e)))?;
        
        let start_idx = start.min(events.len());
        let end_idx = end.min(events.len());
        
        if start_idx >= end_idx {
            return Ok(Vec::new());
        }
        
        let slice = &events[start_idx..end_idx];
        Ok(slice.to_vec())
    }
}

impl StateDiffer for StateComparer {
    fn compare_positions(
        &self,
        context: &ExecutionContext,
        position1: usize,
        position2: usize,
    ) -> std::result::Result<StateDiff, DebugError> {
        // Clone the context so we can navigate without affecting the original
        let mut context_clone = context.clone();
        
        // Get position info
        let position1_info = {
            self.time_travel.jump_to_position(&mut context_clone, position1)?;
            self.inspector.get_position_info(&context_clone)?
        };
        
        // Get state at position1
        let state1 = self.inspector.get_all_variables(&context_clone)?;
        
        // Get position info and state at position2
        let position2_info = {
            self.time_travel.jump_to_position(&mut context_clone, position2)?;
            self.inspector.get_position_info(&context_clone)?
        };
        
        let state2 = self.inspector.get_all_variables(&context_clone)?;
        
        // Extract changes
        let changes = self.extract_changes(state1, state2);
        
        // Get events between positions
        let events = if position1 < position2 {
            self.get_events_between(context, position1, position2)?
        } else {
            self.get_events_between(context, position2, position1)?
        };
        
        Ok(StateDiff {
            source_position: position1_info,
            target_position: position2_info,
            variable_changes: changes,
            events_between: events,
        })
    }
    
    fn compare_around_event(
        &self,
        context: &ExecutionContext,
        position: usize,
    ) -> std::result::Result<StateDiff, DebugError> {
        // Compare the state just before and just after the event
        self.compare_positions(context, position.saturating_sub(1), position)
    }
    
    fn compare_with_current(
        &self,
        context: &ExecutionContext,
        position: usize,
    ) -> std::result::Result<StateDiff, DebugError> {
        // Get the current position
        let current_position = context.execution_position()
            .map_err(|e| DebugError::InspectionError(format!("Failed to get execution position: {:?}", e)))?;
        
        // Compare the given position with the current position
        self.compare_positions(context, position, current_position)
    }
    
    fn find_variable_change(
        &self,
        context: &ExecutionContext,
        variable_name: &str,
        start_position: usize,
        end_position: usize,
    ) -> std::result::Result<Option<usize>, DebugError> {
        // Clone the context so we can navigate without affecting the original
        let mut context_clone = context.clone();
        
        // Get the starting state
        self.time_travel.jump_to_position(&mut context_clone, start_position)?;
        let mut prev_var = self.inspector.get_variable(&context_clone, variable_name)?;
        
        // Binary search would be more efficient, but we'll use linear search for clarity
        for pos in (start_position + 1)..=end_position {
            // Move to the next position
            self.time_travel.jump_to_position(&mut context_clone, pos)?;
            
            // Get the variable at this position
            let curr_var = self.inspector.get_variable(&context_clone, variable_name)?;
            
            // Check if the variable changed
            let changed = match (&prev_var, &curr_var) {
                (None, Some(_)) => true,                        // Created
                (Some(_), None) => true,                        // Deleted
                (Some(prev), Some(curr)) => prev.value != curr.value,  // Modified
                _ => false,
            };
            
            if changed {
                return Ok(Some(pos));
            }
            
            prev_var = curr_var;
        }
        
        // No change found
        Ok(None)
    }
}

// Add tests for the StateDiffer implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    // This would be a real test in the actual implementation
    // For now, we'll just have a placeholder
    #[test]
    fn test_compare_positions() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_compare_around_event() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_compare_with_current() {
        // Test implementation would go here
    }
    
    #[test]
    fn test_find_variable_change() {
        // Test implementation would go here
    }
} 