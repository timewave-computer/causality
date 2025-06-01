//! Core effect types for Layer 2
//!
//! This module defines the effect-specific constructs for the effect algebra layer.
//! Generic expression constructs have been moved to Layer 1 terms.

use crate::lambda::{Term, TypeInner};

//-----------------------------------------------------------------------------
// Effect Expressions
//-----------------------------------------------------------------------------

/// An effect expression in Layer 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectExpr {
    /// The effect expression kind
    pub kind: EffectExprKind,
    
    /// Optional type annotation
    pub ty: Option<TypeInner>,
}

/// Different kinds of effect expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectExprKind {
    /// Pure computation: pure t
    Pure(Term),
    
    /// Effect binding: bind e (x -> e')
    Bind {
        effect: Box<EffectExpr>,
        var: String,
        body: Box<EffectExpr>,
    },
    
    /// Effect performance: perform effect args
    Perform {
        effect_tag: String,
        args: Vec<Term>,
    },
    
    /// Effect handling: handle e with { ... }
    Handle {
        expr: Box<EffectExpr>,
        handlers: Vec<EffectHandler>,
    },
    
    /// Parallel composition: parallel e1 e2
    Parallel {
        left: Box<EffectExpr>,
        right: Box<EffectExpr>,
    },
    
    /// Racing effects: race e1 e2
    Race {
        left: Box<EffectExpr>,
        right: Box<EffectExpr>,
    },
}

//-----------------------------------------------------------------------------
// Effect Handlers
//-----------------------------------------------------------------------------

/// Handler for a specific effect
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectHandler {
    /// Effect tag to handle
    pub effect_tag: String,
    
    /// Parameters of the effect
    pub params: Vec<String>,
    
    /// Continuation parameter name
    pub continuation: String,
    
    /// Handler body
    pub body: EffectExpr,
}

//-----------------------------------------------------------------------------
// Source Locations (kept for error reporting)
//-----------------------------------------------------------------------------

/// Source code span for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Start position
    pub start: Position,
    
    /// End position
    pub end: Position,
    
    /// Source file (optional)
    pub file: Option<String>,
}

/// Position in source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: u32,
    
    /// Column number (1-indexed)
    pub column: u32,
    
    /// Byte offset
    pub offset: u32,
}

//-----------------------------------------------------------------------------
// Constructor Helpers
//-----------------------------------------------------------------------------

impl EffectExpr {
    /// Create a new effect expression
    pub fn new(kind: EffectExprKind) -> Self {
        Self { kind, ty: None }
    }
    
    /// Add type annotation
    pub fn with_type(mut self, ty: TypeInner) -> Self {
        self.ty = Some(ty);
        self
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{EffectExpr, EffectExprKind, EffectHandler}; // Items from current module
    use crate::lambda::{Term, Literal, TypeInner, base::BaseType}; // Items from Layer 1

    // Helper to create a simple term for testing
    fn make_term(name: &str) -> Term {
        Term::var(name)
    }

    // Helper to create a literal term for testing
    fn make_literal_term(val: i64) -> Term {
        Term::literal(Literal::Int(val.try_into().unwrap()))
    }

    // Helper to create a simple pure effect expression for testing
    fn make_pure_effect(name: &str) -> EffectExpr {
        EffectExpr::new(EffectExprKind::Pure(make_term(name)))
    }

    fn make_pure_literal_effect(val: i64) -> EffectExpr {
        EffectExpr::new(EffectExprKind::Pure(make_literal_term(val)))
    }

    // --- Test EffectExpr Construction Helpers

    #[test]
    fn test_effect_expr_new_and_with_type() {
        let term_x = make_term("x");
        let effect_expr_kind = EffectExprKind::Pure(term_x.clone());
        let effect_expr = EffectExpr::new(effect_expr_kind.clone()); // Clone kind for assertion
        
        assert_eq!(effect_expr.kind, effect_expr_kind);
        assert_eq!(effect_expr.ty, None);

        let ty_ann = TypeInner::Base(BaseType::Bool);
        let typed_effect_expr = effect_expr.with_type(ty_ann.clone());
        
        // Check that the kind is preserved and type is added
        assert_eq!(typed_effect_expr.kind, effect_expr_kind);
        assert_eq!(typed_effect_expr.ty, Some(ty_ann));
    }

    // --- Test EffectExprKind Variants

    #[test]
    fn test_effect_expr_pure() {
        let term_val = make_literal_term(42);
        let effect_expr = EffectExpr::new(EffectExprKind::Pure(term_val.clone()));

        if let EffectExprKind::Pure(term) = effect_expr.kind {
            assert_eq!(term, term_val);
        } else {
            panic!("Expected EffectExprKind::Pure");
        }
        assert_eq!(effect_expr.ty, None);
    }

    #[test]
    fn test_effect_expr_bind() {
        let initial_effect = make_pure_literal_effect(1);
        let body_effect = make_pure_literal_effect(2);

        let effect_expr = EffectExpr::new(EffectExprKind::Bind {
            effect: Box::new(initial_effect.clone()),
            var: "x".to_string(),
            body: Box::new(body_effect.clone()),
        });

        if let EffectExprKind::Bind { effect, var, body } = effect_expr.kind {
            assert_eq!(*effect, initial_effect);
            assert_eq!(var, "x");
            assert_eq!(*body, body_effect);
        } else {
            panic!("Expected EffectExprKind::Bind");
        }
        assert_eq!(effect_expr.ty, None);
    }

    #[test]
    fn test_effect_expr_perform() {
        let arg1 = make_term("arg1");
        let arg2 = make_literal_term(100);
        let effect_expr = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "MyEffect".to_string(),
            args: vec![arg1.clone(), arg2.clone()],
        });

        if let EffectExprKind::Perform { effect_tag, args } = effect_expr.kind {
            assert_eq!(effect_tag, "MyEffect");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], arg1);
            assert_eq!(args[1], arg2);
        } else {
            panic!("Expected EffectExprKind::Perform");
        }
        assert_eq!(effect_expr.ty, None);
    }

    #[test]
    fn test_effect_expr_parallel() {
        let left_effect = make_pure_effect("left_op");
        let right_effect = make_pure_effect("right_op");

        let effect_expr = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(left_effect.clone()),
            right: Box::new(right_effect.clone()),
        });

        if let EffectExprKind::Parallel { left, right } = effect_expr.kind {
            assert_eq!(*left, left_effect);
            assert_eq!(*right, right_effect);
        } else {
            panic!("Expected EffectExprKind::Parallel");
        }
        assert_eq!(effect_expr.ty, None);
    }

    #[test]
    fn test_effect_expr_race() {
        let left_effect = make_pure_effect("first_op");
        let right_effect = make_pure_effect("second_op");

        let effect_expr = EffectExpr::new(EffectExprKind::Race {
            left: Box::new(left_effect.clone()),
            right: Box::new(right_effect.clone()),
        });

        if let EffectExprKind::Race { left, right } = effect_expr.kind {
            assert_eq!(*left, left_effect);
            assert_eq!(*right, right_effect);
        } else {
            panic!("Expected EffectExprKind::Race");
        }
        assert_eq!(effect_expr.ty, None);
    }

    // --- Test EffectHandler Construction

    #[test]
    fn test_effect_handler_construction() {
        let handler_body = make_pure_effect("handler_result");
        let handler = EffectHandler {
            effect_tag: "HandleThisEffect".to_string(),
            params: vec!["p1".to_string(), "p2".to_string()],
            continuation: "k".to_string(),
            body: handler_body.clone(),
        };

        assert_eq!(handler.effect_tag, "HandleThisEffect");
        assert_eq!(handler.params, vec!["p1".to_string(), "p2".to_string()]);
        assert_eq!(handler.continuation, "k");
        assert_eq!(handler.body, handler_body);
    }

    // --- Test EffectExprKind::Handle Construction (using EffectHandler)

    #[test]
    fn test_effect_expr_handle() {
        let expr_to_handle = make_pure_effect("expression_needing_handler");
        
        let handler1_body = make_pure_effect("handler1_logic_output");
        let handler1 = EffectHandler {
            effect_tag: "EffectA".to_string(),
            params: vec!["a".to_string()],
            continuation: "resume_a".to_string(),
            body: handler1_body.clone(),
        };

        let handler2_body = make_pure_effect("handler2_logic_output");
        let handler2 = EffectHandler {
            effect_tag: "EffectB".to_string(),
            params: vec!["b1".to_string(), "b2".to_string()],
            continuation: "resume_b".to_string(),
            body: handler2_body.clone(),
        };

        let effect_expr = EffectExpr::new(EffectExprKind::Handle {
            expr: Box::new(expr_to_handle.clone()),
            handlers: vec![handler1.clone(), handler2.clone()],
        });

        if let EffectExprKind::Handle { expr, handlers } = effect_expr.kind {
            assert_eq!(*expr, expr_to_handle);
            assert_eq!(handlers.len(), 2);
            assert_eq!(handlers[0], handler1);
            assert_eq!(handlers[1], handler2);
        } else {
            panic!("Expected EffectExprKind::Handle");
        }
        assert_eq!(effect_expr.ty, None);
    }
}
 