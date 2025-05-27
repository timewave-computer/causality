//! Core Chain Types and Interfaces
//!
//! This module defines the foundational types and interfaces for blockchain interactions in the
//! Causality system. These types use bounded sizes and fixed-width fields where possible to
//! ensure compatibility with zero-knowledge proofs and deterministic behavior.
//!
//! ## Type Organization
//!
//! * **Core Types**: `BlockId`, `ChainId`, `TransactionStatus`
//! * **Transaction Types**: `CausalityTransaction`, `CausalityTransactionId`
//! * **Error Types**: `ChainClientError`, `ApiError`
//! * **Client Interface**: `ChainClient` trait definition

use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::hash::Hash;

//-----------------------------------------------------------------------------
// Core Chain Types
//-----------------------------------------------------------------------------

/// Chain-agnostic block identifier with a fixed size for ZK compatibility
///
/// This is typically a block hash, with a fixed size of 32 bytes to ensure
/// consistent memory layout and serialization.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default,
)]
pub struct BlockId(pub [u8; 32]);

/// Chain identifier with numeric representation
///
/// Represents a specific blockchain network (e.g., Ethereum mainnet = 1,
/// Polygon = 137). This type provides a consistent way to identify chains
/// across the Causality system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainId(pub u32);

/// Status of a transaction on a blockchain
///
/// This enum represents the lifecycle states of a transaction, from
/// initial submission through inclusion and confirmation, or failure states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction has been submitted but not yet included in a block
    Pending,

    /// Transaction has been included in a block but not yet finalized
    Included {
        /// Block number where the transaction was included
        block_number: Option<u64>,
        /// Block hash where the transaction was included
        block_hash: Option<BlockId>,
        /// Timestamp of inclusion
        timestamp: u64,
    },

    /// Transaction has been finalized and confirmed
    Confirmed {
        /// Block number where the transaction was finalized
        block_number: Option<u64>,
        /// Block hash where the transaction was finalized
        block_hash: Option<BlockId>,
        /// Timestamp of confirmation
        timestamp: u64,
    },

    /// Transaction failed
    Failed {
        /// Error message
        error: String,
        /// Timestamp of failure
        timestamp: u64,
    },

    /// Transaction was rejected
    Rejected {
        /// Reason for rejection
        reason: String,
        /// Timestamp of rejection
        timestamp: u64,
    },
    /// Transaction not found (useful for get_transaction_status)
    NotFound,
}

impl TransactionStatus {
    /// Check if the transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self, TransactionStatus::Confirmed { .. })
    }

    /// Get the block hash if available
    pub fn block_hash(&self) -> Option<BlockId> {
        match self {
            TransactionStatus::Included { block_hash, .. } => *block_hash,
            TransactionStatus::Confirmed { block_hash, .. } => *block_hash,
            _ => None,
        }
    }

    /// Get the block number if available
    pub fn block_number(&self) -> Option<u64> {
        match self {
            TransactionStatus::Included { block_number, .. } => *block_number,
            TransactionStatus::Confirmed { block_number, .. } => *block_number,
            _ => None,
        }
    }

    /// Shortcut property for backward compatibility
    pub fn confirmed(&self) -> bool {
        self.is_confirmed()
    }
}

//-----------------------------------------------------------------------------
// Causality Specific Chain Types
//-----------------------------------------------------------------------------

/// Represents a unique transaction ID on a specific chain.
/// This is often a hash, but represented as a String for broader compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CausalityTransactionId(pub String);

/// Represents a transaction within the Causality system.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalityTransaction {
    pub id: CausalityTransactionId,
    pub status: TransactionStatus,
    pub block_hash: Option<BlockId>,
    pub block_number: Option<u64>,
    pub payload: Vec<u8>,
    pub timestamp: Option<u64>,
    pub metadata: Option<String>, // Placeholder for future use
}

//-----------------------------------------------------------------------------
// Error Types for Chain Interactions
//-----------------------------------------------------------------------------

/// Represents errors that can occur during chain client operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainClientError {
    ConnectionError(String),
    RpcError(String),
    TimeoutError(String),
    AuthenticationError(String),
    NotFoundError(String), // e.g. transaction not found, block not found
    InvalidInput(String),  // e.g. malformed address, invalid payload
    InsufficientFunds(String),
    TransactionFailed(String), // e.g. reverted, out of gas
    RateLimitExceeded(String),
    DecodingError(String), // Error deserializing response or on-chain data
    EncodingError(String), // Error serializing request data - ADDED
    Unsupported,           // e.g. feature not supported by this chain
    Unknown,               // For any other errors
    Custom(String), // For crate-specific errors that don't fit above - Assuming this was intended
}

/// A general API error structure for chain interactions.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub kind: ChainClientError,
    pub message: String,
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(
        kind: ChainClientError,
        message: String,
        details: Option<String>,
    ) -> Self {
        Self {
            kind,
            message,
            details,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}

//-----------------------------------------------------------------------------
// Chain Client Trait
//-----------------------------------------------------------------------------

use async_trait::async_trait;

/// Defines the interface for a chain client.
/// This trait allows interaction with different blockchains in a generic way.
#[async_trait]
pub trait ChainClient: Send + Sync {
    /// Returns the chain ID this client is configured for.
    fn chain_id(&self) -> ChainId;

    /// Retrieves the primary address associated with the client's signer/wallet.
    async fn address(&self) -> Result<String, ApiError>;

    /// Retrieves the balance of a given asset for the client's primary address.
    /// If `asset_id` is None, it should fetch the balance of the native chain asset.
    async fn balance(&self, asset_id: Option<String>) -> Result<String, ApiError>;

    /// Retrieves the current block height of the blockchain.
    async fn get_block_height(&self) -> Result<u64, ApiError>;

    /// Retrieves the status of a transaction by its `CausalityTransactionId`.
    async fn get_transaction_status(
        &self,
        tx_id: &CausalityTransactionId,
    ) -> Result<TransactionStatus, ApiError>;

    /// Submits a transaction payload to the blockchain.
    /// The payload is chain-specific.
    /// Returns a `CausalityTransactionId` for the submitted transaction.
    async fn submit_transaction(
        &self,
        payload: Vec<u8>,
        // metadata: Option<TransactionMetadata>, // Consider adding metadata if needed
    ) -> Result<CausalityTransactionId, ApiError>;

    /// Retrieves the details of a transaction by its `CausalityTransactionId`.
    /// Returns `Ok(None)` if the transaction is not found.
    async fn get_transaction(
        &self,
        tx_id: &CausalityTransactionId,
    ) -> Result<Option<CausalityTransaction>, ApiError>;
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// BlockId
impl Encode for BlockId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for BlockId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 32 {
            return Err(DecodeError { message: format!("BlockId requires exactly 32 bytes, got {}", bytes.len()) });
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(BlockId(array))
    }
}

impl SimpleSerialize for BlockId {}

// ChainId
impl Encode for ChainId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}

impl Decode for ChainId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let id = u32::from_ssz_bytes(bytes)
            .map_err(|e| DecodeError { message: format!("Failed to decode ChainId: {}", e) })?;
        Ok(ChainId(id))
    }
}

impl SimpleSerialize for ChainId {}

// TransactionStatus
impl Encode for TransactionStatus {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        match self {
            TransactionStatus::Pending => {
                bytes.push(0u8);
            }
            TransactionStatus::Included { block_number, block_hash, timestamp } => {
                bytes.push(1u8);
                
                // Serialize block_number as Option
                match block_number {
                    Some(num) => {
                        bytes.push(1u8);
                        bytes.extend(num.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
                
                // Serialize block_hash as Option
                match block_hash {
                    Some(hash) => {
                        bytes.push(1u8);
                        bytes.extend(hash.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
                
                bytes.extend(timestamp.as_ssz_bytes());
            }
            TransactionStatus::Confirmed { block_number, block_hash, timestamp } => {
                bytes.push(2u8);
                
                // Serialize block_number as Option
                match block_number {
                    Some(num) => {
                        bytes.push(1u8);
                        bytes.extend(num.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
                
                // Serialize block_hash as Option
                match block_hash {
                    Some(hash) => {
                        bytes.push(1u8);
                        bytes.extend(hash.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
                
                bytes.extend(timestamp.as_ssz_bytes());
            }
            TransactionStatus::Failed { error, timestamp } => {
                bytes.push(3u8);
                bytes.extend(error.as_ssz_bytes());
                bytes.extend(timestamp.as_ssz_bytes());
            }
            TransactionStatus::Rejected { reason, timestamp } => {
                bytes.push(4u8);
                bytes.extend(reason.as_ssz_bytes());
                bytes.extend(timestamp.as_ssz_bytes());
            }
            TransactionStatus::NotFound => {
                bytes.push(5u8);
            }
        }
        
        bytes
    }
}

impl Decode for TransactionStatus {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for TransactionStatus".to_string() });
        }
        
        let variant = bytes[0];
        let mut offset = 1;
        
        match variant {
            0 => Ok(TransactionStatus::Pending),
            1 | 2 => {
                // Decode block_number option
                let has_block_number = bytes[offset];
                offset += 1;
                
                let block_number = if has_block_number == 1 {
                    let num = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                        .map_err(|e| DecodeError { message: format!("Failed to decode block_number: {}", e) })?;
                    offset += 8;
                    Some(num)
                } else {
                    None
                };
                
                // Decode block_hash option
                let has_block_hash = bytes[offset];
                offset += 1;
                
                let block_hash = if has_block_hash == 1 {
                    let hash = BlockId::from_ssz_bytes(&bytes[offset..offset + 32])
                        .map_err(|e| DecodeError { message: format!("Failed to decode block_hash: {}", e) })?;
                    offset += 32;
                    Some(hash)
                } else {
                    None
                };
                
                let timestamp = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                    .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
                
                if variant == 1 {
                    Ok(TransactionStatus::Included { block_number, block_hash, timestamp })
                } else {
                    Ok(TransactionStatus::Confirmed { block_number, block_hash, timestamp })
                }
            }
            3 => {
                let error = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode error: {}", e) })?;
                let error_size = error.as_ssz_bytes().len();
                offset += error_size;
                
                let timestamp = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                    .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
                
                Ok(TransactionStatus::Failed { error, timestamp })
            }
            4 => {
                let reason = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode reason: {}", e) })?;
                let reason_size = reason.as_ssz_bytes().len();
                offset += reason_size;
                
                let timestamp = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                    .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
                
                Ok(TransactionStatus::Rejected { reason, timestamp })
            }
            5 => Ok(TransactionStatus::NotFound),
            other => Err(DecodeError { message: format!("Invalid TransactionStatus variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for TransactionStatus {}

// CausalityTransactionId
impl Encode for CausalityTransactionId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}

impl Decode for CausalityTransactionId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let id = String::from_ssz_bytes(bytes)
            .map_err(|e| DecodeError { message: format!("Failed to decode CausalityTransactionId: {}", e) })?;
        Ok(CausalityTransactionId(id))
    }
}

impl SimpleSerialize for CausalityTransactionId {}

// CausalityTransaction
impl Encode for CausalityTransaction {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.status.as_ssz_bytes());
        
        // Serialize block_hash as Option
        match &self.block_hash {
            Some(hash) => {
                bytes.push(1u8);
                bytes.extend(hash.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize block_number as Option
        match self.block_number {
            Some(num) => {
                bytes.push(1u8);
                bytes.extend(num.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize payload
        bytes.extend((self.payload.len() as u64).as_ssz_bytes());
        bytes.extend(&self.payload);
        
        // Serialize timestamp as Option
        match self.timestamp {
            Some(ts) => {
                bytes.push(1u8);
                bytes.extend(ts.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize metadata as Option
        match &self.metadata {
            Some(meta) => {
                bytes.push(1u8);
                bytes.extend(meta.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for CausalityTransaction {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode id
        let id = CausalityTransactionId::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode id: {}", e) })?;
        let id_size = id.as_ssz_bytes().len();
        offset += id_size;
        
        // Decode status
        let status = TransactionStatus::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode status: {}", e) })?;
        let status_size = status.as_ssz_bytes().len();
        offset += status_size;
        
        // Decode block_hash option
        let has_block_hash = bytes[offset];
        offset += 1;
        
        let block_hash = if has_block_hash == 1 {
            let hash = BlockId::from_ssz_bytes(&bytes[offset..offset + 32])
                .map_err(|e| DecodeError { message: format!("Failed to decode block_hash: {}", e) })?;
            offset += 32;
            Some(hash)
        } else {
            None
        };
        
        // Decode block_number option
        let has_block_number = bytes[offset];
        offset += 1;
        
        let block_number = if has_block_number == 1 {
            let num = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode block_number: {}", e) })?;
            offset += 8;
            Some(num)
        } else {
            None
        };
        
        // Decode payload
        let payload_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode payload length: {}", e) })? as usize;
        offset += 8;
        
        let payload = bytes[offset..offset + payload_len].to_vec();
        offset += payload_len;
        
        // Decode timestamp option
        let has_timestamp = bytes[offset];
        offset += 1;
        
        let timestamp = if has_timestamp == 1 {
            let ts = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
            offset += 8;
            Some(ts)
        } else {
            None
        };
        
        // Decode metadata option
        let has_metadata = bytes[offset];
        offset += 1;
        
        let metadata = if has_metadata == 1 {
            let meta = String::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode metadata: {}", e) })?;
            Some(meta)
        } else {
            None
        };
        
        Ok(CausalityTransaction {
            id,
            status,
            block_hash,
            block_number,
            payload,
            timestamp,
            metadata,
        })
    }
}

impl SimpleSerialize for CausalityTransaction {}

// ChainClientError
impl Encode for ChainClientError {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        match self {
            ChainClientError::ConnectionError(msg) => {
                bytes.push(0u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::RpcError(msg) => {
                bytes.push(1u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::TimeoutError(msg) => {
                bytes.push(2u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::AuthenticationError(msg) => {
                bytes.push(3u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::NotFoundError(msg) => {
                bytes.push(4u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::InvalidInput(msg) => {
                bytes.push(5u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::InsufficientFunds(msg) => {
                bytes.push(6u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::TransactionFailed(msg) => {
                bytes.push(7u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::RateLimitExceeded(msg) => {
                bytes.push(8u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::DecodingError(msg) => {
                bytes.push(9u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::EncodingError(msg) => {
                bytes.push(10u8);
                bytes.extend(msg.as_ssz_bytes());
            }
            ChainClientError::Unsupported => {
                bytes.push(11u8);
            }
            ChainClientError::Unknown => {
                bytes.push(12u8);
            }
            ChainClientError::Custom(msg) => {
                bytes.push(13u8);
                bytes.extend(msg.as_ssz_bytes());
            }
        }
        
        bytes
    }
}

impl Decode for ChainClientError {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for ChainClientError".to_string() });
        }
        
        let variant = bytes[0];
        let offset = 1;
        
        match variant {
            0 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode ConnectionError message: {}", e) })?;
                Ok(ChainClientError::ConnectionError(msg))
            }
            1 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode RpcError message: {}", e) })?;
                Ok(ChainClientError::RpcError(msg))
            }
            2 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode TimeoutError message: {}", e) })?;
                Ok(ChainClientError::TimeoutError(msg))
            }
            3 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode AuthenticationError message: {}", e) })?;
                Ok(ChainClientError::AuthenticationError(msg))
            }
            4 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode NotFoundError message: {}", e) })?;
                Ok(ChainClientError::NotFoundError(msg))
            }
            5 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode InvalidInput message: {}", e) })?;
                Ok(ChainClientError::InvalidInput(msg))
            }
            6 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode InsufficientFunds message: {}", e) })?;
                Ok(ChainClientError::InsufficientFunds(msg))
            }
            7 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode TransactionFailed message: {}", e) })?;
                Ok(ChainClientError::TransactionFailed(msg))
            }
            8 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode RateLimitExceeded message: {}", e) })?;
                Ok(ChainClientError::RateLimitExceeded(msg))
            }
            9 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode DecodingError message: {}", e) })?;
                Ok(ChainClientError::DecodingError(msg))
            }
            10 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode EncodingError message: {}", e) })?;
                Ok(ChainClientError::EncodingError(msg))
            }
            11 => Ok(ChainClientError::Unsupported),
            12 => Ok(ChainClientError::Unknown),
            13 => {
                let msg = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode Custom message: {}", e) })?;
                Ok(ChainClientError::Custom(msg))
            }
            other => Err(DecodeError { message: format!("Invalid ChainClientError variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for ChainClientError {}

// ApiError
impl Encode for ApiError {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.kind.as_ssz_bytes());
        bytes.extend(self.message.as_ssz_bytes());
        
        // Serialize details as Option
        match &self.details {
            Some(details) => {
                bytes.push(1u8);
                bytes.extend(details.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for ApiError {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode kind
        let kind = ChainClientError::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode kind: {}", e) })?;
        let kind_size = kind.as_ssz_bytes().len();
        offset += kind_size;
        
        // Decode message
        let message = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode message: {}", e) })?;
        let message_size = message.as_ssz_bytes().len();
        offset += message_size;
        
        // Decode details option
        let has_details = bytes[offset];
        offset += 1;
        
        let details = if has_details == 1 {
            let details_str = String::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode details: {}", e) })?;
            Some(details_str)
        } else {
            None
        };
        
        Ok(ApiError {
            kind,
            message,
            details,
        })
    }
}

impl SimpleSerialize for ApiError {}
