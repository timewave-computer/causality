// CosmWasm Adapter Factory
//
// This module provides a factory for creating CosmWasm domain adapters.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::DomainId;
use crate::domain::adapter::{DomainAdapter, DomainAdapterFactory};
use super::adapter::{CosmWasmAdapter, CosmWasmAdapterConfig};

/// Configuration for the CosmWasm adapter factory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmAdapterFactoryConfig {
    /// Default RPC URL template
    pub default_rpc_url_template: String,
    /// Default explorer URL template
    pub default_explorer_url_template: Option<String>,
    /// Default gas price
    pub default_gas_price: f64,
    /// Default fee denom
    pub default_fee_denom: String,
}

impl Default for CosmWasmAdapterFactoryConfig {
    fn default() -> Self {
        Self {
            default_rpc_url_template: "https://{chain_id}.cosmos.network".to_string(),
            default_explorer_url_template: Some("https://explorer.cosmos.network/{chain_id}".to_string()),
            default_gas_price: 0.025,
            default_fee_denom: "uatom".to_string(),
        }
    }
}

/// Factory for creating CosmWasm adapters
#[derive(Debug, Clone)]
pub struct CosmWasmAdapterFactory {
    /// Factory configuration
    config: CosmWasmAdapterFactoryConfig,
}

impl CosmWasmAdapterFactory {
    /// Create a new CosmWasm adapter factory
    pub fn new(config: CosmWasmAdapterFactoryConfig) -> Self {
        Self { config }
    }
    
    /// Create a new CosmWasm adapter factory with default configuration
    pub fn default() -> Self {
        Self::new(CosmWasmAdapterFactoryConfig::default())
    }
    
    /// Format a URL template with chain ID
    fn format_url_template(&self, template: &str, chain_id: &str) -> String {
        template.replace("{chain_id}", chain_id)
    }
}

#[async_trait]
impl DomainAdapterFactory for CosmWasmAdapterFactory {
    async fn create_adapter(&self, config: HashMap<String, String>) -> Result<Box<dyn DomainAdapter>> {
        // Extract required parameters
        let domain_id = config.get("domain_id")
            .ok_or_else(|| Error::InvalidArgument("Missing domain_id parameter".into()))?;
        
        let chain_id = config.get("chain_id")
            .ok_or_else(|| Error::InvalidArgument("Missing chain_id parameter".into()))?;
        
        let name = config.get("name")
            .cloned()
            .unwrap_or_else(|| chain_id.clone());
        
        // Determine RPC URL
        let rpc_url = config.get("rpc_url")
            .cloned()
            .unwrap_or_else(|| self.format_url_template(&self.config.default_rpc_url_template, chain_id));
        
        // Determine explorer URL
        let explorer_url = config.get("explorer_url").cloned().or_else(|| {
            self.config.default_explorer_url_template.as_ref().map(|template| {
                self.format_url_template(template, chain_id)
            })
        });
        
        // Extract other parameters or use defaults
        let description = config.get("description").cloned();
        let native_denom = config.get("native_denom").cloned().unwrap_or_else(|| "uatom".to_string());
        let gas_price = config.get("gas_price")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(self.config.default_gas_price);
        let fee_denom = config.get("fee_denom").cloned().unwrap_or_else(|| self.config.default_fee_denom.clone());
        
        // Create adapter configuration
        let adapter_config = CosmWasmAdapterConfig {
            domain_id: DomainId::new(domain_id),
            name,
            description,
            rpc_url,
            chain_id: chain_id.clone(),
            explorer_url,
            native_denom,
            gas_price,
            fee_denom,
        };
        
        // Create and return the adapter
        let adapter = CosmWasmAdapter::new(adapter_config)?;
        Ok(Box::new(adapter))
    }
    
    fn supported_domain_types(&self) -> Vec<String> {
        vec!["cosmwasm".to_string(), "cosmos".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_adapter() {
        let factory = CosmWasmAdapterFactory::default();
        
        let mut config = HashMap::new();
        config.insert("domain_id".to_string(), "osmosis_1".to_string());
        config.insert("chain_id".to_string(), "osmosis-1".to_string());
        config.insert("name".to_string(), "Osmosis".to_string());
        config.insert("native_denom".to_string(), "uosmo".to_string());
        
        let adapter = factory.create_adapter(config).await.unwrap();
        
        assert_eq!(adapter.domain_id().to_string(), "osmosis_1");
        
        // Test with minimal config
        let mut minimal_config = HashMap::new();
        minimal_config.insert("domain_id".to_string(), "juno_1".to_string());
        minimal_config.insert("chain_id".to_string(), "juno-1".to_string());
        
        let minimal_adapter = factory.create_adapter(minimal_config).await.unwrap();
        
        assert_eq!(minimal_adapter.domain_id().to_string(), "juno_1");
    }
    
    #[test]
    fn test_format_url_template() {
        let factory = CosmWasmAdapterFactory::default();
        
        let formatted = factory.format_url_template("https://{chain_id}.example.com", "test-1");
        assert_eq!(formatted, "https://test-1.example.com");
    }
} 