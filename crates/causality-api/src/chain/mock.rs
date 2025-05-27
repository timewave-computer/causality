//! Mock Chain Client Implementation
//!
//! This module provides a fully functional mock implementation of the chain client
//! interface for testing without external dependencies. All types maintain
//! ZK compatibility with bounded sizes and deterministic behavior.

use async_trait::async_trait;
use crate::serialization::{Encode, Decode};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use causality_types::utils::get_current_time_ms;
use causality_types::primitive::ids::IntentId;
use causality_types::core::Intent;

// Corrected imports for ChainClient trait and its associated types
use crate::chain::types::{
    ApiError, BlockId, CausalityTransaction, CausalityTransactionId,
    ChainClientError, ChainId, TransactionStatus,
};
use crate::chain::ChainClient;

// Imports for MockIntentHandler, assuming these are correctly defined in intent.rs
// and that intent.rs also uses the new ApiError from chain::types if needed.
use super::intent::{
    IntentHandler, IntentMetadata, IntentQueryResult, IntentStatus,
};

//-----------------------------------------------------------------------------
// Mock Chain Client
//-----------------------------------------------------------------------------

/// Mock implementation of a chain client for testing
pub struct MockChainClient {
    /// Chain ID
    chain_id: ChainId,

    /// Client address
    address: String,

    /// Current block height
    block_height: Arc<Mutex<u64>>,

    /// Transactions stored by ID
    transactions: Arc<Mutex<HashMap<CausalityTransactionId, CausalityTransaction>>>,

    /// Intents stored by ID
    intents: Arc<Mutex<HashMap<IntentId, Vec<u8>>>>,

    /// Intent metadata stored by ID
    intent_metadata: Arc<Mutex<HashMap<IntentId, IntentMetadata>>>,

    /// Balances stored by address
    balances: Arc<Mutex<HashMap<Option<String>, u128>>>,
}

impl MockChainClient {
    /// Create a new mock chain client
    pub fn new(chain_id: ChainId, address: &str) -> Self {
        let mut balances = HashMap::new();
        balances.insert(None, 1_000_000_000_000);

        Self {
            chain_id,
            address: address.to_string(),
            block_height: Arc::new(Mutex::new(1)),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            intents: Arc::new(Mutex::new(HashMap::new())),
            intent_metadata: Arc::new(Mutex::new(HashMap::new())),
            balances: Arc::new(Mutex::new(balances)),
        }
    }

    /// Advance the block height
    pub fn advance_block(&self, blocks: u64) {
        let mut height = self.block_height.lock().unwrap();
        *height += blocks;
    }

    /// Set the balance for an asset ID (None for native asset)
    pub fn set_balance(&self, asset_id: Option<String>, amount: u128) {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(asset_id, amount);
    }

    /// Get the balance for an asset ID (None for native asset) for the client's address
    pub fn get_balance_for_client_address(
        &self,
        asset_id: Option<String>,
    ) -> Option<u128> {
        self.balances.lock().unwrap().get(&asset_id).copied()
    }

    /// Compute IntentId from Intent using content hash
    fn compute_intent_id(intent: &Intent) -> IntentId {
        let intent_data = intent.as_ssz_bytes();
        let hash = Sha256::digest(&intent_data);
        IntentId::new(hash.into())
    }

    /// Register an intent directly (mock-specific helper)
    pub fn register_intent_mock(&self, intent: Intent) -> IntentId {
        let intent_id = Self::compute_intent_id(&intent);
        let intent_data = intent.as_ssz_bytes();

        self.intents.lock().unwrap().insert(intent_id, intent_data);

        let metadata = IntentMetadata {
            block_height: *self.block_height.lock().unwrap(),
            timestamp: get_current_time_ms(),
            tx_hash: format!("mock_tx_intent_{:?}", intent_id),
            status: IntentStatus::Pending,
        };
        self.intent_metadata
            .lock()
            .unwrap()
            .insert(intent_id, metadata);
        intent_id
    }

    /// Update intent status (mock-specific helper)
    pub fn update_intent_status_mock(
        &self,
        intent_id: &IntentId,
        status: IntentStatus,
    ) {
        if let Some(metadata) =
            self.intent_metadata.lock().unwrap().get_mut(intent_id)
        {
            metadata.status = status;
        }
    }
}

#[async_trait]
impl ChainClient for MockChainClient {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    async fn address(&self) -> Result<String, ApiError> {
        Ok(self.address.clone())
    }

    async fn submit_transaction(
        &self,
        payload: Vec<u8>,
    ) -> Result<CausalityTransactionId, ApiError> {
        let tx_id_string = format!("mock_tx_{}", get_current_time_ms());
        let tx_id = CausalityTransactionId(tx_id_string);
        let current_block = *self.block_height.lock().unwrap();
        let now = get_current_time_ms();

        let transaction = CausalityTransaction {
            id: tx_id.clone(),
            status: TransactionStatus::Included {
                block_number: Some(current_block),
                block_hash: Some(BlockId([current_block as u8; 32])),
                timestamp: now,
            },
            block_hash: Some(BlockId([current_block as u8; 32])),
            block_number: Some(current_block),
            payload,
            timestamp: Some(now),
            metadata: None,
        };
        self.transactions
            .lock()
            .unwrap()
            .insert(tx_id.clone(), transaction);
        Ok(tx_id)
    }

    async fn balance(&self, asset_id: Option<String>) -> Result<String, ApiError> {
        let balances = self.balances.lock().unwrap();
        match balances.get(&asset_id) {
            Some(amount) => Ok(amount.to_string()),
            None => {
                if asset_id.is_none() {
                    Err(ApiError::new(
                        ChainClientError::Unknown,
                        "Native balance not found for mock client".to_string(),
                        None,
                    ))
                } else {
                    Ok("0".to_string())
                }
            }
        }
    }

    async fn get_block_height(&self) -> Result<u64, ApiError> {
        Ok(*self.block_height.lock().unwrap())
    }

    async fn get_transaction_status(
        &self,
        tx_id: &CausalityTransactionId,
    ) -> Result<TransactionStatus, ApiError> {
        let transactions = self.transactions.lock().unwrap();
        if let Some(transaction) = transactions.get(tx_id) {
            Ok(transaction.status.clone())
        } else {
            Ok(TransactionStatus::NotFound)
        }
    }

    async fn get_transaction(
        &self,
        tx_id: &CausalityTransactionId,
    ) -> Result<Option<CausalityTransaction>, ApiError> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions.get(tx_id).cloned())
    }
}

//-----------------------------------------------------------------------------
// Mock Intent Handler
//-----------------------------------------------------------------------------

/// Mock implementation of an intent handler for testing
pub struct MockIntentHandler {
    /// The underlying chain client
    client: Arc<MockChainClient>,
}

impl MockIntentHandler {
    /// Create a new mock intent handler
    pub fn new(client: Arc<MockChainClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IntentHandler for MockIntentHandler {
    async fn submit_intent(&self, intent: Intent) -> Result<IntentId, ApiError> {
        Ok(self.client.register_intent_mock(intent))
    }

    async fn query_intent(
        &self,
        intent_id: &IntentId,
    ) -> Result<IntentQueryResult, ApiError> {
        let intents = self.client.intents.lock().unwrap();
        let intent_data = intents.get(intent_id).cloned();

        let metadata_map = self.client.intent_metadata.lock().unwrap();
        let metadata = metadata_map.get(intent_id).cloned();

        if metadata.is_none() {
            return Ok(IntentQueryResult {
                intent: None,
                status: IntentStatus::Unknown,
                transaction_id: None,
                metadata: None,
            });
        }

        let current_metadata = metadata.unwrap();

        let intent_instance = if let Some(data) = &intent_data {
            Some(Intent::from_ssz_bytes(data.as_slice()).map_err(|e| {
                let error_message = format!("Failed to deserialize intent: {}", e.message);
                ApiError::new(
                    ChainClientError::DecodingError(error_message.clone()),
                    error_message,
                    None,
                )
            })?)
        } else {
            None
        };

        Ok(IntentQueryResult {
            intent: intent_instance,
            status: current_metadata.status.clone(),
            transaction_id: Some(CausalityTransactionId(
                current_metadata.tx_hash.clone(),
            )),
            metadata: Some(current_metadata),
        })
    }

    async fn update_intent_status(
        &self,
        intent_id: &IntentId,
        status: IntentStatus,
    ) -> Result<(), ApiError> {
        self.client.update_intent_status_mock(intent_id, status);
        Ok(())
    }
}
