// Comprehensive tests for the Temporal Effect Language (TEL)

// Include the TEL test modules
mod tel;

// Main tests for TEL functionality
#[cfg(test)]
mod tests {
    use causality::tel::{
        Authorization, AuthorizedEffect, CircuitType, Condition, ConditionalEffect, Effect,
        FactType, Predicate, ResourceContents, TelError, TimeCondition, TimedEffect,
    };
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_tel_core_functionality() {
        // Test that we can create and use TEL effects
        let deposit = Effect::deposit("ethereum", "eth", 1000000000000000000);
        let addr = vec![1, 2, 3, 4];

        // Test effect chaining with builder pattern
        let complex_effect = deposit
            .with_auth(Authorization::Signature {
                address: addr.clone(),
                signature: vec![9, 8, 7, 6],
            })
            .with_condition(Condition::Time(TimeCondition::After(1679305200000)))
            .with_timeout(1679391600000);

        assert!(matches!(complex_effect, TimedEffect { .. }));
    }
}
