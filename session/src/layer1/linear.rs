// Linear type checking - ensuring each resource is used exactly once

use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use crate::layer1::types::Type;
use serde::{Serialize, Deserialize};

/// Variable identifier - with ordering for deterministic iteration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Variable(pub String);

/// Linear typing context - tracks which variables have been used
#[derive(Debug, Clone)]
pub struct LinearContext {
    /// Variable bindings (variable -> type) - deterministic ordering
    bindings: BTreeMap<Variable, Type>,
    
    /// Variables that have been used - deterministic ordering
    used: BTreeSet<Variable>,
}

/// Linear typing errors
#[derive(Error, Debug)]
pub enum LinearityError {
    #[error("Variable {0:?} not found in context")]
    VariableNotFound(Variable),
    
    #[error("Variable {0:?} used more than once")]
    VariableUsedTwice(Variable),
    
    #[error("Variable {0:?} not used")]
    VariableNotUsed(Variable),
    
    #[error("Cannot split context: variable {0:?} appears in both branches")]
    ContextSplitConflict(Variable),
    
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: Type, got: Type },
}

impl Default for LinearContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearContext {
    /// Create a new empty context
    pub fn new() -> Self {
        LinearContext {
            bindings: BTreeMap::new(),
            used: BTreeSet::new(),
        }
    }
    
    /// Add a new variable binding
    pub fn bind(&mut self, var: Variable, ty: Type) -> Result<(), LinearityError> {
        if self.bindings.contains_key(&var) {
            return Err(LinearityError::VariableUsedTwice(var));
        }
        self.bindings.insert(var, ty);
        Ok(())
    }
    
    /// Use a variable (mark it as consumed)
    pub fn use_var(&mut self, var: &Variable) -> Result<Type, LinearityError> {
        // Check variable exists
        let ty = self.bindings.get(var)
            .ok_or_else(|| LinearityError::VariableNotFound(var.clone()))?
            .clone();
        
        // Check not already used
        if self.used.contains(var) {
            return Err(LinearityError::VariableUsedTwice(var.clone()));
        }
        
        // Mark as used
        self.used.insert(var.clone());
        Ok(ty)
    }
    
    /// Check if a variable has been used
    pub fn is_used(&self, var: &Variable) -> bool {
        self.used.contains(var)
    }
    
    /// Get the type of a variable without using it (for checking only)
    pub fn get_type(&self, var: &Variable) -> Result<&Type, LinearityError> {
        self.bindings.get(var)
            .ok_or_else(|| LinearityError::VariableNotFound(var.clone()))
    }
    
    /// Split context for parallel composition
    /// Each variable can only appear in one branch
    pub fn split(self) -> Result<(LinearContext, LinearContext), LinearityError> {
        let mut ctx1 = LinearContext::new();
        let ctx2 = LinearContext::new();
        
        // For simplicity, we'll put all variables in the first context
        // In a real implementation, we'd analyze which variables are used where
        for (var, ty) in self.bindings {
            if !self.used.contains(&var) {
                ctx1.bindings.insert(var, ty);
            }
        }
        
        Ok((ctx1, ctx2))
    }
    
    /// Merge two contexts after parallel composition
    /// All variables in both contexts must have been used
    pub fn merge(ctx1: LinearContext, ctx2: LinearContext) -> Result<LinearContext, LinearityError> {
        let mut result = LinearContext::new();
        
        // Check all variables in ctx1 were used
        for var in ctx1.bindings.keys() {
            if !ctx1.used.contains(var) {
                return Err(LinearityError::VariableNotUsed(var.clone()));
            }
        }
        
        // Check all variables in ctx2 were used
        for var in ctx2.bindings.keys() {
            if !ctx2.used.contains(var) {
                return Err(LinearityError::VariableNotUsed(var.clone()));
            }
        }
        
        // Merge used variables
        result.used.extend(ctx1.used);
        result.used.extend(ctx2.used);
        
        Ok(result)
    }
    
    /// Check that all linear variables have been used
    pub fn check_all_used(&self) -> Result<(), LinearityError> {
        for (var, ty) in &self.bindings {
            // Only check linear types (not unrestricted ones)
            if self.is_linear_type(ty) && !self.used.contains(var) {
                return Err(LinearityError::VariableNotUsed(var.clone()));
            }
        }
        Ok(())
    }
    
    /// Check if a type requires linear usage
    fn is_linear_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Record(_) => true,   // Records (messages) are always linear
            Type::Session(_) => true,  // Sessions are always linear
            _ => false,  // Other types can be used freely
        }
    }
}

/// Split a context based on variable usage analysis
pub fn split_context_by_usage(
    ctx: LinearContext,
    vars1: BTreeSet<Variable>,
    vars2: BTreeSet<Variable>,
) -> Result<(LinearContext, LinearContext), LinearityError> {
    // Check for conflicts
    for var in &vars1 {
        if vars2.contains(var) {
            return Err(LinearityError::ContextSplitConflict(var.clone()));
        }
    }
    
    let mut ctx1 = LinearContext::new();
    let mut ctx2 = LinearContext::new();
    
    for (var, ty) in ctx.bindings {
        if ctx.used.contains(&var) {
            // Already used variables go nowhere
            continue;
        }
        
        if vars1.contains(&var) {
            ctx1.bindings.insert(var.clone(), ty);
        } else if vars2.contains(&var) {
            ctx2.bindings.insert(var.clone(), ty);
        } else {
            // Variables not used in either branch - this is an error
            return Err(LinearityError::VariableNotUsed(var));
        }
    }
    
    Ok((ctx1, ctx2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::types::RowType;
    
    #[test]
    fn test_linear_usage() {
        let mut ctx = LinearContext::new();
        let var = Variable("x".to_string());
        
        // Bind variable with a record type
        let record_type = Type::Record(RowType::from_fields(vec![
            ("value".to_string(), Type::Int)
        ]));
        ctx.bind(var.clone(), record_type.clone()).unwrap();
        
        // Use it once - should succeed
        let ty = ctx.use_var(&var).unwrap();
        assert!(matches!(ty, Type::Record(_)));
        
        // Use it again - should fail
        assert!(matches!(
            ctx.use_var(&var),
            Err(LinearityError::VariableUsedTwice(_))
        ));
    }
    
    #[test]
    fn test_linearity_check() {
        let mut ctx = LinearContext::new();
        
        // Linear variable (record/message)
        let linear_var = Variable("msg".to_string());
        let record_type = Type::Record(RowType::from_fields(vec![
            ("data".to_string(), Type::Int)
        ]));
        ctx.bind(linear_var.clone(), record_type).unwrap();
        
        // Non-linear variable (int)
        let nonlinear_var = Variable("n".to_string());
        ctx.bind(nonlinear_var, Type::Int).unwrap();
        
        // Check without using linear variable - should fail
        assert!(matches!(
            ctx.check_all_used(),
            Err(LinearityError::VariableNotUsed(_))
        ));
        
        // Use linear variable
        ctx.use_var(&linear_var).unwrap();
        
        // Now check should pass (non-linear var doesn't need to be used)
        ctx.check_all_used().unwrap();
    }
    
    #[test]
    fn test_context_split() {
        let ctx = LinearContext::new();
        let vars1 = BTreeSet::from([Variable("x".to_string())]);
        let vars2 = BTreeSet::from([Variable("y".to_string())]);
        
        // Non-overlapping split should succeed
        let (_, _) = split_context_by_usage(ctx, vars1.clone(), vars2).unwrap();
        
        // Overlapping split should fail
        let ctx2 = LinearContext::new();
        let result = split_context_by_usage(ctx2, vars1.clone(), vars1);
        assert!(matches!(result, Err(LinearityError::ContextSplitConflict(_))));
    }
}
