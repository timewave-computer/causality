// CosmWasm Domain Effects
//
// This module implements CosmWasm-specific domain effects for Cosmos-based blockchains
// that support the CosmWasm smart contract platform.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use causality_domain::{
    domain::{DomainId, Transaction, TransactionId, TransactionReceipt},
    fact::{FactQuery, Fact, FactResult},
    types::{Result as DomainResult, Error as DomainError},
};

use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
use crate::effect_id::EffectId;
use crate::domain_effect::{
    DomainAdapterEffect, DomainContext, DomainQueryEffect, DomainTransactionEffect
};

/// CosmWasm Contract Execute Effect
///
/// This effect represents an execution of a smart contract on a CosmWasm-compatible blockchain.
#[derive(Debug)]
pub struct CosmWasmExecuteEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Contract address
    contract_address: String,
    
    /// Execute message (JSON)
    msg: String,
    
    /// Funds to send with the execution (optional)
    funds: Option<Vec<(String, u128)>>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl CosmWasmExecuteEffect {
    /// Create a new CosmWasm contract execute effect
    pub fn new(
        domain_id: impl Into<String>,
        contract_address: impl Into<String>,
        msg: impl Into<String>,
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            contract_address: contract_address.into(),
            msg: msg.into(),
            funds: None,
            parameters: HashMap::new(),
        }
    }
    
    /// Add funds to send with the execution
    pub fn with_funds(mut self, denom: impl Into<String>, amount: u128) -> Self {
        let funds = self.funds.get_or_insert_with(Vec::new);
        funds.push((denom.into(), amount));
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set the sender address
    pub fn with_sender(self, sender: impl Into<String>) -> Self {
        self.with_parameter("sender", sender)
    }
    
    /// Set the gas limit parameter
    pub fn with_gas_limit(self, gas_limit: impl Into<String>) -> Self {
        self.with_parameter("gas_limit", gas_limit)
    }
    
    /// Set the fee amount
    pub fn with_fee(self, fee_amount: impl Into<String>, fee_denom: impl Into<String>) -> Self {
        let mut result = self.with_parameter("fee_amount", fee_amount);
        result = result.with_parameter("fee_denom", fee_denom);
        result
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
    
    /// Get the execute message
    pub fn msg(&self) -> &str {
        &self.msg
    }
    
    /// Get the funds to send with execution
    pub fn funds(&self) -> Option<&Vec<(String, u128)>> {
        self.funds.as_ref()
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<String>) -> EffectResult<EffectOutcome> {
        match result {
            Ok(tx_hash) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("tx_hash", tx_hash);
                
                // Add function details to outcome
                outcome = outcome.with_data("contract_address", self.contract_address.clone());
                outcome = outcome.with_data("msg_type", "execute");
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("CosmWasm execute failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for CosmWasmExecuteEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cosmwasm_execute"
    }
    
    fn description(&self) -> &str {
        "CosmWasm contract execute"
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for CosmWasmExecuteEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// CosmWasm Contract Query Effect
///
/// This effect represents a query to a smart contract on a CosmWasm-compatible blockchain.
#[derive(Debug)]
pub struct CosmWasmQueryEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Contract address
    contract_address: String,
    
    /// Query message (JSON)
    query: String,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl CosmWasmQueryEffect {
    /// Create a new CosmWasm contract query effect
    pub fn new(
        domain_id: impl Into<String>,
        contract_address: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            contract_address: contract_address.into(),
            query: query.into(),
            parameters: HashMap::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Specify a block height to query at
    pub fn at_height(self, height: u64) -> Self {
        self.with_parameter("height", height.to_string())
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
    
    /// Get the query message
    pub fn query(&self) -> &str {
        &self.query
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<String>) -> EffectResult<EffectOutcome> {
        match result {
            Ok(data) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("result", data);
                
                // Add query details to outcome
                outcome = outcome.with_data("contract_address", self.contract_address.clone());
                outcome = outcome.with_data("msg_type", "query");
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("CosmWasm query failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for CosmWasmQueryEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cosmwasm_query"
    }
    
    fn description(&self) -> &str {
        "CosmWasm contract query"
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for CosmWasmQueryEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// CosmWasm Contract Instantiate Effect
///
/// This effect represents instantiation of a new smart contract on a CosmWasm-compatible blockchain.
#[derive(Debug)]
pub struct CosmWasmInstantiateEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Code ID of the contract to instantiate
    code_id: u64,
    
    /// Instantiate message (JSON)
    msg: String,
    
    /// Label for the contract
    label: String,
    
    /// Funds to send with the instantiation (optional)
    funds: Option<Vec<(String, u128)>>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl CosmWasmInstantiateEffect {
    /// Create a new CosmWasm contract instantiate effect
    pub fn new(
        domain_id: impl Into<String>,
        code_id: u64,
        msg: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            code_id,
            msg: msg.into(),
            label: label.into(),
            funds: None,
            parameters: HashMap::new(),
        }
    }
    
    /// Add funds to send with the instantiation
    pub fn with_funds(mut self, denom: impl Into<String>, amount: u128) -> Self {
        let funds = self.funds.get_or_insert_with(Vec::new);
        funds.push((denom.into(), amount));
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set the sender address
    pub fn with_sender(self, sender: impl Into<String>) -> Self {
        self.with_parameter("sender", sender)
    }
    
    /// Set the admin address
    pub fn with_admin(self, admin: impl Into<String>) -> Self {
        self.with_parameter("admin", admin)
    }
    
    /// Set the gas limit parameter
    pub fn with_gas_limit(self, gas_limit: impl Into<String>) -> Self {
        self.with_parameter("gas_limit", gas_limit)
    }
    
    /// Set the fee amount
    pub fn with_fee(self, fee_amount: impl Into<String>, fee_denom: impl Into<String>) -> Self {
        let mut result = self.with_parameter("fee_amount", fee_amount);
        result = result.with_parameter("fee_denom", fee_denom);
        result
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the code ID
    pub fn code_id(&self) -> u64 {
        self.code_id
    }
    
    /// Get the instantiate message
    pub fn msg(&self) -> &str {
        &self.msg
    }
    
    /// Get the contract label
    pub fn label(&self) -> &str {
        &self.label
    }
    
    /// Get the funds to send with instantiation
    pub fn funds(&self) -> Option<&Vec<(String, u128)>> {
        self.funds.as_ref()
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<(String, String)>) -> EffectResult<EffectOutcome> {
        match result {
            Ok((tx_hash, contract_address)) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("tx_hash", tx_hash);
                outcome = outcome.with_data("contract_address", contract_address);
                
                // Add function details to outcome
                outcome = outcome.with_data("code_id", self.code_id.to_string());
                outcome = outcome.with_data("label", self.label.clone());
                outcome = outcome.with_data("msg_type", "instantiate");
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("CosmWasm instantiate failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for CosmWasmInstantiateEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cosmwasm_instantiate"
    }
    
    fn description(&self) -> &str {
        "CosmWasm contract instantiate"
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for CosmWasmInstantiateEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// CosmWasm Code Upload Effect
///
/// This effect represents uploading new Wasm code to a CosmWasm-compatible blockchain.
#[derive(Debug)]
pub struct CosmWasmCodeUploadEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Wasm bytecode (base64 encoded)
    wasm_bytecode: String,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl CosmWasmCodeUploadEffect {
    /// Create a new CosmWasm code upload effect
    pub fn new(
        domain_id: impl Into<String>,
        wasm_bytecode: impl Into<String>,
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            wasm_bytecode: wasm_bytecode.into(),
            parameters: HashMap::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set the sender address
    pub fn with_sender(self, sender: impl Into<String>) -> Self {
        self.with_parameter("sender", sender)
    }
    
    /// Set the gas limit parameter
    pub fn with_gas_limit(self, gas_limit: impl Into<String>) -> Self {
        self.with_parameter("gas_limit", gas_limit)
    }
    
    /// Set the fee amount
    pub fn with_fee(self, fee_amount: impl Into<String>, fee_denom: impl Into<String>) -> Self {
        let mut result = self.with_parameter("fee_amount", fee_amount);
        result = result.with_parameter("fee_denom", fee_denom);
        result
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the Wasm bytecode
    pub fn wasm_bytecode(&self) -> &str {
        &self.wasm_bytecode
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<(String, u64)>) -> EffectResult<EffectOutcome> {
        match result {
            Ok((tx_hash, code_id)) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("tx_hash", tx_hash);
                outcome = outcome.with_data("code_id", code_id.to_string());
                
                // Add upload details to outcome
                outcome = outcome.with_data("bytecode_size", self.wasm_bytecode.len().to_string());
                outcome = outcome.with_data("msg_type", "upload");
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("CosmWasm code upload failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for CosmWasmCodeUploadEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cosmwasm_upload"
    }
    
    fn description(&self) -> &str {
        "CosmWasm code upload"
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for CosmWasmCodeUploadEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Helper functions for creating CosmWasm-specific effects

/// Create a new CosmWasm contract execute effect
pub fn cosmwasm_execute(
    domain_id: impl Into<String>,
    contract_address: impl Into<String>,
    msg: impl Into<String>,
) -> CosmWasmExecuteEffect {
    CosmWasmExecuteEffect::new(domain_id, contract_address, msg)
}

/// Create a new CosmWasm contract query effect
pub fn cosmwasm_query(
    domain_id: impl Into<String>,
    contract_address: impl Into<String>,
    query: impl Into<String>,
) -> CosmWasmQueryEffect {
    CosmWasmQueryEffect::new(domain_id, contract_address, query)
}

/// Create a new CosmWasm contract instantiate effect
pub fn cosmwasm_instantiate(
    domain_id: impl Into<String>,
    code_id: u64,
    msg: impl Into<String>,
    label: impl Into<String>,
) -> CosmWasmInstantiateEffect {
    CosmWasmInstantiateEffect::new(domain_id, code_id, msg, label)
}

/// Create a new CosmWasm code upload effect
pub fn cosmwasm_upload(
    domain_id: impl Into<String>,
    wasm_bytecode: impl Into<String>,
) -> CosmWasmCodeUploadEffect {
    CosmWasmCodeUploadEffect::new(domain_id, wasm_bytecode)
} 