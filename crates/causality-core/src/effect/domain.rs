// Domain-specific effect module for causality-core
//
// This module implements the domain-specific effect system, which handles operations on domains,
// resources, and capabilities.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

// Import from parent module
use super::{
    Effect, EffectContext, EffectError, EffectOutcome, EffectResult, EffectType
};
use crate::effect::context::Capability;
use crate::resource::ResourceId;
// Define these types directly in this module to avoid import error
pub type EffectId = String;
pub type EffectTypeId = String;

/// Domain identifier
pub type DomainId = String;

/// Domain execution boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionBoundary {
    /// Can execute in any domain
    Any,
    
    /// Can only execute in the specified domain
    Domain,
    
    /// Can execute across domain boundaries
    CrossDomain,
    
    /// Can only execute at a specific boundary
    Boundary,
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

/// Effect outcome status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectOutcomeStatus {
    /// Effect was successful
    Success,
    /// Effect failed
    Failure,
    /// Effect is pending
    Pending,
}

/// Outcome of a domain effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEffectOutcome {
    /// The target domain ID
    pub domain_id: DomainId,
    
    /// The effect ID
    pub effect_id: String,
    
    /// The outcome status
    pub status: EffectOutcomeStatus,
    
    /// The effect result data
    pub data: Option<HashMap<String, String>>,
    
    /// Error information if the effect failed
    pub error: Option<String>,
}

impl DomainEffectOutcome {
    /// Create a successful outcome
    pub fn success(domain_id: DomainId, effect_id: String, data: Option<HashMap<String, String>>) -> Self {
        Self {
            domain_id,
            effect_id,
            status: EffectOutcomeStatus::Success,
            data,
            error: None,
        }
    }
    
    /// Create a failed outcome
    pub fn failure(domain_id: DomainId, effect_id: String, error: String) -> Self {
        Self {
            domain_id,
            effect_id,
            status: EffectOutcomeStatus::Failure,
            data: None,
            error: Some(error),
        }
    }
    
    /// Convert to a regular EffectOutcome
    pub fn to_effect_outcome(&self) -> EffectOutcome {
        let mut data = HashMap::new();
        if let Some(ref result_data) = self.data {
            data = result_data.clone();
        }
        
        // Add domain_id to the data
        data.insert("domain_id".to_string(), self.domain_id.clone());
        
        // Add effect_id to the data
        data.insert("effect_id".to_string(), self.effect_id.clone());
        
        let mut outcome = match self.status {
            EffectOutcomeStatus::Success => EffectOutcome::success(data),
            EffectOutcomeStatus::Failure => {
                let mut outcome = EffectOutcome::success(data);
                outcome.status = super::outcome::EffectStatus::Failure;
                if let Some(ref error) = self.error {
                    outcome.error_message = Some(error.clone());
                } else {
                    outcome.error_message = Some("Unknown domain effect error".to_string());
                }
                outcome
            },
            EffectOutcomeStatus::Pending => {
                let mut outcome = EffectOutcome::success(data);
                outcome.status = super::outcome::EffectStatus::Pending;
                outcome
            }
        };
        
        outcome
    }
    
    /// Check if the outcome was successful
    pub fn is_success(&self) -> bool {
        self.status == EffectOutcomeStatus::Success
    }
    
    /// Check if the outcome failed
    pub fn is_failure(&self) -> bool {
        self.status == EffectOutcomeStatus::Failure
    }
    
    /// Check if the outcome is pending
    pub fn is_pending(&self) -> bool {
        self.status == EffectOutcomeStatus::Pending
    }

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any {
        self
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
pub trait DomainParameterValidator: Send + Sync + Debug {
    /// Validate domain parameters
    fn validate(&self, parameters: &HashMap<String, String>) -> ParameterValidationResult;
    
    /// Get required parameters
    fn required_parameters(&self) -> Vec<String>;
    
    /// Get optional parameters
    fn optional_parameters(&self) -> Vec<String>;
    
    /// Check if a parameter is valid for this domain
    fn is_valid_parameter(&self, name: &str, value: &str) -> bool;
    
    /// Normalize a parameter value
    fn normalize_parameter(&self, _name: &str, value: &str) -> String {
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
    fn validate_parameters(&self) -> EffectResult<EffectOutcome>;
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String>;
    
    /// Adapt the effect context for the target domain
    fn adapt_context(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Handle this effect within the specified domain using the adapted context
    async fn handle_in_domain(
        &self,
        context: &dyn EffectContext,
        handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome>;
}

/// Trait for domain effects that support downcasting
pub trait DowncastDomainEffect: DomainEffect {
    /// Helper method to downcast to a specific domain effect type
    fn downcast_domain_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

// Blanket implementation for all types that implement DomainEffect
impl<T: DomainEffect> DowncastDomainEffect for T {}

/// Trait for cross-domain effects
#[async_trait]
pub trait CrossDomainEffect: DomainEffect {
    /// Get the source domain ID
    fn source_domain_id(&self) -> &DomainId;
    
    /// Get the target domain ID
    fn target_domain_id(&self) -> &DomainId;
    
    /// Validate cross-domain parameters
    fn validate_cross_domain(&self) -> EffectResult<EffectOutcome>;
    
    /// Adapt the effect for the target domain
    fn adapt_for_target(&self) -> EffectResult<DomainEffectOutcome>;
    
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
    
    /// Parameter transform flags - simpler than using function pointers
    pub parameter_transform_options: HashMap<String, String>,
}

impl DomainCapabilityMapping {
    /// Create a new domain capability mapping
    pub fn new(source_domain: DomainId, target_domain: DomainId) -> Self {
        Self {
            source_domain,
            target_domain,
            resource_mappings: HashMap::new(),
            capability_mappings: HashMap::new(),
            parameter_transform_options: HashMap::new(),
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
    
    /// Add a parameter transform option
    pub fn add_parameter_transform(&mut self, param: String, transform_type: String) {
        self.parameter_transform_options.insert(param, transform_type);
    }
    
    /// Map a resource from source to target domain
    pub fn map_resource(&self, source: &ResourceId) -> Option<ResourceId> {
        self.resource_mappings.get(source).cloned()
    }
    
    /// Map a capability from source to target domain
    pub fn map_capability(&self, source: &str) -> Option<String> {
        self.capability_mappings.get(source).cloned()
    }
    
    /// Transform a parameter based on registered transform options
    pub fn transform_parameter(&self, name: &str, value: &str) -> String {
        if let Some(transform_type) = self.parameter_transform_options.get(name) {
            match transform_type.as_str() {
                "uppercase" => value.to_uppercase(),
                "lowercase" => value.to_lowercase(),
                "prefixed" => format!("{}_{}", self.target_domain, value),
                _ => value.to_string(),
            }
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
    ) -> EffectResult<EffectOutcome> {
        // Error check for the mapping
        if !self.mappings.contains_key(source_domain) {
            return Err(EffectError::InvalidOperation(format!(
                "No domain mapping found for source domain: {}", source_domain
            )));
        }
        
        // Return success with context data
        Ok(EffectOutcome::success_with_data(source_context.metadata().clone()))
    }
}

/// Cross-domain support type
#[derive(Debug, Clone, PartialEq)]
pub enum CrossDomainSupport {
    /// No cross-domain support
    None,
    /// Simple cross-domain support
    Simple,
    /// Full cross-domain support
    Full,
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
    
    /// Check if this handler supports cross-domain effects
    fn cross_domain_support(&self) -> Option<CrossDomainSupport> {
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
#[derive(Debug, Clone)]
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
        // Register as a cross-domain handler first
        self.cross_domain_handlers.push(handler.clone());
        
        // Now create a wrapper that delegates to the cross-domain handler
        // This avoids the need for trait casting
        let domain_id = handler.domain_id().clone();
        let cross_domain_handler = handler;
        
        // Create a wrapper DomainEffectHandler that delegates to the cross-domain handler
        // This avoids the need for a trait upcast
        struct CrossDomainWrapper {
            inner: Arc<dyn CrossDomainEffectHandler>,
        }
        
        impl Debug for CrossDomainWrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("CrossDomainWrapper")
                    .field("inner", &"CrossDomainEffectHandler")
                    .finish()
            }
        }
        
        #[async_trait]
        impl DomainEffectHandler for CrossDomainWrapper {
            fn domain_id(&self) -> &DomainId {
                self.inner.domain_id()
            }
            
            fn can_handle(&self, effect: &dyn DomainEffect) -> bool {
                self.inner.can_handle(effect)
            }
            
            async fn handle_domain_effect(
                &self,
                effect: &dyn DomainEffect,
                context: &dyn EffectContext,
            ) -> EffectResult<EffectOutcome> {
                self.inner.handle_domain_effect(effect, context).await
            }
            
            fn context_adapter(&self) -> Option<&DomainContextAdapter> {
                self.inner.context_adapter()
            }
            
            fn cross_domain_support(&self) -> Option<CrossDomainSupport> {
                self.inner.cross_domain_support()
            }
        }
        
        let wrapper = CrossDomainWrapper { inner: cross_domain_handler };
        self.handlers.entry(domain_id)
            .or_insert_with(Vec::new)
            .push(Arc::new(wrapper));
    }
    
    /// Get handlers for the given domain
    pub fn get_handlers(&self, domain_id: &DomainId) -> Vec<Arc<dyn DomainEffectHandler>> {
        self.handlers.get(domain_id)
            .map(|h| h.clone())
            .unwrap_or_default()
    }
    
    /// Check if a handler exists for the given domain and effect type
    pub fn has_handler_for_type(&self, domain_id: &DomainId, _effect_type_id: &EffectTypeId) -> bool {
        if let Some(handlers) = self.handlers.get(domain_id) {
            for _handler in handlers {
                // For a proper implementation we would need to check if the handler can handle this type
                // For now, just return true if there is any handler for this domain
                return true;
            }
        }
        false
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
        _source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Option<Arc<dyn CrossDomainEffectHandler>> {
        let _default_ctx = SimpleEffectContext::new(crate::effect::types::EffectId::from(target_domain.to_string()));
        
        for handler in &self.cross_domain_handlers {
            if handler.domain_id() == "*" && matches!(handler.cross_domain_support(), Some(CrossDomainSupport::Full)) {
                return Some(handler.clone());
            }
        }
        
        None
    }
    
    /// Execute a domain effect
    pub fn execute_effect(&self, effect: &dyn DomainEffect, _context: &dyn EffectContext) 
        -> EffectResult<EffectOutcome> {
        // Find a handler for this effect
        let _handler = self.get_handler_for_effect(effect)
            .ok_or_else(|| EffectError::ExecutionError(format!(
                "No handler found for domain effect {}",
                effect.description()
            )))?;
            
        // For now, just return a simple success outcome
        // In a real implementation, this would delegate to handle_domain_effect
        let data = HashMap::new();
        Ok(EffectOutcome::success(data))
    }
    
    /// Execute a cross-domain effect
    pub async fn execute_cross_domain_effect(
        &self,
        effect: &dyn CrossDomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Find a cross-domain handler
        let source_domain = effect.source_domain_id();
        let target_domain = effect.target_domain_id();
        
        let handler = self.get_cross_domain_handler(source_domain, target_domain)
            .ok_or_else(|| EffectError::ExecutionError(
                format!("No cross-domain handler found for {} -> {}", source_domain, target_domain)
            ))?;
        
        // Validate the cross-domain effect
        effect.validate_cross_domain()?;
        
        // Create appropriate contexts for source and target domains
        let mut source_ctx = SimpleEffectContext::new(crate::effect::types::EffectId::from(source_domain.to_string()))
            .with_metadata("domain_id", source_domain);
            
        let mut target_ctx = SimpleEffectContext::new(crate::effect::types::EffectId::from(target_domain.to_string()))
            .with_metadata("domain_id", target_domain);
        
        // Execute the cross-domain effect
        let result = handler.handle_cross_domain_effect(
            effect,
            &source_ctx,
            &target_ctx
        ).await?;
        
        Ok(result)
    }
}

/// Basic domain effect implementation
#[derive(Debug, Clone)]
pub struct BasicDomainEffect {
    /// Domain ID
    domain_id: DomainId,
    
    /// Effect type
    type_id: EffectType,
    
    /// Execution boundary
    boundary: ExecutionBoundary,
    
    /// Effect parameters
    parameters: HashMap<String, String>,
}

impl BasicDomainEffect {
    /// Create a new basic domain effect
    pub fn new(
        type_id: EffectType,
        domain_id: DomainId,
    ) -> Self {
        Self {
            domain_id,
            type_id,
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
        self.parameters = parameters;
        self
    }
    
    /// Get the parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

#[async_trait]
impl Effect for BasicDomainEffect {
    fn effect_type(&self) -> EffectType {
        self.type_id.clone()
    }
    
    fn description(&self) -> String {
        format!("{:?} effect in domain {}", self.type_id, self.domain_id)
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Use success() method to construct a successful outcome with empty data HashMap
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
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
        self.boundary.clone()
    }
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String> {
        self.parameters.clone()
    }
    
    /// Validate parameters for this effect
    fn validate_parameters(&self) -> EffectResult<EffectOutcome> {
        // Basic parameter validation that always succeeds
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    /// Adapt the context for the domain
    fn adapt_context(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Return the context as is, wrapped in a success outcome
        Ok(EffectOutcome::success_with_data(context.metadata().clone()))
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
    /// Source domain ID
    source_domain_id: DomainId,
    
    /// Target domain ID
    target_domain_id: DomainId,
    
    /// Effect type
    type_id: EffectType,
    
    /// Effect parameters
    parameters: HashMap<String, String>,
}

impl BasicCrossDomainEffect {
    /// Create a new basic cross-domain effect
    pub fn new(
        type_id: EffectType,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
    ) -> Self {
        Self {
            source_domain_id,
            target_domain_id,
            type_id,
            parameters: HashMap::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple parameters
    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.parameters = parameters;
        self
    }
    
    /// Get the parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

#[async_trait]
impl Effect for BasicCrossDomainEffect {
    fn effect_type(&self) -> EffectType {
        self.type_id.clone()
    }
    
    fn description(&self) -> String {
        format!("{:?} cross-domain effect from {} to {}", 
                self.type_id, self.source_domain_id, self.target_domain_id)
    }
    
    async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Cross-domain effects need special handling
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
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
        ExecutionBoundary::CrossDomain
    }
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = self.parameters.clone();
        params.insert("source_domain".to_string(), self.source_domain_id.to_string());
        params.insert("target_domain".to_string(), self.target_domain_id.to_string());
        params
    }
    
    /// Validate parameters for this effect
    fn validate_parameters(&self) -> EffectResult<EffectOutcome> {
        // Validate parameters for cross-domain effect
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    /// Adapt the context for the domain
    fn adapt_context(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Return the context with source domain info
        Ok(EffectOutcome::success_with_data(context.metadata().clone()))
    }
    
    /// Handle this effect within the specified domain using the adapted context
    async fn handle_in_domain(
        &self,
        context: &dyn EffectContext,
        handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        // For cross-domain effects, we should use the cross-domain handler if available
        // Instead of trying to downcast, check if handler implements a specific method
        if let Some(_cross_domain_method) = handler.cross_domain_support() {
            // If cross-domain is supported, use that path
            // Create an adapted context
            let _result = self.adapt_context(context)?;
            
            // Handle in the cross-domain context
            handler.handle_domain_effect(self, context).await
        } else {
            // If we don't have cross-domain support, return an error
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
    
    /// Validate cross-domain parameters
    fn validate_cross_domain(&self) -> EffectResult<EffectOutcome> {
        // Basic validation that always succeeds for now
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    /// Adapt the effect for the target domain
    fn adapt_for_target(&self) -> EffectResult<DomainEffectOutcome> {
        // Basic adaptation logic
        let data = HashMap::new();
        Ok(DomainEffectOutcome::success(
            self.target_domain_id.clone(),
            "adapted".to_string(),
            Some(data)
        ))
    }
    
    /// Handle this effect across domains
    async fn handle_across_domains(
        &self,
        source_context: &dyn EffectContext,
        target_context: &dyn EffectContext,
        source_handler: &dyn DomainEffectHandler,
        target_handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        // Handle in source domain first
        let source_outcome = source_handler.handle_domain_effect(self, source_context).await?;
        
        // If the source handler succeeded, adapt the effect for the target domain
        if source_outcome.status == crate::effect::outcome::EffectStatus::Success {
            // Create a new effect for the target domain
            let target_effect = BasicDomainEffect::new(
                self.effect_type(),
                self.target_domain_id.clone()
            );
            
            // Handle in target domain
            target_handler.handle_domain_effect(&target_effect, target_context).await
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
        let type_id = EffectType::Custom(effect_type.to_string());
        
        BasicDomainEffect::new(type_id, domain_id)
    }
    
    /// Create a basic cross-domain effect
    pub fn create_cross_domain_effect(
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        effect_type: &str,
    ) -> BasicCrossDomainEffect {
        let type_id = EffectType::Custom(effect_type.to_string());
        
        BasicCrossDomainEffect::new(type_id, source_domain_id, target_domain_id)
    }
}

/// Enhanced domain context adapter
#[derive(Debug)]
pub struct EnhancedDomainContextAdapter {
    /// The domain ID
    domain_id: DomainId,
    
    /// The domain capability mappings
    mappings: HashMap<DomainId, DomainCapabilityMapping>,
    
    /// Parameter validators for this domain (now implements Debug)
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
    ) -> EffectResult<EffectOutcome> {
        // Error check for the mapping
        if !self.mappings.contains_key(source_domain) {
            return Err(EffectError::InvalidOperation(format!(
                "No domain mapping found for source domain: {}", source_domain
            )));
        }
        
        // Return success with context data
        Ok(EffectOutcome::success_with_data(source_context.metadata().clone()))
    }
    
    /// Create a cross-domain context pair
    pub fn create_cross_domain_contexts(
        &self,
        source_context: &dyn EffectContext,
        source_domain: &DomainId,
        target_domain: &DomainId,
        _target_adapter: &EnhancedDomainContextAdapter,
    ) -> EffectResult<EffectOutcome> {
        // Adapt source context
        let source_outcome = self.adapt_context(source_context, source_domain)?;
        
        // Create a target context with the target domain ID
        let mut target_metadata = HashMap::new();
        target_metadata.insert("domain_id".to_string(), target_domain.to_string());
        
        // Copy any metadata from the source context to target
        if source_outcome.status == crate::effect::outcome::EffectStatus::Success {
            for (key, value) in &source_outcome.data {
                if key != "domain_id" {
                    target_metadata.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Return both contexts in the metadata
        let mut combined_data = HashMap::new();
        combined_data.insert("source_domain".to_string(), source_domain.to_string());
        combined_data.insert("target_domain".to_string(), target_domain.to_string());
        
        Ok(EffectOutcome::success_with_data(combined_data))
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
    fn enhanced_validation(&self) -> EffectResult<EffectOutcome> {
        let parameters = self.domain_parameters();
        
        // Check for required parameters
        let required = vec!["domain_id", "effect_type"];
        for req in required {
            if !parameters.contains_key(req) {
                return Err(EffectError::InvalidOperation(
                    format!("Missing required parameter: {}", req)
                ));
            }
        }
        
        // In a real implementation, additional validation logic would go here
        
        Ok(EffectOutcome::success_with_data(parameters))
    }
}

// Implement the extension trait for all types that implement DomainEffect
impl<T: DomainEffect + ?Sized> DomainEffectExt for T {}

/// Simple implementation of EffectContext for domain effects
#[derive(Debug, Clone)]
pub struct SimpleEffectContext {
    effect_id: crate::effect::types::EffectId,
    capabilities: Vec<Capability>,
    resources: HashSet<ResourceId>,
    metadata: HashMap<String, String>,
    parent: Option<Arc<dyn EffectContext>>,
}

impl SimpleEffectContext {
    pub fn new(effect_id: crate::effect::types::EffectId) -> Self {
        Self {
            effect_id,
            capabilities: Vec::new(),
            resources: HashSet::new(),
            metadata: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(effect_id: crate::effect::types::EffectId, parent: Arc<dyn EffectContext>) -> Self {
        Self {
            effect_id,
            capabilities: Vec::new(),
            resources: HashSet::new(),
            metadata: HashMap::new(),
            parent: Some(parent),
        }
    }
    
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        let capability_str = capability.into();
        self.capabilities.push(Capability::new(causality_types::ContentId::parse(&capability_str).unwrap_or_else(|_| causality_types::ContentId::new(capability_str)), crate::effect::types::Right::Read));
        self
    }
    
    pub fn with_resource(mut self, resource_id: ResourceId) -> Self {
        self.resources.insert(resource_id);
        self
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl EffectContext for SimpleEffectContext {
    fn effect_id(&self) -> &crate::effect::types::EffectId {
        &self.effect_id
    }
    
    fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    fn resources(&self) -> &HashSet<ResourceId> {
        &self.resources
    }
    
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
        self.parent.as_ref()
    }
    
    fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability) || 
            self.parent
                .as_ref()
                .map(|p| p.has_capability(capability))
                .unwrap_or(false)
    }
    
    fn derive_context(&self, effect_id: crate::effect::types::EffectId) -> Box<dyn EffectContext> {
        Box::new(SimpleEffectContext {
            effect_id,
            capabilities: self.capabilities.clone(),
            resources: self.resources.clone(),
            metadata: self.metadata.clone(),
            parent: self.parent.clone(),
        })
    }
    
    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
        let mut new_capabilities = self.capabilities.clone();
        new_capabilities.extend(capabilities);
        
        Box::new(SimpleEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: new_capabilities,
            resources: self.resources.clone(),
            metadata: self.metadata.clone(),
            parent: self.parent.clone(),
        })
    }
    
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        let mut new_resources = self.resources.clone();
        new_resources.extend(resources);
        
        Box::new(SimpleEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            resources: new_resources,
            metadata: self.metadata.clone(),
            parent: self.parent.clone(),
        })
    }
    
    fn with_additional_metadata(&self, additional_metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut new_metadata = self.metadata.clone();
        new_metadata.extend(additional_metadata);
        
        Box::new(SimpleEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            resources: self.resources.clone(),
            metadata: new_metadata,
            parent: self.parent.clone(),
        })
    }
    
    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(SimpleEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            resources: self.resources.clone(),
            metadata: self.metadata.clone(),
            parent: self.parent.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
} 