// TEL adapter registry
// Original file: src/tel/adapter/registry.rs

// Registry for domain adapters
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_tel::{Effect, DomainId};
use causality_tel::{TelError, TelResult};
use super::traits::{EffectCompiler, CompilerContext, CompilationResult};
use super::common::ValidationError;

/// Configuration for a domain adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Domain ID this adapter handles
    pub domain_id: String,
    /// Name of the adapter
    pub name: String,
    /// Version of the adapter
    pub version: String,
    /// Optional connection configuration
    pub connection: Option<ConnectionConfig>,
    /// Additional adapter-specific parameters
    pub parameters: HashMap<String, String>,
}

/// Connection configuration for external adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Type of connection
    pub connection_type: ConnectionType,
    /// Connection endpoint (URL, path, etc.)
    pub endpoint: String,
    /// Authentication parameters
    pub auth: Option<HashMap<String, String>>,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether to use secure connection
    pub secure: bool,
}

/// Types of adapter connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    /// Direct connection (in-process)
    Direct,
    /// HTTP connection
    Http,
    /// WebSocket connection
    WebSocket,
    /// gRPC connection
    Grpc,
    /// IPC connection
    Ipc,
}

/// Metadata about an adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterMetadata {
    /// Name of the adapter
    pub name: String,
    /// Version of the adapter
    pub version: String,
    /// Domain ID this adapter handles
    pub domain_id: String,
    /// Description of the adapter
    pub description: String,
    /// Effects supported by this adapter
    pub supported_effects: Vec<String>,
    /// Status of the adapter
    pub status: AdapterStatus,
}

/// Status of an adapter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdapterStatus {
    /// Adapter is available and ready
    Available,
    /// Adapter is connecting or initializing
    Connecting,
    /// Adapter is temporarily unavailable
    Unavailable,
    /// Adapter has errors
    Error,
}

/// Registry for domain adapters
///
/// This registry maintains a collection of domain adapters that can compile
/// TEL effects into domain-specific formats. It allows registering, unregistering,
/// and querying adapters by domain ID.
pub struct AdapterRegistry {
    /// Map of domain IDs to adapter entries
    adapters: RwLock<HashMap<String, AdapterEntry>>,
    /// Default context for compilations
    default_context: CompilerContext,
}

/// Internal entry for an adapter in the registry
struct AdapterEntry {
    /// Reference to the adapter
    adapter: Arc<dyn EffectCompiler<Output = Vec<u8>>>,
    /// Configuration for the adapter
    config: AdapterConfig,
    /// Metadata for the adapter
    metadata: AdapterMetadata,
}

impl AdapterRegistry {
    /// Create a new adapter registry
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            default_context: CompilerContext {
                domain_parameters: HashMap::new(),
                resource_ids: HashMap::new(),
                chain_context: None,
                options: CompilationOptions::default(),
            },
        }
    }
    
    /// Register a new adapter
    pub fn register(&mut self, adapter: Box<dyn EffectCompiler<Output = Vec<u8>>>) -> Result<(), TelError> {
        let adapter = Arc::from(adapter);
        let metadata = adapter.metadata();
        let domain_id = metadata.domain_id.clone();
        
        let config = AdapterConfig {
            domain_id: domain_id.clone(),
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            connection: None,
            parameters: HashMap::new(),
        };
        
        let entry = AdapterEntry {
            adapter,
            config,
            metadata,
        };
        
        let mut adapters = self.adapters.write().unwrap();
        adapters.insert(domain_id, entry);
        
        Ok(())
    }
    
    /// Unregister an adapter
    pub fn unregister(&mut self, domain_id: &str) -> bool {
        let mut adapters = self.adapters.write().unwrap();
        adapters.remove(domain_id).is_some()
    }
    
    /// Get the best adapter for a domain
    pub fn get_adapter(&self, domain_id: &str) -> Result<Arc<dyn EffectCompiler<Output = Vec<u8>>>, TelError> {
        let adapters = self.adapters.read().unwrap();
        match adapters.get(domain_id) {
            Some(entry) => Ok(entry.adapter.clone()),
            None => Err(TelError::AdapterNotFound(domain_id.to_string())),
        }
    }
    
    /// Get adapter by name and version
    pub fn get_adapter_by_name(&self, name: &str, version: Option<&str>) -> Result<Arc<dyn EffectCompiler<Output = Vec<u8>>>, TelError> {
        let adapters = self.adapters.read().unwrap();
        
        for entry in adapters.values() {
            if entry.metadata.name == name {
                if let Some(version_str) = version {
                    if entry.metadata.version == version_str {
                        return Ok(entry.adapter.clone());
                    }
                } else {
                    return Ok(entry.adapter.clone());
                }
            }
        }
        
        Err(TelError::AdapterNotFound(format!("{}:{}", name, version.unwrap_or("any"))))
    }
    
    /// Update adapter status
    pub fn update_status(&mut self, domain_id: &str, status: AdapterStatus) -> Result<(), TelError> {
        let mut adapters = self.adapters.write().unwrap();
        
        match adapters.get_mut(domain_id) {
            Some(entry) => {
                entry.metadata.status = status;
                Ok(())
            },
            None => Err(TelError::AdapterNotFound(domain_id.to_string())),
        }
    }
    
    /// List all adapters
    pub fn list_adapters(&self) -> Vec<AdapterMetadata> {
        let adapters = self.adapters.read().unwrap();
        adapters.values().map(|entry| entry.metadata.clone()).collect()
    }
    
    /// Compile an effect using the appropriate adapter
    pub fn compile_effect(&self, effect: &Effect, context: Option<CompilerContext>) -> Result<CompilationResult, TelError> {
        let domain_id = match effect {
            Effect::Deposit(e) => &e.domain,
            Effect::Withdraw(e) => &e.domain,
            Effect::Transfer(e) => &e.domain,
            Effect::Call(e) => &e.domain,
            Effect::Deploy(e) => &e.domain,
            Effect::Create(e) => &e.domain,
            Effect::Update(e) => &e.domain,
            Effect::Delete(e) => &e.domain,
            Effect::Query(e) => &e.domain,
            Effect::Fact(e) => &e.domain,
            Effect::Lock(e) => &e.domain,
            Effect::Unlock(e) => &e.domain,
            Effect::SignMessage(e) => &e.domain,
            Effect::VerifySignature(e) => &e.domain,
            Effect::Prove(e) => &e.domain,
            Effect::Verify(e) => &e.domain,
            Effect::Composite(e) => return self.compile_composite(e, context),
            Effect::Conditional(e) => return self.compile_conditional(e, context),
            Effect::Authorized(e) => return self.compile_authorized(e, context),
            Effect::Timed(e) => return self.compile_timed(e, context),
        };
        
        let adapter = self.get_adapter(domain_id)?;
        let ctx = context.unwrap_or_else(|| self.default_context.clone());
        
        adapter.compile(effect, &ctx)
    }
    
    /// Compile a composite effect
    fn compile_composite(&self, effect: &CompositeEffect, context: Option<CompilerContext>) -> Result<CompilationResult, TelError> {
        // For simplicity, we'll just compile the first effect in the sequence
        // In a real implementation, we'd compile all effects and potentially combine them
        if let Some(first_effect) = effect.effects.first() {
            return self.compile_effect(first_effect, context);
        }
        
        Err(TelError::CompilationError("Empty composite effect".to_string()))
    }
    
    /// Compile a conditional effect
    fn compile_conditional(&self, effect: &ConditionalEffect, context: Option<CompilerContext>) -> Result<CompilationResult, TelError> {
        // For simplicity, we'll just compile the underlying effect
        // In a real implementation, we'd handle the condition appropriately
        self.compile_effect(&effect.effect, context)
    }
    
    /// Compile an authorized effect
    fn compile_authorized(&self, effect: &AuthorizedEffect, context: Option<CompilerContext>) -> Result<CompilationResult, TelError> {
        // For simplicity, we'll just compile the underlying effect
        // In a real implementation, we'd handle the authorization appropriately
        self.compile_effect(&effect.effect, context)
    }
    
    /// Compile a timed effect
    fn compile_timed(&self, effect: &TimedEffect, context: Option<CompilerContext>) -> Result<CompilationResult, TelError> {
        // For simplicity, we'll just compile the underlying effect
        // In a real implementation, we'd handle the timing appropriately
        self.compile_effect(&effect.effect, context)
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
} 