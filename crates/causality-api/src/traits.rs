// Trait definitions for chain configurations and domain interactions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration trait for blockchain interactions
#[async_trait]
pub trait ChainConfig {
    const CHAIN_NAME: &'static str;
    const CHAIN_ID: &'static str;
    const DEFAULT_RPC_PORT: &'static str;
    const CHAIN_TYPE: &'static str;
    
    /// Get the chain identifier
    fn chain_id(&self) -> String {
        Self::CHAIN_ID.to_string()
    }
    
    /// Get the RPC endpoint URL
    fn rpc_url(&self) -> String {
        format!("http://localhost:{}", Self::DEFAULT_RPC_PORT)
    }
    
    /// Get the chain-specific configuration
    fn config(&self) -> ChainSpecificConfig {
        ChainSpecificConfig::default()
    }
}

/// Chain-specific configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSpecificConfig {
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub block_time: Option<u64>,
    pub confirmation_blocks: Option<u64>,
}

impl Default for ChainSpecificConfig {
    fn default() -> Self {
        Self {
            gas_limit: Some(1_000_000),
            gas_price: Some(1_000_000_000), // 1 gwei
            block_time: Some(6), // 6 seconds
            confirmation_blocks: Some(12),
        }
    }
}

/// Ethereum chain configuration
#[derive(Debug, Clone)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub config: ChainSpecificConfig,
}

#[async_trait]
impl ChainConfig for EthereumConfig {
    const CHAIN_NAME: &'static str = "ethereum";
    const CHAIN_ID: &'static str = "1";
    const DEFAULT_RPC_PORT: &'static str = "8545";
    const CHAIN_TYPE: &'static str = "evm";
    
    fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }
    
    fn config(&self) -> ChainSpecificConfig {
        self.config.clone()
    }
}

/// Cosmos chain configuration
#[derive(Debug, Clone)]
pub struct CosmosConfig {
    pub rpc_url: String,
    pub config: ChainSpecificConfig,
}

#[async_trait]
impl ChainConfig for CosmosConfig {
    const CHAIN_NAME: &'static str = "cosmos";
    const CHAIN_ID: &'static str = "cosmoshub-4";
    const DEFAULT_RPC_PORT: &'static str = "26657";
    const CHAIN_TYPE: &'static str = "cosmos";
    
    fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }
    
    fn config(&self) -> ChainSpecificConfig {
        self.config.clone()
    }
} 