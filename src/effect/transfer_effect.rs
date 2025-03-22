use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::address::Address;
use crate::resource::{ResourceId, Right, ResourceAPI, ResourceApiError, ResourceApiResult};
use crate::program_account::{ProgramAccount, AssetProgramAccount};
use crate::effect::{
    Effect, ProgramAccountEffect, EffectResult, EffectError, EffectOutcome, ResourceChange, ResourceChangeType
};
use crate::effect::boundary::{EffectContext, ExecutionBoundary};

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

/// An effect for transferring resources from one account to another
pub struct TransferEffect {
    resource_api: Arc<dyn ResourceAPI>,
}

impl TransferEffect {
    /// Create a new transfer effect
    pub fn new(resource_api: Arc<dyn ResourceAPI>) -> Self {
        Self {
            resource_api,
        }
    }
    
    /// Execute the transfer with specific parameters
    async fn execute_transfer(&self, context: &EffectContext, params: &TransferParams) -> EffectResult<(ResourceId, ResourceId)> {
        // Find the source capability
        let source_capability = context.capabilities.iter()
            .find(|cap| {
                let cap_obj = cap.capability();
                let resource_matches = cap_obj.resource_id() == params.source_resource_id.to_string();
                let has_right = cap_obj.has_right(&Right::Transfer);
                resource_matches && has_right
            })
            .ok_or_else(|| EffectError::CapabilityError(
                format!("Missing transfer capability for source resource {}", params.source_resource_id)
            ))?;
        
        // Find the destination capability
        let dest_capability = context.capabilities.iter()
            .find(|cap| {
                let cap_obj = cap.capability();
                let resource_matches = cap_obj.resource_id() == params.destination_resource_id.to_string();
                let has_right = cap_obj.has_right(&Right::Write);
                resource_matches && has_right
            })
            .ok_or_else(|| EffectError::CapabilityError(
                format!("Missing write capability for destination resource {}", params.destination_resource_id)
            ))?;
        
        // Get the source resource
        let source_resource = self.resource_api.get_resource_mut(
            source_capability,
            &params.source_resource_id,
        ).await.map_err(|e| match e {
            ResourceApiError::NotFound(_) => EffectError::ResourceError(
                format!("Source resource not found: {}", params.source_resource_id)
            ),
            ResourceApiError::AccessDenied(_) => EffectError::AuthorizationFailed(
                format!("Access denied to source resource: {}", params.source_resource_id)
            ),
            _ => EffectError::ExecutionError(format!("Failed to get source resource: {}", e)),
        })?;
        
        // Get the destination resource
        let dest_resource = self.resource_api.get_resource_mut(
            dest_capability,
            &params.destination_resource_id,
        ).await.map_err(|e| match e {
            ResourceApiError::NotFound(_) => EffectError::ResourceError(
                format!("Destination resource not found: {}", params.destination_resource_id)
            ),
            ResourceApiError::AccessDenied(_) => EffectError::AuthorizationFailed(
                format!("Access denied to destination resource: {}", params.destination_resource_id)
            ),
            _ => EffectError::ExecutionError(format!("Failed to get destination resource: {}", e)),
        })?;
        
        // Read the source data
        let source_data = source_resource.data().to_vec();
        
        // Handle specific transfer logic depending on resource types
        match params.amount {
            Some(amount) if amount > 0 => {
                // Fungible asset transfer
                // Parse source data as an amount
                let source_amount = String::from_utf8(source_data.clone())
                    .map_err(|_| EffectError::ExecutionError("Failed to parse source amount".into()))?
                    .parse::<u64>()
                    .map_err(|_| EffectError::ExecutionError("Source is not a valid amount".into()))?;
                
                // Check if source has enough funds
                if source_amount < amount {
                    return Err(EffectError::ResourceError(format!(
                        "Insufficient funds: have {}, need {}",
                        source_amount, amount
                    )));
                }
                
                // Parse destination data as an amount
                let dest_data = dest_resource.data().to_vec();
                let dest_amount = String::from_utf8(dest_data)
                    .map_err(|_| EffectError::ExecutionError("Failed to parse destination amount".into()))?
                    .parse::<u64>()
                    .map_err(|_| EffectError::ExecutionError("Destination is not a valid amount".into()))?;
                
                // Update source resource
                let new_source_amount = source_amount - amount;
                let new_source_data = new_source_amount.to_string().into_bytes();
                
                // Update destination resource
                let new_dest_amount = dest_amount + amount;
                let new_dest_data = new_dest_amount.to_string().into_bytes();
                
                // Perform the updates
                self.resource_api.update_resource(
                    source_capability,
                    &params.source_resource_id,
                    Some(new_source_data),
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to update source resource: {}", e)
                ))?;
                
                self.resource_api.update_resource(
                    dest_capability,
                    &params.destination_resource_id,
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
                    &params.destination_resource_id,
                    Some(source_data.clone()),
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to update destination resource: {}", e)
                ))?;
                
                // Clear the source resource (transfer complete)
                self.resource_api.update_resource(
                    source_capability,
                    &params.source_resource_id,
                    Some(vec![]), // Empty data
                    None,
                ).await.map_err(|e| EffectError::ExecutionError(
                    format!("Failed to clear source resource: {}", e)
                ))?;
            }
        }
        
        Ok((params.source_resource_id.clone(), params.destination_resource_id.clone()))
    }
}

#[async_trait]
impl Effect for TransferEffect {
    fn name(&self) -> &str {
        "transfer"
    }
    
    fn description(&self) -> &str {
        "Transfer resources from one account to another"
    }
    
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
        // Dynamic capabilities are checked during execution
        vec![]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Parse parameters from the context
        let source_id = context.parameters.get("source_resource_id")
            .ok_or_else(|| EffectError::InvalidParameter("Missing source_resource_id".into()))?;
        
        let dest_id = context.parameters.get("destination_resource_id")
            .ok_or_else(|| EffectError::InvalidParameter("Missing destination_resource_id".into()))?;
        
        let amount = context.parameters.get("amount")
            .map(|s| s.parse::<u64>().ok())
            .flatten();
        
        // Create transfer parameters
        let params = TransferParams {
            source_resource_id: ResourceId::from(source_id.clone()),
            destination_resource_id: ResourceId::from(dest_id.clone()),
            amount,
            additional_params: context.parameters.clone(),
        };
        
        // Execute the transfer
        let (source_id, dest_id) = self.execute_transfer(&context, &params).await?;
        
        // Create the outcome
        let source_hash = format!("hash:{}", uuid::Uuid::new_v4());
        let dest_hash = format!("hash:{}", uuid::Uuid::new_v4());
        
        let resource_changes = vec![
            ResourceChange {
                resource_id: source_id,
                change_type: ResourceChangeType::Transferred,
                previous_state_hash: Some(format!("old:{}", uuid::Uuid::new_v4())),
                new_state_hash: source_hash,
            },
            ResourceChange {
                resource_id: dest_id,
                change_type: ResourceChangeType::Updated,
                previous_state_hash: Some(format!("old:{}", uuid::Uuid::new_v4())),
                new_state_hash: dest_hash,
            },
        ];
        
        // Create outcome with results
        let mut result_data = HashMap::new();
        result_data.insert("amount".to_string(), amount.unwrap_or(1).to_string());
        result_data.insert("source_id".to_string(), source_id.to_string());
        result_data.insert("destination_id".to_string(), dest_id.to_string());
        
        let outcome = EffectOutcome {
            execution_id: context.execution_id,
            success: true,
            result: Some(serde_json::to_value(result_data).unwrap_or_default()),
            error: None,
            resource_changes,
            metadata: context.parameters,
        };
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        // This effect must run inside the system since it modifies resources
        boundary == ExecutionBoundary::InsideSystem
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::InsideSystem
    }
}

impl ProgramAccountEffect for TransferEffect {
    fn applicable_account_types(&self) -> Vec<&'static str> {
        vec!["asset", "token", "nft"]
    }
    
    fn can_apply_to(&self, account: &dyn ProgramAccount) -> bool {
        // Check if this is an asset program account
        if let Some(_) = account.as_any().downcast_ref::<AssetProgramAccount>() {
            return true;
        }
        
        // Check if account type is supported
        let account_type = account.account_type();
        self.applicable_account_types().contains(&account_type)
    }
    
    fn display_name(&self) -> &str {
        "Transfer"
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("icon".to_string(), "arrow-right".to_string());
        params.insert("description".to_string(), "Transfer assets to another account".to_string());
        params.insert("color".to_string(), "#4CAF50".to_string());
        params
    }
} 