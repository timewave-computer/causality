// EVM storage strategy
// Original file: src/domain_adapters/evm/storage_strategy.rs

// Ethereum-specific storage strategies for ResourceRegister
//
// This module implements domain-specific storage strategies for Ethereum and
// other EVM-compatible chains, as part of the unified Resource-Register model.

use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use ethers::prelude::*;
use ethers::types::{Address, H256, U256};

use causality_types::Address as CausalityAddress;
use crate::domain::DomainId;
use crate::resource::{ContentId, ResourceRegister};
use causality_core::resource::{
    StorageStrategy, 
    StateVisibility
};
use causality_core::crypto::{
    Commitment, 
    NullifierId
};
use causality_core::capability::Right;
use crate::effect::{
    Effect, 
    EffectContext, 
    EffectResult, 
    EffectError, 
    EffectOutcome, 
    StorageEffect
};
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::{
    StoreResult, 
    ReadResult, 
    StoreOnChainEffect, 
    ReadFromChainEffect, 
    StoreCommitmentEffect
};
use causality_types::{Error, Result};

/// Ethereum register storage contract interface
#[derive(Debug)]
pub struct EthRegisterStore<M: Middleware> {
    /// Contract instance
    contract: ethers::contract::Contract<M>,
    /// Contract address
    address: Address,
}

impl<M: Middleware> EthRegisterStore<M> {
    /// Create a new Ethereum register store contract
    pub fn new(address: Address, client: Arc<M>) -> Result<Self> {
        // ABI for the contract (simplified for this example)
        const ABI: &str = r#"[
            {
                "inputs": [
                    {"name": "registerId", "type": "bytes32"},
                    {"name": "data", "type": "bytes"},
                    {"name": "visibility", "type": "uint8"}
                ],
                "name": "storeRegister",
                "outputs": [{"name": "", "type": "bool"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"name": "registerId", "type": "bytes32"}
                ],
                "name": "getRegister",
                "outputs": [{"name": "", "type": "bytes"}],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {"name": "registerId", "type": "bytes32"},
                    {"name": "commitment", "type": "bytes32"}
                ],
                "name": "storeCommitment",
                "outputs": [{"name": "", "type": "bool"}],
                "stateMutability": "nonpayable",
                "type": "function"
            },
            {
                "inputs": [
                    {"name": "registerId", "type": "bytes32"},
                    {"name": "nullifier", "type": "bytes32"}
                ],
                "name": "storeNullifier",
                "outputs": [{"name": "", "type": "bool"}],
                "stateMutability": "nonpayable",
                "type": "function"
            }
        ]"#;

        let contract = ethers::contract::Contract::new(address, ABI.parse().unwrap(), client);
        
        Ok(Self {
            contract,
            address,
        })
    }
    
    /// Store a register on-chain
    pub async fn store_register(&self, id: &ContentId, data: Vec<u8>, visibility: u8) -> Result<H256> {
        let id_bytes = to_resource_register_id_bytes(id)?;
        
        let call = self.contract.method::<_, bool>(
            "storeRegister", 
            (id_bytes, data, visibility)
        ).map_err(|e| Error::DomainApiError(format!("Failed to create contract call: {}", e)))?;
        
        let pending_tx = call.send().await
            .map_err(|e| Error::TransactionError(format!("Failed to send transaction: {}", e)))?;
            
        let receipt = pending_tx.await
            .map_err(|e| Error::TransactionError(format!("Failed to get transaction receipt: {}", e)))?
            .ok_or_else(|| Error::TransactionError("No transaction receipt returned".to_string()))?;
            
        Ok(receipt.transaction_hash)
    }
    
    /// Get a register from on-chain
    pub async fn get_register(&self, id: &ContentId) -> Result<Vec<u8>> {
        let id_bytes = to_resource_register_id_bytes(id)?;
        
        let result: Vec<u8> = self.contract.method::<_, Vec<u8>>(
            "getRegister", 
            (id_bytes,)
        )
        .map_err(|e| Error::DomainApiError(format!("Failed to create contract call: {}", e)))?
        .call().await
        .map_err(|e| Error::DomainApiError(format!("Failed to call contract: {}", e)))?;
        
        Ok(result)
    }
    
    /// Store a commitment on-chain
    pub async fn store_commitment(&self, id: &ContentId, commitment: &[u8; 32]) -> Result<H256> {
        let id_bytes = to_resource_register_id_bytes(id)?;
        
        let call = self.contract.method::<_, bool>(
            "storeCommitment", 
            (id_bytes, *commitment)
        ).map_err(|e| Error::DomainApiError(format!("Failed to create contract call: {}", e)))?;
        
        let pending_tx = call.send().await
            .map_err(|e| Error::TransactionError(format!("Failed to send transaction: {}", e)))?;
            
        let receipt = pending_tx.await
            .map_err(|e| Error::TransactionError(format!("Failed to get transaction receipt: {}", e)))?
            .ok_or_else(|| Error::TransactionError("No transaction receipt returned".to_string()))?;
            
        Ok(receipt.transaction_hash)
    }
    
    /// Store a nullifier on-chain
    pub async fn store_nullifier(&self, id: &ContentId, nullifier: &[u8; 32]) -> Result<H256> {
        let id_bytes = to_resource_register_id_bytes(id)?;
        
        let call = self.contract.method::<_, bool>(
            "storeNullifier", 
            (id_bytes, *nullifier)
        ).map_err(|e| Error::DomainApiError(format!("Failed to create contract call: {}", e)))?;
        
        let pending_tx = call.send().await
            .map_err(|e| Error::TransactionError(format!("Failed to send transaction: {}", e)))?;
            
        let receipt = pending_tx.await
            .map_err(|e| Error::TransactionError(format!("Failed to get transaction receipt: {}", e)))?
            .ok_or_else(|| Error::TransactionError("No transaction receipt returned".to_string()))?;
            
        Ok(receipt.transaction_hash)
    }
}

/// Convert a content ID to bytes32 for Ethereum contracts
fn to_resource_register_id_bytes(id: &ContentId) -> Result<[u8; 32]> {
    let mut bytes = [0u8; 32];
    let id_bytes = id.as_bytes();
    
    if id_bytes.len() > 32 {
        return Err(Error::ValidationError("Resource register ID is too long for Ethereum storage".to_string()));
    }
    
    bytes[..id_bytes.len()].copy_from_slice(id_bytes);
    Ok(bytes)
}

/// Backward compatibility
#[deprecated(since = "0.8.0", note = "Use to_resource_register_id_bytes instead")]
fn to_register_id_bytes(id: &ContentId) -> Result<[u8; 32]> {
    to_resource_register_id_bytes(id)
}

/// Ethereum-specific implementation of StoreOnChainEffect
pub struct EthereumStoreEffect {
    /// Base storage effect
    inner: StoreOnChainEffect,
    /// Contract address for storage
    contract_address: Address,
    /// Provider for Ethereum
    provider: Provider<Http>,
}

impl EthereumStoreEffect {
    /// Create a new Ethereum store effect
    pub fn new(
        inner: StoreOnChainEffect,
        contract_address: Address,
        provider: Provider<Http>,
    ) -> Self {
        Self {
            inner,
            contract_address,
            provider,
        }
    }
    
    /// Factory function to create a new Ethereum store effect
    pub fn create(
        register_id: ContentId,
        fields: HashSet<String>,
        domain_id: DomainId,
        invoker: CausalityAddress,
        contract_address: Address,
        rpc_url: &str,
    ) -> Result<Arc<dyn Effect>> {
        // Create the base effect
        let inner = StoreOnChainEffect::new(
            register_id,
            fields,
            domain_id,
            invoker,
        );
        
        // Create the provider
        let provider = Provider::<Http>::try_from(rpc_url.to_string())
            .map_err(|e| Error::DomainConnectionError(format!("Failed to create provider: {}", e)))?;
        
        // Create the Ethereum-specific effect
        let effect = Self {
            inner,
            contract_address,
            provider,
        };
        
        Ok(Arc::new(effect))
    }
}

#[async_trait]
impl Effect for EthereumStoreEffect {
    fn name(&self) -> &str {
        "ethereum_store_on_chain"
    }
    
    fn description(&self) -> &str {
        "Stores register data on Ethereum chain"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, causality_core::capability::Right)> {
        self.inner.required_capabilities()
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Get the register from the context
        let register = context.get_resource(&self.inner.register_id())
            .ok_or_else(|| EffectError::ResourceNotFound(self.inner.register_id().clone()))?;
        
        // Serialize the register to bytes
        let register_data = serde_json::to_vec(&register)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to serialize register: {}", e)))?;
        
        // Convert visibility to uint8
        let visibility = match register.storage_strategy {
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public } => 0u8,
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Private } => 1u8,
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Permissioned(_) } => 2u8,
            _ => 0u8, // Default to public for other strategies
        };
        
        // Create an Ethereum client
        let client = Arc::new(self.provider.clone());
        
        // Create the contract instance
        let store = EthRegisterStore::new(self.contract_address, client)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to create store: {}", e)))?;
        
        // Store the register
        let tx_hash = store.store_register(&register.id, register_data, visibility).await
            .map_err(|e| EffectError::ExecutionError(format!("Failed to store register: {}", e)))?;
        
        let result = StoreResult::Success {
            transaction_id: format!("{:?}", tx_hash),
        };
        
        // Create the effect outcome
        let mut outcome = EffectOutcome::new(context.execution_id.clone());
        outcome.success = true;
        outcome.result = Some(serde_json::to_value(result)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to serialize result: {}", e)))?);
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: crate::effect::ExecutionBoundary) -> bool {
        self.inner.can_execute_in(boundary)
    }
    
    fn preferred_boundary(&self) -> crate::effect::ExecutionBoundary {
        self.inner.preferred_boundary()
    }
}

/// Ethereum-specific implementation of StoreCommitmentEffect
pub struct EthereumCommitmentEffect {
    /// Base commitment effect
    inner: StoreCommitmentEffect,
    /// Contract address for storage
    contract_address: Address,
    /// Provider for Ethereum
    provider: Provider<Http>,
}

impl EthereumCommitmentEffect {
    /// Create a new Ethereum commitment effect
    pub fn new(
        inner: StoreCommitmentEffect,
        contract_address: Address,
        provider: Provider<Http>,
    ) -> Self {
        Self {
            inner,
            contract_address,
            provider,
        }
    }
    
    /// Factory function to create a new Ethereum commitment effect
    pub fn create(
        register_id: ContentId,
        commitment: Commitment,
        domain_id: DomainId,
        invoker: CausalityAddress,
        contract_address: Address,
        rpc_url: &str,
    ) -> Result<Arc<dyn Effect>> {
        // Create the base effect
        let inner = StoreCommitmentEffect::new(
            register_id,
            commitment,
            domain_id,
            invoker,
        );
        
        // Create the provider
        let provider = Provider::<Http>::try_from(rpc_url.to_string())
            .map_err(|e| Error::DomainConnectionError(format!("Failed to create provider: {}", e)))?;
        
        // Create the Ethereum-specific effect
        let effect = Self {
            inner,
            contract_address,
            provider,
        };
        
        Ok(Arc::new(effect))
    }
}

#[async_trait]
impl Effect for EthereumCommitmentEffect {
    fn name(&self) -> &str {
        "ethereum_store_commitment"
    }
    
    fn description(&self) -> &str {
        "Stores register commitment on Ethereum chain"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, causality_core::capability::Right)> {
        self.inner.required_capabilities()
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Get the commitment from the context
        let commitment = self.inner.commitment();
        
        // Create an Ethereum client
        let client = Arc::new(self.provider.clone());
        
        // Create the contract instance
        let store = EthRegisterStore::new(self.contract_address, client)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to create store: {}", e)))?;
        
        // Store the commitment
        let tx_hash = store.store_commitment(&self.inner.register_id(), &commitment.0).await
            .map_err(|e| EffectError::ExecutionError(format!("Failed to store commitment: {}", e)))?;
        
        let result = StoreResult::Success {
            transaction_id: format!("{:?}", tx_hash),
        };
        
        // Create the effect outcome
        let mut outcome = EffectOutcome::new(context.execution_id.clone());
        outcome.success = true;
        outcome.result = Some(serde_json::to_value(result)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to serialize result: {}", e)))?);
        
        Ok(outcome)
    }
    
    fn can_execute_in(&self, boundary: crate::effect::ExecutionBoundary) -> bool {
        self.inner.can_execute_in(boundary)
    }
    
    fn preferred_boundary(&self) -> crate::effect::ExecutionBoundary {
        self.inner.preferred_boundary()
    }
}

/// Factory for creating Ethereum storage effects
pub struct EthereumStorageEffectFactory {
    /// Contract address for storage
    contract_address: Address,
    /// RPC URL for the Ethereum network
    rpc_url: String,
    /// Domain ID for this domain
    domain_id: DomainId,
}

impl EthereumStorageEffectFactory {
    /// Create a new Ethereum storage effect factory
    pub fn new(contract_address: Address, rpc_url: String, domain_id: DomainId) -> Self {
        Self {
            contract_address,
            rpc_url,
            domain_id,
        }
    }
    
    /// Create a store on-chain effect
    pub fn create_store_effect(
        &self,
        register_id: ContentId,
        fields: HashSet<String>,
        invoker: CausalityAddress,
    ) -> Result<Arc<dyn Effect>> {
        EthereumStoreEffect::create(
            register_id,
            fields,
            self.domain_id.clone(),
            invoker,
            self.contract_address,
            &self.rpc_url,
        )
    }
    
    /// Create a store commitment effect
    pub fn create_commitment_effect(
        &self,
        register_id: ContentId,
        commitment: Commitment,
        invoker: CausalityAddress,
    ) -> Result<Arc<dyn Effect>> {
        EthereumCommitmentEffect::create(
            register_id,
            commitment,
            self.domain_id.clone(),
            invoker,
            self.contract_address,
            &self.rpc_url,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::core::k256::SecretKey;
    use ethers::signers::{Signer, Wallet};
    use std::str::FromStr;
    
    #[tokio::test]
    async fn test_ethereum_storage_effect() {
        // This is just a mock test - in a real test we would use a test provider
        // For now, we'll just check that the objects can be created
        
        let contract_address = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let rpc_url = "http://localhost:8545";
        let domain_id = DomainId::new("ethereum-test");
        
        let factory = EthereumStorageEffectFactory::new(
            contract_address,
            rpc_url.to_string(),
            domain_id,
        );
        
        let mut fields = HashSet::new();
        fields.insert("field1".to_string());
        fields.insert("field2".to_string());
        
        let invoker = CausalityAddress::new("0x1234567890123456789012345678901234567890");
        
        let effect = factory.create_store_effect(
            "register-123".to_string(),
            fields,
            invoker,
        );
        
        assert!(effect.is_ok());
    }
} 
