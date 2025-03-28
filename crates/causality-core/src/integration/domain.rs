// Domain Integration Layer
//
// This module provides domain-specific adapters for integrating various domains
// with the effect system and cross-domain resource protocol.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;
use causality_types::ContentId;

use crate::capability::{Capability, Right};
use crate::resource::{
    ResourceTypeId, CrossDomainResourceId, ResourceProjectionType,
    VerificationLevel, ResourceReference, ResourceTransferOperation,
    CrossDomainResourceProtocol, DomainResourceAdapter,
    CrossDomainProtocolError, CrossDomainProtocolResult,
    Resource,
};
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    domain::{
        DomainId, DomainEffect, DomainEffectHandler, DomainEffectOutcome,
        DomainCapabilityMapping, ParameterValidationResult, DomainParameterValidator,
        EnhancedDomainContextAdapter, EnhancedDomainEffectHandler,
    },
};

/// Domain integration error
#[derive(Error, Debug)]
pub enum DomainIntegrationError {
    #[error("Domain not supported: {0}")]
    DomainNotSupported(String),
    
    #[error("Resource type not supported: {0}")]
    ResourceTypeNotSupported(String),
    
    #[error("Capability error: {0}")]
    CapabilityError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),
    
    #[error("Integration error: {0}")]
    IntegrationError(String),
}

/// Domain integration result
pub type DomainIntegrationResult<T> = Result<T, DomainIntegrationError>;

/// Domain adapter factory for creating domain-specific adapters
#[async_trait]
pub trait DomainAdapterFactory: Send + Sync + Debug {
    /// Create a domain effect handler for the specified domain
    async fn create_effect_handler(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainEffectHandler>>;
    
    /// Create a domain resource adapter for the specified domain
    async fn create_resource_adapter(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainResourceAdapter>>;
    
    /// Get supported domains
    fn supported_domains(&self) -> Vec<DomainId>;
    
    /// Check if a domain is supported
    fn is_domain_supported(&self, domain_id: &DomainId) -> bool {
        self.supported_domains().iter().any(|d| d == domain_id)
    }
}

/// Domain validation helpers
pub struct DomainValidation;

impl DomainValidation {
    /// Validate resource types for a domain
    pub fn validate_resource_type(
        domain_id: &DomainId,
        resource_type: &ResourceTypeId,
        supported_types: &[ResourceTypeId],
    ) -> DomainIntegrationResult<()> {
        if !supported_types.iter().any(|t| t.is_compatible_with(resource_type)) {
            return Err(DomainIntegrationError::ResourceTypeNotSupported(
                format!("Resource type {} not supported in domain {}", resource_type, domain_id)
            ));
        }
        
        Ok(())
    }
    
    /// Validate domain capabilities
    pub fn validate_capabilities(
        required: &[Capability],
        provided: &[Capability],
    ) -> DomainIntegrationResult<()> {
        for req in required {
            if !provided.iter().any(|p| p.satisfies(req)) {
                return Err(DomainIntegrationError::CapabilityError(
                    format!("Missing required capability: {:?}", req)
                ));
            }
        }
        
        Ok(())
    }
}

/// Generic domain adapter base implementation
pub struct GenericDomainAdapter {
    /// Domain ID
    domain_id: DomainId,
    
    /// Supported resource types
    supported_resource_types: Vec<ResourceTypeId>,
    
    /// Required capabilities for operations
    required_capabilities: HashMap<String, Vec<Capability<dyn Resource>>>,
    
    /// Parameter validators
    parameter_validators: Vec<Arc<dyn DomainParameterValidator>>,
    
    /// Domain capability mappings
    capability_mappings: HashMap<DomainId, LocalDomainCapabilityMapping>,
}

impl GenericDomainAdapter {
    /// Create a new generic domain adapter
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            supported_resource_types: Vec::new(),
            required_capabilities: HashMap::new(),
            parameter_validators: Vec::new(),
            capability_mappings: HashMap::new(),
        }
    }
    
    /// Add supported resource type
    pub fn add_resource_type(&mut self, resource_type: ResourceTypeId) -> &mut Self {
        self.supported_resource_types.push(resource_type);
        self
    }
    
    /// Add required capability for an operation
    pub fn add_required_capability(
        &mut self, 
        operation: impl Into<String>, 
        capability: Capability<dyn Resource>,
    ) -> &mut Self {
        let op = operation.into();
        self.required_capabilities
            .entry(op)
            .or_insert_with(Vec::new)
            .push(capability);
        self
    }
    
    /// Add parameter validator
    pub fn add_parameter_validator(
        &mut self, 
        validator: Arc<dyn DomainParameterValidator>,
    ) -> &mut Self {
        self.parameter_validators.push(validator);
        self
    }
    
    /// Add capability mapping
    pub fn add_capability_mapping(
        &mut self, 
        mapping: LocalDomainCapabilityMapping,
    ) -> &mut Self {
        self.capability_mappings.insert(mapping.domain_id.clone(), mapping);
        self
    }
    
    /// Get domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get supported resource types
    pub fn supported_resource_types(&self) -> &[ResourceTypeId] {
        &self.supported_resource_types
    }
    
    /// Validate resource type is supported
    pub fn validate_resource_type(&self, resource_type: &ResourceTypeId) -> DomainIntegrationResult<()> {
        DomainValidation::validate_resource_type(
            &self.domain_id,
            resource_type,
            &self.supported_resource_types,
        )
    }
    
    /// Validate capabilities for an operation
    pub fn validate_capabilities(
        &self,
        operation: &str,
        provided: &[Capability<dyn Resource>],
    ) -> DomainIntegrationResult<()> {
        if let Some(required) = self.required_capabilities.get(operation) {
            DomainValidation::validate_capabilities(required, provided)
        } else {
            Ok(())
        }
    }
    
    /// Validate parameters using all registered validators
    pub fn validate_parameters(&self, parameters: &HashMap<String, String>) -> ParameterValidationResult {
        let mut result = ParameterValidationResult::success(HashMap::new());
        
        for validator in &self.parameter_validators {
            result = result.combine(validator.validate(parameters));
        }
        
        result
    }
    
    /// Create an enhanced domain context adapter from this adapter
    pub fn create_context_adapter(&self) -> EnhancedDomainContextAdapter {
        let mut adapter = EnhancedDomainContextAdapter::new(self.domain_id.clone());
        
        // Add capability mappings
        for mapping in self.capability_mappings.values() {
            adapter.add_mapping(mapping.clone());
        }
        
        // Add parameter validators
        for validator in &self.parameter_validators {
            adapter.add_parameter_validator(validator.clone());
        }
        
        adapter
    }
}

/// Domain effect router that routes domain effects to appropriate handlers
pub struct DomainEffectRouter {
    /// Domain handlers by domain ID
    handlers: HashMap<DomainId, Arc<dyn DomainEffectHandler>>,
    
    /// Domain adapter factory
    adapter_factory: Arc<dyn DomainAdapterFactory>,
}

impl DomainEffectRouter {
    /// Create a new domain effect router
    pub fn new(adapter_factory: Arc<dyn DomainAdapterFactory>) -> Self {
        Self {
            handlers: HashMap::new(),
            adapter_factory,
        }
    }
    
    /// Add a domain handler
    pub fn add_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) -> &mut Self {
        self.handlers.insert(handler.domain_id().clone(), handler);
        self
    }
    
    /// Get a handler for the domain
    pub async fn get_handler(&self, domain_id: &DomainId) -> DomainIntegrationResult<Arc<dyn DomainEffectHandler>> {
        // Check if we already have a handler
        if let Some(handler) = self.handlers.get(domain_id) {
            return Ok(handler.clone());
        }
        
        // Create a new handler using the factory
        self.adapter_factory.create_effect_handler(domain_id).await
    }
    
    /// Route a domain effect to the appropriate handler
    pub async fn route_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let domain_id = effect.domain_id();
        
        // Get the handler for this domain
        let handler = self.get_handler(domain_id).await
            .map_err(|e| EffectError::ExecutionError(e.to_string()))?;
        
        // Create an adapted context
        let adapted_context = effect.adapt_context(context)?;
        
        // Handle the effect
        handler.handle_domain_effect(effect, adapted_context.as_ref()).await
    }
}

/// Domain resource router that routes resource operations to appropriate adapters
pub struct DomainResourceRouter {
    /// Domain resource adapters by domain ID
    adapters: HashMap<DomainId, Arc<dyn DomainResourceAdapter>>,
    
    /// Domain adapter factory
    adapter_factory: Arc<dyn DomainAdapterFactory>,
    
    /// Cross-domain resource protocol
    cross_domain_protocol: Arc<dyn CrossDomainResourceProtocol>,
}

impl DomainResourceRouter {
    /// Create a new domain resource router
    pub fn new(
        adapter_factory: Arc<dyn DomainAdapterFactory>,
        cross_domain_protocol: Arc<dyn CrossDomainResourceProtocol>,
    ) -> Self {
        Self {
            adapters: HashMap::new(),
            adapter_factory,
            cross_domain_protocol,
        }
    }
    
    /// Add a domain resource adapter
    pub fn add_adapter(&mut self, adapter: Arc<dyn DomainResourceAdapter>) -> &mut Self {
        self.adapters.insert(adapter.domain_id().clone(), adapter);
        self
    }
    
    /// Get an adapter for the domain
    pub async fn get_adapter(&self, domain_id: &DomainId) -> DomainIntegrationResult<Arc<dyn DomainResourceAdapter>> {
        // Check if we already have an adapter
        if let Some(adapter) = self.adapters.get(domain_id) {
            return Ok(adapter.clone());
        }
        
        // Create a new adapter using the factory
        self.adapter_factory.create_resource_adapter(domain_id).await
    }
    
    /// Route a resource operation to the appropriate adapter
    pub async fn route_resource_operation(
        &self,
        operation: &ResourceTransferOperation,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference> {
        // Get the source and target domains
        let source_domain = &operation.source_domain;
        let target_domain = &operation.target_domain;
        
        // Perform the transfer using the cross-domain protocol
        self.cross_domain_protocol.transfer_resource(operation.clone(), context).await
    }
}

/// Basic implementation of a domain adapter factory
pub struct BasicDomainAdapterFactory {
    /// Available domain adapters
    domain_adapters: HashMap<DomainId, Box<dyn Fn() -> DomainIntegrationResult<(
        Arc<dyn DomainEffectHandler>, 
        Arc<dyn DomainResourceAdapter>
    )> + Send + Sync>>,
}

impl BasicDomainAdapterFactory {
    /// Create a new basic domain adapter factory
    pub fn new() -> Self {
        Self {
            domain_adapters: HashMap::new(),
        }
    }
    
    /// Register a domain adapter creator
    pub fn register_domain<F>(&mut self, domain_id: DomainId, creator: F) -> &mut Self 
    where
        F: Fn() -> DomainIntegrationResult<(
            Arc<dyn DomainEffectHandler>, 
            Arc<dyn DomainResourceAdapter>
        )> + Send + Sync + 'static,
    {
        self.domain_adapters.insert(domain_id, Box::new(creator));
        self
    }
}

impl Debug for BasicDomainAdapterFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicDomainAdapterFactory")
            .field("supported_domains", &self.supported_domains())
            .finish()
    }
}

#[async_trait]
impl DomainAdapterFactory for BasicDomainAdapterFactory {
    /// Create a domain effect handler for the specified domain
    async fn create_effect_handler(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainEffectHandler>> {
        if let Some(creator) = self.domain_adapters.get(domain_id) {
            let (handler, _) = creator()?;
            Ok(handler)
        } else {
            Err(DomainIntegrationError::DomainNotSupported(
                format!("Domain {} not supported", domain_id)
            ))
        }
    }
    
    /// Create a domain resource adapter for the specified domain
    async fn create_resource_adapter(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainResourceAdapter>> {
        if let Some(creator) = self.domain_adapters.get(domain_id) {
            let (_, adapter) = creator()?;
            Ok(adapter)
        } else {
            Err(DomainIntegrationError::DomainNotSupported(
                format!("Domain {} not supported", domain_id)
            ))
        }
    }
    
    /// Get supported domains
    fn supported_domains(&self) -> Vec<DomainId> {
        self.domain_adapters.keys().cloned().collect()
    }
}

/// Create a domain integration layer with default configuration
pub fn create_domain_integration_layer(
    cross_domain_protocol: Arc<dyn CrossDomainResourceProtocol>,
) -> (
    Arc<DomainEffectRouter>,
    Arc<DomainResourceRouter>,
    Arc<dyn DomainAdapterFactory>,
) {
    // Create the domain adapter factory
    let adapter_factory = Arc::new(BasicDomainAdapterFactory::new());
    
    // Create the routers
    let effect_router = Arc::new(DomainEffectRouter::new(adapter_factory.clone()));
    let resource_router = Arc::new(DomainResourceRouter::new(
        adapter_factory.clone(),
        cross_domain_protocol,
    ));
    
    (effect_router, resource_router, adapter_factory)
}

/// Domain capability mapping
#[derive(Debug, Clone)]
pub struct LocalDomainCapabilityMapping {
    /// The domain ID
    pub domain_id: DomainId,
    
    /// Required capabilities for different operations
    pub required_capabilities: HashMap<String, Vec<Capability<dyn Resource>>>,
} 