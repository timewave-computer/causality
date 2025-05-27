//! Chain Client Factory
//!
//! This module provides factory functions for creating blockchain client instances.
//! It simplifies the creation of properly configured clients for different chains.
//!
//! ## Factory Organization
//!
//! * **Utility Functions**: Simple initialization of chain-specific clients
//! * **Factory Functions**: Construction of typed `ValenceChainClient` instances
//! * **Type Re-exports**: Chain-specific types from valence-domain-clients

// Re-export the ValenceChainClient for convenience
pub use crate::chain::valence_client::ValenceChainClient;

// Re-export common types from valence-domain-clients
pub use valence_core::error::ClientError;
pub use valence_core::transaction::TransactionResponse;

// Blockchain-specific client types
#[cfg(feature = "neutron")]
pub use valence_cosmos::chains::neutron::NeutronClient;
#[cfg(feature = "ethereum")]
pub use valence_evm::chains::ethereum::EthereumClient;

// Re-export EVM types
#[cfg(feature = "ethereum")]
pub use valence_evm::types::{
    EvmAddress, EvmBytes, EvmHash, EvmLog, EvmTransactionReceipt,
    EvmTransactionRequest, EvmU256,
};

// Re-export Cosmos types
#[cfg(feature = "neutron")]
pub use valence_cosmos::types::{
    CosmosAccount, CosmosBaseAccount, CosmosCoin, CosmosFee, CosmosGasInfo,
    CosmosHeader,
};

// Import necessary types from our crates
// use crate::chain::types::ChainId;  // Unused, removed
// use std::sync::Arc;  // Unused, removed

// Import our wrapper types for valence clients
#[cfg(feature = "neutron")]
use super::valence_client::cosmos::CosmosValenceChainClient;
#[cfg(feature = "ethereum")]
use super::valence_client::evm::EvmValenceChainClient;

//-----------------------------------------------------------------------------
// Client Initialization Functions
//-----------------------------------------------------------------------------

/// Create a base Ethereum client with minimal configuration
#[cfg(feature = "ethereum")]
pub fn ethereum_client_init(endpoint_url: &str) -> EthereumClient {
    // The new API requires endpoint_url to be a &str and also takes mnemonic and derivation path
    EthereumClient::new(endpoint_url, "", None)
        .expect("Failed to create Ethereum client")
}

/// Create a base Neutron client with minimal configuration
#[cfg(feature = "neutron")]
pub async fn neutron_client_init(endpoint_url: &str) -> NeutronClient {
    // The new API requires async initialization and takes additional parameters
    NeutronClient::new(endpoint_url, "pion-1", "", None)
        .await
        .expect("Failed to create Neutron client")
}

//-----------------------------------------------------------------------------
// Chain Client Factory Functions
//-----------------------------------------------------------------------------

/// Create a new Ethereum chain client
#[cfg(feature = "ethereum")]
pub fn create_ethereum_client(endpoint_url: &str) -> EvmValenceChainClient {
    let client = ethereum_client_init(endpoint_url);
    let valence_client = crate::chain::valence_client::ValenceChainClient {
        client,
        chain_id: crate::chain::types::ChainId(1), // Ethereum mainnet
    };

    EvmValenceChainClient(valence_client)
}

/// Create a new Neutron chain client
#[cfg(feature = "neutron")]
pub async fn create_neutron_client(endpoint_url: &str) -> CosmosValenceChainClient {
    let client = neutron_client_init(endpoint_url).await;
    let valence_client = crate::chain::valence_client::ValenceChainClient {
        client,
        // Chain ID is inferred from the client config
        chain_id: crate::chain::types::ChainId(0), // Will be overridden by the chain_id() implementation
    };

    CosmosValenceChainClient(valence_client)
}
