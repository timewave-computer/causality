/// Rust templates for generating effect adapter code.
///
/// This module provides constants for different templates used in generating
/// Rust code for effect adapters, including adapter implementations, effect methods,
/// fact methods, proof methods, RPC clients, types, utilities, tests, documentation,
/// and examples.

/// Template for the main adapter implementation.
pub const ADAPTER_TEMPLATE: &str = r#"//! {{DOMAIN_PASCALCASE}} Effect Adapter
//!
//! This module provides an effect adapter for the {{DOMAIN_ID}} domain.
//! Auto-generated from adapter schema.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::error::{Error, Result};
use crate::types::{ContentId, DomainId};
use crate::effect_adapters::{
    EffectAdapter, DomainConfig, EffectParams, TransactionReceipt,
    FactType, FactObservationMeta, AdapterError, ProofError, ObservationError,
};

use super::types::*;
use super::utils::*;

/// {{DOMAIN_PASCALCASE}} adapter for the {{DOMAIN_ID}} domain
pub struct {{ADAPTER_NAME}} {
    /// Domain ID
    domain_id: DomainId,
    
    /// Domain configuration
    config: DomainConfig,
    
    /// HTTP client
    client: Client,
    
    /// Next request ID
    next_request_id: AtomicU64,
    
    /// RPC clients
    {{RPC_CLIENTS}}
}

impl {{ADAPTER_NAME}} {
    /// Create a new {{DOMAIN_PASCALCASE}} adapter
    pub fn new(config: DomainConfig) -> Result<Self> {
        // Validate that this is a {{DOMAIN_ID}} domain
        if config.domain_id.as_ref() != "{{DOMAIN_ID}}" {
            return Err(Error::ValidationError(format!(
                "Expected domain ID '{{DOMAIN_ID}}', got '{}'",
                config.domain_id
            )));
        }
        
        // Create HTTP client
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(
                config.timeout_ms.unwrap_or(30000)
            ))
            .build()
            .map_err(|e| Error::InitializationError(format!(
                "Failed to create HTTP client: {}", e
            )))?;
        
        // Create the adapter
        let adapter = {{ADAPTER_NAME}} {
            domain_id: config.domain_id.clone(),
            config: config.clone(),
            client: client.clone(),
            next_request_id: AtomicU64::new(1),
            // Initialize RPC clients
            // ...
        };
        
        Ok(adapter)
    }
    
    // Effect handling methods
    {{EFFECT_METHODS}}
    
    // Fact observation methods
    {{FACT_METHODS}}
    
    // Proof validation methods
    {{PROOF_METHODS}}
    
    /// Get the next request ID
    fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[async_trait]
impl EffectAdapter for {{ADAPTER_NAME}} {
    async fn apply_effect(&self, params: EffectParams) -> std::result::Result<TransactionReceipt, AdapterError> {
        match params.effect_type.as_str() {
            {{#each effects}}
            "{{effect_type}}" => self.handle_{{effect_name}}(params).await,
            {{/each}}
            _ => Err(AdapterError::UnsupportedOperation(format!(
                "Unsupported effect type: {}", params.effect_type
            ))),
        }
    }
    
    async fn validate_proof(&self, proof: &[u8], fact_type: &str) -> std::result::Result<bool, ProofError> {
        match fact_type {
            {{#each proofs}}
            "{{proof_type}}" => self.validate_{{proof_name}}_proof(proof).await,
            {{/each}}
            _ => Err(ProofError::InvalidFormat(format!(
                "Unsupported proof type: {}", fact_type
            ))),
        }
    }
    
    async fn observe_fact(&self, fact_type: &str, params: &HashMap<String, String>) -> std::result::Result<(FactType, FactObservationMeta), ObservationError> {
        match fact_type {
            {{#each facts}}
            "{{fact_type}}" => self.observe_{{fact_name}}(params).await,
            {{/each}}
            _ => Err(ObservationError::InvalidFormat(format!(
                "Unsupported fact type: {}", fact_type
            ))),
        }
    }
    
    fn get_domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn get_config(&self) -> &DomainConfig {
        &self.config
    }
    
    fn update_config(&mut self, config: DomainConfig) -> Result<()> {
        if config.domain_id != self.domain_id {
            return Err(Error::ValidationError(format!(
                "Cannot update config: domain ID mismatch (expected {}, got {})",
                self.domain_id, config.domain_id
            )));
        }
        
        self.config = config;
        Ok(())
    }
}
"#;

/// Template for generating an effect method.
pub const EFFECT_METHOD_TEMPLATE: &str = r#"/// Handle {{EFFECT_TYPE}} effect
async fn handle_{{EFFECT_NAME}}(&self, params: EffectParams) -> std::result::Result<TransactionReceipt, AdapterError> {
    // Validate parameters
{{PARAM_VALIDATION}}
    
    // Prepare transaction
    let request_id = self.next_request_id();
    let method = "{{RPC_CALL}}";
    
    // TODO: Implement {{EFFECT_TYPE}} effect handling
    
    // For now, return a mock receipt
    Ok(TransactionReceipt {
        domain_id: self.domain_id.clone(),
        transaction_id: format!("tx-{}", request_id),
        content_id: None,
        timestamp: None,
        status: true,
        metadata: HashMap::new(),
    })
}"#;

/// Template for generating a fact observation method.
pub const FACT_METHOD_TEMPLATE: &str = r#"/// Observe {{FACT_TYPE}} fact
async fn observe_{{FACT_NAME}}(&self, params: &HashMap<String, String>) -> std::result::Result<(FactType, FactObservationMeta), ObservationError> {
    // Extract parameters
{{PARAM_HANDLING}}
    
    // Prepare request
    let request_id = self.next_request_id();
    let method = "{{RPC_CALL}}";
    
    // TODO: Implement {{FACT_TYPE}} fact observation
    
    // For now, return a mock fact
    Ok((FactType::Custom("{{FACT_NAME}}".to_string()), FactObservationMeta {
        domain_id: self.domain_id.clone(),
        fact_type: "{{FACT_TYPE}}".to_string(),
        content_id: None,
        timestamp: None,
        data: "mock_data".to_string(),
        metadata: HashMap::new(),
    })
}"#;

/// Template for generating a proof validation method.
pub const PROOF_METHOD_TEMPLATE: &str = r#"/// Validate {{PROOF_TYPE}} proof
async fn validate_{{PROOF_NAME}}_proof(&self, proof: &[u8]) -> std::result::Result<bool, ProofError> {
    // TODO: Implement {{PROOF_TYPE}} proof validation using {{VERIFICATION_METHOD}}
    
    // For now, return true
    Ok(true)
}"#;

/// Template for generating an RPC client.
pub const RPC_CLIENT_TEMPLATE: &str = r#"/// {{RPC_NAME}} client
struct {{RPC_STRUCT}}Client {
    /// HTTP client
    client: Client,
    
    /// Endpoint URL
    endpoint: String,
    
    /// Request timeout (ms)
    timeout_ms: u64,
    
    /// Authentication headers
    auth_headers: HashMap<String, String>,
}

impl {{RPC_STRUCT}}Client {
    /// Create a new client
    pub fn new(
        client: Client, 
        endpoint: String, 
        timeout_ms: u64, 
        auth_headers: HashMap<String, String>
    ) -> Self {
        {{RPC_STRUCT}}Client {
            client,
            endpoint,
            timeout_ms,
            auth_headers,
        }
    }
    
    /// Call an RPC method
    pub async fn call(
        &self, 
        method: &str, 
        http_method: &str, 
        params: serde_json::Value
    ) -> Result<serde_json::Value, reqwest::Error> {
        // Prepare request
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        // Send request
        let response = match http_method {
            "POST" => {
                let mut req = self.client.post(&self.endpoint)
                    .timeout(std::time::Duration::from_millis(self.timeout_ms))
                    .json(&request_body);
                
                // Add auth headers
                for (key, value) in &self.auth_headers {
                    req = req.header(key, value);
                }
                
                req.send().await?
            },
            "GET" => {
                let mut req = self.client.get(&self.endpoint)
                    .timeout(std::time::Duration::from_millis(self.timeout_ms));
                
                // Add auth headers
                for (key, value) in &self.auth_headers {
                    req = req.header(key, value);
                }
                
                req.send().await?
            },
            _ => {
                panic!("Unsupported HTTP method: {}", http_method);
            }
        };
        
        // Parse response
        let response_body: serde_json::Value = response.json().await?;
        
        Ok(response_body)
    }
    
    // Method implementations for each RPC method
{{METHOD_IMPLS}}
}"#;

/// Template for generating type definitions.
pub const TYPES_TEMPLATE: &str = r#"//! {{DOMAIN_PASCALCASE}} types
//!
//! This module defines the types used by the {{DOMAIN_PASCALCASE}} adapter.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// {{DOMAIN_PASCALCASE}} transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{DOMAIN_PASCALCASE}}Transaction {
    /// Transaction hash
    pub hash: String,
    
    /// From address
    pub from: String,
    
    /// To address (if any)
    pub to: Option<String>,
    
    /// Transaction value
    pub value: String,
    
    /// Gas price
    pub gas_price: String,
    
    /// Gas limit
    pub gas_limit: String,
    
    /// Input data
    pub input: String,
    
    /// Nonce
    pub nonce: u64,
    
    /// Chain ID
    pub chain_id: String,
    
    /// Additional data
    pub additional_data: HashMap<String, String>,
}

/// {{DOMAIN_PASCALCASE}} transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{DOMAIN_PASCALCASE}}TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Transaction index
    pub transaction_index: String,
    
    /// Block hash
    pub block_hash: String,
    
    /// Block number
    pub block_number: String,
    
    /// From address
    pub from: String,
    
    /// To address (if any)
    pub to: Option<String>,
    
    /// Cumulative gas used
    pub cumulative_gas_used: String,
    
    /// Gas used
    pub gas_used: String,
    
    /// Contract address (if created)
    pub contract_address: Option<String>,
    
    /// Status (1 for success, 0 for failure)
    pub status: String,
    
    /// Logs
    pub logs: Vec<{{DOMAIN_PASCALCASE}}Log>,
    
    /// Additional data
    pub additional_data: HashMap<String, String>,
}

/// {{DOMAIN_PASCALCASE}} log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{DOMAIN_PASCALCASE}}Log {
    /// Log index
    pub log_index: String,
    
    /// Transaction index
    pub transaction_index: String,
    
    /// Transaction hash
    pub transaction_hash: String,
    
    /// Block hash
    pub block_hash: String,
    
    /// Block number
    pub block_number: String,
    
    /// Address
    pub address: String,
    
    /// Data
    pub data: String,
    
    /// Topics
    pub topics: Vec<String>,
    
    /// Removed flag
    pub removed: bool,
}

/// {{DOMAIN_PASCALCASE}} block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{DOMAIN_PASCALCASE}}Block {
    /// Block number
    pub number: String,
    
    /// Block hash
    pub hash: String,
    
    /// Parent hash
    pub parent_hash: String,
    
    /// Timestamp
    pub timestamp: String,
    
    /// Transactions root
    pub transactions_root: String,
    
    /// State root
    pub state_root: String,
    
    /// Receipts root
    pub receipts_root: String,
    
    /// Transactions
    pub transactions: Vec<String>,
    
    /// Additional data
    pub additional_data: HashMap<String, String>,
}

/// {{DOMAIN_PASCALCASE}} proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{DOMAIN_PASCALCASE}}Proof {
    /// Block number
    pub block_number: String,
    
    /// Block hash
    pub block_hash: String,
    
    /// Proof data
    pub proof_data: Vec<u8>,
    
    /// Verified flag
    pub verified: bool,
    
    /// Additional data
    pub additional_data: HashMap<String, String>,
}
"#;

/// Template for generating utility functions.
pub const UTILS_TEMPLATE: &str = r#"//! {{DOMAIN_PASCALCASE}} utilities
//!
//! This module provides utility functions for the {{DOMAIN_PASCALCASE}} adapter.

use std::collections::HashMap;
use serde_json::Value;
use hex;

/// Convert a hex string to a decimal string
pub fn hex_to_decimal(hex_str: &str) -> Result<String, String> {
    // Remove '0x' prefix if present
    let clean_hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    
    // Parse hex string
    match u128::from_str_radix(clean_hex, 16) {
        Ok(value) => Ok(value.to_string()),
        Err(err) => Err(format!("Failed to parse hex string: {}", err)),
    }
}

/// Convert a decimal string to a hex string
pub fn decimal_to_hex(decimal_str: &str) -> Result<String, String> {
    // Parse decimal string
    match decimal_str.parse::<u128>() {
        Ok(value) => Ok(format!("0x{:x}", value)),
        Err(err) => Err(format!("Failed to parse decimal string: {}", err)),
    }
}

/// Parse a JSON-RPC response
pub fn parse_jsonrpc_response(response: Value) -> Result<Value, String> {
    // Check for error
    if let Some(error) = response.get("error") {
        return Err(format!("JSON-RPC error: {}", error));
    }
    
    // Extract result
    if let Some(result) = response.get("result") {
        Ok(result.clone())
    } else {
        Err("JSON-RPC response missing 'result' field".to_string())
    }
}

/// Build authentication headers from config
pub fn build_auth_headers(config: &HashMap<String, String>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    if let Some(api_key) = config.get("api_key") {
        headers.insert("X-API-Key".to_string(), api_key.clone());
    }
    
    if let Some(auth_token) = config.get("auth_token") {
        headers.insert("Authorization".to_string(), format!("Bearer {}", auth_token));
    }
    
    headers
}

/// Get a template endpoint URL with variables filled in
pub fn get_endpoint_url(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}
"#;

/// Template for generating adapter unit tests.
pub const ADAPTER_TEST_TEMPLATE: &str = r#"//! Tests for the {{ADAPTER_NAME}}
//!
//! This module contains unit tests for the {{ADAPTER_NAME}}.

use std::collections::HashMap;
use crate::types::DomainId;
use crate::effect_adapters::{DomainConfig, EffectParams};
use crate::effect_adapters::{{DOMAIN_ID}}::{{ADAPTER_NAME}};

/// Create a test config for the {{ADAPTER_NAME}}
fn create_test_config() -> DomainConfig {
    DomainConfig {
        domain_id: DomainId::new("{{DOMAIN_ID}}"),
        rpc_endpoints: vec!["http://localhost:8545".to_string()],
        chain_id: Some("1".to_string()),
        network_id: Some("1".to_string()),
        timeout_ms: Some(5000),
        gas_limit: Some(100000),
        auth: HashMap::new(),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_adapter_creation() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    assert_eq!(adapter.get_domain_id().as_ref(), "{{DOMAIN_ID}}");
}

#[tokio::test]
async fn test_config_validation() {
    let mut config = create_test_config();
    config.domain_id = DomainId::new("invalid");
    
    let result = {{ADAPTER_NAME}}::new(config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_config() {
    let config = create_test_config();
    let mut adapter = {{ADAPTER_NAME}}::new(config.clone()).unwrap();
    
    let mut new_config = config.clone();
    new_config.timeout_ms = Some(10000);
    
    let result = adapter.update_config(new_config);
    assert!(result.is_ok());
    assert_eq!(adapter.get_config().timeout_ms, Some(10000));
}
"#;

/// Template for generating effect method unit tests.
pub const EFFECT_TEST_TEMPLATE: &str = r#"//! Tests for {{ADAPTER_NAME}} effects
//!
//! This module contains unit tests for the {{ADAPTER_NAME}} effects.

use std::collections::HashMap;
use crate::types::DomainId;
use crate::effect_adapters::{DomainConfig, EffectParams, EffectAdapter};
use crate::effect_adapters::{{DOMAIN_ID}}::{{ADAPTER_NAME}};

/// Create a test config for the {{ADAPTER_NAME}}
fn create_test_config() -> DomainConfig {
    DomainConfig {
        domain_id: DomainId::new("{{DOMAIN_ID}}"),
        rpc_endpoints: vec!["http://localhost:8545".to_string()],
        chain_id: Some("1".to_string()),
        network_id: Some("1".to_string()),
        timeout_ms: Some(5000),
        gas_limit: Some(100000),
        auth: HashMap::new(),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_transfer_effect() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    let mut params = HashMap::new();
    params.insert("from".to_string(), "0x1234".as_bytes().to_vec());
    params.insert("to".to_string(), "0x5678".as_bytes().to_vec());
    params.insert("value".to_string(), "1000".as_bytes().to_vec());
    
    let effect_params = EffectParams {
        effect_type: "transfer".to_string(),
        params,
        source: Some("0x1234".to_string()),
        destination: Some("0x5678".to_string()),
        asset: Some("ETH".to_string()),
        amount: Some("1000".to_string()),
        data: None,
        signature: None,
        gas_limit: Some(21000),
        gas_price: Some(5000000000),
        nonce: Some(1),
        metadata: HashMap::new(),
    };
    
    let result = adapter.apply_effect(effect_params).await;
    assert!(result.is_ok());
    
    let receipt = result.unwrap();
    assert_eq!(receipt.domain_id, DomainId::new("{{DOMAIN_ID}}"));
    assert!(receipt.status);
}

#[tokio::test]
async fn test_unsupported_effect() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    let effect_params = EffectParams {
        effect_type: "unsupported".to_string(),
        params: HashMap::new(),
        source: None,
        destination: None,
        asset: None,
        amount: None,
        data: None,
        signature: None,
        gas_limit: None,
        gas_price: None,
        nonce: None,
        metadata: HashMap::new(),
    };
    
    let result = adapter.apply_effect(effect_params).await;
    assert!(result.is_err());
}
"#;

/// Template for generating fact method unit tests.
pub const FACT_TEST_TEMPLATE: &str = r#"//! Tests for {{ADAPTER_NAME}} facts
//!
//! This module contains unit tests for the {{ADAPTER_NAME}} facts.

use std::collections::HashMap;
use crate::types::DomainId;
use crate::effect_adapters::{DomainConfig, EffectAdapter};
use crate::effect_adapters::{{DOMAIN_ID}}::{{ADAPTER_NAME}};

/// Create a test config for the {{ADAPTER_NAME}}
fn create_test_config() -> DomainConfig {
    DomainConfig {
        domain_id: DomainId::new("{{DOMAIN_ID}}"),
        rpc_endpoints: vec!["http://localhost:8545".to_string()],
        chain_id: Some("1".to_string()),
        network_id: Some("1".to_string()),
        timeout_ms: Some(5000),
        gas_limit: Some(100000),
        auth: HashMap::new(),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_balance_fact() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    let mut params = HashMap::new();
    params.insert("address".to_string(), "0x1234".to_string());
    params.insert("blockNumber".to_string(), "latest".to_string());
    
    let result = adapter.observe_fact("balance", &params).await;
    assert!(result.is_ok());
    
    let fact = result.unwrap();
    assert_eq!(fact.domain_id, DomainId::new("{{DOMAIN_ID}}"));
    assert_eq!(fact.fact_type, "balance");
}

#[tokio::test]
async fn test_unsupported_fact() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    let params = HashMap::new();
    
    let result = adapter.observe_fact("unsupported", &params).await;
    assert!(result.is_err());
}
"#;

/// Template for generating proof method unit tests.
pub const PROOF_TEST_TEMPLATE: &str = r#"//! Tests for {{ADAPTER_NAME}} proofs
//!
//! This module contains unit tests for the {{ADAPTER_NAME}} proofs.

use std::collections::HashMap;
use crate::types::DomainId;
use crate::effect_adapters::{DomainConfig, EffectAdapter};
use crate::effect_adapters::{{DOMAIN_ID}}::{{ADAPTER_NAME}};

/// Create a test config for the {{ADAPTER_NAME}}
fn create_test_config() -> DomainConfig {
    DomainConfig {
        domain_id: DomainId::new("{{DOMAIN_ID}}"),
        rpc_endpoints: vec!["http://localhost:8545".to_string()],
        chain_id: Some("1".to_string()),
        network_id: Some("1".to_string()),
        timeout_ms: Some(5000),
        gas_limit: Some(100000),
        auth: HashMap::new(),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_transaction_proof() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    let proof = b"mock transaction proof data";
    
    let result = adapter.validate_proof(proof, "transaction").await;
    assert!(result.is_ok());
    
    let valid = result.unwrap();
    assert!(valid);
}

#[tokio::test]
async fn test_unsupported_proof() {
    let config = create_test_config();
    let adapter = {{ADAPTER_NAME}}::new(config).unwrap();
    
    let proof = b"mock proof data";
    
    let result = adapter.validate_proof(proof, "unsupported").await;
    assert!(result.is_err());
}
"#;

/// Template for generating adapter documentation.
pub const README_TEMPLATE: &str = r#"# {{ADAPTER_NAME}}

This is a generated effect adapter for the {{DOMAIN_ID}} domain ({{DOMAIN_TYPE}}).

## Features

- Apply effects to the {{DOMAIN_ID}} domain
- Observe facts from the {{DOMAIN_ID}} domain
- Validate proofs from the {{DOMAIN_ID}} domain

## Usage

```rust
use crate::effect_adapters::{DomainConfig, EffectAdapter};
use crate::effect_adapters::{{DOMAIN_ID}}::{{ADAPTER_NAME}};

// Create configuration
let config = DomainConfig {
    domain_id: DomainId::new("{{DOMAIN_ID}}"),
    rpc_endpoints: vec!["https://...".to_string()],
    // other config...
};

// Create adapter
let adapter = {{ADAPTER_NAME}}::new(config)?;

// Use adapter
let params = EffectParams {
    effect_type: "transfer".to_string(),
    // other params...
};

let receipt = adapter.apply_effect(params).await?;
```

See the examples directory for more usage examples.

## Configuration

The adapter requires the following configuration:

- `rpc_endpoints`: List of RPC endpoints to connect to
- `chain_id`: Chain ID (for blockchain domains)
- `network_id`: Network ID (for blockchain domains)
- `timeout_ms`: Request timeout in milliseconds
- `gas_limit`: Default gas limit for transactions (for blockchain domains)
- `auth`: Authentication credentials (see API.md for details)

## API Reference

See [API.md](docs/API.md) for the complete API reference.
"#;

/// Template for generating detailed API documentation.
pub const API_DOCS_TEMPLATE: &str = r#"# {{ADAPTER_NAME}} API Reference

This document describes the API for the {{ADAPTER_NAME}}.

## Configuration

```rust
DomainConfig {
    domain_id: DomainId::new("{{DOMAIN_ID}}"),
    rpc_endpoints: vec!["https://...".to_string()],
    chain_id: Some("1".to_string()),
    network_id: Some("1".to_string()),
    timeout_ms: Some(30000),
    gas_limit: Some(100000),
    auth: {
        // Authentication credentials
        "api_key": "YOUR_API_KEY",
        "project_id": "YOUR_PROJECT_ID",
    },
    metadata: {
        // Additional configuration
        "max_retries": "3",
        "retry_delay_ms": "1000",
    },
}
```

## Effects

The adapter supports the following effects:

{{#each effects}}
### {{effect_type}}

Apply a {{effect_type}} effect to the {{DOMAIN_ID}} domain.

**Parameters:**

{{#each required_fields}}
- `{{this}}` (required): {{description}}
{{/each}}

{{#each optional_fields}}
- `{{this}}` (optional): {{description}}
{{/each}}

**Example:**

```rust
let params = EffectParams {
    effect_type: "{{effect_type}}".to_string(),
    params: {
        // Required parameters
        {{#each required_fields}}
        "{{this}}": value,
        {{/each}}
        // Optional parameters
        {{#each optional_fields}}
        "{{this}}": value,
        {{/each}}
    },
    // Additional fields
    source: Some("0x...".to_string()),
    destination: Some("0x...".to_string()),
    asset: Some("ETH".to_string()),
    amount: Some("1.0".to_string()),
    data: None,
    signature: None,
    gas_limit: Some(21000),
    gas_price: Some(5000000000),
    nonce: Some(1),
    metadata: HashMap::new(),
};

let receipt = adapter.apply_effect(params).await?;
```

{{/each}}

## Facts

The adapter supports the following facts:

{{#each facts}}
### {{fact_type}}

Observe a {{fact_type}} fact from the {{DOMAIN_ID}} domain.

**Parameters:**

{{#each required_fields}}
- `{{this}}` (required): {{description}}
{{/each}}

**Example:**

```rust
let params = HashMap::new();
{{#each required_fields}}
params.insert("{{this}}".to_string(), "value".to_string());
{{/each}}

let fact = adapter.observe_fact("{{fact_type}}", &params).await?;
```

{{/each}}

## Proofs

The adapter supports the following proofs:

{{#each proofs}}
### {{proof_type}}

Validate a {{proof_type}} proof from the {{DOMAIN_ID}} domain.

**Example:**

```rust
let proof = get_proof_data();
let valid = adapter.validate_proof(proof, "{{proof_type}}").await?;
```

{{/each}}
"#;

/// Template for generating a basic example.
pub const BASIC_EXAMPLE_TEMPLATE: &str = r#"//! Basic usage example for the {{ADAPTER_NAME}}
//!
//! This example demonstrates how to create and use the {{ADAPTER_NAME}}.

use std::collections::HashMap;
use causality::types::DomainId;
use causality::effect_adapters::{DomainConfig, EffectAdapter, EffectParams};
use causality::effect_adapters::{{DOMAIN_ID}}::{{ADAPTER_NAME}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let mut config = DomainConfig {
        domain_id: DomainId::new("{{DOMAIN_ID}}"),
        rpc_endpoints: vec!["http://localhost:8545".to_string()],
        chain_id: Some("1".to_string()),
        network_id: Some("1".to_string()),
        timeout_ms: Some(30000),
        gas_limit: Some(100000),
        auth: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    // Add authentication if needed
    config.auth.insert("api_key".to_string(), "YOUR_API_KEY".to_string());
    
    // Create adapter
    let adapter = {{ADAPTER_NAME}}::new(config)?;
    println!("Created adapter for domain: {}", adapter.get_domain_id());
    
    // Use adapter (example functionality)
    let domain_id = adapter.get_domain_id();
    println!("Adapter domain ID: {}", domain_id);
    
    let config = adapter.get_config();
    println!("Adapter config: {:?}", config);
    
    Ok(())
}
"#;

/// Template for generating a complete example project.
pub const COMPLETE_EXAMPLE_TEMPLATE: &str = r#"# Complete {{ADAPTER_NAME}} Example Project

This directory contains a complete example project demonstrating how to use the {{ADAPTER_NAME}} adapter.

## Structure

- `src/main.rs`: Main example application
- `src/config.rs`: Configuration handling
- `src/examples/`: Individual examples for different adapter functionality
  - `transfer.rs`: Example for token transfers
  - `contract.rs`: Example for contract interactions
  - `observation.rs`: Example for fact observation
  - `verification.rs`: Example for proof verification

## Running the Examples

```bash
# Run basic example
cargo run --example basic

# Run transfer example
cargo run --example transfer

# Run contract example
cargo run --example contract

# Run observation example
cargo run --example observation

# Run verification example
cargo run --example verification
```

## Main Example

```rust
// src/main.rs
use causality::effect_adapters::{EffectAdapter, DomainConfig};
use causality::effect_adapters::{{MODULE_NAME}}::{{ADAPTER_NAME}};
use examples::{transfer, contract, observation, verification};

mod config;
mod examples;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_config()?;
    let adapter = {{ADAPTER_NAME}}::new("{{DOMAIN_ID_EXAMPLE}}".to_string(), config);
    
    println!("Running transfer example...");
    transfer::run(&adapter).await?;
    
    println!("Running contract example...");
    contract::run(&adapter).await?;
    
    println!("Running observation example...");
    observation::run(&adapter).await?;
    
    println!("Running verification example...");
    verification::run(&adapter).await?;
    
    println!("All examples completed successfully!");
    Ok(())
}
```

## Configuration Example

```rust
// src/config.rs
use causality::effect_adapters::DomainConfig;
use std::collections::HashMap;
use std::env;

pub fn load_config() -> Result<DomainConfig, Box<dyn std::error::Error>> {
    let api_key = env::var("{{DOMAIN_NAME_UPPER}}_API_KEY")
        .ok()
        .or_else(|| Some("demo_key".to_string()));
        
    let rpc_endpoint = env::var("{{DOMAIN_NAME_UPPER}}_RPC_ENDPOINT")
        .unwrap_or_else(|_| "https://{{DOMAIN_ENDPOINT_EXAMPLE}}".to_string());
        
    let mut additional_params = HashMap::new();
    additional_params.insert("timeout_seconds".to_string(), "30".to_string());
    
    let config = DomainConfig {
        rpc_endpoint,
        chain_id: "{{CHAIN_ID_EXAMPLE}}".to_string(),
        network_id: "{{NETWORK_ID_EXAMPLE}}".to_string(),
        gas_limit: Some("2000000".to_string()),
        api_key,
        additional_params,
    };
    
    Ok(config)
}
```
"#;

#[cfg(test)]
mod tests {
    #[test]
    fn test_template_constants_exist() {
        assert!(!super::ADAPTER_TEMPLATE.is_empty());
        assert!(!super::EFFECT_METHOD_TEMPLATE.is_empty());
        assert!(!super::FACT_METHOD_TEMPLATE.is_empty());
        assert!(!super::PROOF_METHOD_TEMPLATE.is_empty());
        assert!(!super::RPC_CLIENT_TEMPLATE.is_empty());
        assert!(!super::TYPES_TEMPLATE.is_empty());
        assert!(!super::UTILS_TEMPLATE.is_empty());
        assert!(!super::ADAPTER_TEST_TEMPLATE.is_empty());
        assert!(!super::EFFECT_TEST_TEMPLATE.is_empty());
        assert!(!super::FACT_TEST_TEMPLATE.is_empty());
        assert!(!super::PROOF_TEST_TEMPLATE.is_empty());
        assert!(!super::README_TEMPLATE.is_empty());
        assert!(!super::API_DOCS_TEMPLATE.is_empty());
        assert!(!super::BASIC_EXAMPLE_TEMPLATE.is_empty());
        assert!(!super::COMPLETE_EXAMPLE_TEMPLATE.is_empty());
    }
} 