pub mod annotation;
pub mod metrics;
pub mod crossing;

#[cfg(feature = "on_chain")]
pub mod on_chain;

#[cfg(feature = "off_chain")]
pub mod off_chain;

use std::sync::Arc;
use serde::{Serialize, Deserialize};

pub use annotation::{
    BoundaryType, 
    CrossingType, 
    BoundarySafe,
    BoundaryTracker,
    boundary, 
    boundary_crossing
};

pub use crossing::{
    BoundaryCrossingError,
    BoundaryCrossingResult,
    BoundaryAuthentication,
    BoundaryCrossingPayload,
    BoundaryCrossingProtocol,
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
            BoundaryType::InsideSystem,
            BoundaryType::OutsideSystem,
            config.max_payload_size,
        ));
        
        let outside_to_inside = Arc::new(DefaultBoundaryCrossingProtocol::new(
            "outside_to_inside",
            BoundaryType::OutsideSystem,
            BoundaryType::InsideSystem,
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
    pub fn register_crossing_protocol(&self, protocol: Arc<dyn BoundaryCrossingProtocol>) {
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