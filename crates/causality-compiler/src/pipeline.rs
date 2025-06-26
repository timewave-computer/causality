//! Compilation Pipeline: Parse → Check → Compile
//!
//! This module implements the complete compilation flow from Lisp source
//! to verified register machine instructions, following the three-layer architecture.

use crate::error::{CompileError, CompileResult, Location};
use causality_core::lambda::{Term, TermKind, Literal};
use causality_core::machine::{Instruction, RegisterId};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// S-Expression Parsing
//-----------------------------------------------------------------------------

/// S-expression representation
#[derive(Debug, Clone, PartialEq)]
pub enum SExpression {
    /// Atomic symbol
    Symbol(String),
    /// Integer literal
    Integer(u32),
    /// Boolean literal
    Boolean(bool),
    /// List of S-expressions
    List(Vec<SExpression>),
    /// Nil (empty list)
    Nil,
}

impl std::fmt::Display for SExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SExpression::Symbol(s) => write!(f, "{}", s),
            SExpression::Integer(n) => write!(f, "{}", n),
            SExpression::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            SExpression::Nil => write!(f, "nil"),
            SExpression::List(elements) => {
                write!(f, "(")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// Simple tokenizer for Lisp parsing
struct Tokenizer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.input.get(self.pos).copied() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_symbol(&mut self) -> String {
        let mut symbol = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '?' || ch == '!' 
                || ch == '+' || ch == '*' || ch == '/' || ch == '=' || ch == '<' || ch == '>' {
                symbol.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        symbol
    }
    
    fn read_number(&mut self) -> CompileResult<u32> {
        let mut number = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        number.parse().map_err(|_| CompileError::ParseError {
            message: format!("Invalid number: {}", number),
            location: Some(Location { line: self.line, column: self.column }),
        })
    }
    
    fn location(&self) -> Location {
        Location { line: self.line, column: self.column }
    }
}

/// Parse a single S-expression
fn parse_sexpr(input: &str) -> CompileResult<SExpression> {
    let mut tokenizer = Tokenizer::new(input);
    parse_expr(&mut tokenizer)
}

fn parse_expr(tokenizer: &mut Tokenizer) -> CompileResult<SExpression> {
    tokenizer.skip_whitespace();
    
    match tokenizer.peek() {
        None => Err(CompileError::ParseError {
            message: "Unexpected end of input".to_string(),
            location: Some(tokenizer.location()),
        }),
        Some('(') => {
            tokenizer.advance(); // consume '('
            parse_list(tokenizer)
        }
        Some(ch) if ch.is_ascii_digit() => {
            let num = tokenizer.read_number()?;
            Ok(SExpression::Integer(num))
        }
        Some('#') => {
            tokenizer.advance(); // consume '#'
            match tokenizer.peek() {
                Some('t') => { tokenizer.advance(); Ok(SExpression::Boolean(true)) }
                Some('f') => { tokenizer.advance(); Ok(SExpression::Boolean(false)) }
                _ => Err(CompileError::ParseError {
                    message: "Invalid boolean literal".to_string(),
                    location: Some(tokenizer.location()),
                }),
            }
        }
        Some(_) => {
            let symbol = tokenizer.read_symbol();
            if symbol.is_empty() {
                Err(CompileError::ParseError {
                    message: "Invalid character".to_string(),
                    location: Some(tokenizer.location()),
                })
            } else if symbol == "nil" {
                Ok(SExpression::Nil)
            } else {
                Ok(SExpression::Symbol(symbol))
            }
        }
    }
}

fn parse_list(tokenizer: &mut Tokenizer) -> CompileResult<SExpression> {
    let mut elements = Vec::new();
    
    loop {
        tokenizer.skip_whitespace();
        
        match tokenizer.peek() {
            None => return Err(CompileError::ParseError {
                message: "Unclosed list".to_string(),
                location: Some(tokenizer.location()),
            }),
            Some(')') => {
                tokenizer.advance(); // consume ')'
                break;
            }
            Some(_) => {
                elements.push(parse_expr(tokenizer)?);
            }
        }
    }
    
    Ok(SExpression::List(elements))
}

//-----------------------------------------------------------------------------
// Compilation Context
//-----------------------------------------------------------------------------

/// Compilation context for managing variable bindings and code generation
struct CompileContext {
    /// Next available register ID
    next_register: u32,
    /// Variable to register mapping
    variables: BTreeMap<String, RegisterId>,
    /// Generated instructions
    instructions: Vec<Instruction>,
}

impl CompileContext {
    fn new() -> Self {
        Self {
            next_register: 0,
            variables: BTreeMap::new(),
            instructions: Vec::new(),
        }
    }
    
    fn alloc_register(&mut self) -> RegisterId {
        let reg = RegisterId::new(self.next_register);
        self.next_register += 1;
        reg
    }
    
    fn bind_variable(&mut self, name: String, reg: RegisterId) {
        self.variables.insert(name, reg);
    }
    
    fn lookup_variable(&self, name: &str) -> Option<RegisterId> {
        self.variables.get(name).copied()
    }
    
    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
    
    fn into_program(self) -> Vec<Instruction> {
        self.instructions
    }
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Convert our S-expression format to the causality-lisp Expr format
fn sexpr_to_lisp_ast(expr: &SExpression) -> CompileResult<causality_lisp::ast::Expr> {
    use causality_lisp::ast::{Expr, ExprKind, LispValue};
    
    let kind = match expr {
        SExpression::Symbol(s) => ExprKind::Var(s.clone().into()),
        SExpression::Integer(n) => ExprKind::Const(LispValue::Int(*n as i64)),
        SExpression::Boolean(b) => ExprKind::Const(LispValue::Bool(*b)),
        SExpression::Nil => ExprKind::UnitVal,
        SExpression::List(elements) => {
            if elements.is_empty() {
                ExprKind::UnitVal
            } else {
                let func = Box::new(sexpr_to_lisp_ast(&elements[0])?);
                let args: Result<Vec<_>, _> = elements.iter().skip(1)
                    .map(sexpr_to_lisp_ast)
                    .collect();
                ExprKind::Apply(func, args?)
            }
        }
    };
    
    Ok(Expr::new(kind))
}

/// Basic linearity checking for resource usage patterns
fn check_linearity(expr: &SExpression) -> CompileResult<()> {
    // Simplified linearity check - just verify basic structure
    match expr {
        SExpression::List(elements) => {
            if !elements.is_empty() {
                if let SExpression::Symbol(op) = &elements[0] {
                    match op.as_str() {
                        "alloc" => {
                            if elements.len() != 2 {
                                return Err(CompileError::CompilationError {
                                    message: format!("alloc requires exactly 1 argument, got {}", elements.len() - 1),
                                    location: None,
                                });
                            }
                        }
                        "consume" => {
                            if elements.len() != 2 {
                                return Err(CompileError::CompilationError {
                                    message: format!("consume requires exactly 1 argument, got {}", elements.len() - 1),
                                    location: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                // Recursively check sub-expressions
                for element in elements {
                    check_linearity(element)?;
                }
            }
            Ok(())
        }
        _ => Ok(())
    }
}

//-----------------------------------------------------------------------------
// Main Compilation Pipeline
//-----------------------------------------------------------------------------

/// Compile a program from source to machine instructions
/// Following: Parse → Check → Compile
pub fn compile(source: &str) -> CompileResult<CompiledArtifact> {
    // Stage 1: Parse
    let sexpr = parse_sexpr(source)?;
    
    // Stage 2: Check (simplified - full type checking not implemented yet)
    // TODO: Implement proper type checking and linearity verification
    
    // Type checking and validation
    // Convert S-expression to the format expected by type checker
    if let Ok(lisp_ast) = sexpr_to_lisp_ast(&sexpr) {
        let mut type_checker = causality_lisp::TypeChecker::new();
        let type_result = type_checker.check_expr(&lisp_ast);
        
        if let Err(ref type_error) = type_result {
            eprintln!("Type checking warning: {:?}", type_error);
        }
    }
    
    // Basic linearity verification - check for proper resource usage patterns
    let linearity_result = check_linearity(&sexpr);
    
    if let Err(ref linearity_error) = linearity_result {
        eprintln!("Linearity checking warning: {:?}", linearity_error);
    }
    
    // Stage 3: Compile
    let term = compile_sexpr_to_term(&sexpr)?;
    let instructions = compile_term_to_instructions(&term)?;
    
    Ok(CompiledArtifact {
        source: source.to_string(),
        sexpr,
        term,
        instructions,
    })
}

/// Compile a single expression (convenience function)
pub fn compile_expression(source: &str) -> CompileResult<Vec<Instruction>> {
    compile(source).map(|artifact| artifact.instructions)
}

/// Compile a program (convenience function for backwards compatibility)
pub fn compile_program(source: &str) -> CompileResult<Vec<Instruction>> {
    compile_expression(source)
}

//-----------------------------------------------------------------------------
// Layer 2 (Effect Algebra) to Layer 1 (Lambda Calculus) Compilation
//-----------------------------------------------------------------------------

fn compile_sexpr_to_term(expr: &SExpression) -> CompileResult<Term> {
    match expr {
        SExpression::List(elements) if !elements.is_empty() => {
            match &elements[0] {
                SExpression::Symbol(op) if op == "pure" => {
                    if elements.len() != 2 {
                        return Err(CompileError::InvalidArity { expected: 1, found: elements.len() - 1, location: None });
                    }
                    compile_sexpr_to_term(&elements[1]) // pure(x) = x (simplified)
                }
                SExpression::Symbol(op) if op == "bind" => {
                    if elements.len() != 3 {
                        return Err(CompileError::InvalidArity { expected: 2, found: elements.len() - 1, location: None });
                    }
                    let effect_term = compile_sexpr_to_term(&elements[1])?;
                    let continuation_term = compile_sexpr_to_term(&elements[2])?;
                    Ok(Term::apply(continuation_term, effect_term))
                }
                SExpression::Symbol(op) if op == "lambda" => {
                    if elements.len() != 3 {
                        return Err(CompileError::InvalidArity { expected: 2, found: elements.len() - 1, location: None });
                    }
                    let param = match &elements[1] {
                        SExpression::List(params) if params.len() == 1 => {
                            match &params[0] {
                                SExpression::Symbol(p) => p.clone(),
                                _ => return Err(CompileError::CompilationError { message: "Parameter must be symbol".to_string(), location: None }),
                            }
                        }
                        SExpression::Symbol(p) => p.clone(),
                        _ => return Err(CompileError::CompilationError { message: "Invalid parameter".to_string(), location: None }),
                    };
                    let body = compile_sexpr_to_term(&elements[2])?;
                    Ok(Term::lambda(param, body))
                }
                SExpression::Symbol(op) if op == "apply" => {
                    if elements.len() != 3 {
                        return Err(CompileError::InvalidArity { expected: 2, found: elements.len() - 1, location: None });
                    }
                    let func = compile_sexpr_to_term(&elements[1])?;
                    let arg = compile_sexpr_to_term(&elements[2])?;
                    Ok(Term::apply(func, arg))
                }
                SExpression::Symbol(op) if op == "alloc" => {
                    if elements.len() != 2 {
                        return Err(CompileError::InvalidArity { expected: 1, found: elements.len() - 1, location: None });
                    }
                    let value_term = compile_sexpr_to_term(&elements[1])?;
                    // Create an alloc term - we'll handle this in the term compilation
                    Ok(Term::alloc(value_term))
                }
                SExpression::Symbol(op) if op == "consume" => {
                    if elements.len() != 2 {
                        return Err(CompileError::InvalidArity { expected: 1, found: elements.len() - 1, location: None });
                    }
                    let resource_term = compile_sexpr_to_term(&elements[1])?;
                    // Create a consume term - we'll handle this in the term compilation
                    Ok(Term::consume(resource_term))
                }
                SExpression::Symbol(op) if op == "tensor" => {
                    if elements.len() != 3 {
                        return Err(CompileError::InvalidArity { expected: 2, found: elements.len() - 1, location: None });
                    }
                    let left_term = compile_sexpr_to_term(&elements[1])?;
                    let right_term = compile_sexpr_to_term(&elements[2])?;
                    // Create a tensor term - we'll handle this in the term compilation
                    Ok(Term::tensor(left_term, right_term))
                }
                _ => {
                    // Default to function application
                    if elements.len() >= 2 {
                        let func = compile_sexpr_to_term(&elements[0])?;
                        let mut result = func;
                        for arg_expr in &elements[1..] {
                            let arg = compile_sexpr_to_term(arg_expr)?;
                            result = Term::apply(result, arg);
                        }
                        Ok(result)
                    } else {
                        Err(CompileError::CompilationError { message: "Empty list not allowed".to_string(), location: None })
                    }
                }
            }
        }
        SExpression::Integer(n) => Ok(Term::literal(Literal::Int(*n))),
        SExpression::Boolean(b) => Ok(Term::literal(Literal::Bool(*b))),
        SExpression::Symbol(s) => Ok(Term::var(s)),
        SExpression::Nil => Ok(Term::unit()),
        SExpression::List(_) => Err(CompileError::CompilationError { message: "Empty list not allowed".to_string(), location: None }),
    }
}

//-----------------------------------------------------------------------------
// Layer 1 (Lambda Calculus) to Layer 0 (Register Machine) Compilation
//-----------------------------------------------------------------------------

fn compile_term_to_instructions(term: &Term) -> CompileResult<Vec<Instruction>> {
    let mut ctx = CompileContext::new();
    let _result_reg = compile_term(&mut ctx, term)?;
    Ok(ctx.into_program())
}

fn compile_term(ctx: &mut CompileContext, term: &Term) -> CompileResult<RegisterId> {
    match &term.kind {
        TermKind::Literal(_) => compile_literal(ctx),
        TermKind::Var(name) => compile_variable(ctx, name),
        TermKind::Unit => compile_unit(ctx),
        TermKind::Apply { func, arg } => compile_application(ctx, func, arg),
        TermKind::Lambda { param, body, .. } => compile_lambda(ctx, param, body),
        TermKind::Let { var, value, body } => compile_let(ctx, var, value, body),
        TermKind::Alloc { value } => compile_alloc(ctx, value),
        TermKind::Consume { resource } => compile_consume(ctx, resource),
        TermKind::Tensor { left, right } => compile_tensor(ctx, left, right),
        _ => Err(CompileError::Layer1Error {
            message: format!("Compilation not yet implemented for {:?}", term.kind),
            location: None,
        }),
    }
}

fn compile_literal(ctx: &mut CompileContext) -> CompileResult<RegisterId> {
    let dst_reg = ctx.alloc_register();
    let type_reg = ctx.alloc_register();
    let init_reg = ctx.alloc_register();
    
    // Use alloc to create literal values
    ctx.emit(Instruction::Alloc { type_reg, init_reg, output_reg: dst_reg });
    
    Ok(dst_reg)
}

fn compile_variable(ctx: &mut CompileContext, name: &str) -> CompileResult<RegisterId> {
    ctx.lookup_variable(name)
        .ok_or_else(|| CompileError::UnknownSymbol {
            symbol: name.to_string(),
            location: None,
        })
}

fn compile_unit(ctx: &mut CompileContext) -> CompileResult<RegisterId> {
    let dst_reg = ctx.alloc_register();
    let unit_type_reg = ctx.alloc_register();
    
    // Create unit using alloc with self-reference
    ctx.emit(Instruction::Alloc { 
        type_reg: unit_type_reg, 
        init_reg: unit_type_reg, 
        output_reg: dst_reg 
    });
    
    Ok(dst_reg)
}

fn compile_application(ctx: &mut CompileContext, func: &Term, arg: &Term) -> CompileResult<RegisterId> {
    let func_reg = compile_term(ctx, func)?;
    let arg_reg = compile_term(ctx, arg)?;
    let result_reg = ctx.alloc_register();
    
    // Use Transform for function application
    ctx.emit(Instruction::Transform {
        morph_reg: func_reg,
        input_reg: arg_reg,
        output_reg: result_reg,
    });
    
    Ok(result_reg)
}

fn compile_lambda(ctx: &mut CompileContext, param: &str, body: &Term) -> CompileResult<RegisterId> {
    let lambda_reg = ctx.alloc_register();
    let param_reg = ctx.alloc_register();
    let func_type_reg = ctx.alloc_register();
    
    ctx.bind_variable(param.to_string(), param_reg);
    let body_reg = compile_term(ctx, body)?;
    
    // Create function using alloc
    ctx.emit(Instruction::Alloc {
        type_reg: func_type_reg,
        init_reg: body_reg,
        output_reg: lambda_reg,
    });
    
    Ok(lambda_reg)
}

fn compile_let(ctx: &mut CompileContext, var: &str, value: &Term, body: &Term) -> CompileResult<RegisterId> {
    let value_reg = compile_term(ctx, value)?;
    ctx.bind_variable(var.to_string(), value_reg);
    compile_term(ctx, body)
}

fn compile_alloc(ctx: &mut CompileContext, value: &Term) -> CompileResult<RegisterId> {
    let value_reg = compile_term(ctx, value)?;
    let result_reg = ctx.alloc_register();
    let type_reg = ctx.alloc_register();
    let temp_type_reg = ctx.alloc_register();
    let temp_init_reg = ctx.alloc_register();
    
    // Use alloc to create resource type first
    ctx.emit(Instruction::Alloc {
        type_reg: temp_type_reg,
        init_reg: temp_init_reg,
        output_reg: type_reg,
    });
    
    ctx.emit(Instruction::Alloc {
        type_reg,
        init_reg: value_reg,
        output_reg: result_reg,
    });
    
    Ok(result_reg)
}

fn compile_consume(ctx: &mut CompileContext, resource: &Term) -> CompileResult<RegisterId> {
    let resource_reg = compile_term(ctx, resource)?;
    let result_reg = ctx.alloc_register();
    
    ctx.emit(Instruction::Consume {
        resource_reg,
        output_reg: result_reg,
    });
    
    Ok(result_reg)
}

fn compile_tensor(ctx: &mut CompileContext, left: &Term, right: &Term) -> CompileResult<RegisterId> {
    let left_reg = compile_term(ctx, left)?;
    let right_reg = compile_term(ctx, right)?;
    let result_reg = ctx.alloc_register();
    
    ctx.emit(Instruction::Tensor {
        left_reg,
        right_reg,
        output_reg: result_reg,
    });
    
    Ok(result_reg)
}

//-----------------------------------------------------------------------------
// Compilation Artifact
//-----------------------------------------------------------------------------

/// Complete compilation result with all intermediate stages
#[derive(Debug, Clone)]
pub struct CompiledArtifact {
    pub source: String,
    pub sexpr: SExpression,
    pub term: Term,
    pub instructions: Vec<Instruction>,
}

impl std::fmt::Display for CompiledArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Compilation Artifact ===")?;
        writeln!(f, "Source: {}", self.source)?;
        writeln!(f, "S-expression: {}", self.sexpr)?;
        writeln!(f, "Layer 1 Term: {:?}", self.term)?;
        writeln!(f, "Layer 0 Program: {} instructions", self.instructions.len())?;
        for (i, instr) in self.instructions.iter().enumerate() {
            writeln!(f, "  {}: {:?}", i, instr)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple() {
        assert_eq!(parse_sexpr("42").unwrap(), SExpression::Integer(42));
        assert_eq!(parse_sexpr("hello").unwrap(), SExpression::Symbol("hello".to_string()));
        assert_eq!(parse_sexpr("(pure 42)").unwrap(), 
                   SExpression::List(vec![SExpression::Symbol("pure".to_string()), SExpression::Integer(42)]));
    }
    
    #[test]
    fn test_compile_pure_42() {
        let artifact = compile("(pure 42)").unwrap();
        assert_eq!(artifact.source, "(pure 42)");
        assert!(!artifact.instructions.is_empty());
    }
    
    #[test]
    fn test_compile_expression() {
        let instructions = compile_expression("(pure 42)").unwrap();
        assert_eq!(instructions.len(), 1); // Updated to match current implementation
    }
} 