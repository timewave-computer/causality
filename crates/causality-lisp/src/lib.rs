//! Causality Lisp
//!
//! A Lisp-based DSL for expressing effects and computations in the Causality framework.
//! Provides an AST, parser, interpreter, compiler, and type checker for Lisp-style programming.
//! 
//! This implementation contains exactly the 11 core Layer 1 primitives with syntactic
//! sugar support for higher-level convenience forms that desugar to the core primitives.

pub mod ast;
pub mod compiler;
pub mod desugar;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod type_checker;
pub mod value;

// Re-export main types
pub use ast::{Expr, ExprKind, LispValue};
pub use compiler::{LispCompiler, CompilerContext, CompileResult as LispCompileResult};
pub use desugar::{SugarExpr, desugar};
pub use error::{LispError, EvalError, ParseError, TypeError};
pub use interpreter::{Interpreter, EvalContext};
pub use parser::{LispParser};
pub use type_checker::{TypeChecker, TypeContext};
pub use value::{Value, ValueKind, Environment};

// Convenience function for quick evaluation
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut parser = LispParser::new();
    parser.parse(input)
}

/// Parse with syntactic sugar support
pub fn parse_sugar(input: &str) -> Result<SugarExpr, ParseError> {
    // For now, just parse as core expression and wrap
    // Full sugar parsing would extend the parser
    let expr = parse(input)?;
    Ok(SugarExpr::core(expr))
}

/// Quick evaluation function
pub fn eval(expr: &Expr) -> Result<Value, EvalError> {
    let mut interpreter = Interpreter::new();
    interpreter.eval(expr)
}

/// Evaluate with sugar desugaring
pub fn eval_sugar(sugar_expr: SugarExpr) -> Result<Value, EvalError> {
    let core_expr = desugar(sugar_expr);
    eval(&core_expr)
}

pub fn run(input: &str) -> Result<Value, LispError> {
    let expr = parse(input)?;
    let result = eval(&expr)?;
    Ok(result)
}

/// Run with sugar support
pub fn run_sugar(input: &str) -> Result<Value, LispError> {
    let sugar_expr = parse_sugar(input)?;
    let result = eval_sugar(sugar_expr)?;
    Ok(result)
}

/// Compile Lisp code to Layer 0 instructions
pub fn compile(input: &str) -> Result<(Vec<causality_core::machine::instruction::Instruction>, causality_core::machine::instruction::RegisterId), LispError> {
    let expr = parse(input)?;
    let mut compiler = LispCompiler::new();
    compiler.compile(&expr)
}

/// E2E: Parse, compile, and prepare for simulation
pub fn compile_for_simulation(input: &str) -> Result<E2EResult, LispError> {
    // Parse the Lisp code
    let expr = parse(input)?;
    
    // Type check
    let mut type_checker = TypeChecker::new();
    let _expr_type = type_checker.check_expr(&expr)?;
    
    // Compile to Layer 0
    let mut compiler = LispCompiler::new();
    let (instructions, result_reg) = compiler.compile(&expr)?;
    
    Ok(E2EResult {
        original_expr: expr,
        instructions: instructions.clone(),
        result_register: result_reg,
        instruction_count: instructions.len(),
    })
}

/// Result of E2E compilation process
#[derive(Debug, Clone)]
pub struct E2EResult {
    /// Original Lisp expression
    pub original_expr: Expr,
    /// Compiled Layer 0 instructions  
    pub instructions: Vec<causality_core::machine::instruction::Instruction>,
    /// Register containing the final result
    pub result_register: causality_core::machine::instruction::RegisterId,
    /// Total number of instructions generated
    pub instruction_count: usize,
} 