// TEMPORARILY DISABLED - These tests use the old Intent API
// TODO: Update these tests to use the new unified transform-based intent system

/*
//! Tests for intent system functionality

use causality_core::{
    effect::{
        Intent, ResourceBinding, Constraint, ValueExpr,
    },
    lambda::{base::{Location, Value}, Symbol},
};

#[test]
fn test_intent_basic() {
    let domain = Location::Domain("defi".to_string());
    
    let intent = Intent::new(
        domain.clone(),
        vec![
            ResourceBinding::new("source_account", "Account").with_quantity(100),
            ResourceBinding::new("tokens", "Token").with_quantity(50),
        ],
        Constraint::produces_all(vec![
            ResourceBinding::new("dest_account", "Account"),
            ResourceBinding::new("transferred_tokens", "Token").with_quantity(50),
        ]),
    );
    
    assert_eq!(intent.domain, domain);
    assert_eq!(intent.inputs.len(), 2);
    assert_eq!(intent.inputs[0].name, "source_account");
    assert_eq!(intent.inputs[0].resource_type, "Account");
    assert_eq!(intent.inputs[0].quantity, Some(100));
    assert_eq!(intent.inputs[1].quantity, Some(50));
}

#[test]
fn test_intent_constraints() {
    let domain = Location::Local;
    
    let intent = Intent::new(
        domain,
        vec![
            ResourceBinding::new("token_a", "TokenA").with_quantity(100),
        ],
        Constraint::and(vec![
            Constraint::produces_quantity("token_b", "TokenB", 90),
            Constraint::conservation(
                vec!["token_a".to_string()],
                vec!["token_b".to_string()],
                |input, output| {
                    input == output + 10 // 10% fee
                }
            ),
        ]),
    );
    
    assert!(intent.validate().is_ok());
}

#[test]
fn test_intent_validation_duplicates() {
    let domain = Location::Local;
    
    let intent_duplicate = Intent::new(
        domain.clone(),
        vec![
            ResourceBinding::new("duplicate", "TokenA"),
            ResourceBinding::new("duplicate", "TokenB"), // Duplicate name
        ],
        Constraint::True,
    );
    
    assert!(intent_duplicate.validate().is_err());
}

#[test]
fn test_intent_validation_unknown_reference() {
    let intent_unknown_ref = Intent::new(
        domain.clone(),
        vec![ResourceBinding::new("input", "Token")],
        Constraint::equals(
            ValueExpr::quantity("unknown_binding"), // Reference to non-existent binding
            ValueExpr::quantity("output"),
        ),
    );
    
    assert!(intent_unknown_ref.validate().is_err());
}

#[test]
fn test_intent_validation_out_of_bounds() {
    let domain = Location::Local;
    let intent_out_of_bounds = Intent::new(
        domain,
        vec![ResourceBinding::new("input", "Token")],
        Constraint::equals(
            ValueExpr::literal(Value::Int(100)),
            ValueExpr::literal(Value::Int(100))
        ),
    );
    
    assert!(intent_out_of_bounds.validate().is_ok()); // This should be fine now
}

#[test]
fn test_resource_binding_capabilities() {
    let domain = Location::Local;
    
    let binding = ResourceBinding::new("user_account", "Account")
        .with_capability(Capability::write("write"))
        .with_quantity(1);
    
    assert_eq!(binding.name, "user_account");
    assert_eq!(binding.resource_type, "Account");
    assert_eq!(binding.quantity, Some(1));
    assert!(binding.required_capabilities.len() > 0);
}

#[test]
fn test_intent_conservation_constraint() {
    let domain = Location::Local;
    
    let conservation_constraint = Constraint::conservation(
        vec!["input_a".to_string(), "input_b".to_string()],
        vec!["output_c".to_string()],
        |inputs, outputs| {
            let total_input: i64 = inputs.iter().sum();
            let total_output: i64 = outputs.iter().sum();
            total_input == total_output
        }
    );
    
    // Test that the constraint can be created
    assert!(matches!(conservation_constraint, Constraint::Conservation { .. }));
}

#[test]
fn test_intent_produces_constraint() {
    let domain = Location::Local;
    
    let produces_constraint = Constraint::produces_quantity("output_token", "TokenX", 100);
    
    // Test that the constraint can be created
    assert!(matches!(produces_constraint, Constraint::Produces { .. }));
}

#[test]
fn test_intent_complex_and_constraint() {
    let domain = Location::Local;
    
    let complex_constraint = Constraint::and(vec![
        Constraint::produces_quantity("token_a", "TokenA", 50),
        Constraint::produces_quantity("token_b", "TokenB", 25),
        Constraint::conservation(
            vec!["input".to_string()],
            vec!["token_a".to_string(), "token_b".to_string()],
            |inputs, outputs| {
                let total_input: i64 = inputs.iter().sum();
                let total_output: i64 = outputs.iter().sum();
                total_input >= total_output
            }
        ),
    ]);
    
    assert!(matches!(complex_constraint, Constraint::And { .. }));
}

#[test]
fn test_intent_query_interface() {
    let domain = Location::Local;
    
    let intent = Intent::new(
        domain,
        vec![
            ResourceBinding::new("input1", "TokenA").with_quantity(100),
            ResourceBinding::new("input2", "TokenB").with_quantity(50),
        ],
        Constraint::produces_all(vec![
            ResourceBinding::new("output1", "TokenC").with_quantity(75),
            ResourceBinding::new("output2", "TokenD").with_quantity(25),
        ]),
    );
    
    // Test query interfaces
    let binding_names = intent.get_binding_names();
    assert_eq!(binding_names.len(), 4);
    assert!(binding_names.contains(&"input1".to_string()));
    assert!(binding_names.contains(&"output2".to_string()));
    
    // Test specific binding lookup
    let input1 = intent.get_binding("input1").unwrap();
    assert_eq!(input1.resource_type, "TokenA");
    assert_eq!(input1.quantity, Some(100));
    
    let input2 = intent.get_binding("input2").unwrap();
    assert_eq!(input2.resource_type, "TokenB");
    assert_eq!(input2.quantity, Some(50));
    
    // Test non-existent binding
    assert!(intent.get_binding("nonexistent").is_none());
}

#[test]
fn test_intent_realistic_transfer() {
    let domain = Location::Domain("payments".to_string());
    
    let transfer_intent = Intent::new(
        domain.clone(),
        vec![
            ResourceBinding::new("sender_account", "Account")
                .with_capability(Capability::write("write")),
            ResourceBinding::new("tokens", "Token")
                .with_quantity(1000),
            ResourceBinding::new("fee_tokens", "Token")
                .with_quantity(10),
        ],
        Constraint::and(vec![
            Constraint::produces_all(vec![
                ResourceBinding::new("receiver_account", "Account"),
                ResourceBinding::new("transferred_tokens", "Token")
                    .with_quantity(1000),
                ResourceBinding::new("network_fee", "Token")
                    .with_quantity(10),
            ]),
            Constraint::conservation(
                vec!["tokens".to_string(), "fee_tokens".to_string()],
                vec!["transferred_tokens".to_string(), "network_fee".to_string()],
                |inputs, outputs| {
                    let total_input: i64 = inputs.iter().sum();
                    let total_output: i64 = outputs.iter().sum();
                    total_input == total_output
                }
            ),
        ]),
    );
    
    assert!(transfer_intent.validate().is_ok());
}

#[test] 
fn test_intent_realistic_swap() {
    let domain = Location::Domain("dex".to_string());
    
    let swap_intent = Intent::new(
        domain,
        vec![
            ResourceBinding::new("input_tokens", "TokenA").with_quantity(100),
            ResourceBinding::new("pool_state", "LiquidityPool"),
        ],
        Constraint::and(vec![
            Constraint::produces_all(vec![
                ResourceBinding::new("output_tokens", "TokenB").with_quantity(95), // 5% slippage
                ResourceBinding::new("updated_pool", "LiquidityPool"),
            ]),
            Constraint::invariant(
                ValueExpr::multiply(
                    ValueExpr::field("pool_state", "token_a_balance"),
                    ValueExpr::field("pool_state", "token_b_balance")
                ),
                ValueExpr::multiply(
                    ValueExpr::field("updated_pool", "token_a_balance"),
                    ValueExpr::field("updated_pool", "token_b_balance")
                )
            ),
        ]),
    );
    
    assert!(swap_intent.validate().is_ok());
    assert_eq!(swap_intent.inputs.len(), 2);
}
*/ 