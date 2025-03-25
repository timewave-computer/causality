// Unified capability system module
// This module contains the unified capability system for domain adapters and effects

pub mod unified;
pub mod conversion;
pub mod verification;

// Re-export key types and traits
pub use unified::{
    UnifiedCapability, EffectCapability, CrossDomainCapability,
    UnifiedCapabilityContext, UnifiedCapabilityManager, EffectContextCapabilityExt
};
pub use conversion::{
    convert_domain_to_effect_capabilities, convert_effect_to_domain_capabilities,
    map_domain_to_effect_capability, map_effect_to_domain_capability,
    convert_domain_adapter_to_effect_context, enhance_effect_context_with_domain_capabilities,
    verify_domain_capabilities, create_inherited_capabilities
};
pub use verification::{
    CapabilityVerifier, DefaultCapabilityVerifier,
    DomainIdGetter, CapabilityRequirements
}; 