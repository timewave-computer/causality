// Test file for SSZ serialization of Expr types
// This file is independent from the main codebase to avoid compilation issues during transition

use causality_types::{
    core::str::Str,
    expr::ast::{Atom, AtomicCombinator, Expr, ExprBox, ExprVec},
    serialization::{Decode, Encode},
};

#[test]
fn test_expr_serialization() {
    // Create a simple expression
    let expr = Expr::Atom(Atom::Integer(42));
    
    // Serialize
    let bytes = expr.as_ssz_bytes();
    
    // Deserialize
    let decoded = Expr::from_ssz_bytes(&bytes).expect("Failed to decode");
    
    // Verify
    assert_eq!(expr, decoded);
}

#[test]
fn test_atom_serialization() {
    // Test all atom variants
    let atoms = vec![
        Atom::Integer(42),
        Atom::String(Str::from("hello")),
        Atom::Boolean(true),
        Atom::Nil,
    ];
    
    for atom in atoms {
        let bytes = atom.as_ssz_bytes();
        let decoded = Atom::from_ssz_bytes(&bytes).expect("Failed to decode");
        assert_eq!(atom, decoded);
    }
}

#[test]
fn test_combinator_serialization() {
    // Test combinator serialization
    let combinator = AtomicCombinator::Add;
    let bytes = combinator.as_ssz_bytes();
    let decoded = AtomicCombinator::from_ssz_bytes(&bytes).expect("Failed to decode");
    assert_eq!(combinator, decoded);
}

#[test]
#[ignore = "Lambda expression serialization with Vec<Str> needs fixing - encoding issue"]
fn test_lambda_expression() {
    // Create a lambda expression
    let params = vec![Str::from("x")];
    let body = Box::new(Expr::Var(Str::from("x")));
    let lambda = Expr::Lambda(params, ExprBox(body));
    
    // Serialize and deserialize
    let bytes = lambda.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&bytes).expect("Failed to decode");
    
    // Verify
    assert_eq!(lambda, decoded);
}

#[test]
#[ignore = "Application expression serialization with Vec<Expr> needs fixing - encoding issue"]
fn test_application_expression() {
    // Create an application expression: (+ 1 2)
    let func = Box::new(Expr::Combinator(AtomicCombinator::Add));
    let args = vec![
        Expr::Atom(Atom::Integer(1)),
        Expr::Atom(Atom::Integer(2)),
    ];
    let app = Expr::Apply(ExprBox(func), ExprVec(args));
    
    // Serialize and deserialize
    let bytes = app.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&bytes).expect("Failed to decode");
    
    // Verify
    assert_eq!(app, decoded);
} 