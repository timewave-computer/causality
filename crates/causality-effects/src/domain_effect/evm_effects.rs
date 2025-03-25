// EVM Domain Effects
//
// This module implements EVM-specific domain effects for Ethereum and EVM-compatible blockchains.

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

/// EVM Contract Call Effect
///
/// This effect represents a call to a smart contract on an EVM-compatible blockchain.
#[derive(Debug)]
pub struct EvmContractCallEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Contract address
    contract_address: String,
    
    /// Function name
    function_name: String,
    
    /// Function arguments
    args: Vec<String>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
    
    /// Whether to execute a state-changing call (transaction) or a view call (query)
    is_view: bool,
}

impl EvmContractCallEffect {
    /// Create a new EVM contract view (read-only) call effect
    pub fn new_view_call(
        domain_id: impl Into<String>,
        contract_address: impl Into<String>,
        function_name: impl Into<String>,
        args: Vec<String>
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            contract_address: contract_address.into(),
            function_name: function_name.into(),
            args,
            parameters: HashMap::new(),
            is_view: true,
        }
    }
    
    /// Create a new EVM contract transaction (state-changing) call effect
    pub fn new_transaction_call(
        domain_id: impl Into<String>,
        contract_address: impl Into<String>,
        function_name: impl Into<String>,
        args: Vec<String>
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            contract_address: contract_address.into(),
            function_name: function_name.into(),
            args,
            parameters: HashMap::new(),
            is_view: false,
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set gas limit parameter
    pub fn with_gas_limit(self, gas_limit: impl Into<String>) -> Self {
        self.with_parameter("gas_limit", gas_limit)
    }
    
    /// Set gas price parameter
    pub fn with_gas_price(self, gas_price: impl Into<String>) -> Self {
        self.with_parameter("gas_price", gas_price)
    }
    
    /// Set value parameter (amount of native currency to send with the call)
    pub fn with_value(self, value: impl Into<String>) -> Self {
        self.with_parameter("value", value)
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
    
    /// Get the function name
    pub fn function_name(&self) -> &str {
        &self.function_name
    }
    
    /// Get the function arguments
    pub fn args(&self) -> &[String] {
        &self.args
    }
    
    /// Check if this is a view call
    pub fn is_view(&self) -> bool {
        self.is_view
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<String>) -> EffectResult<EffectOutcome> {
        match result {
            Ok(value) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("result", value);
                
                // Add function details to outcome
                outcome = outcome.with_data("contract_address", self.contract_address.clone());
                outcome = outcome.with_data("function_name", self.function_name.clone());
                outcome = outcome.with_data("is_view", self.is_view.to_string());
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("EVM contract call failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for EvmContractCallEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        if self.is_view {
            "evm_contract_view"
        } else {
            "evm_contract_transaction"
        }
    }
    
    fn description(&self) -> &str {
        if self.is_view {
            "EVM contract view call"
        } else {
            "EVM contract transaction call"
        }
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for EvmContractCallEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// EVM State Query Effect
///
/// This effect represents a query for state data on an EVM-compatible blockchain.
#[derive(Debug)]
pub struct EvmStateQueryEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Query type
    query_type: EvmStateQueryType,
    
    /// Parameters
    parameters: HashMap<String, String>,
}

/// Type of EVM state query
#[derive(Debug, Clone)]
pub enum EvmStateQueryType {
    /// Query account balance
    Balance(String),
    
    /// Query storage at address and slot
    Storage(String, String),
    
    /// Query account code
    Code(String),
    
    /// Query account nonce
    Nonce(String),
    
    /// Query block information
    Block(String),
    
    /// Query transaction information
    Transaction(String),
    
    /// Query receipt information
    Receipt(String),
    
    /// Query gas price
    GasPrice,
    
    /// Query gas limit
    GasLimit,
    
    /// Query chain ID
    ChainId,
}

impl EvmStateQueryEffect {
    /// Create a new EVM state query effect
    pub fn new(domain_id: impl Into<String>, query_type: EvmStateQueryType) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            query_type,
            parameters: HashMap::new(),
        }
    }
    
    /// Create a balance query
    pub fn balance(domain_id: impl Into<String>, address: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::Balance(address.into()))
    }
    
    /// Create a storage query
    pub fn storage(
        domain_id: impl Into<String>, 
        address: impl Into<String>, 
        slot: impl Into<String>
    ) -> Self {
        Self::new(domain_id, EvmStateQueryType::Storage(address.into(), slot.into()))
    }
    
    /// Create a code query
    pub fn code(domain_id: impl Into<String>, address: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::Code(address.into()))
    }
    
    /// Create a nonce query
    pub fn nonce(domain_id: impl Into<String>, address: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::Nonce(address.into()))
    }
    
    /// Create a block query
    pub fn block(domain_id: impl Into<String>, block_id: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::Block(block_id.into()))
    }
    
    /// Create a transaction query
    pub fn transaction(domain_id: impl Into<String>, tx_hash: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::Transaction(tx_hash.into()))
    }
    
    /// Create a receipt query
    pub fn receipt(domain_id: impl Into<String>, tx_hash: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::Receipt(tx_hash.into()))
    }
    
    /// Create a gas price query
    pub fn gas_price(domain_id: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::GasPrice)
    }
    
    /// Create a gas limit query
    pub fn gas_limit(domain_id: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::GasLimit)
    }
    
    /// Create a chain ID query
    pub fn chain_id(domain_id: impl Into<String>) -> Self {
        Self::new(domain_id, EvmStateQueryType::ChainId)
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Specify a block number
    pub fn at_block(self, block_number: impl Into<String>) -> Self {
        self.with_parameter("block_number", block_number)
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the query type
    pub fn query_type(&self) -> &EvmStateQueryType {
        &self.query_type
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<String>) -> EffectResult<EffectOutcome> {
        match result {
            Ok(value) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("result", value);
                
                // Add query details to outcome
                match &self.query_type {
                    EvmStateQueryType::Balance(address) => {
                        outcome = outcome.with_data("query_type", "balance");
                        outcome = outcome.with_data("address", address);
                    },
                    EvmStateQueryType::Storage(address, slot) => {
                        outcome = outcome.with_data("query_type", "storage");
                        outcome = outcome.with_data("address", address);
                        outcome = outcome.with_data("slot", slot);
                    },
                    EvmStateQueryType::Code(address) => {
                        outcome = outcome.with_data("query_type", "code");
                        outcome = outcome.with_data("address", address);
                    },
                    EvmStateQueryType::Nonce(address) => {
                        outcome = outcome.with_data("query_type", "nonce");
                        outcome = outcome.with_data("address", address);
                    },
                    EvmStateQueryType::Block(block_id) => {
                        outcome = outcome.with_data("query_type", "block");
                        outcome = outcome.with_data("block_id", block_id);
                    },
                    EvmStateQueryType::Transaction(tx_hash) => {
                        outcome = outcome.with_data("query_type", "transaction");
                        outcome = outcome.with_data("tx_hash", tx_hash);
                    },
                    EvmStateQueryType::Receipt(tx_hash) => {
                        outcome = outcome.with_data("query_type", "receipt");
                        outcome = outcome.with_data("tx_hash", tx_hash);
                    },
                    EvmStateQueryType::GasPrice => {
                        outcome = outcome.with_data("query_type", "gas_price");
                    },
                    EvmStateQueryType::GasLimit => {
                        outcome = outcome.with_data("query_type", "gas_limit");
                    },
                    EvmStateQueryType::ChainId => {
                        outcome = outcome.with_data("query_type", "chain_id");
                    },
                }
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("EVM state query failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for EvmStateQueryEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "evm_state_query"
    }
    
    fn description(&self) -> &str {
        "EVM state query"
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for EvmStateQueryEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// EVM Gas Estimation Effect
///
/// This effect represents a gas estimation for a transaction on an EVM-compatible blockchain.
#[derive(Debug)]
pub struct EvmGasEstimationEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Contract address
    contract_address: String,
    
    /// Function name
    function_name: String,
    
    /// Function arguments
    args: Vec<String>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl EvmGasEstimationEffect {
    /// Create a new EVM gas estimation effect
    pub fn new(
        domain_id: impl Into<String>,
        contract_address: impl Into<String>,
        function_name: impl Into<String>,
        args: Vec<String>
    ) -> Self {
        Self {
            id: EffectId::new(),
            domain_id: domain_id.into(),
            contract_address: contract_address.into(),
            function_name: function_name.into(),
            args,
            parameters: HashMap::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set value parameter (amount of native currency to send with the call)
    pub fn with_value(self, value: impl Into<String>) -> Self {
        self.with_parameter("value", value)
    }
    
    /// Specify a caller address
    pub fn with_from(self, from: impl Into<String>) -> Self {
        self.with_parameter("from", from)
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
    
    /// Get the function name
    pub fn function_name(&self) -> &str {
        &self.function_name
    }
    
    /// Get the function arguments
    pub fn args(&self) -> &[String] {
        &self.args
    }
    
    /// Create a domain context from an effect context
    pub fn create_domain_context<'a>(&self, context: &'a EffectContext) -> DomainContext<'a> {
        DomainContext::new(context)
    }
    
    /// Map the domain result to an effect outcome
    pub fn map_outcome(&self, result: DomainResult<u64>) -> EffectResult<EffectOutcome> {
        match result {
            Ok(gas) => {
                let mut outcome = EffectOutcome::success(self.id.clone());
                outcome = outcome.with_data("gas_estimate", gas.to_string());
                
                // Add function details to outcome
                outcome = outcome.with_data("contract_address", self.contract_address.clone());
                outcome = outcome.with_data("function_name", self.function_name.clone());
                
                Ok(outcome)
            },
            Err(e) => {
                Ok(EffectOutcome::failure(self.id.clone(), format!("EVM gas estimation failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl Effect for EvmGasEstimationEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "evm_gas_estimation"
    }
    
    fn description(&self) -> &str {
        "EVM gas estimation"
    }
    
    async fn execute(&self, _context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Base implementation - will be handled by a specific handler
        Err(EffectError::Unimplemented)
    }
}

impl DomainAdapterEffect for EvmGasEstimationEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Helper functions for creating EVM-specific effects

/// Create a new EVM contract view call effect
pub fn evm_view_call(
    domain_id: impl Into<String>,
    contract_address: impl Into<String>,
    function_name: impl Into<String>,
    args: Vec<String>
) -> EvmContractCallEffect {
    EvmContractCallEffect::new_view_call(domain_id, contract_address, function_name, args)
}

/// Create a new EVM contract transaction call effect
pub fn evm_transaction_call(
    domain_id: impl Into<String>,
    contract_address: impl Into<String>,
    function_name: impl Into<String>,
    args: Vec<String>
) -> EvmContractCallEffect {
    EvmContractCallEffect::new_transaction_call(domain_id, contract_address, function_name, args)
}

/// Query EVM account balance
pub fn evm_balance(domain_id: impl Into<String>, address: impl Into<String>) -> EvmStateQueryEffect {
    EvmStateQueryEffect::balance(domain_id, address)
}

/// Query EVM storage
pub fn evm_storage(
    domain_id: impl Into<String>, 
    address: impl Into<String>, 
    slot: impl Into<String>
) -> EvmStateQueryEffect {
    EvmStateQueryEffect::storage(domain_id, address, slot)
}

/// Query EVM account code
pub fn evm_code(domain_id: impl Into<String>, address: impl Into<String>) -> EvmStateQueryEffect {
    EvmStateQueryEffect::code(domain_id, address)
}

/// Query EVM transaction
pub fn evm_transaction(domain_id: impl Into<String>, tx_hash: impl Into<String>) -> EvmStateQueryEffect {
    EvmStateQueryEffect::transaction(domain_id, tx_hash)
}

/// Query EVM block
pub fn evm_block(domain_id: impl Into<String>, block_id: impl Into<String>) -> EvmStateQueryEffect {
    EvmStateQueryEffect::block(domain_id, block_id)
}

/// Estimate gas for an EVM transaction
pub fn evm_estimate_gas(
    domain_id: impl Into<String>,
    contract_address: impl Into<String>,
    function_name: impl Into<String>,
    args: Vec<String>
) -> EvmGasEstimationEffect {
    EvmGasEstimationEffect::new(domain_id, contract_address, function_name, args)
} 