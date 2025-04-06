// Domain-specific effect module for causality-core
//
// This module implements the domain-specific effect system, which handles operations on domains,
// resources, and capabilities.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

// Import from parent module
use super::{
    Effect, EffectContext, EffectError, EffectOutcome, EffectResult, EffectType
};
// Import EffectId and EffectTypeId from types module
use super::types::EffectTypeId;
use crate::effect::context::Capability;
use crate::resource::ResourceId;

/// Domain identifier
pub type DomainId = String;

/// Execution boundary for domain effects
#[derive(Debug, Clone)]
pub enum ExecutionBoundary {
    /// Can execute in any domain
    Any,
    /// Can only execute in a specific domain
    Domain(String),
    /// Can execute across domains
    CrossDomain(String),
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
    
    #[error("Invalid handler combination: {0}")]
    InvalidHandlerCombination(String),
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
    
    /// Convert a domain effect outcome to a regular effect outcome
    pub fn to_effect_outcome(&self) -> EffectOutcome {
        let mut data = HashMap::new();
        if let Some(ref result_data) = self.data {
            data = result_data.clone();
        }
        
        match self.status {
            EffectOutcomeStatus::Success => EffectOutcome::success(data),
            EffectOutcomeStatus::Failure => {
                let error = self.error.clone().unwrap_or_else(|| "Unknown domain effect error".to_string());
                EffectOutcome::failure(error)
            },
            EffectOutcomeStatus::Pending => {
                let mut outcome = EffectOutcome::success(data);
                outcome.status = super::outcome::EffectStatus::Pending;
                outcome
            }
        }
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

/// A trait for domain-specific effects
pub trait DomainEffect: Effect {
    /// Get the ID of the domain this effect belongs to.
    fn domain_id(&self) -> &DomainId;

    /// Get the execution boundary for this effect
    fn execution_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Any
    }
    
    /// Check if this effect can be executed in the target domain
    fn can_execute_in(&self, domain_id: &DomainId) -> bool {
        self.domain_id() == domain_id
    }

    /// Validate this effect
    fn validate(&self) -> Result<(), String>;
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String>;

    /// Adapt this effect's context for the domain
    fn adapt_context(&self, context: &dyn EffectContext) -> Result<(), String>;
}

/// A trait for handling domain effects
#[async_trait::async_trait]
pub trait DomainEffectHandling: DomainEffect {
    /// Handle this effect in the specified domain
    async fn handle_in_domain<H: DomainEffectHandler + Send + Sync>(
        self: &Self,
        context: &dyn EffectContext,
        handler: &H,
    ) -> Result<DomainEffectOutcome, DomainEffectError> {
        // Instead of trying to cast self to a trait object, explicitly pass the required
        // domain effect information to the handler
        let effect_type = self.effect_type();
        let domain_id = self.domain_id();
        let parameters = self.domain_parameters();
        
        // Call a custom method on the handler that takes the individual components
        // instead of a trait object
        handler.handle_domain_effect_with_data(
            effect_type, 
            domain_id, 
            &parameters, 
            context
        ).await
    }
}

// Blanket implementation for all types that implement DomainEffect
#[async_trait::async_trait]
impl<T: DomainEffect + Send + Sync> DomainEffectHandling for T {}

/// A trait for cross-domain effects
pub trait CrossDomainEffect: DomainEffect {
    /// Get the source domain ID for this effect
    fn source_domain_id(&self) -> &DomainId;
    
    /// Get the target domain ID for this effect
    fn target_domain_id(&self) -> &DomainId;
    
    /// Adapt context for the source domain
    fn adapt_source_context(&self, context: &dyn EffectContext) -> Result<Box<dyn EffectContext>, String>;
    
    /// Adapt context for the target domain
    fn adapt_target_context(&self, context: &dyn EffectContext) -> Result<Box<dyn EffectContext>, String>;
    
    /// View this effect as a source domain effect
    fn as_source_domain_effect(&self) -> &dyn DomainEffect where Self: Sized {
        self
    }
    
    /// View this effect as a target domain effect
    fn as_target_domain_effect(&self) -> &dyn DomainEffect where Self: Sized {
        self
    }
    
    /// Validate cross-domain parameters
    fn validate_cross_domain(&self) -> Result<(), String>;
    
    /// Create a basic domain effect for the source domain
    fn to_source_domain_effect(&self) -> BasicDomainEffect {
        let parameters = self.domain_parameters();
        BasicDomainEffect::new(
            self.effect_type(),
            self.source_domain_id().clone(),
        ).with_parameters(parameters)
    }
    
    /// Create a basic domain effect for the target domain
    fn to_target_domain_effect(&self) -> BasicDomainEffect {
        let parameters = self.domain_parameters();
        BasicDomainEffect::new(
            self.effect_type(),
            self.target_domain_id().clone(),
        ).with_parameters(parameters)
    }
}

/// Async handler for cross-domain effects
#[async_trait::async_trait]
pub trait CrossDomainEffectHandling: CrossDomainEffect {
    /// Handle an effect across domains
    async fn handle_across_domains(
        &self,
        source_context: &dyn EffectContext,
        target_context: &dyn EffectContext,
        source_handler: &dyn DomainEffectHandler, 
        target_handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome>;
}

/// Default implementation of CrossDomainEffectHandling
#[async_trait::async_trait]
impl<T: CrossDomainEffect + Send + Sync> CrossDomainEffectHandling for T {
    async fn handle_across_domains(
        &self,
        source_context: &dyn EffectContext,
        target_context: &dyn EffectContext,
        source_handler: &dyn DomainEffectHandler, 
        target_handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        // Handle in the respective domains using the new helper methods
        let source_effect = self.to_source_domain_effect();
        let target_effect = self.to_target_domain_effect();
        
        let domain_outcome = source_handler.handle_domain_effect(
            &source_effect,
            source_context,
        ).await
        .map_err(EffectError::from)?;
        
        let _ = target_handler.handle_domain_effect(
            &target_effect,
            target_context,
        ).await
        .map_err(EffectError::from)?;
        
        // Convert to a regular effect outcome
        Ok(domain_outcome.to_effect_outcome())
    }
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

/// Domain effect handler trait
#[async_trait::async_trait]
pub trait DomainEffectHandler: Send + Sync + Debug {
    /// Get the domain ID for this handler
    fn domain_id(&self) -> &str;
    
    /// Check if this handler can handle the given effect
    fn can_handle(&self, effect: &dyn DomainEffect) -> bool {
        effect.domain_id() == self.domain_id()
    }
    
    /// Handle a domain effect
    async fn handle_domain_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> Result<DomainEffectOutcome, DomainEffectError>;
    
    /// Handle a domain effect using explicit data instead of a trait object
    /// This is used by the DomainEffectHandling trait to avoid Self size issues
    async fn handle_domain_effect_with_data(
        &self,
        effect_type: EffectType,
        domain_id: &str,
        parameters: &HashMap<String, String>,
        context: &dyn EffectContext,
    ) -> Result<DomainEffectOutcome, DomainEffectError> {
        // Create a BasicDomainEffect to wrap the data
        let basic_effect = BasicDomainEffect::new(
            effect_type,
            domain_id.to_string(),
        ).with_parameters(parameters.clone());
        
        // Use the regular handler method
        self.handle_domain_effect(&basic_effect, context).await
    }
}

/// Enum to hold either a regular or cross-domain handler
#[derive(Debug, Clone)]
pub enum HandlerVariant {
    /// Regular domain handler
    Domain(Arc<dyn DomainEffectHandler>),
}

impl HandlerVariant {
    /// Get the handler's domain ID
    pub fn domain_id(&self) -> &str {
        match self {
            HandlerVariant::Domain(handler) => handler.domain_id(),
        }
    }
    
    /// Check if this handler can handle the given effect type
    pub fn can_handle(&self, effect: &dyn DomainEffect) -> bool {
        match self {
            HandlerVariant::Domain(handler) => handler.can_handle(effect),
        }
    }
}

/// Registry for domain effects and handlers
#[derive(Debug, Default)]
pub struct DomainEffectRegistry {
    /// Domain-specific handlers organized by domain ID
    handlers: HashMap<String, Vec<HandlerVariant>>,
}

impl Clone for DomainEffectRegistry {
    fn clone(&self) -> Self {
        Self {
            handlers: self.handlers.clone()
        }
    }
}

impl DomainEffectRegistry {
    /// Create a new registry for domain effects
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register a handler for a specific domain
    pub fn register_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) {
        let domain_id = handler.domain_id().to_string();
        self.handlers
            .entry(domain_id)
            .or_insert_with(Vec::new)
            .push(HandlerVariant::Domain(handler));
    }
    
    /// Get all handlers for a specific domain
    pub fn get_handlers(&self, domain_id: &str) -> Vec<HandlerVariant> {
        self.handlers
            .get(domain_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get a handler for a specific domain and effect
    pub fn get_handler_for_effect(&self, effect: &dyn DomainEffect) -> Option<HandlerVariant> {
        let domain_id = effect.domain_id().to_string();
        if let Some(handlers) = self.handlers.get(&domain_id) {
            for handler_variant in handlers {
                match handler_variant {
                    HandlerVariant::Domain(handler) => {
                        if handler.can_handle(effect) {
                            return Some(HandlerVariant::Domain(handler.clone()));
                        }
                    }
                }
            }
        }
        None
    }

    /// Get a handler for a specific domain
    pub fn get_handler_for_domain(&self, domain_id: &str) -> Option<HandlerVariant> {
        if let Some(handlers) = self.handlers.get(domain_id) {
            if !handlers.is_empty() {
                return Some(handlers[0].clone());
            }
        }
        None
    }
    
    /// Execute a domain effect (now async)
    pub async fn execute_effect( // Add async keyword
        &self, 
        effect: &dyn DomainEffect, 
        context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Find an appropriate handler for this effect
        let handler_variant = self.get_handler_for_effect(effect)
            .ok_or_else(|| {
                EffectError::HandlerNotFound(format!(
                    "No handler found for effect with domain ID: {}",
                    effect.domain_id()
                ))
            })?;

        // Execute the effect using the handler
        let domain_outcome = match handler_variant {
            HandlerVariant::Domain(handler) => {
                handler.handle_domain_effect(effect, context).await.map_err(EffectError::from)?
            }
        };

        // Convert the domain outcome to a generic effect outcome
        Ok(domain_outcome.to_effect_outcome())
    }
    
    /// Execute a cross-domain effect
    pub async fn execute_cross_domain_effect<T: CrossDomainEffect + Send + Sync>(
        &self,
        effect: &T,
        context: &dyn EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Get handlers for both domains
        let source_handler = self.get_handler_for_domain(&effect.source_domain_id())
            .ok_or_else(|| EffectError::InvalidOperation(
                format!("No handler registered for source domain {}", effect.source_domain_id())
            ))?;
            
        let target_handler = self.get_handler_for_domain(&effect.target_domain_id())
            .ok_or_else(|| EffectError::InvalidOperation(
                format!("No handler registered for target domain {}", effect.target_domain_id())
            ))?;
            
        // Extract the handler from the variants - since we only have Domain variant now
        let HandlerVariant::Domain(source_h) = source_handler;
        let HandlerVariant::Domain(target_h) = target_handler;
        
        // Adapt the context for each domain
        let source_ctx = effect.adapt_source_context(context)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to adapt source context: {}", e)))?;
            
        let target_ctx = effect.adapt_target_context(context)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to adapt target context: {}", e)))?;
        
        // Use the CrossDomainEffectHandling trait to handle the effect
        effect.handle_across_domains(
            source_ctx.as_ref(),
            target_ctx.as_ref(),
            source_h.as_ref(),
            target_h.as_ref(),
        ).await
    }

    /// Check if a handler exists for a specific type in a domain
    pub fn has_handler_for_type(&self, domain_id: &str, _effect_type_id: &EffectTypeId) -> bool {
        // Check if we have any handlers for this domain
        if let Some(handlers) = self.handlers.get(domain_id) {
            // For now, we're not checking specific effect types since domain
            // handlers handle all effects for their domain
            !handlers.is_empty()
        } else {
            false
        }
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
    
    async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // Use success() method to construct a successful outcome with empty data HashMap
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl DomainEffect for BasicDomainEffect {
    /// Get the ID of the domain this effect belongs to.
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the execution boundary for this effect
    fn execution_boundary(&self) -> ExecutionBoundary {
        self.boundary.clone()
    }
    
    /// Validate parameters for this effect
    fn validate(&self) -> Result<(), String> {
        // Basic parameter validation that always succeeds
        Ok(())
    }
    
    /// Get domain-specific parameters
    fn domain_parameters(&self) -> HashMap<String, String> {
        self.parameters.clone()
    }
    
    /// Adapt the context for the domain
    fn adapt_context(&self, _context: &dyn EffectContext) -> Result<(), String> {
        // Return the context as is, wrapped in a success outcome
        Ok(())
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

// Add impl Effect for BasicCrossDomainEffect
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
        // Cross-domain effects should be handled by handle_across_domains,
        // not directly executed via the base Effect trait.
        Err(EffectError::ExecutionError(
            "CrossDomainEffect cannot be executed directly, use handle_across_domains".to_string()
        ))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl DomainEffect for BasicCrossDomainEffect {
    fn domain_id(&self) -> &DomainId {
        &self.source_domain_id
    }
    
    fn execution_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::CrossDomain(self.target_domain_id.clone())
    }
    
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("source_domain".to_string(), self.source_domain_id.clone());
        params.insert("target_domain".to_string(), self.target_domain_id.clone());
        params
    }
    
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
    
    fn adapt_context(&self, _context: &dyn EffectContext) -> Result<(), String> {
        Ok(())
    }
}

#[async_trait]
impl CrossDomainEffect for BasicCrossDomainEffect {
    fn source_domain_id(&self) -> &DomainId {
        &self.source_domain_id
    }
    
    fn target_domain_id(&self) -> &DomainId {
        &self.target_domain_id
    }
    
    fn adapt_source_context(&self, context: &dyn EffectContext) -> Result<Box<dyn EffectContext>, String> {
        Ok(context.clone_context())
    }
    
    fn adapt_target_context(&self, context: &dyn EffectContext) -> Result<Box<dyn EffectContext>, String> {
        Ok(context.clone_context())
    }
    
    fn validate_cross_domain(&self) -> Result<(), String> {
        if self.source_domain_id.is_empty() {
            return Err("Source domain ID cannot be empty".to_string());
        }
        if self.target_domain_id.is_empty() {
            return Err("Target domain ID cannot be empty".to_string());
        }
        if self.source_domain_id == self.target_domain_id {
            return Err("Source and target domain IDs must be different".to_string());
        }
        Ok(())
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

// Add conversion from DomainEffectError to EffectError
impl From<DomainEffectError> for EffectError {
    fn from(err: DomainEffectError) -> Self {
        match err {
            DomainEffectError::DomainNotFound(msg) => EffectError::NotFound(msg),
            DomainEffectError::OperationError(msg) => EffectError::ExecutionError(msg),
            DomainEffectError::ValidationError(msg) => EffectError::ValidationError(msg),
            DomainEffectError::ParameterError(msg) => EffectError::InvalidParameter(msg),
            DomainEffectError::CrossDomainError(msg) => EffectError::ExecutionError(format!("Cross-domain error: {}", msg)),
            DomainEffectError::BoundaryError(msg) => EffectError::ExecutionError(format!("Boundary error: {}", msg)),
            DomainEffectError::InvalidHandlerCombination(msg) => EffectError::ExecutionError(msg),
        }
    }
} 