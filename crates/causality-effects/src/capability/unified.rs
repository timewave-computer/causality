// Unified capability model for domain adapters and effects
// This file implements the unified capability model that bridges
// domain adapter capabilities with effect system capabilities.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_domain::domain::{DomainId, DomainType};
use causality_domain::capability::{DomainCapability, DomainCapabilityManager};
use causality_types::Result;
use crate::effect::{EffectContext, EffectId, EffectResult, EffectError};
use crate::types::EffectType;

/// Unified capability that can represent both domain and effect capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnifiedCapability {
    /// Domain-specific capability
    Domain(DomainCapability),
    
    /// Effect-specific capability
    Effect(EffectCapability),
    
    /// Cross-domain capability
    CrossDomain(CrossDomainCapability),
}

/// Standard effect capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectCapability {
    // Resource capabilities
    CreateResource,
    ReadResource,
    UpdateResource,
    DeleteResource,
    
    // Transaction capabilities
    SubmitTransaction,
    SignTransaction,
    
    // ZK capabilities
    GenerateProof,
    VerifyProof,
    
    // TEL capabilities
    ExecuteTEL,
    CompileTEL,
    
    // System capabilities
    AccessRegistry,
    ModifyRegistry,
    
    // Identity capabilities
    ImpersonateIdentity,
    DelegateCapability,
    
    // Custom capability
    Custom(String),
}

/// Cross-domain capabilities for operations spanning multiple domains
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossDomainCapability {
    // Asset bridge capabilities
    TransferAssets,
    LockAssets,
    ReleaseAssets,
    
    // Messaging capabilities
    SendMessage,
    ReceiveMessage,
    
    // Proof capabilities
    VerifyCrossDomainProof,
    GenerateCrossDomainProof,
    
    // Orchestration capabilities
    OrchestrateCrossDomainOperation,
    MonitorCrossDomainOperation,
    
    // Resource management capabilities
    ResourceLocking { lock_type: String },
    ResourceDependency { dependency_type: String },
    FullResourceControl,
    
    // Custom capability
    Custom(String),
}

impl UnifiedCapability {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            UnifiedCapability::Domain(cap) => format!("domain:{}", cap.to_string()),
            UnifiedCapability::Effect(cap) => format!("effect:{}", effect_capability_to_string(cap)),
            UnifiedCapability::CrossDomain(cap) => format!("cross:{}", cross_domain_capability_to_string(cap)),
        }
    }
    
    /// Parse from string
    pub fn from_string(s: &str) -> Option<Self> {
        if s.starts_with("domain:") {
            let cap_str = &s[7..];
            DomainCapability::from_string(cap_str).map(UnifiedCapability::Domain)
        } else if s.starts_with("effect:") {
            let cap_str = &s[7..];
            effect_capability_from_string(cap_str).map(UnifiedCapability::Effect)
        } else if s.starts_with("cross:") {
            let cap_str = &s[6..];
            cross_domain_capability_from_string(cap_str).map(UnifiedCapability::CrossDomain)
        } else {
            None
        }
    }
}

// Helper function to convert EffectCapability to string
fn effect_capability_to_string(cap: &EffectCapability) -> String {
    match cap {
        EffectCapability::CreateResource => "create_resource".to_string(),
        EffectCapability::ReadResource => "read_resource".to_string(),
        EffectCapability::UpdateResource => "update_resource".to_string(),
        EffectCapability::DeleteResource => "delete_resource".to_string(),
        EffectCapability::SubmitTransaction => "submit_transaction".to_string(),
        EffectCapability::SignTransaction => "sign_transaction".to_string(),
        EffectCapability::GenerateProof => "generate_proof".to_string(),
        EffectCapability::VerifyProof => "verify_proof".to_string(),
        EffectCapability::ExecuteTEL => "execute_tel".to_string(),
        EffectCapability::CompileTEL => "compile_tel".to_string(),
        EffectCapability::AccessRegistry => "access_registry".to_string(),
        EffectCapability::ModifyRegistry => "modify_registry".to_string(),
        EffectCapability::ImpersonateIdentity => "impersonate_identity".to_string(),
        EffectCapability::DelegateCapability => "delegate_capability".to_string(),
        EffectCapability::Custom(name) => format!("custom_{}", name),
    }
}

// Helper function to parse EffectCapability from string
fn effect_capability_from_string(s: &str) -> Option<EffectCapability> {
    match s {
        "create_resource" => Some(EffectCapability::CreateResource),
        "read_resource" => Some(EffectCapability::ReadResource),
        "update_resource" => Some(EffectCapability::UpdateResource),
        "delete_resource" => Some(EffectCapability::DeleteResource),
        "submit_transaction" => Some(EffectCapability::SubmitTransaction),
        "sign_transaction" => Some(EffectCapability::SignTransaction),
        "generate_proof" => Some(EffectCapability::GenerateProof),
        "verify_proof" => Some(EffectCapability::VerifyProof),
        "execute_tel" => Some(EffectCapability::ExecuteTEL),
        "compile_tel" => Some(EffectCapability::CompileTEL),
        "access_registry" => Some(EffectCapability::AccessRegistry),
        "modify_registry" => Some(EffectCapability::ModifyRegistry),
        "impersonate_identity" => Some(EffectCapability::ImpersonateIdentity),
        "delegate_capability" => Some(EffectCapability::DelegateCapability),
        s if s.starts_with("custom_") => Some(EffectCapability::Custom(s[7..].to_string())),
        _ => None,
    }
}

// Helper function to convert CrossDomainCapability to string
fn cross_domain_capability_to_string(cap: &CrossDomainCapability) -> String {
    match cap {
        CrossDomainCapability::TransferAssets => "transfer_assets".to_string(),
        CrossDomainCapability::LockAssets => "lock_assets".to_string(),
        CrossDomainCapability::ReleaseAssets => "release_assets".to_string(),
        CrossDomainCapability::SendMessage => "send_message".to_string(),
        CrossDomainCapability::ReceiveMessage => "receive_message".to_string(),
        CrossDomainCapability::VerifyCrossDomainProof => "verify_cross_domain_proof".to_string(),
        CrossDomainCapability::GenerateCrossDomainProof => "generate_cross_domain_proof".to_string(),
        CrossDomainCapability::OrchestrateCrossDomainOperation => "orchestrate_cross_domain_operation".to_string(),
        CrossDomainCapability::MonitorCrossDomainOperation => "monitor_cross_domain_operation".to_string(),
        CrossDomainCapability::ResourceLocking { lock_type } => format!("resource_locking_{}", lock_type),
        CrossDomainCapability::ResourceDependency { dependency_type } => format!("resource_dependency_{}", dependency_type),
        CrossDomainCapability::FullResourceControl => "full_resource_control".to_string(),
        CrossDomainCapability::Custom(name) => format!("custom_{}", name),
    }
}

// Helper function to parse CrossDomainCapability from string
fn cross_domain_capability_from_string(s: &str) -> Option<CrossDomainCapability> {
    match s {
        "transfer_assets" => Some(CrossDomainCapability::TransferAssets),
        "lock_assets" => Some(CrossDomainCapability::LockAssets),
        "release_assets" => Some(CrossDomainCapability::ReleaseAssets),
        "send_message" => Some(CrossDomainCapability::SendMessage),
        "receive_message" => Some(CrossDomainCapability::ReceiveMessage),
        "verify_cross_domain_proof" => Some(CrossDomainCapability::VerifyCrossDomainProof),
        "generate_cross_domain_proof" => Some(CrossDomainCapability::GenerateCrossDomainProof),
        "orchestrate_cross_domain_operation" => Some(CrossDomainCapability::OrchestrateCrossDomainOperation),
        "monitor_cross_domain_operation" => Some(CrossDomainCapability::MonitorCrossDomainOperation),
        s if s.starts_with("resource_locking_") => {
            let lock_type = &s[18..];
            Some(CrossDomainCapability::ResourceLocking { lock_type: lock_type.to_string() })
        },
        s if s.starts_with("resource_dependency_") => {
            let dependency_type = &s[18..];
            Some(CrossDomainCapability::ResourceDependency { dependency_type: dependency_type.to_string() })
        },
        "full_resource_control" => Some(CrossDomainCapability::FullResourceControl),
        s if s.starts_with("custom_") => Some(CrossDomainCapability::Custom(s[7..].to_string())),
        _ => None,
    }
}

/// Unified capability context for both domain and effect capabilities
#[derive(Debug, Clone)]
pub struct UnifiedCapabilityContext {
    /// Domain-specific capabilities by domain ID
    domain_capabilities: HashMap<DomainId, HashSet<DomainCapability>>,
    
    /// Effect capabilities
    effect_capabilities: HashSet<EffectCapability>,
    
    /// Cross-domain capabilities
    cross_domain_capabilities: HashSet<CrossDomainCapability>,
}

impl UnifiedCapabilityContext {
    /// Create a new empty capability context
    pub fn new() -> Self {
        Self {
            domain_capabilities: HashMap::new(),
            effect_capabilities: HashSet::new(),
            cross_domain_capabilities: HashSet::new(),
        }
    }
    
    /// Create from an effect context
    pub fn from_effect_context(context: &EffectContext) -> Self {
        let mut result = Self::new();
        
        // Extract capabilities from the effect context
        // This would parse the capability strings and organize them
        for capability_string in context.capabilities() {
            if let Some(capability) = UnifiedCapability::from_string(&capability_string) {
                match capability {
                    UnifiedCapability::Domain(domain_cap) => {
                        // Extract domain ID from context parameters or use a default
                        let domain_id = context.get_parameter("domain_id")
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "default".to_string());
                        
                        result.add_domain_capability(domain_id, domain_cap);
                    },
                    UnifiedCapability::Effect(effect_cap) => {
                        result.add_effect_capability(effect_cap);
                    },
                    UnifiedCapability::CrossDomain(cross_cap) => {
                        result.add_cross_domain_capability(cross_cap);
                    }
                }
            }
        }
        
        result
    }
    
    /// Add a domain capability
    pub fn add_domain_capability(&mut self, domain_id: impl Into<String>, capability: DomainCapability) {
        let domain_id = domain_id.into();
        self.domain_capabilities
            .entry(domain_id)
            .or_insert_with(HashSet::new)
            .insert(capability);
    }
    
    /// Add an effect capability
    pub fn add_effect_capability(&mut self, capability: EffectCapability) {
        self.effect_capabilities.insert(capability);
    }
    
    /// Add a cross-domain capability
    pub fn add_cross_domain_capability(&mut self, capability: CrossDomainCapability) {
        self.cross_domain_capabilities.insert(capability);
    }
    
    /// Check if the context has a specific domain capability
    pub fn has_domain_capability(&self, domain_id: &str, capability: &DomainCapability) -> bool {
        self.domain_capabilities
            .get(domain_id)
            .map(|caps| caps.contains(capability))
            .unwrap_or(false)
    }
    
    /// Check if the context has a specific effect capability
    pub fn has_effect_capability(&self, capability: &EffectCapability) -> bool {
        self.effect_capabilities.contains(capability)
    }
    
    /// Check if the context has a specific cross-domain capability
    pub fn has_cross_domain_capability(&self, capability: &CrossDomainCapability) -> bool {
        self.cross_domain_capabilities.contains(capability)
    }
    
    /// Convert to a set of capability strings for an effect context
    pub fn to_capability_strings(&self) -> Vec<String> {
        let mut result = Vec::new();
        
        // Add domain capabilities
        for (domain_id, caps) in &self.domain_capabilities {
            for cap in caps {
                result.push(UnifiedCapability::Domain(cap.clone()).to_string());
            }
        }
        
        // Add effect capabilities
        for cap in &self.effect_capabilities {
            result.push(UnifiedCapability::Effect(cap.clone()).to_string());
        }
        
        // Add cross-domain capabilities
        for cap in &self.cross_domain_capabilities {
            result.push(UnifiedCapability::CrossDomain(cap.clone()).to_string());
        }
        
        result
    }
    
    /// Convert to an effect context
    pub fn to_effect_context(&self, identity: String) -> EffectContext {
        let mut context = EffectContext::new();
        
        // Set identity
        context.set_identity(identity);
        
        // Add capabilities
        for cap_string in self.to_capability_strings() {
            context.add_capability(cap_string);
        }
        
        // Add domain IDs as parameters for domain capabilities
        for domain_id in self.domain_capabilities.keys() {
            context.set_parameter("domain_id", domain_id.clone());
        }
        
        context
    }
}

/// Unified capability manager for both domain and effect capabilities
pub struct UnifiedCapabilityManager {
    /// Domain capability manager
    domain_manager: Arc<DomainCapabilityManager>,
    
    /// Effect capabilities by identity
    effect_capabilities: RwLock<HashMap<String, HashSet<EffectCapability>>>,
    
    /// Cross-domain capabilities by identity
    cross_domain_capabilities: RwLock<HashMap<String, HashSet<CrossDomainCapability>>>,
}

impl UnifiedCapabilityManager {
    /// Create a new unified capability manager
    pub fn new(domain_manager: Arc<DomainCapabilityManager>) -> Self {
        Self {
            domain_manager,
            effect_capabilities: RwLock::new(HashMap::new()),
            cross_domain_capabilities: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register effect capabilities for an identity
    pub fn register_effect_capabilities(&self, identity: &str, capabilities: HashSet<EffectCapability>) -> Result<()> {
        let mut caps = self.effect_capabilities.write().unwrap();
        caps.insert(identity.to_string(), capabilities);
        Ok(())
    }
    
    /// Register cross-domain capabilities for an identity
    pub fn register_cross_domain_capabilities(&self, identity: &str, capabilities: HashSet<CrossDomainCapability>) -> Result<()> {
        let mut caps = self.cross_domain_capabilities.write().unwrap();
        caps.insert(identity.to_string(), capabilities);
        Ok(())
    }
    
    /// Get effect capabilities for an identity
    pub fn get_effect_capabilities(&self, identity: &str) -> Option<HashSet<EffectCapability>> {
        let caps = self.effect_capabilities.read().unwrap();
        caps.get(identity).cloned()
    }
    
    /// Get cross-domain capabilities for an identity
    pub fn get_cross_domain_capabilities(&self, identity: &str) -> Option<HashSet<CrossDomainCapability>> {
        let caps = self.cross_domain_capabilities.read().unwrap();
        caps.get(identity).cloned()
    }
    
    /// Check if an identity has a specific effect capability
    pub fn has_effect_capability(&self, identity: &str, capability: &EffectCapability) -> bool {
        let caps = self.effect_capabilities.read().unwrap();
        caps.get(identity)
            .map(|set| set.contains(capability))
            .unwrap_or(false)
    }
    
    /// Check if an identity has a specific cross-domain capability
    pub fn has_cross_domain_capability(&self, identity: &str, capability: &CrossDomainCapability) -> bool {
        let caps = self.cross_domain_capabilities.read().unwrap();
        caps.get(identity)
            .map(|set| set.contains(capability))
            .unwrap_or(false)
    }
    
    /// Create a unified capability context for an identity
    pub fn create_context(&self, identity: &str) -> Result<UnifiedCapabilityContext> {
        let mut context = UnifiedCapabilityContext::new();
        
        // Add effect capabilities
        if let Some(effect_caps) = self.get_effect_capabilities(identity) {
            for cap in effect_caps {
                context.add_effect_capability(cap);
            }
        }
        
        // Add cross-domain capabilities
        if let Some(cross_caps) = self.get_cross_domain_capabilities(identity) {
            for cap in cross_caps {
                context.add_cross_domain_capability(cap);
            }
        }
        
        // Add domain capabilities from domain manager
        // This would require domain-specific logic to map identity to domain-specific identities
        
        Ok(context)
    }
    
    /// Convert a domain capability to an effect capability
    pub fn map_domain_to_effect_capability(&self, domain_cap: &DomainCapability) -> Option<EffectCapability> {
        match domain_cap {
            DomainCapability::SendTransaction => Some(EffectCapability::SubmitTransaction),
            DomainCapability::SignTransaction => Some(EffectCapability::SignTransaction),
            DomainCapability::ReadState => Some(EffectCapability::ReadResource),
            DomainCapability::WriteState => Some(EffectCapability::UpdateResource),
            DomainCapability::GenerateProof => Some(EffectCapability::GenerateProof),
            DomainCapability::VerifyProof => Some(EffectCapability::VerifyProof),
            // For other domain capabilities, there may not be a direct mapping
            _ => None,
        }
    }
    
    /// Convert an effect capability to a domain capability
    pub fn map_effect_to_domain_capability(&self, effect_cap: &EffectCapability) -> Option<DomainCapability> {
        match effect_cap {
            EffectCapability::SubmitTransaction => Some(DomainCapability::SendTransaction),
            EffectCapability::SignTransaction => Some(DomainCapability::SignTransaction),
            EffectCapability::ReadResource => Some(DomainCapability::ReadState),
            EffectCapability::UpdateResource => Some(DomainCapability::WriteState),
            EffectCapability::GenerateProof => Some(DomainCapability::GenerateProof),
            EffectCapability::VerifyProof => Some(DomainCapability::VerifyProof),
            // For other effect capabilities, there may not be a direct mapping
            _ => None,
        }
    }
    
    /// Check if an effect requires a specific domain capability
    pub fn effect_requires_domain_capability(&self, effect_type: &EffectType, domain_id: &str) -> Option<DomainCapability> {
        match effect_type {
            EffectType::Create => Some(DomainCapability::WriteState),
            EffectType::Read => Some(DomainCapability::ReadState),
            EffectType::Update => Some(DomainCapability::WriteState),
            EffectType::Delete => Some(DomainCapability::WriteState),
            EffectType::Transfer => Some(DomainCapability::SendTransaction),
            EffectType::Execute => Some(DomainCapability::ExecuteContract),
            EffectType::Call => Some(DomainCapability::QueryContract),
            EffectType::CompileZkProgram => Some(DomainCapability::ZkProve),
            EffectType::GenerateZkProof => Some(DomainCapability::ZkProve),
            EffectType::VerifyZkProof => Some(DomainCapability::ZkVerify),
            // For other effect types, there may not be a direct mapping to domain capabilities
            _ => None,
        }
    }
}

/// Trait for effects that require domain capabilities
pub trait DomainCapabilityAware {
    /// Get required domain capabilities
    fn required_domain_capabilities(&self) -> Vec<(DomainId, DomainCapability)>;
    
    /// Check if the effect has all required domain capabilities in the context
    fn has_required_domain_capabilities(&self, context: &UnifiedCapabilityContext) -> bool {
        for (domain_id, capability) in self.required_domain_capabilities() {
            if !context.has_domain_capability(&domain_id, &capability) {
                return false;
            }
        }
        true
    }
}

/// Extension trait for effect context to support unified capabilities
pub trait EffectContextCapabilityExt {
    /// Convert to a unified capability context
    fn to_unified_capability_context(&self) -> UnifiedCapabilityContext;
    
    /// Check if the context has a specific domain capability
    fn has_domain_capability(&self, domain_id: &str, capability: &DomainCapability) -> bool;
    
    /// Check if the context has a specific effect capability
    fn has_effect_capability(&self, capability: &EffectCapability) -> bool;
    
    /// Check if the context has a specific cross-domain capability
    fn has_cross_domain_capability(&self, capability: &CrossDomainCapability) -> bool;
    
    /// Add a domain capability to the context
    fn add_domain_capability(&mut self, domain_id: &str, capability: DomainCapability);
    
    /// Add an effect capability to the context
    fn add_effect_capability(&mut self, capability: EffectCapability);
    
    /// Add a cross-domain capability to the context
    fn add_cross_domain_capability(&mut self, capability: CrossDomainCapability);
}

// Implementation of the extension trait for EffectContext
impl EffectContextCapabilityExt for EffectContext {
    fn to_unified_capability_context(&self) -> UnifiedCapabilityContext {
        UnifiedCapabilityContext::from_effect_context(self)
    }
    
    fn has_domain_capability(&self, domain_id: &str, capability: &DomainCapability) -> bool {
        self.to_unified_capability_context().has_domain_capability(domain_id, capability)
    }
    
    fn has_effect_capability(&self, capability: &EffectCapability) -> bool {
        self.to_unified_capability_context().has_effect_capability(capability)
    }
    
    fn has_cross_domain_capability(&self, capability: &CrossDomainCapability) -> bool {
        self.to_unified_capability_context().has_cross_domain_capability(capability)
    }
    
    fn add_domain_capability(&mut self, domain_id: &str, capability: DomainCapability) {
        self.add_capability(UnifiedCapability::Domain(capability).to_string());
        self.set_parameter("domain_id", domain_id.to_string());
    }
    
    fn add_effect_capability(&mut self, capability: EffectCapability) {
        self.add_capability(UnifiedCapability::Effect(capability).to_string());
    }
    
    fn add_cross_domain_capability(&mut self, capability: CrossDomainCapability) {
        self.add_capability(UnifiedCapability::CrossDomain(capability).to_string());
    }
} 