// Effect adapter for accounts
// Original file: src/program_account/effect_adapter.rs

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Debug;
use async_trait::async_trait;
use serde_json::Value;

use causality_types::Address;
use crate::resource::{ResourceId, ResourceAPI, CapabilityRef, Right};
use crate::program_account::{ProgramAccount, ProgramAccountId, ProgramAccountError, ProgramAccountResult};
use crate::effect::{Effect, ProgramAccountEffect, EffectResult, EffectError, EffectOutcome, EffectManager};
use causality_effects::{EffectContext, ExecutionBoundary, ChainBoundary};

/// Interface for program account effect interaction
#[async_trait]
pub trait ProgramAccountEffectAdapter: Send + Sync {
    /// Get available effects for a program account
    async fn get_available_effects(&self, account_id: &ProgramAccountId) -> ProgramAccountResult<Vec<EffectInfo>>;
    
    /// Execute an effect on a program account
    async fn execute_effect(
        &self,
        account_id: &ProgramAccountId,
        effect_name: &str,
        parameters: HashMap<String, String>,
    ) -> ProgramAccountResult<EffectOutcome>;
    
    /// Get capabilities for a program account to use with effects
    async fn get_account_capabilities(&self, account_id: &ProgramAccountId) -> ProgramAccountResult<Vec<CapabilityRef>>;
}

/// Information about an effect available to a program account
#[derive(Debug, Clone)]
pub struct EffectInfo {
    /// The name of the effect
    pub name: String,
    
    /// Display name shown to users
    pub display_name: String,
    
    /// Description of the effect
    pub description: String,
    
    /// UI display parameters
    pub display_parameters: HashMap<String, String>,
    
    /// Required parameters for execution
    pub required_parameters: Vec<EffectParameter>,
    
    /// Execution boundary
    pub boundary: ExecutionBoundary,
}

/// Parameter information for effects
#[derive(Debug, Clone)]
pub struct EffectParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub param_type: EffectParameterType,
    
    /// Display name
    pub display_name: String,
    
    /// Parameter description
    pub description: String,
    
    /// Whether the parameter is required
    pub required: bool,
    
    /// Default value (if any)
    pub default_value: Option<String>,
}

/// Types of effect parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectParameterType {
    /// String parameter
    String,
    
    /// Numeric parameter
    Number,
    
    /// Boolean parameter
    Boolean,
    
    /// Resource ID parameter
    ResourceId,
    
    /// Account ID parameter
    AccountId,
    
    /// Address parameter
    Address,
    
    /// Selection from a list of options
    Selection,
}

/// Implementation of ProgramAccountEffectAdapter
pub struct ProgramAccountEffectAdapterImpl {
    /// Effect manager
    effect_manager: Arc<EffectManager>,
    
    /// Resource API
    resource_api: Arc<dyn ResourceAPI>,
    
    /// Account registry (maps account IDs to accounts)
    account_registry: HashMap<ProgramAccountId, Arc<dyn ProgramAccount>>,
    
    /// Account capabilities (maps account IDs to capabilities)
    account_capabilities: HashMap<ProgramAccountId, Vec<CapabilityRef>>,
}

impl ProgramAccountEffectAdapterImpl {
    /// Create a new program account effect adapter
    pub fn new(effect_manager: Arc<EffectManager>, resource_api: Arc<dyn ResourceAPI>) -> Self {
        Self {
            effect_manager,
            resource_api,
            account_registry: HashMap::new(),
            account_capabilities: HashMap::new(),
        }
    }
    
    /// Register a program account
    pub fn register_account(&mut self, account: Arc<dyn ProgramAccount>) -> ProgramAccountResult<()> {
        let account_id = account.id().clone();
        self.account_registry.insert(account_id, account);
        Ok(())
    }
    
    /// Register capabilities for a program account
    pub fn register_account_capabilities(&mut self, account_id: &ProgramAccountId, capabilities: Vec<CapabilityRef>) -> ProgramAccountResult<()> {
        if !self.account_registry.contains_key(account_id) {
            return Err(ProgramAccountError::NotFound(format!("Account not found: {}", account_id)));
        }
        
        self.account_capabilities.insert(account_id.clone(), capabilities);
        Ok(())
    }
    
    /// Get a program account by ID
    fn get_account(&self, account_id: &ProgramAccountId) -> ProgramAccountResult<Arc<dyn ProgramAccount>> {
        self.account_registry.get(account_id)
            .cloned()
            .ok_or_else(|| ProgramAccountError::NotFound(format!("Account not found: {}", account_id)))
    }
    
    /// Create an effect context for a program account
    async fn create_effect_context(
        &self,
        account_id: &ProgramAccountId,
        parameters: HashMap<String, String>,
    ) -> ProgramAccountResult<EffectContext> {
        let account = self.get_account(account_id)?;
        
        // Get account capabilities
        let capabilities = self.get_account_capabilities(account_id).await?;
        
        // Create the context
        let context = EffectContext::new_inside(account.owner().clone())
            .with_capabilities(capabilities);
        
        // Add parameters to context
        let context_with_params = parameters.iter().fold(
            context,
            |ctx, (key, value)| ctx.with_parameter(key, value)
        );
        
        Ok(context_with_params)
    }
    
    /// Get effect parameters for a program account effect
    fn get_effect_parameters(&self, effect: &Arc<dyn ProgramAccountEffect>) -> Vec<EffectParameter> {
        let mut params = Vec::new();
        
        // Common parameters for all effects
        params.push(EffectParameter {
            name: "account_id".to_string(),
            param_type: EffectParameterType::AccountId,
            display_name: "Account".to_string(),
            description: "The account to apply the effect to".to_string(),
            required: true,
            default_value: None,
        });
        
        // Add specific parameters based on effect type
        if effect.name() == "transfer" {
            params.push(EffectParameter {
                name: "source_resource_id".to_string(),
                param_type: EffectParameterType::ResourceId,
                display_name: "Source".to_string(),
                description: "The source resource to transfer from".to_string(),
                required: true,
                default_value: None,
            });
            
            params.push(EffectParameter {
                name: "destination_resource_id".to_string(),
                param_type: EffectParameterType::ResourceId,
                display_name: "Destination".to_string(),
                description: "The destination resource to transfer to".to_string(),
                required: true,
                default_value: None,
            });
            
            params.push(EffectParameter {
                name: "amount".to_string(),
                param_type: EffectParameterType::Number,
                display_name: "Amount".to_string(),
                description: "The amount to transfer".to_string(),
                required: false,
                default_value: Some("1".to_string()),
            });
        }
        
        params
    }
}

#[async_trait]
impl ProgramAccountEffectAdapter for ProgramAccountEffectAdapterImpl {
    async fn get_available_effects(&self, account_id: &ProgramAccountId) -> ProgramAccountResult<Vec<EffectInfo>> {
        let account = self.get_account(account_id)?;
        
        // Get all program account effects
        let all_effects = self.effect_manager.registry().get_all();
        
        let mut available_effects = Vec::new();
        
        for effect in all_effects {
            // Try to cast to ProgramAccountEffect
            if let Some(pa_effect) = effect.as_any().downcast_ref::<dyn ProgramAccountEffect>() {
                // Check if this effect can be applied to this account
                if pa_effect.can_apply_to(&*account) {
                    // Create effect info
                    let effect_info = EffectInfo {
                        name: pa_effect.name().to_string(),
                        display_name: pa_effect.display_name().to_string(),
                        description: pa_effect.description().to_string(),
                        display_parameters: pa_effect.display_parameters(),
                        required_parameters: self.get_effect_parameters(&effect),
                        boundary: pa_effect.preferred_boundary(),
                    };
                    
                    available_effects.push(effect_info);
                }
            }
        }
        
        Ok(available_effects)
    }
    
    async fn execute_effect(
        &self,
        account_id: &ProgramAccountId,
        effect_name: &str,
        parameters: HashMap<String, String>,
    ) -> ProgramAccountResult<EffectOutcome> {
        // Create effect context
        let mut effect_params = parameters.clone();
        
        // Add account ID to parameters
        effect_params.insert("account_id".to_string(), account_id.to_string());
        
        // Create context with capabilities
        let context = self.create_effect_context(account_id, effect_params).await?;
        
        // Execute the effect
        let outcome = self.effect_manager.execute_effect(effect_name, context).await
            .map_err(|e| ProgramAccountError::EffectError(format!("{}", e)))?;
        
        Ok(outcome)
    }
    
    async fn get_account_capabilities(&self, account_id: &ProgramAccountId) -> ProgramAccountResult<Vec<CapabilityRef>> {
        // Get capabilities from the map
        let capabilities = self.account_capabilities.get(account_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(capabilities)
    }
}

/// Extension trait for Effect to support downcasting
pub trait EffectExt: Effect {
    /// Downcast to Any
    fn as_any(&self) -> &dyn std::any::Any;
} 
