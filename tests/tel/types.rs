#[cfg(test)]
mod tests {
    use causality::tel::{
        Address, Amount, AssetId, CircuitType, DomainId, Effect, FactType, ResourceContents,
        ResourceId,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use std::collections::HashMap;

    #[test]
    fn test_effect_serialization() {
        let addr1 = vec![1, 2, 3, 4];
        let addr2 = vec![5, 6, 7, 8];

        let deposit = Effect::Deposit {
            domain: "ethereum".to_string(),
            asset: "eth".to_string(),
            amount: 1000000000000000000,
        };

        let transfer = Effect::Transfer {
            from: addr1.clone(),
            to: addr2.clone(),
            asset: "eth".to_string(),
            amount: 500000000000000000,
        };

        let sequence = Effect::Sequence(vec![deposit, transfer]);

        // Serialize to JSON
        let json_str = serde_json::to_string(&sequence).expect("Failed to serialize");

        // Deserialize from JSON
        let deserialized: Effect = serde_json::from_str(&json_str).expect("Failed to deserialize");

        match deserialized {
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
    fn test_resource_contents() {
        let token_balance = ResourceContents::TokenBalance {
            asset: "eth".to_string(),
            amount: 1000000000000000000,
        };

        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), json!("Crypto Kitty #123"));
        metadata.insert("image".to_string(), json!("https://example.com/cat.jpg"));

        let nft = ResourceContents::NFT {
            collection: "CryptoKitties".to_string(),
            token_id: "123".to_string(),
            metadata,
        };

        let mut components = HashMap::new();
        components.insert("token".to_string(), token_balance);
        components.insert("nft".to_string(), nft);

        let composite = ResourceContents::Composite { components };

        // Serialize to JSON
        let json_str = serde_json::to_string(&composite).expect("Failed to serialize");

        // Deserialize from JSON
        let deserialized: ResourceContents =
            serde_json::from_str(&json_str).expect("Failed to deserialize");

        match deserialized {
            ResourceContents::Composite { components } => {
                assert_eq!(components.len(), 2);
                assert!(components.contains_key("token"));
                assert!(components.contains_key("nft"));

                match &components["token"] {
                    ResourceContents::TokenBalance { asset, amount } => {
                        assert_eq!(asset, "eth");
                        assert_eq!(*amount, 1000000000000000000);
                    }
                    _ => panic!("Component should be TokenBalance"),
                }

                match &components["nft"] {
                    ResourceContents::NFT {
                        collection,
                        token_id,
                        metadata,
                    } => {
                        assert_eq!(collection, "CryptoKitties");
                        assert_eq!(token_id, "123");
                        assert_eq!(metadata.get("name"), Some(&json!("Crypto Kitty #123")));
                        assert_eq!(
                            metadata.get("image"),
                            Some(&json!("https://example.com/cat.jpg"))
                        );
                    }
                    _ => panic!("Component should be NFT"),
                }
            }
            _ => panic!("ResourceContents should be Composite"),
        }
    }
}
