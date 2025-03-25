// Account registry
// Original file: src/program_account/registry.rs

// Program Account Registry Implementation
//
// This module provides a registry for program accounts, allowing for
// account creation, lookup, and management of available effects.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::domain::DomainId;
use causality_types::{Error, Result};
use causality_types::Address;
use crate::program_account::{
    ProgramAccount, ProgramAccountRegistry, AvailableEffect
};
use causality_resource::BaseAccount;
use causality_resource::AssetAccount;
use causality_resource::UtilityAccount;
use crate::program_account::domain_bridge_account::DomainBridgeAccount;

/// The types of program accounts that can be created
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// Basic account
    Basic,
    /// Asset account for tokens/NFTs
    Asset,
    /// Utility account for data storage
    Utility,
    /// Domain bridge account
    DomainBridge,
}

impl AccountType {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "basic" => Some(AccountType::Basic),
            "asset" => Some(AccountType::Asset),
            "utility" => Some(AccountType::Utility),
            "domain_bridge" => Some(AccountType::DomainBridge),
            _ => None,
        }
    }
    
    /// Convert to string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            AccountType::Basic => "basic",
            AccountType::Asset => "asset",
            AccountType::Utility => "utility",
            AccountType::DomainBridge => "domain_bridge",
        }
    }
}

/// A registry for program accounts
pub struct StandardProgramAccountRegistry {
    /// Program accounts in the registry
    accounts: RwLock<HashMap<String, Arc<RwLock<dyn ProgramAccount + Send + Sync>>>>,
    
    /// Mapping from owner to account IDs
    owner_to_accounts: RwLock<HashMap<String, HashSet<String>>>,
    
    /// Available effects by domain
    domain_effects: RwLock<HashMap<DomainId, HashMap<String, AvailableEffect>>>,
    
    /// Registered domains
    domains: RwLock<HashSet<DomainId>>,
    
    /// Counter for generating unique IDs
    id_counter: RwLock<u64>,
}

impl StandardProgramAccountRegistry {
    /// Create a new program account registry
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
            owner_to_accounts: RwLock::new(HashMap::new()),
            domain_effects: RwLock::new(HashMap::new()),
            domains: RwLock::new(HashSet::new()),
            id_counter: RwLock::new(0),
        }
    }
    
    /// Generate a unique ID
    fn generate_id(&self, prefix: &str) -> Result<String> {
        let mut counter = self.id_counter.write().map_err(|_| Error::LockError)?;
        let id = *counter;
        *counter += 1;
        Ok(format!("{}-{}", prefix, id))
    }
    
    /// Create a specific type of account
    pub fn create_specific_account(
        &self,
        account_type: AccountType,
        owner: Address,
        name: String,
        initial_domains: Option<HashSet<DomainId>>,
    ) -> Result<Box<dyn ProgramAccount>> {
        // Generate a unique ID for the account
        let account_id = self.generate_id("acc")?;
        
        // Ensure initial domains are registered
        if let Some(domains) = &initial_domains {
            let registered_domains = self.domains.read().map_err(|_| Error::LockError)?;
            for domain in domains {
                if !registered_domains.contains(domain) {
                    return Err(Error::NotFound(format!("Domain not registered: {}", domain)));
                }
            }
        }
        
        // Create the account based on the specified type
        let account: Arc<RwLock<dyn ProgramAccount + Send + Sync>> = match account_type {
            AccountType::Basic => {
                let account = BaseAccount::new(
                    account_id.clone(),
                    owner.clone(),
                    name,
                    "basic".to_string(),
                    initial_domains.clone(),
                );
                Arc::new(RwLock::new(account))
            },
            AccountType::Asset => {
                let account = AssetAccount::new(
                    account_id.clone(),
                    owner.clone(),
                    name,
                    initial_domains.clone(),
                );
                Arc::new(RwLock::new(account))
            },
            AccountType::Utility => {
                let account = UtilityAccount::new(
                    account_id.clone(),
                    owner.clone(),
                    name,
                    initial_domains.clone(),
                );
                Arc::new(RwLock::new(account))
            },
            AccountType::DomainBridge => {
                let account = DomainBridgeAccount::new(
                    account_id.clone(),
                    owner.clone(),
                    name,
                    initial_domains.clone(),
                );
                Arc::new(RwLock::new(account))
            },
        };
        
        // Register available effects for initial domains
        if let Some(domains) = &initial_domains {
            let domain_effects = self.domain_effects.read().map_err(|_| Error::LockError)?;
            let mut account_write = account.write().map_err(|_| Error::LockError)?;
            
            for domain in domains {
                if let Some(effects) = domain_effects.get(domain) {
                    for effect in effects.values() {
                        // In a real implementation, we would use a trait method here
                        // but for now, we can't directly access method due to the dynamic typing
                        if let Some(base_account) = account_write.downcast_mut::<BaseAccount>() {
                            base_account.register_effect(effect.clone())?;
                        }
                    }
                }
            }
        }
        
        // Store the account in the registry
        {
            let mut accounts = self.accounts.write().map_err(|_| Error::LockError)?;
            accounts.insert(account_id.clone(), account.clone());
        }
        
        // Update the owner-to-accounts mapping
        {
            let mut owner_to_accounts = self.owner_to_accounts.write().map_err(|_| Error::LockError)?;
            let owner_accounts = owner_to_accounts
                .entry(owner.to_string())
                .or_insert_with(HashSet::new);
            owner_accounts.insert(account_id.clone());
        }
        
        // Create a read guard for the account
        let account_read = account.read().map_err(|_| Error::LockError)?;
        
        // Use dynamic dispatch to return the account as a trait object
        Ok(Box::new(AccountWrapper::new(&*account_read)))
    }
}

impl ProgramAccountRegistry for StandardProgramAccountRegistry {
    fn create_account(
        &self,
        owner: Address,
        name: String,
        account_type: String,
        initial_domains: Option<HashSet<DomainId>>,
    ) -> Result<Box<dyn ProgramAccount>> {
        // Parse the account type
        let parsed_type = AccountType::from_str(&account_type)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid account type: {}", account_type)))?;
        
        // Create the account
        self.create_specific_account(parsed_type, owner, name, initial_domains)
    }
    
    fn get_account(&self, account_id: &str) -> Result<Option<Box<dyn ProgramAccount>>> {
        let accounts = self.accounts.read().map_err(|_| Error::LockError)?;
        
        if let Some(wrapped_account) = accounts.get(account_id) {
            let account_read = wrapped_account.read().map_err(|_| Error::LockError)?;
            Ok(Some(Box::new(AccountWrapper::new(&*account_read))))
        } else {
            Ok(None)
        }
    }
    
    fn get_accounts_for_owner(&self, owner: &Address) -> Result<Vec<Box<dyn ProgramAccount>>> {
        let owner_to_accounts = self.owner_to_accounts.read().map_err(|_| Error::LockError)?;
        let accounts = self.accounts.read().map_err(|_| Error::LockError)?;
        
        let owner_accounts = match owner_to_accounts.get(&owner.to_string()) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        let mut result = Vec::new();
        for account_id in owner_accounts {
            if let Some(wrapped_account) = accounts.get(account_id) {
                let account_read = wrapped_account.read().map_err(|_| Error::LockError)?;
                result.push(Box::new(AccountWrapper::new(&*account_read)));
            }
        }
        
        Ok(result)
    }
    
    fn register_effect(&self, effect: AvailableEffect) -> Result<()> {
        let domain_id = effect.domain_id.clone().ok_or_else(|| {
            Error::InvalidArgument("Effect must have a domain ID".to_string())
        })?;
        
        // Ensure the domain is registered
        {
            let domains = self.domains.read().map_err(|_| Error::LockError)?;
            if !domains.contains(&domain_id) {
                return Err(Error::NotFound(format!("Domain not registered: {}", domain_id)));
            }
        }
        
        // Register the effect
        {
            let mut domain_effects = self.domain_effects.write().map_err(|_| Error::LockError)?;
            let effects = domain_effects
                .entry(domain_id.clone())
                .or_insert_with(HashMap::new);
            effects.insert(effect.id.clone(), effect.clone());
        }
        
        // Add the effect to all accounts that have access to this domain
        {
            let accounts = self.accounts.read().map_err(|_| Error::LockError)?;
            for wrapped_account in accounts.values() {
                let mut account = wrapped_account.write().map_err(|_| Error::LockError)?;
                
                // Check if the account has access to the domain
                // This is a bit hacky since we can't directly access the domains() method
                // In a real implementation, we would use a trait method for this
                if let Some(base_account) = account.downcast_mut::<BaseAccount>() {
                    if base_account.domains().contains(&domain_id) {
                        base_account.register_effect(effect.clone())?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn get_effects_for_domain(&self, domain_id: &DomainId) -> Result<Vec<AvailableEffect>> {
        let domain_effects = self.domain_effects.read().map_err(|_| Error::LockError)?;
        
        if let Some(effects) = domain_effects.get(domain_id) {
            Ok(effects.values().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn register_domain(&self, domain_id: DomainId) -> Result<()> {
        let mut domains = self.domains.write().map_err(|_| Error::LockError)?;
        domains.insert(domain_id);
        Ok(())
    }
    
    fn get_domains(&self) -> Result<HashSet<DomainId>> {
        let domains = self.domains.read().map_err(|_| Error::LockError)?;
        Ok(domains.clone())
    }
}

/// A wrapper around a ProgramAccount for dynamic dispatch
struct AccountWrapper<'a> {
    account: &'a dyn ProgramAccount,
}

impl<'a> AccountWrapper<'a> {
    fn new(account: &'a dyn ProgramAccount) -> Self {
        Self { account }
    }
}

impl<'a> ProgramAccount for AccountWrapper<'a> {
    fn id(&self) -> &str {
        self.account.id()
    }
    
    fn owner(&self) -> &Address {
        self.account.owner()
    }
    
    fn name(&self) -> &str {
        self.account.name()
    }
    
    fn account_type(&self) -> &str {
        self.account.account_type()
    }
    
    fn domains(&self) -> &HashSet<DomainId> {
        self.account.domains()
    }
    
    fn resources(&self) -> Vec<crate::program_account::ProgramAccountResource> {
        self.account.resources()
    }
    
    fn get_resource(&self, resource_id: &crate::resource::ContentId) -> Result<Option<crate::program_account::ProgramAccountResource>> {
        self.account.get_resource(resource_id)
    }
    
    fn available_effects(&self) -> Vec<crate::program_account::AvailableEffect> {
        self.account.available_effects()
    }
    
    fn get_effect(&self, effect_id: &str) -> Result<Option<crate::program_account::AvailableEffect>> {
        self.account.get_effect(effect_id)
    }
    
    fn execute_effect(
        &self,
        effect_id: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&causality_types::TraceId>,
    ) -> Result<crate::program_account::EffectResult> {
        self.account.execute_effect(effect_id, parameters, trace_id)
    }
    
    fn capabilities(&self) -> Vec<crate::program_account::ProgramAccountCapability> {
        self.account.capabilities()
    }
    
    fn has_capability(&self, action: &str) -> bool {
        self.account.has_capability(action)
    }
    
    fn grant_capability(&mut self, capability: crate::program_account::ProgramAccountCapability) -> Result<()> {
        Err(Error::PermissionDenied("Cannot grant capability through read-only account wrapper".to_string()))
    }
    
    fn revoke_capability(&mut self, capability_id: &str) -> Result<()> {
        Err(Error::PermissionDenied("Cannot revoke capability through read-only account wrapper".to_string()))
    }
    
    fn get_balance(&self, asset_id: &str) -> Result<u64> {
        self.account.get_balance(asset_id)
    }
    
    fn get_all_balances(&self) -> Result<HashMap<String, u64>> {
        self.account.get_all_balances()
    }
    
    fn transaction_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<crate::program_account::TransactionRecord>> {
        self.account.transaction_history(limit, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = StandardProgramAccountRegistry::new();
        assert!(registry.get_domains().unwrap().is_empty());
    }
    
    #[test]
    fn test_domain_registration() {
        let registry = StandardProgramAccountRegistry::new();
        let domain_id = DomainId::new("test-domain");
        
        registry.register_domain(domain_id.clone()).unwrap();
        
        let domains = registry.get_domains().unwrap();
        assert_eq!(domains.len(), 1);
        assert!(domains.contains(&domain_id));
    }
    
    #[test]
    fn test_basic_account_creation() {
        let registry = StandardProgramAccountRegistry::new();
        let domain_id = DomainId::new("test-domain");
        registry.register_domain(domain_id.clone()).unwrap();
        
        let mut initial_domains = HashSet::new();
        initial_domains.insert(domain_id.clone());
        
        let account = registry.create_account(
            Address::new("owner-1"),
            "Test Account".to_string(),
            "basic".to_string(),
            Some(initial_domains),
        ).unwrap();
        
        assert_eq!(account.name(), "Test Account");
        assert_eq!(account.owner().to_string(), "owner-1");
        assert_eq!(account.account_type(), "basic");
        assert!(account.domains().contains(&domain_id));
    }
    
    #[test]
    fn test_specialized_account_creation() {
        let registry = StandardProgramAccountRegistry::new();
        let domain_id = DomainId::new("test-domain");
        registry.register_domain(domain_id.clone()).unwrap();
        
        let mut initial_domains = HashSet::new();
        initial_domains.insert(domain_id.clone());
        
        // Create an asset account
        let asset_account = registry.create_specific_account(
            AccountType::Asset,
            Address::new("owner-1"),
            "Asset Account".to_string(),
            Some(initial_domains.clone()),
        ).unwrap();
        
        assert_eq!(asset_account.name(), "Asset Account");
        assert_eq!(asset_account.account_type(), "asset");
        
        // Create a utility account
        let utility_account = registry.create_specific_account(
            AccountType::Utility,
            Address::new("owner-1"),
            "Utility Account".to_string(),
            Some(initial_domains.clone()),
        ).unwrap();
        
        assert_eq!(utility_account.name(), "Utility Account");
        assert_eq!(utility_account.account_type(), "utility");
        
        // Create a domain bridge account
        let bridge_account = registry.create_specific_account(
            AccountType::DomainBridge,
            Address::new("owner-1"),
            "Bridge Account".to_string(),
            Some(initial_domains.clone()),
        ).unwrap();
        
        assert_eq!(bridge_account.name(), "Bridge Account");
        assert_eq!(bridge_account.account_type(), "domain_bridge");
    }
    
    #[test]
    fn test_effect_registration() {
        let registry = StandardProgramAccountRegistry::new();
        let domain_id = DomainId::new("test-domain");
        registry.register_domain(domain_id.clone()).unwrap();
        
        let effect = AvailableEffect {
            id: "effect-1".to_string(),
            name: "Test Effect".to_string(),
            description: "A test effect".to_string(),
            domain_id: Some(domain_id.clone()),
            parameters: Vec::new(),
            requires_authorization: false,
        };
        
        registry.register_effect(effect.clone()).unwrap();
        
        let effects = registry.get_effects_for_domain(&domain_id).unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].id, "effect-1");
    }
    
    #[test]
    fn test_account_lookup() {
        let registry = StandardProgramAccountRegistry::new();
        let domain_id = DomainId::new("test-domain");
        registry.register_domain(domain_id.clone()).unwrap();
        
        let mut initial_domains = HashSet::new();
        initial_domains.insert(domain_id.clone());
        
        let owner = Address::new("owner-1");
        let account = registry.create_account(
            owner.clone(),
            "Test Account".to_string(),
            "basic".to_string(),
            Some(initial_domains),
        ).unwrap();
        
        let account_id = account.id();
        
        // Look up by ID
        let looked_up = registry.get_account(account_id).unwrap().unwrap();
        assert_eq!(looked_up.name(), "Test Account");
        
        // Look up by owner
        let owner_accounts = registry.get_accounts_for_owner(&owner).unwrap();
        assert_eq!(owner_accounts.len(), 1);
        assert_eq!(owner_accounts[0].name(), "Test Account");
    }
} 
