use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::crypto::{Signature, PublicKey, KeyPair};
use crate::types::{DomainId, FactId};
use crate::time::effect::{
    TimeError, 
    CausalTimeEffect, 
    ClockTimeEffect, 
    TimeAttestation, 
    TemporalDistance,
    TimeSource,
    TemporalQueryType
};
use crate::time::service::{
    CausalTimeService, 
    ClockTimeService, 
    TimeService,
    TimeAttestationStore,
    FactTimeStore
};

/// In-memory implementation of CausalTimeService
pub struct MemoryCausalTimeService {
    /// Logical clocks by domain
    logical_clocks: Mutex<HashMap<DomainId, u64>>,
    
    /// Vector clocks by domain
    vector_clocks: Mutex<HashMap<DomainId, HashMap<DomainId, u64>>>,
    
    /// System key pair for signing attestations
    key_pair: KeyPair,
}

impl MemoryCausalTimeService {
    /// Create a new memory-based causal time service
    pub fn new(key_pair: KeyPair) -> Self {
        Self {
            logical_clocks: Mutex::new(HashMap::new()),
            vector_clocks: Mutex::new(HashMap::new()),
            key_pair,
        }
    }
    
    /// Initialize a domain's clocks if they don't exist
    fn ensure_domain_initialized(&self, domain_id: &DomainId) {
        let mut logical_clocks = self.logical_clocks.lock().unwrap();
        logical_clocks.entry(domain_id.clone()).or_insert(0);
        
        let mut vector_clocks = self.vector_clocks.lock().unwrap();
        vector_clocks.entry(domain_id.clone()).or_insert_with(HashMap::new);
    }
}

#[async_trait]
impl CausalTimeService for MemoryCausalTimeService {
    async fn get_logical_clock(&self, domain_id: &DomainId) -> Result<u64, TimeError> {
        self.ensure_domain_initialized(domain_id);
        let logical_clocks = self.logical_clocks.lock().unwrap();
        
        Ok(*logical_clocks.get(domain_id).unwrap_or(&0))
    }
    
    async fn get_vector_clock(&self, domain_id: &DomainId) -> Result<HashMap<DomainId, u64>, TimeError> {
        self.ensure_domain_initialized(domain_id);
        let vector_clocks = self.vector_clocks.lock().unwrap();
        
        Ok(vector_clocks.get(domain_id).cloned().unwrap_or_default())
    }
    
    async fn advance_logical_clock(&self, domain_id: &DomainId) -> Result<u64, TimeError> {
        self.ensure_domain_initialized(domain_id);
        let mut logical_clocks = self.logical_clocks.lock().unwrap();
        
        let clock = logical_clocks.entry(domain_id.clone()).or_insert(0);
        *clock += 1;
        
        // Also update the vector clock
        let mut vector_clocks = self.vector_clocks.lock().unwrap();
        let vclock = vector_clocks.entry(domain_id.clone()).or_insert_with(HashMap::new);
        vclock.insert(domain_id.clone(), *clock);
        
        Ok(*clock)
    }
    
    async fn update_vector_clock(
        &self, 
        domain_id: &DomainId,
        updates: HashMap<DomainId, u64>,
    ) -> Result<(), TimeError> {
        self.ensure_domain_initialized(domain_id);
        let mut vector_clocks = self.vector_clocks.lock().unwrap();
        
        let vclock = vector_clocks.entry(domain_id.clone()).or_insert_with(HashMap::new);
        
        // Update with max values
        for (update_domain, update_clock) in updates {
            let entry = vclock.entry(update_domain).or_insert(0);
            *entry = std::cmp::max(*entry, update_clock);
        }
        
        Ok(())
    }
    
    async fn create_causal_time_effect(
        &self,
        domain_id: &DomainId,
        dependencies: Vec<FactId>,
    ) -> Result<CausalTimeEffect, TimeError> {
        // Get current logical clock and advance it
        let current_clock = self.advance_logical_clock(domain_id).await?;
        
        // Get current vector clock
        let vector_clock = self.get_vector_clock(domain_id).await?;
        
        Ok(CausalTimeEffect {
            domain_id: domain_id.clone(),
            logical_clock: current_clock,
            vector_clock_updates: vector_clock,
            dependencies,
        })
    }
}

/// In-memory implementation of ClockTimeService
pub struct MemoryClockTimeService {
    /// Last attested time by domain
    attested_times: Mutex<HashMap<DomainId, DateTime<Utc>>>,
    
    /// System key pair for signing attestations
    key_pair: KeyPair,
}

impl MemoryClockTimeService {
    /// Create a new memory-based clock time service
    pub fn new(key_pair: KeyPair) -> Self {
        Self {
            attested_times: Mutex::new(HashMap::new()),
            key_pair,
        }
    }
}

#[async_trait]
impl ClockTimeService for MemoryClockTimeService {
    async fn get_current_time(&self) -> Result<DateTime<Utc>, TimeError> {
        Ok(Utc::now())
    }
    
    async fn get_time_attestation(&self) -> Result<TimeAttestation, TimeError> {
        let current_time = Utc::now();
        let time_str = current_time.to_rfc3339();
        
        // In a real implementation, we would sign the time
        // For now, create a simple signature
        let signature_data = format!("time:{}", time_str).into_bytes();
        let signature = self.key_pair.sign(&signature_data).map_err(|e| 
            TimeError::AttestationError(format!("Failed to sign time attestation: {}", e))
        )?;
        
        Ok(TimeAttestation {
            source: "system_time".to_string(),
            time: current_time,
            signature,
            public_key: self.key_pair.public_key(),
        })
    }
    
    async fn verify_attestation(&self, attestation: &TimeAttestation) -> Result<bool, TimeError> {
        // In a real implementation, we would verify the signature against trusted keys
        // For now, just verify that we can verify with the provided public key
        let time_str = attestation.time.to_rfc3339();
        let signature_data = format!("time:{}", time_str).into_bytes();
        
        let is_valid = attestation.public_key.verify(&signature_data, &attestation.signature)
            .map_err(|e| TimeError::AttestationError(format!("Failed to verify attestation: {}", e)))?;
        
        Ok(is_valid)
    }
    
    async fn create_clock_time_effect(
        &self,
        domain_id: &DomainId,
    ) -> Result<ClockTimeEffect, TimeError> {
        let current_time = Utc::now();
        let attestation = self.get_time_attestation().await?;
        
        // Record the attested time for this domain
        let mut attested_times = self.attested_times.lock().unwrap();
        attested_times.insert(domain_id.clone(), current_time);
        
        Ok(ClockTimeEffect {
            domain_id: domain_id.clone(),
            wall_time: current_time,
            time_source: TimeSource::LocalSystem,
            attestation: Some(attestation),
        })
    }
}

/// In-memory implementation of TimeAttestationStore
pub struct MemoryTimeAttestationStore {
    /// Attestations by domain
    attestations: Mutex<HashMap<DomainId, Vec<TimeAttestation>>>,
}

impl MemoryTimeAttestationStore {
    /// Create a new memory-based time attestation store
    pub fn new() -> Self {
        Self {
            attestations: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl TimeAttestationStore for MemoryTimeAttestationStore {
    async fn store_attestation(
        &self,
        domain_id: DomainId,
        attestation: TimeAttestation,
    ) -> Result<(), TimeError> {
        let mut attestations = self.attestations.lock().unwrap();
        let domain_attestations = attestations.entry(domain_id).or_insert_with(Vec::new);
        domain_attestations.push(attestation);
        Ok(())
    }
    
    async fn get_attestation(
        &self,
        domain_id: &DomainId,
    ) -> Result<Option<TimeAttestation>, TimeError> {
        let attestations = self.attestations.lock().unwrap();
        let domain_attestations = attestations.get(domain_id);
        
        // Return the most recent attestation if available
        if let Some(att_list) = domain_attestations {
            if !att_list.is_empty() {
                return Ok(Some(att_list.last().unwrap().clone()));
            }
        }
        
        Ok(None)
    }
    
    async fn get_attestations(
        &self,
        domain_id: &DomainId,
    ) -> Result<Vec<TimeAttestation>, TimeError> {
        let attestations = self.attestations.lock().unwrap();
        let domain_attestations = attestations.get(domain_id).cloned().unwrap_or_default();
        Ok(domain_attestations)
    }
}

/// In-memory implementation of FactTimeStore
pub struct MemoryFactTimeStore {
    /// Logical time by fact and domain
    logical_times: Mutex<HashMap<(FactId, DomainId), u64>>,
    
    /// Wall clock time by fact and domain
    wall_times: Mutex<HashMap<(FactId, DomainId), DateTime<Utc>>>,
    
    /// Dependencies by fact
    dependencies: Mutex<HashMap<FactId, Vec<FactId>>>,
    
    /// Dependents by fact (inverse of dependencies)
    dependents: Mutex<HashMap<FactId, Vec<FactId>>>,
}

impl MemoryFactTimeStore {
    /// Create a new memory-based fact time store
    pub fn new() -> Self {
        Self {
            logical_times: Mutex::new(HashMap::new()),
            wall_times: Mutex::new(HashMap::new()),
            dependencies: Mutex::new(HashMap::new()),
            dependents: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl FactTimeStore for MemoryFactTimeStore {
    async fn record_logical_time(
        &self,
        fact_id: &FactId,
        domain_id: &DomainId,
        logical_time: u64,
    ) -> Result<(), TimeError> {
        let mut logical_times = self.logical_times.lock().unwrap();
        logical_times.insert((fact_id.clone(), domain_id.clone()), logical_time);
        Ok(())
    }
    
    async fn record_wall_time(
        &self,
        fact_id: &FactId,
        domain_id: &DomainId,
        wall_time: DateTime<Utc>,
    ) -> Result<(), TimeError> {
        let mut wall_times = self.wall_times.lock().unwrap();
        wall_times.insert((fact_id.clone(), domain_id.clone()), wall_time);
        Ok(())
    }
    
    async fn get_logical_time(
        &self,
        fact_id: &FactId,
        domain_id: &DomainId,
    ) -> Result<Option<u64>, TimeError> {
        let logical_times = self.logical_times.lock().unwrap();
        let time = logical_times.get(&(fact_id.clone(), domain_id.clone())).copied();
        Ok(time)
    }
    
    async fn get_wall_time(
        &self,
        fact_id: &FactId,
        domain_id: &DomainId,
    ) -> Result<Option<DateTime<Utc>>, TimeError> {
        let wall_times = self.wall_times.lock().unwrap();
        let time = wall_times.get(&(fact_id.clone(), domain_id.clone())).copied();
        Ok(time)
    }
    
    async fn record_dependencies(
        &self,
        fact_id: &FactId,
        dependencies: &[FactId],
    ) -> Result<(), TimeError> {
        // Record dependencies
        {
            let mut deps = self.dependencies.lock().unwrap();
            deps.insert(fact_id.clone(), dependencies.to_vec());
        }
        
        // Record dependents (inverse relationship)
        {
            let mut deps = self.dependents.lock().unwrap();
            for dep_id in dependencies {
                let dependents = deps.entry(dep_id.clone()).or_insert_with(Vec::new);
                if !dependents.contains(fact_id) {
                    dependents.push(fact_id.clone());
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_dependencies(
        &self,
        fact_id: &FactId,
    ) -> Result<Vec<FactId>, TimeError> {
        let deps = self.dependencies.lock().unwrap();
        let fact_deps = deps.get(fact_id).cloned().unwrap_or_default();
        Ok(fact_deps)
    }
    
    async fn get_dependents(
        &self,
        fact_id: &FactId,
    ) -> Result<Vec<FactId>, TimeError> {
        let deps = self.dependents.lock().unwrap();
        let fact_deps = deps.get(fact_id).cloned().unwrap_or_default();
        Ok(fact_deps)
    }
}

/// Combined memory-based implementation of TimeService
pub struct MemoryTimeService {
    /// Causal time service
    causal_time: Arc<MemoryCausalTimeService>,
    
    /// Clock time service
    clock_time: Arc<MemoryClockTimeService>,
    
    /// Fact time store
    fact_store: Arc<MemoryFactTimeStore>,
}

impl MemoryTimeService {
    /// Create a new memory-based time service
    pub fn new(key_pair: KeyPair) -> Self {
        Self {
            causal_time: Arc::new(MemoryCausalTimeService::new(key_pair.clone())),
            clock_time: Arc::new(MemoryClockTimeService::new(key_pair)),
            fact_store: Arc::new(MemoryFactTimeStore::new()),
        }
    }
    
    /// Get the fact time store
    pub fn fact_store(&self) -> Arc<MemoryFactTimeStore> {
        self.fact_store.clone()
    }
}

#[async_trait]
impl TimeService for MemoryTimeService {
    fn causal_time(&self) -> &dyn CausalTimeService {
        &*self.causal_time
    }
    
    fn clock_time(&self) -> &dyn ClockTimeService {
        &*self.clock_time
    }
    
    async fn happened_before(
        &self,
        fact1: &FactId,
        fact2: &FactId,
    ) -> Result<bool, TimeError> {
        // Get all domains where both facts exist
        let domains = vec![]; // In a real impl, we would query for domains with both facts
        
        for domain_id in &domains {
            // Get logical times for the facts in this domain
            let time1 = self.fact_store.get_logical_time(fact1, domain_id).await?;
            let time2 = self.fact_store.get_logical_time(fact2, domain_id).await?;
            
            if let (Some(t1), Some(t2)) = (time1, time2) {
                if t1 < t2 {
                    return Ok(true);
                }
            }
        }
        
        // Check dependencies
        let deps2 = self.fact_store.get_dependencies(fact2).await?;
        if deps2.contains(fact1) {
            return Ok(true);
        }
        
        // Recursively check dependencies of dependencies
        for dep in &deps2 {
            if self.happened_before(fact1, dep).await? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn temporal_distance(
        &self,
        fact1: &FactId,
        fact2: &FactId,
    ) -> Result<TemporalDistance, TimeError> {
        // Check for direct causal relationship in logical time
        let domains = vec![]; // In a real impl, we would query for domains with both facts
        
        for domain_id in &domains {
            // Get logical times for the facts in this domain
            let time1 = self.fact_store.get_logical_time(fact1, domain_id).await?;
            let time2 = self.fact_store.get_logical_time(fact2, domain_id).await?;
            
            if let (Some(t1), Some(t2)) = (time1, time2) {
                if t1 < t2 {
                    return Ok(TemporalDistance::Causal(t2 - t1));
                }
            }
        }
        
        // Check for wall clock time difference
        let domains = vec![]; // In a real impl, we would query for domains with both facts
        
        for domain_id in &domains {
            // Get wall times for the facts in this domain
            let time1 = self.fact_store.get_wall_time(fact1, domain_id).await?;
            let time2 = self.fact_store.get_wall_time(fact2, domain_id).await?;
            
            if let (Some(t1), Some(t2)) = (time1, time2) {
                let duration = t2.signed_duration_since(t1);
                if !duration.num_nanoseconds().unwrap_or(0) < 0 {
                    return Ok(TemporalDistance::Temporal(std::time::Duration::from_nanos(
                        duration.num_nanoseconds().unwrap_or(0) as u64
                    )));
                }
            }
        }
        
        Ok(TemporalDistance::Unknown)
    }
    
    async fn get_timeline(
        &self,
        facts: &[FactId],
    ) -> Result<Vec<FactId>, TimeError> {
        // This is a simplified implementation - in reality this would be more complex
        // It would need to build a partial order from causal relationships
        
        // For now, just sort based on dependencies
        let mut result = facts.to_vec();
        result.sort_by(|a, b| {
            let a_happens_before_b = self.happened_before(a, b);
            let b_happens_before_a = self.happened_before(b, a);
            
            match (a_happens_before_b, b_happens_before_a) {
                (Ok(true), _) => std::cmp::Ordering::Less,
                (_, Ok(true)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
        
        Ok(result)
    }
    
    async fn are_concurrent(
        &self,
        facts: &[FactId],
    ) -> Result<bool, TimeError> {
        // Facts are concurrent if none of them has a happens-before relationship with any other
        for i in 0..facts.len() {
            for j in i+1..facts.len() {
                if self.happened_before(&facts[i], &facts[j]).await? {
                    return Ok(false);
                }
                if self.happened_before(&facts[j], &facts[i]).await? {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
} 