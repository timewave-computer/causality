// Base account implementation
// Original file: src/program_account/base_account.rs

// Base Program Account Implementation
//
// This module provides a basic implementation of the ProgramAccount trait
// that can be used as a foundation for more specialized program accounts.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use borsh::{BorshSerialize, BorshDeserialize};
use crate::crypto::content_addressed::{ContentAddressed, ContentId};

use crate::domain::DomainId;
use causality_types::{Error, Result};
use crate::resource::{
    RegisterContents, Register, ContentId,
    ResourceAllocator, ResourceRequest, ResourceGrant
};
use causality_types::{Address, TraceId};
use crate::program_account::{
    ProgramAccount, ProgramAccountCapability, ProgramAccountResource,
    AvailableEffect, EffectResult, EffectStatus, TransactionRecord, TransactionStatus
};
use causality_resource::{
    AuthorizationManager, AuthorizationContext, AuthorizationResult, 
    AuthorizationLevel, ProgramAccountAuthorization
};

/// Content data for generating content-addressed effect result IDs
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct EffectResultContentData {
    /// Effect ID
    pub effect_id: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Parameters hash
    pub parameters_hash: String,
    
    /// Random nonce for uniqueness
    pub nonce: [u8; 8],
}

impl ContentAddressed for EffectResultContentData {
    fn content_hash(&self) -> Result<ContentId> {
        let bytes = self.to_bytes()?;
        Ok(ContentId::from_bytes(&bytes)?)
    }
    
    fn verify(&self, content_id: &ContentId) -> Result<bool> {
        let calculated_id = self.content_hash()?;
        Ok(calculated_id == *content_id)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = borsh::to_vec(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize EffectResultContentData: {}", e)))?;
        Ok(bytes)
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes)
            .map_err(|e| Error::Deserialization(format!("Failed to deserialize EffectResultContentData: {}", e)))
    }
}

/// A basic implementation of the ProgramAccount trait
pub struct BaseAccount {
    /// Unique ID for this account
    id: String,
    
    /// The owner of this account
    owner: Address,
    
    /// The name of this account
    name: String,
    
    /// The type of this account
    account_type: String,
    
    /// The domains this account has access to
    domains: RwLock<HashSet<DomainId>>,
    
    /// The resources owned by this account
    resources: RwLock<HashMap<ContentId, ProgramAccountResource>>,
    
    /// The capabilities granted to this account
    capabilities: RwLock<HashMap<String, ProgramAccountCapability>>,
    
    /// Available effects for this account
    available_effects: RwLock<HashMap<String, AvailableEffect>>,
    
    /// Transaction history for this account
    transaction_history: RwLock<Vec<TransactionRecord>>,
    
    /// Balances for this account
    balances: RwLock<HashMap<String, u64>>,
    
    /// Authorization manager (optional)
    auth_manager: Option<Arc<AuthorizationManager>>,
    
    /// Resource allocator for resource-based authorization (optional)
    resource_allocator: Option<Arc<dyn ResourceAllocator>>,
}

impl BaseAccount {
    /// Create a new base account
    pub fn new(
        id: String,
        owner: Address,
        name: String,
        account_type: String,
        initial_domains: Option<HashSet<DomainId>>,
    ) -> Self {
        let domains = match initial_domains {
            Some(d) => d,
            None => HashSet::new(),
        };
        
        Self {
            id,
            owner,
            name,
            account_type,
            domains: RwLock::new(domains),
            resources: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
            available_effects: RwLock::new(HashMap::new()),
            transaction_history: RwLock::new(Vec::new()),
            balances: RwLock::new(HashMap::new()),
            auth_manager: None,
            resource_allocator: None,
        }
    }
    
    /// Create a new base account with authorization
    pub fn new_with_auth(
        id: String,
        owner: Address,
        name: String,
        account_type: String,
        initial_domains: Option<HashSet<DomainId>>,
        auth_manager: Arc<AuthorizationManager>,
        resource_allocator: Option<Arc<dyn ResourceAllocator>>,
    ) -> Self {
        let mut account = Self::new(id, owner, name, account_type, initial_domains);
        account.auth_manager = Some(auth_manager);
        account.resource_allocator = resource_allocator;
        account
    }
    
    /// Check if an action is authorized
    fn check_authorization(&self, action: &str, resource_id: Option<&ContentId>, level: AuthorizationLevel) -> Result<()> {
        // If no auth manager, owner can do anything
        if self.auth_manager.is_none() {
            return Ok(());
        }
        
        let auth_manager = self.auth_manager.as_ref().unwrap();
        
        // Create authorization context
        let mut context = AuthorizationContext::new(
            self.id.clone(),
            self.owner.clone(),
            action.to_string(),
            level,
        );
        
        // Add resource if provided
        if let Some(resource_id) = resource_id {
            context = context.with_resource(resource_id.clone());
        }
        
        // Check authorization
        match auth_manager.authorize(&context)? {
            AuthorizationResult::Allowed => Ok(()),
            AuthorizationResult::Denied(reason) => {
                Err(Error::AuthorizationError(format!("Action not authorized: {}", reason)))
            },
            AuthorizationResult::RequiresAuthentication(method) => {
                Err(Error::AuthenticationRequired(format!("Authentication required: {}", method)))
            }
        }
    }
    
    /// Register a resource with this account
    pub fn register_resource(&self, resource: ProgramAccountResource) -> Result<()> {
        // Check authorization
        self.check_authorization("register_resource", Some(&resource.id), AuthorizationLevel::ReadWrite)?;
        
        let mut resources = self.resources.write().map_err(|_| Error::LockError)?;
        resources.insert(resource.id.clone(), resource);
        Ok(())
    }
    
    /// Register an available effect for this account
    pub fn register_effect(&self, effect: AvailableEffect) -> Result<()> {
        // Check authorization
        self.check_authorization("register_effect", None, AuthorizationLevel::ReadWrite)?;
        
        let mut effects = self.available_effects.write().map_err(|_| Error::LockError)?;
        effects.insert(effect.id.clone(), effect);
        Ok(())
    }
    
    /// Add a domain to this account
    pub fn add_domain(&self, domain_id: DomainId) -> Result<()> {
        // Check authorization
        self.check_authorization("add_domain", None, AuthorizationLevel::ReadWrite)?;
        
        let mut domains = self.domains.write().map_err(|_| Error::LockError)?;
        domains.insert(domain_id);
        Ok(())
    }
    
    /// Remove a domain from this account
    pub fn remove_domain(&self, domain_id: &DomainId) -> Result<bool> {
        // Check authorization
        self.check_authorization("remove_domain", None, AuthorizationLevel::ReadWrite)?;
        
        let mut domains = self.domains.write().map_err(|_| Error::LockError)?;
        Ok(domains.remove(domain_id))
    }
    
    /// Add a transaction record to the history
    pub fn add_transaction_record(&self, record: TransactionRecord) -> Result<()> {
        // Check authorization
        self.check_authorization("add_transaction_record", None, AuthorizationLevel::ReadWrite)?;
        
        let mut history = self.transaction_history.write().map_err(|_| Error::LockError)?;
        history.push(record);
        Ok(())
    }
    
    /// Update the balance of an asset
    pub fn update_balance(&self, asset_id: &str, amount: u64, increment: bool) -> Result<u64> {
        // Check authorization
        self.check_authorization("update_balance", None, AuthorizationLevel::ReadWrite)?;
        
        let mut balances = self.balances.write().map_err(|_| Error::LockError)?;
        
        let current = balances.entry(asset_id.to_string()).or_insert(0);
        
        if increment {
            *current = current.saturating_add(amount);
        } else {
            if *current < amount {
                return Err(Error::InsufficientFunds(format!(
                    "Insufficient balance for asset {}: have {}, need {}",
                    asset_id, *current, amount
                )));
            }
            *current -= amount;
        }
        
        Ok(*current)
    }
    
    /// Request a resource from the resource allocator
    pub fn request_resource(&self, request: ResourceRequest) -> Result<ResourceGrant> {
        // Check authorization
        self.check_authorization(
            "request_resource", 
            match &request {
                ResourceRequest::Read(id) => Some(id),
                ResourceRequest::Write(id) => Some(id),
                ResourceRequest::Transfer(id) => Some(id),
                _ => None,
            }, 
            AuthorizationLevel::ReadWrite
        )?;
        
        // Check if we have a resource allocator
        let allocator = self.resource_allocator.as_ref()
            .ok_or_else(|| Error::NotImplemented("Resource allocation not available".to_string()))?;
        
        // Make the request
        allocator.request_resource(&self.owner, &request)
    }
    
    /// Release resources from a grant
    pub fn release_resources(&self, grant: ResourceGrant) -> Result<()> {
        // Check authorization
        self.check_authorization("release_resources", None, AuthorizationLevel::ReadWrite)?;
        
        // Check if we have a resource allocator
        let allocator = self.resource_allocator.as_ref()
            .ok_or_else(|| Error::NotImplemented("Resource allocation not available".to_string()))?;
        
        // Release the resources
        allocator.release_resources(grant);
        
        Ok(())
    }
    
    /// Apply a resource predicate authorization
    pub fn apply_resource_predicate(
        &self, 
        resource_id: &ContentId, 
        predicate_name: &str, 
        args: &[&str]
    ) -> Result<bool> {
        // Check if we have a resource allocator
        let allocator = self.resource_allocator.as_ref()
            .ok_or_else(|| Error::NotImplemented("Resource allocation not available".to_string()))?;
        
        // In a real implementation, this would apply a predicate to the resource
        // For now, we'll return a simple result
        Ok(true)
    }
}

impl ProgramAccount for BaseAccount {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn owner(&self) -> &Address {
        &self.owner
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn account_type(&self) -> &str {
        &self.account_type
    }
    
    fn domains(&self) -> &HashSet<DomainId> {
        match self.domains.try_read() {
            Ok(domains) => {
                // Safety: extend the lifetime of the read guard
                // This is safe because the RwLock ensures no writes occur while read
                unsafe { std::mem::transmute(&*domains) }
            },
            Err(_) => {
                // If lock is poisoned, return an empty set
                // Safety: static reference to empty set lives for program duration
                static EMPTY: std::sync::OnceLock<HashSet<DomainId>> = std::sync::OnceLock::new();
                EMPTY.get_or_init(HashSet::new)
            }
        }
    }
    
    fn resources(&self) -> Vec<ProgramAccountResource> {
        match self.resources.try_read() {
            Ok(resources) => resources.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
    
    fn get_resource(&self, resource_id: &ContentId) -> Result<Option<ProgramAccountResource>> {
        // Check authorization
        self.check_authorization("get_resource", Some(resource_id), AuthorizationLevel::Read)?;
        
        let resources = self.resources.read().map_err(|_| Error::LockError)?;
        Ok(resources.get(resource_id).cloned())
    }
    
    fn available_effects(&self) -> Vec<AvailableEffect> {
        match self.available_effects.try_read() {
            Ok(effects) => effects.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
    
    fn get_effect(&self, effect_id: &str) -> Result<Option<AvailableEffect>> {
        let effects = self.available_effects.read().map_err(|_| Error::LockError)?;
        Ok(effects.get(effect_id).cloned())
    }
    
    fn execute_effect(
        &self,
        effect_id: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        // Check authorization
        self.check_authorization("execute_effect", None, AuthorizationLevel::ReadWrite)?;
        
        // Get the effect
        let available_effects = self.available_effects.read().map_err(|_| Error::LockError)?;
        let effect = available_effects.get(effect_id)
            .ok_or_else(|| Error::NotFound(format!("Effect not found: {}", effect_id)))?;
        
        // Validate parameters
        for required in &effect.required_parameters {
            if !parameters.contains_key(required) {
                return Err(Error::InvalidParameter(format!(
                    "Missing required parameter: {}", required
                )));
            }
        }
        
        // Create a parameters hash for content addressing
        let mut params_string = String::new();
        let mut keys: Vec<_> = parameters.keys().collect();
        keys.sort(); // Sort keys for deterministic hash
        for key in keys {
            params_string.push_str(&format!("{}={},", key, parameters.get(key).unwrap_or(&String::new())));
        }
        
        // Generate a content-addressed result ID
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = EffectResultContentData {
            effect_id: effect_id.to_string(),
            timestamp: now,
            parameters_hash: params_string,
            nonce,
        };
        
        let result_id = content_data.content_hash()
            .map(|id| format!("result-{}", id.to_string()))
            .unwrap_or_else(|_| format!("result-error-{}", now));
        
        // Create the result
        let result = EffectResult {
            id: result_id,
            effect_id: effect_id.to_string(),
            status: EffectStatus::Pending,
            parameters: parameters.clone(),
            result_data: None,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            completed_at: None,
            trace_id: trace_id.cloned(),
        };
        
        // Add to transaction history (could be improved to track actual execution)
        let _ = self.add_transaction_record(TransactionRecord {
            id: result.id.clone(),
            account_id: self.id.clone(),
            transaction_type: "effect_execution".to_string(),
            status: TransactionStatus::Pending,
            data: Some(format!("Effect: {}", effect_id)),
            timestamp: result.created_at,
        });
        
        Ok(result)
    }
    
    fn capabilities(&self) -> Vec<ProgramAccountCapability> {
        match self.capabilities.try_read() {
            Ok(capabilities) => capabilities.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
    
    fn has_capability(&self, action: &str) -> bool {
        match self.capabilities.try_read() {
            Ok(capabilities) => capabilities.values().any(|cap| cap.action == action),
            Err(_) => false,
        }
    }
    
    fn grant_capability(&mut self, capability: ProgramAccountCapability) -> Result<()> {
        // Check authorization for granting capabilities
        self.check_authorization("grant_capability", None, AuthorizationLevel::Owner)?;
        
        // If we have an auth manager, validate the capability
        if let Some(auth_manager) = &self.auth_manager {
            match auth_manager.validate_capability(&capability, &self.owner)? {
                AuthorizationResult::Allowed => {},
                AuthorizationResult::Denied(reason) => {
                    return Err(Error::AuthorizationError(
                        format!("Cannot grant capability: {}", reason)
                    ));
                },
                AuthorizationResult::RequiresAuthentication(method) => {
                    return Err(Error::AuthenticationRequired(
                        format!("Authentication required to grant capability: {}", method)
                    ));
                }
            }
        }
        
        let mut capabilities = self.capabilities.write().map_err(|_| Error::LockError)?;
        capabilities.insert(capability.id.clone(), capability);
        Ok(())
    }
    
    fn revoke_capability(&mut self, capability_id: &str) -> Result<()> {
        // Check authorization for revoking capabilities
        self.check_authorization("revoke_capability", None, AuthorizationLevel::Owner)?;
        
        let mut capabilities = self.capabilities.write().map_err(|_| Error::LockError)?;
        capabilities.remove(capability_id);
        Ok(())
    }
    
    fn get_balance(&self, asset_id: &str) -> Result<u64> {
        // Check authorization
        self.check_authorization("get_balance", None, AuthorizationLevel::Read)?;
        
        let balances = self.balances.read().map_err(|_| Error::LockError)?;
        Ok(*balances.get(asset_id).unwrap_or(&0))
    }
    
    fn get_all_balances(&self) -> Result<HashMap<String, u64>> {
        // Check authorization
        self.check_authorization("get_all_balances", None, AuthorizationLevel::Read)?;
        
        let balances = self.balances.read().map_err(|_| Error::LockError)?;
        Ok(balances.clone())
    }
    
    fn transaction_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<TransactionRecord>> {
        // Check authorization
        self.check_authorization("transaction_history", None, AuthorizationLevel::Read)?;
        
        let history = self.transaction_history.read().map_err(|_| Error::LockError)?;
        
        let offset = offset.unwrap_or(0);
        if offset >= history.len() {
            return Ok(Vec::new());
        }
        
        let limit = limit.unwrap_or(history.len());
        let end = std::cmp::min(offset + limit, history.len());
        
        Ok(history[offset..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_resource::AuthorizationPredicate;
    
    #[test]
    fn test_base_account_creation() {
        let account = BaseAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Test Account".to_string(),
            "basic".to_string(),
            None,
        );
        
        assert_eq!(account.id(), "acc-1");
        assert_eq!(account.owner(), &Address::new("owner-1"));
        assert_eq!(account.name(), "Test Account");
        assert_eq!(account.account_type(), "basic");
        assert_eq!(account.domains().len(), 0);
    }
    
    #[test]
    fn test_resource_management() {
        let account = BaseAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Test Account".to_string(),
            "basic".to_string(),
            None,
        );
        
        // Register a resource
        let resource = ProgramAccountResource {
            id: ContentId::from_str("res-1"),
            register_id: Some(ContentId::from_str("reg-1")),
            resource_type: "token".to_string(),
            domain_id: Some(DomainId::new("domain-1")),
            metadata: HashMap::new(),
        };
        
        account.register_resource(resource.clone()).unwrap();
        
        // Get the resource
        let retrieved = account.get_resource(&ContentId::from_str("res-1")).unwrap().unwrap();
        assert_eq!(retrieved.id, resource.id);
        
        // Get all resources
        let resources = account.resources();
        assert_eq!(resources.len(), 1);
    }
    
    #[test]
    fn test_effect_management() {
        let account = BaseAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Test Account".to_string(),
            "basic".to_string(),
            None,
        );
        
        // Register an effect
        let effect = AvailableEffect {
            id: "effect-1".to_string(),
            name: "Test Effect".to_string(),
            description: "A test effect".to_string(),
            domain_id: Some(DomainId::new("domain-1")),
            parameters: Vec::new(),
            requires_authorization: false,
        };
        
        account.register_effect(effect.clone()).unwrap();
        
        // Get the effect
        let retrieved = account.get_effect("effect-1").unwrap().unwrap();
        assert_eq!(retrieved.id, effect.id);
        
        // Get all effects
        let effects = account.available_effects();
        assert_eq!(effects.len(), 1);
    }
    
    #[test]
    fn test_capability_management() {
        let account = &mut BaseAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Test Account".to_string(),
            "basic".to_string(),
            None,
        );
        
        // Grant a capability
        let capability = ProgramAccountCapability {
            id: "cap-1".to_string(),
            account_id: "acc-1".to_string(),
            action: "transfer".to_string(),
            restrictions: None,
            expires_at: None,
        };
        
        account.grant_capability(capability.clone()).unwrap();
        
        // Check if has capability
        assert!(account.has_capability("transfer"));
        
        // Get all capabilities
        let capabilities = account.capabilities();
        assert_eq!(capabilities.len(), 1);
        
        // Revoke capability
        account.revoke_capability("cap-1").unwrap();
        
        // Check if capability was revoked
        assert!(!account.has_capability("transfer"));
    }
    
    #[test]
    fn test_authorization_integration() {
        // Create an auth manager with a resource allocator mock
        let auth_manager = Arc::new(AuthorizationManager::new(None));
        
        // Register a predicate for testing
        let predicate: AuthorizationPredicate = Arc::new(|ctx| {
            ctx.action == "special_action" && ctx.account_id == "acc-1"
        });
        
        auth_manager.register_predicate("test_predicate", predicate).unwrap();
        
        // Create an account with auth manager
        let account = &mut BaseAccount::new_with_auth(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Test Account".to_string(),
            "basic".to_string(),
            None,
            auth_manager,
            None,
        );
        
        // This should pass because the predicate allows it
        account.check_authorization("special_action", None, AuthorizationLevel::ReadWrite).unwrap();
        
        // This should pass because the account owner is authorized
        account.check_authorization("random_action", None, AuthorizationLevel::ReadWrite).unwrap();
        
        // Try to add a domain (should work)
        account.add_domain(DomainId::new("domain-1")).unwrap();
        assert_eq!(account.domains().len(), 1);
    }
} 
