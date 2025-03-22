// Resource Logic Trait Hierarchy
//
// This module implements the ResourceLogic trait hierarchy with validation methods
// as described in ADR-022, providing type-safe operations for different resource types.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::resource::{ResourceId, ResourceRegister, FungibilityDomain, Quantity};
use crate::tel::types::Metadata;
use crate::error::{Error, Result};
use crate::address::Address;

/// Core trait for resource logic implementations
#[async_trait]
pub trait ResourceLogic: Send + Sync {
    /// Get the type name of this resource logic
    fn type_name(&self) -> &'static str;
    
    /// Validate that the resource is well-formed
    async fn validate(&self, resource: &ResourceRegister) -> Result<()>;
    
    /// Check if a resource can be transferred
    async fn can_transfer(&self, resource: &ResourceRegister, quantity: Option<Quantity>) -> Result<bool>;
    
    /// Check if resources can be merged
    async fn can_merge(&self, resource_a: &ResourceRegister, resource_b: &ResourceRegister) -> Result<bool>;
    
    /// Check if a resource can be split
    async fn can_split(&self, resource: &ResourceRegister, quantity: Quantity) -> Result<bool>;
    
    /// Get default metadata for this resource type
    fn default_metadata(&self) -> Metadata;
    
    /// Create a copy of this resource logic
    fn clone_logic(&self) -> Box<dyn ResourceLogic>;
    
    /// Check if the resource logic supports a specific operation
    fn supports_operation(&self, operation: &str) -> bool;
}

/// Resource logic for fungible resources
#[derive(Clone, Debug)]
pub struct FungibleResourceLogic {
    /// Minimum quantity allowed
    pub min_quantity: Quantity,
    
    /// Maximum quantity allowed
    pub max_quantity: Quantity,
    
    /// Supported operations
    pub supported_operations: HashSet<String>,
    
    /// Domains that can interact with this resource type
    pub compatible_domains: HashSet<String>,
}

impl Default for FungibleResourceLogic {
    fn default() -> Self {
        let mut supported_ops = HashSet::new();
        supported_ops.insert("transfer".to_string());
        supported_ops.insert("merge".to_string());
        supported_ops.insert("split".to_string());
        
        Self {
            min_quantity: Quantity(0),
            max_quantity: Quantity(u128::MAX),
            supported_operations: supported_ops,
            compatible_domains: HashSet::new(),
        }
    }
}

#[async_trait]
impl ResourceLogic for FungibleResourceLogic {
    fn type_name(&self) -> &'static str {
        "fungible"
    }
    
    async fn validate(&self, resource: &ResourceRegister) -> Result<()> {
        // Check quantity is within bounds
        if resource.quantity < self.min_quantity {
            return Err(Error::ValidationError(
                format!("Quantity {} is less than minimum {}", resource.quantity.0, self.min_quantity.0)
            ));
        }
        
        if resource.quantity > self.max_quantity {
            return Err(Error::ValidationError(
                format!("Quantity {} exceeds maximum {}", resource.quantity.0, self.max_quantity.0)
            ));
        }
        
        // Fungible resources must have a fungibility domain
        if resource.fungibility_domain.0.is_empty() {
            return Err(Error::ValidationError(
                "Fungible resource must have a non-empty fungibility domain".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn can_transfer(&self, resource: &ResourceRegister, quantity: Option<Quantity>) -> Result<bool> {
        // Check if resource is in a transferable state
        use crate::resource::resource_register::RegisterState;
        if resource.state != RegisterState::Active {
            return Ok(false);
        }
        
        // Check if quantity is valid
        if let Some(qty) = quantity {
            // Can't transfer more than exists or less than minimum
            if qty > resource.quantity || qty < self.min_quantity {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    async fn can_merge(&self, resource_a: &ResourceRegister, resource_b: &ResourceRegister) -> Result<bool> {
        // Check if both resources have the same fungibility domain
        if resource_a.fungibility_domain != resource_b.fungibility_domain {
            return Ok(false);
        }
        
        // Check if both resources are fungible
        if resource_a.resource_logic.type_name() != "fungible" 
           || resource_b.resource_logic.type_name() != "fungible" {
            return Ok(false);
        }
        
        // Check if the merged quantity would exceed maximum
        let total_quantity = Quantity(resource_a.quantity.0.saturating_add(resource_b.quantity.0));
        if total_quantity > self.max_quantity {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    async fn can_split(&self, resource: &ResourceRegister, quantity: Quantity) -> Result<bool> {
        // Check if resource has enough quantity
        if resource.quantity < quantity {
            return Ok(false);
        }
        
        // Check if remaining amount would be valid
        let remaining = Quantity(resource.quantity.0.saturating_sub(quantity.0));
        if remaining < self.min_quantity {
            return Ok(false);
        }
        
        // Check if requested amount is valid
        if quantity < self.min_quantity {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    fn default_metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        metadata.insert("type".to_string(), serde_json::Value::String("fungible".to_string()));
        metadata.insert("divisible".to_string(), serde_json::Value::Bool(true));
        metadata
    }
    
    fn clone_logic(&self) -> Box<dyn ResourceLogic> {
        Box::new(self.clone())
    }
    
    fn supports_operation(&self, operation: &str) -> bool {
        self.supported_operations.contains(operation)
    }
}

/// Resource logic for non-fungible resources
#[derive(Clone, Debug)]
pub struct NonFungibleResourceLogic {
    /// Whether the resource can be transferred
    pub transferable: bool,
    
    /// Whether the resource can be burned
    pub burnable: bool,
    
    /// Required metadata fields
    pub required_metadata: HashSet<String>,
    
    /// Supported operations
    pub supported_operations: HashSet<String>,
}

impl Default for NonFungibleResourceLogic {
    fn default() -> Self {
        let mut supported_ops = HashSet::new();
        supported_ops.insert("transfer".to_string());
        supported_ops.insert("inspect".to_string());
        
        let mut required_metadata = HashSet::new();
        required_metadata.insert("id".to_string());
        
        Self {
            transferable: true,
            burnable: false,
            required_metadata,
            supported_operations: supported_ops,
        }
    }
}

#[async_trait]
impl ResourceLogic for NonFungibleResourceLogic {
    fn type_name(&self) -> &'static str {
        "non_fungible"
    }
    
    async fn validate(&self, resource: &ResourceRegister) -> Result<()> {
        // Non-fungible resources must have quantity of 1
        if resource.quantity != Quantity(1) {
            return Err(Error::ValidationError(
                format!("Non-fungible resource must have quantity of 1, got {}", resource.quantity.0)
            ));
        }
        
        // Check that all required metadata fields are present
        for field in &self.required_metadata {
            if !resource.metadata.contains_key(field) {
                return Err(Error::ValidationError(
                    format!("Missing required metadata field: {}", field)
                ));
            }
        }
        
        Ok(())
    }
    
    async fn can_transfer(&self, resource: &ResourceRegister, _quantity: Option<Quantity>) -> Result<bool> {
        // Check if resource is transferable
        if !self.transferable {
            return Ok(false);
        }
        
        // Check if resource is in a transferable state
        use crate::resource::resource_register::RegisterState;
        if resource.state != RegisterState::Active {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    async fn can_merge(&self, _resource_a: &ResourceRegister, _resource_b: &ResourceRegister) -> Result<bool> {
        // Non-fungible resources cannot be merged
        Ok(false)
    }
    
    async fn can_split(&self, _resource: &ResourceRegister, _quantity: Quantity) -> Result<bool> {
        // Non-fungible resources cannot be split
        Ok(false)
    }
    
    fn default_metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        metadata.insert("type".to_string(), serde_json::Value::String("non_fungible".to_string()));
        metadata.insert("divisible".to_string(), serde_json::Value::Bool(false));
        metadata
    }
    
    fn clone_logic(&self) -> Box<dyn ResourceLogic> {
        Box::new(self.clone())
    }
    
    fn supports_operation(&self, operation: &str) -> bool {
        self.supported_operations.contains(operation)
    }
}

/// Resource logic for capability resources
#[derive(Clone, Debug)]
pub struct CapabilityResourceLogic {
    /// Whether the resource can be delegated
    pub delegatable: bool,
    
    /// Whether the resource can be transferred
    pub transferable: bool,
    
    /// Maximum delegation depth
    pub max_delegation_depth: Option<u32>,
    
    /// Supported operations
    pub supported_operations: HashSet<String>,
}

impl Default for CapabilityResourceLogic {
    fn default() -> Self {
        let mut supported_ops = HashSet::new();
        supported_ops.insert("delegate".to_string());
        supported_ops.insert("revoke".to_string());
        supported_ops.insert("validate".to_string());
        
        Self {
            delegatable: true,
            transferable: false,
            max_delegation_depth: Some(3),
            supported_operations: supported_ops,
        }
    }
}

#[async_trait]
impl ResourceLogic for CapabilityResourceLogic {
    fn type_name(&self) -> &'static str {
        "capability"
    }
    
    async fn validate(&self, resource: &ResourceRegister) -> Result<()> {
        // Capability resources must have quantity of 1
        if resource.quantity != Quantity(1) {
            return Err(Error::ValidationError(
                format!("Capability resource must have quantity of 1, got {}", resource.quantity.0)
            ));
        }
        
        // Must have rights metadata
        if !resource.metadata.contains_key("rights") {
            return Err(Error::ValidationError(
                "Capability resource must have 'rights' metadata".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn can_transfer(&self, resource: &ResourceRegister, _quantity: Option<Quantity>) -> Result<bool> {
        // Check if capability is transferable
        if !self.transferable {
            return Ok(false);
        }
        
        // Check if resource is in a transferable state
        use crate::resource::resource_register::RegisterState;
        if resource.state != RegisterState::Active {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    async fn can_merge(&self, _resource_a: &ResourceRegister, _resource_b: &ResourceRegister) -> Result<bool> {
        // Capability resources cannot be merged
        Ok(false)
    }
    
    async fn can_split(&self, _resource: &ResourceRegister, _quantity: Quantity) -> Result<bool> {
        // Capability resources cannot be split
        Ok(false)
    }
    
    fn default_metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        metadata.insert("type".to_string(), serde_json::Value::String("capability".to_string()));
        metadata.insert("delegatable".to_string(), serde_json::Value::Bool(self.delegatable));
        metadata
    }
    
    fn clone_logic(&self) -> Box<dyn ResourceLogic> {
        Box::new(self.clone())
    }
    
    fn supports_operation(&self, operation: &str) -> bool {
        self.supported_operations.contains(operation)
    }
}

/// Resource logic for data resources
#[derive(Clone, Debug)]
pub struct DataResourceLogic {
    /// Whether the data can be updated
    pub updatable: bool,
    
    /// Whether the data can be deleted
    pub deletable: bool,
    
    /// Maximum data size in bytes
    pub max_size: Option<usize>,
    
    /// Supported operations
    pub supported_operations: HashSet<String>,
}

impl Default for DataResourceLogic {
    fn default() -> Self {
        let mut supported_ops = HashSet::new();
        supported_ops.insert("read".to_string());
        supported_ops.insert("update".to_string());
        
        Self {
            updatable: true,
            deletable: true,
            max_size: Some(1024 * 1024), // 1MB
            supported_operations: supported_ops,
        }
    }
}

#[async_trait]
impl ResourceLogic for DataResourceLogic {
    fn type_name(&self) -> &'static str {
        "data"
    }
    
    async fn validate(&self, resource: &ResourceRegister) -> Result<()> {
        // Data resources must have data metadata
        if !resource.metadata.contains_key("data") {
            return Err(Error::ValidationError(
                "Data resource must have 'data' metadata".to_string()
            ));
        }
        
        // Check data size if maximum is specified
        if let Some(max_size) = self.max_size {
            if let Some(data) = resource.metadata.get("data") {
                let data_str = serde_json::to_string(data)
                    .map_err(|e| Error::ValidationError(format!("Failed to serialize data: {}", e)))?;
                
                if data_str.len() > max_size {
                    return Err(Error::ValidationError(
                        format!("Data size {} exceeds maximum {}", data_str.len(), max_size)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    async fn can_transfer(&self, resource: &ResourceRegister, _quantity: Option<Quantity>) -> Result<bool> {
        // Data resources are not transferable
        Ok(false)
    }
    
    async fn can_merge(&self, _resource_a: &ResourceRegister, _resource_b: &ResourceRegister) -> Result<bool> {
        // Data resources cannot be merged
        Ok(false)
    }
    
    async fn can_split(&self, _resource: &ResourceRegister, _quantity: Quantity) -> Result<bool> {
        // Data resources cannot be split
        Ok(false)
    }
    
    fn default_metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        metadata.insert("type".to_string(), serde_json::Value::String("data".to_string()));
        metadata.insert("updatable".to_string(), serde_json::Value::Bool(self.updatable));
        metadata
    }
    
    fn clone_logic(&self) -> Box<dyn ResourceLogic> {
        Box::new(self.clone())
    }
    
    fn supports_operation(&self, operation: &str) -> bool {
        self.supported_operations.contains(operation)
    }
}

/// A factory for creating resource logic implementations
pub struct ResourceLogicFactory {
    /// Custom logic registrations
    custom_logic: HashMap<String, Box<dyn Fn() -> Box<dyn ResourceLogic> + Send + Sync>>,
}

impl Default for ResourceLogicFactory {
    fn default() -> Self {
        Self {
            custom_logic: HashMap::new(),
        }
    }
}

impl ResourceLogicFactory {
    /// Create a new resource logic factory
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create resource logic for the given type
    pub fn create(&self, logic_type: &str) -> Result<Box<dyn ResourceLogic>> {
        match logic_type {
            "fungible" => Ok(Box::new(FungibleResourceLogic::default())),
            "non_fungible" => Ok(Box::new(NonFungibleResourceLogic::default())),
            "capability" => Ok(Box::new(CapabilityResourceLogic::default())),
            "data" => Ok(Box::new(DataResourceLogic::default())),
            custom => {
                // Check for registered custom logic
                if let Some(factory) = self.custom_logic.get(custom) {
                    Ok(factory())
                } else {
                    Err(Error::NotFound(format!("Resource logic type not found: {}", custom)))
                }
            }
        }
    }
    
    /// Register custom resource logic
    pub fn register_custom<F>(&mut self, type_name: &str, factory: F)
    where
        F: Fn() -> Box<dyn ResourceLogic> + Send + Sync + 'static,
    {
        self.custom_logic.insert(type_name.to_string(), Box::new(factory));
    }
}

/// Validation service for resources
pub struct ResourceValidationService {
    /// Factory for creating resource logic
    logic_factory: ResourceLogicFactory,
}

impl ResourceValidationService {
    /// Create a new resource validation service
    pub fn new(logic_factory: ResourceLogicFactory) -> Self {
        Self {
            logic_factory,
        }
    }
    
    /// Validate a resource
    pub async fn validate(&self, resource: &ResourceRegister) -> Result<()> {
        // Get the logic for this resource type
        let logic = match &resource.resource_logic {
            crate::resource::resource_register::ResourceLogic::Fungible => {
                self.logic_factory.create("fungible")?
            },
            crate::resource::resource_register::ResourceLogic::NonFungible => {
                self.logic_factory.create("non_fungible")?
            },
            crate::resource::resource_register::ResourceLogic::Capability => {
                self.logic_factory.create("capability")?
            },
            crate::resource::resource_register::ResourceLogic::Data => {
                self.logic_factory.create("data")?
            },
            crate::resource::resource_register::ResourceLogic::Custom(name) => {
                self.logic_factory.create(name)?
            },
        };
        
        // Validate using the appropriate logic
        logic.validate(resource).await
    }
    
    /// Check if a transfer operation is valid
    pub async fn validate_transfer(
        &self, 
        resource: &ResourceRegister, 
        quantity: Option<Quantity>
    ) -> Result<bool> {
        // Get the logic for this resource type
        let logic = match &resource.resource_logic {
            crate::resource::resource_register::ResourceLogic::Fungible => {
                self.logic_factory.create("fungible")?
            },
            crate::resource::resource_register::ResourceLogic::NonFungible => {
                self.logic_factory.create("non_fungible")?
            },
            crate::resource::resource_register::ResourceLogic::Capability => {
                self.logic_factory.create("capability")?
            },
            crate::resource::resource_register::ResourceLogic::Data => {
                self.logic_factory.create("data")?
            },
            crate::resource::resource_register::ResourceLogic::Custom(name) => {
                self.logic_factory.create(name)?
            },
        };
        
        // Check if transfer is valid
        logic.can_transfer(resource, quantity).await
    }
    
    /// Check if a merge operation is valid
    pub async fn validate_merge(
        &self,
        resource_a: &ResourceRegister,
        resource_b: &ResourceRegister
    ) -> Result<bool> {
        // Resources must have the same logic type
        if resource_a.resource_logic != resource_b.resource_logic {
            return Ok(false);
        }
        
        // Get the logic for this resource type
        let logic = match &resource_a.resource_logic {
            crate::resource::resource_register::ResourceLogic::Fungible => {
                self.logic_factory.create("fungible")?
            },
            crate::resource::resource_register::ResourceLogic::NonFungible => {
                self.logic_factory.create("non_fungible")?
            },
            crate::resource::resource_register::ResourceLogic::Capability => {
                self.logic_factory.create("capability")?
            },
            crate::resource::resource_register::ResourceLogic::Data => {
                self.logic_factory.create("data")?
            },
            crate::resource::resource_register::ResourceLogic::Custom(name) => {
                self.logic_factory.create(name)?
            },
        };
        
        // Check if merge is valid
        logic.can_merge(resource_a, resource_b).await
    }
    
    /// Check if a split operation is valid
    pub async fn validate_split(
        &self,
        resource: &ResourceRegister,
        quantity: Quantity
    ) -> Result<bool> {
        // Get the logic for this resource type
        let logic = match &resource.resource_logic {
            crate::resource::resource_register::ResourceLogic::Fungible => {
                self.logic_factory.create("fungible")?
            },
            crate::resource::resource_register::ResourceLogic::NonFungible => {
                self.logic_factory.create("non_fungible")?
            },
            crate::resource::resource_register::ResourceLogic::Capability => {
                self.logic_factory.create("capability")?
            },
            crate::resource::resource_register::ResourceLogic::Data => {
                self.logic_factory.create("data")?
            },
            crate::resource::resource_register::ResourceLogic::Custom(name) => {
                self.logic_factory.create(name)?
            },
        };
        
        // Check if split is valid
        logic.can_split(resource, quantity).await
    }
} 