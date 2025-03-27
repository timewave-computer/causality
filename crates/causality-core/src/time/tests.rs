#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use causality_types::time_snapshot::{TimeEffect, TimeEffectResult, AttestationSource};
    use crate::effects::{EffectHandlerRegistry, EffectSystem};
    use crate::time::{TimeEffectHandlerFactory, TimeService};
    use tokio::runtime::Runtime;
    use std::time::Duration;
    use causality_types::time_snapshot::Timestamp;
    use crate::time::{
        provider, service, Timestamp, Duration,
        TimeProviderFactory, TimeService as TimeServiceAlias
    };
    use crate::time::map;
    use crate::time::snapshot::{is_snapshot_valid_at};

    use crate::effect::{
        EffectContext, EffectRegistry, 
        domain::DomainId,
        registry::BasicEffectRegistry,
    };
    use crate::time::{
        effect::{
            TimeEffect, TimeEffectType, TimeEffectHandler, BasicTimeEffectHandler,
            AdvanceCausalTimeEffect, SetClockTimeEffect, AttestationSource,
            TimeDomainId, TimeAttestation,
        },
        adapter::{TimeSystemAdapter, TimeSystemAdapterFactory},
        ClockTime, TimeDelta,
    };
    use crate::time::provider::TimeProviderFactory;
    use crate::time::effect_handler::InMemoryAttestationStore;
    use crate::time::map::TimeMapSnapshot;

    #[tokio::test]
    async fn test_time_effect_handler() {
        // Create a time service
        let time_service = Arc::new(TimeService::new());
        
        // Create a registry and register the time effect handler
        let mut registry = EffectHandlerRegistry::new();
        TimeEffectHandlerFactory::register(&mut registry, time_service.clone());
        
        // Create an effect system
        let effect_system = EffectSystem::new(registry);
        
        // Create a clock attestation effect
        let effect = TimeEffect::ClockAttestation {
            domain_id: "test_domain".to_string(),
            timestamp: 1234567890,
            source: AttestationSource::Operator {
                operator_id: "test_operator".to_string(),
                signature: "test_signature".to_string(),
            },
            confidence: 0.95,
        };
        
        // Handle the effect
        let result = effect_system.handle::<TimeEffect, TimeEffectResult>(effect).await;
        
        // Verify the result
        assert!(result.is_ok());
        if let Ok(result) = result {
            match result {
                TimeEffectResult::ClockUpdate { domain_id, timestamp, confidence } => {
                    assert_eq!(domain_id, "test_domain");
                    assert_eq!(timestamp, 1234567890);
                    assert_eq!(confidence, 0.95);
                }
                _ => panic!("Unexpected result"),
            }
        }
        
        // Verify that the time service was updated
        let timestamp = time_service.get_domain_timestamp("test_domain").unwrap();
        assert!(timestamp.is_some());
        // In a real test, we would verify the actual timestamp value
    }
    
    #[tokio::test]
    async fn test_causal_time_update() {
        // Create a time service
        let time_service = Arc::new(TimeService::new());
        
        // Create a registry and register the time effect handler
        let mut registry = EffectHandlerRegistry::new();
        TimeEffectHandlerFactory::register(&mut registry, time_service.clone());
        
        // Create an effect system
        let effect_system = EffectSystem::new(registry);
        
        // Create a causal update effect
        let effect = TimeEffect::CausalUpdate {
            operations: vec!["op1".to_string(), "op2".to_string()],
            ordering: vec![("op1".to_string(), "op2".to_string())],
        };
        
        // Handle the effect
        let result = effect_system.handle::<TimeEffect, TimeEffectResult>(effect).await;
        
        // Verify the result
        assert!(result.is_ok());
        if let Ok(result) = result {
            match result {
                TimeEffectResult::CausalUpdate { affected_operations, .. } => {
                    assert_eq!(affected_operations.len(), 2);
                    assert!(affected_operations.contains(&"op1".to_string()));
                    assert!(affected_operations.contains(&"op2".to_string()));
                }
                _ => panic!("Unexpected result"),
            }
        }
    }
    
    #[tokio::test]
    async fn test_time_map_update() {
        // Create a time service
        let time_service = Arc::new(TimeService::new());
        
        // Create a registry and register the time effect handler
        let mut registry = EffectHandlerRegistry::new();
        TimeEffectHandlerFactory::register(&mut registry, time_service.clone());
        
        // Create an effect system
        let effect_system = EffectSystem::new(registry);
        
        // Create a time map update effect
        let mut positions = std::collections::HashMap::new();
        positions.insert("domain1".to_string(), 100);
        positions.insert("domain2".to_string(), 200);
        
        let mut proofs = std::collections::HashMap::new();
        proofs.insert("domain1".to_string(), "proof1".to_string());
        proofs.insert("domain2".to_string(), "proof2".to_string());
        
        let effect = TimeEffect::TimeMapUpdate {
            positions,
            proofs,
        };
        
        // Handle the effect
        let result = effect_system.handle::<TimeEffect, TimeEffectResult>(effect).await;
        
        // Verify the result
        assert!(result.is_ok());
        if let Ok(result) = result {
            match result {
                TimeEffectResult::TimeMapUpdate { domains_updated, .. } => {
                    assert_eq!(domains_updated.len(), 2);
                    assert!(domains_updated.contains(&"domain1".to_string()));
                    assert!(domains_updated.contains(&"domain2".to_string()));
                }
                _ => panic!("Unexpected result"),
            }
        }
        
        // Verify that the time service was updated
        let domain1_timestamp = time_service.get_domain_timestamp("domain1").unwrap();
        let domain2_timestamp = time_service.get_domain_timestamp("domain2").unwrap();
        
        assert!(domain1_timestamp.is_some());
        assert!(domain2_timestamp.is_some());
        // In a real test, we would verify the actual timestamp values
    }

    #[test]
    fn test_real_time_provider() {
        let rt = Runtime::new().unwrap();
        let provider = provider::TimeProviderFactory::create_real_time_provider();
        
        // Test now() function
        let now = rt.block_on(async {
            provider.now().await.unwrap()
        });
        assert!(now > Timestamp::zero());
        
        // Test sleep function
        let duration = Duration::from_millis(10);
        let before = rt.block_on(async {
            provider.now().await.unwrap()
        });
        rt.block_on(async {
            provider.sleep(duration).await.unwrap();
        });
        let after = rt.block_on(async {
            provider.now().await.unwrap()
        });
        assert!(after > before);
        
        // Test deadline function
        let deadline_duration = Duration::from_millis(100);
        let deadline = rt.block_on(async {
            provider.deadline(deadline_duration).await.unwrap()
        });
        let now = rt.block_on(async {
            provider.now().await.unwrap()
        });
        assert!(deadline > now);
    }
    
    #[test]
    fn test_simulation_provider() {
        let rt = Runtime::new().unwrap();
        let initial_time = Timestamp::from_millis(1000);
        let provider = provider::TimeProviderFactory::create_simulation_provider(
            Some(initial_time),
            Some(2.0) // 2x speed
        );
        
        // Test now() function returns the initial time
        let now = rt.block_on(async {
            provider.now().await.unwrap()
        });
        assert_eq!(now, initial_time);
        
        // Test sleep advances the simulated time
        let duration = Duration::from_millis(100);
        rt.block_on(async {
            provider.sleep(duration).await.unwrap();
        });
        let after_sleep = rt.block_on(async {
            provider.now().await.unwrap()
        });
        assert_eq!(after_sleep, initial_time + duration);
        
        // Test updating domain positions
        rt.block_on(async {
            provider.update_domain_position("test_domain", 2000).await.unwrap();
        });
        
        // Check domain timestamp
        let domain_ts = rt.block_on(async {
            provider.domain_timestamp("test_domain").await.unwrap()
        });
        assert_eq!(domain_ts, Some(Timestamp::from_nanos(2000)));
        
        // Test snapshot creation
        let snapshot = rt.block_on(async {
            provider.snapshot().await.unwrap()
        });
        assert!(snapshot.positions.contains_key("test_domain"));
    }
    
    #[test]
    fn test_time_service_with_provider() {
        let rt = Runtime::new().unwrap();
        let provider = provider::TimeProviderFactory::create_simulation_provider(
            Some(Timestamp::from_millis(1000)),
            Some(1.0)
        );
        
        let service = service::TimeService::with_provider(provider.clone());
        
        // Test handling a clock attestation effect
        rt.block_on(async {
            service.handle_clock_attestation(
                "test_domain".to_string(),
                5000,
                causality_types::time_snapshot::AttestationSource::Operator {
                    operator_id: "test".to_string(),
                    signature: "sig".to_string(),
                },
                0.9,
            ).await.unwrap();
        });
        
        // Check that both the service and provider have the updated timestamp
        let service_ts = rt.block_on(async {
            service.get_domain_timestamp("test_domain").await.unwrap()
        });
        let provider_ts = rt.block_on(async {
            provider.domain_timestamp("test_domain").await.unwrap()
        });
        
        assert_eq!(service_ts, Some(5000));
        assert_eq!(provider_ts, Some(Timestamp::from_nanos(5000)));
    }

    #[test]
    fn test_time_map() {
        let mut time_map = map::TimeMap::new();
        
        // Update some positions
        time_map.update_position("domain1", 100);
        time_map.update_position("domain2", 200);
        
        // Verify positions
        assert_eq!(time_map.get_position("domain1").unwrap().get_timestamp(), 100);
        assert_eq!(time_map.get_position("domain2").unwrap().get_timestamp(), 200);
        
        // Test snapshot
        let snapshot = time_map.snapshot();
        assert_eq!(snapshot.positions.get("domain1").unwrap().get_timestamp(), 100);
        assert_eq!(snapshot.positions.get("domain2").unwrap().get_timestamp(), 200);
        
        // Test comparability
        time_map.mark_comparable("domain1", "domain2");
        assert!(time_map.are_comparable("domain1", "domain2"));
        assert!(time_map.are_comparable("domain2", "domain1"));
        
        // Test merging
        let mut time_map2 = map::TimeMap::new();
        time_map2.update_position("domain1", 150); // Should override in merge
        time_map2.update_position("domain3", 300); // New domain
        
        time_map.merge(&time_map2);
        
        assert_eq!(time_map.get_position("domain1").unwrap().get_timestamp(), 150);
        assert_eq!(time_map.get_position("domain2").unwrap().get_timestamp(), 200);
        assert_eq!(time_map.get_position("domain3").unwrap().get_timestamp(), 300);
    }
    
    #[test]
    fn test_snapshot_validity() {
        let mut time_map = map::TimeMap::new();
        time_map.update_position("domain1", 100);
        time_map.update_position("domain2", 200);
        
        let snapshot1 = time_map.snapshot();
        
        // Update the time map further
        time_map.update_position("domain1", 150);
        time_map.update_position("domain3", 300);
        
        let snapshot2 = time_map.snapshot();
        
        // Test validity
        assert!(is_snapshot_valid_at(&snapshot1, &snapshot2));
        assert!(!is_snapshot_valid_at(&snapshot2, &snapshot1));
    }

    // Simple effect context for testing
    #[derive(Debug)]
    struct TestContext {
        capabilities: Vec<String>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                capabilities: vec![
                    "time.use_untrusted_sources".to_string(),
                    "time.advance_causal".to_string(),
                    "time.set_clock".to_string(),
                ],
            }
        }
    }

    impl EffectContext for TestContext {
        fn get_capability(&self, name: &str) -> crate::effect::EffectResult<()> {
            if self.capabilities.contains(&name.to_string()) {
                Ok(())
            } else {
                Err(crate::effect::EffectError::CapabilityError(
                    format!("Missing capability: {}", name)
                ))
            }
        }
        
        fn has_capability(&self, name: &str) -> crate::effect::EffectResult<()> {
            self.get_capability(name)
        }
        
        fn verify_resource_access(&self, _resource_id: &crate::resource::ResourceId) -> crate::effect::EffectResult<()> {
            Ok(())
        }
        
        fn get_metadata(&self, _key: &str) -> Option<String> {
            None
        }
        
        fn with_additional_metadata(&self, _metadata: std::collections::HashMap<String, String>) -> Box<dyn EffectContext> {
            Box::new(Self {
                capabilities: self.capabilities.clone(),
            })
        }
    }

    #[tokio::test]
    async fn test_time_effect_system() {
        // Create a domain ID
        let domain_id = DomainId::new("test_domain");
        
        // Create a time provider
        let time_provider = TimeProviderFactory::create_in_memory();
        
        // Create an attestation store
        let attestation_store = Arc::new(InMemoryAttestationStore::new());
        
        // Create the adapter
        let adapter = TimeSystemAdapter::new(
            domain_id.clone(),
            time_provider,
            attestation_store,
        );
        
        // Create a registry
        let mut registry = BasicEffectRegistry::new();
        
        // Register the adapter
        adapter.register(&mut registry).expect("Failed to register adapter");
        
        // Create a test context
        let context = TestContext::new();
        
        // Test advancing causal time
        let advance_outcome = adapter.advance_causal_time(
            TimeDelta::from(10),
            "test advancement",
            &context,
        ).await.expect("Failed to advance causal time");
        
        assert!(advance_outcome.is_success());
        
        // Test setting clock time
        let clock_time = ClockTime::from_secs(1000);
        let set_clock_outcome = adapter.set_clock_time(
            clock_time,
            AttestationSource::SystemClock,
            &context,
        ).await.expect("Failed to set clock time");
        
        assert!(set_clock_outcome.is_success());
        
        // Test registering an attestation
        let attestation = TimeAttestation::new(
            ClockTime::from_secs(2000),
            AttestationSource::NTP,
        );
        
        let attestation_outcome = adapter.register_attestation(
            attestation,
            &context,
        ).await.expect("Failed to register attestation");
        
        assert!(attestation_outcome.is_success());
    }

    #[tokio::test]
    async fn test_time_system_adapter_factory() {
        // Create a domain ID
        let domain_id = DomainId::new("test_domain");
        
        // Create a registry
        let mut registry = BasicEffectRegistry::new();
        
        // Register an adapter for the domain
        let adapter = TimeSystemAdapterFactory::register_for_domain(
            &mut registry,
            domain_id.clone(),
        ).expect("Failed to register adapter for domain");
        
        // Create a test context
        let context = TestContext::new();
        
        // Test advancing causal time
        let advance_outcome = adapter.advance_causal_time(
            TimeDelta::from(10),
            "test advancement",
            &context,
        ).await.expect("Failed to advance causal time");
        
        assert!(advance_outcome.is_success());
    }
} 