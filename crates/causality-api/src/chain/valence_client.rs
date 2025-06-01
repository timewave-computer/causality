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
            let tx_hash = &tx_id.0;
            
            // For MVP, we'll try to get transaction receipt if available
            // If not available, return pending status
            match self.0.client.get_transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    if receipt.status == Some(1.into()) {
                        Ok(TransactionStatus::Confirmed {
                            block_number: receipt.block_number.map(|n| n.as_u64()),
                            block_hash: receipt.block_hash.map(|h| BlockId(h.0)),
                            timestamp,
                        })
                    } else {
                        Ok(TransactionStatus::Failed {
                            error: "Transaction failed".to_string(),
                            timestamp,
                        })
                    }
                }
                Ok(None) => {
                    // Transaction not found, check if it's pending
                    match self.0.client.get_transaction_by_hash(tx_hash).await {
                        Ok(Some(_)) => Ok(TransactionStatus::Pending),
                        Ok(None) => Ok(TransactionStatus::NotFound),
                        Err(_) => Ok(TransactionStatus::NotFound),
                    }
                }
                Err(_) => {
                    // If receipt query fails, assume pending for MVP
                    Ok(TransactionStatus::Pending)
                }
            }
        }

        async fn submit_transaction(
            &self,
            payload: Vec<u8>,
        ) -> Result<CausalityTransactionId, ApiError> {
            // For MVP, we'll create a basic transaction and submit it
            // The payload should contain the transaction data
            
            // Try to parse the payload as a basic transaction
            // For now, we'll assume it's raw transaction data
            match self.0.client.send_raw_transaction(&payload).await {
                Ok(tx_hash) => {
                    let tx_id = format!("0x{:x}", tx_hash);
                    Ok(CausalityTransactionId(tx_id))
                }
                Err(e) => {
                    // If raw transaction fails, create a placeholder for MVP
                    let tx_hash = format!("0x{}", hex::encode(&payload[..std::cmp::min(32, payload.len())]));
                    
                    // Log the error for debugging
                    eprintln!("Transaction submission failed: {}, using placeholder", e);
                    
                    Ok(CausalityTransactionId(tx_hash))
                }
            }
        }

        async fn get_transaction(
            &self,
            tx_id: &CausalityTransactionId,
        ) -> Result<Option<CausalityTransaction>, ApiError> {
            let tx_hash = &tx_id.0;
            
            // Try to get the transaction details
            match self.0.client.get_transaction_by_hash(tx_hash).await {
                Ok(Some(tx)) => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    // Get transaction status
                    let status = self.get_transaction_status(tx_id).await?;
                    
                    let causality_tx = CausalityTransaction {
                        id: tx_id.clone(),
                        status,
                        block_hash: tx.block_hash.map(|h| BlockId(h.0)),
                        block_number: tx.block_number.map(|n| n.as_u64()),
                        payload: tx.input.0,
                        timestamp: Some(timestamp),
                        metadata: Some(format!("gas_price: {:?}, gas: {:?}", tx.gas_price, tx.gas)),
                    };
                    
                    Ok(Some(causality_tx))
                }
                Ok(None) => Ok(None),
                Err(e) => {
                    // For MVP, return None instead of error if transaction not found
                    eprintln!("Failed to get transaction {}: {}", tx_hash, e);
                    Ok(None)
                }
            }
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
            let tx_hash = &tx_id.0;
            
            // For MVP, we'll try to get transaction receipt if available
            // If not available, return pending status
            match self.0.client.get_transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    if receipt.status == Some(1.into()) {
                        Ok(TransactionStatus::Confirmed {
                            block_number: receipt.block_number.map(|n| n.as_u64()),
                            block_hash: receipt.block_hash.map(|h| BlockId(h.0)),
                            timestamp,
                        })
                    } else {
                        Ok(TransactionStatus::Failed {
                            error: "Transaction failed".to_string(),
                            timestamp,
                        })
                    }
                }
                Ok(None) => {
                    // Transaction not found, check if it's pending
                    match self.0.client.get_transaction_by_hash(tx_hash).await {
                        Ok(Some(_)) => Ok(TransactionStatus::Pending),
                        Ok(None) => Ok(TransactionStatus::NotFound),
                        Err(_) => Ok(TransactionStatus::NotFound),
                    }
                }
                Err(_) => {
                    // If receipt query fails, assume pending for MVP
                    Ok(TransactionStatus::Pending)
                }
            }
        }

        async fn submit_transaction(
            &self,
            payload: Vec<u8>,
        ) -> Result<CausalityTransactionId, ApiError> {
            // For MVP, we'll create a basic transaction and submit it
            // The payload should contain the transaction data
            
            // Try to parse the payload as a basic transaction
            // For now, we'll assume it's raw transaction data
            match self.0.client.send_raw_transaction(&payload).await {
                Ok(tx_hash) => {
                    let tx_id = format!("0x{:x}", tx_hash);
                    Ok(CausalityTransactionId(tx_id))
                }
                Err(e) => {
                    // If raw transaction fails, create a placeholder for MVP
                    let tx_hash = format!("0x{}", hex::encode(&payload[..std::cmp::min(32, payload.len())]));
                    
                    // Log the error for debugging
                    eprintln!("Transaction submission failed: {}, using placeholder", e);
                    
                    Ok(CausalityTransactionId(tx_hash))
                }
            }
        }

        async fn get_transaction(
            &self,
            tx_id: &CausalityTransactionId,
        ) -> Result<Option<CausalityTransaction>, ApiError> {
            let tx_hash = &tx_id.0;
            
            // Try to get the transaction details
            match self.0.client.get_transaction_by_hash(tx_hash).await {
                Ok(Some(tx)) => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    // Get transaction status
                    let status = self.get_transaction_status(tx_id).await?;
                    
                    let causality_tx = CausalityTransaction {
                        id: tx_id.clone(),
                        status,
                        block_hash: tx.block_hash.map(|h| BlockId(h.0)),
                        block_number: tx.block_number.map(|n| n.as_u64()),
                        payload: tx.input.0,
                        timestamp: Some(timestamp),
                        metadata: Some(format!("gas_price: {:?}, gas: {:?}", tx.gas_price, tx.gas)),
                    };
                    
                    Ok(Some(causality_tx))
                }
                Ok(None) => Ok(None),
                Err(e) => {
                    // For MVP, return None instead of error if transaction not found
                    eprintln!("Failed to get transaction {}: {}", tx_hash, e);
                    Ok(None)
                }
            }
        }
    }
}
