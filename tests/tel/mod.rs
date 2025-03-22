// Tests for the Temporal Effect Language (TEL) module
pub mod builder;
pub mod types;

#[cfg(test)]
mod tests {
    use causality::tel::{
        Authorization, AuthorizedEffect, Condition, ConditionalEffect, Effect, Predicate,
        TimeCondition, TimedEffect,
    };
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_basic_effect_creation() {
        let deposit = Effect::deposit("ethereum", "eth", 1000000000000000000);

        match deposit {
            Effect::Deposit {
                domain,
                asset,
                amount,
            } => {
                assert_eq!(domain, "ethereum");
                assert_eq!(asset, "eth");
                assert_eq!(amount, 1000000000000000000);
            }
            _ => panic!("Effect should be Deposit"),
        }
    }

    #[test]
    fn test_effect_composition() {
        let addr1 = vec![1, 2, 3, 4];
        let addr2 = vec![5, 6, 7, 8];

        let effect1 = Effect::deposit("ethereum", "eth", 1000000000000000000);
        let effect2 = Effect::transfer(addr1.clone(), addr2.clone(), "eth", 500000000000000000);

        let sequence = Effect::sequence(vec![effect1, effect2]);

        match sequence {
            Effect::Sequence(effects) => {
                assert_eq!(effects.len(), 2);

                match &effects[0] {
                    Effect::Deposit {
                        domain,
                        asset,
                        amount,
                    } => {
                        assert_eq!(domain, "ethereum");
                        assert_eq!(asset, "eth");
                        assert_eq!(*amount, 1000000000000000000);
                    }
                    _ => panic!("First effect should be Deposit"),
                }

                match &effects[1] {
                    Effect::Transfer {
                        from,
                        to,
                        asset,
                        amount,
                    } => {
                        assert_eq!(from, &addr1);
                        assert_eq!(to, &addr2);
                        assert_eq!(asset, "eth");
                        assert_eq!(*amount, 500000000000000000);
                    }
                    _ => panic!("Second effect should be Transfer"),
                }
            }
            _ => panic!("Effect should be Sequence"),
        }
    }

    #[test]
    fn test_builder_pattern() {
        let addr = vec![1, 2, 3, 4];
        let sig = vec![9, 8, 7, 6];
        let now = 1679305200000; // March 20, 2023 in milliseconds

        let effect = Effect::deposit("ethereum", "eth", 1000000000000000000)
            .with_auth(Authorization::Signature {
                address: addr.clone(),
                signature: sig.clone(),
            })
            .with_condition(Condition::Time(TimeCondition::After(now)))
            .with_timeout(now + 86400000); // 24 hours later

        assert!(matches!(effect, TimedEffect { .. }));
        assert_eq!(effect.timeout, now + 86400000);

        match effect.effect {
            Effect::Deposit {
                domain,
                asset,
                amount,
            } => {
                assert_eq!(domain, "ethereum");
                assert_eq!(asset, "eth");
                assert_eq!(amount, 1000000000000000000);
            }
            _ => panic!("Inner effect should be Deposit"),
        }
    }
}
