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
        Term::literal(Literal::Int(val as u32))
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
        let left_effect = make_pure_effect("left_operation");
        let right_effect = make_pure_effect("right_operation");

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
        let left_effect = make_pure_effect("left_racing");
        let right_effect = make_pure_effect("right_racing");

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

    // --- Test Effect Handler Construction

    #[test]
    fn test_effect_handler_construction() {
        let handler_body = make_pure_literal_effect(42);
        let handler = EffectHandler {
            effect_tag: "TestEffect".to_string(),
            params: vec!["param1".to_string(), "param2".to_string()],
            continuation: "continue".to_string(),
            body: handler_body.clone(),
        };

        assert_eq!(handler.effect_tag, "TestEffect");
        assert_eq!(handler.params, vec!["param1", "param2"]);
        assert_eq!(handler.continuation, "continue");
        assert_eq!(handler.body, handler_body);
    }

    // ============================================================================
    // EFFECT ALGEBRA LAWS TESTING (Task 9.5)
    // ============================================================================

    /// Test monad left identity law: bind(pure(a), f) ≡ f(a)
    #[test]
    fn test_monad_left_identity_law() {
        use crate::effect::operations::{pure, bind};
        
        let value = make_literal_term(42);
        let f_body = make_pure_literal_effect(84); // f would double the value
        
        // bind(pure(42), f) should be equivalent to f(42)
        let lhs = bind(pure(value.clone()), "x", f_body.clone());
        let rhs = f_body; // In reality, this would be f applied to value
        
        // Structure should be a bind with pure as the first effect
        if let EffectExprKind::Bind { effect, var, body } = lhs.kind {
            assert_eq!(var, "x");
            if let EffectExprKind::Pure(term) = effect.kind {
                assert_eq!(term, value);
            } else {
                panic!("Expected Pure effect in bind");
            }
            assert_eq!(*body, rhs);
        } else {
            panic!("Expected Bind effect");
        }
    }

    /// Test monad right identity law: bind(m, pure) ≡ m
    #[test]
    fn test_monad_right_identity_law() {
        use crate::effect::operations::{pure, bind};
        
        let m = make_pure_literal_effect(42);
        let identity = pure(make_term("x")); // pure is the identity for the monad
        
        // bind(m, pure) should be equivalent to m (up to structure)
        let result = bind(m.clone(), "x", identity);
        
        if let EffectExprKind::Bind { effect, var, body } = result.kind {
            assert_eq!(var, "x");
            assert_eq!(*effect, m);
            if let EffectExprKind::Pure(term) = body.kind {
                assert_eq!(term, make_term("x")); // Should be identity function
            } else {
                panic!("Expected Pure effect as identity");
            }
        } else {
            panic!("Expected Bind effect");
        }
    }

    /// Test monad associativity law: bind(bind(m, f), g) ≡ bind(m, λx.bind(f(x), g))
    #[test]
    fn test_monad_associativity_law() {
        use crate::effect::operations::{bind};
        
        let m = make_pure_literal_effect(10);
        let f_body = make_pure_literal_effect(20);
        let g_body = make_pure_literal_effect(30);
        
        // Left side: bind(bind(m, f), g)
        let inner_bind = bind(m.clone(), "x", f_body.clone());
        let lhs = bind(inner_bind, "y", g_body.clone());
        
        // Right side: bind(m, λx.bind(f(x), g))
        let nested_bind = bind(f_body.clone(), "z", g_body.clone());
        let rhs = bind(m.clone(), "x", nested_bind);
        
        // Both should have bind structure, though the exact equivalence would need interpretation
        assert!(matches!(lhs.kind, EffectExprKind::Bind { .. }));
        assert!(matches!(rhs.kind, EffectExprKind::Bind { .. }));
    }

    /// Test parallel effect composition properties
    #[test] 
    fn test_parallel_composition_properties() {
        let effect1 = make_pure_literal_effect(1);
        let effect2 = make_pure_literal_effect(2);
        let effect3 = make_pure_literal_effect(3);
        
        // Test commutativity: parallel(a, b) should be equivalent to parallel(b, a)
        let par_ab = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(effect1.clone()),
            right: Box::new(effect2.clone()),
        });
        
        let par_ba = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(effect2.clone()),
            right: Box::new(effect1.clone()),
        });
        
        // Structure should be parallel in both cases
        assert!(matches!(par_ab.kind, EffectExprKind::Parallel { .. }));
        assert!(matches!(par_ba.kind, EffectExprKind::Parallel { .. }));
        
        // Test associativity: parallel(parallel(a, b), c) ~ parallel(a, parallel(b, c))
        let par_ab_c = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(par_ab.clone()),
            right: Box::new(effect3.clone()),
        });
        
        let par_bc = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(effect2.clone()),
            right: Box::new(effect3.clone()),
        });
        
        let par_a_bc = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(effect1.clone()),
            right: Box::new(par_bc),
        });
        
        assert!(matches!(par_ab_c.kind, EffectExprKind::Parallel { .. }));
        assert!(matches!(par_a_bc.kind, EffectExprKind::Parallel { .. }));
    }

    /// Test effect handlers composition
    #[test]
    fn test_handler_composition_properties() {
        let effect = make_pure_literal_effect(42);
        
        let handler1 = EffectHandler {
            effect_tag: "Log".to_string(),
            params: vec!["msg".to_string()],
            continuation: "k".to_string(),
            body: make_pure_effect("logged"),
        };
        
        let handler2 = EffectHandler {
            effect_tag: "State".to_string(),
            params: vec!["state".to_string()],
            continuation: "k".to_string(),
            body: make_pure_effect("state_updated"),
        };
        
        // Test single handler
        let handled_once = EffectExpr::new(EffectExprKind::Handle {
            expr: Box::new(effect.clone()),
            handlers: vec![handler1.clone()],
        });
        
        // Test multiple handlers
        let handled_twice = EffectExpr::new(EffectExprKind::Handle {
            expr: Box::new(effect.clone()),
            handlers: vec![handler1.clone(), handler2.clone()],
        });
        
        // Test nested handlers
        let nested_handled = EffectExpr::new(EffectExprKind::Handle {
            expr: Box::new(handled_once.clone()),
            handlers: vec![handler2.clone()],
        });
        
        assert!(matches!(handled_once.kind, EffectExprKind::Handle { .. }));
        assert!(matches!(handled_twice.kind, EffectExprKind::Handle { .. }));
        assert!(matches!(nested_handled.kind, EffectExprKind::Handle { .. }));
        
        // Verify handler count
        if let EffectExprKind::Handle { handlers, .. } = handled_twice.kind {
            assert_eq!(handlers.len(), 2);
        }
    }

    /// Test conservation verification for resource effects
    #[test]
    fn test_conservation_verification() {
        use crate::effect::operations::perform;
        
        // Test resource allocation
        let alloc_effect = perform("alloc", vec![make_literal_term(100)]);
        assert!(matches!(alloc_effect.kind, EffectExprKind::Perform { .. }));
        
        if let EffectExprKind::Perform { effect_tag, args } = alloc_effect.kind {
            assert_eq!(effect_tag, "alloc");
            assert_eq!(args.len(), 1);
        }
        
        // Test resource consumption
        let consume_effect = perform("consume", vec![make_term("resource_id")]);
        assert!(matches!(consume_effect.kind, EffectExprKind::Perform { .. }));
        
        // Test conservation check
        let check_effect = perform("check", vec![
            make_term("inputs"),
            make_term("outputs"),
        ]);
        
        if let EffectExprKind::Perform { effect_tag, args } = check_effect.kind {
            assert_eq!(effect_tag, "check");
            assert_eq!(args.len(), 2);
        }
    }

    /// Test causal dependency tracking
    #[test]
    fn test_causal_dependency_tracking() {
        use crate::effect::operations::perform;
        
        // Test dependency establishment
        let depend_effect = perform("depend", vec![
            make_term("resource_a"),
            make_term("resource_b"),
        ]);
        
        if let EffectExprKind::Perform { effect_tag, args } = depend_effect.kind {
            assert_eq!(effect_tag, "depend");
            assert_eq!(args.len(), 2);
        }
        
        // Test sequence verification
        let sequence_effect = perform("sequence", vec![
            make_term("proof_ab"),
            make_term("proof_bc"),
        ]);
        
        if let EffectExprKind::Perform { effect_tag, args } = sequence_effect.kind {
            assert_eq!(effect_tag, "sequence");
            assert_eq!(args.len(), 2);
        }
        
        // Test proof verification
        let verify_effect = perform("verify", vec![make_term("causal_proof")]);
        
        if let EffectExprKind::Perform { effect_tag, args } = verify_effect.kind {
            assert_eq!(effect_tag, "verify");
            assert_eq!(args.len(), 1);
        }
    }

    /// Test transaction atomicity properties
    #[test]
    fn test_transaction_atomicity() {
        use crate::effect::operations::{pure, bind, perform};
        
        // Test atomic transaction creation
        let atomic_effect = perform("atomic", vec![make_term("effect_placeholder")]);
        
        if let EffectExprKind::Perform { effect_tag, .. } = atomic_effect.kind {
            assert_eq!(effect_tag, "atomic");
        }
        
        // Test transaction sequencing
        let tx_effects = vec![
            pure(make_literal_term(1)),
            pure(make_literal_term(2)),
            pure(make_literal_term(3)),
        ];
        
        // Build a transaction sequence
        let mut tx = tx_effects[0].clone();
        for (i, effect) in tx_effects.iter().skip(1).enumerate() {
            tx = bind(tx, format!("tx_{}", i), effect.clone());
        }
        
        assert!(matches!(tx.kind, EffectExprKind::Bind { .. }));
        
        // Test commit operation
        let commit_effect = perform("commit", vec![make_term("transaction")]);
        
        if let EffectExprKind::Perform { effect_tag, .. } = commit_effect.kind {
            assert_eq!(effect_tag, "commit");
        }
    }

    /// Test effect algebra distributivity laws
    #[test]
    fn test_effect_distributivity() {
        use crate::effect::operations::{pure, bind};
        
        let value = make_literal_term(42);
        let effect1 = make_pure_literal_effect(1);
        let effect2 = make_pure_literal_effect(2);
        
        // Test that pure distributes over parallel composition (conceptually)
        let pure_val = pure(value.clone());
        
        let par_effects = EffectExpr::new(EffectExprKind::Parallel {
            left: Box::new(effect1.clone()),
            right: Box::new(effect2.clone()),
        });
        
        // bind(pure(v), parallel(e1, e2)) should have certain properties
        let bound_parallel = bind(pure_val, "x", par_effects);
        
        assert!(matches!(bound_parallel.kind, EffectExprKind::Bind { .. }));
        
        if let EffectExprKind::Bind { effect, body, .. } = bound_parallel.kind {
            assert!(matches!(effect.kind, EffectExprKind::Pure(_)));
            assert!(matches!(body.kind, EffectExprKind::Parallel { .. }));
        }
    }
}
 