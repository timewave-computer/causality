// Domain Effect Module - Integrates domain adapters with the effect system
//
// This module provides the core types and implementation for treating domain
// adapter operations as effects, and integrating effects with domain adapters.

pub mod domain_registry;
pub mod handler;
pub mod examples;
pub mod domain_selection;
pub mod evm_effects;
pub mod cosmwasm_effects;
pub mod zk_effects;

#[cfg(test)]
pub mod test;

// Re-export key types and functions from domain_registry
pub use self::domain_registry::{
    EffectDomainRegistry, DomainEffectHandler, EffectDomainAdapterFactory,
    create_domain_registry
};

// Re-export key types and functions from handler
pub use self::handler::{
    DomainEffectHandlerAdapter, create_domain_handler, create_domain_handler_with_new_registry
};

// Re-export key types and functions from domain_selection
pub use self::domain_selection::{
    DomainSelectionEffect, DomainSelectionHandler, SelectionCriteria,
    select_domains_by_type, select_domains_by_capability, 
    select_domains_by_type_and_capability, select_domains_by_name, select_domains_custom
};

// Re-export EVM-specific effects and functions
pub use self::evm_effects::{
    EvmContractCallEffect, EvmStateQueryEffect, EvmGasEstimationEffect, EvmStateQueryType,
    evm_view_call, evm_transaction_call, evm_balance, evm_storage, evm_code,
    evm_transaction, evm_block, evm_estimate_gas
};

// Re-export CosmWasm-specific effects and functions
pub use self::cosmwasm_effects::{
    CosmWasmExecuteEffect, CosmWasmQueryEffect, CosmWasmInstantiateEffect, CosmWasmCodeUploadEffect,
    cosmwasm_execute, cosmwasm_query, cosmwasm_instantiate, cosmwasm_upload
};

// Re-export ZK-specific effects and functions
pub use self::zk_effects::{
    ZkProveEffect, ZkVerifyEffect, ZkWitnessEffect, ZkProofCompositionEffect,
    zk_prove, zk_verify, zk_witness, zk_compose
};

// Re-export example functions
pub use self::examples::{
    query_domain_example, submit_transaction_example, 
    check_capability_example, integration_example,
    domain_selection_example, evm_effects_example,
    cosmwasm_effects_example, zk_effects_example
};

// Re-export the core traits and types from the main file
pub use super::domain_effect::{
    DomainAdapterEffect, DomainContext, DomainQueryEffect,
    DomainTransactionEffect, DomainTimeMapEffect, DomainCapabilityEffect,
    query_domain_fact, submit_domain_transaction, get_domain_time_map, check_domain_capability
}; 