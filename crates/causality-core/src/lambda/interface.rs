//! Layer 1 interface to Layer 0
//!
//! This module defines how Layer 1 terms compile down to Layer 0 instructions.

use super::{Term, TermKind, Literal};
use crate::machine::{Instruction, RegisterId, LiteralValue};

//-----------------------------------------------------------------------------
// Compilation Context
//-----------------------------------------------------------------------------

/// Context for compiling Layer 1 terms to Layer 0 instructions
pub struct CompilationContext {
    /// Next available register
    next_register: u32,
    
    /// Variable to register mapping
    var_registers: std::collections::HashMap<String, RegisterId>,
    
    /// Generated instructions
    instructions: Vec<Instruction>,
}

impl CompilationContext {
    /// Create a new compilation context
    pub fn new() -> Self {
        Self {
            next_register: 0,
            var_registers: std::collections::HashMap::new(),
            instructions: Vec::new(),
        }
    }
    
    /// Allocate a fresh register
    pub fn fresh_register(&mut self) -> RegisterId {
        let reg = RegisterId(self.next_register);
        self.next_register += 1;
        reg
    }
    
    /// Get register for a variable
    pub fn get_var_register(&self, var: &str) -> Option<RegisterId> {
        self.var_registers.get(var).copied()
    }
    
    /// Bind a variable to a register
    pub fn bind_var(&mut self, var: String, reg: RegisterId) {
        self.var_registers.insert(var, reg);
    }
    
    /// Emit an instruction
    pub fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }
    
    /// Get the generated instructions
    pub fn instructions(self) -> Vec<Instruction> {
        self.instructions
    }
}

//-----------------------------------------------------------------------------
// Term Compilation
//-----------------------------------------------------------------------------

/// Compile a Layer 1 term to Layer 0 instructions
pub fn compile_term(term: &Term) -> Result<Vec<Instruction>, CompileError> {
    let mut ctx = CompilationContext::new();
    let _result_reg = compile_term_to_register(term, &mut ctx)?;
    Ok(ctx.instructions())
}

/// Compile a term and put result in a register
fn compile_term_to_register(
    term: &Term,
    ctx: &mut CompilationContext,
) -> Result<RegisterId, CompileError> {
    match &term.kind {
        // Variables
        TermKind::Var(name) => {
            ctx.get_var_register(name)
                .ok_or_else(|| CompileError::UnboundVariable(name.clone()))
        }
        
        // Literals - need to store in a register first
        TermKind::Literal(lit) => {
            let reg = ctx.fresh_register();
            let value_reg = ctx.fresh_register();
            
            // First store the literal in a register
            let _value = match lit {
                Literal::Bool(b) => LiteralValue::Bool(*b),
                Literal::Int(i) => LiteralValue::Int(*i),
                Literal::Symbol(s) => LiteralValue::Symbol(s.clone()),
            };
            
            // For now, we use a hack: store literal in register via witness
            // In a real implementation, we'd have a proper literal loading instruction
            ctx.emit(Instruction::Witness { out_reg: value_reg });
            
            // Then move it to the destination
            ctx.emit(Instruction::Move {
                src: value_reg,
                dst: reg,
            });
            
            Ok(reg)
        }
        
        // Unit
        TermKind::Unit => {
            let reg = ctx.fresh_register();
            // Similar hack for unit value
            ctx.emit(Instruction::Witness { out_reg: reg });
            Ok(reg)
        }
        
        // Function application
        TermKind::Apply { func, arg } => {
            let func_reg = compile_term_to_register(func, ctx)?;
            let arg_reg = compile_term_to_register(arg, ctx)?;
            let result_reg = ctx.fresh_register();
            
            ctx.emit(Instruction::Apply {
                fn_reg: func_reg,
                arg_reg: arg_reg,
                out_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Resource allocation
        TermKind::Alloc { value } => {
            let value_reg = compile_term_to_register(value, ctx)?;
            let type_reg = ctx.fresh_register(); // Would need actual type
            let result_reg = ctx.fresh_register();
            
            ctx.emit(Instruction::Alloc {
                type_reg: type_reg,
                val_reg: value_reg,
                out_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Resource consumption
        TermKind::Consume { resource } => {
            let resource_reg = compile_term_to_register(resource, ctx)?;
            let result_reg = ctx.fresh_register();
            
            ctx.emit(Instruction::Consume {
                resource_reg: resource_reg,
                out_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // TODO: Implement compilation for other term kinds
        _ => Err(CompileError::NotImplemented(format!("{:?}", term.kind))),
    }
}

//-----------------------------------------------------------------------------
// Compilation Errors
//-----------------------------------------------------------------------------

/// Errors that can occur during compilation
#[derive(Debug, Clone)]
pub enum CompileError {
    /// Variable not bound in context
    UnboundVariable(String),
    
    /// Feature not yet implemented
    NotImplemented(String),
    
    /// Type error
    TypeError(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnboundVariable(var) => write!(f, "Unbound variable: {}", var),
            CompileError::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
            CompileError::TypeError(msg) => write!(f, "Type error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {} 