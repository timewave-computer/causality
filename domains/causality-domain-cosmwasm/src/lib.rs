// CosmWasm domain adapter module
// Original file: src/domain_adapters/cosmwasm/mod.rs

// CosmWasm Domain Adapter
//
// This module provides integration with CosmWasm-based blockchains,
// allowing Causality to interact with smart contracts deployed on these chains.

// Core CosmWasm adapter implementation
pub mod adapter;

// Type definitions for CosmWasm
pub mod types;

// Storage strategy implementation for CosmWasm
pub mod storage_strategy;

// Re-export core types
pub use adapter::{
    CosmWasmAdapter,
    CosmWasmAdapterConfig,
    CosmWasmExecuteEffect,
    CosmWasmQueryEffect,
    cosmwasm_execute,
    cosmwasm_query,
};
pub use types::{
    CosmWasmAddress,
    CosmWasmMessage,
    CosmWasmMessageType,
    CosmWasmQueryResult,
    CosmWasmExecutionResult,
    CosmWasmCode,
    CosmWasmEvent,
    Coin,
    coin,
};
pub use storage_strategy::{
    CosmWasmStorageEffectFactory,
    CosmWasmStoreEffect,
    CosmWasmCommitmentEffect,
};

// Factory function to create a new CosmWasm adapter
pub fn create_cosmwasm_adapter(config: CosmWasmAdapterConfig) -> causality_types::Result<CosmWasmAdapter> {
    CosmWasmAdapter::new(config)
}

pub mod config;
pub mod abci;
pub mod effects;
pub mod error;
pub mod zk;

// Re-export key types
pub use effects::{CosmWasmExecuteEffect, CosmWasmQueryEffect};
pub use storage_strategy::{CosmWasmStoreEffect, CosmWasmCommitmentEffect, CosmWasmStorageEffectFactory};
pub use zk::{
    CosmWasmZkCompileEffect, CosmWasmZkWitnessEffect, 
    CosmWasmZkProveEffect, CosmWasmZkVerifyEffect
};
pub use error::CosmWasmError; 