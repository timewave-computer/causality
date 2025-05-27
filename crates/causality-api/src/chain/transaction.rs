//! Chain-Agnostic Transaction Types
//!
//! This module defines transaction data structures that work across different blockchains
//! with bounded fields for ZK compatibility. These types provide a unified interface for
//! transaction handling regardless of the underlying blockchain implementation.
//!
//! ## Type Organization
//!
//! * **Transaction**: Core transaction data structure with recipient and payload
//! * **TransactionMetadata**: Chain-specific transaction configuration parameters

use super::types::ChainId;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};

//-----------------------------------------------------------------------------
// Chain-Agnostic Transaction
//-----------------------------------------------------------------------------

/// Chain-agnostic transaction data
///
/// This structure represents the core data needed for a blockchain transaction
/// in a way that works across different blockchain implementations.
#[derive(Debug, Clone)]
pub struct ChainTransaction {
    /// The recipient of the transaction (address in bytes)
    /// For account-based chains like Ethereum, this is the recipient address
    /// For UTXO-based chains, this can encode an output script
    pub recipient: Vec<u8>,

    /// The transaction data payload
    /// For token transfers, the first 16 bytes are the amount in little-endian
    /// Additional data can be chain-specific (like token address for ERC20)
    pub data: Vec<u8>,
}

/// Chain-agnostic transaction metadata
///
/// This structure contains configuration parameters for a transaction
/// that affect how it's processed but are not part of the core transaction data.
#[derive(Debug, Clone)]
pub struct ChainTransactionMetadata {
    /// The chain ID this transaction is intended for
    pub chain_id: ChainId,

    /// Maximum gas/fee the transaction is allowed to consume
    /// For UTXO chains, this can represent the fee rate
    pub gas_limit: u64,

    /// Gas price or fee rate the transaction is willing to pay
    /// This is chain-specific and may represent different concepts:
    /// - For Ethereum: gas price in wei
    /// - For Solana: compute unit price in lamports
    /// - For Bitcoin: fee rate in satoshis/vbyte
    pub fee_rate: u64,

    /// Nonce or sequence number for the transaction
    /// - For Ethereum: account nonce
    /// - For Solana: recent blockhash as bytes
    /// - For Bitcoin: can be ignored
    pub nonce: Option<Vec<u8>>,
}

/// Transaction submission result
#[derive(Debug, Clone)]
pub struct ChainTransactionResult {
    /// Transaction ID or hash
    pub tx_id: Vec<u8>,

    /// Block number/height where the transaction was included
    /// None if the transaction is pending or rejected
    pub block_number: Option<u64>,

    /// Gas or compute units used by the transaction
    /// None if the transaction is pending or the chain doesn't have this concept
    pub gas_used: Option<u64>,

    /// Whether the transaction was successful
    pub success: bool,

    /// Chain-specific response data
    pub data: Option<Vec<u8>>,
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// ChainTransaction
impl Encode for ChainTransaction {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize recipient length and data
        bytes.extend((self.recipient.len() as u64).as_ssz_bytes());
        bytes.extend(&self.recipient);
        
        // Serialize data length and data
        bytes.extend((self.data.len() as u64).as_ssz_bytes());
        bytes.extend(&self.data);
        
        bytes
    }
}

impl Decode for ChainTransaction {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode recipient
        let recipient_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode recipient length: {}", e) })? as usize;
        offset += 8;
        
        let recipient = bytes[offset..offset + recipient_len].to_vec();
        offset += recipient_len;
        
        // Decode data
        let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
        offset += 8;
        
        let data = bytes[offset..offset + data_len].to_vec();
        
        Ok(ChainTransaction {
            recipient,
            data,
        })
    }
}

impl SimpleSerialize for ChainTransaction {}

// ChainTransactionMetadata
impl Encode for ChainTransactionMetadata {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.chain_id.as_ssz_bytes());
        bytes.extend(self.gas_limit.as_ssz_bytes());
        bytes.extend(self.fee_rate.as_ssz_bytes());
        
        // Serialize nonce as Option
        match &self.nonce {
            Some(nonce) => {
                bytes.push(1u8); // has value
                bytes.extend((nonce.len() as u64).as_ssz_bytes());
                bytes.extend(nonce);
            }
            None => {
                bytes.push(0u8); // no value
            }
        }
        
        bytes
    }
}

impl Decode for ChainTransactionMetadata {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let chain_id = ChainId::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode chain_id: {}", e) })?;
        let chain_id_size = chain_id.as_ssz_bytes().len();
        offset += chain_id_size;
        
        let gas_limit = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode gas_limit: {}", e) })?;
        offset += 8;
        
        let fee_rate = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode fee_rate: {}", e) })?;
        offset += 8;
        
        // Decode nonce option
        let has_nonce = bytes[offset];
        offset += 1;
        
        let nonce = if has_nonce == 1 {
            let nonce_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode nonce length: {}", e) })? as usize;
            offset += 8;
            Some(bytes[offset..offset + nonce_len].to_vec())
        } else {
            None
        };
        
        Ok(ChainTransactionMetadata {
            chain_id,
            gas_limit,
            fee_rate,
            nonce,
        })
    }
}

impl SimpleSerialize for ChainTransactionMetadata {}

// ChainTransactionResult
impl Encode for ChainTransactionResult {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize tx_id
        bytes.extend((self.tx_id.len() as u64).as_ssz_bytes());
        bytes.extend(&self.tx_id);
        
        // Serialize block_number as Option
        match self.block_number {
            Some(block) => {
                bytes.push(1u8);
                bytes.extend(block.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize gas_used as Option
        match self.gas_used {
            Some(gas) => {
                bytes.push(1u8);
                bytes.extend(gas.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        // Serialize success
        bytes.push(if self.success { 1u8 } else { 0u8 });
        
        // Serialize data as Option
        match &self.data {
            Some(data) => {
                bytes.push(1u8);
                bytes.extend((data.len() as u64).as_ssz_bytes());
                bytes.extend(data);
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for ChainTransactionResult {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode tx_id
        let tx_id_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode tx_id length: {}", e) })? as usize;
        offset += 8;
        
        let tx_id = bytes[offset..offset + tx_id_len].to_vec();
        offset += tx_id_len;
        
        // Decode block_number option
        let has_block_number = bytes[offset];
        offset += 1;
        
        let block_number = if has_block_number == 1 {
            let block = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode block_number: {}", e) })?;
            offset += 8;
            Some(block)
        } else {
            None
        };
        
        // Decode gas_used option
        let has_gas_used = bytes[offset];
        offset += 1;
        
        let gas_used = if has_gas_used == 1 {
            let gas = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode gas_used: {}", e) })?;
            offset += 8;
            Some(gas)
        } else {
            None
        };
        
        // Decode success
        let success = bytes[offset] == 1;
        offset += 1;
        
        // Decode data option
        let has_data = bytes[offset];
        offset += 1;
        
        let data = if has_data == 1 {
            let data_len = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode data length: {}", e) })? as usize;
            offset += 8;
            Some(bytes[offset..offset + data_len].to_vec())
        } else {
            None
        };
        
        Ok(ChainTransactionResult {
            tx_id,
            block_number,
            gas_used,
            success,
            data,
        })
    }
}

impl SimpleSerialize for ChainTransactionResult {}
