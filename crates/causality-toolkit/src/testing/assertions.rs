//! Custom assertions for testing causality types

use causality_types::{
    core::{Effect, Handler, Intent, Resource},
    expr::value::{ValueExpr, Number},
};

/// Assert that two resources are equal
pub fn assert_resource_eq(left: &Resource, right: &Resource) {
    assert_eq!(left.id, right.id, "Resource IDs don't match");
    assert_eq!(left.name, right.name, "Resource names don't match");
    assert_eq!(left.domain_id, right.domain_id, "Resource domain IDs don't match");
    assert_eq!(left.resource_type, right.resource_type, "Resource types don't match");
    assert_eq!(left.quantity, right.quantity, "Resource quantities don't match");
    // Note: timestamps might differ slightly, so we don't assert exact equality
}

/// Assert that two effects are equal
pub fn assert_effect_eq(left: &Effect, right: &Effect) {
    assert_eq!(left.id, right.id, "Effect IDs don't match");
    assert_eq!(left.name, right.name, "Effect names don't match");
    assert_eq!(left.domain_id, right.domain_id, "Effect domain IDs don't match");
    assert_eq!(left.effect_type, right.effect_type, "Effect types don't match");
    assert_eq!(left.inputs.len(), right.inputs.len(), "Effect input counts don't match");
    assert_eq!(left.outputs.len(), right.outputs.len(), "Effect output counts don't match");
    
    // Compare inputs
    for (i, (left_input, right_input)) in left.inputs.iter().zip(right.inputs.iter()).enumerate() {
        assert_eq!(left_input.resource_type, right_input.resource_type, 
                   "Effect input {} resource types don't match", i);
        assert_eq!(left_input.quantity, right_input.quantity, 
                   "Effect input {} quantities don't match", i);
        assert_eq!(left_input.domain_id, right_input.domain_id, 
                   "Effect input {} domain IDs don't match", i);
    }
    
    // Compare outputs
    for (i, (left_output, right_output)) in left.outputs.iter().zip(right.outputs.iter()).enumerate() {
        assert_eq!(left_output.resource_type, right_output.resource_type, 
                   "Effect output {} resource types don't match", i);
        assert_eq!(left_output.quantity, right_output.quantity, 
                   "Effect output {} quantities don't match", i);
        assert_eq!(left_output.domain_id, right_output.domain_id, 
                   "Effect output {} domain IDs don't match", i);
    }
}

/// Assert that two handlers are equal
pub fn assert_handler_eq(left: &Handler, right: &Handler) {
    assert_eq!(left.id, right.id, "Handler IDs don't match");
    assert_eq!(left.name, right.name, "Handler names don't match");
    assert_eq!(left.domain_id, right.domain_id, "Handler domain IDs don't match");
    assert_eq!(left.handles_type, right.handles_type, "Handler types don't match");
    assert_eq!(left.priority, right.priority, "Handler priorities don't match");
    assert_eq!(left.expression, right.expression, "Handler expressions don't match");
}

/// Assert that two intents are equal
pub fn assert_intent_eq(left: &Intent, right: &Intent) {
    assert_eq!(left.id, right.id, "Intent IDs don't match");
    assert_eq!(left.name, right.name, "Intent names don't match");
    assert_eq!(left.domain_id, right.domain_id, "Intent domain IDs don't match");
    assert_eq!(left.priority, right.priority, "Intent priorities don't match");
    assert_eq!(left.inputs.len(), right.inputs.len(), "Intent input counts don't match");
    assert_eq!(left.outputs.len(), right.outputs.len(), "Intent output counts don't match");
    
    // Compare inputs
    for (i, (left_input, right_input)) in left.inputs.iter().zip(right.inputs.iter()).enumerate() {
        assert_eq!(left_input.resource_type, right_input.resource_type, 
                   "Intent input {} resource types don't match", i);
        assert_eq!(left_input.quantity, right_input.quantity, 
                   "Intent input {} quantities don't match", i);
        assert_eq!(left_input.domain_id, right_input.domain_id, 
                   "Intent input {} domain IDs don't match", i);
    }
    
    // Compare outputs
    for (i, (left_output, right_output)) in left.outputs.iter().zip(right.outputs.iter()).enumerate() {
        assert_eq!(left_output.resource_type, right_output.resource_type, 
                   "Intent output {} resource types don't match", i);
        assert_eq!(left_output.quantity, right_output.quantity, 
                   "Intent output {} quantities don't match", i);
        assert_eq!(left_output.domain_id, right_output.domain_id, 
                   "Intent output {} domain IDs don't match", i);
    }
}

/// Assert that two value expressions are equal
pub fn assert_value_expr_eq(left: &ValueExpr, right: &ValueExpr) {
    match (left, right) {
        (ValueExpr::Nil, ValueExpr::Nil) => {},
        (ValueExpr::Bool(l), ValueExpr::Bool(r)) => assert_eq!(l, r, "Boolean values don't match"),
        (ValueExpr::String(l), ValueExpr::String(r)) => assert_eq!(l, r, "String values don't match"),
        (ValueExpr::Number(l), ValueExpr::Number(r)) => {
            match (l, r) {
                (Number::Integer(l_int), Number::Integer(r_int)) => {
                    assert_eq!(l_int, r_int, "Integer values don't match");
                },
                (Number::Decimal(l_decimal), Number::Decimal(r_decimal)) => {
                    assert_eq!(l_decimal, r_decimal, "Decimal values don't match");
                },
                _ => panic!("Number types don't match: {:?} vs {:?}", l, r),
            }
        },
        (ValueExpr::List(l), ValueExpr::List(r)) => {
            assert_eq!(l.len(), r.len(), "List lengths don't match");
            for (left_item, right_item) in l.iter().zip(r.iter()) {
                assert_value_expr_eq(left_item, right_item);
            }
        },
        (ValueExpr::Map(l), ValueExpr::Map(r)) => {
            assert_eq!(l.len(), r.len(), "Map sizes don't match");
            for (key, left_value) in l.iter() {
                match r.get(key) {
                    Some(right_value) => assert_value_expr_eq(left_value, right_value),
                    None => panic!("Key {:?} not found in right map", key),
                }
            }
        },
        (ValueExpr::Record(l), ValueExpr::Record(r)) => {
            assert_eq!(l.len(), r.len(), "Record sizes don't match");
            for (key, left_value) in l.iter() {
                match r.get(key) {
                    Some(right_value) => assert_value_expr_eq(left_value, right_value),
                    None => panic!("Key {:?} not found in right record", key),
                }
            }
        },
        _ => panic!("ValueExpr types don't match: {:?} vs {:?}", left, right),
    }
}

/// Assert that a resource has the expected properties
pub fn assert_resource_properties(
    resource: &Resource,
    expected_name: &str,
    expected_type: &str,
    expected_quantity: u64,
) {
    assert_eq!(resource.name.as_str(), expected_name, "Resource name doesn't match");
    assert_eq!(resource.resource_type.as_str(), expected_type, "Resource type doesn't match");
    assert_eq!(resource.quantity, expected_quantity, "Resource quantity doesn't match");
}

/// Assert that an effect has the expected properties
pub fn assert_effect_properties(
    effect: &Effect,
    expected_name: &str,
    expected_type: &str,
    expected_input_count: usize,
    expected_output_count: usize,
) {
    assert_eq!(effect.name.as_str(), expected_name, "Effect name doesn't match");
    assert_eq!(effect.effect_type.as_str(), expected_type, "Effect type doesn't match");
    assert_eq!(effect.inputs.len(), expected_input_count, "Effect input count doesn't match");
    assert_eq!(effect.outputs.len(), expected_output_count, "Effect output count doesn't match");
}

/// Assert that a handler has the expected properties
pub fn assert_handler_properties(
    handler: &Handler,
    expected_name: &str,
    expected_handles_type: &str,
    expected_priority: u32,
) {
    assert_eq!(handler.name.as_str(), expected_name, "Handler name doesn't match");
    assert_eq!(handler.handles_type.as_str(), expected_handles_type, "Handler type doesn't match");
    assert_eq!(handler.priority, expected_priority, "Handler priority doesn't match");
}

/// Assert that an intent has the expected properties
pub fn assert_intent_properties(
    intent: &Intent,
    expected_name: &str,
    expected_priority: u32,
    expected_input_count: usize,
    expected_output_count: usize,
) {
    assert_eq!(intent.name.as_str(), expected_name, "Intent name doesn't match");
    assert_eq!(intent.priority, expected_priority, "Intent priority doesn't match");
    assert_eq!(intent.inputs.len(), expected_input_count, "Intent input count doesn't match");
    assert_eq!(intent.outputs.len(), expected_output_count, "Intent output count doesn't match");
}

/// Assert that a collection contains an item matching a predicate
pub fn assert_contains<T, F>(collection: &[T], predicate: F, message: &str)
where
    F: Fn(&T) -> bool,
{
    assert!(
        collection.iter().any(predicate),
        "Collection does not contain expected item: {}",
        message
    );
}

/// Assert that a collection does not contain an item matching a predicate
pub fn assert_not_contains<T, F>(collection: &[T], predicate: F, message: &str)
where
    F: Fn(&T) -> bool,
{
    assert!(
        !collection.iter().any(predicate),
        "Collection contains unexpected item: {}",
        message
    );
}

/// Assert that two collections have the same length
pub fn assert_same_length<T, U>(left: &[T], right: &[U], message: &str) {
    assert_eq!(
        left.len(),
        right.len(),
        "Collections have different lengths: {}",
        message
    );
}

/// Assert that a floating point value is within tolerance of expected value
pub fn assert_within_tolerance(actual: f64, expected: f64, tolerance: f64, message: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "Value {} is not within tolerance {} of expected {}: {}",
        actual,
        tolerance,
        expected,
        message
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::fixtures::*;

    #[test]
    fn test_assert_resource_eq() {
        let resource1 = create_test_resource("test", "token", 100);
        let resource2 = create_test_resource("test", "token", 100);
        assert_resource_eq(&resource1, &resource2);
    }

    #[test]
    fn test_assert_effect_eq() {
        let effect1 = create_test_effect("test", "transfer", vec![], vec![]);
        let effect2 = create_test_effect("test", "transfer", vec![], vec![]);
        assert_effect_eq(&effect1, &effect2);
    }

    #[test]
    fn test_assert_value_expr_eq() {
        use crate::testing::fixtures::value_exprs::*;
        
        let expr1 = string("test");
        let expr2 = string("test");
        assert_value_expr_eq(&expr1, &expr2);
        
        let expr3 = integer(42);
        let expr4 = integer(42);
        assert_value_expr_eq(&expr3, &expr4);
    }

    #[test]
    fn test_assert_resource_properties() {
        let resource = create_test_resource("test_resource", "token", 100);
        assert_resource_properties(&resource, "test_resource", "token", 100);
    }

    #[test]
    fn test_assert_contains() {
        let collection = vec![1, 2, 3, 4, 5];
        assert_contains(&collection, |&x| x == 3, "Should contain 3");
    }

    #[test]
    fn test_assert_within_tolerance() {
        assert_within_tolerance(1.0, 1.1, 0.2, "Should be within tolerance");
    }

    #[test]
    #[should_panic(expected = "Should be within tolerance")]
    fn test_assert_within_tolerance_fail() {
        assert_within_tolerance(1.0, 2.0, 0.5, "Should be within tolerance");
    }
} 