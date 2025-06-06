//! Effects and constraints
//!
//! This module defines effects to be performed and constraints to be checked
//! during register machine execution.

use super::instruction::{RegisterId, Instruction, EffectCall, ConstraintExpr};

/// Effect to be performed
#[derive(Debug, Clone)]
pub struct Effect {
    /// Effect call information
    pub call: EffectCall,
    
    /// Result register (where to store the result)
    pub result_register: Option<RegisterId>,
}

/// Constraint to be checked
#[derive(Debug, Clone)]
pub struct Constraint {
    /// Constraint expression
    pub expr: ConstraintExpr,
    
    /// Instructions to execute on failure
    pub on_failure: Vec<Instruction>,
}

impl Effect {
    /// Create a new effect
    pub fn new(call: EffectCall, result_register: Option<RegisterId>) -> Self {
        Self {
            call,
            result_register,
        }
    }
    
    /// Get the effect tag
    pub fn tag(&self) -> &String {
        &self.call.tag
    }
    
    /// Get the effect arguments
    pub fn args(&self) -> &[RegisterId] {
        &self.call.args
    }
}

impl Constraint {
    /// Create a new constraint
    pub fn new(expr: ConstraintExpr, on_failure: Vec<Instruction>) -> Self {
        Self {
            expr,
            on_failure,
        }
    }
    
    /// Check if the constraint is satisfied
    /// 
    /// Note: This is a placeholder - actual constraint evaluation
    /// will be implemented when we have the full evaluation context
    pub fn is_satisfied(&self) -> bool {
        match &self.expr {
            ConstraintExpr::True => true,
            ConstraintExpr::False => false,
            _ => false // Placeholder - needs evaluation context
        }
    }
} 