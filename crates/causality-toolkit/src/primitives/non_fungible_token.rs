//! Non-Fungible Token Primitive
//!
//! A comprehensive NFT implementation supporting standard ERC-721-like operations

use super::*;

// TODO: Implement non-fungible token primitive

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonFungibleToken {
    pub id: EntityId,
} 