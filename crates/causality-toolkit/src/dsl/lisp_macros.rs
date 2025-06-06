//! Ergonomic macros for constructing Causality Lisp expressions
//!
//! This module provides Rust macros that make it easier to construct
//! Causality Lisp AST expressions from Rust code.

// Note: This file contains only macros, so normal imports aren't needed
// The macros reference types directly in the expansion

/// Create a lambda expression with ergonomic syntax
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::{lambda, var};
/// 
/// let lambda_expr = lambda!("x", var!("x"));
/// ```
#[macro_export]
macro_rules! lambda {
    ($param:expr, $body:expr) => {
        causality_lisp::ast::Expr::lambda(
            vec![causality_lisp::ast::Param::new($param)],
            $body,
        )
    };
}

/// Create a function application expression
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::{app, var, int_val};
/// 
/// let app_expr = app!(var!("f"), int_val!(42));
/// ```
#[macro_export]
macro_rules! app {
    ($func:expr, $arg:expr) => {
        causality_lisp::ast::Expr::apply($func, vec![$arg])
    };
}

/// Create a let binding expression (using let-tensor for now)
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::{let_bind, int_val, var};
/// 
/// let let_expr = let_bind!("x", int_val!(42), var!("x"));
/// ```
#[macro_export]
macro_rules! let_bind {
    ($var:expr, $value:expr, $body:expr) => {
        // For simplicity, use alloc to create a resource and consume it in the body
        {
            let resource = causality_lisp::ast::Expr::alloc($value);
            let bound_body = $body;
            causality_lisp::ast::Expr::let_unit(resource, bound_body)
        }
    };
}

/// Create a conditional if expression (using case on sum types)
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::{if_expr, bool_val, int_val, var};
/// 
/// let if_expression = if_expr!(var!("x"), int_val!(1), int_val!(0));
/// ```
#[macro_export]
macro_rules! if_expr {
    ($cond:expr, $then_expr:expr, $else_expr:expr) => {
        causality_lisp::ast::Expr::case(
            $cond,
            "true_branch",
            $then_expr,
            "false_branch",
            $else_expr,
        )
    };
}

/// Create a variable reference
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::var;
/// 
/// let var_expr = var!("my_variable");
/// ```
#[macro_export]
macro_rules! var {
    ($name:expr) => {
        causality_lisp::ast::Expr::variable($name)
    };
}

/// Create an integer constant
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::int_val;
/// 
/// let int_expr = int_val!(42);
/// ```
#[macro_export]
macro_rules! int_val {
    ($val:expr) => {
        causality_lisp::ast::Expr::constant(causality_lisp::ast::LispValue::Int($val))
    };
}

/// Create a boolean constant
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::bool_val;
/// 
/// let bool_expr = bool_val!(true);
/// ```
#[macro_export]
macro_rules! bool_val {
    ($val:expr) => {
        causality_lisp::ast::Expr::constant(causality_lisp::ast::LispValue::Bool($val))
    };
}

/// Create a string constant
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::string_val;
/// 
/// let string_expr = string_val!("hello");
/// ```
#[macro_export]
macro_rules! string_val {
    ($val:expr) => {
        causality_lisp::ast::Expr::constant(causality_lisp::ast::LispValue::String(
            causality_core::system::content_addressing::Str::new($val)
        ))
    };
}

/// Create a unit constant
/// 
/// # Examples
/// 
/// ```
/// use causality_toolkit::unit_val;
/// 
/// let unit_expr = unit_val!();
/// ```
#[macro_export]
macro_rules! unit_val {
    () => {
        causality_lisp::ast::Expr::unit()
    };
}

// Re-export the macros for easier access
pub use lambda;
pub use app;
pub use let_bind;
pub use if_expr;
pub use var;
pub use int_val;
pub use bool_val;
pub use string_val;
pub use unit_val;

#[cfg(test)]
mod tests {
    use super::*;
    use causality_lisp::ast::{LispValue, ExprKind};

    #[test]
    fn test_lambda_macro() {
        let lambda_expr = lambda!("x", var!("x"));
        
        match &lambda_expr.kind {
            ExprKind::Lambda(params, body) => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name.as_str(), "x");
                assert!(matches!(body.kind, ExprKind::Var(_)));
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_app_macro() {
        let func = var!("f");
        let arg = int_val!(42);
        let app_expr = app!(func, arg);
        
        match &app_expr.kind {
            ExprKind::Apply(func, args) => {
                assert!(matches!(func.kind, ExprKind::Var(_)));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].kind, ExprKind::Const(LispValue::Int(42))));
            }
            _ => panic!("Expected Apply expression"),
        }
    }

    #[test]
    fn test_let_bind_macro() {
        let value = int_val!(42);
        let body = var!("x");
        let let_expr = let_bind!("x", value, body);
        
        // let_bind creates a let_unit with alloc, so check for that structure
        match &let_expr.kind {
            ExprKind::LetUnit(resource, body) => {
                assert!(matches!(resource.kind, ExprKind::Alloc(_)));
                assert!(matches!(body.kind, ExprKind::Var(_)));
            }
            _ => panic!("Expected LetUnit expression"),
        }
    }

    #[test]
    fn test_if_expr_macro() {
        let cond = bool_val!(true);
        let then_expr = int_val!(1);
        let else_expr = int_val!(0);
        let if_expression = if_expr!(cond, then_expr, else_expr);
        
        match &if_expression.kind {
            ExprKind::Case(cond, _, then_branch, _, else_branch) => {
                assert!(matches!(cond.kind, ExprKind::Const(LispValue::Bool(true))));
                assert!(matches!(then_branch.kind, ExprKind::Const(LispValue::Int(1))));
                assert!(matches!(else_branch.kind, ExprKind::Const(LispValue::Int(0))));
            }
            _ => panic!("Expected Case expression"),
        }
    }

    #[test]
    fn test_var_macro() {
        let var_expr = var!("test_var");
        
        match &var_expr.kind {
            ExprKind::Var(symbol) => {
                assert_eq!(symbol.as_str(), "test_var");
            }
            _ => panic!("Expected Var expression"),
        }
    }

    #[test]
    fn test_value_macros() {
        let int_expr = int_val!(42);
        let bool_expr = bool_val!(true);
        let string_expr = string_val!("hello");
        let unit_expr = unit_val!();
        
        assert!(matches!(int_expr.kind, ExprKind::Const(LispValue::Int(42))));
        assert!(matches!(bool_expr.kind, ExprKind::Const(LispValue::Bool(true))));
        assert!(matches!(string_expr.kind, ExprKind::Const(LispValue::String(_))));
        assert!(matches!(unit_expr.kind, ExprKind::UnitVal));
    }

    #[test]
    fn test_complex_expression() {
        // Create: (lambda x => if x then 1 else 0)
        let lambda_body = if_expr!(
            var!("x"),
            int_val!(1),
            int_val!(0)
        );
        let lambda_expr = lambda!("x", lambda_body);
        
        // Apply it to true: ((lambda x => if x then 1 else 0) true)
        let application = app!(lambda_expr, bool_val!(true));
        
        match &application.kind {
            ExprKind::Apply(func, args) => {
                assert!(matches!(func.kind, ExprKind::Lambda(_, _)));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].kind, ExprKind::Const(LispValue::Bool(true))));
            }
            _ => panic!("Expected application"),
        }
    }
} 