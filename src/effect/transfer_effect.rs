use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::types::DomainId;
use crate::resource::{ResourceId, Right};
use crate::resource::api::{ResourceAPI, ResourceApiError, ResourceApiResult};
use crate::effect::ProgramAccount;
use crate::effect::{
    Effect, EffectResult, EffectError, EffectOutcome, EffectId, ProgramAccountEffect
};
use crate::effect::boundary::{EffectContext, ExecutionBoundary};

#[cfg(feature = "domain")]
use crate::domain::{BoundaryParameters};

use crate::log::fact_snapshot::{FactDependency, FactDependencyType, FactId, FactSnapshot};
use crate::log::FactEntry;

/// Parameters for the TransferEffect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferParams {
    /// The source resource to transfer from
    pub source_resource_id: ResourceId,
    
    /// The destination resource to transfer to
    pub destination_resource_id: ResourceId,
    
    /// The amount to transfer (for fungible resources)
    pub amount: Option<u64>,
    
    /// Additional parameters for the transfer
    pub additional_params: HashMap<String, String>,
}

/// The TransferEffect handles transferring resources between accounts
#[derive(Debug, Clone)]
pub struct TransferEffect {
    /// Unique identifier for this effect
    id: EffectId,
    /// Resource API for interacting with resources
    resource_api: Arc<dyn ResourceAPI>,
    /// Parameters for the transfer operation
    params: TransferParams,
    /// Data associated with this effect
    data: HashMap<String, String>,
    /// Fact snapshot for this effect
    fact_snapshot: Option<FactSnapshot>,
    /// Fact dependencies for this effect
    fact_dependencies: Vec<FactDependency>,
}

impl TransferEffect {
    /// Create a new transfer effect
    pub fn new(
        resource_api: Arc<dyn ResourceAPI>,
        params: TransferParams,
    ) -> Self {
        TransferEffect {
            id: EffectId::new_unique(),
            resource_api,
            params,
            data: HashMap::new(),
            fact_snapshot: None,
            fact_dependencies: Vec::new(),
        }
    }

    /// Factory method to create a new transfer effect
    pub fn create_transfer(resource_api: Arc<dyn ResourceAPI>, params: TransferParams) -> Arc<dyn Effect> {
        Arc::new(Self::new(resource_api, params))
    }

    /// Execute the transfer operation
    async fn execute_transfer(&self, ctx: &EffectContext) -> EffectResult<EffectOutcome> {
        // Find the source capability
        let source_capability = ctx.capabilities.iter()
            .find(|cap| {
                let cap_obj = cap.capability();
                let resource_matches = cap_obj.resource_id() == self.params.source_resource_id.to_string();
                let has_right = cap_obj.has_right(&Right::Transfer);
                resource_matches && has_right
            })
            .ok_or_else(|| EffectError::CapabilityError(
                format!("Missing transfer capability for source resource {}", self.params.source_resource_id)
            ))?;
        
        // Find the destination capability
        let dest_capability = ctx.capabilities.iter()
            .find(|cap| {
                let cap_obj = cap.capability();
                let resource_matches = cap_obj.resource_id() == self.params.destination_resource_id.to_string();
                let has_right = cap_obj.has_right(&Right::Write);
                resource_matches && has_right
            })
            .ok_or_else(|| EffectError::CapabilityError(
                format!("Missing write capability for destination resource {}", self.params.destination_resource_id)
            ))?;
        
        // Get the source resource
        let source_resource = self.resource_api.get_resource_mut(
            source_capability,
            &self.params.source_resource_id,
        ).await.map_err(|e| match e {
            ResourceApiError::NotFound(_) => EffectError::ResourceError(
                format!("Source resource not found: {}", self.params.source_resource_id)
            ),
            ResourceApiError::AccessDenied(_) => EffectError::AuthorizationFailed(
                format!("Access denied to source resource: {}", self.params.source_resource_id)
            ),
            _ => EffectError::ExecutionError(format!("Failed to get source resource: {}", e)),
        })?;
        
        // Get the destination resource
        let dest_resource = self.resource_api.get_resource_mut(
            dest_capability,
            &self.params.destination_resource_id,
        ).await.map_err(|e| match e {
            ResourceApiError::NotFound(_) => EffectError::ResourceError(
                format!("Destination resource not found: {}", self.params.destination_resource_id)
            ),
            ResourceApiError::AccessDenied(_) => EffectError::AuthorizationFailed(
                format!("Access denied to destination resource: {}", self.params.destination_resource_id)
            ),
            _ => EffectError::ExecutionError(format!("Failed to get destination resource: {}", e)),
        })?;
        
        // Validate transfer constraints
        if source_resource.is_locked() {
            return Err(EffectError::ResourceError(
                format!("Source resource is locked: {}", self.params.source_resource_id)
            ));
        }
        
        if dest_resource.is_locked() {
            return Err(EffectError::ResourceError(
                format!("Destination resource is locked: {}", self.params.destination_resource_id)
            ));
        }
        
        let source_data = source_resource.data(source_capability).await.map_err(|e| 
            EffectError::ResourceError(format!("Failed to read source data: {}", e))
        )?;
        let dest_data = dest_resource.data(dest_capability).await.map_err(|e|
            EffectError::ResourceError(format!("Failed to read destination data: {}", e))
        )?;
        
        if source_data.is_empty() {
            return Err(EffectError::ResourceError(
                format!("Source resource is empty: {}", self.params.source_resource_id)
            ));
        }
        
        // Handle specific transfer logic depending on resource types
        match self.params.amount {
            Some(amount) if amount > 0 => {
                // Fungible asset transfer
                let source_amount = source_resource.get_amount()
                    .ok_or_else(|| EffectError::ResourceError(
                        format!("Source resource does not have an amount: {}", self.params.source_resource_id)
                    ))?;
                
                if source_amount < amount {
                    return Err(EffectError::ResourceError(
                        format!("Insufficient amount in source resource: {}", self.params.source_resource_id)
                    ));
                }
                
                let dest_amount = dest_resource.get_amount().unwrap_or(0);
                
                // Create updated resource data
                let mut new_source_data = source_data.clone();
                let mut new_dest_data = dest_data.clone();
                
                // Update the amounts
                source_resource.set_amount(source_amount - amount);
                dest_resource.set_amount(dest_amount + amount);
                
                // Perform the updates
                self.resource_api.update_resource(
                    source_capability,
                    &self.params.source_resource_id,
                    Some(new_source_data),
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to update source resource: {}", e)
                ))?;
                
                self.resource_api.update_resource(
                    dest_capability,
                    &self.params.destination_resource_id,
                    Some(new_dest_data),
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to update destination resource: {}", e)
                ))?;
            },
            _ => {
                // Non-fungible transfer (whole resource)
                // Update the destination resource with source data
                self.resource_api.update_resource(
                    dest_capability,
                    &self.params.destination_resource_id,
                    Some(source_data.clone()),
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to update destination resource: {}", e)
                ))?;
                
                // Clear the source resource (transfer complete)
                self.resource_api.update_resource(
                    source_capability,
                    &self.params.source_resource_id,
                    Some(vec![]), // Empty data
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to update source resource: {}", e)
                ))?;
            }
        }
        
        Ok(EffectOutcome {
            id: EffectId::new_unique(),
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: Some(ctx.execution_id),
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}

#[async_trait]
impl Effect for TransferEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }

    fn name(&self) -> &str {
        "transfer"
    }

    fn display_name(&self) -> String {
        "Transfer Resources".to_string()
    }

    fn description(&self) -> String {
        "Transfers resources between accounts".to_string()
    }

    fn execute(&self, ctx: &EffectContext) -> EffectResult<EffectOutcome> {
        // Return a placeholder outcome in synchronous context
        Ok(EffectOutcome {
            id: EffectId::new_unique(),
            success: false,
            data: HashMap::new(),
            error: Some("Transfer effect must be executed asynchronously".to_string()),
            execution_id: Some(ctx.execution_id),
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        })
    }

    async fn execute_async(&self, ctx: &EffectContext) -> EffectResult<EffectOutcome> {
        self.execute_transfer(ctx).await
    }

    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        matches!(boundary, ExecutionBoundary::CrossSystem)
    }

    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::CrossSystem
    }

    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("from".to_string(), self.params.source_resource_id.to_string());
        params.insert("to".to_string(), self.params.destination_resource_id.to_string());
        if let Some(amount) = self.params.amount {
            params.insert("amount".to_string(), amount.to_string());
        }
        // Add other parameters from additional_params
        for (key, value) in &self.params.additional_params {
            params.insert(key.clone(), value.clone());
        }
        params
    }

    fn fact_dependencies(&self) -> Vec<FactDependency> {
        // Return the fact dependencies
        self.fact_dependencies.clone()
    }

    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        // Return the fact snapshot if one exists
        self.fact_snapshot.clone()
    }

    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // No fact dependencies to validate in this implementation
        Ok(())
    }
}

impl ProgramAccountEffect for TransferEffect {
    fn applicable_account_types(&self) -> Vec<&'static str> {
        vec!["asset"]
    }

    fn can_apply_to(&self, account: &dyn ProgramAccount) -> bool {
        account.account_type() == "asset"
    }
}

/// Implementation of a transfer effect
#[derive(Debug)]
pub struct TransferEffectImpl {
    // ... existing fields ...
}

/// Factory function to create a new transfer effect
pub fn create_transfer_effect(
    // ... existing parameters ...
) -> Arc<dyn Effect> {
    // ... existing implementation ...
}

// Update any other functions that return TransferEffect to specify Output type 