//! Test fixtures and data creation utilities for developers
//!
//! This module provides convenient functions for creating test data and fixtures
//! for use in testing Causality Resource Model applications.

use causality_types::{
    primitive::{
        DomainId, EntityId, ResourceId, HandlerId, EffectId,
        Str,
        Timestamp,
        Number,
    },
    effect::domain::Domain,
    resource::{Resource, ResourceFlow},
    effect::{Effect, Handler, Intent},
    expression::value::{ValueExpr, ValueExprVec, ValueExprMap},
};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Domain Fixtures
//-----------------------------------------------------------------------------

/// Create a test domain with deterministic ID
pub fn create_test_domain(name: &str) -> Domain {
    let mut domain_bytes = [0u8; 32];
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    domain_bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    
    Domain::new_with_smt(DomainId::new(domain_bytes), name.to_string())
}

/// Create a test domain ID from a string
pub fn create_test_domain_id(name: &str) -> DomainId {
    let mut domain_bytes = [0u8; 32];
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    domain_bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    DomainId::new(domain_bytes)
}

//-----------------------------------------------------------------------------
// Resource Fixtures
//-----------------------------------------------------------------------------

/// Create a test resource with deterministic ID
pub fn create_test_resource(name: &str, resource_type: &str, quantity: u64) -> Resource {
    let resource_id = create_test_entity_id(name);
    let domain_id = create_test_domain_id("test-domain");
    
    Resource::new(
        resource_id,
        Str::from(name),
        domain_id,
        Str::from(resource_type),
        quantity,
        Timestamp::now(),
    )
}

/// Create a test resource ID from a string
pub fn create_test_resource_id(name: &str) -> ResourceId {
    let mut id_bytes = [0u8; 32];
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    id_bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    ResourceId::new(id_bytes)
}

/// Create a simple test resource with default type
pub fn create_simple_test_resource(name: &str, quantity: u64) -> Resource {
    create_test_resource(name, "test_resource", quantity)
}

/// Create a test resource with token type
pub fn create_token_test_resource(name: &str, quantity: u64) -> Resource {
    create_test_resource(name, "token", quantity)
}

/// Create a test resource with compute type
pub fn create_compute_test_resource(name: &str, quantity: u64) -> Resource {
    create_test_resource(name, "compute", quantity)
}

//-----------------------------------------------------------------------------
// Effect Fixtures
//-----------------------------------------------------------------------------

/// Create a test effect with deterministic ID
pub fn create_test_effect(name: &str, effect_type: &str, inputs: Vec<ResourceFlow>, outputs: Vec<ResourceFlow>) -> Effect {
    let effect_id = create_test_entity_id(name);
    let domain_id = create_test_domain_id("test-domain");
    
    Effect::new(
        effect_id,
        Str::from(name),
        domain_id,
        Str::from(effect_type),
    )
    .with_inputs(inputs)
    .with_outputs(outputs)
    .with_timestamp(Timestamp::now())
}

/// Create a test effect ID from a string
pub fn create_test_effect_id(name: &str) -> EffectId {
    let mut id_bytes = [0u8; 32];
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    id_bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    EffectId::new(id_bytes)
}

/// Create a simple test effect with no inputs/outputs
pub fn create_simple_test_effect(name: &str) -> Effect {
    create_test_effect(name, "test_effect", vec![], vec![])
}

/// Create a test effect with resource flow input
pub fn create_test_effect_with_input(name: &str, input_type: &str, input_quantity: u64) -> Effect {
    let domain_id = create_test_domain_id("test-domain");
    let input = ResourceFlow::new(Str::from(input_type), input_quantity, domain_id);
    create_test_effect(name, "test_effect", vec![input], vec![])
}

/// Create a test effect with resource flow output
pub fn create_test_effect_with_output(name: &str, output_type: &str, output_quantity: u64) -> Effect {
    let domain_id = create_test_domain_id("test-domain");
    let output = ResourceFlow::new(Str::from(output_type), output_quantity, domain_id);
    create_test_effect(name, "test_effect", vec![], vec![output])
}

//-----------------------------------------------------------------------------
// Handler Fixtures
//-----------------------------------------------------------------------------

/// Create a test handler with deterministic ID
pub fn create_test_handler(name: &str, handles_type: &str, priority: u32) -> Handler {
    let handler_id = create_test_entity_id(name);
    let domain_id = create_test_domain_id("test-domain");
    
    Handler::new(
        handler_id,
        Str::from(name),
        domain_id,
        Str::from(handles_type),
    )
    .with_priority(priority)
    .with_timestamp(Timestamp::now())
}

/// Create a test handler ID from a string
pub fn create_test_handler_id(name: &str) -> HandlerId {
    let mut id_bytes = [0u8; 32];
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    id_bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    HandlerId::new(id_bytes)
}

/// Create a simple test handler with default priority
pub fn create_simple_test_handler(name: &str, handles_type: &str) -> Handler {
    create_test_handler(name, handles_type, 1)
}

//-----------------------------------------------------------------------------
// Intent Fixtures
//-----------------------------------------------------------------------------

/// Create a test intent with deterministic ID
pub fn create_test_intent(name: &str, priority: u32, inputs: Vec<ResourceFlow>, outputs: Vec<ResourceFlow>) -> Intent {
    let intent_id = create_test_entity_id(name);
    let domain_id = create_test_domain_id("test-domain");
    
    Intent::new(
        intent_id,
        Str::from(name),
        domain_id,
        priority,
    )
    .with_inputs(inputs)
    .with_outputs(outputs)
    .with_timestamp(Timestamp::now())
}

/// Create a test entity ID from a string
pub fn create_test_entity_id(name: &str) -> EntityId {
    let mut id_bytes = [0u8; 32];
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    id_bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    EntityId::new(id_bytes)
}

/// Create a simple test intent with default priority
pub fn create_simple_test_intent(name: &str) -> Intent {
    create_test_intent(name, 1, vec![], vec![])
}

/// Create a test intent with resource flows
pub fn create_test_intent_with_flows(name: &str, inputs: Vec<ResourceFlow>, outputs: Vec<ResourceFlow>) -> Intent {
    create_test_intent(name, 1, inputs, outputs)
}

//-----------------------------------------------------------------------------
// Value Expression Fixtures
//-----------------------------------------------------------------------------

/// Utilities for creating test value expressions
pub mod value_exprs {
    use super::*;

    /// Create a string value expression
    pub fn string(s: &str) -> ValueExpr {
        ValueExpr::String(Str::from(s))
    }

    /// Create an integer value expression
    pub fn integer(i: i64) -> ValueExpr {
        ValueExpr::Number(Number::Integer(i))
    }

    /// Create a boolean value expression
    pub fn boolean(b: bool) -> ValueExpr {
        ValueExpr::Bool(b)
    }

    /// Create a unit value expression
    pub fn unit() -> ValueExpr {
        ValueExpr::Nil
    }

    /// Create a list value expression
    pub fn list(items: Vec<ValueExpr>) -> ValueExpr {
        ValueExpr::List(ValueExprVec::from(items))
    }

    /// Create a map value expression
    pub fn map(pairs: Vec<(String, ValueExpr)>) -> ValueExpr {
        let mut map = BTreeMap::new();
        for (key, value) in pairs {
            map.insert(Str::from(key), value);
        }
        ValueExpr::Map(ValueExprMap(map))
    }
}

//-----------------------------------------------------------------------------
// Resource Flow Fixtures
//-----------------------------------------------------------------------------

/// Create a test resource flow
pub fn create_test_resource_flow(resource_type: &str, quantity: u64, domain: &str) -> ResourceFlow {
    let domain_id = create_test_domain_id(domain);
    ResourceFlow::new(Str::from(resource_type), quantity, domain_id)
}

/// Create a token resource flow
pub fn create_token_flow(quantity: u64) -> ResourceFlow {
    create_test_resource_flow("token", quantity, "test-domain")
}

/// Create a compute resource flow
pub fn create_compute_flow(quantity: u64) -> ResourceFlow {
    create_test_resource_flow("compute", quantity, "test-domain")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_domain() {
        let domain = create_test_domain("test");
        assert_eq!(domain.name, "test");
    }

    #[test]
    fn test_create_test_resource() {
        let resource = create_test_resource("test_resource", "token", 100);
        assert_eq!(resource.name.as_str(), "test_resource");
        assert_eq!(resource.resource_type.as_str(), "token");
        assert_eq!(resource.quantity, 100);
    }

    #[test]
    fn test_create_test_effect() {
        let effect = create_test_effect("test_effect", "transfer", vec![], vec![]);
        assert_eq!(effect.name.as_str(), "test_effect");
        assert_eq!(effect.effect_type.as_str(), "transfer");
    }

    #[test]
    fn test_create_test_handler() {
        let handler = create_test_handler("test_handler", "token", 5);
        assert_eq!(handler.name.as_str(), "test_handler");
        assert_eq!(handler.handles_type.as_str(), "token");
        assert_eq!(handler.priority, 5);
    }

    #[test]
    fn test_create_test_intent() {
        let intent = create_test_intent("test_intent", 3, vec![], vec![]);
        assert_eq!(intent.name.as_str(), "test_intent");
        assert_eq!(intent.priority, 3);
    }

    #[test]
    fn test_value_expr_fixtures() {
        let string_expr = value_exprs::string("test");
        let int_expr = value_exprs::integer(42);
        let bool_expr = value_exprs::boolean(true);
        let unit_expr = value_exprs::unit();
        let list_expr = value_exprs::list(vec![int_expr.clone(), bool_expr.clone()]);
        let map_expr = value_exprs::map(vec![
            ("key1".to_string(), string_expr.clone()),
            ("key2".to_string(), int_expr.clone()),
        ]);

        assert!(matches!(string_expr, ValueExpr::String(_)));
        assert!(matches!(int_expr, ValueExpr::Number(_)));
        assert!(matches!(bool_expr, ValueExpr::Bool(_)));
        assert!(matches!(unit_expr, ValueExpr::Nil));
        assert!(matches!(list_expr, ValueExpr::List(_)));
        assert!(matches!(map_expr, ValueExpr::Map(_)));
    }

    #[test]
    fn test_resource_flow_fixtures() {
        let token_flow = create_token_flow(100);
        let compute_flow = create_compute_flow(50);

        assert_eq!(token_flow.resource_type.as_str(), "token");
        assert_eq!(token_flow.quantity, 100);
        assert_eq!(compute_flow.resource_type.as_str(), "compute");
        assert_eq!(compute_flow.quantity, 50);
    }
} 