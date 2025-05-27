// Expression Tests (Integration)

use causality_types::{
    core::id::ExprId,
    core::numeric::Number,
    core::str::str_from_string,
    expr::ast::{Atom, AtomicCombinator, Expr, ExprBox, ExprVec},
    expr::value::ValueExpr,
    serialization::{Decode, Encode},
};
use sha2::{Digest, Sha256};

//-----------------------------------------------------------------------------
// Test
//-----------------------------------------------------------------------------

#[test]
#[ignore = "Expression serialization with Vec<Str> needs fixing - encoding issue"]
fn test_expr_serialization_roundtrip() {
    // Test simple expression serialization/deserialization
    let expr = Expr::Const(ValueExpr::Number(Number::Integer(42)));
    let bytes = expr.as_ssz_bytes();
    let deserialized = Expr::from_ssz_bytes(&bytes).expect("Failed to deserialize");

    assert_eq!(expr, deserialized);

    // Test more complex expression serialization/deserialization
    // Using Apply with the Let combinator instead of Let directly
    let expr = Expr::Apply(
        ExprBox(Box::new(Expr::Combinator(AtomicCombinator::Let))),
        ExprVec(vec![
            Expr::Var(str_from_string("x")),
            Expr::Atom(Atom::Integer(42)),
            Expr::Var(str_from_string("x")),
        ]),
    );

    assert_roundtrip_works(&expr);
}

#[test]
fn test_expr_id_stability() {
    // Test that ExprId is stable across serialization/deserialization
    let expr = Expr::Const(ValueExpr::Number(Number::Integer(42)));
    let id1 = compute_expr_id(&expr);

    // Serialize and deserialize
    let bytes = expr.as_ssz_bytes();
    let deserialized = Expr::from_ssz_bytes(&bytes).expect("Failed to deserialize");

    // Compute ID again
    let id2 = compute_expr_id(&deserialized);

    // IDs should match
    assert_eq!(id1, id2);

    // Different expressions should have different IDs
    let expr2 = Expr::Const(ValueExpr::Number(Number::Integer(43)));
    let id3 = compute_expr_id(&expr2);

    assert_ne!(id1, id3);
}

/// Helper function to compute expression ID
fn compute_expr_id(expr: &Expr) -> ExprId {
    let bytes = expr.as_ssz_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    
    let mut id_bytes = [0u8; 32];
    id_bytes.copy_from_slice(&result[..32]);
    ExprId::new(id_bytes)
}

#[test]
#[ignore = "Expression variant serialization with Vec<Str> needs fixing - encoding issue"]
fn test_expr_variant_serialization() {
    // Test each expression variant

    // Const
    let expr = Expr::Const(ValueExpr::Number(Number::Integer(42)));
    assert_roundtrip_works(&expr);

    // Var
    let expr = Expr::Var(str_from_string("x"));
    assert_roundtrip_works(&expr);

    // Let (now implemented using Apply with the Let combinator)
    let expr = Expr::Apply(
        ExprBox(Box::new(Expr::Combinator(AtomicCombinator::Let))),
        ExprVec(vec![
            Expr::Var(str_from_string("x")),
            Expr::Atom(Atom::Integer(42)),
            Expr::Var(str_from_string("x")),
        ]),
    );
    assert_roundtrip_works(&expr);

    // Conditional expression (now using If combinator)
    let expr = Expr::Apply(
        ExprBox(Box::new(Expr::Combinator(AtomicCombinator::If))),
        ExprVec(vec![
            Expr::Atom(Atom::Boolean(true)),
            Expr::Atom(Atom::Integer(1)),
            Expr::Atom(Atom::Integer(2)),
        ]),
    );
    assert_roundtrip_works(&expr);

    // Lambda
    let expr = Expr::Lambda(
        vec![str_from_string("x")],
        ExprBox(Box::new(Expr::Var(str_from_string("x")))),
    );
    assert_roundtrip_works(&expr);

    // Apply
    let expr = Expr::Apply(
        ExprBox(Box::new(Expr::Var(str_from_string("f")))),
        ExprVec(vec![Expr::Var(str_from_string("x"))]),
    );
    assert_roundtrip_works(&expr);
}

// Helper function for testing roundtrip serialization
fn assert_roundtrip_works(expr: &Expr) {
    let bytes = expr.as_ssz_bytes();
    let deserialized = Expr::from_ssz_bytes(&bytes).expect("Failed to deserialize");
    assert_eq!(*expr, deserialized);
}
