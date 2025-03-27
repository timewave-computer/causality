// Domain Effect Framework
//
// This module provides the foundation for domain-specific effects within the Causality
// system. Each domain represents a separate execution context with its own rules,
// resources, and capabilities.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;

use super::{Effect, EffectContext, EffectError, EffectOutcome, EffectResult};
use super::types::{EffectId, EffectTypeId, ExecutionBoundary};
use crate::resource_types::ResourceId;

/// Domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainId(String);

impl DomainId {
    /// Create a new domain ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the inner string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for DomainId {
    type Err = EffectError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl std::fmt::Display for DomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Domain effect errors
#[derive(Error, Debug)]
pub enum DomainEffectError {
    #[error("Domain not found: {0}")]
    DomainNotFound(String),
    
    #[error("Domain operation error: {0}")]
    OperationError(String),
    
    #[error("Domain validation error: {0}")]
    ValidationError(String),
    
    #[error("Domain parameter error: {0}")]
    ParameterError(String),
    
    #[error("Cross-domain error: {0}")]
    CrossDomainError(String),
    
    #[error("Domain boundary error: {0}")]
    BoundaryError(String),
}

/// Domain effect result
pub type DomainEffectResult<T> = Result<T, DomainEffectError>;

/// Domain effect outcome containing domain-specific result data
#[derive(Debug, Clone)]
pub struct DomainEffectOutcome {
    /// Domain ID
    pub domain_id: DomainId,
    
    /// Effect ID
    pub effect_id: EffectId,
    
    /// Whether the operation was successful
    pub success: bool,
    
    /// Domain-specific result data
    pub result: Option<HashMap<String, String>>,
    
    /// Error information if operation failed
    pub error: Option<String>,
    
    /// Cross-domain references if applicable
    pub cross_domain_refs: Option<HashMap<DomainId, String>>,
}

impl DomainEffectOutcome {
    /// Create a successful outcome
    pub fn success(
        domain_id: DomainId,
        effect_id: EffectId,
        result: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            domain_id,
            effect_id,
            success: true,
            result,
            error: None,
            cross_domain_refs: None,
        }
    }
    
    /// Create a failure outcome
    pub fn failure(
        domain_id: DomainId,
        effect_id: EffectId,
        error: String,
    ) -> Self {
        Self {
            domain_id,
            effect_id,
            success: false,
            result: None,
            error: Some(error),
            cross_domain_refs: None,
        }
    }
    
    /// Add cross-domain references
    pub fn with_cross_domain_refs(mut self, refs: HashMap<DomainId, String>) -> Self {
        self.cross_domain_refs = Some(refs);
        self
    }
    
    /// Convert to generic EffectOutcome
    pub fn to_effect_outcome(&self) -> EffectOutcome {
        if self.success {
            let mut result_data: HashMap<String, String> = HashMap::new();
            result_data.insert("domain_id".to_string(), self.domain_id.to_string());
            
            if let Some(result) = &self.result {
                result_data.extend(result.clone());
            }
            
            if let Some(refs) = &self.cross_domain_refs {
                for (domain, reference) in refs {
                    result_data.insert(format!("cross_domain_ref:{}", domain), reference.clone());
                }
            }
            
            EffectOutcome::Success(Box::new(result_data))
        } else {
            EffectOutcome::Error(Box::new(EffectError::ExecutionError(
                self.error.clone().unwrap_or_else(|| "Unknown domain error".to_string())
            )))
        }
    }
}

/// Domain parameter validation result
#[derive(Debug, Clone)]
pub struct ParameterValidationResult {
    /// Whether validation was successful
    pub valid: bool,
    
    /// Validation errors if any
    pub errors: Vec<String>,
    
    /// Normalized parameters
    pub normalized_parameters: HashMap<String, String>,
}

impl ParameterValidationResult {
    /// Create a successful validation result
    pub fn success(normalized_parameters: HashMap<String, String>) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            normalized_parameters,
        }
    }
    
    /// Create a failed validation result
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            normalized_parameters: HashMap::new(),
        }
    }
    
    /// Add a validation error
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }
    
    /// Combine with another validation result
    pub fn combine(mut self, other: ParameterValidationResult) -> Self {
        self.valid = self.valid && other.valid;
        self.errors.extend(other.errors);
        self.normalized_parameters.extend(other.normalized_parameters);
        self
    }
}

/// Parameter validator for domain effects
pub trait DomainParameterValidator: Send + Sync {
    /// Validate domain parameters
    fn validate(&self, parameters: &HashMap<String, String>) -> ParameterValidationResult;
    
    /// Get required parameters
    fn required_parameters(&self) -> Vec<String>;
    
    /// Get optional parameters
    fn optional_parameters(&self) -> Vec<String>;
    
    /// Check if a parameter is valid for this domain
    fn is_valid_parameter(&self, name: &str, value: &str) -> bool;
    
    /// Normalize a parameter value
    fn normalize_parameter(&self, name: &str, value: &str) -> String {
        value.to_string()
    }
}

/// Trait for domain-specific effects
#[async_trait]
pub trait DomainEffect: Effect + Debug + Send + Sync {
    /// Get the domain ID this effect operates on
    fn domain_id(&self) -> &DomainId;
    
    /// Get the execution boundary for this effect
    fn execution_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Any
    }
    
    /// Check if this effect can be executed in the target domain
    fn can_execute_in(&self, domain_id: &DomainId) -> bool {
        self.domain_id() == domain_id
    }
    
    /// Validate the domain parameters for this effect
    fn validate_parameters(&self) -> EffectResult<()> {
        Ok(())
    }
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    
    /// Adapt the effect context for the target domain
    fn adapt_context(&self, context: &dyn EffectContext) -> EffectResult<Box<dyn EffectContext>> {
        // Create a modified context with the domain ID
        let mut metadata = HashMap::new();
        metadata.insert("domain_id".to_string(), self.domain_id().to_string());
        metadata.insert("execution_boundary".to_string(), format!("{:?}", self.execution_boundary()));
        
        Ok(context.with_additional_metadata(metadata))
    }
    
    /// Handle this effect within the specified domain using the adapted context
    async fn handle_in_domain(
        &self,
        context: &dyn EffectContext,
        handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome>;
}

/// Trait for cross-domain effects
#[async_trait]
pub trait CrossDomainEffect: DomainEffect {
    /// Get the source domain ID
    fn source_domain_id(&self) -> &DomainId;
    
    /// Get the target domain ID
    fn target_domain_id(&self) -> &DomainId;
    
    /// Validate cross-domain parameters
    fn validate_cross_domain(&self) -> EffectResult<()> {
        if self.source_domain_id() == self.target_domain_id() {
            return Err(EffectError::ValidationError(
                "Source and target domains must be different for cross-domain effects".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Adapt the effect for the target domain
    fn adapt_for_target(&self) -> EffectResult<Box<dyn DomainEffect>>;
    
    /// Handle this effect across domains
    async fn handle_across_domains(
        &self,
        source_context: &dyn EffectContext,
        target_context: &dyn EffectContext,
        source_handler: &dyn DomainEffectHandler,
        target_handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome>;
}

/// Domain capability mapping
#[derive(Debug, Clone)]
pub struct DomainCapabilityMapping {
    /// Source domain
    pub source_domain: DomainId,
    
    /// Target domain
    pub target_domain: DomainId,
    
    /// Resource mappings from source to target domain
    pub resource_mappings: HashMap<ResourceId, ResourceId>,
    
    /// Capability mappings from source to target domain
    pub capability_mappings: HashMap<String, String>,
    
    /// Parameter transformations
    pub parameter_transforms: HashMap<String, Box<dyn Fn(&str) -> String + Send + Sync>>,
}

impl DomainCapabilityMapping {
    /// Create a new domain capability mapping
    pub fn new(source_domain: DomainId, target_domain: DomainId) -> Self {
        Self {
            source_domain,
            target_domain,
            resource_mappings: HashMap::new(),
            capability_mappings: HashMap::new(),
            parameter_transforms: HashMap::new(),
        }
    }
    
    /// Add a resource mapping
    pub fn add_resource_mapping(&mut self, source: ResourceId, target: ResourceId) {
        self.resource_mappings.insert(source, target);
    }
    
    /// Add a capability mapping
    pub fn add_capability_mapping(&mut self, source: String, target: String) {
        self.capability_mappings.insert(source, target);
    }
    
    /// Map a resource from source to target domain
    pub fn map_resource(&self, source: &ResourceId) -> Option<ResourceId> {
        self.resource_mappings.get(source).cloned()
    }
    
    /// Map a capability from source to target domain
    pub fn map_capability(&self, source: &str) -> Option<String> {
        self.capability_mappings.get(source).cloned()
    }
    
    /// Transform a parameter from source to target domain
    pub fn transform_parameter(&self, name: &str, value: &str) -> String {
        if let Some(transform) = self.parameter_transforms.get(name) {
            transform(value)
        } else {
            value.to_string()
        }
    }
}

/// Domain context adapter
#[derive(Debug)]
pub struct DomainContextAdapter {
    /// The domain ID
    domain_id: DomainId,
    
    /// The domain capability mappings
    mappings: HashMap<DomainId, DomainCapabilityMapping>,
}

impl DomainContextAdapter {
    /// Create a new domain context adapter
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            mappings: HashMap::new(),
        }
    }
    
    /// Add a capability mapping
    pub fn add_mapping(&mut self, mapping: DomainCapabilityMapping) {
        self.mappings.insert(mapping.source_domain.clone(), mapping);
    }
    
    /// Adapt a context from another domain to this domain
    pub fn adapt_context(
        &self,
        source_context: &dyn EffectContext,
        source_domain: &DomainId,
    ) -> EffectResult<Box<dyn EffectContext>> {
        let mapping = self.mappings.get(source_domain).ok_or_else(|| {
            EffectError::ValidationError(format!(
                "No capability mapping found from domain {} to {}",
                source_domain, self.domain_id
            ))
        })?;
        
        // Create a new context with the adapted capabilities and resources
        Ok(source_context.with_additional_metadata(HashMap::new()))
    }
}

/// Domain effect handler
#[async_trait]
pub trait DomainEffectHandler: Send + Sync + Debug {
    /// Get the domain ID this handler operates on
    fn domain_id(&self) -> &DomainId;
    
    /// Check if this handler can handle the given domain effect
    fn can_handle(&self, effect: &dyn DomainEffect) -> bool {
        effect.can_execute_in(self.domain_id())
    }
    
    /// Handle a domain effect
    async fn handle_domain_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Get the domain context adapter for this handler
    fn context_adapter(&self) -> Option<&DomainContextAdapter> {
        None
    }
}

/// Cross-domain effect handler
#[async_trait]
pub trait CrossDomainEffectHandler: DomainEffectHandler {
    /// Get the domains this handler can bridge between
    fn supported_domains(&self) -> Vec<(DomainId, DomainId)>;
    
    /// Check if this handler can handle cross-domain effects between the given domains
    fn can_handle_cross_domain(
        &self,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> bool {
        self.supported_domains().iter().any(|(src, tgt)| {
            src == source_domain && tgt == target_domain
        })
    }
    
    /// Handle a cross-domain effect
    async fn handle_cross_domain_effect(
        &self,
        effect: &dyn CrossDomainEffect,
        source_context: &dyn EffectContext,
        target_context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Get the capability mapping for the specified domains
    fn capability_mapping(
        &self,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Option<&DomainCapabilityMapping>;
}

/// Domain effect registry
#[derive(Debug)]
pub struct DomainEffectRegistry {
    /// Handlers by domain ID
    handlers: HashMap<DomainId, Vec<Arc<dyn DomainEffectHandler>>>,
    
    /// Cross-domain handlers
    cross_domain_handlers: Vec<Arc<dyn CrossDomainEffectHandler>>,
}

impl DomainEffectRegistry {
    /// Create a new domain effect registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            cross_domain_handlers: Vec::new(),
        }
    }
    
    /// Register a domain effect handler
    pub fn register_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) {
        let domain_id = handler.domain_id().clone();
        self.handlers.entry(domain_id)
            .or_insert_with(Vec::new)
            .push(handler);
    }
    
    /// Register a cross-domain effect handler
    pub fn register_cross_domain_handler(&mut self, handler: Arc<dyn CrossDomainEffectHandler>) {
        // Register as a regular handler for its primary domain
        self.register_handler(handler.clone() as Arc<dyn DomainEffectHandler>);
        
        // Also register as a cross-domain handler
        self.cross_domain_handlers.push(handler);
    }
    
    /// Get handlers for the given domain
    pub fn get_handlers(&self, domain_id: &DomainId) -> Vec<Arc<dyn DomainEffectHandler>> {
        self.handlers.get(domain_id)
            .map(|h| h.clone())
            .unwrap_or_default()
    }
    
    /// Get a handler for the given domain effect
    pub fn get_handler_for_effect(&self, effect: &dyn DomainEffect) -> Option<Arc<dyn DomainEffectHandler>> {
        let domain_id = effect.domain_id();
        let handlers = self.get_handlers(domain_id);
        
        for handler in handlers {
            if handler.can_handle(effect) {
                return Some(handler);
            }
        }
        
        None
    }
    
    /// Get a cross-domain handler for the given domains
    pub fn get_cross_domain_handler(
        &self,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Option<Arc<dyn CrossDomainEffectHandler>> {
        for handler in &self.cross_domain_handlers {
            if handler.can_handle_cross_domain(source_domain, target_domain) {
                return Some(handler.clone());
            }
        }
        
        None
    }
    
    /// Execute a domain effect
    pub async fn execute_domain_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let handler = self.get_handler_for_effect(effect)
            .ok_or_else(|| EffectError::ExecutionError(
                format!("No handler found for domain: {}", effect.domain_id())
            ))?;
        
        // Adapt the context for the domain
        let adapted_context = effect.adapt_context(context)?;
        
        // Handle the effect
        handler.handle_domain_effect(effect, adapted_context.as_ref()).await
    }
    
    /// Execute a cross-domain effect
    pub async fn execute_cross_domain_effect(
        &self,
        effect: &dyn CrossDomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let source_domain = effect.source_domain_id();
        let target_domain = effect.target_domain_id();
        
        // Find the appropriate cross-domain handler
        let handler = self.get_cross_domain_handler(source_domain, target_domain)
            .ok_or_else(|| EffectError::ExecutionError(
                format!("No cross-domain handler found for domains: {} -> {}", 
                    source_domain, target_domain)
            ))?;
        
        // Adapt the context for source domain
        let source_context = effect.adapt_context(context)?;
        
        // Adapt the adapted source context for target domain
        let target_context = if let Some(adapter) = handler.context_adapter() {
            adapter.adapt_context(source_context.as_ref(), source_domain)?
        } else {
            // Create a basic target context with the target domain ID
            let mut metadata = HashMap::new();
            metadata.insert("domain_id".to_string(), target_domain.to_string());
            source_context.with_additional_metadata(metadata)
        };
        
        // Handle the cross-domain effect
        handler.handle_cross_domain_effect(
            effect,
            source_context.as_ref(),
            target_context.as_ref(),
        ).await
    }
}

/// Basic domain effect implementation
#[derive(Debug, Clone)]
pub struct BasicDomainEffect {
    /// Effect ID
    id: EffectId,
    
    /// Effect type ID
    type_id: EffectTypeId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Execution boundary
    boundary: ExecutionBoundary,
    
    /// Effect parameters
    parameters: HashMap<String, String>,
}

impl BasicDomainEffect {
    /// Create a new basic domain effect
    pub fn new(
        id: EffectId,
        type_id: EffectTypeId,
        domain_id: DomainId,
    ) -> Self {
        Self {
            id,
            type_id,
            domain_id,
            boundary: ExecutionBoundary::Any,
            parameters: HashMap::new(),
        }
    }
    
    /// Set the execution boundary
    pub fn with_boundary(mut self, boundary: ExecutionBoundary) -> Self {
        self.boundary = boundary;
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple parameters
    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.parameters.extend(parameters);
        self
    }
    
    /// Get the parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

#[async_trait]
impl Effect for BasicDomainEffect {
    /// Get the ID of this effect
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    /// Get the type ID of this effect
    fn type_id(&self) -> EffectTypeId {
        self.type_id.clone()
    }
    
    /// Get the execution boundary for this effect
    fn boundary(&self) -> ExecutionBoundary {
        self.boundary
    }
    
    /// Clone this effect into a boxed effect
    fn clone_effect(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

#[async_trait]
impl DomainEffect for BasicDomainEffect {
    /// Get the domain ID this effect operates on
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the execution boundary for this effect
    fn execution_boundary(&self) -> ExecutionBoundary {
        self.boundary
    }
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String> {
        self.parameters.clone()
    }
    
    /// Handle this effect within the specified domain using the adapted context
    async fn handle_in_domain(
        &self,
        context: &dyn EffectContext,
        handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        handler.handle_domain_effect(self, context).await
    }
}

/// Basic cross-domain effect implementation
#[derive(Debug, Clone)]
pub struct BasicCrossDomainEffect {
    /// Base domain effect
    base: BasicDomainEffect,
    
    /// Source domain ID
    source_domain_id: DomainId,
    
    /// Target domain ID
    target_domain_id: DomainId,
}

impl BasicCrossDomainEffect {
    /// Create a new basic cross-domain effect
    pub fn new(
        id: EffectId,
        type_id: EffectTypeId,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
    ) -> Self {
        Self {
            base: BasicDomainEffect::new(
                id,
                type_id,
                source_domain_id.clone(),
            ).with_boundary(ExecutionBoundary::Boundary),
            source_domain_id,
            target_domain_id,
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.base = self.base.with_parameter(key, value);
        self
    }
    
    /// Add multiple parameters
    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.base = self.base.with_parameters(parameters);
        self
    }
    
    /// Get the parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        self.base.parameters()
    }
}

#[async_trait]
impl Effect for BasicCrossDomainEffect {
    /// Get the ID of this effect
    fn id(&self) -> &EffectId {
        self.base.id()
    }
    
    /// Get the type ID of this effect
    fn type_id(&self) -> EffectTypeId {
        self.base.type_id()
    }
    
    /// Get the execution boundary for this effect
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Boundary
    }
    
    /// Clone this effect into a boxed effect
    fn clone_effect(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

#[async_trait]
impl DomainEffect for BasicCrossDomainEffect {
    /// Get the domain ID this effect operates on
    fn domain_id(&self) -> &DomainId {
        &self.source_domain_id
    }
    
    /// Get the execution boundary for this effect
    fn execution_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Boundary
    }
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = self.base.domain_parameters();
        params.insert("target_domain".to_string(), self.target_domain_id.to_string());
        params
    }
    
    /// Handle this effect within the specified domain using the adapted context
    async fn handle_in_domain(
        &self,
        context: &dyn EffectContext,
        handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        // For cross-domain effects, we should use the cross-domain handler instead
        if let Some(cross_handler) = handler.as_any().downcast_ref::<dyn CrossDomainEffectHandler>() {
            let adapted_context = self.adapt_context(context)?;
            
            cross_handler.handle_cross_domain_effect(
                self,
                context,
                adapted_context.as_ref(),
            ).await
        } else {
            Err(EffectError::ExecutionError(
                "Cannot handle cross-domain effect with a regular domain handler".to_string()
            ))
        }
    }
}

#[async_trait]
impl CrossDomainEffect for BasicCrossDomainEffect {
    /// Get the source domain ID
    fn source_domain_id(&self) -> &DomainId {
        &self.source_domain_id
    }
    
    /// Get the target domain ID
    fn target_domain_id(&self) -> &DomainId {
        &self.target_domain_id
    }
    
    /// Adapt the effect for the target domain
    fn adapt_for_target(&self) -> EffectResult<Box<dyn DomainEffect>> {
        // Create a new basic domain effect for the target domain
        let effect = BasicDomainEffect::new(
            EffectId::new(),
            self.base.type_id(),
            self.target_domain_id.clone(),
        ).with_parameters(self.base.parameters().clone());
        
        Ok(Box::new(effect))
    }
    
    /// Handle this effect across domains
    async fn handle_across_domains(
        &self,
        source_context: &dyn EffectContext,
        target_context: &dyn EffectContext,
        source_handler: &dyn DomainEffectHandler,
        target_handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        // First handle in source domain
        let source_outcome = source_handler.handle_domain_effect(self, source_context).await?;
        
        // If the source handler succeeded, adapt the effect for the target domain
        if source_outcome.is_success() {
            let target_effect = self.adapt_for_target()?;
            
            // Handle in target domain
            target_handler.handle_domain_effect(target_effect.as_ref(), target_context).await
        } else {
            // Return the source outcome if it failed
            Ok(source_outcome)
        }
    }
}

/// Extension trait for casting handlers to other types
pub trait AsAny {
    /// Cast to Any
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Domain effect factory
pub struct DomainEffectFactory;

impl DomainEffectFactory {
    /// Create a basic domain effect
    pub fn create_domain_effect(
        domain_id: DomainId,
        effect_type: &str,
    ) -> BasicDomainEffect {
        let effect_id = EffectId::new();
        let type_id = EffectTypeId::new(effect_type);
        
        BasicDomainEffect::new(effect_id, type_id, domain_id)
    }
    
    /// Create a basic cross-domain effect
    pub fn create_cross_domain_effect(
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        effect_type: &str,
    ) -> BasicCrossDomainEffect {
        let effect_id = EffectId::new();
        let type_id = EffectTypeId::new(effect_type);
        
        BasicCrossDomainEffect::new(effect_id, type_id, source_domain_id, target_domain_id)
    }
}

/// Enhanced domain context adapter
#[derive(Debug)]
pub struct EnhancedDomainContextAdapter {
    /// The domain ID
    domain_id: DomainId,
    
    /// The domain capability mappings
    mappings: HashMap<DomainId, DomainCapabilityMapping>,
    
    /// Parameter validators for this domain
    parameter_validators: Vec<Arc<dyn DomainParameterValidator>>,
}

impl EnhancedDomainContextAdapter {
    /// Create a new enhanced domain context adapter
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            mappings: HashMap::new(),
            parameter_validators: Vec::new(),
        }
    }
    
    /// Add a capability mapping
    pub fn add_mapping(&mut self, mapping: DomainCapabilityMapping) {
        self.mappings.insert(mapping.source_domain.clone(), mapping);
    }
    
    /// Add a parameter validator
    pub fn add_parameter_validator(&mut self, validator: Arc<dyn DomainParameterValidator>) {
        self.parameter_validators.push(validator);
    }
    
    /// Validate parameters for this domain
    pub fn validate_parameters(&self, parameters: &HashMap<String, String>) -> ParameterValidationResult {
        let mut result = ParameterValidationResult::success(HashMap::new());
        
        for validator in &self.parameter_validators {
            let validator_result = validator.validate(parameters);
            result = result.combine(validator_result);
        }
        
        result
    }
    
    /// Adapt a context from another domain to this domain
    pub fn adapt_context(
        &self,
        source_context: &dyn EffectContext,
        source_domain: &DomainId,
    ) -> EffectResult<Box<dyn EffectContext>> {
        let mapping = self.mappings.get(source_domain).ok_or_else(|| {
            EffectError::ValidationError(format!(
                "No capability mapping found from domain {} to {}",
                source_domain, self.domain_id
            ))
        })?;
        
        // Map capabilities from source to target domain
        let mut adapted_capabilities = Vec::new();
        for cap in source_context.capabilities() {
            // In a real implementation, this would map capabilities based on the mapping
            adapted_capabilities.push(cap.clone());
        }
        
        // Map resources from source to target domain
        let mut adapted_resources = HashSet::new();
        for resource in source_context.resources() {
            if let Some(mapped_resource) = mapping.map_resource(resource) {
                adapted_resources.insert(mapped_resource);
            } else {
                adapted_resources.insert(resource.clone());
            }
        }
        
        // Set up metadata for the target domain
        let mut metadata = HashMap::new();
        metadata.insert("domain_id".to_string(), self.domain_id.to_string());
        
        // Copy relevant metadata from source context
        for (key, value) in source_context.metadata() {
            if !key.starts_with("domain_") { // Don't copy domain-specific metadata
                metadata.insert(key.clone(), value.clone());
            }
        }
        
        // Create a new context with adapted capabilities, resources, and metadata
        let adapted_context = source_context
            .with_additional_capabilities(adapted_capabilities)
            .with_additional_resources(adapted_resources)
            .with_additional_metadata(metadata);
        
        Ok(adapted_context)
    }
    
    /// Create a cross-domain context pair
    pub fn create_cross_domain_contexts(
        &self,
        source_context: &dyn EffectContext,
        source_domain: &DomainId,
        target_domain: &DomainId,
        target_adapter: &EnhancedDomainContextAdapter,
    ) -> EffectResult<(Box<dyn EffectContext>, Box<dyn EffectContext>)> {
        // Adapt source context
        let adapted_source = self.adapt_context(source_context, source_domain)?;
        
        // Adapt for target domain
        let adapted_target = target_adapter.adapt_context(&adapted_source, &self.domain_id)?;
        
        Ok((adapted_source, adapted_target))
    }
}

/// Enhanced domain effect handler
#[async_trait]
pub trait EnhancedDomainEffectHandler: DomainEffectHandler {
    /// Get the domain context adapter
    fn enhanced_context_adapter(&self) -> Option<&EnhancedDomainContextAdapter> {
        None
    }
    
    /// Validate domain parameters
    fn validate_domain_parameters(
        &self,
        parameters: &HashMap<String, String>,
    ) -> ParameterValidationResult {
        if let Some(adapter) = self.enhanced_context_adapter() {
            adapter.validate_parameters(parameters)
        } else {
            ParameterValidationResult::success(parameters.clone())
        }
    }
    
    /// Process domain effect outcome
    fn process_outcome(
        &self,
        outcome: DomainEffectOutcome,
    ) -> EffectResult<EffectOutcome> {
        Ok(outcome.to_effect_outcome())
    }
    
    /// Handle a domain effect with enhanced processing
    async fn handle_domain_effect_enhanced(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<DomainEffectOutcome>;
}

/// Extension methods for domain effects
pub trait DomainEffectExt: DomainEffect {
    /// Create a domain-specific parameter map
    fn create_parameter_map(&self) -> HashMap<String, String> {
        let mut parameters = HashMap::new();
        
        // Add domain ID
        parameters.insert("domain_id".to_string(), self.domain_id().to_string());
        
        // Add execution boundary
        parameters.insert("execution_boundary".to_string(), format!("{:?}", self.execution_boundary()));
        
        // Add domain-specific parameters
        parameters.extend(self.domain_parameters());
        
        parameters
    }
    
    /// Validate with a specific validator
    fn validate_with(
        &self,
        validator: &dyn DomainParameterValidator,
    ) -> ParameterValidationResult {
        validator.validate(&self.domain_parameters())
    }
    
    /// Enhanced validation for domain parameters
    fn enhanced_validation(&self) -> EffectResult<HashMap<String, String>> {
        let parameters = self.domain_parameters();
        
        // Check for required parameters
        let required = vec!["domain_id", "effect_type"];
        for req in required {
            if !parameters.contains_key(req) {
                return Err(EffectError::ValidationError(
                    format!("Missing required parameter: {}", req)
                ));
            }
        }
        
        // In a real implementation, additional validation logic would go here
        
        Ok(parameters)
    }
}

// Implement the extension trait for all types that implement DomainEffect
impl<T: DomainEffect + ?Sized> DomainEffectExt for T {} 