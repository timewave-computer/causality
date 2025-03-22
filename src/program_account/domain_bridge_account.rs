// Domain Bridge Program Account Implementation
//
// This module provides a specialized implementation of the ProgramAccount trait
// for cross-domain operations and bridging resources between domains.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::domain::DomainId;
use crate::error::{Error, Result};
use crate::resource::{RegisterId, RegisterContents, Register, ResourceId};
use crate::types::{Address, TraceId};
use crate::program_account::{
    ProgramAccount, DomainBridgeProgramAccount, ProgramAccountCapability, 
    ProgramAccountResource, AvailableEffect, EffectResult, EffectStatus,
    TransactionRecord, TransactionStatus, CrossDomainTransfer, TransferStatus
};
use crate::program_account::base_account::BaseAccount;

/// A specialized implementation of the ProgramAccount trait for domain bridging operations
pub struct DomainBridgeAccount {
    /// The base account implementation
    base: BaseAccount,
    
    /// Pending cross-domain transfers
    pending_transfers: RwLock<HashMap<String, CrossDomainTransfer>>,
    
    /// Completed cross-domain transfers
    completed_transfers: RwLock<Vec<CrossDomainTransfer>>,
    
    /// Domain adapter registry (would connect to domain-specific adapters in a real implementation)
    domain_adapters: RwLock<HashMap<DomainId, ()>>, // Placeholder for actual adapter implementations
}

impl DomainBridgeAccount {
    /// Create a new domain bridge account
    pub fn new(
        id: String,
        owner: Address,
        name: String,
        initial_domains: Option<HashSet<DomainId>>,
    ) -> Self {
        Self {
            base: BaseAccount::new(
                id,
                owner,
                name,
                "domain_bridge".to_string(),
                initial_domains,
            ),
            pending_transfers: RwLock::new(HashMap::new()),
            completed_transfers: RwLock::new(Vec::new()),
            domain_adapters: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a domain adapter
    pub fn register_domain_adapter(&self, domain_id: DomainId) -> Result<()> {
        let mut adapters = self.domain_adapters.write().map_err(|_| Error::LockError)?;
        
        // In a real implementation, this would create or connect to a domain-specific adapter
        adapters.insert(domain_id, ());
        
        Ok(())
    }
    
    /// Check if a domain adapter is registered
    pub fn has_domain_adapter(&self, domain_id: &DomainId) -> Result<bool> {
        let adapters = self.domain_adapters.read().map_err(|_| Error::LockError)?;
        Ok(adapters.contains_key(domain_id))
    }
    
    /// Get a transfer by ID
    pub fn get_transfer(&self, transfer_id: &str) -> Result<Option<CrossDomainTransfer>> {
        // Check pending transfers
        {
            let pending = self.pending_transfers.read().map_err(|_| Error::LockError)?;
            if let Some(transfer) = pending.get(transfer_id) {
                return Ok(Some(transfer.clone()));
            }
        }
        
        // Check completed transfers
        {
            let completed = self.completed_transfers.read().map_err(|_| Error::LockError)?;
            for transfer in completed.iter() {
                if transfer.id == transfer_id {
                    return Ok(Some(transfer.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Complete a pending transfer
    fn complete_transfer(&self, transfer_id: &str) -> Result<CrossDomainTransfer> {
        let transfer = {
            let mut pending = self.pending_transfers.write().map_err(|_| Error::LockError)?;
            
            let mut transfer = pending.remove(transfer_id)
                .ok_or_else(|| Error::NotFound(format!("Transfer not found: {}", transfer_id)))?;
            
            // Update the transfer status and completion time
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs());
            
            transfer
        };
        
        // Add to completed transfers
        let mut completed = self.completed_transfers.write().map_err(|_| Error::LockError)?;
        completed.push(transfer.clone());
        
        Ok(transfer)
    }
    
    /// Update a transfer's status
    fn update_transfer_status(&self, transfer_id: &str, status: TransferStatus) -> Result<()> {
        let mut pending = self.pending_transfers.write().map_err(|_| Error::LockError)?;
        
        let transfer = pending.get_mut(transfer_id)
            .ok_or_else(|| Error::NotFound(format!("Transfer not found: {}", transfer_id)))?;
        
        transfer.status = status;
        
        Ok(())
    }
}

impl ProgramAccount for DomainBridgeAccount {
    fn id(&self) -> &str {
        self.base.id()
    }
    
    fn owner(&self) -> &Address {
        self.base.owner()
    }
    
    fn name(&self) -> &str {
        self.base.name()
    }
    
    fn account_type(&self) -> &str {
        self.base.account_type()
    }
    
    fn domains(&self) -> &HashSet<DomainId> {
        self.base.domains()
    }
    
    fn resources(&self) -> Vec<ProgramAccountResource> {
        self.base.resources()
    }
    
    fn get_resource(&self, resource_id: &ResourceId) -> Result<Option<ProgramAccountResource>> {
        self.base.get_resource(resource_id)
    }
    
    fn available_effects(&self) -> Vec<AvailableEffect> {
        self.base.available_effects()
    }
    
    fn get_effect(&self, effect_id: &str) -> Result<Option<AvailableEffect>> {
        self.base.get_effect(effect_id)
    }
    
    fn execute_effect(
        &self,
        effect_id: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        self.base.execute_effect(effect_id, parameters, trace_id)
    }
    
    fn capabilities(&self) -> Vec<ProgramAccountCapability> {
        self.base.capabilities()
    }
    
    fn has_capability(&self, action: &str) -> bool {
        self.base.has_capability(action)
    }
    
    fn grant_capability(&mut self, capability: ProgramAccountCapability) -> Result<()> {
        self.base.grant_capability(capability)
    }
    
    fn revoke_capability(&mut self, capability_id: &str) -> Result<()> {
        self.base.revoke_capability(capability_id)
    }
    
    fn get_balance(&self, asset_id: &str) -> Result<u64> {
        self.base.get_balance(asset_id)
    }
    
    fn get_all_balances(&self) -> Result<HashMap<String, u64>> {
        self.base.get_all_balances()
    }
    
    fn transaction_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<TransactionRecord>> {
        self.base.transaction_history(limit, offset)
    }
}

impl DomainBridgeProgramAccount for DomainBridgeAccount {
    fn transfer_to_domain(
        &self,
        resource_id: &ResourceId,
        target_domain: &DomainId,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        // Ensure we have the resource
        let resource = self.get_resource(resource_id)?
            .ok_or_else(|| Error::NotFound(format!("Resource not found: {}", resource_id)))?;
        
        // Ensure we have access to the source domain
        let source_domain = resource.domain_id
            .clone()
            .ok_or_else(|| Error::InvalidArgument("Resource does not have a domain".to_string()))?;
        
        if !self.domains().contains(&source_domain) {
            return Err(Error::PermissionDenied(format!(
                "Account does not have access to source domain: {}", source_domain
            )));
        }
        
        // Ensure we have access to the target domain
        if !self.domains().contains(target_domain) {
            return Err(Error::PermissionDenied(format!(
                "Account does not have access to target domain: {}", target_domain
            )));
        }
        
        // Ensure we have adapters for both domains
        if !self.has_domain_adapter(&source_domain)? {
            return Err(Error::InvalidState(format!(
                "No adapter registered for source domain: {}", source_domain
            )));
        }
        
        if !self.has_domain_adapter(target_domain)? {
            return Err(Error::InvalidState(format!(
                "No adapter registered for target domain: {}", target_domain
            )));
        }
        
        // Create a new transfer ID
        let transfer_id = format!("transfer-{}", Uuid::new_v4());
        
        // Get the current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create a cross-domain transfer
        let transfer = CrossDomainTransfer {
            id: transfer_id.clone(),
            resource_id: resource_id.clone(),
            source_domain,
            target_domain: target_domain.clone(),
            status: TransferStatus::Initiated,
            initiated_at: timestamp,
            completed_at: None,
            error: None,
            proof: None,
        };
        
        // Store the pending transfer
        {
            let mut pending = self.pending_transfers.write().map_err(|_| Error::LockError)?;
            pending.insert(transfer_id.clone(), transfer.clone());
        }
        
        // In a real implementation, this would:
        // 1. Lock the resource in the source domain
        // 2. Generate a proof of the lock
        // 3. Submit the proof to the target domain
        // 4. Create the resource in the target domain
        // 5. Update the transfer status at each step
        
        // For this example, we'll simulate the process
        self.update_transfer_status(&transfer_id, TransferStatus::SourceLocked)?;
        self.update_transfer_status(&transfer_id, TransferStatus::ProofGenerated)?;
        self.update_transfer_status(&transfer_id, TransferStatus::ProofSubmitted)?;
        self.update_transfer_status(&transfer_id, TransferStatus::TargetCreated)?;
        
        // Complete the transfer
        let completed_transfer = self.complete_transfer(&transfer_id)?;
        
        // Create a new resource in the target domain
        let target_resource_id = ResourceId::from_str(&format!("bridged-{}", Uuid::new_v4()));
        
        // Create a ProgramAccountResource for the bridged resource
        let bridged_resource = ProgramAccountResource {
            id: target_resource_id.clone(),
            register_id: None, // In a real implementation, this would be set
            resource_type: resource.resource_type.clone(),
            domain_id: Some(target_domain.clone()),
            metadata: {
                let mut meta = resource.metadata.clone();
                meta.insert("source_resource".to_string(), resource_id.to_string());
                meta.insert("source_domain".to_string(), source_domain.to_string());
                meta.insert("transfer_id".to_string(), transfer_id.clone());
                meta
            },
        };
        
        // Register the resource with the base account
        self.base.register_resource(bridged_resource.clone())?;
        
        // Add a transaction record
        let record = TransactionRecord {
            id: transfer_id.clone(),
            transaction_type: "domain_transfer".to_string(),
            timestamp,
            status: TransactionStatus::Confirmed,
            resources: vec![resource_id.clone(), target_resource_id.clone()],
            effects: Vec::new(),
            domains: vec![source_domain.clone(), target_domain.clone()],
            metadata: parameters,
        };
        
        self.base.add_transaction_record(record)?;
        
        // Create a result
        let result = EffectResult {
            id: transfer_id,
            status: EffectStatus::Completed,
            transaction_id: trace_id.map(|id| id.to_string()),
            new_resources: vec![bridged_resource],
            modified_resources: vec![resource],
            consumed_resources: Vec::new(),
            outputs: HashMap::from([
                ("target_resource_id".to_string(), target_resource_id.to_string()),
                ("status".to_string(), "completed".to_string()),
            ]),
            error: None,
        };
        
        Ok(result)
    }
    
    fn import_from_domain(
        &self,
        source_domain: &DomainId,
        resource_reference: &str,
        parameters: HashMap<String, String>,
        trace_id: Option<&TraceId>,
    ) -> Result<EffectResult> {
        // Ensure we have access to the source domain
        if !self.domains().contains(source_domain) {
            return Err(Error::PermissionDenied(format!(
                "Account does not have access to source domain: {}", source_domain
            )));
        }
        
        // Get the target domain from parameters
        let target_domain_str = parameters.get("target_domain")
            .ok_or_else(|| Error::InvalidArgument("Missing target_domain parameter".to_string()))?;
        
        let target_domain = DomainId::new(target_domain_str);
        
        // Ensure we have access to the target domain
        if !self.domains().contains(&target_domain) {
            return Err(Error::PermissionDenied(format!(
                "Account does not have access to target domain: {}", target_domain
            )));
        }
        
        // Ensure we have adapters for both domains
        if !self.has_domain_adapter(source_domain)? {
            return Err(Error::InvalidState(format!(
                "No adapter registered for source domain: {}", source_domain
            )));
        }
        
        if !self.has_domain_adapter(&target_domain)? {
            return Err(Error::InvalidState(format!(
                "No adapter registered for target domain: {}", target_domain
            )));
        }
        
        // Create a new transfer ID
        let transfer_id = format!("import-{}", Uuid::new_v4());
        
        // Get the current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // In a real implementation, this would:
        // 1. Query the source domain for the resource
        // 2. Verify the resource exists and is available
        // 3. Lock the resource in the source domain
        // 4. Generate a proof of the lock
        // 5. Create the resource in the target domain
        
        // Create a resource ID from the reference
        let source_resource_id = ResourceId::from_str(resource_reference);
        let target_resource_id = ResourceId::from_str(&format!("imported-{}", Uuid::new_v4()));
        
        // Create a cross-domain transfer
        let transfer = CrossDomainTransfer {
            id: transfer_id.clone(),
            resource_id: source_resource_id.clone(),
            source_domain: source_domain.clone(),
            target_domain: target_domain.clone(),
            status: TransferStatus::Initiated,
            initiated_at: timestamp,
            completed_at: None,
            error: None,
            proof: None,
        };
        
        // Store the pending transfer
        {
            let mut pending = self.pending_transfers.write().map_err(|_| Error::LockError)?;
            pending.insert(transfer_id.clone(), transfer.clone());
        }
        
        // Simulate the import process
        self.update_transfer_status(&transfer_id, TransferStatus::SourceLocked)?;
        self.update_transfer_status(&transfer_id, TransferStatus::ProofGenerated)?;
        self.update_transfer_status(&transfer_id, TransferStatus::ProofSubmitted)?;
        self.update_transfer_status(&transfer_id, TransferStatus::TargetCreated)?;
        
        // Complete the transfer
        let completed_transfer = self.complete_transfer(&transfer_id)?;
        
        // Get resource type from parameters
        let resource_type = parameters.get("resource_type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        
        // Create a ProgramAccountResource for the imported resource
        let imported_resource = ProgramAccountResource {
            id: target_resource_id.clone(),
            register_id: None, // In a real implementation, this would be set
            resource_type,
            domain_id: Some(target_domain.clone()),
            metadata: {
                let mut meta = parameters.clone();
                meta.insert("source_resource".to_string(), resource_reference.to_string());
                meta.insert("source_domain".to_string(), source_domain.to_string());
                meta.insert("transfer_id".to_string(), transfer_id.clone());
                meta
            },
        };
        
        // Register the resource with the base account
        self.base.register_resource(imported_resource.clone())?;
        
        // Add a transaction record
        let record = TransactionRecord {
            id: transfer_id.clone(),
            transaction_type: "domain_import".to_string(),
            timestamp,
            status: TransactionStatus::Confirmed,
            resources: vec![target_resource_id.clone()],
            effects: Vec::new(),
            domains: vec![source_domain.clone(), target_domain.clone()],
            metadata: parameters,
        };
        
        self.base.add_transaction_record(record)?;
        
        // Create a result
        let result = EffectResult {
            id: transfer_id,
            status: EffectStatus::Completed,
            transaction_id: trace_id.map(|id| id.to_string()),
            new_resources: vec![imported_resource],
            modified_resources: Vec::new(),
            consumed_resources: Vec::new(),
            outputs: HashMap::from([
                ("target_resource_id".to_string(), target_resource_id.to_string()),
                ("status".to_string(), "completed".to_string()),
            ]),
            error: None,
        };
        
        Ok(result)
    }
    
    fn pending_transfers(&self) -> Result<Vec<CrossDomainTransfer>> {
        let pending = self.pending_transfers.read().map_err(|_| Error::LockError)?;
        Ok(pending.values().cloned().collect())
    }
    
    fn transfer_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<CrossDomainTransfer>> {
        let completed = self.completed_transfers.read().map_err(|_| Error::LockError)?;
        
        let offset = offset.unwrap_or(0);
        if offset >= completed.len() {
            return Ok(Vec::new());
        }
        
        let limit = limit.unwrap_or(completed.len());
        let end = std::cmp::min(offset + limit, completed.len());
        
        Ok(completed[offset..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_bridge_account_creation() {
        let account = DomainBridgeAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Domain Bridge Account".to_string(),
            None,
        );
        
        assert_eq!(account.id(), "acc-1");
        assert_eq!(account.owner().to_string(), "owner-1");
        assert_eq!(account.name(), "Domain Bridge Account");
        assert_eq!(account.account_type(), "domain_bridge");
    }
    
    #[test]
    fn test_domain_adapter_registration() {
        let account = DomainBridgeAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Domain Bridge Account".to_string(),
            None,
        );
        
        let domain = DomainId::new("test-domain");
        
        // Initially, no adapter is registered
        assert!(!account.has_domain_adapter(&domain).unwrap());
        
        // Register an adapter
        account.register_domain_adapter(domain.clone()).unwrap();
        
        // Now there should be an adapter
        assert!(account.has_domain_adapter(&domain).unwrap());
    }
    
    #[test]
    fn test_transfer_validation() {
        let mut domains = HashSet::new();
        let source_domain = DomainId::new("source-domain");
        let target_domain = DomainId::new("target-domain");
        domains.insert(source_domain.clone());
        domains.insert(target_domain.clone());
        
        let account = DomainBridgeAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Domain Bridge Account".to_string(),
            Some(domains),
        );
        
        // Register domain adapters
        account.register_domain_adapter(source_domain.clone()).unwrap();
        account.register_domain_adapter(target_domain.clone()).unwrap();
        
        // Create a resource in the source domain
        let resource_id = ResourceId::from_str("resource-1");
        let resource = ProgramAccountResource {
            id: resource_id.clone(),
            register_id: None,
            resource_type: "test".to_string(),
            domain_id: Some(source_domain.clone()),
            metadata: HashMap::new(),
        };
        
        account.base.register_resource(resource).unwrap();
        
        // Perform a transfer
        let result = account.transfer_to_domain(
            &resource_id,
            &target_domain,
            HashMap::new(),
            None,
        ).unwrap();
        
        assert_eq!(result.status, EffectStatus::Completed);
        assert_eq!(result.new_resources.len(), 1);
        
        // Verify the transfer is in the history
        let history = account.transfer_history(None, None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, TransferStatus::Completed);
        assert_eq!(history[0].source_domain, source_domain);
        assert_eq!(history[0].target_domain, target_domain);
    }
    
    #[test]
    fn test_import_from_domain() {
        let mut domains = HashSet::new();
        let source_domain = DomainId::new("source-domain");
        let target_domain = DomainId::new("target-domain");
        domains.insert(source_domain.clone());
        domains.insert(target_domain.clone());
        
        let account = DomainBridgeAccount::new(
            "acc-1".to_string(),
            Address::new("owner-1"),
            "Domain Bridge Account".to_string(),
            Some(domains),
        );
        
        // Register domain adapters
        account.register_domain_adapter(source_domain.clone()).unwrap();
        account.register_domain_adapter(target_domain.clone()).unwrap();
        
        // Perform an import
        let mut parameters = HashMap::new();
        parameters.insert("target_domain".to_string(), target_domain.to_string());
        parameters.insert("resource_type".to_string(), "token".to_string());
        
        let result = account.import_from_domain(
            &source_domain,
            "external-resource-1",
            parameters,
            None,
        ).unwrap();
        
        assert_eq!(result.status, EffectStatus::Completed);
        assert_eq!(result.new_resources.len(), 1);
        assert_eq!(result.new_resources[0].resource_type, "token");
        
        // Verify the import is in the history
        let history = account.transfer_history(None, None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, TransferStatus::Completed);
        assert_eq!(history[0].source_domain, source_domain);
        assert_eq!(history[0].target_domain, target_domain);
    }
} 