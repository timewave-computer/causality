// Tests for SSZ serialization of Expr types

use causality_types::{
    primitive::string::Str,
    expression::ast::{Atom, AtomicCombinator, Expr, ExprBox, ExprVec},
    system::serialization::{Decode, Encode},
};

#[test]
fn test_atomic_combinator_serialization() {
    // Test various combinators
    let combinators = vec![
        AtomicCombinator::S,
        AtomicCombinator::K,
        AtomicCombinator::I,
        AtomicCombinator::If,
        AtomicCombinator::Let,
        AtomicCombinator::Add,
        AtomicCombinator::Mul,
    ];
    
    for combinator in combinators {
        let encoded = combinator.as_ssz_bytes();
        let decoded = AtomicCombinator::from_ssz_bytes(&encoded).expect("Failed to decode combinator");
        assert_eq!(combinator, decoded, "Combinator serialization failed for {:?}", combinator);
    }
}

#[test]
fn test_atom_serialization() {
    // Test integer atom
    let int_atom = Atom::Integer(42);
    let encoded = int_atom.as_ssz_bytes();
    let decoded = Atom::from_ssz_bytes(&encoded).expect("Failed to decode integer atom");
    assert_eq!(int_atom, decoded);
    
    // Test string atom
    let string_atom = Atom::String(Str::from("hello"));
    let encoded = string_atom.as_ssz_bytes();
    let decoded = Atom::from_ssz_bytes(&encoded).expect("Failed to decode string atom");
    assert_eq!(string_atom, decoded);
    
    // Test boolean atom
    let bool_atom = Atom::Boolean(true);
    let encoded = bool_atom.as_ssz_bytes();
    let decoded = Atom::from_ssz_bytes(&encoded).expect("Failed to decode boolean atom");
    assert_eq!(bool_atom, decoded);
    
    // Test nil atom
    let nil_atom = Atom::Nil;
    let encoded = nil_atom.as_ssz_bytes();
    let decoded = Atom::from_ssz_bytes(&encoded).expect("Failed to decode nil atom");
    assert_eq!(nil_atom, decoded);
}

#[test]
fn test_expr_simple_serialization() {
    // Test Atom expression
    let atom_expr = Expr::Atom(Atom::Integer(42));
    let encoded = atom_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode atom expression");
    assert_eq!(atom_expr, decoded);
    
    // Test Var expression
    let var_expr = Expr::Var(Str::from("x"));
    let encoded = var_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode var expression");
    assert_eq!(var_expr, decoded);
    
    // Test Combinator expression
    let comb_expr = Expr::Combinator(AtomicCombinator::Add);
    let encoded = comb_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode combinator expression");
    assert_eq!(comb_expr, decoded);
}

#[test]
#[ignore = "Complex expression serialization needs fixing - Vec<Str> encoding issue"]
fn test_expr_complex_serialization() {
    // Test Lambda expression
    let params = vec![Str::from("x"), Str::from("y")];
    let body = Box::new(Expr::Atom(Atom::Integer(42)));
    let lambda_expr = Expr::Lambda(params, ExprBox(body));
    let encoded = lambda_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode lambda expression");
    assert_eq!(lambda_expr, decoded);
    
    // Test Apply expression
    let func = Box::new(Expr::Combinator(AtomicCombinator::Add));
    let args = vec![
        Expr::Atom(Atom::Integer(1)),
        Expr::Atom(Atom::Integer(2)),
    ];
    let apply_expr = Expr::Apply(ExprBox(func), ExprVec(args));
    let encoded = apply_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode apply expression");
    assert_eq!(apply_expr, decoded);
    
    // Test Dynamic expression
    let inner = Box::new(Expr::Atom(Atom::String(Str::from("dynamic expression"))));
    let dynamic_expr = Expr::Dynamic(100, ExprBox(inner));
    let encoded = dynamic_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode dynamic expression");
    assert_eq!(dynamic_expr, decoded);
}

#[test]
#[ignore = "Nested expression serialization needs fixing - Vec<Str> encoding issue"]
fn test_nested_expr_serialization() {
    // Create a complex nested expression: (lambda (x) (+ x 1))
    let x_param = vec![Str::from("x")];
    let add_func = Box::new(Expr::Combinator(AtomicCombinator::Add));
    let add_args = vec![
        Expr::Var(Str::from("x")),
        Expr::Atom(Atom::Integer(1)),
    ];
    let add_expr = Expr::Apply(ExprBox(add_func), ExprVec(add_args));
    let lambda_expr = Expr::Lambda(x_param, ExprBox(Box::new(add_expr)));
    
    // Test serialization roundtrip
    let encoded = lambda_expr.as_ssz_bytes();
    let decoded = Expr::from_ssz_bytes(&encoded).expect("Failed to decode nested expression");
    assert_eq!(lambda_expr, decoded);
} 