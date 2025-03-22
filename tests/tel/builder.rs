#[cfg(test)]
mod tests {
    use causality::tel::{
        Authorization, AuthorizedEffect, Condition, ConditionalEffect, Effect, FactType, Predicate,
        ResourceContents, TimeCondition, TimedEffect,
    };
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_effect_builders() {
        let addr1 = vec![1, 2, 3, 4];
        let addr2 = vec![5, 6, 7, 8];

        // Test deposit builder
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

        // Test withdraw builder
        let withdraw = Effect::withdraw("ethereum", "eth", 1000000000000000000, addr1.clone());
        match withdraw {
            Effect::Withdraw {
                domain,
                asset,
                amount,
                address,
            } => {
                assert_eq!(domain, "ethereum");
                assert_eq!(asset, "eth");
                assert_eq!(amount, 1000000000000000000);
                assert_eq!(address, addr1);
            }
            _ => panic!("Effect should be Withdraw"),
        }

        // Test transfer builder
        let transfer = Effect::transfer(addr1.clone(), addr2.clone(), "eth", 500000000000000000);
        match transfer {
            Effect::Transfer {
                from,
                to,
                asset,
                amount,
            } => {
                assert_eq!(from, addr1);
                assert_eq!(to, addr2);
                assert_eq!(asset, "eth");
                assert_eq!(amount, 500000000000000000);
            }
            _ => panic!("Effect should be Transfer"),
        }

        // Test sequence builder
        let sequence = Effect::sequence(vec![deposit, withdraw, transfer]);
        match sequence {
            Effect::Sequence(effects) => {
                assert_eq!(effects.len(), 3);
            }
            _ => panic!("Effect should be Sequence"),
        }

        // Test resource creation builder
        let token_balance = ResourceContents::TokenBalance {
            asset: "eth".to_string(),
            amount: 1000000000000000000,
        };

        let create_resource = Effect::create_resource(token_balance.clone());
        match create_resource {
            Effect::ResourceCreate { contents } => match contents {
                ResourceContents::TokenBalance { asset, amount } => {
                    assert_eq!(asset, "eth");
                    assert_eq!(amount, 1000000000000000000);
                }
                _ => panic!("ResourceContents should be TokenBalance"),
            },
            _ => panic!("Effect should be ResourceCreate"),
        }

        // Test with_auth builder
        let resource_id = [0u8; 32];
        let auth_effect = Effect::update_resource(resource_id, token_balance).with_auth(
            Authorization::Signature {
                address: addr1.clone(),
                signature: vec![9, 8, 7, 6],
            },
        );

        assert!(matches!(auth_effect, AuthorizedEffect { .. }));
        assert!(matches!(
            auth_effect.authorization,
            Authorization::Signature { .. }
        ));

        // Test with_condition builder
        let now = 1679305200000; // March 20, 2023 in milliseconds
        let cond_effect = Effect::deposit("ethereum", "eth", 1000000000000000000)
            .with_condition(Condition::Time(TimeCondition::After(now)));

        assert!(matches!(cond_effect, ConditionalEffect { .. }));
        match cond_effect.condition {
            Condition::Time(TimeCondition::After(timestamp)) => {
                assert_eq!(timestamp, now);
            }
            _ => panic!("Condition should be Time(After)"),
        }

        // Test with_timeout builder
        let timed_effect =
            Effect::deposit("ethereum", "eth", 1000000000000000000).with_timeout(now + 86400000); // 24 hours later

        assert!(matches!(timed_effect, TimedEffect { .. }));
        assert_eq!(timed_effect.timeout, now + 86400000);
    }

    #[test]
    fn test_chained_builders() {
        let addr = vec![1, 2, 3, 4];
        let sig = vec![9, 8, 7, 6];
        let now = 1679305200000; // March 20, 2023 in milliseconds

        // Test chaining auth -> condition
        let effect1 = Effect::deposit("ethereum", "eth", 1000000000000000000)
            .with_auth(Authorization::Signature {
                address: addr.clone(),
                signature: sig.clone(),
            })
            .with_condition(Condition::Time(TimeCondition::After(now)));

        assert!(matches!(effect1, ConditionalEffect { .. }));

        // Test chaining condition -> auth
        let effect2 = Effect::deposit("ethereum", "eth", 1000000000000000000)
            .with_condition(Condition::Time(TimeCondition::After(now)))
            .with_auth(Authorization::Signature {
                address: addr.clone(),
                signature: sig.clone(),
            });

        assert!(matches!(effect2, AuthorizedEffect { .. }));

        // Test full chain
        let effect3 = Effect::deposit("ethereum", "eth", 1000000000000000000)
            .with_auth(Authorization::Signature {
                address: addr.clone(),
                signature: sig.clone(),
            })
            .with_condition(Condition::Time(TimeCondition::After(now)))
            .with_timeout(now + 86400000);

        assert!(matches!(effect3, TimedEffect { .. }));

        // Test different ordering
        let effect4 = Effect::deposit("ethereum", "eth", 1000000000000000000)
            .with_timeout(now + 86400000)
            .with_condition(Condition::Time(TimeCondition::After(now)))
            .with_auth(Authorization::Signature {
                address: addr.clone(),
                signature: sig.clone(),
            });

        assert!(matches!(effect4, AuthorizedEffect { .. }));
    }
}
