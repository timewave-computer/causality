//! Layer 1 interface to Layer 0 minimal instruction set
//!
//! This module defines how Layer 1 terms compile down to the minimal 5-operation
//! instruction set based on symmetric monoidal closed category theory.

use super::{Term, TermKind, Literal};
use crate::machine::{Instruction, RegisterId, MachineValue};
use crate::lambda::{TypeInner, BaseType};

//-----------------------------------------------------------------------------
// Compilation Context
//-----------------------------------------------------------------------------

/// Context for compiling Layer 1 terms to minimal instruction set
pub struct CompilationContext {
    /// Next available register
    next_register: u32,
    
    /// Variable to register mapping
    var_registers: std::collections::BTreeMap<String, RegisterId>,
    
    /// Generated instructions
    instructions: Vec<Instruction>,
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationContext {
    /// Create a new compilation context
    pub fn new() -> Self {
        Self {
            next_register: 0,
            var_registers: std::collections::BTreeMap::new(),
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
    
    /// Create a literal value register
    pub fn alloc_literal(&mut self, lit: &Literal) -> RegisterId {
        let type_reg = self.fresh_register();
        let init_reg = self.fresh_register();
        let result_reg = self.fresh_register();
        
        // Store type information
        let type_value = match lit {
            Literal::Bool(_) => MachineValue::Type(TypeInner::Base(BaseType::Bool)),
            Literal::Int(_) => MachineValue::Type(TypeInner::Base(BaseType::Int)),
            Literal::Symbol(_) => MachineValue::Type(TypeInner::Base(BaseType::Symbol)),
            Literal::Unit => MachineValue::Type(TypeInner::Base(BaseType::Unit)),
        };
        
        // Store initial value
        let init_value = match lit {
            Literal::Bool(b) => MachineValue::Bool(*b),
            Literal::Int(i) => MachineValue::Int(*i),
            Literal::Symbol(s) => MachineValue::Symbol(s.clone()),
            Literal::Unit => MachineValue::Unit,
        };
        
        // For now, we'll need to find a way to get these values into registers
        // This is a limitation of the current system - we need a way to bootstrap literals
        // TODO: Add a "Load" instruction or use witness instructions
        
        // Allocate the literal as a resource
        self.emit(Instruction::Alloc {
            type_reg,
            init_reg,
            output_reg: result_reg,
        });
        
        result_reg
    }
}

//-----------------------------------------------------------------------------
// Term Compilation to Minimal Instruction Set
//-----------------------------------------------------------------------------

/// Compile a Layer 1 term to the minimal 5-operation instruction set
pub fn compile_term(term: &Term) -> Result<Vec<Instruction>, CompileError> {
    let mut ctx = CompilationContext::new();
    let _result_reg = compile_term_to_register(term, &mut ctx)?;
    Ok(ctx.instructions())
}

/// Compile a term and put result in a register using only the 5 minimal operations
fn compile_term_to_register(
    term: &Term,
    ctx: &mut CompilationContext,
) -> Result<RegisterId, CompileError> {
    match &term.kind {
        // Variables - just return the bound register
        TermKind::Var(name) => {
            ctx.get_var_register(name)
                .ok_or_else(|| CompileError::UnboundVariable(name.clone()))
        }
        
        // Literals - allocate as resources
        TermKind::Literal(lit) => {
            Ok(ctx.alloc_literal(lit))
        }
        
        // Unit values
        TermKind::Unit => {
            let unit_lit = Literal::Unit;
            Ok(ctx.alloc_literal(&unit_lit))
        }
        
        // Function application - use Transform instruction
        TermKind::Apply { func, arg } => {
            let func_reg = compile_term_to_register(func, ctx)?;
            let arg_reg = compile_term_to_register(arg, ctx)?;
            let result_reg = ctx.fresh_register();
            
            // Transform: apply function morphism to argument
            ctx.emit(Instruction::Transform {
                morph_reg: func_reg,
                input_reg: arg_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Resource allocation - direct Alloc instruction
        TermKind::Alloc { value } => {
            let value_reg = compile_term_to_register(value, ctx)?;
            let type_reg = ctx.fresh_register(); // TODO: infer proper type
            let result_reg = ctx.fresh_register();
            
            ctx.emit(Instruction::Alloc {
                type_reg,
                init_reg: value_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Resource consumption - direct Consume instruction
        TermKind::Consume { resource } => {
            let resource_reg = compile_term_to_register(resource, ctx)?;
            let result_reg = ctx.fresh_register();
            
            ctx.emit(Instruction::Consume {
                resource_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Transform constructors - create morphisms using Compose
        TermKind::Transform { input_type: _, output_type: _, location: _, body } => {
            // Compile the transform body as a morphism
            let body_reg = compile_term_to_register(body, ctx)?;
            
            // For now, return the body register as the morphism
            // In a full implementation, this would create a proper morphism
            Ok(body_reg)
        }
        
        // Transform application - use Transform instruction  
        TermKind::ApplyTransform { transform, arg } => {
            let transform_reg = compile_term_to_register(transform, ctx)?;
            let arg_reg = compile_term_to_register(arg, ctx)?;
            let result_reg = ctx.fresh_register();
            
            ctx.emit(Instruction::Transform {
                morph_reg: transform_reg,
                input_reg: arg_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Location annotations - for now, compile body locally
        TermKind::At { location: _, body } => {
            compile_term_to_register(body, ctx)
        }
        
        // Let bindings - compile value and bind to variable
        TermKind::Let { var, value, body } => {
            let value_reg = compile_term_to_register(value, ctx)?;
            ctx.bind_var(var.clone(), value_reg);
            compile_term_to_register(body, ctx)
        }
        
        // Lambda abstractions - create function morphisms
        TermKind::Lambda { param, param_type: _, body } => {
            let func_reg = ctx.fresh_register();
            let param_reg = ctx.fresh_register();
            
            // Bind parameter
            ctx.bind_var(param.clone(), param_reg);
            
            // Compile body
            let body_reg = compile_term_to_register(body, ctx)?;
            
            // Create function type
            let func_type_reg = ctx.fresh_register();
            let init_reg = ctx.fresh_register();
            
            // Allocate function as a resource
            ctx.emit(Instruction::Alloc {
                type_reg: func_type_reg,
                init_reg,
                output_reg: func_reg,
            });
            
            Ok(func_reg)
        }
        
        // Product variant doesn't exist in current TermKind
        // TermKind::Product { left, right } => {
        //     let left_instr = compile_term(*left, det_sys)?;
        //     let right_instr = compile_term(*right, det_sys)?;
        //     // Create tensor product instruction
        //     Ok(vec![/* product compilation */])
        // }
        
        // TODO: Implement compilation for other term kinds using minimal instruction set
        _ => Err(CompileError::NotImplemented(format!("{:?}", term.kind))),
    }
}

//-----------------------------------------------------------------------------
// Compilation Errors
//-----------------------------------------------------------------------------

/// Errors that can occur during compilation to minimal instruction set
#[derive(Debug, Clone)]
pub enum CompileError {
    /// Variable not bound in context
    UnboundVariable(String),
    
    /// Feature not yet implemented
    NotImplemented(String),
    
    /// Type error
    TypeError(String),
    
    /// Invalid morphism composition
    InvalidComposition(String),
    
    /// Resource error
    ResourceError(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnboundVariable(var) => write!(f, "Unbound variable: {}", var),
            CompileError::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
            CompileError::TypeError(msg) => write!(f, "Type error: {}", msg),
            CompileError::InvalidComposition(msg) => write!(f, "Invalid morphism composition: {}", msg),
            CompileError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {} 