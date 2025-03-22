use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use async_trait::async_trait;
use uuid::Uuid;

use crate::address::Address;
use crate::resource::{
    ResourceId, ResourceCapability, CapabilityId, CapabilityRef, CapabilityRepository,
    Right, Restrictions, CapabilityError, CapabilityResult,
    ResourceAPI, ResourceReader, ResourceWriter, ResourceMetadata, ResourceState,
    ResourceQuery, ResourceUpdateOptions, ResourceApiError, ResourceApiResult,
    MemoryResourceWriter,
};

/// A memory-backed implementation of ResourceAPI for testing
pub struct MemoryResourceAPI {
    /// Repository of capabilities
    capability_repo: Arc<Mutex<CapabilityRepository>>,
    
    /// Map of resource IDs to resources
    resources: Arc<RwLock<HashMap<ResourceId, MemoryResourceWriter>>>,
    
    /// Root capability ID with admin rights to all resources
    root_capability_id: CapabilityId,
}

impl MemoryResourceAPI {
    /// Create a new memory-backed resource API
    pub fn new(admin_address: Address) -> Self {
        let mut capability_repo = CapabilityRepository::new();
        
        // Create a root capability with all rights
        let root_capability = ResourceCapability::new(
            "*", // Special resource ID meaning "all resources"
            "root",
            admin_address.clone(),
            admin_address.clone(),
            vec![
                Right::Read,
                Right::Write,
                Right::Delete,
                Right::Transfer,
                Right::Delegate,
                Right::Custom("Admin".into()),
            ],
        );
        
        let root_capability_ref = capability_repo.register(root_capability);
        
        Self {
            capability_repo: Arc::new(Mutex::new(capability_repo)),
            resources: Arc::new(RwLock::new(HashMap::new())),
            root_capability_id: root_capability_ref.id().clone(),
        }
    }
    
    /// Get the root capability
    pub fn root_capability(&self) -> CapabilityRef {
        let repo = self.capability_repo.lock().unwrap();
        repo.get(&self.root_capability_id).expect("Root capability must exist")
    }
    
    /// Create a resource ID
    fn create_resource_id(&self, resource_type: &str) -> ResourceId {
        // In a real implementation, this would create a deterministic ID based on more factors
        let uuid = Uuid::new_v4();
        ResourceId::from(format!("{}:{}", resource_type, uuid))
    }
    
    /// Check if a capability can access a resource
    async fn check_resource_access(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
        required_right: &Right,
    ) -> ResourceApiResult<()> {
        // Validate the capability
        let repo = self.capability_repo.lock().unwrap();
        let cap_ref = repo.validate(capability.id())?;
        
        // Check if the capability has the required right
        if !cap_ref.capability().has_right(required_right) {
            return Err(ResourceApiError::AccessDenied(format!(
                "Capability does not have the required right: {:?}",
                required_right
            )));
        }
        
        // Check if the capability targets this resource or has a wildcard
        let cap = cap_ref.capability();
        if cap.resource_id() != "*" && cap.resource_id() != resource_id.to_string() {
            return Err(ResourceApiError::AccessDenied(format!(
                "Capability is for resource '{}', not '{}'",
                cap.resource_id(),
                resource_id
            )));
        }
        
        Ok(())
    }
}

#[async_trait]
impl ResourceAPI for MemoryResourceAPI {
    async fn create_resource(
        &self,
        capability: &CapabilityRef,
        resource_type: &str,
        owner: &Address,
        data: Vec<u8>,
        metadata: Option<HashMap<String, String>>,
    ) -> ResourceApiResult<(ResourceId, CapabilityRef)> {
        // Check if the capability allows resource creation
        let cap = capability.capability();
        if !cap.has_right(&Right::Write) && !cap.has_right(&Right::Custom("CreateResource".into())) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have create resource rights".into()
            ));
        }
        
        // Create a resource ID
        let resource_id = self.create_resource_id(resource_type);
        
        // Create metadata
        let now = crate::resource::api::unix_timestamp_now();
        let meta = ResourceMetadata {
            resource_type: resource_type.to_string(),
            owner: owner.clone(),
            created_at: now,
            updated_at: now,
            domain: None,
            content_type: None,
            size: Some(data.len() as u64),
            custom: metadata.unwrap_or_default(),
        };
        
        // Create the resource
        let resource = MemoryResourceWriter {
            id: resource_id.clone(),
            resource_type: resource_type.to_string(),
            owner: owner.clone(),
            data,
            metadata: meta,
            state: ResourceState::Active,
        };
        
        // Store the resource
        {
            let mut resources = self.resources.write().unwrap();
            resources.insert(resource_id.clone(), resource);
        }
        
        // Create a capability for the owner
        let new_capability = ResourceCapability::new(
            resource_id.to_string(),
            resource_type,
            owner.clone(), // Issuer is the owner
            owner.clone(), // Holder is the owner
            vec![
                Right::Read,
                Right::Write,
                Right::Delete,
                Right::Transfer,
                Right::Delegate,
            ],
        );
        
        // Register the capability
        let new_capability_ref = {
            let mut repo = self.capability_repo.lock().unwrap();
            repo.register(new_capability)
        };
        
        Ok((resource_id, new_capability_ref))
    }
    
    async fn get_resource(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
    ) -> ResourceApiResult<Box<dyn ResourceReader + Send + Sync>> {
        // Check access
        self.check_resource_access(capability, resource_id, &Right::Read).await?;
        
        // Get the resource
        let resources = self.resources.read().unwrap();
        let resource = resources.get(resource_id).ok_or_else(|| {
            ResourceApiError::NotFound(format!("Resource not found: {}", resource_id))
        })?;
        
        // Clone the resource and return as a reader
        Ok(Box::new(resource.clone()))
    }
    
    async fn get_resource_mut(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
    ) -> ResourceApiResult<Box<dyn ResourceWriter + Send + Sync>> {
        // Check access
        self.check_resource_access(capability, resource_id, &Right::Write).await?;
        
        // Get the resource
        let mut resources = self.resources.write().unwrap();
        let resource = resources.get_mut(resource_id).ok_or_else(|| {
            ResourceApiError::NotFound(format!("Resource not found: {}", resource_id))
        })?;
        
        // Clone the resource and return as a writer
        // Note: In a real implementation, we would use a more sophisticated
        // locking mechanism to ensure write consistency
        Ok(Box::new(resource.clone()))
    }
    
    async fn find_resources(
        &self,
        capability: &CapabilityRef,
        query: ResourceQuery,
    ) -> ResourceApiResult<Vec<Box<dyn ResourceReader + Send + Sync>>> {
        // Validate the capability
        let repo = self.capability_repo.lock().unwrap();
        repo.validate(capability.id())?;
        
        // Get all resources
        let resources = self.resources.read().unwrap();
        
        // Apply query filters and collect matching resources
        let mut results: Vec<Box<dyn ResourceReader + Send + Sync>> = Vec::new();
        
        for (id, resource) in resources.iter() {
            // Check access for each resource
            let cap = capability.capability();
            if !cap.has_right(&Right::Read) {
                continue;
            }
            
            if cap.resource_id() != "*" && cap.resource_id() != id.to_string() {
                continue;
            }
            
            // Apply type filter
            if let Some(ref type_filter) = query.resource_type {
                if resource.resource_type() != type_filter {
                    continue;
                }
            }
            
            // Apply owner filter
            if let Some(ref owner_filter) = query.owner {
                if &resource.metadata.owner != owner_filter {
                    continue;
                }
            }
            
            // Apply domain filter
            if let Some(ref domain_filter) = query.domain {
                if resource.metadata.domain.as_deref() != Some(domain_filter) {
                    continue;
                }
            }
            
            // Apply metadata filters
            let mut metadata_match = true;
            for (key, value) in &query.metadata {
                if resource.metadata.custom.get(key) != Some(value) {
                    metadata_match = false;
                    break;
                }
            }
            
            if !metadata_match {
                continue;
            }
            
            // Add the resource to results
            results.push(Box::new(resource.clone()));
        }
        
        // Apply sorting if requested
        if let Some(sort_field) = &query.sort_by {
            results.sort_by(|a, b| {
                let a_val = match sort_field.as_str() {
                    "created_at" => a.metadata.created_at.to_string(),
                    "updated_at" => a.metadata.updated_at.to_string(),
                    "resource_type" => a.resource_type().to_string(),
                    _ => a.metadata.custom.get(sort_field)
                        .cloned()
                        .unwrap_or_default(),
                };
                
                let b_val = match sort_field.as_str() {
                    "created_at" => b.metadata.created_at.to_string(),
                    "updated_at" => b.metadata.updated_at.to_string(),
                    "resource_type" => b.resource_type().to_string(),
                    _ => b.metadata.custom.get(sort_field)
                        .cloned()
                        .unwrap_or_default(),
                };
                
                if query.ascending {
                    a_val.cmp(&b_val)
                } else {
                    b_val.cmp(&a_val)
                }
            });
        }
        
        // Apply pagination
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results[offset..].to_vec();
            } else {
                results.clear();
            }
        }
        
        if let Some(limit) = query.limit {
            if limit < results.len() {
                results = results[..limit].to_vec();
            }
        }
        
        Ok(results)
    }
    
    async fn update_resource(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
        data: Option<Vec<u8>>,
        options: Option<ResourceUpdateOptions>,
    ) -> ResourceApiResult<()> {
        // Check access
        self.check_resource_access(capability, resource_id, &Right::Write).await?;
        
        // Get the resource
        let mut resources = self.resources.write().unwrap();
        let resource = resources.get_mut(resource_id).ok_or_else(|| {
            ResourceApiError::NotFound(format!("Resource not found: {}", resource_id))
        })?;
        
        // Update data if provided
        if let Some(new_data) = data {
            resource.data = new_data;
            resource.metadata.updated_at = crate::resource::api::unix_timestamp_now();
            resource.metadata.size = Some(resource.data.len() as u64);
        }
        
        // Apply update options if provided
        if let Some(options) = options {
            if let Some(new_type) = options.resource_type {
                resource.resource_type = new_type;
                resource.metadata.resource_type = resource.resource_type.clone();
            }
            
            if let Some(new_owner) = options.owner {
                resource.owner = new_owner;
                resource.metadata.owner = resource.owner.clone();
            }
            
            if let Some(new_domain) = options.domain {
                resource.metadata.domain = Some(new_domain);
            }
            
            // Update metadata
            if options.override_metadata {
                resource.metadata.custom = options.metadata;
            } else {
                for (key, value) in options.metadata {
                    resource.metadata.custom.insert(key, value);
                }
            }
            
            resource.metadata.updated_at = crate::resource::api::unix_timestamp_now();
        }
        
        Ok(())
    }
    
    async fn delete_resource(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
    ) -> ResourceApiResult<()> {
        // Check access
        self.check_resource_access(capability, resource_id, &Right::Delete).await?;
        
        // Get the resource
        let mut resources = self.resources.write().unwrap();
        let resource = resources.get_mut(resource_id).ok_or_else(|| {
            ResourceApiError::NotFound(format!("Resource not found: {}", resource_id))
        })?;
        
        // Mark as deleted
        resource.state = ResourceState::Deleted;
        
        Ok(())
    }
    
    async fn resource_exists(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
    ) -> ResourceApiResult<bool> {
        // Validate the capability
        let repo = self.capability_repo.lock().unwrap();
        repo.validate(capability.id())?;
        
        // Check if the resource exists
        let resources = self.resources.read().unwrap();
        Ok(resources.contains_key(resource_id))
    }
    
    async fn create_capability(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
        rights: Vec<Right>,
        holder: &Address,
    ) -> ResourceApiResult<CapabilityRef> {
        // Check if the issuer capability has delegate rights
        let cap = capability.capability();
        if !cap.has_right(&Right::Delegate) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have delegate rights".into()
            ));
        }
        
        // Check if the resource exists
        {
            let resources = self.resources.read().unwrap();
            if !resources.contains_key(resource_id) {
                return Err(ResourceApiError::NotFound(format!(
                    "Resource not found: {}", resource_id
                )));
            }
        }
        
        // Get the resource type
        let resource_type = {
            let resources = self.resources.read().unwrap();
            resources.get(resource_id)
                .map(|r| r.resource_type.clone())
                .unwrap()
        };
        
        // Create the new capability
        let new_capability = ResourceCapability::new(
            resource_id.to_string(),
            resource_type,
            cap.holder().clone(), // Issuer is the current holder
            holder.clone(),
            rights,
        );
        
        // Register the capability
        let new_capability_ref = {
            let mut repo = self.capability_repo.lock().unwrap();
            repo.register(new_capability)
        };
        
        Ok(new_capability_ref)
    }
    
    async fn get_capabilities(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
    ) -> ResourceApiResult<Vec<CapabilityRef>> {
        // Validate the capability
        let repo = self.capability_repo.lock().unwrap();
        let cap_ref = repo.validate(capability.id())?;
        
        // Check if the capability has admin rights or is for this resource
        let cap = cap_ref.capability();
        if !cap.has_right(&Right::Custom("Admin".into())) && 
           cap.resource_id() != "*" &&
           cap.resource_id() != resource_id.to_string() {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have access to view capabilities for this resource".into()
            ));
        }
        
        // Get all capabilities for the resource
        let capabilities = repo.get_for_resource(&resource_id.to_string());
        
        Ok(capabilities)
    }
    
    async fn revoke_capability(
        &self,
        capability: &CapabilityRef,
        capability_to_revoke: &CapabilityId,
    ) -> ResourceApiResult<()> {
        // Validate the revoker capability
        let mut repo = self.capability_repo.lock().unwrap();
        let cap_ref = repo.validate(capability.id())?;
        
        // Get the capability to revoke
        let to_revoke = repo.get(capability_to_revoke).ok_or_else(|| {
            ResourceApiError::NotFound(format!("Capability not found: {}", capability_to_revoke))
        })?;
        
        // Check if the revoker has the right to revoke
        let revoker = cap_ref.capability();
        if !revoker.has_right(&Right::Custom("Admin".into())) && 
           revoker.issuer() != to_revoke.capability().issuer() {
            return Err(ResourceApiError::AccessDenied(
                "Only the issuer or an admin can revoke a capability".into()
            ));
        }
        
        // Revoke the capability
        repo.revoke(capability_to_revoke)?;
        
        Ok(())
    }
    
    async fn delegate_capability(
        &self,
        capability: &CapabilityRef,
        resource_id: &ResourceId,
        rights: Vec<Right>,
        new_holder: &Address,
    ) -> ResourceApiResult<CapabilityRef> {
        // Validate the delegator capability
        let mut repo = self.capability_repo.lock().unwrap();
        let cap_ref = repo.validate(capability.id())?;
        
        // Check if the capability has delegate rights
        let delegator = cap_ref.capability();
        if !delegator.has_right(&Right::Delegate) {
            return Err(ResourceApiError::AccessDenied(
                "Capability does not have delegate rights".into()
            ));
        }
        
        // Get the resource type
        let resource_type = {
            let resources = self.resources.read().unwrap();
            resources.get(resource_id)
                .map(|r| r.resource_type.clone())
                .ok_or_else(|| ResourceApiError::NotFound(format!(
                    "Resource not found: {}", resource_id
                )))?
        };
        
        // Create a new capability via delegation
        let new_capability = delegator.delegate(
            new_holder.clone(),
            rights,
            None, // Use the same restrictions
        )?;
        
        // Register the new capability
        let new_capability_ref = repo.register(new_capability);
        
        Ok(new_capability_ref)
    }
    
    async fn compose_capabilities(
        &self,
        capabilities: &[CapabilityRef],
        new_holder: &Address,
    ) -> ResourceApiResult<CapabilityRef> {
        if capabilities.is_empty() {
            return Err(ResourceApiError::InvalidOperation(
                "Cannot compose empty capability list".into()
            ));
        }
        
        // Validate all capabilities
        let mut repo = self.capability_repo.lock().unwrap();
        let mut validated_caps = Vec::new();
        
        for capability in capabilities {
            validated_caps.push(repo.validate(capability.id())?);
        }
        
        // Find the intersection of rights
        let mut common_rights = capabilities[0].capability().rights().to_vec();
        
        for capability in &capabilities[1..] {
            let cap_rights = capability.capability().rights();
            common_rights.retain(|right| cap_rights.contains(right));
        }
        
        if common_rights.is_empty() {
            return Err(ResourceApiError::InvalidOperation(
                "Composed capabilities have no common rights".into()
            ));
        }
        
        // Use the resource ID and type from the first capability
        let first_cap = validated_caps[0].capability();
        let resource_id = first_cap.resource_id();
        let resource_type = first_cap.resource_type();
        
        // Create a new capability with the common rights
        let new_capability = ResourceCapability::new(
            resource_id,
            resource_type,
            first_cap.holder().clone(), // Issuer is the holder of the first capability
            new_holder.clone(),
            common_rights,
        );
        
        // Register the new capability
        let new_capability_ref = repo.register(new_capability);
        
        Ok(new_capability_ref)
    }
} 