//! Tests for symmetric type constructors with intro/elim rules

use causality_core::{
    SumValue, UnitValue,
    SumIntro, SumElim, UnitIntro, UnitElim,
};
use causality_core::lambda::tensor::{TensorValue, TensorElim};

#[test]
fn test_product_intro_elim() {
    // Test product introduction
    let prod = TensorValue::new(42, "hello");
    
    // Test product elimination
    let result = prod.elim_tensor(|x, y| format!("{} {}", x, y));
    assert_eq!(result, "42 hello");
}

#[test]
fn test_product_projections() {
    let prod = TensorValue::new(10, 20);
    
    // Test projections via elimination
    assert_eq!(prod.clone().fst(), 10);
    assert_eq!(prod.snd(), 20);
}

#[test]
fn test_sum_intro_elim() {
    // Test left introduction
    let sum_left: SumValue<i32, String> = SumValue::intro_left(42);
    
    // Test elimination
    let result = sum_left.elim_sum(
        |n| format!("number: {}", n),
        |s| format!("string: {}", s)
    );
    assert_eq!(result, "number: 42");
    
    // Test right introduction  
    let sum_right: SumValue<i32, String> = SumValue::intro_right("world".to_string());
    let result = sum_right.elim_sum(
        |n| format!("number: {}", n),
        |s| format!("string: {}", s)
    );
    assert_eq!(result, "string: world");
}

#[test]
fn test_unit_intro_elim() {
    // Test unit introduction
    let unit = UnitValue::intro_unit();
    
    // Test unit elimination (can only produce constant)
    let result = unit.elim_unit(|| 42);
    assert_eq!(result, 42);
}

#[test]
fn test_nested_types() {
    // Test nested product of sums    
    let left: SumValue<i32, bool> = SumValue::intro_left(10);
    let right: SumValue<String, f64> = SumValue::intro_right(3.14);
    let nested = TensorValue::new(left, right);
    
    // Eliminate the nested structure
    let result = nested.elim_tensor(|l, r| {
        let left_str = l.elim_sum(
            |i| format!("int: {}", i),
            |b| format!("bool: {}", b)
        );
        let right_str = r.elim_sum(
            |s| format!("string: {}", s),
            |f| format!("float: {}", f)
        );
        format!("({}, {})", left_str, right_str)
    });
    
    assert_eq!(result, "(int: 10, float: 3.14)");
}

#[test]
fn test_symmetry_of_operations() {
    // Verify that intro/elim are symmetric for products
    let x = 5;
    let y = "test";
    let prod = TensorValue::new(x, y);
    let (a, b) = prod.elim_tensor(|a, b| (a, b));
    assert_eq!((x, y), (a, b));
    
    // Verify that intro/elim are symmetric for sums
    let val = 42;
    let sum: SumValue<i32, String> = SumValue::intro_left(val);
    let recovered = sum.elim_sum(|x| x, |_| panic!("wrong branch"));
    assert_eq!(val, recovered);
}

#[test]
fn test_convenience_methods() {
    // Test that convenience methods work with intro/elim pattern
    let prod = TensorValue::new(1, 2);
    assert_eq!(prod.clone().into_parts(), (1, 2));
    assert_eq!(prod.as_parts(), (&1, &2));
    
    let sum: SumValue<i32, String> = SumValue::left(10);
    assert!(sum.is_left());
    assert!(!sum.is_right());
} 