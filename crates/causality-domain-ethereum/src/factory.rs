// EVM adapter factory
// Original file: src/domain_adapters/evm/factory.rs

// EVM Adapter Factory
//
// This module provides a factory for creating EVM domain adapters.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use causality_types::DomainId;
use causality_domain::{DomainAdapter, DomainAdapterFactory};
use super::adapter::{EthereumAdapter, EthereumConfig};

/// Configuration for the EVM adapter factory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumAdapterFactoryConfig {
    /// Default RPC URL template
    pub default_rpc_url_template: String,
    /// Default explorer URL template
    pub default_explorer_url_template: Option<String>,
    /// Default gas price (in gwei)
    pub default_gas_price: f64,
    /// Default chain ID
    pub default_chain_id: u64,
}

impl Default for EthereumAdapterFactoryConfig {
    fn default() -> Self {
        Self {
            default_rpc_url_template: "https://{chain_id}.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            default_explorer_url_template: Some("https://etherscan.io".to_string()),
            default_gas_price: 20.0, // Default gas price in gwei
            default_chain_id: 1, // Ethereum mainnet
        }
    }
}

/// Factory for creating EVM adapters
#[derive(Debug, Clone)]
pub struct EthereumAdapterFactory {
    /// Factory configuration
    config: EthereumAdapterFactoryConfig,
}

impl EthereumAdapterFactory {
    /// Create a new EVM adapter factory
    pub fn new(config: EthereumAdapterFactoryConfig) -> Self {
        Self { config }
    }
    
    /// Create a new EVM adapter factory with default configuration
    pub fn default() -> Self {
        Self::new(EthereumAdapterFactoryConfig::default())
    }
    
    /// Format a URL template with chain ID
    fn format_url_template(&self, template: &str, chain_id: &str) -> String {
        template.replace("{chain_id}", chain_id)
    }
}

#[async_trait]
impl DomainAdapterFactory for EthereumAdapterFactory {
    async fn create_adapter(&self, config: HashMap<String, String>) -> Result<Box<dyn DomainAdapter>> {
        // Extract required parameters
        let domain_id = config.get("domain_id")
            .ok_or_else(|| Error::InvalidArgument("Missing domain_id parameter".into()))?;
        
        // Extract optional parameters or use defaults
        let name = config.get("name")
            .cloned()
            .unwrap_or_else(|| "Ethereum".to_string());
        
        let description = config.get("description").cloned();
        
        // Determine chain ID
        let chain_id = config.get("chain_id")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(self.config.default_chain_id);
        
        // Determine RPC URL
        let rpc_url = config.get("rpc_url")
            .cloned()
            .unwrap_or_else(|| self.format_url_template(
                &self.config.default_rpc_url_template, 
                &chain_id.to_string()
            ));
        
        // Determine explorer URL
        let explorer_url = config.get("explorer_url").cloned().or_else(|| {
            self.config.default_explorer_url_template.clone()
        });
        
        // Native currency symbol
        let native_currency = config.get("native_currency")
            .cloned()
            .unwrap_or_else(|| "ETH".to_string());
        
        // Create adapter configuration
        let adapter_config = EthereumConfig {
            domain_id: DomainId::new(domain_id),
            name,
            description,
            rpc_url,
            chain_id,
            explorer_url,
            native_currency,
        };
        
        // Create and return the adapter
        let adapter = EthereumAdapter::new(adapter_config)?;
        Ok(Box::new(adapter))
    }
    
    fn supported_domain_types(&self) -> Vec<String> {
        vec!["evm".to_string(), "ethereum".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_adapter() {
        let factory = EthereumAdapterFactory::default();
        
        let mut config = HashMap::new();
        config.insert("domain_id".to_string(), "ethereum_1".to_string());
        config.insert("chain_id".to_string(), "1".to_string());
        config.insert("name".to_string(), "Ethereum Mainnet".to_string());
        
        let adapter = factory.create_adapter(config).await.unwrap();
        
        assert_eq!(adapter.domain_id().to_string(), "ethereum_1");
        
        // Test with minimal config
        let mut minimal_config = HashMap::new();
        minimal_config.insert("domain_id".to_string(), "optimism_1".to_string());
        
        let minimal_adapter = factory.create_adapter(minimal_config).await.unwrap();
        
        assert_eq!(minimal_adapter.domain_id().to_string(), "optimism_1");
    }
    
    #[test]
    fn test_format_url_template() {
        let factory = EthereumAdapterFactory::default();
        
        let formatted = factory.format_url_template("https://{chain_id}.example.com", "10");
        assert_eq!(formatted, "https://10.example.com");
    }
} 