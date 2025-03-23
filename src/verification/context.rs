// Verification Context Module
//
// This module defines the context for verification operations.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::types::{DomainId, Timestamp};
use crate::domain::map::map::TimeMap;
use super::{VerificationCapability, VerificationOptions, VerificationType};

/// Domain-specific context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainContext {
    /// Domain ID
    pub domain_id: DomainId,
    /// Latest known timestamp
    pub latest_timestamp: Timestamp,
    /// Domain-specific verification parameters
    pub parameters: HashMap<String, String>,
    /// Domain-specific capabilities
    pub capabilities: HashSet<VerificationCapability>,
}

impl DomainContext {
    /// Create a new domain context
    pub fn new(domain_id: DomainId, latest_timestamp: Timestamp) -> Self {
        Self {
            domain_id,
            latest_timestamp,
            parameters: HashMap::new(),
            capabilities: HashSet::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: VerificationCapability) -> Self {
        self.capabilities.insert(capability);
        self
    }
    
    /// Get parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

/// Controller registry for ancestral verification
#[derive(Debug, Clone, Default)]
pub struct ControllerRegistry {
    /// Known controller labels and their information
    controllers: HashMap<String, ControllerInfo>,
}

/// Information about a controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerInfo {
    /// Controller label
    pub label: String,
    /// Public key (if available)
    pub public_key: Option<Vec<u8>>,
    /// Parent controller label (if any)
    pub parent: Option<String>,
    /// Trust level (0.0 to 1.0)
    pub trust_level: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ControllerRegistry {
    /// Create a new controller registry
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
        }
    }
    
    /// Register a controller
    pub fn register_controller(&mut self, info: ControllerInfo) {
        self.controllers.insert(info.label.clone(), info);
    }
    
    /// Get controller info
    pub fn get_controller(&self, label: &str) -> Option<&ControllerInfo> {
        self.controllers.get(label)
    }
    
    /// Check if a controller exists
    pub fn has_controller(&self, label: &str) -> bool {
        self.controllers.contains_key(label)
    }
    
    /// Get the trust level for a controller
    pub fn get_trust_level(&self, label: &str) -> f64 {
        self.get_controller(label)
            .map(|info| info.trust_level)
            .unwrap_or(0.0)
    }
}

/// Effect history for logical verification
#[derive(Debug, Clone, Default)]
pub struct EffectHistory {
    // Placeholder for effect history
    // This would contain a record of effect executions
    // and their outcomes for logical verification
}

/// A prover implementation
pub trait Prover: Send + Sync {
    /// Generate a proof for the given subject and witness
    fn generate_proof(&self, subject: &[u8], witness: &[u8]) -> Result<Vec<u8>, String>;
    
    /// Verify a proof for the given subject
    fn verify_proof(&self, subject: &[u8], proof: &[u8]) -> Result<bool, String>;
    
    /// Get the prover name
    fn name(&self) -> &str;
}

/// Context for verification operations
#[derive(Debug, Clone)]
pub struct VerificationContext {
    /// Domain-specific context information
    pub domain_context: HashMap<DomainId, DomainContext>,
    
    /// Time map for temporal verification
    pub time_map: Arc<TimeMap>,
    
    /// Controller registry for ancestral verification
    pub controller_registry: Arc<ControllerRegistry>,
    
    /// Effect history for logical verification
    pub effect_history: Arc<EffectHistory>,
    
    /// Verification capabilities available in this context
    pub capabilities: HashSet<VerificationCapability>,
    
    /// Prover implementation to use
    pub prover: Option<Arc<dyn Prover>>,
    
    /// Verification options
    pub options: VerificationOptions,
}

impl VerificationContext {
    /// Create a new verification context
    pub fn new() -> Self {
        Self {
            domain_context: HashMap::new(),
            time_map: Arc::new(TimeMap::default()),
            controller_registry: Arc::new(ControllerRegistry::new()),
            effect_history: Arc::new(EffectHistory::default()),
            capabilities: HashSet::new(),
            prover: None,
            options: VerificationOptions::default(),
        }
    }
    
    /// Create a new verification context with time map
    pub fn with_time_map(time_map: Arc<TimeMap>, options: VerificationOptions) -> Self {
        Self {
            domain_context: HashMap::new(),
            time_map,
            controller_registry: Arc::new(ControllerRegistry::new()),
            effect_history: Arc::new(EffectHistory::default()),
            capabilities: HashSet::new(),
            prover: None,
            options,
        }
    }
    
    /// Add domain context
    pub fn with_domain_context(mut self, domain_context: DomainContext) -> Self {
        self.domain_context.insert(domain_context.domain_id.clone(), domain_context);
        self
    }
    
    /// Add domain context (mutable version)
    pub fn add_domain_context(&mut self, domain_context: DomainContext) {
        self.domain_context.insert(domain_context.domain_id.clone(), domain_context);
    }
    
    /// Get domain context
    pub fn get_domain_context(&self, domain_id: &DomainId) -> Option<&DomainContext> {
        self.domain_context.get(domain_id)
    }
    
    /// Set controller registry
    pub fn with_controller_registry(mut self, registry: Arc<ControllerRegistry>) -> Self {
        self.controller_registry = registry;
        self
    }
    
    /// Set effect history
    pub fn with_effect_history(mut self, history: Arc<EffectHistory>) -> Self {
        self.effect_history = history;
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: VerificationCapability) -> Self {
        self.capabilities.insert(capability);
        self
    }
    
    /// Add a capability (mutable version)
    pub fn add_capability(&mut self, capability: VerificationCapability) {
        self.capabilities.insert(capability);
    }
    
    /// Set prover
    pub fn with_prover(mut self, prover: Arc<dyn Prover>) -> Self {
        self.prover = Some(prover);
        self
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, capability: &VerificationCapability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Check if domain-specific capability is available
    pub fn has_domain_capability(&self, domain_id: &DomainId, capability: &VerificationCapability) -> bool {
        self.domain_context.get(domain_id)
            .map(|context| context.capabilities.contains(capability))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_context() {
        let domain_id = DomainId::new("test_domain".to_string());
        let timestamp = Timestamp::now();
        
        let context = DomainContext::new(domain_id.clone(), timestamp)
            .with_parameter("chain_id", "ethereum-1")
            .with_capability(VerificationCapability::ZkProving);
        
        assert_eq!(context.domain_id, domain_id);
        assert_eq!(context.latest_timestamp, timestamp);
        assert_eq!(context.parameters.get("chain_id"), Some(&"ethereum-1".to_string()));
        assert!(context.capabilities.contains(&VerificationCapability::ZkProving));
    }
    
    #[test]
    fn test_controller_registry() {
        let mut registry = ControllerRegistry::new();
        
        let controller_info = ControllerInfo {
            label: "controller1".to_string(),
            public_key: Some(vec![1, 2, 3, 4]),
            parent: None,
            trust_level: 0.9,
            metadata: HashMap::new(),
        };
        
        registry.register_controller(controller_info);
        
        assert!(registry.has_controller("controller1"));
        assert_eq!(registry.get_trust_level("controller1"), 0.9);
        assert_eq!(registry.get_trust_level("unknown"), 0.0);
    }
} 