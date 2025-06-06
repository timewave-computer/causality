//! Tests for the Intent-based programming system

use causality_core::{
    effect::{
        Intent, ResourceBinding, Constraint, ValueExpr,
        capability::Capability,
    },
    lambda::base::Value,
    system::content_addressing::{DomainId, Str},
};

#[test]
fn test_intent_basic_creation() {
    let domain = DomainId::from_content(&Str::new("test_domain"));
    
    let intent = Intent::new(
        domain,
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
fn test_intent_with_constraints() {
    let domain = DomainId::from_content(&Str::new("swap_domain"));
    
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
            ),
            Constraint::equals(
                ValueExpr::quantity("token_a"),
                ValueExpr::literal(Value::Int(100)),
            )
        ]),
    );
    
    // Verify intent validation passes
    assert!(intent.validate().is_ok());
}

#[test]
fn test_intent_validation_errors() {
    let domain = DomainId::from_content(&Str::new("validation_domain"));
    
    // Test duplicate binding names
    let intent_duplicate = Intent::new(
        domain,
        vec![
            ResourceBinding::new("duplicate", "TokenA"),
            ResourceBinding::new("duplicate", "TokenB"), // Duplicate name
        ],
        Constraint::True,
    );
    
    assert!(intent_duplicate.validate().is_err());
    
    // Test constraint referencing unknown binding
    let intent_unknown_ref = Intent::new(
        domain,
        vec![ResourceBinding::new("input", "Token")],
        Constraint::equals(
            ValueExpr::quantity("unknown_binding"), // Reference to non-existent binding
            ValueExpr::quantity("output"),
        ),
    );
    
    assert!(intent_unknown_ref.validate().is_err());
    
    // Test out of bounds input reference
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
fn test_resource_binding_builder_pattern() {
    let read_cap = Capability::read("read");
    let write_cap = Capability::write("write");
    
    let binding = ResourceBinding::new("user_account", "Account")
        .with_quantity(1000)
        .with_capability(read_cap.clone())
        .with_capability(write_cap.clone())
        .with_constraint(Constraint::equals(
            ValueExpr::resource("user_account"),
            ValueExpr::literal(Value::Symbol(Str::new("Account"))),
        ))
        .with_metadata(Value::Bool(true));
    
    assert_eq!(binding.name, "user_account");
    assert_eq!(binding.resource_type, "Account");
    assert_eq!(binding.quantity, Some(1000));
    assert_eq!(binding.capabilities.len(), 2);
    assert!(binding.capabilities.contains(&read_cap));
    assert!(binding.capabilities.contains(&write_cap));
    assert_eq!(binding.constraints.len(), 1);
    assert!(matches!(binding.metadata, Value::Bool(true)));
}

#[test]
fn test_constraint_builder_helpers() {
    // Test AND constraint
    let and_constraint = Constraint::and(vec![
        Constraint::True,
        Constraint::equals(ValueExpr::literal(Value::Int(1)), ValueExpr::literal(Value::Int(1))),
    ]);
    
    // Test OR constraint
    let or_constraint = Constraint::or(vec![
        Constraint::False,
        Constraint::True,
    ]);
    
    // Test NOT constraint
    let not_constraint = Constraint::not(Constraint::False);
    
    // Test capability constraint
    let cap_constraint = Constraint::has_capability(
        "resource",
        "read",
    );
    
    assert!(matches!(and_constraint, Constraint::And(_)));
    assert!(matches!(or_constraint, Constraint::Or(_)));
    assert!(matches!(not_constraint, Constraint::Not(_)));
    assert!(matches!(cap_constraint, Constraint::HasCapability(_, _)));
}

#[test]
fn test_value_expression_builders() {
    // Test literal expressions
    let literal_int = ValueExpr::literal(Value::Int(42));
    let literal_bool = ValueExpr::literal(Value::Bool(true));
    
    // Test resource references
    let resource_ref = ValueExpr::resource("my_resource");
    let quantity_ref = ValueExpr::quantity("token_amount");
    
    // Test arithmetic expressions
    let add_expr = ValueExpr::add(
        ValueExpr::literal(Value::Int(10)),
        ValueExpr::literal(Value::Int(20)),
    );
    
    let complex_expr = ValueExpr::add(
        ValueExpr::quantity("input_amount"),
        ValueExpr::literal(Value::Int(5)),
    );
    
    assert!(matches!(literal_int, ValueExpr::Literal(Value::Int(42))));
    assert!(matches!(literal_bool, ValueExpr::Literal(Value::Bool(true))));
    assert!(matches!(resource_ref, ValueExpr::ResourceRef(_)));
    assert!(matches!(quantity_ref, ValueExpr::QuantityRef(_)));
    assert!(matches!(add_expr, ValueExpr::Add(_, _)));
    assert!(matches!(complex_expr, ValueExpr::Add(_, _)));
}

#[test]
fn test_intent_binding_queries() {
    let domain = DomainId::from_content(&Str::new("query_domain"));
    
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
    
    // Test binding name queries
    let binding_names = intent.get_binding_names();
    assert_eq!(binding_names.len(), 2); // Only inputs are named bindings
    assert!(binding_names.contains(&"input1".to_string()));
    assert!(binding_names.contains(&"input2".to_string()));
    
    // Test specific binding lookup
    let input1 = intent.get_binding("input1").unwrap();
    assert_eq!(input1.name, "input1");
    assert_eq!(input1.resource_type, "TokenA");
    assert_eq!(input1.quantity, Some(100));
    
    let input2 = intent.get_binding("input2").unwrap();
    assert_eq!(input2.name, "input2");
    assert_eq!(input2.resource_type, "TokenB");
    assert_eq!(input2.quantity, Some(50));
    
    // Test non-existent binding
    assert!(intent.get_binding("nonexistent").is_none());
}

#[test]
fn test_real_world_intent_scenarios() {
    let domain = DomainId::from_content(&Str::new("defi_domain"));
    
    // Scenario 1: Token transfer with fee
    let transfer_intent = Intent::new(
        domain,
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
            ),
            Constraint::has_capability(
                "sender_account",
                "write",
            )
        ]),
    );
    
    assert!(transfer_intent.validate().is_ok());
    
    // Scenario 2: Liquidity pool swap
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
            Constraint::equals(
                ValueExpr::quantity("input_tokens"),
                ValueExpr::literal(Value::Int(100)),
            ),
            Constraint::GreaterThan(
                ValueExpr::quantity("output_tokens"),
                ValueExpr::literal(Value::Int(90)), // Minimum output
            )
        ]),
    );
    
    assert!(swap_intent.validate().is_ok());
    assert_eq!(swap_intent.inputs.len(), 2);
} 