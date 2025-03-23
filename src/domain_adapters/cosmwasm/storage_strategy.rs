// CosmWasm-specific storage strategies for ResourceRegister
//
// This module implements domain-specific storage strategies for CosmWasm chains,
// as part of the unified Resource-Register model.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::address::Address;
use crate::domain::DomainId;
use crate::resource::{ResourceId, ResourceRegister};
use crate::resource::resource_register::{
    StorageStrategy, 
    StateVisibility, 
    NullifierId
};
use crate::crypto::merkle::Commitment;
use crate::effect::{
    Effect, 
    EffectId,
    EffectContext,
    EffectResult,
    EffectOutcome,
    EffectError,
    ExecutionBoundary
};
use crate::effect::storage::{
    StoreResult, 
    ReadResult, 
    StoreOnChainEffect, 
    ReadFromChainEffect, 
    StoreCommitmentEffect
};
use crate::error::{Error, Result};
use crate::log::fact_snapshot::{FactSnapshot, FactDependency};

use super::types::{
    CosmWasmAddress,
    Coin
};
use super::adapter::{
    CosmWasmAdapterConfig,
    CosmWasmExecuteEffect
};

/// CosmWasm storage effect for storing register data on chain
#[derive(Debug, Clone)]
pub struct CosmWasmStoreEffect {
    /// Unique identifier
    id: EffectId,
    /// Resource register ID to store
    register_id: ResourceId,
    /// Fields to include in the store operation
    fields: HashSet<String>,
    /// Domain ID
    domain_id: DomainId,
    /// Address of the invoker
    invoker: Address,
    /// Contract address for storage
    contract_address: CosmWasmAddress,
    /// Fact dependencies
    fact_deps: Vec<FactDependency>,
    /// Fact snapshot
    snapshot: Option<FactSnapshot>,
}

impl CosmWasmStoreEffect {
    /// Create a new CosmWasm store effect
    pub fn new(
        register_id: ResourceId,
        fields: HashSet<String>,
        domain_id: DomainId,
        invoker: Address,
        contract_address: impl Into<CosmWasmAddress>,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            register_id,
            fields,
            domain_id,
            invoker,
            contract_address: contract_address.into(),
            fact_deps: Vec::new(),
            snapshot: None,
        }
    }
    
    /// Get the register ID
    pub fn register_id(&self) -> &ResourceId {
        &self.register_id
    }
    
    /// Add a fact dependency
    pub fn with_fact_dependency(mut self, dependency: FactDependency) -> Self {
        self.fact_deps.push(dependency);
        self
    }
    
    /// Set the fact snapshot
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.snapshot = Some(snapshot);
        self
    }
}

#[async_trait]
impl Effect for CosmWasmStoreEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }

    fn name(&self) -> &str {
        "cosmwasm_store_on_chain"
    }
    
    fn display_name(&self) -> String {
        format!("Store on CosmWasm chain: {}", self.register_id)
    }
    
    fn description(&self) -> String {
        format!("Store register data on CosmWasm contract {}", self.contract_address)
    }
    
    fn execute(&self, _context: &EffectContext) -> Result<EffectOutcome> {
        // Synchronous execution is not supported
        Err(Error::OperationNotSupported("CosmWasm storage requires async context".into()))
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get the register from the context
        let register = context.get_resource(&self.register_id)
            .ok_or_else(|| EffectError::ResourceNotFound(self.register_id.clone()))?;
        
        // Create the execute message for storing the register
        let message = json!({
            "store_register": {
                "register_id": register.id.to_string(),
                "fields": self.fields.iter().collect::<Vec<_>>(),
                "data": register.data,
                "visibility": match register.storage_strategy {
                    StorageStrategy::FullyOnChain { visibility: StateVisibility::Public } => "public",
                    StorageStrategy::FullyOnChain { visibility: StateVisibility::Private } => "private",
                    StorageStrategy::FullyOnChain { visibility: StateVisibility::Permissioned(_) } => "permissioned",
                    _ => "public", // Default
                }
            }
        });
        
        // Create the execute effect
        let execute_effect = CosmWasmExecuteEffect::new(
            self.contract_address.0.clone(),
            message
        );
        
        // Execute the effect
        let outcome = execute_effect.execute_async(context).await?;
        
        // Check if the execution was successful
        if !outcome.is_success() {
            return Err(EffectError::ExecutionError(format!("Failed to store register: {:?}", outcome)));
        }
        
        // Create the result
        let result = StoreResult::Success {
            transaction_id: outcome.data.get("transaction_id")
                .map(|v| v.as_str().unwrap_or("unknown").to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        };
        
        // Return the outcome
        Ok(EffectOutcome::success(self.id.clone())
            .with_result(result)
            .with_data("register_id", self.register_id.to_string())
            .with_data("contract", self.contract_address.0.clone()))
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::Domain
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Domain
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("register_id".to_string(), self.register_id.to_string());
        params.insert("contract".to_string(), self.contract_address.0.clone());
        params.insert("fields".to_string(), self.fields.iter().cloned().collect::<Vec<_>>().join(", "));
        params
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.fact_deps.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // Basic validation logic
        if self.fact_deps.is_empty() {
            return Ok(());
        }
        
        // Ensure all dependencies have valid IDs
        for dep in &self.fact_deps {
            if dep.fact_id.is_empty() {
                return Err(Error::ValidationError("Fact dependency has empty ID".into()));
            }
        }
        
        Ok(())
    }
}

/// CosmWasm commitment effect for storing register commitments on chain
#[derive(Debug, Clone)]
pub struct CosmWasmCommitmentEffect {
    /// Unique identifier
    id: EffectId,
    /// Resource register ID
    register_id: ResourceId,
    /// Commitment to store
    commitment: Commitment,
    /// Domain ID
    domain_id: DomainId,
    /// Address of the invoker
    invoker: Address,
    /// Contract address for storage
    contract_address: CosmWasmAddress,
    /// Fact dependencies
    fact_deps: Vec<FactDependency>,
    /// Fact snapshot
    snapshot: Option<FactSnapshot>,
}

impl CosmWasmCommitmentEffect {
    /// Create a new CosmWasm commitment effect
    pub fn new(
        register_id: ResourceId,
        commitment: Commitment,
        domain_id: DomainId,
        invoker: Address,
        contract_address: impl Into<CosmWasmAddress>,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            register_id,
            commitment,
            domain_id,
            invoker,
            contract_address: contract_address.into(),
            fact_deps: Vec::new(),
            snapshot: None,
        }
    }
    
    /// Get the register ID
    pub fn register_id(&self) -> &ResourceId {
        &self.register_id
    }
    
    /// Get the commitment
    pub fn commitment(&self) -> &Commitment {
        &self.commitment
    }
    
    /// Add a fact dependency
    pub fn with_fact_dependency(mut self, dependency: FactDependency) -> Self {
        self.fact_deps.push(dependency);
        self
    }
    
    /// Set the fact snapshot
    pub fn with_fact_snapshot(mut self, snapshot: FactSnapshot) -> Self {
        self.snapshot = Some(snapshot);
        self
    }
}

#[async_trait]
impl Effect for CosmWasmCommitmentEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }

    fn name(&self) -> &str {
        "cosmwasm_store_commitment"
    }
    
    fn display_name(&self) -> String {
        format!("Store commitment on CosmWasm chain: {}", self.register_id)
    }
    
    fn description(&self) -> String {
        format!("Store register commitment on CosmWasm contract {}", self.contract_address)
    }
    
    fn execute(&self, _context: &EffectContext) -> Result<EffectOutcome> {
        // Synchronous execution is not supported
        Err(Error::OperationNotSupported("CosmWasm commitment storage requires async context".into()))
    }
    
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Convert commitment to hex string
        let commitment_hex = hex::encode(&self.commitment.0);
        
        // Create the execute message for storing the commitment
        let message = json!({
            "store_commitment": {
                "register_id": self.register_id.to_string(),
                "commitment": commitment_hex,
            }
        });
        
        // Create the execute effect
        let execute_effect = CosmWasmExecuteEffect::new(
            self.contract_address.0.clone(),
            message
        );
        
        // Execute the effect
        let outcome = execute_effect.execute_async(context).await?;
        
        // Check if the execution was successful
        if !outcome.is_success() {
            return Err(EffectError::ExecutionError(format!("Failed to store commitment: {:?}", outcome)));
        }
        
        // Create the result
        let result = StoreResult::Success {
            transaction_id: outcome.data.get("transaction_id")
                .map(|v| v.as_str().unwrap_or("unknown").to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        };
        
        // Return the outcome
        Ok(EffectOutcome::success(self.id.clone())
            .with_result(result)
            .with_data("register_id", self.register_id.to_string())
            .with_data("contract", self.contract_address.0.clone())
            .with_data("commitment", commitment_hex))
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::Domain
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Domain
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("register_id".to_string(), self.register_id.to_string());
        params.insert("contract".to_string(), self.contract_address.0.clone());
        params.insert("commitment".to_string(), hex::encode(&self.commitment.0));
        params
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.fact_deps.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // Basic validation logic
        if self.fact_deps.is_empty() {
            return Ok(());
        }
        
        // Ensure all dependencies have valid IDs
        for dep in &self.fact_deps {
            if dep.fact_id.is_empty() {
                return Err(Error::ValidationError("Fact dependency has empty ID".into()));
            }
        }
        
        Ok(())
    }
}

/// Factory for creating CosmWasm storage effects
#[derive(Debug, Clone)]
pub struct CosmWasmStorageEffectFactory {
    /// Contract address for storage
    contract_address: CosmWasmAddress,
    /// Domain ID for this domain
    domain_id: DomainId,
}

impl CosmWasmStorageEffectFactory {
    /// Create a new CosmWasm storage effect factory
    pub fn new(contract_address: impl Into<CosmWasmAddress>, domain_id: DomainId) -> Self {
        Self {
            contract_address: contract_address.into(),
            domain_id,
        }
    }
    
    /// Create a store effect for the given register
    pub fn create_store_effect(
        &self,
        register_id: ResourceId,
        fields: HashSet<String>,
        invoker: Address,
    ) -> Result<CosmWasmStoreEffect> {
        Ok(CosmWasmStoreEffect::new(
            register_id,
            fields,
            self.domain_id.clone(),
            invoker,
            self.contract_address.0.clone()
        ))
    }
    
    /// Create a commitment effect for the given register
    pub fn create_commitment_effect(
        &self,
        register_id: ResourceId,
        commitment: Commitment,
        invoker: Address,
    ) -> Result<CosmWasmCommitmentEffect> {
        Ok(CosmWasmCommitmentEffect::new(
            register_id,
            commitment,
            self.domain_id.clone(),
            invoker,
            self.contract_address.0.clone()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cosmwasm_storage_effect() {
        // Create a register ID
        let register_id = ResourceId::new_unique();
        
        // Create a domain ID
        let domain_id = DomainId::new("cosmwasm".to_string());
        
        // Create an invoker
        let invoker = Address::from("addr123");
        
        // Create a contract address
        let contract_address = "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr";
        
        // Create a factory
        let factory = CosmWasmStorageEffectFactory::new(
            contract_address,
            domain_id.clone()
        );
        
        // Create a store effect
        let store_effect = factory.create_store_effect(
            register_id.clone(),
            HashSet::new(),
            invoker.clone(),
        ).unwrap();
        
        assert_eq!(store_effect.register_id(), &register_id);
        assert_eq!(store_effect.contract_address.0, contract_address);
        
        // Create a commitment
        let commitment = Commitment(vec![1, 2, 3, 4]);
        
        // Create a commitment effect
        let commitment_effect = factory.create_commitment_effect(
            register_id.clone(),
            commitment.clone(),
            invoker.clone(),
        ).unwrap();
        
        assert_eq!(commitment_effect.register_id(), &register_id);
        assert_eq!(commitment_effect.commitment(), &commitment);
        assert_eq!(commitment_effect.contract_address.0, contract_address);
    }
} 