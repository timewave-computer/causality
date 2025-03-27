// Domain Adapter Implementation for Testing
//
// This module provides concrete implementations of domain adapters for 
// testing and demonstration purposes.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::capability::{Capability, BasicCapability};
use crate::content::ContentId;
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    domain::{
        DomainId, DomainEffect, DomainEffectHandler, DomainEffectOutcome,
        ParameterValidationResult, DomainParameterValidator,
    },
};
use crate::resource::{
    ResourceTypeId, CrossDomainResourceId, ResourceProjectionType, 
    VerificationLevel, ResourceReference, ResourceTransferOperation,
    CrossDomainResourceProtocol, DomainResourceAdapter,
    CrossDomainProtocolResult, CrossDomainProtocolError,
};

use super::domain::{
    DomainIntegrationError, DomainIntegrationResult,
    GenericDomainAdapter, DomainAdapterFactory,
};

/// Test domain parameter validator
pub struct TestParameterValidator {
    required_params: Vec<String>,
    optional_params: Vec<String>,
}

impl TestParameterValidator {
    /// Create a new test parameter validator
    pub fn new() -> Self {
        Self {
            required_params: vec!["action".to_string(), "resource_id".to_string()],
            optional_params: vec!["timestamp".to_string(), "metadata".to_string()],
        }
    }
}

impl DomainParameterValidator for TestParameterValidator {
    /// Validate domain parameters
    fn validate(&self, parameters: &HashMap<String, String>) -> ParameterValidationResult {
        let mut result = ParameterValidationResult::success(parameters.clone());
        
        // Check required parameters
        for param in &self.required_params {
            if !parameters.contains_key(param) {
                result.add_error(format!("Missing required parameter: {}", param));
            }
        }
        
        // Check parameter values
        if let Some(action) = parameters.get("action") {
            if !["create", "read", "update", "delete", "transfer"].contains(&action.as_str()) {
                result.add_error(format!("Invalid action: {}", action));
            }
        }
        
        result
    }
    
    /// Get required parameters
    fn required_parameters(&self) -> Vec<String> {
        self.required_params.clone()
    }
    
    /// Get optional parameters
    fn optional_parameters(&self) -> Vec<String> {
        self.optional_params.clone()
    }
    
    /// Check if a parameter is valid for this domain
    fn is_valid_parameter(&self, name: &str, value: &str) -> bool {
        if name == "action" {
            return ["create", "read", "update", "delete", "transfer"].contains(&value);
        }
        
        true
    }
}

/// Test domain effect handler
pub struct TestDomainEffectHandler {
    /// Domain ID
    domain_id: DomainId,
    
    /// Domain adapter
    adapter: GenericDomainAdapter,
    
    /// Execution log
    execution_log: Arc<Mutex<Vec<String>>>,
}

impl TestDomainEffectHandler {
    /// Create a new test domain effect handler
    pub fn new(domain_id: DomainId) -> Self {
        let mut adapter = GenericDomainAdapter::new(domain_id.clone());
        
        // Add supported resource types
        adapter.add_resource_type(ResourceTypeId::new("document"));
        adapter.add_resource_type(ResourceTypeId::new("user"));
        
        // Add required capabilities
        adapter.add_required_capability("create", BasicCapability::new("resource.create"));
        adapter.add_required_capability("read", BasicCapability::new("resource.read"));
        adapter.add_required_capability("update", BasicCapability::new("resource.update"));
        adapter.add_required_capability("delete", BasicCapability::new("resource.delete"));
        adapter.add_required_capability("transfer", BasicCapability::new("resource.transfer"));
        
        // Add parameter validator
        adapter.add_parameter_validator(Arc::new(TestParameterValidator::new()));
        
        Self {
            domain_id,
            adapter,
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get execution log
    pub fn execution_log(&self) -> Vec<String> {
        let log = self.execution_log.lock().unwrap();
        log.clone()
    }
    
    /// Log an execution
    fn log_execution(&self, message: &str) {
        let mut log = self.execution_log.lock().unwrap();
        log.push(message.to_string());
    }
}

impl std::fmt::Debug for TestDomainEffectHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestDomainEffectHandler")
            .field("domain_id", &self.domain_id)
            .finish()
    }
}

#[async_trait]
impl DomainEffectHandler for TestDomainEffectHandler {
    /// Get the domain ID this handler operates on
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Handle a domain effect
    async fn handle_domain_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Log the effect handling
        self.log_execution(&format!("Handling effect: {:?} in domain: {}", effect.id(), self.domain_id));
        
        // Validate parameters
        let parameters = effect.domain_parameters();
        let validation_result = self.adapter.validate_parameters(&parameters);
        
        if !validation_result.valid {
            let errors = validation_result.errors.join(", ");
            return Err(EffectError::ValidationError(errors));
        }
        
        // Check capabilities
        if let Some(action) = parameters.get("action") {
            if let Err(e) = self.adapter.validate_capabilities(action, context.capabilities()) {
                return Err(EffectError::ValidationError(e.to_string()));
            }
        }
        
        // Handle the effect based on the action
        let action = parameters.get("action").unwrap_or(&"read".to_string());
        let result = match action.as_str() {
            "create" => {
                self.log_execution("Creating resource");
                let mut result_data = HashMap::new();
                result_data.insert("status".to_string(), "created".to_string());
                result_data.insert("resource_id".to_string(), parameters.get("resource_id").unwrap_or(&"unknown".to_string()).clone());
                
                // Return success outcome
                DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    effect.id().clone(),
                    Some(result_data),
                )
            },
            "read" => {
                self.log_execution("Reading resource");
                let mut result_data = HashMap::new();
                result_data.insert("status".to_string(), "read".to_string());
                result_data.insert("resource_id".to_string(), parameters.get("resource_id").unwrap_or(&"unknown".to_string()).clone());
                
                // Return success outcome
                DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    effect.id().clone(),
                    Some(result_data),
                )
            },
            "update" => {
                self.log_execution("Updating resource");
                let mut result_data = HashMap::new();
                result_data.insert("status".to_string(), "updated".to_string());
                result_data.insert("resource_id".to_string(), parameters.get("resource_id").unwrap_or(&"unknown".to_string()).clone());
                
                // Return success outcome
                DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    effect.id().clone(),
                    Some(result_data),
                )
            },
            "delete" => {
                self.log_execution("Deleting resource");
                let mut result_data = HashMap::new();
                result_data.insert("status".to_string(), "deleted".to_string());
                result_data.insert("resource_id".to_string(), parameters.get("resource_id").unwrap_or(&"unknown".to_string()).clone());
                
                // Return success outcome
                DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    effect.id().clone(),
                    Some(result_data),
                )
            },
            "transfer" => {
                self.log_execution("Transferring resource");
                let mut result_data = HashMap::new();
                result_data.insert("status".to_string(), "transferred".to_string());
                result_data.insert("resource_id".to_string(), parameters.get("resource_id").unwrap_or(&"unknown".to_string()).clone());
                
                // Return success outcome
                DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    effect.id().clone(),
                    Some(result_data),
                )
            },
            _ => {
                // Unknown action
                return Err(EffectError::ValidationError(format!("Unknown action: {}", action)));
            }
        };
        
        // Convert to EffectOutcome
        Ok(result.to_effect_outcome())
    }
}

/// Test domain resource adapter
pub struct TestDomainResourceAdapter {
    /// Domain ID
    domain_id: DomainId,
    
    /// Resource references
    references: Arc<Mutex<HashMap<String, ResourceReference>>>,
}

impl TestDomainResourceAdapter {
    /// Create a new test domain resource adapter
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            references: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get a resource reference by ID
    pub fn get_reference(&self, id: &str) -> Option<ResourceReference> {
        let references = self.references.lock().unwrap();
        references.get(id).cloned()
    }
    
    /// Add a resource reference
    fn add_reference(&self, reference: ResourceReference) {
        let mut references = self.references.lock().unwrap();
        references.insert(reference.id.to_string(), reference);
    }
}

impl std::fmt::Debug for TestDomainResourceAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestDomainResourceAdapter")
            .field("domain_id", &self.domain_id)
            .finish()
    }
}

#[async_trait]
impl DomainResourceAdapter for TestDomainResourceAdapter {
    /// Get the domain ID this adapter operates on
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Check if a resource exists in this domain
    async fn has_resource(
        &self,
        resource_id: &CrossDomainResourceId,
        _context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<bool> {
        let references = self.references.lock().unwrap();
        Ok(references.contains_key(&resource_id.to_string()))
    }
    
    /// Create a resource reference in this domain
    async fn create_reference(
        &self,
        resource_id: CrossDomainResourceId,
        projection_type: ResourceProjectionType,
        verification_level: VerificationLevel,
        _source_data: Option<Vec<u8>>,
        _context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference> {
        // Create reference
        let reference = ResourceReference::new(
            resource_id,
            projection_type,
            verification_level,
            self.domain_id.clone(),
        );
        
        // Store reference
        self.add_reference(reference.clone());
        
        Ok(reference)
    }
    
    /// Get a resource reference from this domain
    async fn get_reference(
        &self,
        resource_id: &CrossDomainResourceId,
        _context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<Option<ResourceReference>> {
        let references = self.references.lock().unwrap();
        Ok(references.get(&resource_id.to_string()).cloned())
    }
    
    /// Verify a resource reference
    async fn verify_reference(
        &self,
        reference: &ResourceReference,
        _context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<bool> {
        let references = self.references.lock().unwrap();
        Ok(references.contains_key(&reference.id.to_string()))
    }
}

/// Test domain adapter factory
pub struct TestDomainAdapterFactory {
    /// Domain adapters
    domain_adapters: Arc<Mutex<HashMap<DomainId, (
        Arc<dyn DomainEffectHandler>,
        Arc<dyn DomainResourceAdapter>,
    )>>>,
}

impl TestDomainAdapterFactory {
    /// Create a new test domain adapter factory
    pub fn new() -> Self {
        Self {
            domain_adapters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register test domains
    pub fn register_test_domains(&self) -> &Self {
        let mut adapters = self.domain_adapters.lock().unwrap();
        
        // Create and register test domains
        for domain_name in &["test", "finance", "content", "user"] {
            let domain_id = DomainId::new(domain_name);
            let effect_handler = Arc::new(TestDomainEffectHandler::new(domain_id.clone()));
            let resource_adapter = Arc::new(TestDomainResourceAdapter::new(domain_id.clone()));
            
            adapters.insert(domain_id, (effect_handler, resource_adapter));
        }
        
        self
    }
}

impl std::fmt::Debug for TestDomainAdapterFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let adapters = self.domain_adapters.lock().unwrap();
        f.debug_struct("TestDomainAdapterFactory")
            .field("domains", &adapters.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[async_trait]
impl DomainAdapterFactory for TestDomainAdapterFactory {
    /// Create a domain effect handler for the specified domain
    async fn create_effect_handler(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainEffectHandler>> {
        let adapters = self.domain_adapters.lock().unwrap();
        
        if let Some((handler, _)) = adapters.get(domain_id) {
            Ok(handler.clone())
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
        let adapters = self.domain_adapters.lock().unwrap();
        
        if let Some((_, adapter)) = adapters.get(domain_id) {
            Ok(adapter.clone())
        } else {
            Err(DomainIntegrationError::DomainNotSupported(
                format!("Domain {} not supported", domain_id)
            ))
        }
    }
    
    /// Get supported domains
    fn supported_domains(&self) -> Vec<DomainId> {
        let adapters = self.domain_adapters.lock().unwrap();
        adapters.keys().cloned().collect()
    }
}

/// Create a test domain integration layer with pre-configured domains
pub fn create_test_domain_integration_layer(
    cross_domain_protocol: Arc<dyn CrossDomainResourceProtocol>,
) -> (
    Arc<crate::integration::domain::DomainEffectRouter>,
    Arc<crate::integration::domain::DomainResourceRouter>,
    Arc<TestDomainAdapterFactory>,
) {
    // Create the domain adapter factory
    let adapter_factory = Arc::new(TestDomainAdapterFactory::new());
    adapter_factory.register_test_domains();
    
    // Create the routers
    let effect_router = Arc::new(crate::integration::domain::DomainEffectRouter::new(
        adapter_factory.clone() as Arc<dyn DomainAdapterFactory>
    ));
    
    let resource_router = Arc::new(crate::integration::domain::DomainResourceRouter::new(
        adapter_factory.clone() as Arc<dyn DomainAdapterFactory>,
        cross_domain_protocol,
    ));
    
    (effect_router, resource_router, adapter_factory)
} 