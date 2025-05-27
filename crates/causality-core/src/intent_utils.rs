// Purpose: Utility functions for working with Intents.

use causality_types::{
    AsId,
    core::id::ResourceId,
    core::str::Str,
    core::Intent,
    core::resource_conversion::ToValueExpr,
    expr::value::{ValueExpr, ValueExprMap, ValueExprVec},
};
use std::collections::BTreeMap;
use causality_types::resource::{ResourceFlow, ResourcePattern}; // Import ResourceFlow

// Converts an Intent to a ValueExpr for serialization or debugging.
pub fn value_expr_from_intent(intent: &Intent) -> ValueExpr {
    let mut map = BTreeMap::new();

    map.insert(Str::from("id"), ValueExpr::String(Str::from(intent.id.to_hex())));
    map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(intent.domain_id.to_hex())));
    map.insert(Str::from("name"), ValueExpr::String(intent.name.clone()));
    map.insert(Str::from("priority"), ValueExpr::Number(causality_types::primitive::number::Number::Integer(intent.priority as i64)));

    map.insert(
        Str::from("inputs"),
        ValueExpr::List(ValueExprVec(
            intent
                .inputs
                .iter()
                .map(|input| input.to_value_expr())
                .collect(),
        )),
    );

    map.insert(
        Str::from("outputs"),
        ValueExpr::List(ValueExprVec(
            intent
                .outputs
                .iter()
                .map(|output| output.to_value_expr())
                .collect(),
        )),
    );

    if let Some(expr_id) = &intent.expression {
        map.insert(Str::from("expression"), ValueExpr::String(Str::from(expr_id.to_hex())));
    }

    map.insert(Str::from("timestamp"), ValueExpr::Number(causality_types::primitive::number::Number::Integer(intent.timestamp.as_nanos() as i64)));

    ValueExpr::Record(ValueExprMap(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::primitive::ids::{IntentId, DomainId, ExprId}; // Updated imports
    use causality_types::core::time::Timestamp;

    fn create_test_intent() -> Intent {
        let domain_id = DomainId::null();
        let input_flow = ResourceFlow::new(Str::from("test_resource"), 1, domain_id);
        let output_flow = ResourceFlow::new(Str::from("test_resource"), 2, domain_id);
        
        Intent {
            id: IntentId::null(),
            name: Str::from("Test Intent"),
            domain_id: DomainId::null(),
            priority: 5,
            inputs: vec![input_flow],
            outputs: vec![output_flow],
            expression: Some(ExprId::new([2;32])),
            timestamp: Timestamp::now(),
        }
    }

    #[test]
    fn test_intent_to_value_expr_conversion() {
        let intent = create_test_intent();
        let value_expr = value_expr_from_intent(&intent);

        match value_expr {
            ValueExpr::Record(map_data) => {
                // Check a few fields to ensure serialization logic is running
                assert_eq!(map_data.0.get(&Str::from("priority")).unwrap(), &ValueExpr::Number(5i64.into()));
                assert!(map_data.0.contains_key(&Str::from("expression")));

                // Check inputs (simplified)
                let inputs_val = map_data.0.get(&Str::from("inputs")).unwrap();
                match inputs_val {
                    ValueExpr::List(list_val) => {
                        assert_eq!(list_val.0.len(), 1);
                        // Further checks can be added for specific ResourceRef fields if needed
                    }
                    _ => panic!("Expected List for inputs"),
                }
                 // Check outputs (simplified)
                let outputs_val = map_data.0.get(&Str::from("outputs")).unwrap();
                match outputs_val {
                    ValueExpr::List(list_val) => {
                        assert_eq!(list_val.0.len(), 1);
                        // Further checks can be added for specific ResourceRef fields if needed
                    }
                    _ => panic!("Expected List for outputs"),
                }
            }
            _ => panic!("Expected Record ValueExpr"),
        }
    }
} 