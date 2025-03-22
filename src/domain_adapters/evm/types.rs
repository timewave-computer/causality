// EVM-specific types
//
// This file contains type definitions specific to the EVM domain.

use serde::{Serialize, Deserialize};
use std::str::FromStr;

// Re-export ethers types that are used throughout the codebase
pub use ethers::core::types::{
    BlockId, BlockNumber, Filter, Log, Block, H160, H256,
    Address, U256, 
    Transaction as EthTransaction,
    TransactionReceipt as EthTxReceipt
};
pub use ethers::providers::{Provider, Http};
pub use ethers::types::U64;

// Import our common types
use crate::types::{BlockHeight, BlockHash};

// Ethereum-specific conversions
// Convert BlockHeight to Ethereum U64
impl From<BlockHeight> for U64 {
    fn from(height: BlockHeight) -> Self {
        U64::from(height.0)
    }
}

// Convert BlockHeight to Ethereum BlockId
impl From<BlockHeight> for BlockId {
    fn from(height: BlockHeight) -> Self {
        BlockId::Number(BlockNumber::Number(height.into()))
    }
}

// Convert BlockHash to Ethereum H256
impl From<&BlockHash> for H256 {
    fn from(hash: &BlockHash) -> Self {
        H256::from_slice(&hash.0)
    }
}

// Convert Ethereum H256 to BlockHash
impl From<H256> for BlockHash {
    fn from(hash: H256) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(hash.as_bytes());
        BlockHash(bytes)
    }
}

// Helper implementation for BlockHeight
impl BlockHeight {
    /// Helper method to convert to Ethereum BlockNumber
    pub fn to_block_number(&self) -> BlockNumber {
        BlockNumber::Number(U64::from(self.0))
    }
}

// Helper implementation for BlockHash
impl BlockHash {
    pub fn from_h256(hash: &[u8]) -> Self {
        BlockHash::new(hash.to_vec())
    }
} 