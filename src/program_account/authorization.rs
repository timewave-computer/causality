// Program Account Authorization Framework
//
// This module implements the user authorization framework for program accounts,
// leveraging the existing resource system's authorization capabilities.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::domain::DomainId;
use crate::error::{Error, Result};
use crate::resource::{
    Register, RegisterId, ContentId, 
    AuthorizationMethod, ResourceAllocator, ResourceRequest
};
use crate::program_account::ProgramAccountCapability;
use crate::types::{Address, TraceId};
use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

/// Authorization level for resources and capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationLevel {
    /// No access
    None,
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
    /// Full control (read, write, delegate)
    Owner,
    /// Administrative access (system-level)
    Admin,
}

/// A predicate used to determine if an operation is authorized
pub type AuthorizationPredicate = Arc<dyn Fn(&AuthorizationContext) -> bool + Send + Sync>;

/// Context for authorization decisions
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    /// The account performing the action
    pub account_id: String,
    /// The resource being accessed (if applicable)
    pub resource_id: Option<ContentId>,
    /// The authorization level being requested
    pub level: AuthorizationLevel,
    /// The specific action being performed
    pub action: String,
    /// The owner of the account
    pub account_owner: Address,
    /// Optional parameters for the authorization decision
    pub parameters: HashMap<String, String>,
    /// Current timestamp
    pub timestamp: u64,
    /// The domain where the operation is being performed
    pub domain_id: Option<DomainId>,
    /// Trace ID for this authorization request
    pub trace_id: Option<TraceId>,
}

impl AuthorizationContext {
    /// Create a new authorization context
    pub fn new(
        account_id: String,
        account_owner: Address,
        action: String,
        level: AuthorizationLevel,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            account_id,
            resource_id: None,
            level,
            action,
            account_owner,
            parameters: HashMap::new(),
            timestamp: now,
            domain_id: None,
            trace_id: None,
        }
    }
    
    /// Add a resource ID to the context
    pub fn with_resource(mut self, resource_id: ContentId) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
    
    /// Add a domain ID to the context
    pub fn with_domain(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    /// Add parameters to the context
    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.parameters = parameters;
        self
    }
    
    /// Add a trace ID to the context
    pub fn with_trace(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/// Result of an authorization check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Access is allowed
    Allowed,
    /// Access is denied with a reason
    Denied(String),
    /// Access requires additional authentication
    RequiresAuthentication(String),
}

/// A role assigned to an account for RBAC
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role {
    /// Name of the role
    pub name: String,
    /// Permissions granted by this role
    pub permissions: HashSet<String>,
    /// Domain-specific permissions
    pub domain_permissions: HashMap<DomainId, HashSet<String>>,
}

/// A delegate authorization that allows one account to act on behalf of another
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct DelegateAuthorization {
    /// ID of this delegation
    pub id: String,
    /// The account delegating authority
    pub delegator: String,
    /// The account receiving the authority
    pub delegate: String,
    /// The actions that are authorized
    pub actions: HashSet<String>,
    /// Scope restrictions (if any)
    pub restrictions: Option<HashMap<String, String>>,
    /// When this delegation expires
    pub expires_at: Option<u64>,
    /// Whether this delegation can be further delegated
    pub can_redelegate: bool,
}

impl ContentAddressed for DelegateAuthorization {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Since HashSet and HashMap don't implement BorshSerialize,
        // we need to create a serializable version of the data
        let actions_vec: Vec<String> = self.actions.iter().cloned().collect();
        let restrictions_vec: Option<Vec<(String, String)>> = self.restrictions.as_ref().map(|r| {
            r.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        });
        
        // Create a serializable struct for content hashing
        #[derive(BorshSerialize)]
        struct DelegateAuthorizationData<'a> {
            delegator: &'a str,
            delegate: &'a str,
            actions: &'a [String],
            restrictions: Option<&'a [(String, String)]>,
            expires_at: Option<u64>,
            can_redelegate: bool,
            timestamp: u64, // Add timestamp for uniqueness
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let data = DelegateAuthorizationData {
            delegator: &self.delegator,
            delegate: &self.delegate,
            actions: &actions_vec,
            restrictions: restrictions_vec.as_ref().map(|r| r.as_slice()),
            expires_at: self.expires_at,
            can_redelegate: self.can_redelegate,
            timestamp,
        };
        
        let serialized = data.try_to_vec().unwrap_or_default();
        hasher.hash(&serialized)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// A signature verification result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureVerificationResult {
    /// Signature is valid
    Valid,
    /// Signature is invalid with a reason
    Invalid(String),
    /// Verification is pending external validation
    Pending,
}

/// The authorization manager for program accounts
pub struct AuthorizationManager {
    /// Role definitions
    roles: RwLock<HashMap<String, Role>>,
    /// Account to role assignments
    account_roles: RwLock<HashMap<String, HashSet<String>>>,
    /// Delegated authorizations
    delegations: RwLock<HashMap<String, DelegateAuthorization>>,
    /// Custom authorization predicates
    predicates: RwLock<HashMap<String, AuthorizationPredicate>>,
    /// Resource system integration
    resource_allocator: Option<Arc<dyn ResourceAllocator>>,
}

impl AuthorizationManager {
    /// Create a new authorization manager
    pub fn new(resource_allocator: Option<Arc<dyn ResourceAllocator>>) -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            account_roles: RwLock::new(HashMap::new()),
            delegations: RwLock::new(HashMap::new()),
            predicates: RwLock::new(HashMap::new()),
            resource_allocator,
        }
    }
    
    /// Define a new role
    pub fn define_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().map_err(|_| Error::LockError)?;
        roles.insert(role.name.clone(), role);
        Ok(())
    }
    
    /// Assign a role to an account
    pub fn assign_role(&self, account_id: &str, role_name: &str) -> Result<()> {
        // Check if the role exists
        let roles = self.roles.read().map_err(|_| Error::LockError)?;
        if !roles.contains_key(role_name) {
            return Err(Error::InvalidArgument(format!("Role does not exist: {}", role_name)));
        }
        
        // Assign the role
        let mut account_roles = self.account_roles.write().map_err(|_| Error::LockError)?;
        let account_role_set = account_roles.entry(account_id.to_string()).or_insert_with(HashSet::new);
        account_role_set.insert(role_name.to_string());
        
        Ok(())
    }
    
    /// Remove a role from an account
    pub fn remove_role(&self, account_id: &str, role_name: &str) -> Result<()> {
        let mut account_roles = self.account_roles.write().map_err(|_| Error::LockError)?;
        
        if let Some(role_set) = account_roles.get_mut(account_id) {
            role_set.remove(role_name);
        }
        
        Ok(())
    }
    
    /// Create a delegation between accounts
    pub fn create_delegation(
        &self,
        delegator: &str,
        delegate: &str,
        actions: HashSet<String>,
        restrictions: Option<HashMap<String, String>>,
        expires_at: Option<u64>,
        can_redelegate: bool,
    ) -> Result<String> {
        // Create a temporary delegation for content ID generation
        let temp_delegation = DelegateAuthorization {
            id: String::new(), // Temporary
            delegator: delegator.to_string(),
            delegate: delegate.to_string(),
            actions: actions.clone(),
            restrictions: restrictions.clone(),
            expires_at,
            can_redelegate,
        };
        
        // Generate a content-derived ID
        let content_id = temp_delegation.content_id();
        let id = format!("delegation:{}", content_id);
        
        // Create the final delegation with the content ID
        let delegation = DelegateAuthorization {
            id: id.clone(),
            delegator: delegator.to_string(),
            delegate: delegate.to_string(),
            actions,
            restrictions,
            expires_at,
            can_redelegate,
        };
        
        // Store the delegation
        let mut delegations = self.delegations.write().map_err(|_| Error::LockError)?;
        delegations.insert(id.clone(), delegation);
        
        Ok(id)
    }
    
    /// Revoke a delegation
    pub fn revoke_delegation(&self, delegation_id: &str) -> Result<()> {
        let mut delegations = self.delegations.write().map_err(|_| Error::LockError)?;
        delegations.remove(delegation_id);
        Ok(())
    }
    
    /// Register a custom authorization predicate
    pub fn register_predicate(&self, name: &str, predicate: AuthorizationPredicate) -> Result<()> {
        let mut predicates = self.predicates.write().map_err(|_| Error::LockError)?;
        predicates.insert(name.to_string(), predicate);
        Ok(())
    }
    
    /// Check if an account has a specific permission
    pub fn has_permission(&self, account_id: &str, permission: &str, domain_id: Option<&DomainId>) -> Result<bool> {
        // Check account roles
        let account_roles = self.account_roles.read().map_err(|_| Error::LockError)?;
        let roles = self.roles.read().map_err(|_| Error::LockError)?;
        
        if let Some(role_names) = account_roles.get(account_id) {
            for role_name in role_names {
                if let Some(role) = roles.get(role_name) {
                    // Check global permissions
                    if role.permissions.contains(permission) {
                        return Ok(true);
                    }
                    
                    // Check domain-specific permissions
                    if let Some(domain_id) = domain_id {
                        if let Some(domain_perms) = role.domain_permissions.get(domain_id) {
                            if domain_perms.contains(permission) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        // Check delegations
        let delegations = self.delegations.read().map_err(|_| Error::LockError)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        for delegation in delegations.values() {
            // Skip expired delegations
            if let Some(expires) = delegation.expires_at {
                if now > expires {
                    continue;
                }
            }
            
            // Check if this delegation applies to this account
            if delegation.delegate == account_id {
                // Check if the action is authorized
                if delegation.actions.contains(permission) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Verify a signature for an operation
    pub fn verify_signature(
        &self,
        account_id: &str,
        action: &str,
        signature: &[u8],
        message: &[u8],
    ) -> Result<SignatureVerificationResult> {
        // In a real implementation, this would verify a cryptographic signature
        // For now, we'll return a simple stub implementation
        if signature.is_empty() {
            return Ok(SignatureVerificationResult::Invalid("Empty signature".to_string()));
        }
        
        // We'd normally verify the signature here against the account's public key
        // For demo purposes, we'll consider it valid
        Ok(SignatureVerificationResult::Valid)
    }
    
    /// Check if an operation is authorized using all available mechanisms
    pub fn authorize(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // First, check if the account owner is performing the action
        if context.account_owner == Address::new(&context.account_id) {
            // Owner can do anything on their own account
            return Ok(AuthorizationResult::Allowed);
        }
        
        // Check permissions based on roles
        if self.has_permission(&context.account_id, &context.action, context.domain_id.as_ref())? {
            return Ok(AuthorizationResult::Allowed);
        }
        
        // Check if there's a resource ID and we have a resource allocator
        if let (Some(resource_id), Some(allocator)) = (&context.resource_id, &self.resource_allocator) {
            // Use the resource system's authorization
            let address = Address::new(&context.account_id);
            
            // Translate the authorization level to a resource request
            let request = match context.level {
                AuthorizationLevel::Read => ResourceRequest::Read(resource_id.clone()),
                AuthorizationLevel::ReadWrite => ResourceRequest::Write(resource_id.clone()),
                AuthorizationLevel::Owner => ResourceRequest::Transfer(resource_id.clone()),
                _ => return Ok(AuthorizationResult::Denied(
                    "Unsupported authorization level for resource".to_string()
                )),
            };
            
            // Use the resource allocator to check authorization
            match allocator.validate_request(&address, &request) {
                Ok(_) => return Ok(AuthorizationResult::Allowed),
                Err(e) => return Ok(AuthorizationResult::Denied(e.to_string())),
            }
        }
        
        // Check custom predicates
        let predicates = self.predicates.read().map_err(|_| Error::LockError)?;
        for predicate in predicates.values() {
            if predicate(context) {
                return Ok(AuthorizationResult::Allowed);
            }
        }
        
        // Default: access denied
        Ok(AuthorizationResult::Denied("Access denied by default policy".to_string()))
    }
    
    /// Convert a capability to an authorization context
    pub fn capability_to_context(
        &self, 
        capability: &ProgramAccountCapability,
        account_owner: &Address,
    ) -> AuthorizationContext {
        let mut context = AuthorizationContext::new(
            capability.account_id.clone(),
            account_owner.clone(),
            capability.action.clone(),
            AuthorizationLevel::ReadWrite, // Default level
        );
        
        // Add restrictions as parameters
        if let Some(restrictions) = &capability.restrictions {
            context = context.with_parameters(restrictions.clone());
        }
        
        context
    }
    
    /// Validate a capability for an operation
    pub fn validate_capability(
        &self,
        capability: &ProgramAccountCapability,
        account_owner: &Address,
    ) -> Result<AuthorizationResult> {
        // Check expiration
        if let Some(expires_at) = capability.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
                
            if now > expires_at {
                return Ok(AuthorizationResult::Denied("Capability has expired".to_string()));
            }
        }
        
        // Convert to context and authorize
        let context = self.capability_to_context(capability, account_owner);
        self.authorize(&context)
    }
}

/// Extension of the resource system's AuthorizationMethod for program accounts
#[derive(Debug, Clone)]
pub enum ProgramAccountAuthorization {
    /// Direct capability-based authorization
    Capability(ProgramAccountCapability),
    
    /// Role-based authorization
    Role {
        /// Account ID
        account_id: String,
        /// Role required
        role: String,
    },
    
    /// Delegated authorization
    Delegation {
        /// Delegation ID
        delegation_id: String,
    },
    
    /// Integration with the resource system's authorization
    ResourceAuthorization(AuthorizationMethod),
    
    /// Multi-factor authorization (requires all methods to succeed)
    MultiFactor(Vec<ProgramAccountAuthorization>),
    
    /// Any of the provided methods (requires at least one to succeed)
    AnyOf(Vec<ProgramAccountAuthorization>),
}

/// Helper to create authorization predicates from closures
pub fn predicate<F>(f: F) -> AuthorizationPredicate
where
    F: Fn(&AuthorizationContext) -> bool + Send + Sync + 'static,
{
    Arc::new(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_authorization_context() {
        let context = AuthorizationContext::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "transfer".to_string(),
            AuthorizationLevel::ReadWrite,
        )
        .with_resource(ContentId::from_str("res-1"))
        .with_domain(DomainId::new("domain-1"));
        
        assert_eq!(context.account_id, "acc-1");
        assert_eq!(context.account_owner, Address::new("owner-1"));
        assert_eq!(context.action, "transfer");
        assert_eq!(context.level, AuthorizationLevel::ReadWrite);
        assert_eq!(context.resource_id, Some(ContentId::from_str("res-1")));
        assert_eq!(context.domain_id, Some(DomainId::new("domain-1")));
    }
    
    #[test]
    fn test_role_management() {
        let auth_manager = AuthorizationManager::new(None);
        
        // Define a role
        let role = Role {
            name: "admin".to_string(),
            permissions: vec!["create", "read", "update", "delete"].into_iter().map(String::from).collect(),
            domain_permissions: HashMap::new(),
        };
        
        // Register the role
        auth_manager.define_role(role).unwrap();
        
        // Assign the role to an account
        auth_manager.assign_role("acc-1", "admin").unwrap();
        
        // Check permissions
        assert!(auth_manager.has_permission("acc-1", "create", None).unwrap());
        assert!(auth_manager.has_permission("acc-1", "read", None).unwrap());
        assert!(!auth_manager.has_permission("acc-2", "create", None).unwrap());
        
        // Remove the role
        auth_manager.remove_role("acc-1", "admin").unwrap();
        
        // Check permissions again
        assert!(!auth_manager.has_permission("acc-1", "create", None).unwrap());
    }
    
    #[test]
    fn test_delegation() {
        let auth_manager = AuthorizationManager::new(None);
        
        // Create a delegation
        let actions: HashSet<String> = vec!["read", "update"].into_iter().map(String::from).collect();
        let delegation_id = auth_manager.create_delegation(
            "acc-1",
            "acc-2",
            actions,
            None,
            None,
            false,
        ).unwrap();
        
        // Check permissions
        assert!(auth_manager.has_permission("acc-2", "read", None).unwrap());
        assert!(auth_manager.has_permission("acc-2", "update", None).unwrap());
        assert!(!auth_manager.has_permission("acc-2", "delete", None).unwrap());
        
        // Revoke the delegation
        auth_manager.revoke_delegation(&delegation_id).unwrap();
        
        // Check permissions again
        assert!(!auth_manager.has_permission("acc-2", "read", None).unwrap());
    }
    
    #[test]
    fn test_custom_predicate() {
        let auth_manager = AuthorizationManager::new(None);
        
        // Create a custom predicate
        let predicate = predicate(|ctx| {
            ctx.action == "special" && ctx.parameters.get("secret") == Some(&"password123".to_string())
        });
        
        // Register the predicate
        auth_manager.register_predicate("special_action", predicate).unwrap();
        
        // Create a context that should pass
        let mut params = HashMap::new();
        params.insert("secret".to_string(), "password123".to_string());
        
        let passing_context = AuthorizationContext::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "special".to_string(),
            AuthorizationLevel::ReadWrite,
        ).with_parameters(params);
        
        // Create a context that should fail
        let failing_context = AuthorizationContext::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "special".to_string(),
            AuthorizationLevel::ReadWrite,
        );
        
        // Check authorization
        match auth_manager.authorize(&passing_context).unwrap() {
            AuthorizationResult::Allowed => {}, // Success
            _ => panic!("Expected context to be authorized"),
        }
        
        match auth_manager.authorize(&failing_context).unwrap() {
            AuthorizationResult::Denied(_) => {}, // Success
            _ => panic!("Expected context to be denied"),
        }
    }
} 
