/// Time effect system
///
/// This module provides a standalone implementation for time-related effects that can be used
/// by both the domain and effects systems without creating cyclic dependencies.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::time_snapshot::{TimeEffect, TimeEffectResult, AttestationSource, TimeMapSnapshot};

/// Domain identifier type
pub type DomainId = String;

/// Domain position in time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainPosition {
    /// The timestamp in the domain
    pub timestamp: u64,
    
    /// The position index (for domains with the same timestamp)
    pub index: u32,
}

impl DomainPosition {
    /// Create a new domain position
    pub fn new(timestamp: u64, index: u32) -> Self {
        Self { timestamp, index }
    }
    
    /// Create a new domain position with a timestamp
    pub fn with_timestamp(timestamp: u64) -> Self {
        Self { timestamp, index: 0 }
    }
    
    /// Get the timestamp
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// Get the index
    pub fn get_index(&self) -> u32 {
        self.index
    }
    
    /// Check if this position is before another position
    pub fn is_before(&self, other: &Self) -> bool {
        self.timestamp < other.timestamp || (self.timestamp == other.timestamp && self.index < other.index)
    }
    
    /// Check if this position is after another position
    pub fn is_after(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp || (self.timestamp == other.timestamp && self.index > other.index)
    }
}

/// Map for tracking relative positions across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMap {
    /// Map of domain IDs to their positions
    positions: HashMap<DomainId, DomainPosition>,
    
    /// Domains with known comparable positions
    comparables: HashMap<DomainId, Vec<DomainId>>,
}

impl TimeMap {
    /// Create a new time map
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            comparables: HashMap::new(),
        }
    }
    
    /// Update the position of a domain
    pub fn update_position(&mut self, domain_id: &str, timestamp: u64) -> Option<DomainPosition> {
        let position = if let Some(existing) = self.positions.get(domain_id) {
            // Only update if timestamp is newer
            if timestamp > existing.timestamp {
                DomainPosition::with_timestamp(timestamp)
            } else {
                return None;
            }
        } else {
            DomainPosition::with_timestamp(timestamp)
        };
        
        self.positions.insert(domain_id.to_string(), position);
        Some(position)
    }
    
    /// Get the position of a domain
    pub fn get_position(&self, domain_id: &str) -> Option<DomainPosition> {
        self.positions.get(domain_id).copied()
    }
    
    /// Mark two domains as comparable
    pub fn mark_comparable(&mut self, domain_a: &str, domain_b: &str) {
        let domain_a = domain_a.to_string();
        let domain_b = domain_b.to_string();
        
        self.comparables
            .entry(domain_a.clone())
            .or_insert_with(Vec::new)
            .push(domain_b.clone());
            
        self.comparables
            .entry(domain_b)
            .or_insert_with(Vec::new)
            .push(domain_a);
    }
    
    /// Check if two domains are comparable
    pub fn are_comparable(&self, domain_a: &str, domain_b: &str) -> bool {
        if domain_a == domain_b {
            return true;
        }
        
        self.comparables
            .get(domain_a)
            .map(|vec| vec.contains(&domain_b.to_string()))
            .unwrap_or(false)
    }
    
    /// Create a snapshot of the current time map
    pub fn create_snapshot(&self) -> TimeMapSnapshot {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Convert our internal positions to the format required by TimeMapSnapshot
        let domain_timestamps: HashMap<String, u64> = self.positions
            .iter()
            .map(|(domain_id, position)| (domain_id.clone(), position.timestamp))
            .collect();
            
        // Create the snapshot
        TimeMapSnapshot::new(
            timestamp,
            domain_timestamps,
            vec![], // In a real implementation, we'd include causality edges
        )
    }
}

impl Default for TimeMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Defines the API for handling time-related effects
pub trait TimeEffectHandler: Send + Sync {
    /// Handle a causal time update
    fn handle_causal_update(
        &self, 
        operations: Vec<String>, 
        ordering: Vec<(String, String)>
    ) -> anyhow::Result<TimeEffectResult>;
    
    /// Handle a clock time attestation
    fn handle_clock_attestation(
        &self,
        domain_id: String,
        timestamp: u64,
        source: AttestationSource,
        confidence: f64,
    ) -> anyhow::Result<TimeEffectResult>;
    
    /// Handle a time map update
    fn handle_time_map_update(
        &self,
        positions: HashMap<String, u64>,
        proofs: HashMap<String, String>,
    ) -> anyhow::Result<TimeEffectResult>;
}

/// Basic time service implementation
pub struct TimeService {
    /// The time map for tracking relative positions across domains
    time_map: Arc<Mutex<TimeMap>>,
    /// The clock sources registered with this service
    clock_sources: Arc<Mutex<HashMap<String, ClockSourceInfo>>>,
}

/// Information about a clock source
#[derive(Debug, Clone)]
struct ClockSourceInfo {
    /// Latest timestamp from this source
    latest_timestamp: u64,
    /// Confidence level for this source
    confidence: f64,
    /// Source type
    source_type: String,
}

impl TimeService {
    /// Create a new time service
    pub fn new() -> Self {
        Self {
            time_map: Arc::new(Mutex::new(TimeMap::new())),
            clock_sources: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get a snapshot of the current time map
    pub fn time_map_snapshot(&self) -> anyhow::Result<TimeMapSnapshot> {
        let time_map = self.time_map.lock().map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        // Create a snapshot of the current time map
        Ok(time_map.create_snapshot())
    }
    
    /// Get the current timestamp for a given domain
    pub fn get_domain_timestamp(&self, domain_id: &str) -> anyhow::Result<Option<u64>> {
        let time_map = self.time_map.lock().map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        Ok(time_map.get_position(domain_id).map(|pos| pos.get_timestamp()))
    }
}

impl Default for TimeService {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeEffectHandler for TimeService {
    fn handle_causal_update(
        &self, 
        operations: Vec<String>, 
        ordering: Vec<(String, String)>
    ) -> anyhow::Result<TimeEffectResult> {
        // In a real implementation, we would update a causal graph here
        // For now, we'll just return a placeholder result
        
        Ok(TimeEffectResult::CausalUpdate {
            graph_hash: "placeholder_hash".to_string(),
            affected_operations: operations,
        })
    }
    
    fn handle_clock_attestation(
        &self,
        domain_id: String,
        timestamp: u64,
        source: AttestationSource,
        confidence: f64,
    ) -> anyhow::Result<TimeEffectResult> {
        // Update the clock source information
        let mut clock_sources = self.clock_sources.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock clock sources"))?;
        
        let source_type = match &source {
            AttestationSource::Blockchain { .. } => "blockchain",
            AttestationSource::User => "user",
            AttestationSource::Operator { .. } => "operator", 
            AttestationSource::Committee { .. } => "committee",
            AttestationSource::Oracle { .. } => "oracle",
            AttestationSource::NTP => "ntp",
            AttestationSource::External(_) => "external",
            AttestationSource::Consensus(_) => "consensus",
            AttestationSource::Custom(_) => "custom",
            AttestationSource::SystemClock => "system_clock",
        };
        
        clock_sources.insert(domain_id.clone(), ClockSourceInfo {
            latest_timestamp: timestamp,
            confidence,
            source_type: source_type.to_string(),
        });
        
        // Update the time map with the new timestamp
        let mut time_map = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        // Update the domain position in the time map
        time_map.update_position(&domain_id, timestamp);
        
        // Return the result
        Ok(TimeEffectResult::ClockUpdate {
            domain_id,
            timestamp,
            confidence,
        })
    }
    
    fn handle_time_map_update(
        &self,
        positions: HashMap<String, u64>,
        proofs: HashMap<String, String>,
    ) -> anyhow::Result<TimeEffectResult> {
        let mut time_map = self.time_map.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock time map"))?;
        
        // Update the time map with the new positions
        let domains_updated: Vec<String> = positions.keys().cloned().collect();
        
        for (domain_id, position) in positions {
            // In a real implementation, we'd validate the proofs
            // before updating the time map
            time_map.update_position(&domain_id, position);
        }
        
        // Return the result
        Ok(TimeEffectResult::TimeMapUpdate {
            map_hash: "placeholder_hash".to_string(),
            domains_updated,
        })
    }
}

/// Wrapper for TimeEffectHandler to make it usable as a trait object
pub struct TimeEffectHandlerWrapper {
    /// The underlying time effect handler
    handler: Arc<TimeService>,
}

impl TimeEffectHandlerWrapper {
    /// Create a new time effect handler wrapper
    pub fn new(handler: Arc<TimeService>) -> Self {
        Self { handler }
    }
    
    /// Handle a time effect
    pub fn handle(&self, effect: TimeEffect) -> anyhow::Result<TimeEffectResult> {
        match effect {
            TimeEffect::CausalUpdate { operations, ordering } => {
                self.handler.handle_causal_update(operations, ordering)
            },
            TimeEffect::ClockAttestation { domain_id, timestamp, source, confidence } => {
                self.handler.handle_clock_attestation(domain_id, timestamp, source, confidence)
            },
            TimeEffect::TimeMapUpdate { positions, proofs } => {
                self.handler.handle_time_map_update(positions, proofs)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_effect_handler() {
        // Create a time service
        let time_service = Arc::new(TimeService::new());
        
        // Create an effect handler
        let effect_handler = TimeEffectHandlerWrapper::new(time_service.clone());
        
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
        let result = effect_handler.handle(effect);
        
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
        if let Some(ts) = timestamp {
            assert_eq!(ts, 1234567890);
        }
    }
} 