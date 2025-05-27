// EVM domain adapter module
//
// This module contains adapter implementations for Ethereum and other EVM-compatible chains.

// Core EVM adapter implementation
pub mod adapter;

// Type definitions for EVM
pub mod types;

// Storage strategy implementation for EVM
pub mod storage_strategy;

// Zero-knowledge operations for EVM
pub mod zk;

// Factory for creating EVM adapters
pub mod factory;

// Re-export core types
pub use adapter::EthereumAdapter;
pub use adapter::EthereumConfig;
pub use factory::{EthereumAdapterFactory, EthereumAdapterFactoryConfig};
pub use types::EvmAddress;
pub use types::EvmTransactionType;
pub use storage_strategy::{
    EthereumStorageEffectFactory,
    EthereumStoreEffect,
    EthereumCommitmentEffect,
};
pub use zk::{
    EvmZkCompileEffect, EvmZkWitnessEffect, 
    EvmZkProveEffect, EvmZkVerifyEffect,
    EvmZkEffectFactory
};

// Factory function to create a new Ethereum adapter
pub fn create_ethereum_adapter(config: EthereumConfig) -> crate::error::Result<EthereumAdapter> {
    EthereumAdapter::new(config)
} 