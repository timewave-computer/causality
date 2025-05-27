//! Blockchain Intent Handling
//!
//! This module provides a standardized interface for intent operations
//! across different blockchains with bounded types for ZK compatibility.
//! All serialization uses SSZ exclusively for deterministic behavior.

use async_trait::async_trait;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::chain::types::{ApiError, CausalityTransactionId, ChainClientError};
use crate::chain::ChainClient;
use causality_types::primitive::ids::IntentId;
use causality_types::core::Intent;

//-----------------------------------------------------------------------------
// Types and Interface
//-----------------------------------------------------------------------------

/// Status of an intent on a blockchain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentStatus {
    /// Intent is pending execution
    Pending,

    /// Intent has been executed
    Executed,

    /// Intent has been rejected
    Rejected,

    /// Intent has expired
    Expired,

    /// Intent status is unknown
    Unknown,
}

/// Metadata for an intent stored on a blockchain
#[derive(Debug, Clone, PartialEq)]
pub struct IntentMetadata {
    /// Block height when the intent was submitted
    pub block_height: u64,

    /// Timestamp when the intent was submitted (in seconds since epoch)
    pub timestamp: u64,

    /// Transaction hash that submitted the intent
    pub tx_hash: String,

    /// Status of the intent on chain
    pub status: IntentStatus,
}

/// Query result for an intent
#[derive(Debug, Clone, PartialEq)]
pub struct IntentQueryResult {
    /// The intent that was queried
    pub intent: Option<Intent>,

    /// Status of the intent
    pub status: IntentStatus,

    /// Transaction ID of the intent
    pub transaction_id: Option<CausalityTransactionId>,

    /// Metadata for the intent if found
    pub metadata: Option<IntentMetadata>,
}

/// Interface for intent handling across different blockchains
#[async_trait]
pub trait IntentHandler: Send + Sync {
    /// Submit an intent to the blockchain
    async fn submit_intent(&self, intent: Intent) -> Result<IntentId, ApiError>;

    /// Query an intent by ID
    async fn query_intent(
        &self,
        intent_id: &IntentId,
    ) -> Result<IntentQueryResult, ApiError>;

    /// Update the status of an intent
    async fn update_intent_status(
        &self,
        intent_id: &IntentId,
        status: IntentStatus,
    ) -> Result<(), ApiError>;
}

//-----------------------------------------------------------------------------
// Implementation using Chain Client
//-----------------------------------------------------------------------------

/// Computes an IntentId from an Intent object.
fn compute_intent_id_from_intent(intent: &Intent) -> IntentId {
    let intent_data = intent.as_ssz_bytes();
    let hash = Sha256::digest(&intent_data);
    IntentId::new(hash.into())
}

/// Intent handler that uses a generic chain client for blockchain operations
pub struct ChainIntentHandler<C: ChainClient> {
    /// The underlying chain client
    client: Arc<C>,
}

impl<C: ChainClient> ChainIntentHandler<C> {
    /// Create a new chain intent handler
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ChainClient + 'static> IntentHandler for ChainIntentHandler<C> {
    async fn submit_intent(&self, intent: Intent) -> Result<IntentId, ApiError> {
        let intent_data = intent.as_ssz_bytes();

        let _tx_id = self.client.submit_transaction(intent_data).await?;

        Ok(compute_intent_id_from_intent(&intent))
    }

    async fn query_intent(
        &self,
        intent_id: &IntentId,
    ) -> Result<IntentQueryResult, ApiError> {
        eprintln!(
            "[ChainIntentHandler] query_intent for {:?} called on a generic client. This will not retrieve intent data.",
            intent_id
        );
        Ok(IntentQueryResult {
            intent: None,
            status: IntentStatus::Unknown,
            transaction_id: None,
            metadata: None,
        })
    }

    async fn update_intent_status(
        &self,
        _intent_id: &IntentId,
        _status: IntentStatus,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ChainClientError::Unsupported,
            "update_intent_status is not generically supported. Requires specific chain logic."
                .to_string(),
            None,
        ))
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// IntentStatus
impl Encode for IntentStatus {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            IntentStatus::Pending => vec![0u8],
            IntentStatus::Executed => vec![1u8],
            IntentStatus::Rejected => vec![2u8],
            IntentStatus::Expired => vec![3u8],
            IntentStatus::Unknown => vec![4u8],
        }
    }
}

impl Decode for IntentStatus {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for IntentStatus".to_string() });
        }
        
        match bytes[0] {
            0 => Ok(IntentStatus::Pending),
            1 => Ok(IntentStatus::Executed),
            2 => Ok(IntentStatus::Rejected),
            3 => Ok(IntentStatus::Expired),
            4 => Ok(IntentStatus::Unknown),
            other => Err(DecodeError { message: format!("Invalid IntentStatus variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for IntentStatus {}

// IntentMetadata
impl Encode for IntentMetadata {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.block_height.as_ssz_bytes());
        bytes.extend(self.timestamp.as_ssz_bytes());
        bytes.extend(self.tx_hash.as_ssz_bytes());
        bytes.extend(self.status.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for IntentMetadata {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let block_height = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode block_height: {}", e) })?;
        offset += 8;
        
        let timestamp = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
        offset += 8;
        
        let tx_hash = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode tx_hash: {}", e) })?;
        let tx_hash_size = tx_hash.as_ssz_bytes().len();
        offset += tx_hash_size;
        
        let status = IntentStatus::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode status: {}", e) })?;
        
        Ok(IntentMetadata {
            block_height,
            timestamp,
            tx_hash,
            status,
        })
    }
}

impl SimpleSerialize for IntentMetadata {}

// IntentQueryResult
impl Encode for IntentQueryResult {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize intent as Option
        match &self.intent {
            Some(intent) => {
                bytes.push(1u8);
                bytes.extend(intent.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes.extend(self.status.as_ssz_bytes());
        
        // Serialize transaction_id as Option
        match &self.transaction_id {
            Some(tx_id) => {
                bytes.push(1u8);
                bytes.extend(tx_id.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize metadata as Option
        match &self.metadata {
            Some(metadata) => {
                bytes.push(1u8);
                bytes.extend(metadata.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for IntentQueryResult {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode intent option
        let has_intent = bytes[offset];
        offset += 1;
        
        let intent = if has_intent == 1 {
            let intent_obj = Intent::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode intent: {}", e) })?;
            let intent_size = intent_obj.as_ssz_bytes().len();
            offset += intent_size;
            Some(intent_obj)
        } else {
            None
        };
        
        // Decode status
        let status = IntentStatus::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode status: {}", e) })?;
        let status_size = status.as_ssz_bytes().len();
        offset += status_size;
        
        // Decode transaction_id option
        let has_transaction_id = bytes[offset];
        offset += 1;
        
        let transaction_id = if has_transaction_id == 1 {
            let tx_id = CausalityTransactionId::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode transaction_id: {}", e) })?;
            let tx_id_size = tx_id.as_ssz_bytes().len();
            offset += tx_id_size;
            Some(tx_id)
        } else {
            None
        };
        
        // Decode metadata option
        let has_metadata = bytes[offset];
        offset += 1;
        
        let metadata = if has_metadata == 1 {
            let metadata_obj = IntentMetadata::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode metadata: {}", e) })?;
            Some(metadata_obj)
        } else {
            None
        };
        
        Ok(IntentQueryResult {
            intent,
            status,
            transaction_id,
            metadata,
        })
    }
}

impl SimpleSerialize for IntentQueryResult {}
