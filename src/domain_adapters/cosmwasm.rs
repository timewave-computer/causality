//! CosmWasm Domain Adapter
//!
//! This module contains adapter implementations for CosmWasm-compatible blockchains,
//! providing integration with CosmWasm smart contracts and the Cosmos SDK ecosystem.

use std::{collections::HashMap, sync::Arc};
use crate::error::{Error, Result};

use super::{
    interfaces::{
        VmType, 
        VmAdapter, 
        CompilationAdapter,
        ZkProofAdapter,
        CrossVmAdapter,
        MultiVmAdapterConfig,
    },
    schemas::{
        AdapterSchema,
        DomainId,
        EffectDefinition,
        FactDefinition,
        ProofDefinition,
    },
    utils::{
        CrossVmBroker,
        CrossVmRequest,
        CrossVmResponse,
    },
    validation::{
        ValidationContext,
        ValidationResult,
        EffectValidator,
    },
};

/// CosmWasm virtual machine type
pub const COSMWASM_VM_TYPE: VmType = VmType::CosmWasm;

/// Contract address type for CosmWasm
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CosmWasmAddress(pub String);

impl CosmWasmAddress {
    /// Create a new CosmWasm address
    pub fn new(address: impl Into<String>) -> Self {
        Self(address.into())
    }
    
    /// Get the address as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CosmWasmAddress {
    fn from(address: String) -> Self {
        Self(address)
    }
}

/// CosmWasm contract code
#[derive(Debug, Clone)]
pub struct CosmWasmCode {
    /// Wasm bytecode
    pub bytecode: Vec<u8>,
    /// Schema for the contract (if available)
    pub schema: Option<serde_json::Value>,
    /// Source code (if available)
    pub source: Option<String>,
}

impl CosmWasmCode {
    /// Create a new CosmWasm contract code
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self {
            bytecode,
            schema: None,
            source: None,
        }
    }
    
    /// Set the schema for the contract
    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.schema = Some(schema);
        self
    }
    
    /// Set the source code for the contract
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// CosmWasm contract message
#[derive(Debug, Clone)]
pub struct CosmWasmMessage {
    /// Message type (instantiate, execute, query, etc.)
    pub msg_type: String,
    /// Message content
    pub content: serde_json::Value,
    /// Sender address
    pub sender: Option<CosmWasmAddress>,
    /// Funds sent with the message
    pub funds: Vec<(String, u64)>,
}

impl CosmWasmMessage {
    /// Create a new CosmWasm message
    pub fn new(msg_type: impl Into<String>, content: serde_json::Value) -> Self {
        Self {
            msg_type: msg_type.into(),
            content,
            sender: None,
            funds: Vec::new(),
        }
    }
    
    /// Set the sender for the message
    pub fn with_sender(mut self, sender: CosmWasmAddress) -> Self {
        self.sender = Some(sender);
        self
    }
    
    /// Add funds to the message
    pub fn add_funds(mut self, denom: impl Into<String>, amount: u64) -> Self {
        self.funds.push((denom.into(), amount));
        self
    }
}

/// CosmWasm query result
#[derive(Debug, Clone)]
pub struct CosmWasmQueryResult {
    /// Result data
    pub data: serde_json::Value,
    /// Error message (if any)
    pub error: Option<String>,
}

/// CosmWasm execution result
#[derive(Debug, Clone)]
pub struct CosmWasmExecutionResult {
    /// Result data
    pub data: serde_json::Value,
    /// Events emitted
    pub events: Vec<(String, HashMap<String, String>)>,
    /// Error message (if any)
    pub error: Option<String>,
}

/// CosmWasm adapter configuration
#[derive(Debug, Clone)]
pub struct CosmWasmAdapterConfig {
    /// Domain ID
    pub domain_id: DomainId,
    /// Chain ID
    pub chain_id: String,
    /// RPC endpoints
    pub rpc_endpoints: Vec<String>,
    /// Account prefix
    pub account_prefix: String,
    /// Gas price
    pub gas_price: Option<String>,
    /// Authorization token (if needed)
    pub auth_token: Option<String>,
    /// Debug mode
    pub debug_mode: bool,
}

impl From<MultiVmAdapterConfig> for CosmWasmAdapterConfig {
    fn from(config: MultiVmAdapterConfig) -> Self {
        let mut result = Self {
            domain_id: config.domain_id,
            chain_id: config.extra_config.get("chain_id")
                .and_then(|v| v.as_str())
                .unwrap_or("cosmwasm")
                .to_string(),
            rpc_endpoints: config.api_endpoints,
            account_prefix: config.extra_config.get("account_prefix")
                .and_then(|v| v.as_str())
                .unwrap_or("wasm")
                .to_string(),
            gas_price: config.extra_config.get("gas_price")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            auth_token: config.auth.map(|a| a.to_string()),
            debug_mode: config.debug_mode,
        };
        
        result
    }
}

/// CosmWasm adapter implementation
#[derive(Debug)]
pub struct CosmWasmAdapter {
    /// Adapter configuration
    config: CosmWasmAdapterConfig,
    /// Effect definitions
    effect_definitions: HashMap<String, EffectDefinition>,
    /// Fact definitions
    fact_definitions: HashMap<String, FactDefinition>,
    /// Proof definitions
    proof_definitions: HashMap<String, ProofDefinition>,
}

impl CosmWasmAdapter {
    /// Create a new CosmWasm adapter
    pub fn new(config: CosmWasmAdapterConfig) -> Self {
        Self {
            config,
            effect_definitions: HashMap::new(),
            fact_definitions: HashMap::new(),
            proof_definitions: HashMap::new(),
        }
    }
    
    /// Register standard effect definitions
    fn register_standard_effects(&mut self) {
        // Deploy contract
        let deploy_effect = EffectDefinition::new(
            "deploy_contract",
            "Deploy a CosmWasm smart contract",
            vec![
                ("code", "Contract bytecode", "bytes"),
                ("init_msg", "Initialization message", "json"),
                ("label", "Contract label", "string"),
                ("admin", "Admin address (optional)", "string"),
            ],
        );
        self.effect_definitions.insert("deploy_contract".to_string(), deploy_effect);
        
        // Execute contract
        let execute_effect = EffectDefinition::new(
            "execute_contract",
            "Execute a CosmWasm smart contract",
            vec![
                ("contract_addr", "Contract address", "string"),
                ("msg", "Execute message", "json"),
                ("funds", "Funds to send", "json"),
            ],
        );
        self.effect_definitions.insert("execute_contract".to_string(), execute_effect);
        
        // Migrate contract
        let migrate_effect = EffectDefinition::new(
            "migrate_contract",
            "Migrate a CosmWasm smart contract",
            vec![
                ("contract_addr", "Contract address", "string"),
                ("new_code_id", "New code ID", "number"),
                ("msg", "Migration message", "json"),
            ],
        );
        self.effect_definitions.insert("migrate_contract".to_string(), migrate_effect);
        
        // Update admin
        let update_admin_effect = EffectDefinition::new(
            "update_admin",
            "Update a contract's admin",
            vec![
                ("contract_addr", "Contract address", "string"),
                ("new_admin", "New admin address", "string"),
            ],
        );
        self.effect_definitions.insert("update_admin".to_string(), update_admin_effect);
    }
    
    /// Register standard fact definitions
    fn register_standard_facts(&mut self) {
        // Contract info
        let contract_info_fact = FactDefinition::new(
            "contract_info",
            "Information about a deployed contract",
            vec![
                ("contract_addr", "Contract address", "string"),
                ("code_id", "Code ID", "number"),
                ("creator", "Creator address", "string"),
                ("admin", "Admin address", "string"),
                ("label", "Contract label", "string"),
            ],
        );
        self.fact_definitions.insert("contract_info".to_string(), contract_info_fact);
        
        // Code info
        let code_info_fact = FactDefinition::new(
            "code_info",
            "Information about uploaded contract code",
            vec![
                ("code_id", "Code ID", "number"),
                ("creator", "Creator address", "string"),
                ("checksum", "Code checksum", "string"),
            ],
        );
        self.fact_definitions.insert("code_info".to_string(), code_info_fact);
        
        // Query result
        let query_result_fact = FactDefinition::new(
            "query_result",
            "Result of a contract query",
            vec![
                ("contract_addr", "Contract address", "string"),
                ("query", "Query message", "json"),
                ("result", "Query result", "json"),
            ],
        );
        self.fact_definitions.insert("query_result".to_string(), query_result_fact);
    }
    
    /// Deploy a CosmWasm contract
    pub fn deploy_contract(
        &self,
        code: CosmWasmCode,
        init_msg: serde_json::Value,
        label: impl Into<String>,
        admin: Option<CosmWasmAddress>,
    ) -> Result<CosmWasmAddress> {
        // In a real implementation, this would interact with a CosmWasm chain
        // For now, we'll return a placeholder address
        let addr = CosmWasmAddress::new(format!("wasm1deploy{}", label.into()));
        
        if self.config.debug_mode {
            println!("Deployed contract: {:?}", addr);
        }
        
        Ok(addr)
    }
    
    /// Execute a CosmWasm contract
    pub fn execute_contract(
        &self,
        contract_addr: &CosmWasmAddress,
        msg: serde_json::Value,
        funds: Vec<(String, u64)>,
        sender: Option<&CosmWasmAddress>,
    ) -> Result<CosmWasmExecutionResult> {
        // In a real implementation, this would interact with a CosmWasm chain
        // For now, we'll return a placeholder result
        let result = CosmWasmExecutionResult {
            data: serde_json::json!({"success": true}),
            events: vec![
                ("wasm".to_string(), 
                 HashMap::from([
                     ("contract_address".to_string(), contract_addr.0.clone()),
                     ("action".to_string(), "execute".to_string()),
                 ]))
            ],
            error: None,
        };
        
        if self.config.debug_mode {
            println!("Executed contract: {:?} with msg: {:?}", contract_addr, msg);
        }
        
        Ok(result)
    }
    
    /// Query a CosmWasm contract
    pub fn query_contract(
        &self,
        contract_addr: &CosmWasmAddress,
        query_msg: serde_json::Value,
    ) -> Result<CosmWasmQueryResult> {
        // In a real implementation, this would interact with a CosmWasm chain
        // For now, we'll return a placeholder result
        let result = CosmWasmQueryResult {
            data: serde_json::json!({"result": "query_result"}),
            error: None,
        };
        
        if self.config.debug_mode {
            println!("Queried contract: {:?} with msg: {:?}", contract_addr, query_msg);
        }
        
        Ok(result)
    }
}

impl VmAdapter for CosmWasmAdapter {
    fn vm_type(&self) -> VmType {
        COSMWASM_VM_TYPE
    }
    
    fn domain_id(&self) -> &DomainId {
        &self.config.domain_id
    }
    
    fn schema(&self) -> AdapterSchema {
        AdapterSchema::new(
            self.config.domain_id.clone(),
            COSMWASM_VM_TYPE,
            self.effect_definitions.values().cloned().collect(),
            self.fact_definitions.values().cloned().collect(),
            self.proof_definitions.values().cloned().collect(),
        )
    }
}

impl CompilationAdapter for CosmWasmAdapter {
    fn compile_program(&self, source: &str, options: &HashMap<String, String>) -> Result<Vec<u8>> {
        // In a real implementation, this would compile Rust source code to Wasm
        // For now, we'll return a placeholder bytecode
        let bytecode = vec![0u8; 32]; // Placeholder Wasm bytecode
        
        if self.config.debug_mode {
            println!("Compiled CosmWasm program: {} bytes", bytecode.len());
        }
        
        Ok(bytecode)
    }
    
    fn validate_program(&self, bytecode: &[u8]) -> Result<()> {
        // In a real implementation, this would validate Wasm bytecode
        // For now, we'll just check that the bytecode is not empty
        if bytecode.is_empty() {
            return Err(Error::Validation("Empty bytecode".to_string()));
        }
        
        Ok(())
    }
}

/// CosmWasm-specific validator for effects
pub struct CosmWasmEffectValidator;

impl CosmWasmEffectValidator {
    /// Create a new CosmWasm effect validator
    pub fn new() -> Self {
        Self
    }
}

impl EffectValidator for CosmWasmEffectValidator {
    fn validate_effect(&self, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::valid();
        
        // Check that the effect is supported
        if !self.supports_effect_type(&context.effect_type) {
            return ValidationResult::invalid(
                "effect_type",
                format!("Unsupported effect type: {}", context.effect_type),
                "UNSUPPORTED_EFFECT",
            );
        }
        
        // Validate based on effect type
        match context.effect_type.as_str() {
            "deploy_contract" => {
                // Validate required fields
                if !context.params.contains_key("code") {
                    result.add_error(
                        "code",
                        "Contract bytecode is required",
                        "MISSING_FIELD",
                    );
                }
                
                if !context.params.contains_key("init_msg") {
                    result.add_error(
                        "init_msg",
                        "Initialization message is required",
                        "MISSING_FIELD",
                    );
                }
                
                if !context.params.contains_key("label") {
                    result.add_error(
                        "label",
                        "Contract label is required",
                        "MISSING_FIELD",
                    );
                }
            }
            "execute_contract" => {
                // Validate required fields
                if !context.params.contains_key("contract_addr") {
                    result.add_error(
                        "contract_addr",
                        "Contract address is required",
                        "MISSING_FIELD",
                    );
                }
                
                if !context.params.contains_key("msg") {
                    result.add_error(
                        "msg",
                        "Execute message is required",
                        "MISSING_FIELD",
                    );
                }
            }
            "migrate_contract" => {
                // Validate required fields
                if !context.params.contains_key("contract_addr") {
                    result.add_error(
                        "contract_addr",
                        "Contract address is required",
                        "MISSING_FIELD",
                    );
                }
                
                if !context.params.contains_key("new_code_id") {
                    result.add_error(
                        "new_code_id",
                        "New code ID is required",
                        "MISSING_FIELD",
                    );
                }
                
                if !context.params.contains_key("msg") {
                    result.add_error(
                        "msg",
                        "Migration message is required",
                        "MISSING_FIELD",
                    );
                }
            }
            "update_admin" => {
                // Validate required fields
                if !context.params.contains_key("contract_addr") {
                    result.add_error(
                        "contract_addr",
                        "Contract address is required",
                        "MISSING_FIELD",
                    );
                }
                
                if !context.params.contains_key("new_admin") {
                    result.add_error(
                        "new_admin",
                        "New admin address is required",
                        "MISSING_FIELD",
                    );
                }
            }
            _ => {
                result.add_error(
                    "effect_type",
                    format!("Unknown effect type: {}", context.effect_type),
                    "UNKNOWN_EFFECT",
                );
            }
        }
        
        result
    }
    
    fn supports_effect_type(&self, effect_type: &str) -> bool {
        matches!(
            effect_type,
            "deploy_contract" | "execute_contract" | "migrate_contract" | "update_admin"
        )
    }
}

/// Factory for creating CosmWasm adapters
pub struct CosmWasmAdapterFactory;

impl CosmWasmAdapterFactory {
    /// Create a new CosmWasm adapter factory
    pub fn new() -> Self {
        Self
    }
    
    /// Create a CosmWasm adapter from a configuration
    pub fn create_adapter(&self, config: MultiVmAdapterConfig) -> Result<CosmWasmAdapter> {
        // Convert the generic config to a CosmWasm-specific config
        let cosmwasm_config = CosmWasmAdapterConfig::from(config);
        
        // Create the adapter
        let mut adapter = CosmWasmAdapter::new(cosmwasm_config);
        
        // Register standard definitions
        adapter.register_standard_effects();
        adapter.register_standard_facts();
        
        Ok(adapter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosmwasm_adapter_basics() {
        let config = CosmWasmAdapterConfig {
            domain_id: DomainId::new("cosmwasm-test"),
            chain_id: "testing-1".to_string(),
            rpc_endpoints: vec!["http://localhost:26657".to_string()],
            account_prefix: "wasm".to_string(),
            gas_price: None,
            auth_token: None,
            debug_mode: true,
        };
        
        let adapter = CosmWasmAdapter::new(config);
        
        assert_eq!(adapter.vm_type(), COSMWASM_VM_TYPE);
        assert_eq!(adapter.domain_id().as_str(), "cosmwasm-test");
    }
    
    #[test]
    fn test_cosmwasm_address() {
        let addr = CosmWasmAddress::new("wasm1abcdef");
        
        assert_eq!(addr.as_str(), "wasm1abcdef");
        
        let addr2: CosmWasmAddress = "wasm1ghijkl".to_string().into();
        assert_eq!(addr2.as_str(), "wasm1ghijkl");
    }
    
    #[test]
    fn test_cosmwasm_validator() {
        let validator = CosmWasmEffectValidator::new();
        
        assert!(validator.supports_effect_type("deploy_contract"));
        assert!(validator.supports_effect_type("execute_contract"));
        assert!(!validator.supports_effect_type("unknown_effect"));
        
        // Create a validation context for deploy_contract
        let mut context = ValidationContext::new(
            DomainId::new("cosmwasm-test"),
            COSMWASM_VM_TYPE,
            "deploy_contract".to_string(),
        );
        
        // Missing required fields
        let result = validator.validate_effect(&context);
        assert!(!result.is_valid());
        
        // Add required fields
        context.add_param("code", serde_json::json!([0, 1, 2, 3]));
        context.add_param("init_msg", serde_json::json!({"count": 0}));
        context.add_param("label", serde_json::json!("my-counter"));
        
        // Should be valid now
        let result = validator.validate_effect(&context);
        assert!(result.is_valid());
    }
} 