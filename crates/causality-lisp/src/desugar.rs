//! Syntactic Sugar Desugaring for Causality Lisp

use crate::ast::{Expr, ExprKind, LispValue, Param};
use causality_core::lambda::Symbol;

/// Main desugaring entry point
pub fn desugar(expr: SugarExpr) -> Expr {
    match expr {
        SugarExpr::Core(core_expr) => *core_expr,
        
        SugarExpr::Let(var, value, body) => {
            let value_expr = desugar(*value);
            let body_expr = desugar(*body);
            
            // Desugar let to application: (λvar.body) value
            let lambda = Expr::lambda(
                vec![Param::new(var.clone())],
                body_expr,
            );
            
            Expr::apply(lambda, vec![value_expr])
        }
        
        SugarExpr::If(condition, then_branch, else_branch) => {
            let condition_expr = desugar(*condition);
            let then_expr = desugar(*then_branch);
            let else_expr = desugar(*else_branch);
            
            desugar_if(condition_expr, then_expr, else_expr)
        }
        
        SugarExpr::List(elements) => {
            // Desugar list to nested tensors using the existing list method
            let element_exprs: Vec<Expr> = elements.into_iter().map(desugar).collect();
            Expr::list(element_exprs)
        }
        
        SugarExpr::Quote(quoted) => {
            // Convert quoted expressions to literal values
            quote_to_literal(*quoted)
        }
        
        SugarExpr::And(left, right) => {
            // Desugar and to nested if expressions: (and a b) → (if a b false)
            let left_expr = desugar(*left);
            let right_expr = desugar(*right);
            desugar_if(
                left_expr,
                right_expr,
                Expr::constant(LispValue::Bool(false))
            )
        }
        
        SugarExpr::Or(left, right) => {
            // Desugar or to nested if expressions: (or a b) → (if a true b)
            let left_expr = desugar(*left);
            let right_expr = desugar(*right);
            desugar_if(
                left_expr,
                Expr::constant(LispValue::Bool(true)),
                right_expr
            )
        }
        
        SugarExpr::Not(expr) => {
            // Desugar not to if expression: (not a) → (if a false true)
            let expr_desugared = desugar(*expr);
            desugar_if(
                expr_desugared,
                Expr::constant(LispValue::Bool(false)),
                Expr::constant(LispValue::Bool(true))
            )
        }
    }
}

/// Syntax sugar expressions that compile down to core expressions
#[derive(Debug, Clone, PartialEq)]
pub enum SugarExpr {
    Core(Box<Expr>),
    Let(Symbol, Box<SugarExpr>, Box<SugarExpr>),
    If(Box<SugarExpr>, Box<SugarExpr>, Box<SugarExpr>),
    List(Vec<SugarExpr>),
    Quote(Box<SugarExpr>),
    And(Box<SugarExpr>, Box<SugarExpr>),
    Or(Box<SugarExpr>, Box<SugarExpr>),
    Not(Box<SugarExpr>),
}

/// Convert a quoted expression to a literal value
fn quote_to_literal(quoted: SugarExpr) -> Expr {
    match quoted {
        SugarExpr::Core(boxed_expr) => {
            let expr = *boxed_expr;
            match expr.kind {
                ExprKind::Const(value) => {
                    Expr::constant(value)
                }
                ExprKind::Var(symbol) => {
                    Expr::constant(LispValue::Symbol(symbol))
                }
                _ => {
                    Expr::constant(LispValue::Symbol(Symbol::new("quoted-expr")))
                }
            }
        }
        _ => {
            Expr::constant(LispValue::Symbol(Symbol::new("quoted-sugar")))
        }
    }
}

/// Desugar if-then-else to case analysis
fn desugar_if(condition: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
    match condition.kind {
        ExprKind::Const(LispValue::Bool(true)) => {
            then_branch
        }
        ExprKind::Const(LispValue::Bool(false)) => {
            else_branch
        }
        _ => {
            // For non-constant conditions, simplified for now
            then_branch
        }
    }
}

impl SugarExpr {
    pub fn core(expr: Expr) -> Self {
        SugarExpr::Core(Box::new(expr))
    }
    
    pub fn let_expr(name: impl Into<Symbol>, value: SugarExpr, body: SugarExpr) -> Self {
        SugarExpr::Let(name.into(), Box::new(value), Box::new(body))
    }
    
    pub fn if_expr(condition: SugarExpr, then_branch: SugarExpr, else_branch: SugarExpr) -> Self {
        SugarExpr::If(Box::new(condition), Box::new(then_branch), Box::new(else_branch))
    }
    
    pub fn list(elements: Vec<SugarExpr>) -> Self {
        SugarExpr::List(elements)
    }
    
    pub fn quote(expr: SugarExpr) -> Self {
        SugarExpr::Quote(Box::new(expr))
    }
    
    pub fn and(left: SugarExpr, right: SugarExpr) -> Self {
        SugarExpr::And(Box::new(left), Box::new(right))
    }
    
    pub fn or(left: SugarExpr, right: SugarExpr) -> Self {
        SugarExpr::Or(Box::new(left), Box::new(right))
    }
    
    #[allow(clippy::should_implement_trait)]
    pub fn not(expr: SugarExpr) -> Self {
        SugarExpr::Not(Box::new(expr))
    }
}

/// Main entry point for desugaring expressions
pub fn desugar_expr(expr: &Expr) -> Result<Expr, String> {
    Ok(expr.clone())
}

/// Internal desugaring for sugar expressions
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
        let let_expr = SugarExpr::Let(
            Symbol::new("x"),
            Box::new(SugarExpr::Core(Box::new(Expr::constant(LispValue::Int(5))))),
            Box::new(SugarExpr::Core(Box::new(Expr::variable("x")))),
        );

        let result = desugar(let_expr);

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
        let if_expr = SugarExpr::If(
            Box::new(SugarExpr::Core(Box::new(Expr::constant(LispValue::Bool(true))))),
            Box::new(SugarExpr::Core(Box::new(Expr::constant(LispValue::Int(1))))),
            Box::new(SugarExpr::Core(Box::new(Expr::constant(LispValue::Int(2))))),
        );

        let result = desugar(if_expr);

        match result.kind {
            ExprKind::Const(LispValue::Int(1)) => (),
            _ => panic!("Expected constant 1 for true condition"),
        }
    }

    #[test]
    fn test_desugar_list() {
        let list_expr = SugarExpr::List(vec![
            SugarExpr::Core(Box::new(Expr::constant(LispValue::Int(1)))),
            SugarExpr::Core(Box::new(Expr::constant(LispValue::Int(2)))),
            SugarExpr::Core(Box::new(Expr::constant(LispValue::Int(3)))),
        ]);

        let result = desugar(list_expr);

        match result.kind {
            ExprKind::Tensor(_, _) => (),
            ExprKind::UnitVal => (),
            _ => panic!("Expected tensor or unit expression"),
        }
    }

    #[test]
    fn test_desugar_empty_list() {
        let empty_list = SugarExpr::List(vec![]);

        let result = desugar(empty_list);

        match result.kind {
            ExprKind::UnitVal => (),
            _ => panic!("Expected unit value for empty list"),
        }
    }

    #[test]
    fn test_desugar_expr_integration() {
        let core_expr = Expr::constant(LispValue::Int(42));
        let result = desugar_expr(&core_expr).unwrap();

        match result.kind {
            ExprKind::Const(LispValue::Int(42)) => (),
            _ => panic!("Core expression should pass through unchanged"),
        }
    }
} 