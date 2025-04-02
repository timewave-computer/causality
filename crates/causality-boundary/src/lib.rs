// Boundary system for cross-domain communication
// Original file: src/boundary/mod.rs

pub mod annotation;
pub mod metrics;
pub mod crossing;

#[cfg(feature = "on_chain")]
pub mod on_chain;

#[cfg(feature = "off_chain")]
pub mod off_chain;

use std::sync::Arc;
use serde::{Serialize, Deserialize};

// Only re-export what we need, being careful to avoid name conflicts
pub use annotation::{
    boundary,
    boundary_crossing,
    BoundarySafe,
    BoundaryTracker,
};

// Re-export boundary types with aliases to avoid conflicts
pub use annotation::{
    BoundaryType as AnnotationBoundaryType,
    CrossingType as AnnotationCrossingType
};

pub use crossing::{
    BoundaryCrossingError,
    BoundaryCrossingResult,
    BoundaryAuthentication,
    BoundaryCrossingPayload,
    BoundaryCrossingProtocol,
    BoundaryCrossingProtocolExt,
    BoundaryCrossingRegistry,
    DefaultBoundaryCrossingProtocol,
};

#[cfg(feature = "on_chain")]
pub use on_chain::{
    OnChainEnvironment,
    ChainAddress,
    ContractInterface,
    ContractCallData,
    ContractCallResult,
    ContractCallProtocol,
    ContractCallAdapter,
};

#[cfg(feature = "off_chain")]
pub use off_chain::{
    OffChainComponentType,
    ComponentId,
    ComponentConfig,
    ConnectionDetails,
    SecuritySettings,
    OffChainComponent,
    ComponentRequest,
    ComponentResponse,
    OffChainComponentProtocol,
    OffChainComponentAdapter,
    OffChainComponentRegistry,
};

/// The global boundary system 
#[derive(Clone)]
pub struct BoundarySystem {
    /// Registry for boundary crossing protocols
    crossing_registry: Arc<BoundaryCrossingRegistry>,
    
    /// Registry for off-chain components
    #[cfg(feature = "off_chain")]
    off_chain_registry: Arc<OffChainComponentRegistry>,
    
    /// Default on-chain adapters for different environments
    #[cfg(feature = "on_chain")]
    on_chain_adapters: std::collections::HashMap<OnChainEnvironment, Arc<ContractCallAdapter>>,
}

/// Configuration for the boundary system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySystemConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    
    /// Enable size limiting
    pub enable_size_limiting: bool,
    
    /// Maximum payload size in bytes
    pub max_payload_size: usize,
    
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Default authentication method
    pub default_auth_method: String,
    
    /// Supported on-chain environments
    #[cfg(feature = "on_chain")]
    pub supported_on_chain_environments: Vec<OnChainEnvironment>,
}

impl Default for BoundarySystemConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            enable_size_limiting: true,
            max_payload_size: 1024 * 1024, // 1MB
            enable_metrics: true,
            default_auth_method: "capability".to_string(),
            #[cfg(feature = "on_chain")]
            supported_on_chain_environments: vec![
                OnChainEnvironment::EVM,
                OnChainEnvironment::CosmWasm,
            ],
        }
    }
}

impl BoundarySystem {
    /// Create a new boundary system with default configuration
    pub fn new() -> Self {
        Self::with_config(BoundarySystemConfig::default())
    }
    
    /// Create a new boundary system with custom configuration
    pub fn with_config(config: BoundarySystemConfig) -> Self {
        let crossing_registry = Arc::new(BoundaryCrossingRegistry::new());
        
        #[cfg(feature = "off_chain")]
        let off_chain_registry = Arc::new(OffChainComponentRegistry::new());
        
        #[cfg(feature = "on_chain")]
        let mut on_chain_adapters = std::collections::HashMap::new();
        
        // Initialize on-chain adapters for each supported environment
        #[cfg(feature = "on_chain")]
        for env in &config.supported_on_chain_environments {
            let protocol = Arc::new(ContractCallProtocol::new(
                &format!("{:?}_protocol", env),
                *env,
            ));
            let adapter = Arc::new(ContractCallAdapter::new(protocol));
            on_chain_adapters.insert(*env, adapter);
        }
        
        // Register default crossing protocols
        let inside_to_outside = Arc::new(DefaultBoundaryCrossingProtocol::new(
            "inside_to_outside",
            annotation::BoundaryType::InsideSystem,
            annotation::BoundaryType::OutsideSystem,
            config.max_payload_size,
        ));
        
        let outside_to_inside = Arc::new(DefaultBoundaryCrossingProtocol::new(
            "outside_to_inside",
            annotation::BoundaryType::OutsideSystem,
            annotation::BoundaryType::InsideSystem,
            config.max_payload_size,
        ));
        
        crossing_registry.register_protocol(inside_to_outside);
        crossing_registry.register_protocol(outside_to_inside);
        
        #[cfg(all(feature = "on_chain", feature = "off_chain"))]
        {
            Self {
                crossing_registry,
                off_chain_registry,
                on_chain_adapters,
            }
        }
        
        #[cfg(all(feature = "on_chain", not(feature = "off_chain")))]
        {
            Self {
                crossing_registry,
                on_chain_adapters,
            }
        }
        
        #[cfg(all(not(feature = "on_chain"), feature = "off_chain"))]
        {
            Self {
                crossing_registry,
                off_chain_registry,
            }
        }
        
        #[cfg(not(any(feature = "on_chain", feature = "off_chain")))]
        {
            Self {
                crossing_registry,
            }
        }
    }
    
    /// Get the crossing registry
    pub fn crossing_registry(&self) -> Arc<BoundaryCrossingRegistry> {
        self.crossing_registry.clone()
    }
    
    /// Get the off-chain component registry
    #[cfg(feature = "off_chain")]
    pub fn off_chain_registry(&self) -> Arc<OffChainComponentRegistry> {
        self.off_chain_registry.clone()
    }
    
    /// Get an on-chain adapter for a specific environment
    #[cfg(feature = "on_chain")]
    pub fn on_chain_adapter(&self, env: OnChainEnvironment) -> Option<Arc<ContractCallAdapter>> {
        self.on_chain_adapters.get(&env).cloned()
    }
    
    /// Register a new crossing protocol
    pub fn register_crossing_protocol(&self, protocol: Arc<DefaultBoundaryCrossingProtocol>) {
        self.crossing_registry.register_protocol(protocol);
    }
    
    /// Register an off-chain component
    #[cfg(feature = "off_chain")]
    pub fn register_off_chain_component(&self, component: Arc<dyn OffChainComponent>) {
        self.off_chain_registry.register(component);
    }
    
    /// Initialize the boundary system
    pub async fn initialize(&self) -> Result<(), String> {
        // Initialize all off-chain components
        #[cfg(feature = "off_chain")]
        self.off_chain_registry.initialize_all().await?;
        
        Ok(())
    }
    
    /// Shutdown the boundary system
    pub async fn shutdown(&self) -> Result<(), String> {
        // Close all off-chain components
        #[cfg(feature = "off_chain")]
        self.off_chain_registry.close_all().await?;
        
        Ok(())
    }
    
    /// Export metrics as JSON
    pub fn export_metrics(&self) -> String {
        metrics::export_metrics_json()
    }
    
    /// Reset metrics
    pub fn reset_metrics(&self) {
        metrics::reset_metrics();
    }
}

/// Initialize a global boundary system instance
pub fn init_boundary_system() -> Arc<BoundarySystem> {
    let system = BoundarySystem::new();
    Arc::new(system)
}

/// Get the global boundary system
pub fn boundary_system() -> Arc<BoundarySystem> {
    use std::sync::OnceLock;
    static BOUNDARY_SYSTEM: OnceLock<Arc<BoundarySystem>> = OnceLock::new();
    
    BOUNDARY_SYSTEM.get_or_init(init_boundary_system).clone()
}

/// Boundary Module
///
/// This module defines boundary types and crossing operations in the system.
/// Boundaries represent different execution/storage contexts such as inside the system,
/// outside the system, on-chain, or off-chain.

/// Handles operations related to boundary crossings
pub struct BoundaryManager;

impl BoundaryManager {
    /// Create a new boundary manager
    pub fn new() -> Self {
        Self
    }
    
    /// Check if a crossing between two boundaries is allowed by system policy
    pub fn is_crossing_allowed(&self, from: annotation::BoundaryType, to: annotation::BoundaryType) -> bool {
        match (from, to) {
            // Prevent crossing from external to internal if not explicitly allowed
            (annotation::BoundaryType::OutsideSystem, annotation::BoundaryType::InsideSystem) => false,
            
            // By default allow all other crossings
            _ => true,
        }
    }
    
    /// Get the crossing type for a boundary crossing
    pub fn get_crossing_type(&self, from: annotation::BoundaryType, to: annotation::BoundaryType) -> annotation::CrossingType {
        // Clone the from and to values before the match to avoid moving them
        let from_clone = from.clone();
        let to_clone = to.clone();
        
        match (from, to) {
            (annotation::BoundaryType::InsideSystem, annotation::BoundaryType::OutsideSystem) => 
                annotation::CrossingType::InsideToOutside,
            (annotation::BoundaryType::OutsideSystem, annotation::BoundaryType::InsideSystem) => 
                annotation::CrossingType::OutsideToInside,
            (annotation::BoundaryType::OffChain, annotation::BoundaryType::OnChain) => 
                annotation::CrossingType::OffChainToOnChain,
            (annotation::BoundaryType::OnChain, annotation::BoundaryType::OffChain) => 
                annotation::CrossingType::OnChainToOffChain,
            // Handle Local boundary type
            (annotation::BoundaryType::Local, _) | (_, annotation::BoundaryType::Local) => 
                annotation::CrossingType::Custom(format!("{:?}_to_{:?}", from_clone, to_clone)),
            // Handle custom boundary types
            _ => annotation::CrossingType::Custom(format!("{:?}_to_{:?}", from_clone, to_clone)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boundary_crossing_types() {
        let manager = BoundaryManager::new();
        
        assert_eq!(
            manager.get_crossing_type(annotation::BoundaryType::InsideSystem, annotation::BoundaryType::OnChain),
            annotation::CrossingType::Custom("InsideSystem_to_OnChain".to_string())
        );
        
        assert_eq!(
            manager.get_crossing_type(annotation::BoundaryType::InsideSystem, annotation::BoundaryType::OutsideSystem),
            annotation::CrossingType::InsideToOutside
        );
    }
    
    #[test]
    fn test_crossing_permissions() {
        let manager = BoundaryManager::new();
        
        // By default, outside to inside should not be allowed
        assert!(!manager.is_crossing_allowed(annotation::BoundaryType::OutsideSystem, annotation::BoundaryType::InsideSystem));
        
        // But inside to outside should be allowed
        assert!(manager.is_crossing_allowed(annotation::BoundaryType::InsideSystem, annotation::BoundaryType::OutsideSystem));
    }
} 