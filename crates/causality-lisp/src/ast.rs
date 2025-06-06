//! Abstract Syntax Tree for Causality Lisp
//!
//! This module defines the AST types for the Causality Lisp language,
//! containing exactly the 11 Layer 1 primitives that form the mathematical
//! foundation of the linear lambda calculus with algebraic data types.

use causality_core::{
    lambda::{base::{TypeInner, Value as CoreValue}, Symbol},
    system::content_addressing::{EntityId, Str},
};
use std::collections::HashMap;

/// Main expression type
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Option<TypeInner>,
    pub span: Option<Span>,
}

/// Expression kinds - exactly 11 core Layer 1 primitives
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // Literals and variables
    Const(LispValue),
    Var(Symbol),
    
    // Unit Type (Terminal Object)
    UnitVal,
    LetUnit(Box<Expr>, Box<Expr>),
    
    // Tensor Product (Monoidal Product ⊗)
    Tensor(Box<Expr>, Box<Expr>),
    LetTensor(Box<Expr>, Symbol, Symbol, Box<Expr>),
    
    // Sum Type (Coproduct ⊕)
    Inl(Box<Expr>),
    Inr(Box<Expr>),
    Case(Box<Expr>, Symbol, Box<Expr>, Symbol, Box<Expr>),
    
    // Linear Functions (Internal Hom ⊸)
    Lambda(Vec<Param>, Box<Expr>),
    Apply(Box<Expr>, Vec<Expr>),
    
    // Resource Management
    Alloc(Box<Expr>),
    Consume(Box<Expr>),
    
    // Record Operations (for capability checking)
    RecordAccess {
        record: Box<Expr>,
        field: String,
    },
    RecordUpdate {
        record: Box<Expr>,
        field: String,
        value: Box<Expr>,
    },
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Symbol,
    pub ty: Option<Symbol>,
}

/// Source location information
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// Lisp values - runtime representation
#[derive(Debug, Clone, PartialEq)]
pub enum LispValue {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Str),
    Symbol(Symbol),
    List(Vec<LispValue>),
    Map(HashMap<Symbol, LispValue>),
    Record(HashMap<Symbol, LispValue>),
    ResourceId(EntityId),
    ExprId(EntityId),
    CoreValue(CoreValue), // Integration with core Value system
}

impl Expr {
    /// Create a new expression with no type information
    pub fn new(kind: ExprKind) -> Self {
        Self {
            kind,
            ty: None,
            span: None,
        }
    }
    
    /// Create a new expression with type information
    pub fn with_type(kind: ExprKind, ty: TypeInner) -> Self {
        Self {
            kind,
            ty: Some(ty),
            span: None,
        }
    }
    
    /// Create a new expression with span information
    pub fn with_span(kind: ExprKind, span: Span) -> Self {
        Self {
            kind,
            ty: None,
            span: Some(span),
        }
    }
    
    /// Create a constant expression
    pub fn constant(value: LispValue) -> Self {
        Self::new(ExprKind::Const(value))
    }
    
    /// Create a variable expression
    pub fn variable(name: impl Into<Symbol>) -> Self {
        Self::new(ExprKind::Var(name.into()))
    }
    
    /// Create the unit value
    pub fn unit() -> Self {
        Self::new(ExprKind::UnitVal)
    }
    
    /// Create a let-unit expression
    pub fn let_unit(unit_expr: Expr, body: Expr) -> Self {
        Self::new(ExprKind::LetUnit(Box::new(unit_expr), Box::new(body)))
    }
    
    /// Create a tensor (pair) expression
    pub fn tensor(left: Expr, right: Expr) -> Self {
        Self::new(ExprKind::Tensor(Box::new(left), Box::new(right)))
    }
    
    /// Create a tensor elimination expression
    pub fn let_tensor(
        pair_expr: Expr,
        left_var: impl Into<Symbol>,
        right_var: impl Into<Symbol>,
        body: Expr,
    ) -> Self {
        Self::new(ExprKind::LetTensor(
            Box::new(pair_expr),
            left_var.into(),
            right_var.into(),
            Box::new(body),
        ))
    }
    
    /// Create a left injection for sum types
    pub fn inl(expr: Expr) -> Self {
        Self::new(ExprKind::Inl(Box::new(expr)))
    }
    
    /// Create a right injection for sum types
    pub fn inr(expr: Expr) -> Self {
        Self::new(ExprKind::Inr(Box::new(expr)))
    }
    
    /// Create a case expression for sum types
    pub fn case(
        expr: Expr,
        left_var: impl Into<Symbol>,
        left_branch: Expr,
        right_var: impl Into<Symbol>,
        right_branch: Expr,
    ) -> Self {
        Self::new(ExprKind::Case(
            Box::new(expr),
            left_var.into(),
            Box::new(left_branch),
            right_var.into(),
            Box::new(right_branch),
        ))
    }
    
    /// Create a lambda expression
    pub fn lambda(params: Vec<Param>, body: Expr) -> Self {
        Self::new(ExprKind::Lambda(params, Box::new(body)))
    }
    
    /// Create a function application
    pub fn apply(func: Expr, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::Apply(Box::new(func), args))
    }
    
    /// Create a resource allocation expression
    pub fn alloc(value: Expr) -> Self {
        Self::new(ExprKind::Alloc(Box::new(value)))
    }
    
    /// Create a resource consumption expression
    pub fn consume(resource: Expr) -> Self {
        Self::new(ExprKind::Consume(Box::new(resource)))
    }
    
    /// Create a record field access expression
    pub fn record_access(record: Expr, field: impl Into<String>) -> Self {
        Self::new(ExprKind::RecordAccess {
            record: Box::new(record),
            field: field.into(),
        })
    }
    
    /// Create a record field update expression
    pub fn record_update(record: Expr, field: impl Into<String>, value: Expr) -> Self {
        Self::new(ExprKind::RecordUpdate {
            record: Box::new(record),
            field: field.into(),
            value: Box::new(value),
        })
    }
    
    /// Create a list expression (using nested tensors)
    pub fn list(elements: Vec<Expr>) -> Self {
        if elements.is_empty() {
            Self::unit()
        } else {
            // Fold right to create nested tensors: [a, b, c] → tensor a (tensor b (tensor c unit))
            elements.into_iter()
                .rev()
                .fold(Self::unit(), |acc, elem| Self::tensor(elem, acc))
        }
    }
}

impl Param {
    /// Create a new parameter without type annotation
    pub fn new(name: impl Into<Symbol>) -> Self {
        Self {
            name: name.into(),
            ty: None,
        }
    }
    
    /// Create a new parameter with type annotation
    pub fn with_type(name: impl Into<Symbol>, ty: impl Into<Symbol>) -> Self {
        Self {
            name: name.into(),
            ty: Some(ty.into()),
        }
    }
}

impl LispValue {
    /// Check if value is considered "truthy" in conditional contexts
    pub fn is_truthy(&self) -> bool {
        match self {
            LispValue::Unit => false,
            LispValue::Bool(b) => *b,
            LispValue::Int(i) => *i != 0,
            LispValue::Float(f) => *f != 0.0,
            LispValue::String(s) => !s.value.is_empty(),
            LispValue::Symbol(_) => true,
            LispValue::List(l) => !l.is_empty(),
            LispValue::Map(m) => !m.is_empty(),
            LispValue::Record(r) => !r.is_empty(),
            LispValue::ResourceId(_) => true,
            LispValue::ExprId(_) => true,
            LispValue::CoreValue(_) => true,
        }
    }
    
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            LispValue::Unit => "Unit",
            LispValue::Bool(_) => "Bool",
            LispValue::Int(_) => "Int",
            LispValue::Float(_) => "Float",
            LispValue::String(_) => "String",
            LispValue::Symbol(_) => "Symbol",
            LispValue::List(_) => "List",
            LispValue::Map(_) => "Map",
            LispValue::Record(_) => "Record",
            LispValue::ResourceId(_) => "ResourceId",
            LispValue::ExprId(_) => "ExprId",
            LispValue::CoreValue(_) => "CoreValue",
        }
    }
}

// Helper functions for creating common expressions
pub mod helpers {
    use super::*;
    
    /// Create an integer constant
    pub fn int(value: i64) -> Expr {
        Expr::constant(LispValue::Int(value))
    }
    
    /// Create a boolean constant
    pub fn bool(value: bool) -> Expr {
        Expr::constant(LispValue::Bool(value))
    }
    
    /// Create a string constant
    pub fn string(value: impl Into<Str>) -> Expr {
        Expr::constant(LispValue::String(value.into()))
    }
    
    /// Create a symbol constant
    pub fn symbol(value: impl Into<Symbol>) -> Expr {
        Expr::constant(LispValue::Symbol(value.into()))
    }
    
    /// Create a unit constant
    pub fn unit() -> Expr {
        Expr::unit()
    }
} 