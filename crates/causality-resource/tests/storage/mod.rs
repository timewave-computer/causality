// Tests for the unified storage model using storage effects
//
// This module tests the integration between the ResourceRegister unified model
// and the Storage Effects that implement domain-specific storage strategies.

use std::collections::HashSet;
use std::sync::Arc;

use crate::resource::{
    ResourceRegister, ResourceId, ResourceLogic, FungibilityDomain, 
    Quantity, StorageStrategy
};
use crate::resource::resource_register::{
    StateVisibility, Commitment, NullifierId
};
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, 
    ExecutionBoundary, EffectRuntime
};
use crate::effect::storage::{
    StoreOnChainEffect, ReadFromChainEffect, StoreCommitmentEffect,
    StoreNullifierEffect, StoreResult, ReadResult
};
use crate::domain::{DomainId, DomainInfo, DomainType, DomainStatus};
use crate::address::Address;
use crate::tel::types::Metadata;
use crate::effect::storage::{
    create_domain_specific_store_effect,
    create_domain_specific_commitment_effect
};

// Helper to create a mock domain info
fn create_mock_domain_info(domain_type: DomainType) -> DomainInfo {
    let mut metadata = serde_json::Map::new();
    let mut endpoints = Vec::new();
    
    match domain_type {
        DomainType::EVM => {
            metadata.insert(
                "register_contract".to_string(), 
                serde_json::Value::String("0x1234567890123456789012345678901234567890".to_string())
            );
            endpoints.push("http://localhost:8545".to_string());
        },
        DomainType::CosmWasm => {
            metadata.insert(
                "register_contract".to_string(), 
                serde_json::Value::String("cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr".to_string())
            );
            metadata.insert(
                "chain_id".to_string(),
                serde_json::Value::String("cosmwasm-testnet".to_string())
            );
            endpoints.push("http://localhost:9090".to_string());
            endpoints.push("http://localhost:1317".to_string());
        },
        _ => {
            // Default values for other domain types
            endpoints.push("http://localhost:8000".to_string());
        }
    }
    
    DomainInfo {
        domain_id: DomainId::new(format!("{:?}-testnet", domain_type).to_lowercase()),
        name: format!("{:?} Testnet", domain_type),
        description: Some(format!("Test domain for {:?}", domain_type)),
        domain_type,
        status: DomainStatus::Active,
        endpoints,
        metadata: serde_json::Value::Object(metadata),
    }
}

// Helper to create a test resource register
fn create_test_resource_register(id: &str, logic: ResourceLogic, strategy: StorageStrategy) -> ResourceRegister {
    ResourceRegister::new(
        id.to_string(),
        logic,
        FungibilityDomain("test".to_string()),
        Quantity(100),
        Metadata::new(),
        strategy,
    )
}

#[tokio::test]
async fn test_generic_storage_effect() {
    // Create a test resource register
    let register_id = "test-register-1".to_string();
    let register = create_test_resource_register(
        &register_id,
        ResourceLogic::Fungible,
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Create fields to store
    let mut fields = HashSet::new();
    fields.insert("id".to_string());
    fields.insert("quantity".to_string());
    
    // Create the storage effect
    let domain_id = DomainId::new("test-domain");
    let invoker = Address::new("test-user");
    let effect = StoreOnChainEffect::new(
        register_id.clone(),
        fields,
        domain_id,
        invoker,
    );
    
    // Create the effect context
    let mut context = EffectContext::default();
    context.register_resource(register_id.clone(), register);
    
    // Execute the effect
    let outcome = effect.execute(context).await.unwrap();
    
    // Verify the outcome
    assert!(outcome.success);
    assert!(outcome.result.is_some());
    
    if let Some(result_value) = outcome.result {
        let result: StoreResult = serde_json::from_value(result_value).unwrap();
        match result {
            StoreResult::Success { transaction_id } => {
                assert!(transaction_id.starts_with("tx-"));
            },
            _ => panic!("Expected success result"),
        }
    }
}

#[tokio::test]
async fn test_domain_specific_storage_effect() {
    // Create domain info for different domains
    let evm_domain = create_mock_domain_info(DomainType::EVM);
    let cosmwasm_domain = create_mock_domain_info(DomainType::CosmWasm);
    
    // Create a test resource register
    let register_id = "test-register-2".to_string();
    let register = create_test_resource_register(
        &register_id,
        ResourceLogic::Fungible,
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Create fields to store
    let mut fields = HashSet::new();
    fields.insert("id".to_string());
    fields.insert("quantity".to_string());
    
    // Test with EVM domain
    let invoker = Address::new("0xuser");
    
    let evm_effect = create_domain_specific_store_effect(
        register_id.clone(),
        fields.clone(),
        evm_domain.domain_id.clone(),
        invoker.clone(),
        &evm_domain,
    ).unwrap();
    
    assert_eq!(evm_effect.name(), "ethereum_store_on_chain");
    
    // Test with CosmWasm domain
    let cosmwasm_effect = create_domain_specific_store_effect(
        register_id.clone(),
        fields.clone(),
        cosmwasm_domain.domain_id.clone(),
        invoker.clone(),
        &cosmwasm_domain,
    ).unwrap();
    
    assert_eq!(cosmwasm_effect.name(), "cosmwasm_store_on_chain");
}

#[tokio::test]
async fn test_commitment_storage_effect() {
    // Create a commitment
    let commitment = Commitment([1u8; 32]);
    
    // Create domain info
    let evm_domain = create_mock_domain_info(DomainType::EVM);
    
    // Create a test resource register
    let register_id = "test-register-3".to_string();
    let register = create_test_resource_register(
        &register_id,
        ResourceLogic::Fungible,
        StorageStrategy::CommitmentBased { 
            commitment: Some(commitment.clone()),
            nullifier: None,
        },
    );
    
    // Create the commitment effect
    let domain_id = evm_domain.domain_id.clone();
    let invoker = Address::new("0xuser");
    
    let effect = create_domain_specific_commitment_effect(
        register_id.clone(),
        commitment.clone(),
        domain_id,
        invoker,
        &evm_domain,
    ).unwrap();
    
    assert_eq!(effect.name(), "ethereum_store_commitment");
}

#[tokio::test]
async fn test_nullifier_storage_effect() {
    // Create a nullifier
    let nullifier = NullifierId([2u8; 32]);
    
    // Create a test resource register
    let register_id = "test-register-4".to_string();
    let register = create_test_resource_register(
        &register_id,
        ResourceLogic::Fungible,
        StorageStrategy::CommitmentBased { 
            commitment: None,
            nullifier: Some(nullifier.clone()),
        },
    );
    
    // Create the nullifier effect
    let domain_id = DomainId::new("test-domain");
    let invoker = Address::new("test-user");
    let effect = StoreNullifierEffect::new(
        register_id.clone(),
        nullifier.clone(),
        domain_id,
        invoker,
    );
    
    // Create the effect context
    let mut context = EffectContext::default();
    context.register_resource(register_id.clone(), register);
    
    // Execute the effect
    let outcome = effect.execute(context).await.unwrap();
    
    // Verify the outcome
    assert!(outcome.success);
    assert!(outcome.result.is_some());
    
    if let Some(result_value) = outcome.result {
        let result: StoreResult = serde_json::from_value(result_value).unwrap();
        match result {
            StoreResult::Success { transaction_id } => {
                assert!(transaction_id.starts_with("tx-"));
            },
            _ => panic!("Expected success result"),
        }
    }
}

#[tokio::test]
async fn test_hybrid_storage_strategy() {
    // Create a test resource register with hybrid storage strategy
    let register_id = "test-register-5".to_string();
    
    let mut on_chain_fields = HashSet::new();
    on_chain_fields.insert("id".to_string());
    on_chain_fields.insert("state".to_string());
    
    let commitment = Commitment([3u8; 32]);
    
    let register = create_test_resource_register(
        &register_id,
        ResourceLogic::Fungible,
        StorageStrategy::Hybrid { 
            on_chain_fields: on_chain_fields.clone(),
            remaining_commitment: Some(commitment.clone()),
        },
    );
    
    // Create the storage effect for on-chain fields
    let domain_id = DomainId::new("test-domain");
    let invoker = Address::new("test-user");
    let effect = StoreOnChainEffect::new(
        register_id.clone(),
        on_chain_fields,
        domain_id.clone(),
        invoker.clone(),
    );
    
    // Create the effect context
    let mut context = EffectContext::default();
    context.register_resource(register_id.clone(), register.clone());
    
    // Execute the effect
    let outcome = effect.execute(context.clone()).await.unwrap();
    
    // Verify the outcome
    assert!(outcome.success);
    
    // Also store the commitment
    let commitment_effect = StoreCommitmentEffect::new(
        register_id.clone(),
        commitment.clone(),
        domain_id,
        invoker,
    );
    
    // Execute the commitment effect
    let outcome = commitment_effect.execute(context).await.unwrap();
    
    // Verify the outcome
    assert!(outcome.success);
} 
