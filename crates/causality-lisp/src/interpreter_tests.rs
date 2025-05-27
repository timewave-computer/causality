#[cfg(test)]
mod tests {
    use crate::context::DefaultExprContext;
    use crate::core::{Evaluable, ExprContextual};
    use crate::parser::parse;
    use crate::Interpreter;
    use causality_types::primitive::string::Str;
    use causality_types::expr::ast::{Atom, AtomicCombinator, Expr};
    use causality_types::expr::result::ExprError;
    use causality_types::expr::value::{ValueExpr, ValueExprMap, ValueExprVec};
    use std::collections::BTreeMap;
    use causality_types::primitive::number::Number;

    // Helper function to create a simple context with predefined values
    fn create_test_context() -> DefaultExprContext {
        let mut ctx = DefaultExprContext::new();
        
        // Add some variables to the context
        ctx.add_binding("x", ValueExpr::Number(Number::Integer(10)));
        ctx.add_binding("y", ValueExpr::Number(Number::Integer(20)));
        ctx.add_binding("name", ValueExpr::String(Str::from("test")));
        
        ctx
    }

    #[tokio::test]
    async fn test_evaluate_primitives() {
        let interpreter = Interpreter::new();
        let ctx = DefaultExprContext::new();
        
        // Test evaluating integer literal
        let int_expr = Expr::Atom(Atom::Integer(42));
        let result = interpreter.evaluate_expr(&int_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(42)));
        
        // Test evaluating string literal
        let str_expr = Expr::Atom(Atom::String(Str::from("hello")));
        let result = interpreter.evaluate_expr(&str_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::String(Str::from("hello")));
        
        // Test evaluating boolean literal
        let bool_expr = Expr::Atom(Atom::Boolean(true));
        let result = interpreter.evaluate_expr(&bool_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Bool(true));
        
        // Test evaluating nil literal
        let nil_expr = Expr::Atom(Atom::Nil);
        let result = interpreter.evaluate_expr(&nil_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Unit);
    }

    #[tokio::test]
    async fn test_evaluate_variables() {
        let interpreter = Interpreter::new();
        let ctx = create_test_context();
        
        // Test evaluating variable 'x'
        let var_expr = Expr::Var(Str::from("x"));
        let result = interpreter.evaluate_expr(&var_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(10)));
        
        // Test evaluating variable 'name'
        let var_expr = Expr::Var(Str::from("name"));
        let result = interpreter.evaluate_expr(&var_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::String(Str::from("test")));
        
        // Test evaluating undefined variable
        let var_expr = Expr::Var(Str::from("undefined"));
        let result = interpreter.evaluate_expr(&var_expr, &ctx).await;
        assert!(result.is_err());
        if let Err(ExprError::UndefinedSymbol { symbol }) = result {
            assert_eq!(symbol.as_str(), "undefined");
        } else {
            panic!("Expected UndefinedSymbol error");
        }
    }

    #[tokio::test]
    async fn test_evaluate_arithmetic() {
        let interpreter = Interpreter::new();
        let ctx = create_test_context();
        
        // Parse and evaluate (+ x y)
        let add_expr = parse("(+ x y)").expect("Failed to parse addition");
        let result = interpreter.evaluate_expr(&add_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(30)));
        
        // Parse and evaluate (- y x)
        let sub_expr = parse("(- y x)").expect("Failed to parse subtraction");
        let result = interpreter.evaluate_expr(&sub_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(10)));
        
        // Parse and evaluate (* x 2)
        let mul_expr = parse("(* x 2)").expect("Failed to parse multiplication");
        let result = interpreter.evaluate_expr(&mul_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(20)));
        
        // Parse and evaluate (/ y 5)
        let div_expr = parse("(/ y 5)").expect("Failed to parse division");
        let result = interpreter.evaluate_expr(&div_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(4)));
    }

    #[tokio::test]
    async fn test_evaluate_conditionals() {
        let interpreter = Interpreter::new();
        let ctx = create_test_context();
        
        // Parse and evaluate (if true x y)
        let if_expr = parse("(if true x y)").expect("Failed to parse if");
        let result = interpreter.evaluate_expr(&if_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(10)));
        
        // Parse and evaluate (if false x y)
        let if_expr = parse("(if false x y)").expect("Failed to parse if");
        let result = interpreter.evaluate_expr(&if_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(20)));
        
        // Parse and evaluate (if (> x 5) "big" "small")
        let if_expr = parse("(if (> x 5) \"big\" \"small\")").expect("Failed to parse complex if");
        let result = interpreter.evaluate_expr(&if_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::String(Str::from("big")));
    }

    #[tokio::test]
    async fn test_evaluate_lambda_and_application() {
        let interpreter = Interpreter::new();
        let ctx = DefaultExprContext::new();
        
        // Parse and evaluate ((fn (a b) (+ a b)) 5 7)
        let lambda_expr = parse("((fn (a b) (+ a b)) 5 7)").expect("Failed to parse lambda application");
        let result = interpreter.evaluate_expr(&lambda_expr, &ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(12)));
        
        // Test with variable capture from outer scope
        let mut ctx_with_z = DefaultExprContext::new();
        ctx_with_z.add_binding("z", ValueExpr::Number(Number::Integer(100)));
        
        // Parse and evaluate ((fn (a) (+ a z)) 5)
        let lambda_expr = parse("((fn (a) (+ a z)) 5)").expect("Failed to parse lambda with capture");
        let result = interpreter.evaluate_expr(&lambda_expr, &ctx_with_z).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValueExpr::Number(Number::Integer(105)));
    }
} 