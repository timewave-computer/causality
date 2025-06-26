//! Layer 2 interface to Layer 1
//!
//! This module defines how Layer 2 effect expressions compile down to Layer 1 terms.

use super::{EffectExpr, EffectExprKind};
use crate::lambda::{Term, Literal, Symbol};

//-----------------------------------------------------------------------------
// Effect Compilation
//-----------------------------------------------------------------------------

/// Compile a Layer 2 effect expression to a Layer 1 term
pub fn compile_effect(effect: &EffectExpr) -> Result<Term, EffectCompileError> {
    match &effect.kind {
        // Pure effects become simple terms
        EffectExprKind::Pure(term) => Ok(term.clone()),
        
        // Bind becomes let binding in Layer 1
        EffectExprKind::Bind { effect, var, body } => {
            let effect_term = compile_effect(effect)?;
            let body_term = compile_effect(body)?;
            
            Ok(Term::let_bind(var.clone(), effect_term, body_term))
        }
        
        // Perform becomes a function call to effect handler
        EffectExprKind::Perform { effect_tag, args } => {
            // For now, we compile perform as a special symbol application
            // In a full implementation, this would lookup the effect handler
            let handler = Term::var(format!("effect_{}", effect_tag));
            
            // Apply handler to each argument in sequence
            args.iter().try_fold(handler, |acc, arg| {
                Ok(Term::apply(acc, arg.clone()))
            })
        }
        
        // Handle is more complex - would need CPS transformation
        EffectExprKind::Handle { .. } => {
            Err(EffectCompileError::NotImplemented("handle".to_string()))
        }
        
        // Parallel/Race need runtime support
        EffectExprKind::Parallel { .. } => {
            Err(EffectCompileError::NotImplemented("parallel".to_string()))
        }
        
        EffectExprKind::Race { .. } => {
            Err(EffectCompileError::NotImplemented("race".to_string()))
        }
        
        // Session operations - compile to Layer 1 session operations
        EffectExprKind::SessionSend { channel, value, continuation } => {
            let channel_term = compile_effect(channel)?;
            let continuation_term = compile_effect(continuation)?;
            
            // For now, compile to a special send application
            let send_op = Term::apply(
                Term::var("session_send"),
                Term::tensor(channel_term, value.clone())
            );
            
            Ok(Term::let_bind("_send_result".to_string(), send_op, continuation_term))
        }
        
        EffectExprKind::SessionReceive { channel, continuation } => {
            let channel_term = compile_effect(channel)?;
            let continuation_term = compile_effect(continuation)?;
            
            // Compile to a receive operation
            let recv_op = Term::apply(Term::var("session_recv"), channel_term);
            
            Ok(Term::let_bind("received_value".to_string(), recv_op, continuation_term))
        }
        
        EffectExprKind::SessionSelect { channel, choice, continuation } => {
            let channel_term = compile_effect(channel)?;
            let continuation_term = compile_effect(continuation)?;
            
            // Compile to a select operation  
            let choice_term = Term::literal(Literal::Symbol(Symbol::new(choice)));
            let select_op = Term::apply(
                Term::var("session_select"),
                Term::tensor(channel_term, choice_term)
            );
            
            Ok(Term::let_bind("_select_result".to_string(), select_op, continuation_term))
        }
        
        EffectExprKind::SessionCase { .. } => {
            // Session case requires more complex compilation (pattern matching)
            Err(EffectCompileError::NotImplemented("session_case".to_string()))
        }
        
        EffectExprKind::WithSession { .. } => {
            // Session context establishment requires runtime support
            Err(EffectCompileError::NotImplemented("with_session".to_string()))
        }
    }
}

//-----------------------------------------------------------------------------
// Transaction Support
//-----------------------------------------------------------------------------

/// Compile a transaction (list of effects) to Layer 1
pub fn compile_transaction(effects: Vec<EffectExpr>) -> Result<Term, EffectCompileError> {
    // For now, transactions are sequential composition
    // In a full implementation, this would add atomicity guarantees
    
    if effects.is_empty() {
        return Ok(Term::unit());
    }
    
    // Compile each effect and sequence them with let bindings
    let mut result = compile_effect(&effects[0])?;
    
    for (i, effect) in effects.iter().skip(1).enumerate() {
        let effect_term = compile_effect(effect)?;
        result = Term::let_bind(
            format!("_tx_{}", i),
            result,
            effect_term,
        );
    }
    
    Ok(result)
}

//-----------------------------------------------------------------------------
// Compilation Errors
//-----------------------------------------------------------------------------

/// Errors that can occur during effect compilation
#[derive(Debug, Clone)]
pub enum EffectCompileError {
    /// Feature not yet implemented
    NotImplemented(String),
    
    /// Invalid effect structure
    InvalidEffect(String),
    
    /// Type error
    TypeError(String),
}

impl std::fmt::Display for EffectCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectCompileError::NotImplemented(feature) => {
                write!(f, "Not implemented: {}", feature)
            }
            EffectCompileError::InvalidEffect(msg) => {
                write!(f, "Invalid effect: {}", msg)
            }
            EffectCompileError::TypeError(msg) => {
                write!(f, "Type error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EffectCompileError {} 