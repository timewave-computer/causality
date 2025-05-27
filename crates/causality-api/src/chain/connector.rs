//! Chain Connector Interface
//!
//! This module defines interfaces for blockchain connectors that provide a unified API
//! for interacting with different blockchain networks. Connectors abstract away blockchain-specific
//! details while maintaining bounded types for ZK compatibility.

use async_trait::async_trait;
use std::sync::Arc;

use super::transaction::{ChainTransaction, ChainTransactionMetadata};
use super::types::{BlockId, ChainId, TransactionStatus};
use crate::gateway::ApiError;
use causality_types::primitive::ids::TransactionId;

//-----------------------------------------------------------------------------
// Chain Connector Interface
//-----------------------------------------------------------------------------

/// Core trait for chain connectors
#[async_trait]
pub trait ChainConnector: Send + Sync {
    /// Submit a transaction to the chain with metadata
    async fn submit_transaction(
        &self,
        transaction: ChainTransaction,
        metadata: ChainTransactionMetadata,
    ) -> Result<TransactionId, ApiError>;

    /// Get the status of a transaction
    async fn get_transaction_status(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<TransactionStatus, ApiError>;

    /// Get the current block height
    async fn get_block_height(&self) -> Result<u64, ApiError>;

    /// Get the block at the given height
    async fn get_block_by_height(&self, height: u64) -> Result<BlockId, ApiError>;

    /// Get the block with the given ID
    async fn get_block_by_id(&self, block_id: &BlockId)
        -> Result<Vec<u8>, ApiError>;

    /// Get the chain ID this connector is for
    fn chain_id(&self) -> ChainId;

    /// Check if the chain connector is healthy
    async fn health_check(&self) -> bool;
}

/// Factory for creating chain connectors
#[async_trait]
pub trait ChainConnectorFactory: Send + Sync {
    /// Create a new chain connector
    async fn create_connector(
        &self,
        chain_id: ChainId,
    ) -> Result<Arc<dyn ChainConnector>, ApiError>;
}
