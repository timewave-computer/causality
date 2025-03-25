// Zero-knowledge proof fact observer
// Original file: src/domain/fact/zkproof_observer.rs

// ZK Proof Fact Observer for Causality
//
// This module provides a specialized observer for ZK proof-related facts.

use std::sync::{Arc, Mutex};
use causality_types::{DomainId, TraceId};
use causality_types::{Error, Result};
use crate::log::{FactLogger, FactMetadata};
use causality_engine_types::{FactType, ZKProofFact};
use causality_engine_snapshot::{FactId, FactSnapshot};
use causality_domain::observer::FactObserver;

/// Observer for ZK proof-related facts
pub struct ZKProofFactObserver {
    /// The fact logger
    logger: Arc<FactLogger>,
    /// The domain ID
    domain_id: DomainId,
    /// Observer name
    observer_name: String,
}

impl ZKProofFactObserver {
    /// Create a new ZK proof fact observer
    pub fn new(
        logger: Arc<FactLogger>,
        domain_id: DomainId,
        observer_name: String,
    ) -> Self {
        ZKProofFactObserver {
            logger,
            domain_id,
            observer_name,
        }
    }
    
    /// Observe a proof verification
    pub fn observe_proof_verification(
        &self,
        trace_id: TraceId,
        verification_key_id: &str,
        proof_hash: &str,
        public_inputs: &[String],
        success: bool,
    ) -> Result<FactId> {
        let zkproof_fact = ZKProofFact::ProofVerification {
            verification_key_id: verification_key_id.to_string(),
            proof_hash: proof_hash.to_string(),
            public_inputs: public_inputs.to_vec(),
            success,
        };
        
        let fact_type = FactType::ZKProofFact(zkproof_fact);
        
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("zkproof-verification".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("zkproof:verification:{}", proof_hash),
            None,
            &fact_type,
            Some(metadata),
        )?;
        
        // Return fact ID (simplified)
        Ok(FactId(format!("zkproof:verification:{}", proof_hash)))
    }
    
    /// Observe a batch verification
    pub fn observe_batch_verification(
        &self,
        trace_id: TraceId,
        verification_key_ids: &[String],
        proof_hashes: &[String],
        public_inputs: &[String],
        success: bool,
    ) -> Result<FactId> {
        let zkproof_fact = ZKProofFact::BatchVerification {
            verification_key_ids: verification_key_ids.to_vec(),
            proof_hashes: proof_hashes.to_vec(),
            public_inputs: public_inputs.to_vec(),
            success,
        };
        
        let fact_type = FactType::ZKProofFact(zkproof_fact);
        
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("zkproof-batch-verification".to_string()));
        
        // Log the fact - use the first proof hash as part of the ID
        let id_hash = if proof_hashes.is_empty() { "batch" } else { &proof_hashes[0] };
        
        self.logger.log_fact(
            trace_id,
            &format!("zkproof:batch:{}", id_hash),
            None,
            &fact_type,
            Some(metadata),
        )?;
        
        // Return fact ID (simplified)
        Ok(FactId(format!("zkproof:batch:{}", id_hash)))
    }
    
    /// Observe circuit execution
    pub fn observe_circuit_execution(
        &self,
        trace_id: TraceId,
        circuit_id: &str,
        private_inputs_hash: &str,
        public_inputs: &[String],
        generated_proof_hash: &str,
    ) -> Result<FactId> {
        let zkproof_fact = ZKProofFact::CircuitExecution {
            circuit_id: circuit_id.to_string(),
            private_inputs_hash: private_inputs_hash.to_string(),
            public_inputs: public_inputs.to_vec(),
            generated_proof_hash: generated_proof_hash.to_string(),
        };
        
        let fact_type = FactType::ZKProofFact(zkproof_fact);
        
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("zkproof-circuit-execution".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("zkproof:circuit:{}:{}", circuit_id, generated_proof_hash),
            None,
            &fact_type,
            Some(metadata),
        )?;
        
        // Return fact ID (simplified)
        Ok(FactId(format!("zkproof:circuit:{}:{}", circuit_id, generated_proof_hash)))
    }
    
    /// Observe proof composition
    pub fn observe_proof_composition(
        &self,
        trace_id: TraceId,
        source_proof_hashes: &[String],
        result_proof_hash: &str,
        composition_circuit_id: &str,
    ) -> Result<FactId> {
        let zkproof_fact = ZKProofFact::ProofComposition {
            source_proof_hashes: source_proof_hashes.to_vec(),
            result_proof_hash: result_proof_hash.to_string(),
            composition_circuit_id: composition_circuit_id.to_string(),
        };
        
        let fact_type = FactType::ZKProofFact(zkproof_fact);
        
        let metadata = FactMetadata::new(&self.observer_name)
            .with_confidence(1.0)
            .with_verification(true, Some("zkproof-composition".to_string()));
        
        // Log the fact
        self.logger.log_fact(
            trace_id,
            &format!("zkproof:composition:{}", result_proof_hash),
            None,
            &fact_type,
            Some(metadata),
        )?;
        
        // Return fact ID (simplified)
        Ok(FactId(format!("zkproof:composition:{}", result_proof_hash)))
    }
    
    /// Create a fact snapshot with ZK proof facts
    pub fn create_zkproof_snapshot(
        &self,
        fact_ids: &[FactId],
    ) -> FactSnapshot {
        let mut snapshot = FactSnapshot::new(&self.observer_name);
        
        for fact_id in fact_ids {
            snapshot.add_fact(fact_id.clone(), self.domain_id.clone());
        }
        
        snapshot
    }
} 