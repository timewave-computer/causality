// service.rs - Service Status implementation for agent service advertisement
//
// This file implements the service status system that allows agents to advertise
// services they offer to other agents in the system.

use crate::resource_types::{ResourceId, ResourceType};
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::resource::{Resource, ResourceError};
use crate::serialization::{Serializable, DeserializationError};
use crate::effect::Effect;

use super::types::{AgentId, AgentType, AgentError};
use super::agent::Agent;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Service status error types
#[derive(Error, Debug)]
pub enum ServiceStatusError {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Registration error
    #[error("Registration error: {0}")]
    RegistrationError(String),
    
    /// Discovery error
    #[error("Discovery error: {0}")]
    DiscoveryError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] DeserializationError),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for service status operations
pub type ServiceStatusResult<T> = Result<T, ServiceStatusError>;

/// State of a service
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceState {
    /// Service is available and fully operational
    Available,
    
    /// Service is unavailable
    Unavailable,
    
    /// Service is available but with degraded performance or functionality
    Degraded {
        /// Description of the degradation
        reason: String,
    },
    
    /// Service is in maintenance mode
    Maintenance {
        /// Description of the maintenance
        reason: String,
        /// Expected end time (timestamp)
        expected_end: Option<u64>,
    },
}

/// Version information for a service
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceVersion {
    /// Major version
    pub major: u32,
    
    /// Minor version
    pub minor: u32,
    
    /// Patch version
    pub patch: u32,
    
    /// Pre-release information
    pub pre_release: Option<String>,
}

/// Service status representing a service offered by an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Resource ID
    id: ResourceId,
    
    /// Agent offering the service
    agent_id: AgentId,
    
    /// Service type
    service_type: String,
    
    /// Service version
    version: ServiceVersion,
    
    /// Current state of the service
    state: ServiceState,
    
    /// Capabilities required to use the service
    required_capabilities: Vec<String>,
    
    /// Network endpoint (if applicable)
    endpoint: Option<String>,
    
    /// Service description
    description: Option<String>,
    
    /// Service metadata
    metadata: HashMap<String, String>,
    
    /// Last updated timestamp
    last_updated: u64,
}

impl ServiceStatus {
    /// Create a new service status
    pub fn new(
        agent_id: AgentId,
        service_type: String,
        version: ServiceVersion,
        state: ServiceState,
        required_capabilities: Vec<String>,
    ) -> Self {
        // For now, we use a default content hash for the ID
        // This will be replaced with a proper hash when the service is registered
        let id = ResourceId::new(ContentHash::default());
        let now = chrono::Utc::now().timestamp() as u64;
        
        Self {
            id,
            agent_id,
            service_type,
            version,
            state,
            required_capabilities,
            endpoint: None,
            description: None,
            metadata: HashMap::new(),
            last_updated: now,
        }
    }
    
    /// Get the agent ID
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    
    /// Get the service type
    pub fn service_type(&self) -> &str {
        &self.service_type
    }
    
    /// Get the service version
    pub fn version(&self) -> &ServiceVersion {
        &self.version
    }
    
    /// Get the service state
    pub fn state(&self) -> &ServiceState {
        &self.state
    }
    
    /// Check if the service is available
    pub fn is_available(&self) -> bool {
        matches!(self.state, ServiceState::Available)
    }
    
    /// Get the required capabilities
    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
    
    /// Get the endpoint
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }
    
    /// Set the endpoint
    pub fn set_endpoint(&mut self, endpoint: impl Into<String>) {
        self.endpoint = Some(endpoint.into());
        self.update_timestamp();
    }
    
    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// Set the description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
        self.update_timestamp();
    }
    
    /// Get service metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
        self.update_timestamp();
    }
    
    /// Remove a metadata value
    pub fn remove_metadata(&mut self, key: &str) {
        self.metadata.remove(key);
        self.update_timestamp();
    }
    
    /// Get the last updated timestamp
    pub fn last_updated(&self) -> u64 {
        self.last_updated
    }
    
    /// Update the service state
    pub fn update_state(&mut self, state: ServiceState) {
        self.state = state;
        self.update_timestamp();
    }
    
    /// Update the timestamp
    fn update_timestamp(&mut self) {
        self.last_updated = chrono::Utc::now().timestamp() as u64;
    }
    
    /// Check if the service requires a specific capability
    pub fn requires_capability(&self, capability: &str) -> bool {
        self.required_capabilities.iter().any(|cap| cap == capability)
    }
}

impl crate::resource::Resource for ServiceStatus {
    fn id(&self) -> crate::resource_types::ResourceId {
        self.id.clone()
    }
    
    fn resource_type(&self) -> crate::resource_types::ResourceType {
        crate::resource_types::ResourceType::new("service_status", "1.0")
    }
    
    fn state(&self) -> crate::resource::ResourceState {
        match self.state {
            ServiceState::Available => crate::resource::ResourceState::Active,
            ServiceState::Unavailable => crate::resource::ResourceState::Frozen,
            ServiceState::Degraded { .. } => crate::resource::ResourceState::Active,
            ServiceState::Maintenance { .. } => crate::resource::ResourceState::Locked,
        }
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> crate::resource::ResourceResult<()> {
        self.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn clone_resource(&self) -> Box<dyn crate::resource::Resource> {
        Box::new(self.clone())
    }
}

/// Builder for creating service status objects
pub struct ServiceStatusBuilder {
    /// Agent ID
    agent_id: AgentId,
    
    /// Service type
    service_type: String,
    
    /// Service version
    version: ServiceVersion,
    
    /// Service state
    state: ServiceState,
    
    /// Required capabilities
    required_capabilities: Vec<String>,
    
    /// Endpoint
    endpoint: Option<String>,
    
    /// Description
    description: Option<String>,
    
    /// Metadata
    metadata: HashMap<String, String>,
}

impl ServiceStatusBuilder {
    /// Create a new service status builder
    pub fn new(agent_id: AgentId, service_type: impl Into<String>) -> Self {
        Self {
            agent_id,
            service_type: service_type.into(),
            version: ServiceVersion {
                major: 1,
                minor: 0,
                patch: 0,
                pre_release: None,
            },
            state: ServiceState::Available,
            required_capabilities: Vec::new(),
            endpoint: None,
            description: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the version
    pub fn version(mut self, major: u32, minor: u32, patch: u32) -> Self {
        self.version = ServiceVersion {
            major,
            minor,
            patch,
            pre_release: None,
        };
        self
    }
    
    /// Set pre-release information
    pub fn pre_release(mut self, pre_release: impl Into<String>) -> Self {
        self.version.pre_release = Some(pre_release.into());
        self
    }
    
    /// Set the state
    pub fn state(mut self, state: ServiceState) -> Self {
        self.state = state;
        self
    }
    
    /// Add a required capability
    pub fn require_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }
    
    /// Add multiple required capabilities
    pub fn require_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.required_capabilities.extend(capabilities);
        self
    }
    
    /// Set the endpoint
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }
    
    /// Set the description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Build the service status
    pub fn build(self) -> ServiceStatus {
        let mut service = ServiceStatus::new(
            self.agent_id,
            self.service_type,
            self.version,
            self.state,
            self.required_capabilities,
        );
        
        if let Some(endpoint) = self.endpoint {
            service.set_endpoint(endpoint);
        }
        
        if let Some(description) = self.description {
            service.set_description(description);
        }
        
        for (key, value) in self.metadata {
            service.set_metadata(key, value);
        }
        
        service
    }
}

/// Service status manager for handling service advertisement and discovery
#[derive(Clone)]
pub struct ServiceStatusManager {
    /// Services indexed by ID
    services: Arc<RwLock<HashMap<ResourceId, ServiceStatus>>>,
    
    /// Services indexed by agent ID
    agent_services: Arc<RwLock<HashMap<AgentId, HashSet<ResourceId>>>>,
    
    /// Services indexed by type
    type_services: Arc<RwLock<HashMap<String, HashSet<ResourceId>>>>,
}

impl ServiceStatusManager {
    /// Create a new service status manager
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            agent_services: Arc::new(RwLock::new(HashMap::new())),
            type_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a service
    pub async fn register_service(&self, mut service: ServiceStatus) -> ServiceStatusResult<ResourceId> {
        // Calculate a proper content hash for the service
        let service_data = serde_json::to_vec(&service)
            .map_err(|e| ServiceStatusError::SerializationError(DeserializationError::Other(e.to_string())))?;
        
        let content_hash = ContentHash::calculate(&service_data);
        let service_id = ResourceId::new(content_hash);
        
        // Update the service ID
        service.id = service_id.clone();
        
        // Lock the services map for writing
        let mut services = self.services.write().await;
        
        // Check if the service already exists
        if services.contains_key(&service_id) {
            return Err(ServiceStatusError::RegistrationError(
                format!("Service with ID {} already exists", service_id)
            ));
        }
        
        // Update the indices
        {
            let mut agent_services = self.agent_services.write().await;
            let agent_set = agent_services.entry(service.agent_id.clone()).or_insert_with(HashSet::new);
            agent_set.insert(service_id.clone());
        }
        
        {
            let mut type_services = self.type_services.write().await;
            let type_set = type_services.entry(service.service_type.clone()).or_insert_with(HashSet::new);
            type_set.insert(service_id.clone());
        }
        
        // Add the service
        services.insert(service_id.clone(), service);
        
        Ok(service_id)
    }
    
    /// Unregister a service
    pub async fn unregister_service(&self, service_id: &ResourceId) -> ServiceStatusResult<()> {
        // Lock the services map for writing
        let mut services = self.services.write().await;
        
        // Get the service
        let service = services.get(service_id).ok_or_else(|| {
            ServiceStatusError::RegistrationError(
                format!("Service with ID {} does not exist", service_id)
            )
        })?;
        
        // Update the indices
        {
            let mut agent_services = self.agent_services.write().await;
            if let Some(agent_set) = agent_services.get_mut(&service.agent_id) {
                agent_set.remove(service_id);
                if agent_set.is_empty() {
                    agent_services.remove(&service.agent_id);
                }
            }
        }
        
        {
            let mut type_services = self.type_services.write().await;
            if let Some(type_set) = type_services.get_mut(&service.service_type) {
                type_set.remove(service_id);
                if type_set.is_empty() {
                    type_services.remove(&service.service_type);
                }
            }
        }
        
        // Remove the service
        services.remove(service_id);
        
        Ok(())
    }
    
    /// Update a service's state
    pub async fn update_service_state(
        &self,
        service_id: &ResourceId,
        state: ServiceState,
    ) -> ServiceStatusResult<()> {
        // Lock the services map for writing
        let mut services = self.services.write().await;
        
        // Get the service
        let service = services.get_mut(service_id).ok_or_else(|| {
            ServiceStatusError::RegistrationError(
                format!("Service with ID {} does not exist", service_id)
            )
        })?;
        
        // Update the state
        service.update_state(state);
        
        Ok(())
    }
    
    /// Get a service by ID
    pub async fn get_service(&self, service_id: &ResourceId) -> ServiceStatusResult<ServiceStatus> {
        // Lock the services map for reading
        let services = self.services.read().await;
        
        // Get the service
        let service = services.get(service_id).ok_or_else(|| {
            ServiceStatusError::DiscoveryError(
                format!("Service with ID {} does not exist", service_id)
            )
        })?;
        
        Ok(service.clone())
    }
    
    /// Get all services for an agent
    pub async fn get_agent_services(&self, agent_id: &AgentId) -> ServiceStatusResult<Vec<ServiceStatus>> {
        // Lock the agent services index for reading
        let agent_services = self.agent_services.read().await;
        
        // Get the service IDs for the agent
        let service_ids = agent_services.get(agent_id).map(|set| set.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        
        // Lock the services map for reading
        let services = self.services.read().await;
        
        // Collect the services
        let result = service_ids.iter()
            .filter_map(|id| services.get(id).cloned())
            .collect();
        
        Ok(result)
    }
    
    /// Get all services of a specific type
    pub async fn get_services_by_type(&self, service_type: &str) -> ServiceStatusResult<Vec<ServiceStatus>> {
        // Lock the type services index for reading
        let type_services = self.type_services.read().await;
        
        // Get the service IDs for the type
        let service_ids = type_services.get(service_type).map(|set| set.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        
        // Lock the services map for reading
        let services = self.services.read().await;
        
        // Collect the services
        let result = service_ids.iter()
            .filter_map(|id| services.get(id).cloned())
            .collect();
        
        Ok(result)
    }
    
    /// Find available services of a specific type
    pub async fn find_available_services(&self, service_type: &str) -> ServiceStatusResult<Vec<ServiceStatus>> {
        let all_services = self.get_services_by_type(service_type).await?;
        
        // Filter for available services
        let available = all_services.into_iter()
            .filter(|s| s.is_available())
            .collect();
        
        Ok(available)
    }
    
    /// Find services that require a specific capability
    pub async fn find_services_requiring_capability(&self, capability: &str) -> ServiceStatusResult<Vec<ServiceStatus>> {
        // Lock the services map for reading
        let services = self.services.read().await;
        
        // Filter for services requiring the capability
        let matching = services.values()
            .filter(|s| s.requires_capability(capability))
            .cloned()
            .collect();
        
        Ok(matching)
    }
    
    /// Check if an agent can access a service
    pub async fn can_access_service(
        &self,
        agent: &dyn Agent,
        service_id: &ResourceId,
    ) -> ServiceStatusResult<bool> {
        // Get the service
        let service = self.get_service(service_id).await?;
        
        // Check if the agent has all required capabilities
        let has_capabilities = service.required_capabilities().iter()
            .all(|cap| agent.has_capability(cap));
        
        Ok(has_capabilities)
    }
    
    /// Get all services
    pub async fn get_all_services(&self) -> ServiceStatusResult<Vec<ServiceStatus>> {
        // Lock the services map for reading
        let services = self.services.read().await;
        
        // Collect all services
        let all_services = services.values().cloned().collect();
        
        Ok(all_services)
    }
}

impl Default for ServiceStatusManager {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for Agent to advertise services
#[async_trait]
pub trait ServiceAdvertisement {
    /// Advertise a service
    async fn advertise_service(&self, service: ServiceStatus) -> ServiceStatusResult<ResourceId>;
    
    /// Update a service state
    async fn update_service_state(&self, service_id: &ResourceId, state: ServiceState) -> ServiceStatusResult<()>;
    
    /// Remove a service advertisement
    async fn remove_service(&self, service_id: &ResourceId) -> ServiceStatusResult<()>;
    
    /// Get all services advertised by this agent
    async fn get_advertised_services(&self) -> ServiceStatusResult<Vec<ServiceStatus>>;
}

/// Service status effect that can be used with the effect system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceStatusEffect {
    /// Agent ID
    pub agent_id: AgentId,
    
    /// Effect type
    pub effect_type: ServiceStatusEffectType,
    
    /// Service ID (for update and remove)
    pub service_id: Option<ResourceId>,
    
    /// Service (for register)
    pub service: Option<ServiceStatus>,
    
    /// New state (for update)
    pub new_state: Option<ServiceState>,
}

/// Types of service status effects
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatusEffectType {
    /// Register a new service
    Register,
    
    /// Update an existing service
    Update,
    
    /// Remove a service
    Remove,
}

impl ServiceStatusEffect {
    /// Create a new register effect
    pub fn register(agent_id: AgentId, service: ServiceStatus) -> Self {
        Self {
            agent_id,
            effect_type: ServiceStatusEffectType::Register,
            service_id: None,
            service: Some(service),
            new_state: None,
        }
    }
    
    /// Create a new update effect
    pub fn update(agent_id: AgentId, service_id: ResourceId, new_state: ServiceState) -> Self {
        Self {
            agent_id,
            effect_type: ServiceStatusEffectType::Update,
            service_id: Some(service_id),
            service: None,
            new_state: Some(new_state),
        }
    }
    
    /// Create a new remove effect
    pub fn remove(agent_id: AgentId, service_id: ResourceId) -> Self {
        Self {
            agent_id,
            effect_type: ServiceStatusEffectType::Remove,
            service_id: Some(service_id),
            service: None,
            new_state: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::agent::types::AgentState;
    use crate::resource::agent::agent::AgentImpl;
    use std::sync::Arc;
    
    // Helper function to create a test agent ID
    fn create_test_agent_id() -> AgentId {
        AgentId::from_content_hash(ContentHash::default().as_bytes(), AgentType::User)
    }
    
    #[tokio::test]
    async fn test_service_status_creation() {
        let agent_id = create_test_agent_id();
        
        // Create a service status
        let service = ServiceStatusBuilder::new(agent_id.clone(), "test-service")
            .version(1, 2, 3)
            .pre_release("alpha")
            .state(ServiceState::Available)
            .require_capability("read")
            .require_capability("write")
            .endpoint("https://example.com/api")
            .description("Test service for unit tests")
            .with_metadata("region", "us-west")
            .build();
        
        // Check the service properties
        assert_eq!(service.agent_id(), &agent_id);
        assert_eq!(service.service_type(), "test-service");
        assert_eq!(service.version().major, 1);
        assert_eq!(service.version().minor, 2);
        assert_eq!(service.version().patch, 3);
        assert_eq!(service.version().pre_release, Some("alpha".to_string()));
        assert!(matches!(service.state(), ServiceState::Available));
        assert_eq!(service.required_capabilities().len(), 2);
        assert!(service.requires_capability("read"));
        assert!(service.requires_capability("write"));
        assert_eq!(service.endpoint(), Some("https://example.com/api"));
        assert_eq!(service.description(), Some("Test service for unit tests"));
        assert_eq!(service.get_metadata("region"), Some(&"us-west".to_string()));
    }
    
    #[tokio::test]
    async fn test_service_state_updates() {
        let agent_id = create_test_agent_id();
        
        // Create a service status
        let mut service = ServiceStatusBuilder::new(agent_id.clone(), "test-service")
            .state(ServiceState::Available)
            .build();
        
        // Check initial state
        assert!(matches!(service.state(), ServiceState::Available));
        
        // Update to degraded
        service.update_state(ServiceState::Degraded {
            reason: "High load".to_string(),
        });
        
        // Check updated state
        if let ServiceState::Degraded { reason } = service.state() {
            assert_eq!(reason, "High load");
        } else {
            panic!("Expected Degraded state");
        }
        
        // Update to maintenance
        let now = chrono::Utc::now().timestamp() as u64;
        service.update_state(ServiceState::Maintenance {
            reason: "Scheduled maintenance".to_string(),
            expected_end: Some(now + 3600), // 1 hour from now
        });
        
        // Check updated state
        if let ServiceState::Maintenance { reason, expected_end } = service.state() {
            assert_eq!(reason, "Scheduled maintenance");
            assert!(expected_end.is_some());
        } else {
            panic!("Expected Maintenance state");
        }
    }
    
    #[tokio::test]
    async fn test_service_manager_registration() {
        let agent_id = create_test_agent_id();
        let manager = ServiceStatusManager::new();
        
        // Create a service status
        let service = ServiceStatusBuilder::new(agent_id.clone(), "test-service")
            .state(ServiceState::Available)
            .build();
        
        // Register the service
        let service_id = manager.register_service(service.clone()).await.unwrap();
        
        // Get the service and verify it
        let retrieved = manager.get_service(&service_id).await.unwrap();
        assert_eq!(retrieved.service_type(), "test-service");
        assert_eq!(retrieved.agent_id(), &agent_id);
    }
    
    #[tokio::test]
    async fn test_service_type_discovery() {
        let agent_id = create_test_agent_id();
        let manager = ServiceStatusManager::new();
        
        // Create and register services of different types
        let service1 = ServiceStatusBuilder::new(agent_id.clone(), "database")
            .state(ServiceState::Available)
            .build();
        
        let service2 = ServiceStatusBuilder::new(agent_id.clone(), "api")
            .state(ServiceState::Available)
            .build();
        
        let service3 = ServiceStatusBuilder::new(agent_id.clone(), "database")
            .state(ServiceState::Degraded { reason: "Slow".to_string() })
            .build();
        
        manager.register_service(service1).await.unwrap();
        manager.register_service(service2).await.unwrap();
        manager.register_service(service3).await.unwrap();
        
        // Find services by type
        let database_services = manager.get_services_by_type("database").await.unwrap();
        assert_eq!(database_services.len(), 2);
        
        let api_services = manager.get_services_by_type("api").await.unwrap();
        assert_eq!(api_services.len(), 1);
        
        // Find available services
        let available_db = manager.find_available_services("database").await.unwrap();
        assert_eq!(available_db.len(), 1);
    }
    
    #[tokio::test]
    async fn test_agent_services() {
        let agent1_id = create_test_agent_id();
        let agent2_id = AgentId::from_content_hash(ContentHash::calculate(b"agent2").as_bytes(), AgentType::User);
        let manager = ServiceStatusManager::new();
        
        // Create services for different agents
        let service1 = ServiceStatusBuilder::new(agent1_id.clone(), "service1")
            .state(ServiceState::Available)
            .build();
        
        let service2 = ServiceStatusBuilder::new(agent1_id.clone(), "service2")
            .state(ServiceState::Available)
            .build();
        
        let service3 = ServiceStatusBuilder::new(agent2_id.clone(), "service3")
            .state(ServiceState::Available)
            .build();
        
        manager.register_service(service1).await.unwrap();
        manager.register_service(service2).await.unwrap();
        manager.register_service(service3).await.unwrap();
        
        // Get services by agent
        let agent1_services = manager.get_agent_services(&agent1_id).await.unwrap();
        assert_eq!(agent1_services.len(), 2);
        
        let agent2_services = manager.get_agent_services(&agent2_id).await.unwrap();
        assert_eq!(agent2_services.len(), 1);
    }
    
    #[tokio::test]
    async fn test_service_unregistration() {
        let agent_id = create_test_agent_id();
        let manager = ServiceStatusManager::new();
        
        // Create and register a service
        let service = ServiceStatusBuilder::new(agent_id.clone(), "test-service")
            .state(ServiceState::Available)
            .build();
        
        let service_id = manager.register_service(service).await.unwrap();
        
        // Verify registration
        assert_eq!(manager.get_all_services().await.unwrap().len(), 1);
        
        // Unregister the service
        manager.unregister_service(&service_id).await.unwrap();
        
        // Verify unregistration
        assert_eq!(manager.get_all_services().await.unwrap().len(), 0);
        
        // Attempting to get the service should now fail
        let result = manager.get_service(&service_id).await;
        assert!(result.is_err());
    }
} 