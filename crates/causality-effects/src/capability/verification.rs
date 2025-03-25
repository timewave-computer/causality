// Capability verification module
// This file implements verification logic for capabilities across domain and effect boundaries

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;

use causality_domain::domain::{DomainId, DomainAdapter};
use causality_domain::capability::DomainCapability;
use causality_types::Result;
use crate::effect::{Effect, EffectContext, EffectId, EffectResult, EffectError};
use crate::capability::unified::{
    UnifiedCapability, EffectCapability, CrossDomainCapability,
    UnifiedCapabilityContext, EffectContextCapabilityExt
};
use crate::capability::conversion::{verify_domain_capabilities};

/// Trait for objects that can verify capabilities across domain and effect boundaries
#[async_trait]
pub trait CapabilityVerifier: Send + Sync {
    /// Verify that the context has the required capabilities for an effect
    async fn verify_capabilities(
        &self,
        effect: &dyn Effect,
        context: &EffectContext
    ) -> EffectResult<()>;
    
    /// Verify that the context has specific domain capabilities
    async fn verify_domain_capabilities(
        &self,
        domain_id: &str,
        capabilities: &[DomainCapability],
        context: &EffectContext
    ) -> EffectResult<()>;
    
    /// Verify that the context has specific effect capabilities
    async fn verify_effect_capabilities(
        &self,
        capabilities: &[EffectCapability],
        context: &EffectContext
    ) -> EffectResult<()>;
    
    /// Verify that the context has specific cross-domain capabilities
    async fn verify_cross_domain_capabilities(
        &self,
        capabilities: &[CrossDomainCapability],
        context: &EffectContext
    ) -> EffectResult<()>;
}

/// Default implementation of the capability verifier
pub struct DefaultCapabilityVerifier {
    /// Required domain capabilities by effect type
    required_domain_capabilities: HashMap<String, Vec<DomainCapability>>,
    
    /// Required effect capabilities by effect type
    required_effect_capabilities: HashMap<String, Vec<EffectCapability>>,
    
    /// Required cross-domain capabilities by effect type
    required_cross_domain_capabilities: HashMap<String, Vec<CrossDomainCapability>>,
}

impl DefaultCapabilityVerifier {
    /// Create a new capability verifier with default capability mappings
    pub fn new() -> Self {
        let mut required_domain_capabilities = HashMap::new();
        let mut required_effect_capabilities = HashMap::new();
        let mut required_cross_domain_capabilities = HashMap::new();
        
        // Set up default domain capability requirements
        required_domain_capabilities.insert(
            "domain_query".to_string(),
            vec![DomainCapability::ReadState]
        );
        
        required_domain_capabilities.insert(
            "domain_transaction".to_string(),
            vec![DomainCapability::SendTransaction]
        );
        
        required_domain_capabilities.insert(
            "evm_contract_call".to_string(),
            vec![DomainCapability::ExecuteContract]
        );
        
        required_domain_capabilities.insert(
            "cosmwasm_execute".to_string(),
            vec![DomainCapability::ExecuteContract]
        );
        
        required_domain_capabilities.insert(
            "zk_prove".to_string(),
            vec![DomainCapability::ZkProve]
        );
        
        // Set up default effect capability requirements
        required_effect_capabilities.insert(
            "resource_create".to_string(),
            vec![EffectCapability::CreateResource]
        );
        
        required_effect_capabilities.insert(
            "resource_read".to_string(),
            vec![EffectCapability::ReadResource]
        );
        
        required_effect_capabilities.insert(
            "resource_update".to_string(),
            vec![EffectCapability::UpdateResource]
        );
        
        required_effect_capabilities.insert(
            "resource_delete".to_string(),
            vec![EffectCapability::DeleteResource]
        );
        
        // Set up default cross-domain capability requirements
        required_cross_domain_capabilities.insert(
            "cross_domain_transfer".to_string(),
            vec![CrossDomainCapability::TransferAssets]
        );
        
        required_cross_domain_capabilities.insert(
            "cross_domain_message".to_string(),
            vec![CrossDomainCapability::SendMessage]
        );
        
        Self {
            required_domain_capabilities,
            required_effect_capabilities,
            required_cross_domain_capabilities,
        }
    }
    
    /// Register required domain capabilities for an effect type
    pub fn register_required_domain_capabilities(
        &mut self,
        effect_type: &str,
        capabilities: Vec<DomainCapability>
    ) {
        self.required_domain_capabilities.insert(effect_type.to_string(), capabilities);
    }
    
    /// Register required effect capabilities for an effect type
    pub fn register_required_effect_capabilities(
        &mut self,
        effect_type: &str,
        capabilities: Vec<EffectCapability>
    ) {
        self.required_effect_capabilities.insert(effect_type.to_string(), capabilities);
    }
    
    /// Register required cross-domain capabilities for an effect type
    pub fn register_required_cross_domain_capabilities(
        &mut self,
        effect_type: &str,
        capabilities: Vec<CrossDomainCapability>
    ) {
        self.required_cross_domain_capabilities.insert(effect_type.to_string(), capabilities);
    }
    
    /// Get required domain capabilities for an effect type
    pub fn get_required_domain_capabilities(&self, effect_type: &str) -> Vec<DomainCapability> {
        self.required_domain_capabilities
            .get(effect_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get required effect capabilities for an effect type
    pub fn get_required_effect_capabilities(&self, effect_type: &str) -> Vec<EffectCapability> {
        self.required_effect_capabilities
            .get(effect_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get required cross-domain capabilities for an effect type
    pub fn get_required_cross_domain_capabilities(&self, effect_type: &str) -> Vec<CrossDomainCapability> {
        self.required_cross_domain_capabilities
            .get(effect_type)
            .cloned()
            .unwrap_or_default()
    }
}

#[async_trait]
impl CapabilityVerifier for DefaultCapabilityVerifier {
    async fn verify_capabilities(
        &self,
        effect: &dyn Effect,
        context: &EffectContext
    ) -> EffectResult<()> {
        let effect_type = effect.effect_type();
        
        // Get domain ID from effect or context
        let domain_id = if let Some(domain_id_method) = effect.as_any().downcast_ref::<dyn DomainIdGetter>() {
            domain_id_method.domain_id().to_string()
        } else {
            context.get_parameter("domain_id")
                .unwrap_or("default")
                .to_string()
        };
        
        // Verify domain capabilities
        let required_domain_caps = self.get_required_domain_capabilities(effect_type);
        if !required_domain_caps.is_empty() {
            self.verify_domain_capabilities(&domain_id, &required_domain_caps, context).await?;
        }
        
        // Verify effect capabilities
        let required_effect_caps = self.get_required_effect_capabilities(effect_type);
        if !required_effect_caps.is_empty() {
            self.verify_effect_capabilities(&required_effect_caps, context).await?;
        }
        
        // Verify cross-domain capabilities
        let required_cross_caps = self.get_required_cross_domain_capabilities(effect_type);
        if !required_cross_caps.is_empty() {
            self.verify_cross_domain_capabilities(&required_cross_caps, context).await?;
        }
        
        Ok(())
    }
    
    async fn verify_domain_capabilities(
        &self,
        domain_id: &str,
        capabilities: &[DomainCapability],
        context: &EffectContext
    ) -> EffectResult<()> {
        verify_domain_capabilities(context, domain_id, capabilities)
    }
    
    async fn verify_effect_capabilities(
        &self,
        capabilities: &[EffectCapability],
        context: &EffectContext
    ) -> EffectResult<()> {
        let unified_context = context.to_unified_capability_context();
        
        for capability in capabilities {
            if !unified_context.has_effect_capability(capability) {
                return Err(EffectError::CapabilityError(
                    format!("Missing effect capability: {:?}", capability)
                ));
            }
        }
        
        Ok(())
    }
    
    async fn verify_cross_domain_capabilities(
        &self,
        capabilities: &[CrossDomainCapability],
        context: &EffectContext
    ) -> EffectResult<()> {
        let unified_context = context.to_unified_capability_context();
        
        for capability in capabilities {
            if !unified_context.has_cross_domain_capability(capability) {
                return Err(EffectError::CapabilityError(
                    format!("Missing cross-domain capability: {:?}", capability)
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for DefaultCapabilityVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for effects that can provide their domain ID
pub trait DomainIdGetter {
    /// Get the domain ID for this effect
    fn domain_id(&self) -> &str;
}

/// Extension for effects to report required capabilities
pub trait CapabilityRequirements {
    /// Get required domain capabilities
    fn required_domain_capabilities(&self) -> Vec<(String, DomainCapability)> {
        Vec::new()
    }
    
    /// Get required effect capabilities
    fn required_effect_capabilities(&self) -> Vec<EffectCapability> {
        Vec::new()
    }
    
    /// Get required cross-domain capabilities
    fn required_cross_domain_capabilities(&self) -> Vec<CrossDomainCapability> {
        Vec::new()
    }
} 