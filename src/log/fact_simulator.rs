// Fact Simulator for Causality
//
// This module provides mechanisms for simulating fact observations
// for testing and development purposes.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::error::{Error, Result};
use crate::log::{FactLogger, FactMetadata, FactEntry, LogStorage};
use crate::log::fact_types::{FactType, RegisterFact, ZKProofFact};
use crate::log::fact_snapshot::{FactId, FactSnapshot, RegisterObservation};

/// Configuration for fact simulation
#[derive(Debug, Clone)]
pub struct FactSimulatorConfig {
    /// The base domain ID
    pub domain_id: DomainId,
    /// The observer name
    pub observer_name: String,
    /// The average time between facts (in milliseconds)
    pub avg_interval_ms: u64,
    /// Random seed for simulation
    pub random_seed: Option<u64>,
    /// Whether to introduce errors/failures
    pub introduce_errors: bool,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
}

impl Default for FactSimulatorConfig {
    fn default() -> Self {
        FactSimulatorConfig {
            domain_id: DomainId::new("simulator"),
            observer_name: "simulator".to_string(),
            avg_interval_ms: 1000,
            random_seed: None,
            introduce_errors: false,
            error_rate: 0.05,
        }
    }
}

/// A builder for simulated facts
#[derive(Debug, Clone)]
pub struct SimulatedFactBuilder {
    /// The fact type
    fact_type: FactType,
    /// The resource ID
    resource_id: Option<ContentId>,
    /// The domain ID
    domain_id: DomainId,
    /// The timestamp
    timestamp: Option<Timestamp>,
    /// Fact metadata
    metadata: FactMetadata,
    /// Additional data
    data: HashMap<String, Vec<u8>>,
}

impl SimulatedFactBuilder {
    /// Create a new simulated fact builder
    pub fn new(fact_type: FactType, domain_id: DomainId, observer: &str) -> Self {
        SimulatedFactBuilder {
            fact_type,
            resource_id: None,
            domain_id,
            timestamp: None,
            metadata: FactMetadata::new(observer),
            data: HashMap::new(),
        }
    }
    
    /// Set the resource ID
    pub fn with_resource(mut self, resource_id: ContentId) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
    
    /// Set the timestamp
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
    
    /// Set the confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.metadata = self.metadata.with_confidence(confidence);
        self
    }
    
    /// Set verification details
    pub fn with_verification(mut self, verifiable: bool, method: Option<String>) -> Self {
        self.metadata = self.metadata.with_verification(verifiable, method);
        self
    }
    
    /// Add data
    pub fn with_data(mut self, key: &str, value: Vec<u8>) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }
    
    /// Build the simulated fact
    pub fn build(self) -> SimulatedFact {
        SimulatedFact {
            fact_type: self.fact_type,
            resource_id: self.resource_id,
            domain_id: self.domain_id,
            timestamp: self.timestamp.unwrap_or_else(Timestamp::now),
            metadata: self.metadata,
            data: self.data,
        }
    }
}

/// A simulated fact
#[derive(Debug, Clone)]
pub struct SimulatedFact {
    /// The fact type
    pub fact_type: FactType,
    /// The resource ID
    pub resource_id: Option<ContentId>,
    /// The domain ID
    pub domain_id: DomainId,
    /// The timestamp
    pub timestamp: Timestamp,
    /// Fact metadata
    pub metadata: FactMetadata,
    /// Additional data
    pub data: HashMap<String, Vec<u8>>,
}

/// Fact simulator for generating simulated facts
pub struct FactSimulator {
    /// The fact logger
    logger: Arc<FactLogger>,
    /// Configuration for simulation
    config: FactSimulatorConfig,
    /// Next fact ID
    next_fact_id: u64,
    /// Random number generator
    rng: std::rc::Rc<std::cell::RefCell<rand::rngs::StdRng>>,
}

impl FactSimulator {
    /// Create a new fact simulator
    pub fn new(
        logger: Arc<FactLogger>,
        config: FactSimulatorConfig,
    ) -> Self {
        // Create RNG with seed
        let seed = config.random_seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            now.as_secs()
        });
        
        use rand::{SeedableRng, rngs::StdRng};
        let rng = StdRng::seed_from_u64(seed);
        
        FactSimulator {
            logger,
            config,
            next_fact_id: 1,
            rng: std::rc::Rc::new(std::cell::RefCell::new(rng)),
        }
    }
    
    /// Generate a unique fact ID
    fn generate_fact_id(&mut self) -> FactId {
        let id = self.next_fact_id;
        self.next_fact_id += 1;
        FactId(format!("sim-{}", id))
    }
    
    /// Simulate a register creation
    pub fn simulate_register_creation(
        &mut self,
        trace_id: TraceId,
        register_id: ContentId,
        initial_data: Vec<u8>,
    ) -> Result<FactId> {
        let register_fact = RegisterFact::RegisterCreation {
            register_id: register_id.clone(),
            initial_data,
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        let metadata = FactMetadata::new(&self.config.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("simulated".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:creation:{}", register_id),
            Some(register_id),
            &fact_type,
            Some(metadata),
        )?;
        
        Ok(self.generate_fact_id())
    }
    
    /// Simulate a register update
    pub fn simulate_register_update(
        &mut self,
        trace_id: TraceId,
        register_id: ContentId,
        new_data: Vec<u8>,
    ) -> Result<FactId> {
        let register_fact = RegisterFact::RegisterUpdate {
            register_id: register_id.clone(),
            new_data,
            previous_version: "simulated-previous-version".to_string(),
        };
        
        let fact_type = FactType::RegisterFact(register_fact);
        
        let metadata = FactMetadata::new(&self.config.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("simulated".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("register:update:{}", register_id),
            Some(register_id),
            &fact_type,
            Some(metadata),
        )?;
        
        Ok(self.generate_fact_id())
    }
    
    /// Simulate a balance fact
    pub fn simulate_balance_fact(
        &mut self,
        trace_id: TraceId,
        resource_id: ContentId,
        balance: u64,
    ) -> Result<FactId> {
        // Create a balance fact
        let fact_type = FactType::BalanceFact;
        
        let metadata = FactMetadata::new(&self.config.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("simulated".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("balance:{}", resource_id),
            Some(resource_id),
            &balance.to_le_bytes(),
            Some(metadata),
        )?;
        
        Ok(self.generate_fact_id())
    }
    
    /// Simulate a transaction fact
    pub fn simulate_transaction_fact(
        &mut self,
        trace_id: TraceId,
        tx_hash: &str,
    ) -> Result<FactId> {
        // Create a transaction fact
        let fact_type = FactType::TransactionFact;
        
        let metadata = FactMetadata::new(&self.config.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("simulated".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("transaction:{}", tx_hash),
            None,
            &tx_hash.as_bytes().to_vec(),
            Some(metadata),
        )?;
        
        Ok(self.generate_fact_id())
    }
    
    /// Simulate a ZK proof verification
    pub fn simulate_proof_verification(
        &mut self,
        trace_id: TraceId,
        verification_key_id: &str,
        proof_hash: &str,
        success: bool,
    ) -> Result<FactId> {
        let zkproof_fact = ZKProofFact::ProofVerification {
            verification_key_id: verification_key_id.to_string(),
            proof_hash: proof_hash.to_string(),
            public_inputs: vec!["simulated-input".to_string()],
            success,
        };
        
        let fact_type = FactType::ZKProofFact(zkproof_fact);
        
        let metadata = FactMetadata::new(&self.config.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("simulated".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("zkproof:verification:{}", proof_hash),
            None,
            &fact_type,
            Some(metadata),
        )?;
        
        Ok(self.generate_fact_id())
    }
    
    /// Simulate a generic fact
    pub fn simulate_fact(
        &mut self,
        trace_id: TraceId,
        fact: SimulatedFact,
    ) -> Result<FactId> {
        // Log the fact
        let fact_type_str = match &fact.fact_type {
            FactType::BalanceFact => "balance",
            FactType::TransactionFact => "transaction",
            FactType::OracleFact => "oracle",
            FactType::BlockFact => "block",
            FactType::TimeFact => "time",
            FactType::RegisterFact(_) => "register",
            FactType::ZKProofFact(_) => "zkproof",
            FactType::Custom(name) => name,
        };
        
        self.logger.log_fact(
            trace_id,
            fact_type_str,
            fact.resource_id,
            &fact.fact_type,
            Some(fact.metadata),
        )?;
        
        Ok(self.generate_fact_id())
    }
    
    /// Create a fact snapshot with simulated facts
    pub fn create_snapshot(
        &mut self,
        fact_ids: &[FactId],
    ) -> FactSnapshot {
        let mut snapshot = FactSnapshot::new(&self.config.observer_name);
        
        for fact_id in fact_ids {
            snapshot.add_fact(fact_id.clone(), self.config.domain_id.clone());
        }
        
        snapshot
    }
} 
