// Capability conversion module for domain and effect systems
// This file implements the conversion logic between domain and effect capabilities

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use causality_domain::domain::{DomainId, DomainType, DomainAdapter};
use causality_domain::capability::{DomainCapability, DomainCapabilityManager};
use causality_types::Result;
use crate::effect::{EffectContext, EffectResult, EffectError};
use crate::capability::unified::{
    UnifiedCapability, EffectCapability, CrossDomainCapability,
    UnifiedCapabilityContext, UnifiedCapabilityManager, EffectContextCapabilityExt
};

/// Converts domain capabilities to effect capabilities
pub fn convert_domain_to_effect_capabilities(
    domain_capabilities: &HashSet<DomainCapability>
) -> HashSet<EffectCapability> {
    let mut effect_capabilities = HashSet::new();
    
    for domain_cap in domain_capabilities {
        if let Some(effect_cap) = map_domain_to_effect_capability(domain_cap) {
            effect_capabilities.insert(effect_cap);
        }
    }
    
    effect_capabilities
}

/// Converts effect capabilities to domain capabilities
pub fn convert_effect_to_domain_capabilities(
    effect_capabilities: &HashSet<EffectCapability>
) -> HashSet<DomainCapability> {
    let mut domain_capabilities = HashSet::new();
    
    for effect_cap in effect_capabilities {
        if let Some(domain_cap) = map_effect_to_domain_capability(effect_cap) {
            domain_capabilities.insert(domain_cap);
        }
    }
    
    domain_capabilities
}

/// Maps a domain capability to an effect capability
pub fn map_domain_to_effect_capability(domain_cap: &DomainCapability) -> Option<EffectCapability> {
    match domain_cap {
        DomainCapability::SendTransaction => Some(EffectCapability::SubmitTransaction),
        DomainCapability::SignTransaction => Some(EffectCapability::SignTransaction),
        DomainCapability::BatchTransactions => None, // No direct mapping
        DomainCapability::DeployContract => None, // No direct mapping
        DomainCapability::ExecuteContract => Some(EffectCapability::ExecuteTEL), // Approximate mapping
        DomainCapability::QueryContract => Some(EffectCapability::ReadResource), // Approximate mapping
        DomainCapability::ReadState => Some(EffectCapability::ReadResource),
        DomainCapability::WriteState => Some(EffectCapability::UpdateResource),
        DomainCapability::VerifySignature => None, // No direct mapping
        DomainCapability::GenerateProof => Some(EffectCapability::GenerateProof),
        DomainCapability::VerifyProof => Some(EffectCapability::VerifyProof),
        DomainCapability::ZkProve => Some(EffectCapability::GenerateProof),
        DomainCapability::ZkVerify => Some(EffectCapability::VerifyProof),
        DomainCapability::Stake => None, // No direct mapping
        DomainCapability::Validate => None, // No direct mapping
        DomainCapability::Vote => None, // No direct mapping
        DomainCapability::ProposeUpgrade => None, // No direct mapping
        DomainCapability::VoteOnProposal => None, // No direct mapping
        DomainCapability::BridgeAssets => None, // Mapped to cross-domain instead
        DomainCapability::VerifyBridgeTransaction => None, // Mapped to cross-domain instead
        DomainCapability::Custom(_) => None, // No direct mapping for custom capabilities
    }
}

/// Maps an effect capability to a domain capability
pub fn map_effect_to_domain_capability(effect_cap: &EffectCapability) -> Option<DomainCapability> {
    match effect_cap {
        EffectCapability::CreateResource => Some(DomainCapability::WriteState),
        EffectCapability::ReadResource => Some(DomainCapability::ReadState),
        EffectCapability::UpdateResource => Some(DomainCapability::WriteState),
        EffectCapability::DeleteResource => Some(DomainCapability::WriteState),
        EffectCapability::SubmitTransaction => Some(DomainCapability::SendTransaction),
        EffectCapability::SignTransaction => Some(DomainCapability::SignTransaction),
        EffectCapability::GenerateProof => Some(DomainCapability::GenerateProof),
        EffectCapability::VerifyProof => Some(DomainCapability::VerifyProof),
        EffectCapability::ExecuteTEL => Some(DomainCapability::ExecuteContract),
        EffectCapability::CompileTEL => None, // No direct mapping
        EffectCapability::AccessRegistry => None, // No direct mapping
        EffectCapability::ModifyRegistry => None, // No direct mapping
        EffectCapability::ImpersonateIdentity => None, // No direct mapping
        EffectCapability::DelegateCapability => None, // No direct mapping
        EffectCapability::Custom(_) => None, // No direct mapping for custom capabilities
    }
}

/// Maps a domain capability to a cross-domain capability
pub fn map_domain_to_cross_domain_capability(domain_cap: &DomainCapability) -> Option<CrossDomainCapability> {
    match domain_cap {
        DomainCapability::BridgeAssets => Some(CrossDomainCapability::TransferAssets),
        DomainCapability::VerifyBridgeTransaction => Some(CrossDomainCapability::VerifyCrossDomainProof),
        _ => None, // Most domain capabilities don't map to cross-domain
    }
}

/// Converts a domain adapter's capabilities to an effect context
pub async fn convert_domain_adapter_to_effect_context(
    adapter: &dyn DomainAdapter,
    identity: String
) -> Result<EffectContext> {
    // Get domain info
    let domain_info = adapter.domain_info().await?;
    let domain_id = adapter.domain_id().clone();
    
    // Get adapter capabilities
    let domain_capabilities: HashSet<DomainCapability> = adapter.capabilities()
        .iter()
        .filter_map(|cap_str| DomainCapability::from_string(cap_str))
        .collect();
    
    // Convert to effect and cross-domain capabilities
    let effect_capabilities = convert_domain_to_effect_capabilities(&domain_capabilities);
    
    // Create an effect context
    let mut context = EffectContext::new();
    context.set_identity(identity);
    
    // Add domain capabilities
    for domain_cap in &domain_capabilities {
        context.add_domain_capability(&domain_id, domain_cap.clone());
    }
    
    // Add effect capabilities
    for effect_cap in &effect_capabilities {
        context.add_effect_capability(effect_cap.clone());
    }
    
    // Add domain info as parameters
    context.set_parameter("domain_id", domain_id.clone());
    context.set_parameter("domain_type", domain_info.domain_type.to_string());
    context.set_parameter("domain_name", domain_info.name.clone());
    
    Ok(context)
}

/// Enhances an effect context with domain adapter capabilities
pub async fn enhance_effect_context_with_domain_capabilities(
    context: &mut EffectContext,
    adapter: &dyn DomainAdapter
) -> Result<()> {
    let domain_id = adapter.domain_id().clone();
    
    // Get adapter capabilities
    let domain_capabilities: HashSet<DomainCapability> = adapter.capabilities()
        .iter()
        .filter_map(|cap_str| DomainCapability::from_string(cap_str))
        .collect();
    
    // Add domain capabilities
    for domain_cap in &domain_capabilities {
        context.add_domain_capability(&domain_id, domain_cap.clone());
    }
    
    // Set domain parameters
    context.set_parameter("domain_id", domain_id.clone());
    let domain_info = adapter.domain_info().await?;
    context.set_parameter("domain_type", domain_info.domain_type.to_string());
    
    Ok(())
}

/// Verifies that an effect context has the required domain capabilities
pub fn verify_domain_capabilities(
    context: &EffectContext,
    domain_id: &str,
    required_capabilities: &[DomainCapability]
) -> EffectResult<()> {
    let unified_context = context.to_unified_capability_context();
    
    for capability in required_capabilities {
        if !unified_context.has_domain_capability(domain_id, capability) {
            return Err(EffectError::CapabilityError(
                format!("Missing domain capability: {:?} for domain {}", capability, domain_id)
            ));
        }
    }
    
    Ok(())
}

/// Creates inherited capabilities for composed effects
pub fn create_inherited_capabilities(
    parent_context: &EffectContext,
    inheritance_rules: &HashMap<String, Vec<String>>
) -> EffectContext {
    let mut child_context = EffectContext::new();
    
    // Copy identity
    if let Some(identity) = parent_context.identity() {
        child_context.set_identity(identity.clone());
    }
    
    // Parse parent capabilities
    let parent_unified = parent_context.to_unified_capability_context();
    
    // Apply inheritance rules
    for (parent_capability, inheritable_capabilities) in inheritance_rules {
        // Check if the parent has this capability
        if let Some(capability) = UnifiedCapability::from_string(parent_capability) {
            let has_capability = match &capability {
                UnifiedCapability::Domain(cap) => {
                    let domain_id = parent_context.get_parameter("domain_id")
                        .unwrap_or("default");
                    parent_unified.has_domain_capability(domain_id, cap)
                },
                UnifiedCapability::Effect(cap) => {
                    parent_unified.has_effect_capability(cap)
                },
                UnifiedCapability::CrossDomain(cap) => {
                    parent_unified.has_cross_domain_capability(cap)
                },
            };
            
            // If the parent has the capability, add all inheritable ones to child
            if has_capability {
                for inheritable in inheritable_capabilities {
                    if let Some(cap) = UnifiedCapability::from_string(inheritable) {
                        match cap {
                            UnifiedCapability::Domain(domain_cap) => {
                                let domain_id = parent_context.get_parameter("domain_id")
                                    .unwrap_or("default");
                                child_context.add_domain_capability(domain_id, domain_cap);
                            },
                            UnifiedCapability::Effect(effect_cap) => {
                                child_context.add_effect_capability(effect_cap);
                            },
                            UnifiedCapability::CrossDomain(cross_cap) => {
                                child_context.add_cross_domain_capability(cross_cap);
                            },
                        }
                    }
                }
            }
        }
    }
    
    // Copy parameters
    for (key, value) in parent_context.parameters() {
        child_context.set_parameter(key, value.clone());
    }
    
    child_context
} 