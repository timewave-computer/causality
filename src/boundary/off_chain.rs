use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use super::annotation::{BoundaryType, CrossingType, BoundarySafe};
use super::crossing::{
    BoundaryCrossingProtocol, 
    BoundaryCrossingPayload, 
    BoundaryAuthentication,
    BoundaryCrossingError,
    BoundaryCrossingResult,
};

/// Types of supported off-chain components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OffChainComponentType {
    /// API Service
    ApiService,
    /// Database Service
    DatabaseService,
    /// File Storage Service
    FileStorageService,
    /// Messaging Service
    MessagingService,
    /// Authentication Service
    AuthenticationService,
    /// Computation Service
    ComputationService,
    /// Custom Service
    CustomService(u32),
}

impl From<OffChainComponentType> for BoundaryType {
    fn from(component_type: OffChainComponentType) -> Self {
        match component_type {
            OffChainComponentType::ApiService => BoundaryType::Custom("api_service".to_string()),
            OffChainComponentType::DatabaseService => BoundaryType::Custom("db_service".to_string()),
            OffChainComponentType::FileStorageService => BoundaryType::Custom("file_service".to_string()),
            OffChainComponentType::MessagingService => BoundaryType::Custom("messaging_service".to_string()),
            OffChainComponentType::AuthenticationService => BoundaryType::Custom("auth_service".to_string()),
            OffChainComponentType::ComputationService => BoundaryType::Custom("computation_service".to_string()),
            OffChainComponentType::CustomService(id) => BoundaryType::Custom(format!("custom_service_{}", id)),
        }
    }
}

/// Identifier for off-chain components
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
    /// Type of component
    pub component_type: OffChainComponentType,
    /// Unique identifier
    pub id: String,
    /// Version
    pub version: String,
}

impl ComponentId {
    /// Create a new component ID
    pub fn new(component_type: OffChainComponentType, id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            component_type,
            id: id.into(),
            version: version.into(),
        }
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.component_type_str(), self.id, self.version)
    }
    
    /// Get the component type as a string
    fn component_type_str(&self) -> &'static str {
        match self.component_type {
            OffChainComponentType::ApiService => "api",
            OffChainComponentType::DatabaseService => "db",
            OffChainComponentType::FileStorageService => "file",
            OffChainComponentType::MessagingService => "messaging",
            OffChainComponentType::AuthenticationService => "auth",
            OffChainComponentType::ComputationService => "computation",
            OffChainComponentType::CustomService(_) => "custom",
        }
    }
}

/// Configuration for an off-chain component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Component ID
    pub id: ComponentId,
    /// Connection details
    pub connection: ConnectionDetails,
    /// Security settings
    pub security: SecuritySettings,
    /// Component-specific configuration
    pub settings: HashMap<String, String>,
}

/// Connection details for an off-chain component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDetails {
    /// Host/endpoint
    pub host: String,
    /// Port
    pub port: Option<u16>,
    /// Protocol
    pub protocol: String,
    /// Path/route
    pub path: Option<String>,
    /// Connection parameters
    pub params: HashMap<String, String>,
}

/// Security settings for an off-chain component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Authentication type
    pub auth_type: String,
    /// Authentication credentials
    pub credentials: Option<HashMap<String, String>>,
    /// TLS/SSL settings
    pub tls_enabled: bool,
    /// Certificate verification
    pub verify_cert: bool,
    /// Rate limiting
    pub rate_limit: Option<u32>,
    /// Timeout in seconds
    pub timeout_seconds: u32,
}

/// Interface for off-chain components
#[async_trait]
pub trait OffChainComponent: Send + Sync + 'static {
    /// Get the component ID
    fn id(&self) -> &ComponentId;
    
    /// Get the component type
    fn component_type(&self) -> OffChainComponentType;
    
    /// Get the component configuration
    fn config(&self) -> &ComponentConfig;
    
    /// Initialize the component
    async fn initialize(&self) -> Result<(), String>;
    
    /// Check if the component is available
    async fn is_available(&self) -> bool;
    
    /// Execute a request on the component
    async fn execute_request(
        &self,
        operation: &str,
        params: HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, String>;
    
    /// Close the component
    async fn close(&self) -> Result<(), String>;
}

/// Request for an off-chain component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRequest {
    /// Component ID
    pub component_id: ComponentId,
    /// Operation to execute
    pub operation: String,
    /// Parameters
    pub params: HashMap<String, Vec<u8>>,
    /// Authentication information
    pub auth: Option<BoundaryAuthentication>,
    /// Request context
    pub context: HashMap<String, String>,
}

impl BoundarySafe for ComponentRequest {
    fn target_boundary(&self) -> BoundaryType {
        self.component_id.component_type.into()
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Failed to deserialize component request: {}", e))
    }
}

/// Response from an off-chain component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentResponse {
    /// Response data
    pub data: Vec<u8>,
    /// Success flag
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
}

impl BoundarySafe for ComponentResponse {
    fn target_boundary(&self) -> BoundaryType {
        BoundaryType::InsideSystem
    }
    
    fn prepare_for_crossing(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    fn from_crossing(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Failed to deserialize component response: {}", e))
    }
}

/// Protocol for off-chain component requests
pub struct OffChainComponentProtocol {
    name: String,
    components: Arc<RwLock<HashMap<ComponentId, Arc<dyn OffChainComponent>>>>,
}

impl OffChainComponentProtocol {
    /// Create a new off-chain component protocol
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            components: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a component
    pub fn register_component(&self, component: Arc<dyn OffChainComponent>) {
        let mut components = self.components.write().unwrap();
        components.insert(component.id().clone(), component);
    }
    
    /// Get a component by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<Arc<dyn OffChainComponent>> {
        let components = self.components.read().unwrap();
        components.get(id).cloned()
    }
    
    /// Find components by type
    pub fn find_components_by_type(&self, component_type: OffChainComponentType) -> Vec<Arc<dyn OffChainComponent>> {
        let components = self.components.read().unwrap();
        components.values()
            .filter(|c| c.component_type() == component_type)
            .cloned()
            .collect()
    }
    
    /// Execute a component request
    async fn execute_component_request(
        &self,
        request: &ComponentRequest,
    ) -> Result<ComponentResponse, String> {
        // Get the component
        let component = self.get_component(&request.component_id)
            .ok_or_else(|| format!("Component not found: {:?}", request.component_id))?;
        
        // Check if the component is available
        if !component.is_available().await {
            return Err(format!("Component is not available: {:?}", request.component_id));
        }
        
        // Execute the request
        match component.execute_request(&request.operation, request.params.clone()).await {
            Ok(data) => Ok(ComponentResponse {
                data,
                success: true,
                error: None,
                metadata: HashMap::new(),
            }),
            Err(e) => Ok(ComponentResponse {
                data: Vec::new(),
                success: false,
                error: Some(e),
                metadata: HashMap::new(),
            }),
        }
    }
}

#[async_trait]
impl BoundaryCrossingProtocol for OffChainComponentProtocol {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn source_boundary(&self) -> BoundaryType {
        BoundaryType::InsideSystem
    }
    
    fn destination_boundary(&self) -> BoundaryType {
        BoundaryType::OutsideSystem
    }
    
    async fn verify_authentication(
        &self,
        payload: &BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<bool> {
        // Extract the request
        let request: ComponentRequest = serde_json::from_slice(&payload.data)
            .map_err(|e| BoundaryCrossingError::InvalidPayload(format!("Invalid request data: {}", e)))?;
        
        // For simplicity, assume authentication is valid
        // In a real implementation, verify credentials
        Ok(true)
    }
    
    async fn process_incoming(
        &self,
        payload: BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<Vec<u8>> {
        // Extract the request
        let request: ComponentRequest = serde_json::from_slice(&payload.data)
            .map_err(|e| BoundaryCrossingError::InvalidPayload(format!("Invalid request data: {}", e)))?;
        
        // Execute the component request
        let response = self.execute_component_request(&request).await
            .map_err(|e| BoundaryCrossingError::ProtocolError(e))?;
        
        // Serialize the response
        let response_data = serde_json::to_vec(&response)
            .map_err(|e| BoundaryCrossingError::InternalError(format!("Failed to serialize response: {}", e)))?;
        
        Ok(response_data)
    }
    
    async fn prepare_outgoing<T: BoundarySafe>(
        &self,
        data: &T,
        auth: BoundaryAuthentication,
    ) -> BoundaryCrossingResult<BoundaryCrossingPayload> {
        // For outgoing calls, data should be ComponentRequest
        let serialized_data = data.prepare_for_crossing();
        
        // Create a new payload
        let payload = BoundaryCrossingPayload::new(
            BoundaryType::InsideSystem,
            BoundaryType::OutsideSystem,
            CrossingType::InsideToOutside,
            serialized_data,
            auth,
        );
        
        Ok(payload)
    }
}

/// Adapter for interacting with off-chain components
pub struct OffChainComponentAdapter {
    protocol: Arc<OffChainComponentProtocol>,
}

impl OffChainComponentAdapter {
    /// Create a new off-chain component adapter
    pub fn new(protocol: Arc<OffChainComponentProtocol>) -> Self {
        Self { protocol }
    }
    
    /// Execute an operation on a component
    pub async fn execute_operation(
        &self,
        component_id: ComponentId,
        operation: &str,
        params: HashMap<String, Vec<u8>>,
        auth: Option<BoundaryAuthentication>,
    ) -> Result<ComponentResponse, String> {
        // Create the request
        let request = ComponentRequest {
            component_id,
            operation: operation.to_string(),
            params,
            auth: auth.clone(),
            context: HashMap::new(),
        };
        
        // Prepare the outgoing payload
        let payload = self.protocol.prepare_outgoing(&request, auth.unwrap_or(BoundaryAuthentication::None)).await
            .map_err(|e| format!("Failed to prepare outgoing request: {}", e))?;
        
        // Process the request
        let response_data = self.protocol.process_incoming(payload).await
            .map_err(|e| format!("Failed to process request: {}", e))?;
        
        // Deserialize the response
        let response: ComponentResponse = serde_json::from_slice(&response_data)
            .map_err(|e| format!("Failed to deserialize response: {}", e))?;
        
        Ok(response)
    }
    
    /// Register a new component
    pub fn register_component(&self, component: Arc<dyn OffChainComponent>) {
        self.protocol.register_component(component);
    }
    
    /// Get a component by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<Arc<dyn OffChainComponent>> {
        self.protocol.get_component(id)
    }
    
    /// Find components by type
    pub fn find_components_by_type(&self, component_type: OffChainComponentType) -> Vec<Arc<dyn OffChainComponent>> {
        self.protocol.find_components_by_type(component_type)
    }
}

/// Registry for off-chain components
pub struct OffChainComponentRegistry {
    adapter: Arc<OffChainComponentAdapter>,
}

impl OffChainComponentRegistry {
    /// Create a new off-chain component registry
    pub fn new() -> Self {
        let protocol = Arc::new(OffChainComponentProtocol::new("off_chain_protocol"));
        let adapter = Arc::new(OffChainComponentAdapter::new(protocol));
        
        Self { adapter }
    }
    
    /// Get the component adapter
    pub fn adapter(&self) -> Arc<OffChainComponentAdapter> {
        self.adapter.clone()
    }
    
    /// Register a new component with the registry
    pub fn register(&self, component: Arc<dyn OffChainComponent>) {
        self.adapter.register_component(component);
    }
    
    /// Initialize all registered components
    pub async fn initialize_all(&self) -> Result<(), String> {
        let components = {
            let protocol = self.adapter.protocol.clone();
            let components_guard = protocol.components.read().unwrap();
            components_guard.values().cloned().collect::<Vec<_>>()
        };
        
        for component in components {
            component.initialize().await?;
        }
        
        Ok(())
    }
    
    /// Close all registered components
    pub async fn close_all(&self) -> Result<(), String> {
        let components = {
            let protocol = self.adapter.protocol.clone();
            let components_guard = protocol.components.read().unwrap();
            components_guard.values().cloned().collect::<Vec<_>>()
        };
        
        for component in components {
            component.close().await?;
        }
        
        Ok(())
    }
} 