// Test Utilities for Fact System
//
// This module provides utilities for testing the fact system.

use std::sync::{Arc, Mutex};
use crate::types::{DomainId, TraceId, Timestamp};
use crate::resource::register::ContentId;
use crate::error::{Error, Result};
use crate::log::{
    FactLogger, FactMetadata, LogStorage, MemoryLogStorage, 
    FactType, RegisterFact, ZKProofFact,
    FactSnapshot, FactId, FactDependency, FactDependencyType,
    FactSimulator, FactSimulatorConfig, SimulatedFact, SimulatedFactBuilder,
    FactReplayEngine, FactReplayConfig
};

/// Creates a test fact logger with an in-memory storage
pub fn create_test_fact_logger() -> (Arc<FactLogger>, Arc<Mutex<MemoryLogStorage>>) {
    let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
    let domain_id = DomainId::new("test-domain");
    let logger = Arc::new(FactLogger::new(storage.clone(), domain_id));
    (logger, storage)
}

/// Creates a test fact simulator
pub fn create_test_fact_simulator() -> (FactSimulator, Arc<FactLogger>, Arc<Mutex<MemoryLogStorage>>) {
    let (logger, storage) = create_test_fact_logger();
    
    let config = FactSimulatorConfig {
        domain_id: DomainId::new("test-domain"),
        observer_name: "test-observer".to_string(),
        avg_interval_ms: 100,
        random_seed: Some(42), // Fixed seed for deterministic tests
        introduce_errors: false,
        error_rate: 0.0,
    };
    
    let simulator = FactSimulator::new(logger.clone(), config);
    
    (simulator, logger, storage)
}

/// Creates a test fact replay engine
pub fn create_test_fact_replay_engine() -> (FactReplayEngine, Arc<Mutex<MemoryLogStorage>>) {
    let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
    
    let config = FactReplayConfig {
        verify_facts: true,
        apply_register_updates: true,
        stop_on_error: true,
        max_facts: None,
        domain_filter: Default::default(),
        resource_filter: Default::default(),
    };
    
    let replay_engine = FactReplayEngine::new(storage.clone(), config);
    
    (replay_engine, storage)
}

/// Creates test register facts
pub fn create_test_register_facts() -> Vec<(ContentId, RegisterFact)> {
    let register_id = ContentId::new("test-register-1");
    let register_id2 = ContentId::new("test-register-2");
    
    vec![
        (
            register_id.clone(),
            RegisterFact::RegisterCreation {
                register_id: register_id.clone(),
                initial_data: vec![1, 2, 3, 4],
            }
        ),
        (
            register_id.clone(),
            RegisterFact::RegisterUpdate {
                register_id: register_id.clone(),
                new_data: vec![5, 6, 7, 8],
                previous_version: "v1".to_string(),
            }
        ),
        (
            register_id.clone(),
            RegisterFact::RegisterTransfer {
                register_id: register_id.clone(),
                source_domain: "domain-1".to_string(),
                target_domain: "domain-2".to_string(),
            }
        ),
        (
            register_id2.clone(),
            RegisterFact::RegisterMerge {
                source_registers: vec![register_id.clone()],
                result_register: register_id2.clone(),
            }
        ),
        (
            register_id.clone(),
            RegisterFact::RegisterSplit {
                source_register: register_id.clone(),
                result_registers: vec![register_id2.clone()],
            }
        ),
    ]
}

/// Creates test ZK proof facts
pub fn create_test_zkproof_facts() -> Vec<ZKProofFact> {
    vec![
        ZKProofFact::ProofVerification {
            verification_key_id: "vk-1".to_string(),
            proof_hash: "proof-hash-1".to_string(),
            public_inputs: vec!["input-1".to_string()],
            success: true,
        },
        ZKProofFact::BatchVerification {
            verification_key_ids: vec!["vk-1".to_string(), "vk-2".to_string()],
            proof_hashes: vec!["proof-hash-1".to_string(), "proof-hash-2".to_string()],
            public_inputs: vec!["input-1".to_string(), "input-2".to_string()],
            success: true,
        },
        ZKProofFact::CircuitExecution {
            circuit_id: "circuit-1".to_string(),
            private_inputs_hash: "private-inputs-hash".to_string(),
            public_inputs: vec!["input-1".to_string()],
            generated_proof_hash: "generated-proof-hash".to_string(),
        },
        ZKProofFact::ProofComposition {
            source_proof_hashes: vec!["proof-hash-1".to_string(), "proof-hash-2".to_string()],
            result_proof_hash: "result-proof-hash".to_string(),
            composition_circuit_id: "composition-circuit".to_string(),
        },
    ]
}

/// Creates a test fact snapshot
pub fn create_test_fact_snapshot(observer: &str) -> FactSnapshot {
    let domain_id = DomainId::new("test-domain");
    let register_id = ContentId::new("test-register-1");
    
    let mut snapshot = FactSnapshot::new(observer);
    
    // Add some fact IDs
    snapshot.add_fact(FactId("fact-1".to_string()), domain_id.clone());
    snapshot.add_fact(FactId("fact-2".to_string()), domain_id.clone());
    
    // Add a register observation
    snapshot.add_register_observation(
        register_id,
        FactId("register-fact-1".to_string()),
        domain_id,
        "data-hash-1",
    );
    
    snapshot
}

/// Creates test transaction facts with dependencies between them
pub fn create_test_transaction_chain() -> Vec<(FactType, FactId, Vec<FactId>)> {
    let balance_fact = FactType::BalanceFact;
    let tx_fact = FactType::TransactionFact;
    
    let balance_id = FactId("balance-1".to_string());
    let tx1_id = FactId("tx-1".to_string());
    let tx2_id = FactId("tx-2".to_string());
    let tx3_id = FactId("tx-3".to_string());
    
    vec![
        (balance_fact, balance_id.clone(), vec![]),
        (tx_fact, tx1_id.clone(), vec![balance_id.clone()]),
        (tx_fact, tx2_id.clone(), vec![tx1_id.clone()]),
        (tx_fact, tx3_id.clone(), vec![tx2_id.clone()]),
    ]
}

/// Test validation function for fact dependencies
pub fn validate_fact_chain(
    chain: &[(FactType, FactId, Vec<FactId>)],
) -> bool {
    let mut seen_facts = std::collections::HashSet::new();
    
    for (_, fact_id, dependencies) in chain {
        // Check if all dependencies have been seen
        for dep_id in dependencies {
            if !seen_facts.contains(dep_id) {
                return false;
            }
        }
        
        // Mark this fact as seen
        seen_facts.insert(fact_id);
    }
    
    true
} 
