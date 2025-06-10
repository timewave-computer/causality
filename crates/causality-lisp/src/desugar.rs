//! Syntactic Sugar Desugaring for Causality Lisp
//!
//! This module implements desugaring transformations from high-level convenience 
//! forms to the core 11 Layer 1 primitives. This allows developers to write more
//! ergonomic code while maintaining the mathematical purity of the core language.

use crate::ast::{Expr, ExprKind, LispValue, Param};
use causality_core::lambda::Symbol;

/// Desugar high-level constructs to core primitives
pub fn desugar(expr: SugarExpr) -> Expr {
    match expr {
        SugarExpr::Core(core_expr) => core_expr,
        SugarExpr::Let(name, value, body) => desugar_let(name, *value, *body),
        SugarExpr::If(condition, then_branch, else_branch) => {
            desugar_if(*condition, *then_branch, *else_branch)
        }
        SugarExpr::List(elements) => desugar_list(elements),
        SugarExpr::Quote(quoted) => desugar_quote(*quoted),
        SugarExpr::And(left, right) => desugar_and(*left, *right),
        SugarExpr::Or(left, right) => desugar_or(*left, *right),
        SugarExpr::Not(expr) => desugar_not(*expr),
        SugarExpr::Cond(clauses, default) => desugar_cond(clauses, default.map(|d| *d)),
    }
}

/// High-level expression forms that desugar to core primitives
#[derive(Debug, Clone, PartialEq)]
pub enum SugarExpr {
    // Core expressions pass through unchanged
    Core(Expr),
    
    // Convenience forms that desugar
    Let(Symbol, Box<SugarExpr>, Box<SugarExpr>),
    If(Box<SugarExpr>, Box<SugarExpr>, Box<SugarExpr>),
    List(Vec<SugarExpr>),
    Quote(Box<SugarExpr>),
    
    // Logical operators
    And(Box<SugarExpr>, Box<SugarExpr>),
    Or(Box<SugarExpr>, Box<SugarExpr>),
    Not(Box<SugarExpr>),
    
    // Conditional chains
    Cond(Vec<(SugarExpr, SugarExpr)>, Option<Box<SugarExpr>>),
}

/// Desugar let binding: `let x = e1 in e2` → `(λx. e2) e1`
fn desugar_let(name: Symbol, value: SugarExpr, body: SugarExpr) -> Expr {
    let desugared_value = desugar(value);
    let desugared_body = desugar(body);
    
    // (λx. body) value
    Expr::apply(
        Expr::lambda(vec![Param::new(name)], desugared_body),
        vec![desugared_value],
    )
}

/// Desugar if expression to case analysis on boolean values
/// `if c then e1 else e2` → `case (bool_to_sum c) of inl _ => e1 | inr _ => e2`
fn desugar_if(condition: SugarExpr, then_branch: SugarExpr, else_branch: SugarExpr) -> Expr {
    let desugared_condition = desugar(condition);
    let desugared_then = desugar(then_branch);
    let desugared_else = desugar(else_branch);
    
    // Convert boolean to sum type for case analysis
    let bool_to_sum = Expr::apply(
        Expr::lambda(
            vec![Param::new("b")],
            Expr::case(
                Expr::variable("b"),
                "_",
                Expr::inl(Expr::unit()),  // true → inl()
                "_", 
                Expr::inr(Expr::unit()),  // false → inr()
            )
        ),
        vec![desugared_condition],
    );
    
    // Case analysis on the sum
    Expr::case(
        bool_to_sum,
        "_",
        desugared_then,
        "_",
        desugared_else,
    )
}

/// Desugar list literals to nested tensor construction
/// `[e1, e2, e3]` → `tensor e1 (tensor e2 (tensor e3 unit))`
fn desugar_list(elements: Vec<SugarExpr>) -> Expr {
    if elements.is_empty() {
        // Empty list is just unit
        Expr::unit()
    } else {
        // Fold right to create nested tensors
        elements.into_iter()
            .map(desugar)
            .rev()
            .fold(Expr::unit(), |acc, elem| Expr::tensor(elem, acc))
    }
}

/// Desugar quote for simple literals
/// `'42` → `42`, `'symbol` → `symbol`, etc.
fn desugar_quote(quoted: SugarExpr) -> Expr {
    match quoted {
        SugarExpr::Core(Expr { kind: ExprKind::Const(value), .. }) => {
            // Already a literal, return as-is
            Expr::constant(value)
        }
        SugarExpr::Core(Expr { kind: ExprKind::Var(symbol), .. }) => {
            // Variable becomes symbol literal
            Expr::constant(LispValue::Symbol(symbol))
        }
        _ => {
            // For now, just return the desugared form
            // Full quote would require more sophisticated handling
            desugar(quoted)
        }
    }
}

/// Desugar logical AND: `and e1 e2` → `if e1 then e2 else false`
fn desugar_and(left: SugarExpr, right: SugarExpr) -> Expr {
    desugar_if(
        left,
        right,
        SugarExpr::Core(Expr::constant(LispValue::Bool(false))),
    )
}

/// Desugar logical OR: `or e1 e2` → `if e1 then true else e2`
fn desugar_or(left: SugarExpr, right: SugarExpr) -> Expr {
    desugar_if(
        left,
        SugarExpr::Core(Expr::constant(LispValue::Bool(true))),
        right,
    )
}

/// Desugar logical NOT: `not e` → `if e then false else true`
fn desugar_not(expr: SugarExpr) -> Expr {
    desugar_if(
        expr,
        SugarExpr::Core(Expr::constant(LispValue::Bool(false))),
        SugarExpr::Core(Expr::constant(LispValue::Bool(true))),
    )
}

/// Desugar conditional chains: `cond [(c1, e1), (c2, e2)] default`
/// → `if c1 then e1 else (if c2 then e2 else default)`
fn desugar_cond(clauses: Vec<(SugarExpr, SugarExpr)>, default: Option<SugarExpr>) -> Expr {
    let mut result = default
        .map(desugar)
        .unwrap_or_else(|| Expr::constant(LispValue::Unit));
    
    // Build nested if expressions from right to left
    for (condition, value) in clauses.into_iter().rev() {
        result = desugar_if(condition, value, SugarExpr::Core(result));
    }
    
    result
}

/// Helper functions for building sugar expressions
impl SugarExpr {
    /// Wrap a core expression
    pub fn core(expr: Expr) -> Self {
        SugarExpr::Core(expr)
    }
    
    /// Create a let expression
    pub fn let_expr(name: impl Into<Symbol>, value: SugarExpr, body: SugarExpr) -> Self {
        SugarExpr::Let(name.into(), Box::new(value), Box::new(body))
    }
    
    /// Create an if expression
    pub fn if_expr(condition: SugarExpr, then_branch: SugarExpr, else_branch: SugarExpr) -> Self {
        SugarExpr::If(Box::new(condition), Box::new(then_branch), Box::new(else_branch))
    }
    
    /// Create a list expression
    pub fn list(elements: Vec<SugarExpr>) -> Self {
        SugarExpr::List(elements)
    }
    
    /// Create a quote expression
    pub fn quote(expr: SugarExpr) -> Self {
        SugarExpr::Quote(Box::new(expr))
    }
    
    /// Create an and expression
    pub fn and(left: SugarExpr, right: SugarExpr) -> Self {
        SugarExpr::And(Box::new(left), Box::new(right))
    }
    
    /// Create an or expression
    pub fn or(left: SugarExpr, right: SugarExpr) -> Self {
        SugarExpr::Or(Box::new(left), Box::new(right))
    }
    
    /// Create a not expression
    #[allow(clippy::should_implement_trait)]
    pub fn not(expr: SugarExpr) -> Self {
        SugarExpr::Not(Box::new(expr))
    }
    
    /// Create a conditional chain
    pub fn cond(clauses: Vec<(SugarExpr, SugarExpr)>, default: Option<SugarExpr>) -> Self {
        SugarExpr::Cond(clauses, default.map(Box::new))
    }
}

/// Main entry point for desugaring expressions
/// For now, core expressions pass through unchanged, but this provides
/// extensibility for future sugar constructs
pub fn desugar_expr(expr: &Expr) -> Result<Expr, String> {
    // For now, all our core expressions pass through unchanged
    // In the future, we could detect sugar patterns in the core AST
    // and transform them here
    Ok(expr.clone())
}

/// Internal desugaring for sugar expressions (when we have them)
pub fn desugar_sugar(sugar: &SugarExpr) -> Result<Expr, String> {
    Ok(desugar(sugar.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, ExprKind, LispValue};
    use causality_core::lambda::Symbol;

    #[test]
    fn test_desugar_let() {
        // Create a simple let expression: let x = 5 in x
        let let_expr = SugarExpr::Let(
            Symbol::new("x"),
            Box::new(SugarExpr::Core(Expr::constant(LispValue::Int(5)))),
            Box::new(SugarExpr::Core(Expr::variable("x"))),
        );

        let result = desugar(let_expr);

        // Should desugar to: (λx. x) 5
        match result.kind {
            ExprKind::Apply(func, args) => {
                assert_eq!(args.len(), 1);
                match &func.kind {
                    ExprKind::Lambda(params, _) => {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].name.as_str(), "x");
                    }
                    _ => panic!("Expected lambda function"),
                }
                match &args[0].kind {
                    ExprKind::Const(LispValue::Int(5)) => (),
                    _ => panic!("Expected constant 5"),
                }
            }
            _ => panic!("Expected function application"),
        }
    }

    #[test]
    fn test_desugar_if() {
        // Create an if expression: if true then 1 else 2
        let if_expr = SugarExpr::If(
            Box::new(SugarExpr::Core(Expr::constant(LispValue::Bool(true)))),
            Box::new(SugarExpr::Core(Expr::constant(LispValue::Int(1)))),
            Box::new(SugarExpr::Core(Expr::constant(LispValue::Int(2)))),
        );

        let result = desugar(if_expr);

        // Should desugar to case analysis
        match result.kind {
            ExprKind::Case(_, _, _, _, _) => {
                // Correct structure - if desugars to case
            }
            _ => panic!("Expected case expression, got {:?}", result.kind),
        }
    }

    #[test]
    fn test_desugar_list() {
        // Create a list: [1, 2, 3]
        let list_expr = SugarExpr::List(vec![
            SugarExpr::Core(Expr::constant(LispValue::Int(1))),
            SugarExpr::Core(Expr::constant(LispValue::Int(2))),
            SugarExpr::Core(Expr::constant(LispValue::Int(3))),
        ]);

        let result = desugar(list_expr);

        // Should desugar to nested tensors: (1, (2, (3, unit)))
        match result.kind {
            ExprKind::Tensor(first, rest) => {
                match &first.kind {
                    ExprKind::Const(LispValue::Int(1)) => (),
                    _ => panic!("Expected first element to be 1"),
                }
                // Check it's properly nested
                match &rest.kind {
                    ExprKind::Tensor(_, _) => (),
                    _ => panic!("Expected nested tensor structure"),
                }
            }
            _ => panic!("Expected tensor expression"),
        }
    }

    #[test]
    fn test_desugar_empty_list() {
        // Create an empty list: []
        let empty_list = SugarExpr::List(vec![]);

        let result = desugar(empty_list);

        // Should desugar to unit
        match result.kind {
            ExprKind::UnitVal => (),
            _ => panic!("Expected unit value for empty list"),
        }
    }

    #[test]
    fn test_desugar_nested_sugar() {
        // Create nested sugar: let x = [1, 2] in if x then 1 else 2
        let nested_expr = SugarExpr::Let(
            Symbol::new("x"),
            Box::new(SugarExpr::List(vec![
                SugarExpr::Core(Expr::constant(LispValue::Int(1))),
                SugarExpr::Core(Expr::constant(LispValue::Int(2))),
            ])),
            Box::new(SugarExpr::If(
                Box::new(SugarExpr::Core(Expr::variable("x"))),
                Box::new(SugarExpr::Core(Expr::constant(LispValue::Int(1)))),
                Box::new(SugarExpr::Core(Expr::constant(LispValue::Int(2)))),
            )),
        );

        let result = desugar(nested_expr);

        // Should successfully desugar all nested constructs
        match result.kind {
            ExprKind::Apply(_, _) => {
                // Let desugars to application - correct
            }
            _ => panic!("Expected top-level application from let desugaring"),
        }
    }

    #[test]
    fn test_desugar_expr_integration() {
        // Test the main desugar_expr function
        let core_expr = Expr::constant(LispValue::Int(42));
        let result = desugar_expr(&core_expr).unwrap();

        // Core expressions should pass through unchanged
        match result.kind {
            ExprKind::Const(LispValue::Int(42)) => (),
            _ => panic!("Core expression should pass through unchanged"),
        }
    }

    #[test] 
    fn test_complex_list_nesting() {
        // Test deeply nested list: [[1, 2], [3, 4]]
        let nested_list = SugarExpr::List(vec![
            SugarExpr::List(vec![
                SugarExpr::Core(Expr::constant(LispValue::Int(1))),
                SugarExpr::Core(Expr::constant(LispValue::Int(2))),
            ]),
            SugarExpr::List(vec![
                SugarExpr::Core(Expr::constant(LispValue::Int(3))),
                SugarExpr::Core(Expr::constant(LispValue::Int(4))),
            ]),
        ]);

        let result = desugar(nested_list);

        // Should produce a properly nested tensor structure
        match result.kind {
            ExprKind::Tensor(_, _) => {
                // Correct - nested list becomes nested tensors
            }
            _ => panic!("Expected tensor structure for nested list"),
        }
    }
} 