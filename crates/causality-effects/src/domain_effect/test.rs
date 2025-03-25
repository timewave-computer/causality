//! Tests for Domain Effects
//!
//! This module contains tests for the domain effect system.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    
    use causality_domain::{
        adapter::{DomainAdapter, DomainAdapterFactory, DomainAdapterRegistry},
        domain::{DomainId, DomainType, DomainInfo, Transaction, TransactionId, TransactionReceipt},
        fact::{FactQuery, Fact, FactResult},
        types::{Result as DomainResult, Error as DomainError},
    };
    
    use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
    use crate::handler::{EffectHandler, HandlerResult};
    use crate::domain_effect::{
        DomainAdapterEffect, DomainContext, DomainQueryEffect,
        DomainTransactionEffect, DomainTimeMapEffect, DomainCapabilityEffect,
        domain_registry::{EffectDomainRegistry, DomainEffectHandler, EffectDomainAdapterFactory},
        handler::{DomainEffectHandlerAdapter, create_domain_handler, create_domain_handler_with_new_registry},
        query_domain_fact, submit_domain_transaction, get_domain_time_map, check_domain_capability,
        domain_selection::{SelectionCriteria, select_domains_by_type, select_domains_by_capability}
    };
    
    // Mock domain adapter for testing
    mock! {
        DomainAdapter {}
        
        #[async_trait]
        impl DomainAdapter for DomainAdapter {
            fn domain_id(&self) -> &DomainId;
            fn domain_info(&self) -> &DomainInfo;
            
            async fn current_height(&self) -> DomainResult<u64>;
            async fn current_hash(&self) -> DomainResult<Vec<u8>>;
            
            async fn observe_fact(&self, query: &FactQuery) -> DomainResult<Fact>;
            async fn submit_transaction(&self, tx: &Transaction) -> DomainResult<TransactionId>;
            async fn get_transaction_status(&self, tx_id: &TransactionId) -> DomainResult<TransactionReceipt>;
            
            async fn has_capability(&self, capability: &str) -> DomainResult<bool>;
        }
    }
    
    // Mock domain adapter factory for testing
    mock! {
        DomainAdapterFactory {}
        
        #[async_trait]
        impl DomainAdapterFactory for DomainAdapterFactory {
            async fn create_adapter(&self, domain_id: DomainId) -> DomainResult<Arc<dyn DomainAdapter>>;
            fn supported_domain_types(&self) -> Vec<DomainType>;
        }
    }
    
    #[tokio::test]
    async fn test_domain_query_effect() {
        // Create a mock domain adapter
        let mut mock_adapter = MockDomainAdapter::new();
        let domain_id = "test-domain".to_string();
        let domain_info = DomainInfo {
            name: "Test Domain".to_string(),
            domain_type: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test domain for unit tests".to_string(),
            metadata: HashMap::new(),
        };
        
        // Set up expectations
        mock_adapter.expect_domain_id()
            .returning(move || &domain_id);
        mock_adapter.expect_domain_info()
            .returning(move || &domain_info);
            
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        
        let test_fact = Fact {
            id: "fact-123".into(),
            domain_id: domain_id.clone(),
            fact_type: "test-fact".to_string(),
            data: data.clone(),
            block_height: Some(123),
            block_hash: Some(vec![1, 2, 3]),
            timestamp: Some(1000),
        };
        
        mock_adapter.expect_observe_fact()
            .with(always())
            .returning(move |_| Ok(test_fact.clone()));
            
        // Create a mock factory
        let mut mock_factory = MockDomainAdapterFactory::new();
        mock_factory.expect_create_adapter()
            .with(eq(domain_id.clone()))
            .returning(move |_| Ok(Arc::new(mock_adapter.clone())));
        mock_factory.expect_supported_domain_types()
            .returning(|| vec!["test".to_string()]);
            
        // Create the registry and handler
        let registry = Arc::new(EffectDomainRegistry::new());
        registry.register_factory(Arc::new(mock_factory));
        let handler = create_domain_handler(registry);
        
        // Create a domain query effect
        let query = FactQuery {
            domain_id: domain_id.clone(),
            fact_type: "test-fact".to_string(),
            parameters: HashMap::new(),
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        let effect = DomainQueryEffect::new(domain_id.clone(), query);
        
        // Create an effect context
        let context = EffectContext::new();
        
        // Execute the effect
        let result = handler.handle(Arc::new(effect), &context).await;
        
        // Check the result
        match result {
            HandlerResult::Handled(outcome) => {
                assert!(outcome.success);
                assert_eq!(outcome.data.get("key"), Some(&"value".to_string()));
                assert_eq!(outcome.data.get("fact_type"), Some(&"test-fact".to_string()));
                assert_eq!(outcome.data.get("block_height"), Some(&"123".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result),
        }
    }
    
    #[tokio::test]
    async fn test_domain_capability_effect() {
        // Create a mock domain adapter
        let mut mock_adapter = MockDomainAdapter::new();
        let domain_id = "test-domain".to_string();
        let domain_info = DomainInfo {
            name: "Test Domain".to_string(),
            domain_type: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test domain for unit tests".to_string(),
            metadata: HashMap::new(),
        };
        
        // Set up expectations
        mock_adapter.expect_domain_id()
            .returning(move || &domain_id);
        mock_adapter.expect_domain_info()
            .returning(move || &domain_info);
            
        mock_adapter.expect_has_capability()
            .with(eq("smart_contracts"))
            .returning(|_| Ok(true));
            
        mock_adapter.expect_has_capability()
            .with(eq("tokens"))
            .returning(|_| Ok(false));
            
        // Create a mock factory
        let mut mock_factory = MockDomainAdapterFactory::new();
        mock_factory.expect_create_adapter()
            .with(eq(domain_id.clone()))
            .returning(move |_| Ok(Arc::new(mock_adapter.clone())));
        mock_factory.expect_supported_domain_types()
            .returning(|| vec!["test".to_string()]);
            
        // Create the registry and handler
        let registry = Arc::new(EffectDomainRegistry::new());
        registry.register_factory(Arc::new(mock_factory));
        let handler = create_domain_handler(registry);
        
        // Create a domain capability effect - should return true
        let effect1 = check_domain_capability(domain_id.clone(), "smart_contracts");
        
        // Create an effect context
        let context = EffectContext::new();
        
        // Execute the effect
        let result1 = handler.handle(Arc::new(effect1), &context).await;
        
        // Check the result
        match result1 {
            HandlerResult::Handled(outcome) => {
                assert!(outcome.success);
                assert_eq!(outcome.data.get("has_capability"), Some(&"true".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result1),
        }
        
        // Create another domain capability effect - should return false
        let effect2 = check_domain_capability(domain_id.clone(), "tokens");
        
        // Execute the effect
        let result2 = handler.handle(Arc::new(effect2), &context).await;
        
        // Check the result
        match result2 {
            HandlerResult::Handled(outcome) => {
                assert!(outcome.success);
                assert_eq!(outcome.data.get("has_capability"), Some(&"false".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result2),
        }
    }

    #[tokio::test]
    async fn test_domain_selection_effect() {
        use std::collections::HashMap;
        use std::sync::Arc;
        
        use async_trait::async_trait;
        use mockall::predicate::*;
        use mockall::*;
        
        use causality_domain::{
            adapter::{DomainAdapter, DomainAdapterFactory, DomainAdapterRegistry},
            domain::{DomainId, DomainType, DomainInfo},
            types::{Result as DomainResult, Error as DomainError},
        };
        
        use crate::effect::{Effect, EffectContext};
        use crate::handler::{EffectHandler, HandlerResult};
        use crate::domain_effect::{
            domain_registry::{EffectDomainRegistry},
            handler::{create_domain_handler},
            domain_selection::{SelectionCriteria, select_domains_by_type, select_domains_by_capability}
        };
        
        // Create two mock domain adapters for different domain types
        let mut mock_adapter1 = MockDomainAdapter::new();
        let domain_id1 = "ethereum:mainnet".to_string();
        let domain_info1 = DomainInfo {
            domain_id: domain_id1.clone(),
            name: "Ethereum Mainnet".to_string(),
            domain_type: "ethereum".to_string(),
            version: "1.0".to_string(),
            description: "Ethereum mainnet".to_string(),
            metadata: HashMap::new(),
        };
        
        let mut mock_adapter2 = MockDomainAdapter::new();
        let domain_id2 = "cosmos:hub".to_string();
        let domain_info2 = DomainInfo {
            domain_id: domain_id2.clone(),
            name: "Cosmos Hub".to_string(),
            domain_type: "cosmos".to_string(),
            version: "1.0".to_string(),
            description: "Cosmos Hub mainnet".to_string(),
            metadata: HashMap::new(),
        };
        
        // Set up expectations for the first mock adapter
        mock_adapter1.expect_domain_id()
            .returning(move || &domain_id1);
        mock_adapter1.expect_domain_info()
            .returning(move || &domain_info1);
        mock_adapter1.expect_has_capability()
            .with(eq("smart_contracts"))
            .returning(|_| Ok(true));
        
        // Set up expectations for the second mock adapter
        mock_adapter2.expect_domain_id()
            .returning(move || &domain_id2);
        mock_adapter2.expect_domain_info()
            .returning(move || &domain_info2);
        mock_adapter2.expect_has_capability()
            .with(eq("smart_contracts"))
            .returning(|_| Ok(false));
        
        // Create mock factories for each adapter
        let mut mock_factory1 = MockDomainAdapterFactory::new();
        mock_factory1.expect_create_adapter()
            .with(eq(domain_id1.clone()))
            .returning(move |_| Ok(Arc::new(mock_adapter1.clone())));
        mock_factory1.expect_supported_domain_types()
            .returning(|| vec!["ethereum".to_string()]);
        
        let mut mock_factory2 = MockDomainAdapterFactory::new();
        mock_factory2.expect_create_adapter()
            .with(eq(domain_id2.clone()))
            .returning(move |_| Ok(Arc::new(mock_adapter2.clone())));
        mock_factory2.expect_supported_domain_types()
            .returning(|| vec!["cosmos".to_string()]);
        
        // Create the registry and handler
        let registry = Arc::new(EffectDomainRegistry::new());
        registry.register_factory(Arc::new(mock_factory1));
        registry.register_factory(Arc::new(mock_factory2));
        
        // Pre-initialize adapters (normally done by selection)
        let _ = registry.get_adapter(&domain_id1).await;
        let _ = registry.get_adapter(&domain_id2).await;
        
        let handler = create_domain_handler(registry);
        
        // Create an effect context
        let context = EffectContext::new();
        
        // Test Case 1: Select by domain type (ethereum)
        let effect1 = select_domains_by_type("ethereum");
        let result1 = handler.handle(Arc::new(effect1), &context).await;
        
        match result1 {
            HandlerResult::Handled(outcome) => {
                // Should find exactly one domain
                assert_eq!(outcome.data.get("count"), Some(&"1".to_string()));
                assert_eq!(outcome.data.get("domain_0_id"), Some(&domain_id1));
                assert_eq!(outcome.data.get("domain_0_type"), Some(&"ethereum".to_string()));
                assert_eq!(outcome.data.get("domain_0_name"), Some(&"Ethereum Mainnet".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result1),
        }
        
        // Test Case 2: Select by capability (smart_contracts)
        let effect2 = select_domains_by_capability("smart_contracts");
        let result2 = handler.handle(Arc::new(effect2), &context).await;
        
        match result2 {
            HandlerResult::Handled(outcome) => {
                // Should find exactly one domain (ethereum supports smart contracts)
                assert_eq!(outcome.data.get("count"), Some(&"1".to_string()));
                assert_eq!(outcome.data.get("domain_0_id"), Some(&domain_id1));
                assert_eq!(outcome.data.get("domain_0_type"), Some(&"ethereum".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result2),
        }
        
        // Test Case 3: Select by name pattern (cosmos)
        let effect3 = select_domains_by_name("cosmos");
        let result3 = handler.handle(Arc::new(effect3), &context).await;
        
        match result3 {
            HandlerResult::Handled(outcome) => {
                // Should find exactly one domain
                assert_eq!(outcome.data.get("count"), Some(&"1".to_string()));
                assert_eq!(outcome.data.get("domain_0_id"), Some(&domain_id2));
                assert_eq!(outcome.data.get("domain_0_type"), Some(&"cosmos".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result3),
        }
    }

    #[tokio::test]
    async fn test_evm_contract_call_effect() {
        use std::collections::HashMap;
        use std::sync::Arc;
        
        use async_trait::async_trait;
        use mockall::predicate::*;
        use mockall::*;
        
        use causality_domain::{
            adapter::{DomainAdapter, DomainAdapterFactory, DomainAdapterRegistry},
            domain::{DomainId, DomainType, DomainInfo},
            types::{Result as DomainResult, Error as DomainError},
        };
        
        use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
        use crate::handler::{EffectHandler, HandlerResult};
        use crate::domain_effect::{
            domain_registry::{EffectDomainRegistry},
            handler::{create_domain_handler},
            EvmContractCallEffect, evm_view_call
        };
        
        // Create a mock domain adapter
        let mut mock_adapter = MockDomainAdapter::new();
        let domain_id = "ethereum:mainnet".to_string();
        let domain_info = DomainInfo {
            domain_id: domain_id.clone(),
            name: "Ethereum Mainnet".to_string(),
            domain_type: "ethereum".to_string(),
            version: "1.0".to_string(),
            description: "Ethereum mainnet for testing".to_string(),
            metadata: HashMap::new(),
        };
        
        // Set up adapter expectations
        mock_adapter.expect_domain_id()
            .returning(move || &domain_id);
        mock_adapter.expect_domain_info()
            .returning(move || &domain_info);
            
        // Mock a view call - for simplicity, we're mocking observe_fact 
        // which would be used to implement view calls in a real adapter
        mock_adapter.expect_observe_fact()
            .with(always())
            .returning(|_| {
                let mut data = HashMap::new();
                data.insert("result".to_string(), "1000000000000000000".to_string()); // 1 ETH in wei
                
                Ok(causality_domain::fact::Fact {
                    id: "eth-balance-fact".into(),
                    domain_id: "ethereum:mainnet".to_string(),
                    fact_type: "evm.balance".to_string(),
                    data,
                    block_height: Some(12345678),
                    block_hash: Some(vec![1, 2, 3, 4]),
                    timestamp: Some(1637000000),
                })
            });
            
        // Mock a transaction call - we're using submit_transaction for this
        mock_adapter.expect_submit_transaction()
            .with(always())
            .returning(|_| {
                Ok("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
            });
        
        // Create a mock factory
        let mut mock_factory = MockDomainAdapterFactory::new();
        mock_factory.expect_create_adapter()
            .with(eq(domain_id.clone()))
            .returning(move |_| Ok(Arc::new(mock_adapter.clone())));
        mock_factory.expect_supported_domain_types()
            .returning(|| vec!["ethereum".to_string()]);
            
        // Create registry and handler
        let registry = Arc::new(EffectDomainRegistry::new());
        registry.register_factory(Arc::new(mock_factory));
        let handler = create_domain_handler(registry);
        
        // Create an effect context
        let context = EffectContext::new();
        
        // Test an EVM view call
        let view_call = evm_view_call(
            domain_id.clone(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
            "balanceOf(address)",
            vec!["0x1234567890123456789012345678901234567890".to_string()]
        );
        
        // Execute the effect
        let result = handler.handle(Arc::new(view_call), &context).await;
        
        // Check the result
        match result {
            HandlerResult::Handled(outcome) => {
                assert!(outcome.success);
                assert_eq!(outcome.data.get("result"), Some(&"1000000000000000000".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_evm_state_query_effect() {
        use std::collections::HashMap;
        use std::sync::Arc;
        
        use async_trait::async_trait;
        use mockall::predicate::*;
        use mockall::*;
        
        use causality_domain::{
            adapter::{DomainAdapter, DomainAdapterFactory, DomainAdapterRegistry},
            domain::{DomainId, DomainType, DomainInfo},
            types::{Result as DomainResult, Error as DomainError},
        };
        
        use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
        use crate::handler::{EffectHandler, HandlerResult};
        use crate::domain_effect::{
            domain_registry::{EffectDomainRegistry},
            handler::{create_domain_handler},
            EvmStateQueryEffect, evm_balance
        };
        
        // Create a mock domain adapter
        let mut mock_adapter = MockDomainAdapter::new();
        let domain_id = "ethereum:mainnet".to_string();
        let domain_info = DomainInfo {
            domain_id: domain_id.clone(),
            name: "Ethereum Mainnet".to_string(),
            domain_type: "ethereum".to_string(),
            version: "1.0".to_string(),
            description: "Ethereum mainnet for testing".to_string(),
            metadata: HashMap::new(),
        };
        
        // Set up adapter expectations
        mock_adapter.expect_domain_id()
            .returning(move || &domain_id);
        mock_adapter.expect_domain_info()
            .returning(move || &domain_info);
            
        // Mock a balance query
        mock_adapter.expect_observe_fact()
            .with(always())
            .returning(|_| {
                let mut data = HashMap::new();
                data.insert("balance".to_string(), "2000000000000000000".to_string()); // 2 ETH in wei
                
                Ok(causality_domain::fact::Fact {
                    id: "eth-balance-fact".into(),
                    domain_id: "ethereum:mainnet".to_string(),
                    fact_type: "evm.balance".to_string(),
                    data,
                    block_height: Some(12345678),
                    block_hash: Some(vec![1, 2, 3, 4]),
                    timestamp: Some(1637000000),
                })
            });
        
        // Create a mock factory
        let mut mock_factory = MockDomainAdapterFactory::new();
        mock_factory.expect_create_adapter()
            .with(eq(domain_id.clone()))
            .returning(move |_| Ok(Arc::new(mock_adapter.clone())));
        mock_factory.expect_supported_domain_types()
            .returning(|| vec!["ethereum".to_string()]);
            
        // Create registry and handler
        let registry = Arc::new(EffectDomainRegistry::new());
        registry.register_factory(Arc::new(mock_factory));
        let handler = create_domain_handler(registry);
        
        // Create an effect context
        let context = EffectContext::new();
        
        // Test an EVM balance query
        let balance_query = evm_balance(
            domain_id.clone(),
            "0x1234567890123456789012345678901234567890"
        );
        
        // Execute the effect
        let result = handler.handle(Arc::new(balance_query), &context).await;
        
        // Check the result
        match result {
            HandlerResult::Handled(outcome) => {
                assert!(outcome.success);
                assert!(outcome.data.contains_key("result"));
                assert_eq!(outcome.data.get("query_type"), Some(&"balance".to_string()));
            },
            _ => panic!("Expected Handled result, got {:?}", result),
        }
    }
} 