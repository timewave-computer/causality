//! Valence Domain Clients Implementation
//!
//! This module implements the ChainClient trait using the valence-domain-clients library
//! to provide access to various blockchain networks.
//!
//! ## Implementation Structure
//!
//! * **Core Client**: The `ValenceChainClient<T>` generic wrapper implementation
//! * **Chain-Specific Modules**: Feature-gated implementations for different blockchains
//!   * `evm` module: Ethereum and other EVM-compatible chains
//!   * `cosmos` module: Cosmos SDK-based blockchains

// Only import what we actually use
use crate::chain::types::ChainId;

/// Core blockchain client implementation that uses valence-domain-clients.
/// This struct serves as a wrapper around various blockchain specific clients.
pub struct ValenceChainClient<T: Send + Sync> {
    /// The inner blockchain client from valence-domain-clients
    pub client: T,
    /// Chain ID this client is configured for
    pub chain_id: ChainId,
}

impl<T: Send + Sync> ValenceChainClient<T> {
    /// Create a new ValenceChainClient with the given client and chain ID
    ///
    /// # Arguments
    /// * `client` - The blockchain-specific client to wrap
    /// * `chain_id` - The chain ID for the client
    ///
    /// # Returns
    /// A new ValenceChainClient instance
    pub fn new(client: T, chain_id: ChainId) -> Self {
        Self { client, chain_id }
    }

    /// Get a reference to the inner client
    ///
    /// # Returns
    /// A reference to the wrapped blockchain client
    pub fn inner(&self) -> &T {
        &self.client
    }


}

//-----------------------------------------------------------------------------
// Ethereum Chain Client Implementation
//-----------------------------------------------------------------------------

#[cfg(feature = "ethereum")]
pub mod evm {
    use super::*;
    use crate::chain::types::{
        ApiError, CausalityTransaction, CausalityTransactionId, ChainClient,
        ChainClientError, TransactionStatus,
    };
    use async_trait::async_trait;
    use valence_evm::base_client::EvmBaseClient;
    use valence_evm::chains::ethereum::EthereumClient;
    // use valence_evm::types::EvmAddress;

    // Use concrete EthereumClient instead of a type parameter
    pub struct EvmValenceChainClient(pub ValenceChainClient<EthereumClient>);

    /// Implementation for Ethereum-compatible blockchains
    #[async_trait]
    impl ChainClient for EvmValenceChainClient {
        fn chain_id(&self) -> ChainId {
            self.0.chain_id
        }

        async fn address(&self) -> Result<String, ApiError> {
            // Get the address from the EVM client using evm_signer_address
            let address = self.0.client.evm_signer_address();
            Ok(address.to_string())
        }

        async fn balance(
            &self,
            _asset_id: Option<String>,
        ) -> Result<String, ApiError> {
            // Get the main address
            let address = self.0.client.evm_signer_address();

            // Query balance for the signer address
            let balance =
                self.0.client.get_balance(&address).await.map_err(|e| {
                    ApiError::new(
                        ChainClientError::RpcError(e.to_string()),
                        "Failed to get balance".to_string(),
                        None,
                    )
                })?;

            Ok(balance.to_string())
        }

        async fn get_block_height(&self) -> Result<u64, ApiError> {
            let block_number =
                self.0.client.get_block_number().await.map_err(|e| {
                    ApiError::new(
                        ChainClientError::RpcError(e.to_string()),
                        "Failed to get block number".to_string(),
                        None,
                    )
                })?;

            Ok(block_number)
        }

        async fn get_transaction_status(
            &self,
            tx_id: &CausalityTransactionId,
        ) -> Result<TransactionStatus, ApiError> {
            let _tx_hash = &tx_id.0;
            
            // For now, return a placeholder status since the receipt method doesn't exist
            // TODO: Implement proper transaction status checking when valence API is available
            let _timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            Ok(TransactionStatus::Pending)
        }

        async fn submit_transaction(
            &self,
            payload: Vec<u8>,
        ) -> Result<CausalityTransactionId, ApiError> {
            // For now, return a placeholder transaction ID
            // TODO: Implement proper transaction submission when valence API is available
            let tx_hash = format!("0x{}", hex::encode(&payload[..std::cmp::min(32, payload.len())]));
            Ok(CausalityTransactionId(tx_hash))
        }

        async fn get_transaction(
            &self,
            tx_id: &CausalityTransactionId,
        ) -> Result<Option<CausalityTransaction>, ApiError> {
            let _tx_hash = &tx_id.0;
            
            // For now, return None since the transaction methods don't exist
            // TODO: Implement proper transaction retrieval when valence API is available
            Ok(None)
        }
    }
}

//-----------------------------------------------------------------------------
// Cosmos Chain Client Implementation
//-----------------------------------------------------------------------------

#[cfg(feature = "neutron")]
pub mod cosmos {
    use super::*;
    use crate::chain::types::{
        ApiError, CausalityTransaction, CausalityTransactionId, ChainClient,
        ChainClientError, TransactionStatus,
    };
    use async_trait::async_trait;
    use valence_cosmos::base_client::CosmosBaseClient;
    use valence_cosmos::chains::neutron::NeutronClient;
    use valence_cosmos::GrpcSigningClient;

    // Use concrete NeutronClient instead of a type parameter
    pub struct CosmosValenceChainClient(pub ValenceChainClient<NeutronClient>);

    /// Implementation for Cosmos blockchains
    #[async_trait]
    impl ChainClient for CosmosValenceChainClient {
        fn chain_id(&self) -> ChainId {
            // For cosmos chains, we'll use a simple hash of the chain ID string
            // to map it to a u32 for our ChainId type
            let chain_id_str = self.0.client.chain_id_str();

            // Simple hash function to convert string to u32
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            chain_id_str.hash(&mut hasher);
            let hash = (hasher.finish() % (u32::MAX as u64)) as u32;

            ChainId(hash)
        }

        async fn address(&self) -> Result<String, ApiError> {
            // For Cosmos chains, we don't have a direct way to get the signer address
            // We'll return a placeholder for now
            Ok("cosmos1placeholder".to_string())
        }

        async fn balance(
            &self,
            asset_id: Option<String>,
        ) -> Result<String, ApiError> {
            // In Cosmos, we need to specify which token we want the balance for
            // For now, we'll use a hard-coded placeholder address since we can't get it directly
            let address = "cosmos1placeholder";
            let denom = asset_id.unwrap_or_else(|| "untrn".to_string());

            let balance = self
                .0
                .client
                .query_balance(address, &denom)
                .await
                .map_err(|e| {
                    ApiError::new(
                        ChainClientError::RpcError(e.to_string()),
                        "Failed to get balance".to_string(),
                        None,
                    )
                })?;

            Ok(balance.to_string())
        }

        async fn get_block_height(&self) -> Result<u64, ApiError> {
            let block_header =
                self.0.client.latest_block_header().await.map_err(|e| {
                    ApiError::new(
                        ChainClientError::RpcError(e.to_string()),
                        "Failed to get block height".to_string(),
                        None,
                    )
                })?;

            // Convert i64 height to u64, ensuring it's positive
            if block_header.height < 0 {
                return Err(ApiError::new(
                    ChainClientError::EncodingError(
                        "Block height is negative".to_string(),
                    ),
                    "Received negative block height".to_string(),
                    None,
                ));
            }

            Ok(block_header.height as u64)
        }

        async fn get_transaction_status(
            &self,
            tx_id: &CausalityTransactionId,
        ) -> Result<TransactionStatus, ApiError> {
            let _tx_hash = &tx_id.0;
            
            // For now, return a placeholder status since the query methods don't exist
            // TODO: Implement proper transaction status checking when valence API is available
            let _timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            Ok(TransactionStatus::Pending)
        }

        async fn submit_transaction(
            &self,
            payload: Vec<u8>,
        ) -> Result<CausalityTransactionId, ApiError> {
            // For now, return a placeholder transaction ID
            // TODO: Implement proper transaction submission when valence API is available
            let tx_hash = format!("cosmos{}", hex::encode(&payload[..std::cmp::min(32, payload.len())]));
            Ok(CausalityTransactionId(tx_hash))
        }

        async fn get_transaction(
            &self,
            tx_id: &CausalityTransactionId,
        ) -> Result<Option<CausalityTransaction>, ApiError> {
            let _tx_hash = &tx_id.0;
            
            // For now, return None since the transaction methods don't exist
            // TODO: Implement proper transaction retrieval when valence API is available
            Ok(None)
        }
    }
}

// Helper function to map valence-core ClientError to our ApiError type
// Commented out since it's not currently used but may be useful in the future
// fn map_error(err: valence_core::error::ClientError) -> ApiError {
//     ApiError::new(
//         ChainClientError::RpcError(err.to_string()),
//         format!("Client error: {}", err),
//         None,
//     )
// }

// Helper function to hash Cosmos chain IDs
// Moved directly into the module where it's used
// fn hash_chain_id(chain_id: &str) -> u32 {
//     use std::collections::hash_map::DefaultHasher;
//     use std::hash::{Hash, Hasher};
//
//     let mut hasher = DefaultHasher::new();
//     chain_id.hash(&mut hasher);
//     (hasher.finish() % (u32::MAX as u64)) as u32
// }
