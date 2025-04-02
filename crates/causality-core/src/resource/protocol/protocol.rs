// Cross-Domain Resource Protocol
//
// This module provides interfaces and implementations for secure cross-domain
// resource references and transfer mechanisms between domains.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use causality_types::ContentId;
use causality_types::domain::DomainId;
use crate::capability::resource::{ResourceCapability, ResourceCapabilityType, CapabilityGrants, Capability};
use crate::identity::IdentityId;
use crate::effect::EffectContext;
use crate::resource::types::{ResourceTypeId, ResourceTypeRegistry, ResourceTypeRegistryError};
use crate::resource::Resource;
use std::sync::RwLock;

/// Unique identifier for a cross-domain resource reference
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrossDomainResourceId {
    /// Content hash of the resource
    pub content_id: ContentId,
    
    /// Source domain where the resource exists
    pub source_domain: DomainId,
    
    /// Resource type
    pub resource_type: ResourceTypeId,
}

impl CrossDomainResourceId {
    /// Create a new cross-domain resource ID
    pub fn new(content_id: ContentId, source_domain: DomainId, resource_type: ResourceTypeId) -> Self {
        Self {
            content_id,
            source_domain,
            resource_type,
        }
    }
}

/// Resource projection types for cross-domain resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceProjectionType {
    /// Read-only projection of a resource
    Shadow,
    
    /// Mutable projection with synchronized state
    Bridged,
    
    /// Resource locked in source domain and represented in target
    Locked,
    
    /// Fully transferred resource
    Transferred,
    
    /// Custom projection type
    Custom(u8),
}

/// Verification level for cross-domain resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// No verification
    None,
    
    /// Hash-based verification
    Hash,
    
    /// Merkle proof verification
    MerkleProof,
    
    /// Zero-knowledge proof verification
    ZkProof,
    
    /// Consensus-based verification
    Consensus,
    
    /// Multi-signature verification
    MultiSig,
    
    /// Custom verification level
    Custom(u8),
}

/// Cross-domain resource reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    /// Resource ID in the source domain
    pub id: CrossDomainResourceId,
    
    /// Projection type
    pub projection_type: ResourceProjectionType,
    
    /// Verification level
    pub verification_level: VerificationLevel,
    
    /// Target domain where this reference exists
    pub target_domain: DomainId,
    
    /// Reference creation timestamp
    pub created_at: u64,
    
    /// Reference expiration timestamp (if any)
    pub expires_at: Option<u64>,
    
    /// Reference metadata
    pub metadata: HashMap<String, String>,
}

impl ResourceReference {
    /// Create a new resource reference
    pub fn new(
        id: CrossDomainResourceId,
        projection_type: ResourceProjectionType,
        verification_level: VerificationLevel,
        target_domain: DomainId,
    ) -> Self {
        Self {
            id,
            projection_type,
            verification_level,
            target_domain,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if this reference is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            expires_at < now
        } else {
            false
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Resource reference verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    /// Reference is valid
    Valid,
    
    /// Reference is invalid
    Invalid(String),
    
    /// Verification is not supported
    Unsupported,
    
    /// Verification failed due to an error
    Error(String),
}

/// Resource transfer status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    /// Transfer is pending
    Pending,
    
    /// Transfer is in progress
    InProgress(f32), // Progress percentage
    
    /// Transfer is complete
    Complete,
    
    /// Transfer failed
    Failed(String),
    
    /// Transfer was cancelled
    Cancelled,
}

/// Resource transfer operation
#[derive(Debug)]
pub struct ResourceTransferOperation {
    /// Transfer ID
    pub id: String,
    
    /// Resource being transferred
    pub resource_id: CrossDomainResourceId,
    
    /// Source domain
    pub source_domain: DomainId,
    
    /// Target domain
    pub target_domain: DomainId,
    
    /// Projection type in target domain
    pub projection_type: ResourceProjectionType,
    
    /// Verification level
    pub verification_level: VerificationLevel,
    
    /// Authorization capability
    pub authorization: Capability<Box<dyn Resource>>,
    
    /// Transfer status
    pub status: TransferStatus,
    
    /// Resource data (if available)
    pub resource_data: Option<Vec<u8>>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Clone for ResourceTransferOperation {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            resource_id: self.resource_id.clone(),
            source_domain: self.source_domain.clone(),
            target_domain: self.target_domain.clone(),
            projection_type: self.projection_type.clone(),
            verification_level: self.verification_level.clone(),
            authorization: create_dummy_capability(),
            status: self.status.clone(),
            resource_data: self.resource_data.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl ResourceTransferOperation {
    /// Create a new resource transfer operation
    pub fn new(
        resource_id: CrossDomainResourceId,
        source_domain: DomainId,
        target_domain: DomainId,
        projection_type: ResourceProjectionType,
        verification_level: VerificationLevel,
        authorization: Capability<Box<dyn Resource>>,
    ) -> Self {
        Self {
            id: crate::id_utils::generate_transfer_id(),
            resource_id,
            source_domain,
            target_domain,
            projection_type,
            verification_level,
            authorization,
            status: TransferStatus::Pending,
            resource_data: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set transfer status
    pub fn with_status(mut self, status: TransferStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set resource data
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.resource_data = Some(data);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Errors that can occur in cross-domain protocol operations
#[derive(Debug, Error)]
pub enum CrossDomainProtocolError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Domain not supported: {0}")]
    DomainNotSupported(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Transfer failed: {0}")]
    TransferFailed(String),
    
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    #[error("Resource type error: {0}")]
    ResourceTypeError(#[from] ResourceTypeRegistryError),
    
    #[error("Domain error: {0}")]
    DomainError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for cross-domain protocol operations
pub type CrossDomainProtocolResult<T> = Result<T, CrossDomainProtocolError>;

/// Cross-domain resource protocol interface
#[async_trait]
pub trait CrossDomainResourceProtocol: Send + Sync + Debug {
    /// Check if this protocol supports a given domain
    async fn supports_domain(&self, domain_id: &DomainId) -> bool;
    
    /// Create a reference to a resource in another domain
    async fn create_reference(
        &self,
        resource_id: CrossDomainResourceId,
        target_domain: DomainId,
        projection_type: ResourceProjectionType,
        verification_level: VerificationLevel,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference>;
    
    /// Verify a resource reference is valid
    async fn verify_reference(
        &self,
        reference: &ResourceReference,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<VerificationResult>;
    
    /// Resolve a resource reference to get its data
    async fn resolve_reference(
        &self,
        reference: &ResourceReference,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<Vec<u8>>;
    
    /// Transfer a resource between domains
    async fn transfer_resource(
        &self,
        operation: ResourceTransferOperation,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference>;
    
    /// Get the status of a transfer operation
    async fn get_transfer_status(
        &self,
        transfer_id: &str,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<TransferStatus>;
    
    /// Synchronize a resource reference with its source
    async fn synchronize_reference(
        &self,
        reference: &ResourceReference,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference>;
}

/// Trait for domain-specific resource adapters
#[async_trait]
pub trait DomainResourceAdapter: Send + Sync + Debug {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Check if this adapter supports a given resource type
    async fn supports_resource_type(&self, resource_type: &ResourceTypeId) -> bool;
    
    /// Store a resource in this domain
    async fn store_resource(
        &self,
        resource_id: &CrossDomainResourceId,
        data: &[u8],
        metadata: &HashMap<String, String>,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<()>;
    
    /// Retrieve a resource from this domain
    async fn retrieve_resource(
        &self,
        resource_id: &CrossDomainResourceId,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<Vec<u8>>;
    
    /// Check if a resource exists in this domain
    async fn has_resource(
        &self,
        resource_id: &CrossDomainResourceId,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<bool>;
    
    /// Verify a resource in this domain
    async fn verify_resource(
        &self,
        resource_id: &CrossDomainResourceId,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<VerificationResult>;
    
    /// Transfer a resource to another domain
    async fn transfer_resource(
        &self,
        operation: &ResourceTransferOperation,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<()>;
    
    /// Receive a resource from another domain
    async fn receive_resource(
        &self,
        operation: &ResourceTransferOperation,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<()>;
    
    /// Update a projected resource
    async fn update_projection(
        &self,
        reference: &ResourceReference,
        data: &[u8],
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<()>;
}

/// Cross-domain resource protocol implementation
#[derive(Debug, Clone)]
pub struct BasicCrossDomainResourceProtocol {
    /// Resource type registry
    resource_type_registry: Arc<dyn ResourceTypeRegistry>,
    
    /// Domain adapters
    domain_adapters: HashMap<DomainId, Arc<dyn DomainResourceAdapter>>,
    
    /// Reference store
    references: HashMap<String, ResourceReference>,
    
    /// Transfer operations
    transfers: HashMap<String, ResourceTransferOperation>,
}

impl BasicCrossDomainResourceProtocol {
    /// Create a new basic cross-domain resource protocol
    pub fn new(resource_type_registry: Arc<dyn ResourceTypeRegistry>) -> Self {
        Self {
            resource_type_registry,
            domain_adapters: HashMap::new(),
            references: HashMap::new(),
            transfers: HashMap::new(),
        }
    }
    
    /// Register a domain adapter
    pub fn register_adapter(&mut self, adapter: Arc<dyn DomainResourceAdapter>) {
        self.domain_adapters.insert(adapter.domain_id().clone(), adapter);
    }
    
    /// Get a domain adapter
    pub fn get_adapter(&self, domain_id: &DomainId) -> CrossDomainProtocolResult<Arc<dyn DomainResourceAdapter>> {
        self.domain_adapters.get(domain_id)
            .cloned()
            .ok_or_else(|| CrossDomainProtocolError::DomainNotSupported(domain_id.to_string()))
    }
    
    /// Store a reference
    fn store_reference(&mut self, reference: ResourceReference) {
        let key = format!("{}:{}", reference.id.content_id, reference.target_domain);
        self.references.insert(key, reference);
    }
    
    /// Get a reference
    fn get_reference(&self, resource_id: &CrossDomainResourceId, target_domain: &DomainId) -> Option<ResourceReference> {
        let key = format!("{}:{}", resource_id.content_id, target_domain);
        self.references.get(&key).cloned()
    }
    
    /// Store a transfer operation
    fn store_transfer(&mut self, operation: ResourceTransferOperation) {
        self.transfers.insert(operation.id.clone(), operation);
    }
    
    /// Get a transfer operation
    fn get_transfer(&self, transfer_id: &str) -> Option<ResourceTransferOperation> {
        self.transfers.get(transfer_id).cloned()
    }
    
    /// Update a transfer operation
    fn update_transfer(&mut self, operation: ResourceTransferOperation) {
        self.transfers.insert(operation.id.clone(), operation);
    }
}

#[async_trait]
impl CrossDomainResourceProtocol for BasicCrossDomainResourceProtocol {
    async fn supports_domain(&self, domain_id: &DomainId) -> bool {
        self.domain_adapters.contains_key(domain_id)
    }
    
    async fn create_reference(
        &self,
        resource_id: CrossDomainResourceId,
        target_domain: DomainId,
        projection_type: ResourceProjectionType,
        verification_level: VerificationLevel,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference> {
        // Check if source domain is supported
        let source_adapter = self.get_adapter(&resource_id.source_domain)?;
        
        // Check if target domain is supported
        let target_adapter = self.get_adapter(&target_domain)?;
        
        // Check if resource exists in source domain
        if !source_adapter.has_resource(&resource_id, context).await? {
            return Err(CrossDomainProtocolError::ResourceNotFound(
                format!("Resource {} not found in domain {}", resource_id.content_id, resource_id.source_domain)
            ));
        }
        
        // Check if resource type is supported in target domain
        if !target_adapter.supports_resource_type(&resource_id.resource_type).await {
            return Err(CrossDomainProtocolError::ProtocolError(
                format!("Resource type {} not supported in domain {}", resource_id.resource_type, target_domain)
            ));
        }
        
        // Create the reference
        let reference = ResourceReference::new(
            resource_id,
            projection_type,
            verification_level,
            target_domain,
        );
        
        // Store the reference (in a real implementation this would be persistent)
        let mut protocol = self.clone();
        protocol.store_reference(reference.clone());
        
        Ok(reference)
    }
    
    async fn verify_reference(
        &self,
        reference: &ResourceReference,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<VerificationResult> {
        // Check if reference is expired
        if reference.is_expired() {
            return Ok(VerificationResult::Invalid("Reference has expired".to_string()));
        }
        
        // Get the source adapter
        let source_adapter = self.get_adapter(&reference.id.source_domain)?;
        
        // Verify the resource exists
        if !source_adapter.has_resource(&reference.id, context).await? {
            return Ok(VerificationResult::Invalid("Resource no longer exists in source domain".to_string()));
        }
        
        // Perform domain-specific verification
        source_adapter.verify_resource(&reference.id, context).await
    }
    
    async fn resolve_reference(
        &self,
        reference: &ResourceReference,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<Vec<u8>> {
        // Check if reference is expired
        if reference.is_expired() {
            return Err(CrossDomainProtocolError::VerificationFailed("Reference has expired".to_string()));
        }
        
        // Get the source adapter
        let source_adapter = self.get_adapter(&reference.id.source_domain)?;
        
        // Retrieve the resource
        source_adapter.retrieve_resource(&reference.id, context).await
    }
    
    async fn transfer_resource(
        &self,
        operation: ResourceTransferOperation,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference> {
        // Check if source domain is supported
        let source_adapter = self.get_adapter(&operation.source_domain)?;
        
        // Check if target domain is supported
        let target_adapter = self.get_adapter(&operation.target_domain)?;
        
        // Check if resource exists in source domain
        if !source_adapter.has_resource(&operation.resource_id, context).await? {
            return Err(CrossDomainProtocolError::ResourceNotFound(
                format!("Resource {} not found in domain {}", 
                    operation.resource_id.content_id, 
                    operation.resource_id.source_domain
                )
            ));
        }
        
        // Start the transfer (source domain prepares the resource)
        let mut operation = operation.clone();
        operation.status = TransferStatus::InProgress(0.0);
        
        let mut protocol = self.clone();
        protocol.store_transfer(operation.clone());
        
        // Execute the transfer on the source domain
        source_adapter.transfer_resource(&operation, context).await?;
        
        // Update transfer status
        operation.status = TransferStatus::InProgress(50.0);
        protocol.update_transfer(operation.clone());
        
        // Receive the resource on the target domain
        target_adapter.receive_resource(&operation, context).await?;
        
        // Update transfer status
        operation.status = TransferStatus::Complete;
        protocol.update_transfer(operation.clone());
        
        // Create a reference for the transferred resource
        let reference = ResourceReference::new(
            operation.resource_id,
            operation.projection_type,
            operation.verification_level,
            operation.target_domain,
        );
        
        // Store the reference
        protocol.store_reference(reference.clone());
        
        Ok(reference)
    }
    
    async fn get_transfer_status(
        &self,
        transfer_id: &str,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<TransferStatus> {
        // Get the transfer operation
        let operation = self.get_transfer(transfer_id)
            .ok_or_else(|| CrossDomainProtocolError::ProtocolError(
                format!("Transfer operation not found: {}", transfer_id)
            ))?;
        
        Ok(operation.status)
    }
    
    async fn synchronize_reference(
        &self,
        reference: &ResourceReference,
        context: &dyn EffectContext,
    ) -> CrossDomainProtocolResult<ResourceReference> {
        // Check if source domain is supported
        let source_adapter = self.get_adapter(&reference.id.source_domain)?;
        
        // Check if target domain is supported
        let target_adapter = self.get_adapter(&reference.target_domain)?;
        
        // Check if resource exists in source domain
        if !source_adapter.has_resource(&reference.id, context).await? {
            return Err(CrossDomainProtocolError::ResourceNotFound(
                format!("Resource {} not found in domain {}", 
                    reference.id.content_id, 
                    reference.id.source_domain
                )
            ));
        }
        
        // Get the latest resource data
        let data = source_adapter.retrieve_resource(&reference.id, context).await?;
        
        // Update the projection in the target domain
        target_adapter.update_projection(reference, &data, context).await?;
        
        // Return the reference (unchanged in this basic implementation)
        Ok(reference.clone())
    }
}

/// Create a new cross-domain resource protocol
pub fn create_cross_domain_protocol(
    resource_type_registry: Arc<dyn ResourceTypeRegistry>,
) -> Arc<dyn CrossDomainResourceProtocol> {
    Arc::new(BasicCrossDomainResourceProtocol::new(resource_type_registry))
}

/// Authorization for a resource protocol operation
#[derive(Debug)]
pub struct ProtocolAuthorization {
    /// The agent ID that is authorizing this operation
    pub agent_id: String,
    
    /// The operation being authorized
    pub operation: String,
    
    /// The authorization capability
    pub authorization: Capability<Box<dyn Resource>>,
    
    /// Timestamp when this authorization was created
    pub timestamp: u64,
}

impl Clone for ProtocolAuthorization {
    fn clone(&self) -> Self {
        Self {
            agent_id: self.agent_id.clone(),
            operation: self.operation.clone(),
            authorization: create_dummy_capability(),
            timestamp: self.timestamp,
        }
    }
}

// Helper function to create a dummy capability for clone implementation
fn create_dummy_capability() -> Capability<Box<dyn Resource>> {
    let resource_id = ContentId::from_bytes_unwrap(&[0, 0, 0, 0]);
    let cap = crate::capability::resource::ResourceCapability::new(
        crate::capability::resource::ResourceCapabilityType::Read,
        crate::capability::resource::CapabilityGrants::read_only(),
        crate::identity::IdentityId::new(),
    );
    cap.to_capability::<Box<dyn Resource>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::domain::DomainId;
    use causality_types::ContentId;
    use crate::resource::types::{InMemoryResourceTypeRegistry, ResourceSchema, ResourceTypeDefinition};
    use std::collections::HashMap;
    // Use async_trait for the test
    use async_trait::async_trait;
    use crate::effect::context::{BasicEffectContext, EffectContextBuilder};
    use crate::effect::types::Right;
    use crate::effect::EffectId;
    
    // Mock domain resource adapter for testing
    #[derive(Debug)]
    struct MockDomainResourceAdapter {
        domain_id: DomainId,
        supported_types: Vec<ResourceTypeId>,
        resources: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }
    
    impl MockDomainResourceAdapter {
        fn new(domain_id: DomainId) -> Self {
            Self {
                domain_id,
                supported_types: Vec::new(),
                resources: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        fn with_supported_type(mut self, resource_type: ResourceTypeId) -> Self {
            self.supported_types.push(resource_type);
            self
        }
        
        fn with_resource(self, resource_id: &CrossDomainResourceId, data: Vec<u8>) -> Self {
            if let Ok(mut resources) = self.resources.write() {
                resources.insert(resource_id.content_id.to_string(), data);
            }
            self
        }
    }
    
    #[async_trait]
    impl DomainResourceAdapter for MockDomainResourceAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn supports_resource_type(&self, resource_type: &ResourceTypeId) -> bool {
            self.supported_types.contains(resource_type)
        }
        
        async fn store_resource(
            &self,
            resource_id: &CrossDomainResourceId,
            data: &[u8],
            _metadata: &HashMap<String, String>,
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<()> {
            if let Ok(mut resources) = self.resources.write() {
                resources.insert(resource_id.content_id.to_string(), data.to_vec());
                Ok(())
            } else {
                Err(CrossDomainProtocolError::InternalError("Failed to acquire write lock".to_string()))
            }
        }
        
        async fn retrieve_resource(
            &self,
            resource_id: &CrossDomainResourceId,
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<Vec<u8>> {
            self.resources.read().unwrap().get(&resource_id.content_id.to_string())
                .cloned()
                .ok_or_else(|| CrossDomainProtocolError::ResourceNotFound(
                    format!("Resource {} not found in domain {}", resource_id.content_id, self.domain_id)
                ))
        }
        
        async fn has_resource(
            &self,
            resource_id: &CrossDomainResourceId,
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<bool> {
            Ok(self.resources.read().unwrap().contains_key(&resource_id.content_id.to_string()))
        }
        
        async fn verify_resource(
            &self,
            resource_id: &CrossDomainResourceId,
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<VerificationResult> {
            if self.resources.read().unwrap().contains_key(&resource_id.content_id.to_string()) {
                Ok(VerificationResult::Valid)
            } else {
                Ok(VerificationResult::Invalid("Resource not found".to_string()))
            }
        }
        
        async fn transfer_resource(
            &self,
            operation: &ResourceTransferOperation,
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<()> {
            // In a real implementation, this would lock or remove the resource
            // For testing, just ensure the resource exists
            if !self.resources.read().unwrap().contains_key(&operation.resource_id.content_id.to_string()) {
                return Err(CrossDomainProtocolError::ResourceNotFound(
                    format!("Resource {} not found in domain {}", 
                        operation.resource_id.content_id, 
                        self.domain_id
                    )
                ));
            }
            Ok(())
        }
        
        async fn receive_resource(
            &self,
            operation: &ResourceTransferOperation,
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<()> {
            if let Ok(mut resources) = self.resources.write() {
                if let Some(data) = &operation.resource_data {
                    resources.insert(operation.resource_id.content_id.to_string(), data.clone());
                } else {
                    resources.insert(operation.resource_id.content_id.to_string(), vec![]);
                }
                Ok(())
            } else {
                Err(CrossDomainProtocolError::InternalError("Failed to acquire write lock".to_string()))
            }
        }
        
        async fn update_projection(
            &self,
            reference: &ResourceReference,
            data: &[u8],
            _context: &dyn EffectContext,
        ) -> CrossDomainProtocolResult<()> {
            if let Ok(mut resources) = self.resources.write() {
                resources.insert(reference.id.content_id.to_string(), data.to_vec());
                Ok(())
            } else {
                Err(CrossDomainProtocolError::InternalError("Failed to acquire write lock".to_string()))
            }
        }
    }
    
    // Helper function to create a test context
    fn create_test_context() -> BasicEffectContext {
        let effect_id = EffectId::new();
        EffectContextBuilder::new(effect_id).build()
    }
    
    // Helper function to create a test capability
    fn create_test_capability() -> Capability<Box<dyn Resource>> {
        let resource_id = ContentId::from_bytes_unwrap(&[1, 2, 3, 4]);
        // Create a resource capability
        let cap = ResourceCapability::new(
            ResourceCapabilityType::Read,
            CapabilityGrants::read_only(),
            IdentityId::new()
        );
        cap.to_capability::<Box<dyn Resource>>()
    }
    
    #[tokio::test]
    async fn test_cross_domain_protocol() {
        // Create resource type registry
        let registry = Arc::new(InMemoryResourceTypeRegistry::new());
        
        // Create protocol
        let mut protocol = BasicCrossDomainResourceProtocol::new(registry.clone());
        
        // Create domains
        let domain1 = DomainId::new("domain1");
        let domain2 = DomainId::new("domain2");
        
        // Create resource type
        let resource_type = ResourceTypeId::new("test_resource");
        
        // Create domain adapters
        let adapter1 = Arc::new(
            MockDomainResourceAdapter::new(domain1.clone())
                .with_supported_type(resource_type.clone())
        );
        
        let adapter2 = Arc::new(
            MockDomainResourceAdapter::new(domain2.clone())
                .with_supported_type(resource_type.clone())
        );
        
        // Register adapters
        protocol.register_adapter(adapter1.clone());
        protocol.register_adapter(adapter2.clone());
        
        // Create resource
        let resource_id = CrossDomainResourceId::new(
            ContentId::from_bytes_unwrap(&[1, 2, 3, 4]),
            domain1.clone(),
            resource_type.clone()
        );
        
        // Create effect context - use a context capability that's different from resource capability
        let effect_id = EffectId::new();
        let context = EffectContextBuilder::new(effect_id)
            .with_capability(crate::effect::context::Capability::new(resource_id.content_id.clone(), Right::Read))
            .build();
        
        // Create a reference (should fail because resource doesn't exist)
        let result = protocol.create_reference(
            resource_id.clone(),
            domain2.clone(),
            ResourceProjectionType::Shadow,
            VerificationLevel::Hash,
            &context
        ).await;
        
        assert!(result.is_err());
        
        // Add resource to domain1
        let adapter1 = Arc::new(
            MockDomainResourceAdapter::new(domain1.clone())
                .with_supported_type(resource_type.clone())
                .with_resource(&resource_id, vec![1, 2, 3, 4])
        );
        
        protocol.register_adapter(adapter1.clone());
        
        // Create a reference (should succeed now)
        let reference = protocol.create_reference(
            resource_id.clone(),
            domain2.clone(),
            ResourceProjectionType::Shadow,
            VerificationLevel::Hash,
            &context
        ).await.unwrap();
        
        // Verify the reference
        let verification = protocol.verify_reference(&reference, &context).await.unwrap();
        assert!(matches!(verification, VerificationResult::Valid));
        
        // Resolve the reference
        let data = protocol.resolve_reference(&reference, &context).await.unwrap();
        assert_eq!(data, vec![1, 2, 3, 4]);
        
        // Create a transfer operation with the proper resource capability
        let capability = create_test_capability();
        let operation = ResourceTransferOperation::new(
            resource_id.clone(),
            domain1.clone(),
            domain2.clone(),
            ResourceProjectionType::Transferred,
            VerificationLevel::Hash,
            capability
        );
        
        // Execute the transfer
        let reference = protocol.transfer_resource(operation.clone(), &context).await.unwrap();
        
        // Check transfer status
        let status = protocol.get_transfer_status(&operation.id, &context).await.unwrap();
        assert!(matches!(status, TransferStatus::Complete));
        
        // Synchronize the reference
        let updated_reference = protocol.synchronize_reference(&reference, &context).await.unwrap();
        assert_eq!(updated_reference.id, reference.id);
    }
} 