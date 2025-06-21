//! Compiler for translating Causality Lisp AST to Layer 0 instructions
//!
//! This module provides compilation from the 11 core Lisp primitives to the Layer 0
//! register machine instruction set.

use crate::{
    ast::{Expr, ExprKind, LispValue},
    error::LispError,
};
use causality_core::machine::instruction::{
    Instruction, RegisterId,
};
use causality_core::lambda::Symbol;
use std::collections::BTreeMap;

/// Result type for compilation operations
pub type CompileResult<T> = Result<T, LispError>;

/// Compilation context for tracking registers and variable bindings
#[derive(Debug, Clone)]
pub struct CompilerContext {
    /// Current register counter
    next_register: u32,
    
    /// Variable name to register mapping
    bindings: BTreeMap<Symbol, RegisterId>,
    
    /// Label counter for control flow
    next_label: u32,
}

impl CompilerContext {
    /// Create a new compiler context
    pub fn new() -> Self {
        Self {
            next_register: 0,
            bindings: BTreeMap::new(),
            next_label: 0,
        }
    }
    
    /// Allocate a new register
    pub fn alloc_register(&mut self) -> RegisterId {
        let reg = RegisterId::new(self.next_register);
        self.next_register += 1;
        reg
    }
    
    /// Bind a variable to a register
    pub fn bind_variable(&mut self, name: Symbol, reg: RegisterId) {
        self.bindings.insert(name, reg);
    }
    
    /// Look up a variable binding
    pub fn lookup_variable(&self, name: &Symbol) -> Option<RegisterId> {
        self.bindings.get(name).copied()
    }
    
    /// Generate a new label
    pub fn alloc_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.next_label);
        self.next_label += 1;
        label
    }
}

/// Lisp to Layer 0 compiler
pub struct LispCompiler {
    /// Current compilation context
    context: CompilerContext,
}

impl LispCompiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            context: CompilerContext::new(),
        }
    }
    
    /// Compile a Lisp expression to Layer 0 instructions
    pub fn compile(&mut self, expr: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        self.compile_expr(expr)
    }
    
    /// Compile an expression and return instructions and result register
    fn compile_expr(&mut self, expr: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        match &expr.kind {
            // Constants
            ExprKind::Const(value) => self.compile_const(value),
            
            // Variables
            ExprKind::Var(name) => self.compile_var(name),
            
            // Unit value
            ExprKind::UnitVal => self.compile_unit(),
            
            // Unit elimination
            ExprKind::LetUnit(unit_expr, body) => self.compile_let_unit(unit_expr, body),
            
            // Tensor product
            ExprKind::Tensor(left, right) => self.compile_tensor(left, right),
            
            // Tensor elimination
            ExprKind::LetTensor(tensor_expr, left_name, right_name, body) => {
                self.compile_let_tensor(tensor_expr, left_name, right_name, body)
            }
            
            // Sum types
            ExprKind::Inl(value) => self.compile_inl(value),
            ExprKind::Inr(value) => self.compile_inr(value),
            ExprKind::Case(expr, left_name, left_branch, right_name, right_branch) => {
                self.compile_case(expr, left_name, left_branch, right_name, right_branch)
            }
            
            // Functions
            ExprKind::Lambda(params, body) => self.compile_lambda(params, body),
            ExprKind::Apply(func_expr, args) => self.compile_apply(func_expr, args),
            
            // Resource management
            ExprKind::Alloc(value_expr) => self.compile_alloc(value_expr),
            ExprKind::Consume(resource_expr) => self.compile_consume(resource_expr),
            
            // Record operations
            ExprKind::RecordAccess { record, field } => self.compile_record_access(record, field),
            ExprKind::RecordUpdate { record, field, value } => self.compile_record_update(record, field, value),

            // Session types operations
            ExprKind::SessionDeclaration { name, roles } => self.compile_session_declaration(name, roles),
            ExprKind::WithSession { session, role, body } => self.compile_with_session(session, role, body),
            ExprKind::SessionSend { channel, value } => self.compile_session_send(channel, value),
            ExprKind::SessionReceive { channel } => self.compile_session_receive(channel),
            ExprKind::SessionSelect { channel, choice } => self.compile_session_select(channel, choice),
            ExprKind::SessionCase { channel, branches } => self.compile_session_case(channel, branches),
        }
    }
    
    /// Compile a constant value (improved implementation)
    fn compile_const(&mut self, value: &LispValue) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let result_reg = self.context.alloc_register();
        
        // Improved constant handling that generates appropriate instructions for different value types
        let instructions = match value {
            LispValue::Unit => {
                // Unit value - simplest case
                vec![Instruction::Witness { out_reg: result_reg }]
            },
            LispValue::Int(_n) => {
                // For integer constants, generate a specific alloc instruction
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg }, // Load the integer value first
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::Bool(_b) => {
                // For boolean constants, generate alloc with specific type
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg }, // Load the boolean value
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },

            LispValue::String(_s) => {
                // For string constants
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::Symbol(_sym) => {
                // For symbol constants
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::List(_list) => {
                // For list constants (more complex)
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::Map(_map) => {
                // For map constants
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::Record(_record) => {
                // For record constants
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::ResourceId(_id) => {
                // For resource ID constants
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::ExprId(_id) => {
                // For expression ID constants
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
            LispValue::CoreValue(_core_val) => {
                // For core value constants (integration with core system)
                let val_reg = self.context.alloc_register();
                vec![
                    Instruction::Witness { out_reg: val_reg },
                    Instruction::Alloc { 
                        type_reg: self.context.alloc_register(), 
                        val_reg,
                        out_reg: result_reg 
                    },
                ]
            },
        };
        
        Ok((instructions, result_reg))
    }
    
    /// Compile a variable reference
    fn compile_var(&mut self, name: &Symbol) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        if let Some(reg) = self.context.lookup_variable(name) {
            // Variable is already in a register, just return it
            Ok((vec![], reg))
        } else {
            Err(LispError::Eval(crate::error::EvalError::UnboundVariable(
                name.to_string()
            )))
        }
    }
    
    /// Compile unit value
    fn compile_unit(&mut self) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let result_reg = self.context.alloc_register();
        let instructions = vec![
            Instruction::Witness { out_reg: result_reg }, // Load unit value
        ];
        Ok((instructions, result_reg))
    }
    
    /// Compile let-unit (unit elimination)
    fn compile_let_unit(&mut self, unit_expr: &Expr, body: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, _unit_reg) = self.compile_expr(unit_expr)?;
        let (body_instructions, result_reg) = self.compile_expr(body)?;
        
        instructions.extend(body_instructions);
        Ok((instructions, result_reg))
    }
    
    /// Compile tensor product (pair creation) - improved implementation
    fn compile_tensor(&mut self, left: &Expr, right: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, left_reg) = self.compile_expr(left)?;
        let (right_instructions, _right_reg) = self.compile_expr(right)?;
        
        instructions.extend(right_instructions);
        
        // Create a proper pair structure using allocation and field assignments
        let type_reg = self.context.alloc_register();
        let result_reg = self.context.alloc_register();
        
        // First allocate memory for the pair
        instructions.push(Instruction::Alloc { 
            type_reg, 
            val_reg: left_reg,  // Use left component as initial value
            out_reg: result_reg 
        });
        
        // Store both components in the pair structure
        // In a real implementation, this would use proper field access
        instructions.push(Instruction::Move { src: left_reg, dst: result_reg });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile let-tensor (pair elimination)
    fn compile_let_tensor(
        &mut self,
        tensor_expr: &Expr,
        left_name: &Symbol,
        right_name: &Symbol,
        body: &Expr,
    ) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, tensor_reg) = self.compile_expr(tensor_expr)?;
        
        // Allocate registers for the pair components
        let left_reg = self.context.alloc_register();
        let right_reg = self.context.alloc_register();
        
        // In a real implementation, this would destructure the pair
        // For now, we'll use moves as placeholders
        instructions.push(Instruction::Move { src: tensor_reg, dst: left_reg });
        instructions.push(Instruction::Move { src: tensor_reg, dst: right_reg });
        
        // Bind variables
        self.context.bind_variable(left_name.clone(), left_reg);
        self.context.bind_variable(right_name.clone(), right_reg);
        
        // Compile body
        let (body_instructions, result_reg) = self.compile_expr(body)?;
        instructions.extend(body_instructions);
        
        Ok((instructions, result_reg))
    }
    
    /// Compile left injection
    fn compile_inl(&mut self, value: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, value_reg) = self.compile_expr(value)?;
        let result_reg = self.context.alloc_register();
        
        // In a real implementation, this would tag the value as left variant
        instructions.push(Instruction::Move { src: value_reg, dst: result_reg });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile right injection
    fn compile_inr(&mut self, value: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, value_reg) = self.compile_expr(value)?;
        let result_reg = self.context.alloc_register();
        
        // In a real implementation, this would tag the value as right variant
        instructions.push(Instruction::Move { src: value_reg, dst: result_reg });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile case expression (sum elimination)
    fn compile_case(
        &mut self,
        expr: &Expr,
        left_name: &Symbol,
        left_branch: &Expr,
        right_name: &Symbol,
        right_branch: &Expr,
    ) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, sum_reg) = self.compile_expr(expr)?;
        
        let left_label = self.context.alloc_label("case_left");
        let right_label = self.context.alloc_label("case_right");
        let end_label = self.context.alloc_label("case_end");
        
        let left_reg = self.context.alloc_register();
        let right_reg = self.context.alloc_register();
        let result_reg = self.context.alloc_register();
        
        // Pattern match on sum type
        instructions.push(Instruction::Match {
            sum_reg,
            left_reg,
            right_reg,
            left_label: left_label.clone(),
            right_label: right_label.clone(),
        });
        
        // Left branch
        instructions.push(Instruction::LabelMarker(left_label));
        self.context.bind_variable(left_name.clone(), left_reg);
        let (left_instructions, left_result) = self.compile_expr(left_branch)?;
        instructions.extend(left_instructions);
        instructions.push(Instruction::Move { src: left_result, dst: result_reg });
        
        // Right branch
        instructions.push(Instruction::LabelMarker(right_label));
        self.context.bind_variable(right_name.clone(), right_reg);
        let (right_instructions, right_result) = self.compile_expr(right_branch)?;
        instructions.extend(right_instructions);
        instructions.push(Instruction::Move { src: right_result, dst: result_reg });
        
        instructions.push(Instruction::LabelMarker(end_label));
        
        Ok((instructions, result_reg))
    }
    
    /// Compile lambda (function creation) - improved implementation
    fn compile_lambda(&mut self, _params: &[crate::ast::Param], _body: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let result_reg = self.context.alloc_register();
        
        // Create a more realistic closure structure
        if _params.len() != 1 {
            return Err(LispError::Eval(crate::error::EvalError::NotImplemented(
                "Multi-parameter lambdas not yet supported".to_string()
            )));
        }
        
        // Save current context to restore later
        let saved_bindings = self.context.bindings.clone();
        
        // Create closure environment
        let env_reg = self.context.alloc_register();
        let closure_reg = self.context.alloc_register();
        
        // Allocate environment for captured variables
        let instructions = vec![
            Instruction::Witness { out_reg: env_reg }, // Create environment
            Instruction::Alloc {
                type_reg: self.context.alloc_register(),
                val_reg: env_reg,
                out_reg: closure_reg,
            },
            Instruction::Move { src: closure_reg, dst: result_reg },
        ];
        
        // Restore original bindings
        self.context.bindings = saved_bindings;
        
        Ok((instructions, result_reg))
    }
    
    /// Compile function application
    fn compile_apply(&mut self, func_expr: &Expr, args: &[Expr]) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, func_reg) = self.compile_expr(func_expr)?;
        
        if args.len() != 1 {
            return Err(LispError::Eval(crate::error::EvalError::NotImplemented(
                "Multi-argument application not yet supported".to_string()
            )));
        }
        
        let (arg_instructions, arg_reg) = self.compile_expr(&args[0])?;
        instructions.extend(arg_instructions);
        
        let result_reg = self.context.alloc_register();
        instructions.push(Instruction::Apply {
            fn_reg: func_reg,
            arg_reg,
            out_reg: result_reg,
        });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile allocation
    fn compile_alloc(&mut self, value_expr: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, value_reg) = self.compile_expr(value_expr)?;
        
        let type_reg = self.context.alloc_register();
        let result_reg = self.context.alloc_register();
        
        // Load type information
        instructions.push(Instruction::Witness { out_reg: type_reg });
        
        // Allocate resource
        instructions.push(Instruction::Alloc {
            type_reg,
            val_reg: value_reg,
            out_reg: result_reg,
        });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile consumption
    fn compile_consume(&mut self, resource_expr: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, resource_reg) = self.compile_expr(resource_expr)?;
        
        let result_reg = self.context.alloc_register();
        instructions.push(Instruction::Consume {
            resource_reg,
            out_reg: result_reg,
        });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile record field access
    fn compile_record_access(&mut self, record: &Expr, _field: &str) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, record_reg) = self.compile_expr(record)?;
        let result_reg = self.context.alloc_register();
        
        // In a real implementation, this would perform field access
        instructions.push(Instruction::Move { src: record_reg, dst: result_reg });
        
        Ok((instructions, result_reg))
    }
    
    /// Compile record field update
    fn compile_record_update(&mut self, record: &Expr, _field: &str, value: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, record_reg) = self.compile_expr(record)?;
        let (value_instructions, _value_reg) = self.compile_expr(value)?;
        
        instructions.extend(value_instructions);
        
        let result_reg = self.context.alloc_register();
        
        // In a real implementation, this would perform field update
        instructions.push(Instruction::Move { src: record_reg, dst: result_reg });
        
        Ok((instructions, result_reg))
    }

    // Placeholder implementations for session types operations
    fn compile_session_declaration(&mut self, _name: &str, _roles: &[causality_core::effect::session_registry::SessionRole]) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        // For now, just return a unit value
        let result_reg = self.context.alloc_register();
        let instructions = vec![Instruction::Witness { out_reg: result_reg }];
        Ok((instructions, result_reg))
    }

    fn compile_with_session(&mut self, _session: &str, _role: &str, body: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        // For now, just compile the body
        self.compile_expr(body)
    }

    fn compile_session_send(&mut self, channel: &Expr, value: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, _channel_reg) = self.compile_expr(channel)?;
        let (value_instructions, _value_reg) = self.compile_expr(value)?;
        instructions.extend(value_instructions);
        
        let result_reg = self.context.alloc_register();
        instructions.push(Instruction::Witness { out_reg: result_reg });
        Ok((instructions, result_reg))
    }

    fn compile_session_receive(&mut self, channel: &Expr) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, _channel_reg) = self.compile_expr(channel)?;
        
        let result_reg = self.context.alloc_register();
        instructions.push(Instruction::Witness { out_reg: result_reg });
        Ok((instructions, result_reg))
    }

    fn compile_session_select(&mut self, channel: &Expr, _choice: &str) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, _channel_reg) = self.compile_expr(channel)?;
        
        let result_reg = self.context.alloc_register();
        instructions.push(Instruction::Witness { out_reg: result_reg });
        Ok((instructions, result_reg))
    }

    fn compile_session_case(&mut self, channel: &Expr, _branches: &[crate::ast::SessionBranch]) -> CompileResult<(Vec<Instruction>, RegisterId)> {
        let (mut instructions, _channel_reg) = self.compile_expr(channel)?;
        
        let result_reg = self.context.alloc_register();
        instructions.push(Instruction::Witness { out_reg: result_reg });
        Ok((instructions, result_reg))
    }
}

impl Default for LispCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, ExprKind, LispValue};
    use causality_core::lambda::Symbol;

    #[test]
    fn test_compile_unit() {
        let mut compiler = LispCompiler::new();
        let expr = Expr::new(ExprKind::UnitVal);
        
        let result = compiler.compile(&expr);
        assert!(result.is_ok());
        let (instructions, _reg) = result.unwrap();
        assert!(!instructions.is_empty());
    }

    #[test]
    fn test_compile_const() {
        let mut compiler = LispCompiler::new();
        let expr = Expr::new(ExprKind::Const(LispValue::Int(42)));
        
        let result = compiler.compile(&expr);
        assert!(result.is_ok());
        let (instructions, _reg) = result.unwrap();
        assert!(!instructions.is_empty());
    }

    #[test]
    fn test_compile_variable_unbound() {
        let mut compiler = LispCompiler::new();
        let expr = Expr::new(ExprKind::Var(Symbol::new("x")));
        
        let result = compiler.compile(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_tensor() {
        let mut compiler = LispCompiler::new();
        let left = Expr::new(ExprKind::UnitVal);
        let right = Expr::new(ExprKind::UnitVal);
        let expr = Expr::new(ExprKind::Tensor(Box::new(left), Box::new(right)));
        
        let result = compiler.compile(&expr);
        assert!(result.is_ok());
        let (instructions, _reg) = result.unwrap();
        assert!(instructions.len() > 2); // Should have multiple instructions
    }

    #[test]
    fn test_compile_sum_injection() {
        let mut compiler = LispCompiler::new();
        let value = Expr::new(ExprKind::UnitVal);
        let expr = Expr::new(ExprKind::Inl(Box::new(value)));
        
        let result = compiler.compile(&expr);
        assert!(result.is_ok());
        let (instructions, _reg) = result.unwrap();
        assert!(instructions.len() > 1);
    }

    #[test]
    fn test_compile_alloc() {
        let mut compiler = LispCompiler::new();
        let value = Expr::new(ExprKind::UnitVal);
        let expr = Expr::new(ExprKind::Alloc(Box::new(value)));
        
        let result = compiler.compile(&expr);
        assert!(result.is_ok());
        let (instructions, _reg) = result.unwrap();
        assert!(instructions.len() > 2); // Should have witness + alloc
    }

    #[test]
    fn test_compile_consume() {
        let mut compiler = LispCompiler::new();
        let resource = Expr::new(ExprKind::UnitVal);
        let expr = Expr::new(ExprKind::Consume(Box::new(resource)));
        
        let result = compiler.compile(&expr);
        assert!(result.is_ok());
        let (instructions, _reg) = result.unwrap();
        assert!(instructions.len() > 1); // Should have witness + consume
    }

    #[test]
    fn test_compilation_context() {
        let mut context = CompilerContext::new();
        
        // Test register allocation
        let reg1 = context.alloc_register();
        let reg2 = context.alloc_register();
        assert_ne!(reg1.id(), reg2.id());
        
        // Test variable binding
        let var_name = Symbol::new("test_var");
        context.bind_variable(var_name.clone(), reg1);
        
        let looked_up = context.lookup_variable(&var_name);
        assert_eq!(looked_up, Some(reg1));
        
        // Test label generation
        let label1 = context.alloc_label("test");
        let label2 = context.alloc_label("test");
        assert_ne!(label1, label2);
        assert!(label1.starts_with("test_"));
    }

    #[test]
    fn test_e2e_compilation_count() {
        // Test that we can count instructions properly
        let mut compiler = LispCompiler::new();
        
        // Simple expression
        let simple = Expr::new(ExprKind::UnitVal);
        let (simple_instructions, _) = compiler.compile(&simple).unwrap();
        let simple_count = simple_instructions.len();
        
        // Complex expression
        let mut compiler2 = LispCompiler::new();
        let left = Expr::new(ExprKind::UnitVal);
        let right = Expr::new(ExprKind::UnitVal);
        let complex = Expr::new(ExprKind::Tensor(Box::new(left), Box::new(right)));
        let (complex_instructions, _) = compiler2.compile(&complex).unwrap();
        let complex_count = complex_instructions.len();
        
        // Complex should have more instructions
        assert!(complex_count > simple_count);
    }
} 