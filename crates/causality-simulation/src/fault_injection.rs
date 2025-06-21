//! Fault injection for resilience testing

use std::collections::BTreeMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use crate::error::SimulationResult;

/// Types of faults that can be injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    /// Network failures
    NetworkPartition { duration_ms: u64 },
    NetworkLatency { additional_latency_ms: u64 },
    PacketLoss { probability: f64 },
    
    /// Resource failures
    ResourceExhaustion { resource_type: String },
    ResourceDelay { delay_ms: u64 },
    
    /// Effect execution failures
    EffectFailure { probability: f64 },
    EffectTimeout { timeout_ms: u64 },
    
    /// System failures
    ProcessCrash,
    MemoryCorruption { probability: f64 },
    
    /// Time-based failures
    ClockSkew { skew_ms: i64 },
    TimeoutExpiry,
}

/// Configuration for fault injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultConfig {
    pub fault_type: FaultType,
    pub target: String, // Component or operation to target
    pub probability: f64, // Probability of fault occurring (0.0 - 1.0)
    pub duration_ms: Option<u64>, // How long the fault lasts
    pub trigger_condition: Option<String>, // Condition to trigger fault
}

/// Manages fault injection during simulation
#[derive(Debug)]
pub struct FaultInjector {
    active_faults: BTreeMap<String, FaultConfig>,
    fault_history: Vec<FaultEvent>,
    rng: StdRng,
    enabled: bool,
}

/// Record of a fault that was injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultEvent {
    pub fault_id: String,
    pub fault_type: FaultType,
    pub target: String,
    pub timestamp: crate::clock::SimulatedTimestamp,
    pub duration_ms: Option<u64>,
    pub triggered: bool,
}

impl FaultInjector {
    /// Create a new fault injector with a random seed
    pub fn new() -> Self {
        Self::with_seed(rand::random())
    }
    
    /// Create a fault injector with a specific seed for deterministic testing
    pub fn with_seed(seed: u64) -> Self {
        Self {
            active_faults: BTreeMap::new(),
            fault_history: Vec::new(),
            rng: StdRng::seed_from_u64(seed),
            enabled: true,
        }
    }
    
    /// Enable or disable fault injection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Add a fault configuration
    pub fn add_fault(&mut self, fault_id: String, config: FaultConfig) -> SimulationResult<()> {
        if config.probability < 0.0 || config.probability > 1.0 {
            return Err(crate::error::SimulationError::FaultInjectionError(
                "Fault probability must be between 0.0 and 1.0".to_string()
            ));
        }
        
        self.active_faults.insert(fault_id, config);
        Ok(())
    }
    
    /// Remove a fault configuration
    pub fn remove_fault(&mut self, fault_id: &str) -> bool {
        self.active_faults.remove(fault_id).is_some()
    }
    
    /// Check if a fault should be triggered for a given target
    pub fn should_trigger_fault(&mut self, target: &str, timestamp: crate::clock::SimulatedTimestamp) -> Option<FaultType> {
        if !self.enabled {
            return None;
        }
        
        // Check all active faults for this target
        for (fault_id, config) in &self.active_faults {
            if config.target == target {
                let random_value: f64 = self.rng.gen();
                if random_value < config.probability {
                    // Record the fault event
                    let event = FaultEvent {
                        fault_id: fault_id.clone(),
                        fault_type: config.fault_type.clone(),
                        target: target.to_string(),
                        timestamp,
                        duration_ms: config.duration_ms,
                        triggered: true,
                    };
                    self.fault_history.push(event);
                    
                    return Some(config.fault_type.clone());
                }
            }
        }
        
        None
    }
    
    /// Inject a specific fault immediately
    pub fn inject_fault(&mut self, target: &str, fault_type: FaultType, timestamp: crate::clock::SimulatedTimestamp) {
        if !self.enabled {
            return;
        }
        
        let event = FaultEvent {
            fault_id: format!("manual_{}", self.fault_history.len()),
            fault_type,
            target: target.to_string(),
            timestamp,
            duration_ms: None,
            triggered: true,
        };
        self.fault_history.push(event);
    }
    
    /// Get the fault history
    pub fn get_fault_history(&self) -> &[FaultEvent] {
        &self.fault_history
    }
    
    /// Clear all faults and history
    pub fn clear(&mut self) {
        self.active_faults.clear();
        self.fault_history.clear();
    }
    
    /// Get statistics about injected faults
    pub fn get_statistics(&self) -> FaultStatistics {
        let total_faults = self.fault_history.len();
        let triggered_faults = self.fault_history.iter().filter(|e| e.triggered).count();
        
        let mut fault_type_counts = BTreeMap::new();
        for event in &self.fault_history {
            if event.triggered {
                let fault_type_name = match &event.fault_type {
                    FaultType::NetworkPartition { .. } => "NetworkPartition",
                    FaultType::NetworkLatency { .. } => "NetworkLatency",
                    FaultType::PacketLoss { .. } => "PacketLoss",
                    FaultType::ResourceExhaustion { .. } => "ResourceExhaustion",
                    FaultType::ResourceDelay { .. } => "ResourceDelay",
                    FaultType::EffectFailure { .. } => "EffectFailure",
                    FaultType::EffectTimeout { .. } => "EffectTimeout",
                    FaultType::ProcessCrash => "ProcessCrash",
                    FaultType::MemoryCorruption { .. } => "MemoryCorruption",
                    FaultType::ClockSkew { .. } => "ClockSkew",
                    FaultType::TimeoutExpiry => "TimeoutExpiry",
                };
                *fault_type_counts.entry(fault_type_name.to_string()).or_insert(0) += 1;
            }
        }
        
        FaultStatistics {
            total_faults,
            triggered_faults,
            fault_type_counts,
        }
    }
}

/// Statistics about fault injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultStatistics {
    pub total_faults: usize,
    pub triggered_faults: usize,
    pub fault_type_counts: BTreeMap<String, usize>,
}

impl Default for FaultInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::SimulatedTimestamp;
    
    #[test]
    fn test_fault_injector_basic() {
        let mut injector = FaultInjector::with_seed(42); // Deterministic seed
        let timestamp = SimulatedTimestamp::from_secs(1000);
        
        // Add a fault with 100% probability
        let config = FaultConfig {
            fault_type: FaultType::EffectFailure { probability: 1.0 },
            target: "test_target".to_string(),
            probability: 1.0,
            duration_ms: Some(5000),
            trigger_condition: None,
        };
        
        injector.add_fault("test_fault".to_string(), config).unwrap();
        
        // Should trigger the fault
        let result = injector.should_trigger_fault("test_target", timestamp);
        assert!(matches!(result, Some(FaultType::EffectFailure { .. })));
        
        // Check statistics
        let stats = injector.get_statistics();
        assert_eq!(stats.triggered_faults, 1);
    }
    
    #[test]
    fn test_fault_injection_disabled() {
        let mut injector = FaultInjector::with_seed(42);
        injector.set_enabled(false);
        
        let config = FaultConfig {
            fault_type: FaultType::ProcessCrash,
            target: "test_target".to_string(),
            probability: 1.0,
            duration_ms: None,
            trigger_condition: None,
        };
        
        injector.add_fault("test_fault".to_string(), config).unwrap();
        
        // Should not trigger when disabled
        let timestamp = SimulatedTimestamp::from_secs(1000);
        let result = injector.should_trigger_fault("test_target", timestamp);
        assert!(result.is_none());
    }
} 