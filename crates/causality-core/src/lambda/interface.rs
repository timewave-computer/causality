//! Interface compilation from Layer 1 terms to minimal machine instructions
//!
//! This module implements the compilation from Layer 1 terms (lambda calculus with
//! linear types, effects, and session types) to the minimal 5-operation instruction set.

use crate::lambda::term::{Term, TermKind, Literal};
use crate::lambda::base::{TypeInner, BaseType};
use crate::machine::instruction::{Instruction, RegisterId};
use crate::machine::value::MachineValue;
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Compilation Context
//-----------------------------------------------------------------------------

/// Compilation context for generating minimal instruction sequences
#[derive(Debug, Clone)]
pub struct CompilationContext {
    /// Next available register
    next_register: u32,
    
    /// Variable to register mapping
    var_registers: BTreeMap<String, RegisterId>,
    
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
            var_registers: BTreeMap::new(),
            instructions: Vec::new(),
        }
    }
    
    /// Allocate a fresh register
    pub fn fresh_register(&mut self) -> RegisterId {
        let reg = RegisterId::new(self.next_register);
        self.next_register += 1;
        reg
    }
    
    /// Get the register for a variable
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
    
    /// Allocate a literal value as a resource using Load instruction pattern
    pub fn alloc_literal(&mut self, lit: &Literal) -> RegisterId {
        let result_reg = self.fresh_register();
        let type_reg = self.fresh_register();
        let init_reg = self.fresh_register();
        
        // Create the literal value
        let _literal_value = match lit {
            Literal::Bool(b) => MachineValue::Bool(*b),
            Literal::Int(i) => MachineValue::Int(*i),
            Literal::Symbol(s) => MachineValue::Symbol(s.clone()),
            Literal::Unit => MachineValue::Unit,
        };
        
        // Create type value for the literal
        let _type_value = match lit {
            Literal::Bool(_) => MachineValue::Type(TypeInner::Base(BaseType::Bool)),
            Literal::Int(_) => MachineValue::Type(TypeInner::Base(BaseType::Int)),
            Literal::Symbol(_) => MachineValue::Type(TypeInner::Base(BaseType::Symbol)),
            Literal::Unit => MachineValue::Type(TypeInner::Base(BaseType::Unit)),
        };
        
        // Load instruction pattern: First allocate type and init values
        // This simulates loading constants into registers
        let meta_type_reg = self.fresh_register();
        let meta_init_reg = self.fresh_register();
        self.emit(Instruction::Alloc {
            type_reg: meta_type_reg, // Meta-type for types
            init_reg: meta_init_reg, // Meta-init for types
            output_reg: type_reg,
        });
        
        let unit_reg = self.fresh_register();
        self.emit(Instruction::Alloc {
            type_reg,
            init_reg: unit_reg, // Unit for init
            output_reg: init_reg,
        });
        
        // Now allocate the actual literal as a resource
        self.emit(Instruction::Alloc {
            type_reg,
            init_reg,
            output_reg: result_reg,
        });
        
        result_reg
    }
    
    /// Infer the proper type for a term
    pub fn infer_type(&self, term: &Term) -> TypeInner {
        match &term.kind {
            TermKind::Literal(lit) => match lit {
                Literal::Bool(_) => TypeInner::Base(BaseType::Bool),
                Literal::Int(_) => TypeInner::Base(BaseType::Int),
                Literal::Symbol(_) => TypeInner::Base(BaseType::Symbol),
                Literal::Unit => TypeInner::Base(BaseType::Unit),
            },
            TermKind::Unit => TypeInner::Base(BaseType::Unit),
            TermKind::Var(_) => {
                // For variables, we'd need type environment
                // Default to Unit for now
                TypeInner::Base(BaseType::Unit)
            },
            TermKind::Apply { func, .. } => {
                // For function application, infer from function type
                let _func_type = self.infer_type(&func);
                match _func_type {
                    TypeInner::LinearFunction(_, output) => *output,
                    _ => TypeInner::Base(BaseType::Unit),
                }
            },
            TermKind::Lambda { param_type, .. } => {
                // Create function type
                let input_type = param_type.clone()
                    .unwrap_or(TypeInner::Base(BaseType::Unit));
                TypeInner::LinearFunction(
                    Box::new(input_type),
                    Box::new(TypeInner::Base(BaseType::Unit)), // Would need to infer from body
                )
            },
            _ => TypeInner::Base(BaseType::Unit), // Default fallback
        }
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
            ctx.get_var_register(&name)
                .ok_or_else(|| CompileError::UnboundVariable(name.clone()))
        }
        
        // Literals - allocate as resources
        TermKind::Literal(lit) => {
            Ok(ctx.alloc_literal(&lit))
        }
        
        // Unit values
        TermKind::Unit => {
            let unit_lit = Literal::Unit;
            Ok(ctx.alloc_literal(&unit_lit))
        }
        
        // Function application - use Transform instruction
        TermKind::Apply { func, arg } => {
            let func_reg = compile_term_to_register(&func, ctx)?;
            let arg_reg = compile_term_to_register(&arg, ctx)?;
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
            let value_reg = compile_term_to_register(&value, ctx)?;
            let _inferred_type = ctx.infer_type(&value);
            let type_reg = ctx.fresh_register();
            let result_reg = ctx.fresh_register();
            
            // Pre-allocate all needed registers
            let meta_type_reg = ctx.fresh_register();
            let meta_init_reg = ctx.fresh_register();
            
            // Allocate type value first
            ctx.emit(Instruction::Alloc {
                type_reg: meta_type_reg,
                init_reg: meta_init_reg,
                output_reg: type_reg,
            });
            
            // Then allocate the actual resource
            ctx.emit(Instruction::Alloc {
                type_reg,
                init_reg: value_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Resource consumption - direct Consume instruction
        TermKind::Consume { resource } => {
            let resource_reg = compile_term_to_register(&resource, ctx)?;
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
            let body_reg = compile_term_to_register(&body, ctx)?;
            
            // Create a morphism by composing with identity
            let identity_reg = ctx.fresh_register();
            let result_reg = ctx.fresh_register();
            
            // Pre-allocate all needed registers
            let id_type_reg = ctx.fresh_register();
            let id_init_reg = ctx.fresh_register();
            
            // Allocate identity morphism
            ctx.emit(Instruction::Alloc {
                type_reg: id_type_reg,
                init_reg: id_init_reg,
                output_reg: identity_reg,
            });
            
            // Compose with body to create the transform morphism
            ctx.emit(Instruction::Compose {
                first_reg: identity_reg,
                second_reg: body_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Session send operation
        TermKind::Send { channel, value } => {
            let channel_reg = compile_term_to_register(&channel, ctx)?;
            let value_reg = compile_term_to_register(&value, ctx)?;
            let result_reg = ctx.fresh_register();
            
            // Use Transform instruction for session send
            ctx.emit(Instruction::Transform {
                morph_reg: channel_reg,
                input_reg: value_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Session receive operation
        TermKind::Receive { channel } => {
            let channel_reg = compile_term_to_register(&channel, ctx)?;
            let result_reg = ctx.fresh_register();
            
            // Use Transform instruction for session receive
            ctx.emit(Instruction::Transform {
                morph_reg: channel_reg,
                input_reg: channel_reg, // Channel acts as both transform and input
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Let binding - Sequential composition pattern
        TermKind::Let { var, value, body } => {
            let value_reg = compile_term_to_register(&value, ctx)?;
            ctx.bind_var(var.clone(), value_reg);
            compile_term_to_register(&body, ctx)
        }
        
        // Lambda abstraction - Create function morphism
        TermKind::Lambda { param: _, param_type, body } => {
            let body_reg = compile_term_to_register(&body, ctx)?;
            
            // Create function type value
            let param_type_inner = param_type.clone()
                .unwrap_or(TypeInner::Base(BaseType::Unit));
                 
            let body_type_inner = ctx.infer_type(&body);
            
            let _func_type = TypeInner::LinearFunction(
                Box::new(param_type_inner),
                Box::new(body_type_inner),
            );
            
            // Allocate type and function
            let type_reg = ctx.fresh_register();
            let meta_type_reg = ctx.fresh_register();
            let meta_init_reg = ctx.fresh_register();
            let result_reg = ctx.fresh_register();
            
            // Allocate type value
            ctx.emit(Instruction::Alloc {
                type_reg: meta_type_reg,
                init_reg: meta_init_reg,
                output_reg: type_reg,
            });
            
            // Allocate function
            ctx.emit(Instruction::Alloc {
                type_reg,
                init_reg: body_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Product construction (pairs)
        TermKind::Tensor { left, right } => {
            let left_reg = compile_term_to_register(&left, ctx)?;
            let right_reg = compile_term_to_register(&right, ctx)?;
            let result_reg = ctx.fresh_register();
            
            // Use Tensor instruction for product construction
            ctx.emit(Instruction::Tensor {
                left_reg,
                right_reg,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
        // Case analysis/pattern matching
        TermKind::Case { scrutinee, left_var, left_body, .. } => {
            let scrutinee_reg = compile_term_to_register(&scrutinee, ctx)?;
            
            // Simplified: just compile one branch for now
            // In a full implementation, this would generate conditional branching
            ctx.bind_var(left_var.clone(), scrutinee_reg);
            let left_result = compile_term_to_register(&left_body, ctx)?;
            
            // Create choice using Compose (simplified)
            let result_reg = ctx.fresh_register();
            ctx.emit(Instruction::Compose {
                first_reg: scrutinee_reg,
                second_reg: left_result,
                output_reg: result_reg,
            });
            
            Ok(result_reg)
        }
        
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
            CompileError::InvalidComposition(msg) => write!(f, "Invalid composition: {}", msg),
            CompileError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {} 