//! Models for Blockchain Interactions
//!
//! This module provides the data structures used for blockchain interactions, including:
//!
//! * Transaction input/output models
//! * Query request/response models
//! * Intent submission and retrieval models
//! * Chain-specific data structures
//!
//! The models in this module are designed to be serializable with SSZ for
//! consistent encoding across the system. They serve as the boundary objects
//! between the Causality system and external blockchain networks.
//!
//! ## Organization
//!
//! * **Core Types**: Re-exported from valence-core
//! * **Intent Models**: Models for intent submission and querying
//! * **Chain-Specific Models**: Ethereum and Neutron transaction and query models

// Re-export core types
pub use valence_core::types::*;

// Re-export blockchain-specific types with feature flags
#[cfg(feature = "neutron")]
pub use valence_cosmos::types as cosmos_types;
#[cfg(feature = "ethereum")]
pub use valence_evm::types as evm_types;

// Import common transaction types
pub use valence_core::transaction::{Event, EventAttribute, TransactionResponse};

// Import error type
pub use valence_core::error::ClientError;

// Additional models specific to causality-api that may not be in valence-domain-clients
use causality_types::primitive::ids::IntentId;
use causality_types::core::Intent;
use crate::serialization::{SimpleSerialize, Encode, Decode, DecodeError};

//-----------------------------------------------------------------------------
// Intent Query & Submission Models
//-----------------------------------------------------------------------------

/// Input for querying an intent by ID
#[derive(Debug, Clone)]
pub struct IntentQueryInput {
    /// Intent ID to query
    pub _intent_id: IntentId,

    /// Optional chain-specific query parameters
    pub chain_params: Option<Vec<u8>>,
}

/// Output from an intent query
#[derive(Debug, Clone)]
pub struct IntentQueryOutput {
    /// The intent that was retrieved, if found
    pub intent: Option<Intent>,

    /// Chain-specific metadata about the intent
    pub metadata: IntentMetadata,
}

/// Metadata for an intent stored on a blockchain
#[derive(Debug, Clone)]
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

//-----------------------------------------------------------------------------
// Intent Status
//-----------------------------------------------------------------------------

/// Status of an intent on chain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentStatus {
    /// Intent is pending execution
    Pending,

    /// Intent execution is in progress
    InProgress,

    /// Intent has been successfully executed
    Executed,

    /// Intent execution failed
    Failed,

    /// Intent was rejected
    Rejected,
}

impl Encode for IntentStatus {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            IntentStatus::Pending => vec![0u8],
            IntentStatus::InProgress => vec![1u8],
            IntentStatus::Executed => vec![2u8],
            IntentStatus::Failed => vec![3u8],
            IntentStatus::Rejected => vec![4u8],
        }
    }
}

impl Decode for IntentStatus {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 1 {
            return Err(DecodeError {
                message: format!("IntentStatus requires exactly 1 byte, got {}", bytes.len()),
            });
        }
        
        match bytes[0] {
            0 => Ok(IntentStatus::Pending),
            1 => Ok(IntentStatus::InProgress),
            2 => Ok(IntentStatus::Executed),
            3 => Ok(IntentStatus::Failed),
            4 => Ok(IntentStatus::Rejected),
            _ => Err(DecodeError {
                message: format!("Invalid IntentStatus variant: {}", bytes[0]),
            }),
        }
    }
}

impl SimpleSerialize for IntentStatus {}

/// Input for submitting an intent to a blockchain
#[derive(Debug, Clone)]
pub struct IntentSubmissionInput {
    /// The intent to submit
    pub intent: Intent,

    /// Optional chain-specific parameters for the submission
    pub chain_params: Option<Vec<u8>>,
}

/// Output from an intent submission
#[derive(Debug, Clone)]
pub struct IntentSubmissionOutput {
    /// ID of the submitted intent
    pub _intent_id: IntentId,

    /// Transaction hash of the submission
    pub tx_hash: String,

    /// Block height when the intent was submitted
    pub block_height: u64,

    /// Fees paid for the submission
    pub fees_paid: Option<u128>,

    /// Status of the intent after submission
    pub status: IntentStatus,
}

//-----------------------------------------------------------------------------
// Neutron Transaction & Query Models
//-----------------------------------------------------------------------------

/// Input model for Neutron transactions
#[derive(Debug, Clone)]
pub struct NeutronTransactionInput {
    /// Target contract address
    pub contract: String,

    /// Call data (serialized proto message)
    pub data: Vec<u8>,
}

/// Output model for Neutron transactions
#[derive(Debug, Clone)]
pub struct NeutronTransactionOutput {
    /// Transaction hash
    pub tx_hash: String,

    /// Block height when the transaction was included
    pub block_height: u64,
}

/// Input model for Neutron queries
#[derive(Debug, Clone)]
pub struct NeutronQueryInput {
    /// Contract address to query
    pub contract: String,

    /// Query data
    pub data: Vec<u8>,
}

/// Output model for Neutron queries
#[derive(Debug, Clone)]
pub struct NeutronQueryOutput {
    /// Block height at which the query was executed
    pub height: u64,

    /// Query result data
    pub data: Vec<u8>,
}

//-----------------------------------------------------------------------------
// Ethereum Transaction & Query Models
//-----------------------------------------------------------------------------

/// Input model for Ethereum transactions
#[derive(Debug, Clone)]
pub struct EthereumTransactionInput {
    /// Target contract address (if any)
    pub to: Option<String>,

    /// Call data
    pub data: Vec<u8>,

    /// Value in wei to send
    pub value: u128,

    /// Gas limit for the transaction
    pub gas_limit: u64,

    /// Gas price for the transaction
    pub gas_price: u64,
}

/// Output model for Ethereum transactions
#[derive(Debug, Clone)]
pub struct EthereumTransactionOutput {
    /// Transaction hash
    pub tx_hash: String,

    /// Block number when the transaction was included
    pub block_number: u64,

    /// Gas used by the transaction
    pub gas_used: u64,

    /// Whether the transaction was successful
    pub success: bool,
}

/// Input model for Ethereum queries
#[derive(Debug, Clone)]
pub enum EthereumQueryInput {
    /// Standard contract call
    Call {
        /// Target contract address
        to: String,

        /// Call data
        data: Vec<u8>,

        /// Optional block number to query at
        block: Option<u64>,
    },

    /// Get balance query
    GetBalance {
        /// Address to check balance for
        address: String,

        /// Optional block number to query at
        block: Option<u64>,
    },

    /// Get storage slot query
    GetStorageAt {
        /// Contract address
        address: String,

        /// Storage slot
        slot: String,

        /// Optional block number
        block: Option<u64>,
    },

    /// Get block information
    GetBlock {
        /// Block number or hash
        block_identifier: String,

        /// Include full transaction objects
        full_transactions: bool,
    },
}

/// Output model for Ethereum queries
#[derive(Debug, Clone)]
pub struct EthereumQueryOutput {
    /// Query result data
    pub data: Vec<u8>,
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// IntentQueryInput
impl Encode for IntentQueryInput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self._intent_id.as_ssz_bytes());
        
        // Serialize chain_params as Option
        match &self.chain_params {
            Some(params) => {
                bytes.push(1u8);
                bytes.extend((params.len() as u64).as_ssz_bytes());
                bytes.extend(params);
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for IntentQueryInput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let _intent_id = IntentId::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode _intent_id: {}", e) })?;
        let intent_id_size = _intent_id.as_ssz_bytes().len();
        offset += intent_id_size;
        
        // Decode chain_params option
        let has_chain_params = bytes[offset];
        offset += 1;
        
        let chain_params = if has_chain_params == 1 {
            let params_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode chain_params length: {}", e) })? as usize;
            offset += 8;
            Some(bytes[offset..offset + params_len].to_vec())
        } else {
            None
        };
        
        Ok(IntentQueryInput {
            _intent_id,
            chain_params,
        })
    }
}

impl SimpleSerialize for IntentQueryInput {}

// IntentQueryOutput
impl Encode for IntentQueryOutput {
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
        
        bytes.extend(self.metadata.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for IntentQueryOutput {
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
        
        let metadata = IntentMetadata::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode metadata: {}", e) })?;
        
        Ok(IntentQueryOutput {
            intent,
            metadata,
        })
    }
}

impl SimpleSerialize for IntentQueryOutput {}

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

// IntentSubmissionInput
impl Encode for IntentSubmissionInput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.intent.as_ssz_bytes());
        
        // Serialize chain_params as Option
        match &self.chain_params {
            Some(params) => {
                bytes.push(1u8);
                bytes.extend((params.len() as u64).as_ssz_bytes());
                bytes.extend(params);
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for IntentSubmissionInput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let intent = Intent::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode intent: {}", e) })?;
        let intent_size = intent.as_ssz_bytes().len();
        offset += intent_size;
        
        // Decode chain_params option
        let has_chain_params = bytes[offset];
        offset += 1;
        
        let chain_params = if has_chain_params == 1 {
            let params_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode chain_params length: {}", e) })? as usize;
            offset += 8;
            Some(bytes[offset..offset + params_len].to_vec())
        } else {
            None
        };
        
        Ok(IntentSubmissionInput {
            intent,
            chain_params,
        })
    }
}

impl SimpleSerialize for IntentSubmissionInput {}

// IntentSubmissionOutput
impl Encode for IntentSubmissionOutput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self._intent_id.as_ssz_bytes());
        bytes.extend(self.tx_hash.as_ssz_bytes());
        bytes.extend(self.block_height.as_ssz_bytes());
        
        // Serialize fees_paid as Option
        match self.fees_paid {
            Some(fees) => {
                bytes.push(1u8);
                bytes.extend(fees.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes.extend(self.status.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for IntentSubmissionOutput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let _intent_id = IntentId::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode _intent_id: {}", e) })?;
        let intent_id_size = _intent_id.as_ssz_bytes().len();
        offset += intent_id_size;
        
        let tx_hash = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode tx_hash: {}", e) })?;
        let tx_hash_size = tx_hash.as_ssz_bytes().len();
        offset += tx_hash_size;
        
        let block_height = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode block_height: {}", e) })?;
        offset += 8;
        
        // Decode fees_paid option
        let has_fees_paid = bytes[offset];
        offset += 1;
        
        let fees_paid = if has_fees_paid == 1 {
            let fees = u128::from_ssz_bytes(&bytes[offset..offset + 16])
                .map_err(|e| DecodeError { message: format!("Failed to decode fees_paid: {}", e) })?;
            offset += 16;
            Some(fees)
        } else {
            None
        };
        
        let status = IntentStatus::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode status: {}", e) })?;
        
        Ok(IntentSubmissionOutput {
            _intent_id,
            tx_hash,
            block_height,
            fees_paid,
            status,
        })
    }
}

impl SimpleSerialize for IntentSubmissionOutput {}

// NeutronTransactionInput
impl Encode for NeutronTransactionInput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.contract.as_ssz_bytes());
        bytes.extend((self.data.len() as u64).as_ssz_bytes());
        bytes.extend(&self.data);
        
        bytes
    }
}

impl Decode for NeutronTransactionInput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let contract = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode contract: {}", e) })?;
        let contract_size = contract.as_ssz_bytes().len();
        offset += contract_size;
        
        let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
        offset += 8;
        
        let data = bytes[offset..offset + data_len].to_vec();
        
        Ok(NeutronTransactionInput {
            contract,
            data,
        })
    }
}

impl SimpleSerialize for NeutronTransactionInput {}

// NeutronTransactionOutput
impl Encode for NeutronTransactionOutput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.tx_hash.as_ssz_bytes());
        bytes.extend(self.block_height.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for NeutronTransactionOutput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let tx_hash = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode tx_hash: {}", e) })?;
        let tx_hash_size = tx_hash.as_ssz_bytes().len();
        offset += tx_hash_size;
        
        let block_height = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode block_height: {}", e) })?;
        
        Ok(NeutronTransactionOutput {
            tx_hash,
            block_height,
        })
    }
}

impl SimpleSerialize for NeutronTransactionOutput {}

// NeutronQueryInput
impl Encode for NeutronQueryInput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.contract.as_ssz_bytes());
        bytes.extend((self.data.len() as u64).as_ssz_bytes());
        bytes.extend(&self.data);
        
        bytes
    }
}

impl Decode for NeutronQueryInput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let contract = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode contract: {}", e) })?;
        let contract_size = contract.as_ssz_bytes().len();
        offset += contract_size;
        
        let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
        offset += 8;
        
        let data = bytes[offset..offset + data_len].to_vec();
        
        Ok(NeutronQueryInput {
            contract,
            data,
        })
    }
}

impl SimpleSerialize for NeutronQueryInput {}

// NeutronQueryOutput
impl Encode for NeutronQueryOutput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.height.as_ssz_bytes());
        bytes.extend((self.data.len() as u64).as_ssz_bytes());
        bytes.extend(&self.data);
        
        bytes
    }
}

impl Decode for NeutronQueryOutput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let height = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode height: {}", e) })?;
        offset += 8;
        
        let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
        offset += 8;
        
        let data = bytes[offset..offset + data_len].to_vec();
        
        Ok(NeutronQueryOutput {
            height,
            data,
        })
    }
}

impl SimpleSerialize for NeutronQueryOutput {}

// EthereumTransactionInput
impl Encode for EthereumTransactionInput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize to as Option
        match &self.to {
            Some(addr) => {
                bytes.push(1u8);
                bytes.extend(addr.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes.extend((self.data.len() as u64).as_ssz_bytes());
        bytes.extend(&self.data);
        bytes.extend(self.value.as_ssz_bytes());
        bytes.extend(self.gas_limit.as_ssz_bytes());
        bytes.extend(self.gas_price.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for EthereumTransactionInput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode to option
        let has_to = bytes[offset];
        offset += 1;
        
        let to = if has_to == 1 {
            let addr = String::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode to address: {}", e) })?;
            let addr_size = addr.as_ssz_bytes().len();
            offset += addr_size;
            Some(addr)
        } else {
            None
        };
        
        let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
        offset += 8;
        
        let data = bytes[offset..offset + data_len].to_vec();
        offset += data_len;
        
        let value = u128::from_ssz_bytes(&bytes[offset..offset + 16])
            .map_err(|e| DecodeError { message: format!("Failed to decode value: {}", e) })?;
        offset += 16;
        
        let gas_limit = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode gas_limit: {}", e) })?;
        offset += 8;
        
        let gas_price = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode gas_price: {}", e) })?;
        
        Ok(EthereumTransactionInput {
            to,
            data,
            value,
            gas_limit,
            gas_price,
        })
    }
}

impl SimpleSerialize for EthereumTransactionInput {}

// EthereumTransactionOutput
impl Encode for EthereumTransactionOutput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.tx_hash.as_ssz_bytes());
        bytes.extend(self.block_number.as_ssz_bytes());
        bytes.extend(self.gas_used.as_ssz_bytes());
        bytes.push(if self.success { 1u8 } else { 0u8 });
        
        bytes
    }
}

impl Decode for EthereumTransactionOutput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let tx_hash = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode tx_hash: {}", e) })?;
        let tx_hash_size = tx_hash.as_ssz_bytes().len();
        offset += tx_hash_size;
        
        let block_number = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode block_number: {}", e) })?;
        offset += 8;
        
        let gas_used = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode gas_used: {}", e) })?;
        offset += 8;
        
        let success = bytes[offset] == 1;
        
        Ok(EthereumTransactionOutput {
            tx_hash,
            block_number,
            gas_used,
            success,
        })
    }
}

impl SimpleSerialize for EthereumTransactionOutput {}

// EthereumQueryInput
impl Encode for EthereumQueryInput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        match self {
            EthereumQueryInput::Call { to, data, block } => {
                bytes.push(0u8);
                bytes.extend(to.as_ssz_bytes());
                bytes.extend((data.len() as u64).as_ssz_bytes());
                bytes.extend(data);
                
                // Serialize block as Option
                match block {
                    Some(b) => {
                        bytes.push(1u8);
                        bytes.extend(b.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
            }
            EthereumQueryInput::GetBalance { address, block } => {
                bytes.push(1u8);
                bytes.extend(address.as_ssz_bytes());
                
                // Serialize block as Option
                match block {
                    Some(b) => {
                        bytes.push(1u8);
                        bytes.extend(b.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
            }
            EthereumQueryInput::GetStorageAt { address, slot, block } => {
                bytes.push(2u8);
                bytes.extend(address.as_ssz_bytes());
                bytes.extend(slot.as_ssz_bytes());
                
                // Serialize block as Option
                match block {
                    Some(b) => {
                        bytes.push(1u8);
                        bytes.extend(b.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8);
                    }
                }
            }
            EthereumQueryInput::GetBlock { block_identifier, full_transactions } => {
                bytes.push(3u8);
                bytes.extend(block_identifier.as_ssz_bytes());
                bytes.push(if *full_transactions { 1u8 } else { 0u8 });
            }
        }
        
        bytes
    }
}

impl Decode for EthereumQueryInput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for EthereumQueryInput".to_string() });
        }
        
        let variant = bytes[0];
        let mut offset = 1;
        
        match variant {
            0 => {
                let to = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode to: {}", e) })?;
                let to_size = to.as_ssz_bytes().len();
                offset += to_size;
                
                let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                    .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
                offset += 8;
                
                let data = bytes[offset..offset + data_len].to_vec();
                offset += data_len;
                
                // Decode block option
                let has_block = bytes[offset];
                offset += 1;
                
                let block = if has_block == 1 {
                    let b = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                        .map_err(|e| DecodeError { message: format!("Failed to decode block: {}", e) })?;
                    Some(b)
                } else {
                    None
                };
                
                Ok(EthereumQueryInput::Call { to, data, block })
            }
            1 => {
                let address = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode address: {}", e) })?;
                let address_size = address.as_ssz_bytes().len();
                offset += address_size;
                
                // Decode block option
                let has_block = bytes[offset];
                offset += 1;
                
                let block = if has_block == 1 {
                    let b = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                        .map_err(|e| DecodeError { message: format!("Failed to decode block: {}", e) })?;
                    Some(b)
                } else {
                    None
                };
                
                Ok(EthereumQueryInput::GetBalance { address, block })
            }
            2 => {
                let address = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode address: {}", e) })?;
                let address_size = address.as_ssz_bytes().len();
                offset += address_size;
                
                let slot = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode slot: {}", e) })?;
                let slot_size = slot.as_ssz_bytes().len();
                offset += slot_size;
                
                // Decode block option
                let has_block = bytes[offset];
                offset += 1;
                
                let block = if has_block == 1 {
                    let b = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                        .map_err(|e| DecodeError { message: format!("Failed to decode block: {}", e) })?;
                    Some(b)
                } else {
                    None
                };
                
                Ok(EthereumQueryInput::GetStorageAt { address, slot, block })
            }
            3 => {
                let block_identifier = String::from_ssz_bytes(&bytes[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode block_identifier: {}", e) })?;
                let block_id_size = block_identifier.as_ssz_bytes().len();
                offset += block_id_size;
                
                let full_transactions = bytes[offset] == 1;
                
                Ok(EthereumQueryInput::GetBlock { block_identifier, full_transactions })
            }
            other => Err(DecodeError { message: format!("Invalid EthereumQueryInput variant: {}", other) }),
        }
    }
}

impl SimpleSerialize for EthereumQueryInput {}

// EthereumQueryOutput
impl Encode for EthereumQueryOutput {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend((self.data.len() as u64).as_ssz_bytes());
        bytes.extend(&self.data);
        
        bytes
    }
}

impl Decode for EthereumQueryOutput {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
        offset += 8;
        
        let data = bytes[offset..offset + data_len].to_vec();
        
        Ok(EthereumQueryOutput {
            data,
        })
    }
}

impl SimpleSerialize for EthereumQueryOutput {}
