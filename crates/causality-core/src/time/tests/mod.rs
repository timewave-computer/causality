use std::sync::Arc;
use std::collections::HashMap;

use crate::effect::{
    Effect, EffectContext, EffectError, EffectRegistry,
    domain::DomainId,
    registry::BasicEffectRegistry,
};
use crate::time::{
    map::{TimeMap, TimeMapSnapshot, DomainPosition},
    effect::{
        TimeEffect, TimeEffectType, TimeEffectHandler, BasicTimeEffectHandler,
        AdvanceCausalTimeEffect, SetClockTimeEffect, AttestationSource,
        TimeDomainId, TimeAttestation,
    },
    adapter::{TimeSystemAdapter, TimeSystemAdapterFactory},
    provider::TimeProviderFactory,
    effect_handler::InMemoryAttestationStore,
    ClockTime, TimeDelta, Timestamp,
};

// Time module tests

// Include test modules
pub mod effect_tests;

#[test]
fn test_time_map_creation() {
    let map = TimeMap::new();
    assert_eq!(map.positions().len(), 0);
}

#[test]
fn test_snapshot_validity() {
    // Create a time map with two domains
    let mut map = TimeMap::new();
    map.update_position("domain1", DomainPosition::from(10));
    map.update_position("domain2", DomainPosition::from(20));
    
    // Create a snapshot
    let snapshot1 = map.snapshot();
    
    // Make some updates
    map.update_position("domain1", DomainPosition::from(15));
    map.update_position("domain3", DomainPosition::from(5));
    
    // Create another snapshot
    let snapshot2 = map.snapshot();
    
    // Check validity
    assert!(is_snapshot_valid_at(&snapshot1, &snapshot2));
    assert!(!is_snapshot_valid_at(&snapshot2, &snapshot1));
}

// Helper function to check if a snapshot is valid at a reference point
fn is_snapshot_valid_at(
    snapshot: &TimeMapSnapshot,
    reference: &TimeMapSnapshot,
) -> bool {
    // A snapshot is valid if it does not claim to observe any domain position
    // that the reference has not seen yet
    for (domain_id, position) in &snapshot.positions {
        if let Some(ref_position) = reference.positions.get(domain_id) {
            if position.is_after(ref_position) {
                return false;
            }
        } else {
            // Reference hasn't observed this domain at all
            return false;
        }
    }
    
    true
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
    
    fn with_additional_metadata(&self, _metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
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