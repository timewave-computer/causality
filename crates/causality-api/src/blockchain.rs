//! Blockchain client wrappers for Causality API
//!
//! This module provides convenient wrappers around valence-domain-clients
//! for interacting with Ethereum and Neutron blockchains.

use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::Engine;

// Import the actual client types from valence-domain-clients
use valence_domain_clients::EthereumClient;
use valence_domain_clients::NeutronClient;
use valence_coprocessor_client::CoprocessorClient;

// Import ZK message types
use crate::types::{
    ZkMessage, ZkMessageSubmissionRequest, ZkMessageSubmissionResponse,
    BatchZkMessageSubmissionRequest, BatchZkMessageSubmissionResponse,
    ZkMessageSubmissionResult, ZkProofData, ZkInput, AuthorizationContext, TransactionStatus, SubmissionResultStatus, BatchStatus,
    ProofEncoding, ZkInputValue, ZkInputType, AuthorizationAction,
};

/// Wrapper around EthereumClient with Causality-specific functionality
#[derive(Clone)]
pub struct EthereumClientWrapper {
    /// Underlying Ethereum client
    client: Arc<EthereumClient>,
    
    /// RPC URL for the Ethereum node
    rpc_url: String,
}

impl std::fmt::Debug for EthereumClientWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthereumClientWrapper")
            .field("rpc_url", &self.rpc_url)
            .finish()
    }
}

impl EthereumClientWrapper {
    /// Create a new Ethereum client wrapper
    pub fn new(rpc_url: impl Into<String>, mnemonic: impl Into<String>) -> Result<Self> {
        let rpc_url = rpc_url.into();
        let mnemonic = mnemonic.into();
        let client = EthereumClient::new(&rpc_url, &mnemonic, None)?;
        
        Ok(Self {
            client: Arc::new(client),
            rpc_url: rpc_url.to_string(),
        })
    }
    
    /// Create a new Ethereum client wrapper with chain ID
    pub fn new_with_chain_id(
        rpc_url: impl Into<String>, 
        mnemonic: impl Into<String>,
        chain_id: &str,
    ) -> Result<Self> {
        let rpc_url = rpc_url.into();
        let mnemonic = mnemonic.into();
        let client = EthereumClient::new(&rpc_url, &mnemonic, Some(chain_id))?;
        
        Ok(Self {
            client: Arc::new(client),
            rpc_url: rpc_url.to_string(),
        })
    }
    
    /// Get the underlying Ethereum client
    pub fn client(&self) -> Arc<EthereumClient> {
        Arc::clone(&self.client)
    }
    
    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
    
    /// Get storage proof for a contract address and storage key
    /// This is a placeholder - actual implementation depends on the valence-domain-clients API
    pub async fn get_storage_proof(
        &self,
        _contract_address: &str,
        _storage_key: &str,
        _block_number: Option<u64>,
    ) -> Result<serde_json::Value> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder
        Ok(serde_json::json!({
            "contract_address": _contract_address,
            "storage_key": _storage_key,
            "proof": "placeholder_proof"
        }))
    }
    
    /// Get storage value for a contract address and storage key
    /// This is a placeholder - actual implementation depends on the valence-domain-clients API
    pub async fn get_storage_value(
        &self,
        _contract_address: &str,
        _storage_key: &str,
        _block_number: Option<u64>,
    ) -> Result<String> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder
        Ok(format!("0x{}", hex::encode([0u8; 32])))
    }
}

/// Wrapper around NeutronClient with Causality-specific functionality
#[derive(Clone)]
pub struct NeutronClientWrapper {
    /// Underlying Neutron client
    client: Arc<NeutronClient>,
    
    /// RPC URL for the Neutron node
    rpc_url: String,
    
    /// Chain ID
    chain_id: String,
    
    /// Gas price configuration
    gas_config: GasConfig,
    
    /// Transaction configuration
    tx_config: TransactionConfig,
}

/// Gas price configuration for Neutron transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    /// Gas price in untrn
    pub gas_price: u64,
    
    /// Gas adjustment factor
    pub gas_adjustment: f64,
    
    /// Maximum gas limit
    pub max_gas: u64,
}

/// Transaction configuration
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    /// Transaction timeout in seconds
    pub timeout: u64,
    
    /// Maximum retry attempts
    pub max_retries: u8,
    
    /// Confirmation blocks to wait
    pub confirmation_blocks: u64,
}

/// CosmWasm contract instance message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInstantiateMsg {
    /// Code ID of the contract
    pub code_id: u64,
    
    /// Admin address (optional)
    pub admin: Option<String>,
    
    /// Contract label
    pub label: String,
    
    /// Instantiation message
    pub msg: serde_json::Value,
    
    /// Funds to send with instantiation
    pub funds: Vec<Coin>,
}

/// CosmWasm contract execution message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecuteMsg {
    /// Contract address
    pub contract_address: String,
    
    /// Execution message
    pub msg: serde_json::Value,
    
    /// Funds to send with execution
    pub funds: Vec<Coin>,
}

/// CosmWasm contract migration message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMigrateMsg {
    /// Contract address
    pub contract_address: String,
    
    /// New code ID
    pub new_code_id: u64,
    
    /// Migration message
    pub msg: serde_json::Value,
}

/// CosmWasm contract query message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractQueryMsg {
    /// Contract address
    pub contract_address: String,
    
    /// Query message
    pub msg: serde_json::Value,
}

/// Coin type for Cosmos SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    /// Denomination
    pub denom: String,
    
    /// Amount
    pub amount: String,
}

/// Transaction response from Neutron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResponse {
    /// Transaction hash
    pub txhash: String,
    
    /// Transaction code (0 = success)
    pub code: u32,
    
    /// Transaction data
    pub data: Option<String>,
    
    /// Raw log
    pub raw_log: String,
    
    /// Transaction logs
    pub logs: Vec<TxLog>,
    
    /// Gas wanted
    pub gas_wanted: u64,
    
    /// Gas used
    pub gas_used: u64,
    
    /// Transaction height
    pub height: u64,
    
    /// Transaction timestamp
    pub timestamp: String,
}

/// Transaction log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxLog {
    /// Message index
    pub msg_index: u32,
    
    /// Log message
    pub log: String,
    
    /// Events
    pub events: Vec<TxEvent>,
}

/// Transaction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEvent {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,
    
    /// Event attributes
    pub attributes: Vec<TxEventAttribute>,
}

/// Transaction event attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEventAttribute {
    /// Attribute key
    pub key: String,
    
    /// Attribute value
    pub value: String,
}

/// Contract deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeploymentResult {
    /// Contract address
    pub contract_address: String,
    
    /// Transaction hash
    pub tx_hash: String,
    
    /// Code ID
    pub code_id: u64,
    
    /// Gas used
    pub gas_used: u64,
    
    /// Block height
    pub height: u64,
}

/// Contract execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecutionResult {
    /// Transaction hash
    pub tx_hash: String,
    
    /// Gas used
    pub gas_used: u64,
    
    /// Block height
    pub height: u64,
    
    /// Execution data
    pub data: Option<String>,
    
    /// Logs
    pub logs: Vec<TxLog>,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            gas_price: 1000, // 1000 untrn
            gas_adjustment: 1.5,
            max_gas: 2_000_000,
        }
    }
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            timeout: 60, // 60 seconds
            max_retries: 3,
            confirmation_blocks: 1,
        }
    }
}

impl std::fmt::Debug for NeutronClientWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NeutronClientWrapper")
            .field("rpc_url", &self.rpc_url)
            .field("chain_id", &self.chain_id)
            .field("gas_config", &self.gas_config)
            .finish()
    }
}

impl NeutronClientWrapper {
    /// Create a new Neutron client wrapper with default configuration
    pub async fn new(
        rpc_url: impl Into<String>,
        chain_id: impl Into<String>,
        mnemonic: impl Into<String>,
    ) -> Result<Self> {
        Self::new_with_config(
            rpc_url,
            chain_id,
            mnemonic,
            GasConfig::default(),
            TransactionConfig::default(),
        ).await
    }
    
    /// Create a new Neutron client wrapper with custom configuration
    pub async fn new_with_config(
        rpc_url: impl Into<String>,
        chain_id: impl Into<String>,
        mnemonic: impl Into<String>,
        gas_config: GasConfig,
        tx_config: TransactionConfig,
    ) -> Result<Self> {
        let rpc_url = rpc_url.into();
        let chain_id = chain_id.into();
        let mnemonic = mnemonic.into();
        let client = NeutronClient::new(&rpc_url, &chain_id, &mnemonic, None).await?;
        
        Ok(Self {
            client: Arc::new(client),
            rpc_url: rpc_url.to_string(),
            chain_id: chain_id.to_string(),
            gas_config,
            tx_config,
        })
    }
    
    /// Get the underlying Neutron client
    pub fn client(&self) -> Arc<NeutronClient> {
        Arc::clone(&self.client)
    }
    
    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
    
    /// Get the chain ID
    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }
    
    /// Get gas configuration
    pub fn gas_config(&self) -> &GasConfig {
        &self.gas_config
    }
    
    /// Get transaction configuration
    pub fn tx_config(&self) -> &TransactionConfig {
        &self.tx_config
    }
    
    /// Upload a contract code to the blockchain
    pub async fn upload_contract_code(
        &self,
        wasm_bytecode: &[u8],
    ) -> Result<u64> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder code ID
        let code_id = 1; // This would be returned from the actual upload
        log::info!("Uploaded contract code with {} bytes, got code ID: {}", wasm_bytecode.len(), code_id);
        Ok(code_id)
    }
    
    /// Instantiate a contract from uploaded code
    pub async fn instantiate_contract(
        &self,
        instantiate_msg: ContractInstantiateMsg,
    ) -> Result<ContractDeploymentResult> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder result
        let contract_address = format!("neutron1{:x}", rand::random::<u64>());
        let tx_hash = format!("tx_{:x}", rand::random::<u64>());
        
        log::info!("Instantiated contract {} from code ID {} with label '{}'", 
                  contract_address, instantiate_msg.code_id, instantiate_msg.label);
        
        Ok(ContractDeploymentResult {
            contract_address,
            tx_hash,
            code_id: instantiate_msg.code_id,
            gas_used: 500_000,
            height: 1000,
        })
    }
    
    /// Execute a contract message
    pub async fn execute_contract(
        &self,
        execute_msg: ContractExecuteMsg,
    ) -> Result<ContractExecutionResult> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder result
        let tx_hash = format!("exec_tx_{:x}", rand::random::<u64>());
        
        log::info!("Executed contract {} with message: {}", 
                  execute_msg.contract_address, execute_msg.msg);
        
        Ok(ContractExecutionResult {
            tx_hash,
            gas_used: 200_000,
            height: 1001,
            data: Some("execution_successful".to_string()),
            logs: Vec::new(),
        })
    }
    
    /// Query a contract
    pub async fn query_contract(
        &self,
        query_msg: ContractQueryMsg,
    ) -> Result<serde_json::Value> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder result
        log::info!("Querying contract {} with message: {}", 
                  query_msg.contract_address, query_msg.msg);
        
        Ok(serde_json::json!({
            "contract_address": query_msg.contract_address,
            "query": query_msg.msg,
            "result": "placeholder_query_result"
        }))
    }
    
    /// Migrate a contract to new code
    pub async fn migrate_contract(
        &self,
        migrate_msg: ContractMigrateMsg,
    ) -> Result<TxResponse> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder result
        let tx_hash = format!("migrate_tx_{:x}", rand::random::<u64>());
        
        log::info!("Migrated contract {} to code ID {} with message: {}", 
                  migrate_msg.contract_address, migrate_msg.new_code_id, migrate_msg.msg);
        
        Ok(TxResponse {
            txhash: tx_hash,
            code: 0,
            data: Some("migration_successful".to_string()),
            raw_log: "migration completed successfully".to_string(),
            logs: Vec::new(),
            gas_wanted: 300_000,
            gas_used: 250_000,
            height: 1002,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Get account balance for a specific denomination
    pub async fn get_balance(&self, _address: &str, denom: &str) -> Result<Coin> {
        // TODO: Implement using the actual valence-domain-clients API
        Ok(Coin {
            denom: denom.to_string(),
            amount: "1000000".to_string(), // 1M units
        })
    }
    
    /// Send tokens to another address
    pub async fn send_tokens(
        &self,
        to_address: &str,
        coins: Vec<Coin>,
    ) -> Result<TxResponse> {
        // TODO: Implement using the actual valence-domain-clients API
        let tx_hash = format!("send_tx_{:x}", rand::random::<u64>());
        
        log::info!("Sent {} tokens to {}", 
                  coins.iter().map(|c| format!("{}{}", c.amount, c.denom)).collect::<Vec<_>>().join(","),
                  to_address);
        
        Ok(TxResponse {
            txhash: tx_hash,
            code: 0,
            data: None,
            raw_log: "transfer successful".to_string(),
            logs: Vec::new(),
            gas_wanted: 100_000,
            gas_used: 80_000,
            height: 1003,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Wait for transaction confirmation
    pub async fn wait_for_tx_confirmation(&self, tx_hash: &str) -> Result<TxResponse> {
        // TODO: Implement actual transaction waiting logic
        // For now, simulate waiting and return success
        log::info!("Waiting for transaction {} confirmation", tx_hash);
        
        // Simulate confirmation delay
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(TxResponse {
            txhash: tx_hash.to_string(),
            code: 0,
            data: Some("confirmed".to_string()),
            raw_log: "transaction confirmed".to_string(),
            logs: Vec::new(),
            gas_wanted: 200_000,
            gas_used: 150_000,
            height: 1004,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Get transaction by hash
    pub async fn get_transaction(&self, tx_hash: &str) -> Result<TxResponse> {
        // TODO: Implement using the actual valence-domain-clients API
        Ok(TxResponse {
            txhash: tx_hash.to_string(),
            code: 0,
            data: Some("transaction_data".to_string()),
            raw_log: "transaction completed".to_string(),
            logs: Vec::new(),
            gas_wanted: 200_000,
            gas_used: 150_000,
            height: 1000,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Get the latest block height
    pub async fn get_latest_height(&self) -> Result<u64> {
        // TODO: Implement using the actual valence-domain-clients API
        Ok(1000) // Placeholder height
    }
    
    /// Create a ZK message from proof data and authorization context
    pub fn create_zk_message(
        &self,
        circuit_id: String,
        proof_bytes: Vec<u8>,
        verification_key_id: String,
        public_inputs: Vec<ZkInput>,
        auth_context: AuthorizationContext,
    ) -> Result<ZkMessage> {
        let message_id = format!("zk_msg_{:x}", rand::random::<u64>());
        let proof_data = ZkProofData {
            proof_bytes: base64::engine::general_purpose::STANDARD.encode(&proof_bytes),
            encoding: ProofEncoding::Base64,
            verification_key_id,
            metadata: crate::types::ProofMetadata {
                generated_at: chrono::Utc::now().timestamp() as u64,
                generation_time_ms: 0, // Placeholder
                prover_service: Some("valence-coprocessor".to_string()),
                constraints_satisfied: true,
                extra: HashMap::new(),
            },
        };
        
        Ok(ZkMessage {
            id: message_id,
            circuit_id,
            proof: proof_data,
            public_inputs,
            auth_context,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature: None, // TODO: Add signature generation
        })
    }
    
    /// Submit a ZK message to a Valence authorization contract
    pub async fn submit_zk_message_to_contract(
        &self,
        request: ZkMessageSubmissionRequest,
    ) -> Result<ZkMessageSubmissionResponse> {
        log::info!("Submitting ZK message {} to contract {}", 
                  request.message.id, request.contract_address);
        
        // Prepare the contract execution message
        let contract_msg = serde_json::json!({
            "submit_zk_proof": {
                "message_id": request.message.id,
                "circuit_id": request.message.circuit_id,
                "proof_data": request.message.proof.proof_bytes,
                "public_inputs": request.message.public_inputs,
                "auth_context": request.message.auth_context,
                "timestamp": request.message.timestamp
            }
        });
        
        let execute_msg = ContractExecuteMsg {
            contract_address: request.contract_address.clone(),
            msg: contract_msg,
            funds: Vec::new(), // No funds needed for ZK proof submission
        };
        
        // Execute the contract
        let execution_result = self.execute_contract(execute_msg).await?;
        
        // Wait for confirmation if requested
        let (status, confirmation) = if request.wait_for_confirmation {
            let confirmed_tx = self.wait_for_tx_confirmation(&execution_result.tx_hash).await?;
            let confirmation = crate::types::TransactionConfirmation {
                confirmed_at: chrono::Utc::now().timestamp() as u64,
                confirmations: 1,
                block_hash: format!("block_hash_{}", confirmed_tx.height),
                tx_index: 0,
            };
            (TransactionStatus::Confirmed, Some(confirmation))
        } else {
            (TransactionStatus::Pending, None)
        };
        
        Ok(ZkMessageSubmissionResponse {
            tx_hash: execution_result.tx_hash,
            status,
            block_height: Some(execution_result.height),
            gas_used: execution_result.gas_used,
            submitted_at: chrono::Utc::now().timestamp() as u64,
            confirmation,
        })
    }
    
    /// Submit multiple ZK messages in batch
    pub async fn submit_batch_zk_messages(
        &self,
        request: BatchZkMessageSubmissionRequest,
    ) -> Result<BatchZkMessageSubmissionResponse> {
        log::info!("Submitting batch of {} ZK messages to contract {}", 
                  request.messages.len(), request.contract_address);
        
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;
        let mut total_gas_used = 0;
        
        // Determine parallelism level
        let max_parallel = request.max_parallel.unwrap_or(5) as usize;
        let messages = request.messages;
        
        // Process messages in chunks for parallel execution
        for chunk in messages.chunks(max_parallel) {
            let mut chunk_futures = Vec::new();
            
            for message in chunk {
                let submission_request = ZkMessageSubmissionRequest {
                    message: message.clone(),
                    contract_address: request.contract_address.clone(),
                    gas_config: request.gas_config.clone(),
                    memo: request.memo.clone(),
                    wait_for_confirmation: request.wait_for_confirmations,
                };
                
                let future = self.submit_zk_message_to_contract(submission_request);
                chunk_futures.push((message.id.clone(), future));
            }
            
            // Wait for all futures in this chunk to complete
            for (message_id, future) in chunk_futures {
                match future.await {
                    Ok(response) => {
                        total_gas_used += response.gas_used;
                        successful_count += 1;
                        results.push(ZkMessageSubmissionResult {
                            message_id,
                            response: Some(response),
                            error: None,
                            status: SubmissionResultStatus::Success,
                        });
                    }
                    Err(error) => {
                        failed_count += 1;
                        results.push(ZkMessageSubmissionResult {
                            message_id,
                            response: None,
                            error: Some(error.to_string()),
                            status: SubmissionResultStatus::Failed,
                        });
                    }
                }
            }
        }
        
        // Determine batch status
        let batch_status = if failed_count == 0 {
            BatchStatus::AllSuccessful
        } else if successful_count > 0 {
            BatchStatus::PartialSuccess
        } else {
            BatchStatus::AllFailed
        };
        
        Ok(BatchZkMessageSubmissionResponse {
            results,
            batch_status,
            total_gas_used,
            submitted_at: chrono::Utc::now().timestamp() as u64,
            successful_count,
            failed_count,
        })
    }
    
    /// Create authorization context for a ZK message
    pub fn create_authorization_context(
        &self,
        authorizer: String,
        target_contract: String,
        action: AuthorizationAction,
        amount: Option<crate::types::AuthorizedAmount>,
        expires_at: Option<u64>,
    ) -> AuthorizationContext {
        AuthorizationContext {
            authorizer,
            target_contract,
            action,
            amount,
            expires_at,
            nonce: rand::random::<u64>(),
            context_data: HashMap::new(),
        }
    }
    
    /// Create ZK input from various value types
    pub fn create_zk_input(
        name: String,
        value: ZkInputValue,
        input_type: ZkInputType,
    ) -> ZkInput {
        ZkInput {
            name,
            value,
            input_type,
        }
    }
    
    /// Encode proof data with different encoding formats
    pub fn encode_proof_data(
        proof_bytes: &[u8],
        encoding: ProofEncoding,
    ) -> String {
        match encoding {
            ProofEncoding::Base64 => base64::engine::general_purpose::STANDARD.encode(proof_bytes),
            ProofEncoding::Hex => hex::encode(proof_bytes),
            ProofEncoding::Binary => {
                // For binary, we'll use base64 as fallback since JSON doesn't support raw bytes
                base64::engine::general_purpose::STANDARD.encode(proof_bytes)
            }
        }
    }
    
    /// Decode proof data from encoded string
    pub fn decode_proof_data(
        encoded_data: &str,
        encoding: ProofEncoding,
    ) -> Result<Vec<u8>> {
        match encoding {
            ProofEncoding::Base64 => {
                base64::engine::general_purpose::STANDARD.decode(encoded_data)
                    .map_err(|e| anyhow::anyhow!("Failed to decode base64: {}", e))
            }
            ProofEncoding::Hex => {
                hex::decode(encoded_data)
                    .map_err(|e| anyhow::anyhow!("Failed to decode hex: {}", e))
            }
            ProofEncoding::Binary => {
                // For binary, we'll try base64 first
                base64::engine::general_purpose::STANDARD.decode(encoded_data)
                    .map_err(|e| anyhow::anyhow!("Failed to decode binary as base64: {}", e))
            }
        }
    }
    
    /// Submit a ZK message to an authorization contract
    /// This is a placeholder - actual implementation depends on the valence-domain-clients API
    pub async fn submit_zk_message(
        &self,
        contract_address: &str,
        message: &[u8],
    ) -> Result<String> {
        // TODO: Implement using the actual valence-domain-clients API
        // For now, return a placeholder transaction hash
        Ok(format!("tx_hash_for_contract_{}_message_len_{}", 
                  contract_address, message.len()))
    }
    
    /// Build a transaction with multiple messages
    pub async fn build_transaction(
        &self,
        messages: Vec<serde_json::Value>,
        memo: Option<String>,
    ) -> Result<String> {
        // TODO: Implement transaction building using valence-domain-clients
        // For now, return a placeholder transaction
        let tx_data = serde_json::json!({
            "messages": messages,
            "memo": memo.unwrap_or_default(),
            "gas_limit": self.gas_config.max_gas,
            "gas_price": self.gas_config.gas_price,
            "chain_id": self.chain_id
        });
        
        Ok(tx_data.to_string())
    }
    
    /// Sign and broadcast a transaction
    pub async fn sign_and_broadcast_tx(&self, _tx_data: &str) -> Result<TxResponse> {
        // TODO: Implement transaction signing and broadcasting
        let tx_hash = format!("signed_tx_{:x}", rand::random::<u64>());
        
        log::info!("Signed and broadcast transaction: {}", tx_hash);
        
        Ok(TxResponse {
            txhash: tx_hash,
            code: 0,
            data: Some("broadcast_successful".to_string()),
            raw_log: "transaction broadcast successfully".to_string(),
            logs: Vec::new(),
            gas_wanted: self.gas_config.max_gas,
            gas_used: (self.gas_config.max_gas as f64 * 0.7) as u64,
            height: 1005,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

/// Wrapper around CoprocessorClient with Causality-specific functionality
#[derive(Debug, Clone)]
pub struct CoprocessorClientWrapper {
    /// Underlying coprocessor client
    client: Arc<CoprocessorClient>,
    
    /// Coprocessor service URL
    service_url: String,
}

impl CoprocessorClientWrapper {
    /// Create a new coprocessor client wrapper
    pub fn new(service_url: impl Into<String>) -> Result<Self> {
        let service_url = service_url.into();
        let client = CoprocessorClient::new();
        
        Ok(Self {
            client: Arc::new(client),
            service_url,
        })
    }
    
    /// Get the underlying coprocessor client
    pub fn client(&self) -> Arc<CoprocessorClient> {
        Arc::clone(&self.client)
    }
    
    /// Get the service URL
    pub fn service_url(&self) -> &str {
        &self.service_url
    }
    
    /// Submit a proof generation request
    /// This is a placeholder - actual implementation depends on the valence-coprocessor-client API
    pub async fn submit_proof_request(
        &self,
        circuit_name: &str,
        witnesses: &[u8],
        _public_inputs: &[u8],
    ) -> Result<String> {
        // TODO: Implement using the actual valence-coprocessor-client API
        // For now, return a placeholder proof ID
        Ok(format!("proof_id_{}_{}_bytes", circuit_name, witnesses.len()))
    }
    
    /// Get proof status and result
    /// This is a placeholder - actual implementation depends on the valence-coprocessor-client API
    pub async fn get_proof_result(&self, proof_id: &str) -> Result<serde_json::Value> {
        // TODO: Implement using the actual valence-coprocessor-client API
        // For now, return a placeholder
        Ok(serde_json::json!({
            "proof_id": proof_id,
            "status": "completed",
            "proof": "placeholder_proof_data"
        }))
    }
    
    /// Wait for proof completion with polling
    pub async fn wait_for_proof(
        &self,
        proof_id: &str,
        timeout_seconds: u64,
    ) -> Result<serde_json::Value> {
        use tokio::time::{sleep, Duration, timeout};
        
        let timeout_duration = Duration::from_secs(timeout_seconds);
        
        timeout(timeout_duration, async {
            loop {
                let result = self.get_proof_result(proof_id).await?;
                
                if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
                    match status {
                        "completed" => return Ok(result),
                        "failed" => {
                            let error = result.get("error")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error");
                            return Err(anyhow::anyhow!("Proof generation failed: {}", error));
                        }
                        "pending" | "running" => {
                            // Continue polling
                            sleep(Duration::from_secs(2)).await;
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unknown proof status: {}", status));
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Missing status in proof result"));
                }
            }
        }).await?
    }
}

//-----------------------------------------------------------------------------
// Authorization Contract Helpers
//-----------------------------------------------------------------------------

/// Valence authorization contract interface
#[derive(Debug, Clone)]
pub struct ValenceAuthorizationContract {
    /// Neutron client for contract interactions
    client: NeutronClientWrapper,
    
    /// Contract address
    contract_address: String,
    
    /// Contract ABI information
    abi: AuthorizationContractAbi,
}

/// Authorization contract ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationContractAbi {
    /// Contract type (authorization, delegation, etc.)
    pub contract_type: AuthorizationContractType,
    
    /// Contract version
    pub version: String,
    
    /// Supported message types
    pub message_types: Vec<ContractMessageType>,
    
    /// Required permissions
    pub required_permissions: Vec<String>,
}

/// Types of authorization contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationContractType {
    /// Basic authorization contract
    BasicAuthorization,
    /// Multi-signature authorization
    MultiSigAuthorization,
    /// Time-locked authorization
    TimeLockedAuthorization,
    /// Threshold authorization
    ThresholdAuthorization,
    /// Custom authorization logic
    CustomAuthorization(String),
}

/// Contract message types for authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractMessageType {
    /// Submit ZK proof for authorization
    SubmitZkProof,
    /// Grant authorization
    GrantAuthorization,
    /// Revoke authorization
    RevokeAuthorization,
    /// Update authorization
    UpdateAuthorization,
    /// Query authorization status
    QueryAuthorization,
}

/// Authorization contract deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationContractConfig {
    /// Contract label
    pub label: String,
    
    /// Admin address
    pub admin: Option<String>,
    
    /// Initial authorized addresses
    pub initial_authorized: Vec<String>,
    
    /// Authorization threshold (for threshold contracts)
    pub threshold: Option<u64>,
    
    /// Time lock duration in seconds (for time-locked contracts)
    pub time_lock_duration: Option<u64>,
    
    /// Contract-specific configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

/// Authorization status query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationStatus {
    /// Whether the address is authorized
    pub is_authorized: bool,
    
    /// Authorization level (if applicable)
    pub level: Option<u64>,
    
    /// Authorization expiry (if applicable)
    pub expires_at: Option<u64>,
    
    /// Authorization metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authorization grant request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationGrantRequest {
    /// Address to grant authorization to
    pub grantee: String,
    
    /// Authorization level
    pub level: Option<u64>,
    
    /// Authorization expiry
    pub expires_at: Option<u64>,
    
    /// Additional permissions
    pub permissions: Vec<String>,
    
    /// Grant metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authorization revoke request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRevokeRequest {
    /// Address to revoke authorization from
    pub revokee: String,
    
    /// Specific permissions to revoke (if empty, revoke all)
    pub permissions: Vec<String>,
    
    /// Revocation reason
    pub reason: Option<String>,
}

impl ValenceAuthorizationContract {
    /// Create a new authorization contract instance
    pub fn new(
        client: NeutronClientWrapper,
        contract_address: String,
        abi: AuthorizationContractAbi,
    ) -> Self {
        Self {
            client,
            contract_address,
            abi,
        }
    }
    
    /// Deploy a new authorization contract
    pub async fn deploy(
        client: NeutronClientWrapper,
        code_id: u64,
        config: AuthorizationContractConfig,
    ) -> Result<Self> {
        // Prepare instantiation message based on contract type
        let init_msg = Self::prepare_init_message(&config)?;
        
        let instantiate_msg = ContractInstantiateMsg {
            code_id,
            admin: config.admin.clone(),
            label: config.label.clone(),
            msg: init_msg,
            funds: Vec::new(),
        };
        
        // Deploy the contract
        let deployment_result = client.instantiate_contract(instantiate_msg).await?;
        
        log::info!("Deployed authorization contract at address: {}", 
                  deployment_result.contract_address);
        
        // Create ABI for the deployed contract
        let abi = AuthorizationContractAbi {
            contract_type: Self::infer_contract_type(&config),
            version: "1.0".to_string(),
            message_types: vec![
                ContractMessageType::SubmitZkProof,
                ContractMessageType::GrantAuthorization,
                ContractMessageType::RevokeAuthorization,
                ContractMessageType::UpdateAuthorization,
                ContractMessageType::QueryAuthorization,
            ],
            required_permissions: vec![
                "admin".to_string(),
                "authorizer".to_string(),
            ],
        };
        
        Ok(Self::new(client, deployment_result.contract_address, abi))
    }
    
    /// Submit a ZK proof to the authorization contract
    pub async fn submit_zk_proof(&self, zk_message: ZkMessage) -> Result<TxResponse> {
        let contract_msg = serde_json::json!({
            "submit_zk_proof": {
                "message_id": zk_message.id,
                "circuit_id": zk_message.circuit_id,
                "proof_data": zk_message.proof.proof_bytes,
                "public_inputs": zk_message.public_inputs,
                "auth_context": zk_message.auth_context,
                "timestamp": zk_message.timestamp,
                "signature": zk_message.signature
            }
        });
        
        let execute_msg = ContractExecuteMsg {
            contract_address: self.contract_address.clone(),
            msg: contract_msg,
            funds: Vec::new(),
        };
        
        let result = self.client.execute_contract(execute_msg).await?;
        
        // Convert to TxResponse format
        Ok(TxResponse {
            txhash: result.tx_hash,
            code: 0,
            data: result.data,
            raw_log: "ZK proof submitted successfully".to_string(),
            logs: result.logs,
            gas_wanted: result.gas_used + 50_000, // Add some buffer
            gas_used: result.gas_used,
            height: result.height,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Grant authorization to an address
    pub async fn grant_authorization(&self, request: AuthorizationGrantRequest) -> Result<TxResponse> {
        let contract_msg = serde_json::json!({
            "grant_authorization": {
                "grantee": request.grantee,
                "level": request.level,
                "expires_at": request.expires_at,
                "permissions": request.permissions,
                "metadata": request.metadata
            }
        });
        
        let execute_msg = ContractExecuteMsg {
            contract_address: self.contract_address.clone(),
            msg: contract_msg,
            funds: Vec::new(),
        };
        
        let result = self.client.execute_contract(execute_msg).await?;
        
        Ok(TxResponse {
            txhash: result.tx_hash,
            code: 0,
            data: result.data,
            raw_log: format!("Authorization granted to {}", request.grantee),
            logs: result.logs,
            gas_wanted: result.gas_used + 50_000,
            gas_used: result.gas_used,
            height: result.height,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Revoke authorization from an address
    pub async fn revoke_authorization(&self, request: AuthorizationRevokeRequest) -> Result<TxResponse> {
        let contract_msg = serde_json::json!({
            "revoke_authorization": {
                "revokee": request.revokee,
                "permissions": request.permissions,
                "reason": request.reason
            }
        });
        
        let execute_msg = ContractExecuteMsg {
            contract_address: self.contract_address.clone(),
            msg: contract_msg,
            funds: Vec::new(),
        };
        
        let result = self.client.execute_contract(execute_msg).await?;
        
        Ok(TxResponse {
            txhash: result.tx_hash,
            code: 0,
            data: result.data,
            raw_log: format!("Authorization revoked from {}", request.revokee),
            logs: result.logs,
            gas_wanted: result.gas_used + 50_000,
            gas_used: result.gas_used,
            height: result.height,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Query authorization status for an address
    pub async fn query_authorization_status(&self, address: &str) -> Result<AuthorizationStatus> {
        let query_msg = serde_json::json!({
            "get_authorization": {
                "address": address
            }
        });
        
        let query_request = ContractQueryMsg {
            contract_address: self.contract_address.clone(),
            msg: query_msg,
        };
        
        let response = self.client.query_contract(query_request).await?;
        
        // Parse the response into AuthorizationStatus
        let status = AuthorizationStatus {
            is_authorized: response
                .get("is_authorized")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            level: response
                .get("level")
                .and_then(|v| v.as_u64()),
            expires_at: response
                .get("expires_at")
                .and_then(|v| v.as_u64()),
            metadata: response
                .get("metadata")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        };
        
        Ok(status)
    }
    
    /// Get contract information
    pub async fn get_contract_info(&self) -> Result<serde_json::Value> {
        let query_msg = serde_json::json!({
            "get_contract_info": {}
        });
        
        let query_request = ContractQueryMsg {
            contract_address: self.contract_address.clone(),
            msg: query_msg,
        };
        
        self.client.query_contract(query_request).await
    }
    
    /// List all authorized addresses
    pub async fn list_authorized_addresses(&self, limit: Option<u32>) -> Result<Vec<String>> {
        let query_msg = serde_json::json!({
            "list_authorized": {
                "limit": limit.unwrap_or(100)
            }
        });
        
        let query_request = ContractQueryMsg {
            contract_address: self.contract_address.clone(),
            msg: query_msg,
        };
        
        let response = self.client.query_contract(query_request).await?;
        
        let addresses = response
            .get("authorized_addresses")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(addresses)
    }
    
    /// Update contract configuration
    pub async fn update_config(&self, new_config: HashMap<String, serde_json::Value>) -> Result<TxResponse> {
        let contract_msg = serde_json::json!({
            "update_config": new_config
        });
        
        let execute_msg = ContractExecuteMsg {
            contract_address: self.contract_address.clone(),
            msg: contract_msg,
            funds: Vec::new(),
        };
        
        let result = self.client.execute_contract(execute_msg).await?;
        
        Ok(TxResponse {
            txhash: result.tx_hash,
            code: 0,
            data: result.data,
            raw_log: "Contract configuration updated".to_string(),
            logs: result.logs,
            gas_wanted: result.gas_used + 50_000,
            gas_used: result.gas_used,
            height: result.height,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
    
    /// Get the contract ABI
    pub fn abi(&self) -> &AuthorizationContractAbi {
        &self.abi
    }
    
    /// Prepare initialization message for contract deployment
    fn prepare_init_message(config: &AuthorizationContractConfig) -> Result<serde_json::Value> {
        let mut init_msg = serde_json::json!({
            "admin": config.admin,
            "initial_authorized": config.initial_authorized,
            "custom_config": config.custom_config
        });
        
        // Add threshold for threshold contracts
        if let Some(threshold) = config.threshold {
            init_msg["threshold"] = serde_json::Value::Number(threshold.into());
        }
        
        // Add time lock duration for time-locked contracts
        if let Some(duration) = config.time_lock_duration {
            init_msg["time_lock_duration"] = serde_json::Value::Number(duration.into());
        }
        
        Ok(init_msg)
    }
    
    /// Infer contract type from configuration
    fn infer_contract_type(config: &AuthorizationContractConfig) -> AuthorizationContractType {
        if config.threshold.is_some() {
            AuthorizationContractType::ThresholdAuthorization
        } else if config.time_lock_duration.is_some() {
            AuthorizationContractType::TimeLockedAuthorization
        } else if config.initial_authorized.len() > 1 {
            AuthorizationContractType::MultiSigAuthorization
        } else {
            AuthorizationContractType::BasicAuthorization
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_client_creation() {
        // Test basic functionality without actual network calls
        // This just tests that the types are correctly defined
        let gas_config = GasConfig::default();
        assert!(gas_config.gas_price > 0);
        assert!(gas_config.gas_adjustment > 0.0);
    }
    
    #[tokio::test]
    async fn test_zk_message_creation() {
        // Skip this test if we can't safely create a mock client
        // In a real implementation, we'd use a proper mock framework
        println!("Skipping test_zk_message_creation due to unsafe mock creation");
        
        // Instead, test the static methods that don't require client initialization
        let public_inputs = [NeutronClientWrapper::create_zk_input(
                "block_hash".to_string(),
                ZkInputValue::Hash([1u8; 32]),
                ZkInputType::BlockHash,
            ),
            NeutronClientWrapper::create_zk_input(
                "tx_hash".to_string(),
                ZkInputValue::Hash([2u8; 32]),
                ZkInputType::TxHash,
            )];
        
        // Test input creation functionality
        assert_eq!(public_inputs.len(), 2);
        assert_eq!(public_inputs[0].name, "block_hash");
        assert_eq!(public_inputs[1].name, "tx_hash");
        assert!(matches!(public_inputs[0].input_type, ZkInputType::BlockHash));
        assert!(matches!(public_inputs[1].input_type, ZkInputType::TxHash));
        
        // Test encoding/decoding functions
        let test_data = b"test proof data";
        let base64_encoded = NeutronClientWrapper::encode_proof_data(test_data, ProofEncoding::Base64);
        let decoded = NeutronClientWrapper::decode_proof_data(&base64_encoded, ProofEncoding::Base64).unwrap();
        assert_eq!(test_data, decoded.as_slice());
    }
    
    #[test]
    fn test_proof_encoding_decoding() {
        let test_data = b"test proof data";
        
        // Test Base64 encoding
        let base64_encoded = NeutronClientWrapper::encode_proof_data(test_data, ProofEncoding::Base64);
        let decoded = NeutronClientWrapper::decode_proof_data(&base64_encoded, ProofEncoding::Base64).unwrap();
        assert_eq!(test_data, decoded.as_slice());
        
        // Test Hex encoding
        let hex_encoded = NeutronClientWrapper::encode_proof_data(test_data, ProofEncoding::Hex);
        let decoded = NeutronClientWrapper::decode_proof_data(&hex_encoded, ProofEncoding::Hex).unwrap();
        assert_eq!(test_data, decoded.as_slice());
        
        // Test Binary encoding (fallback to Base64)
        let binary_encoded = NeutronClientWrapper::encode_proof_data(test_data, ProofEncoding::Binary);
        let decoded = NeutronClientWrapper::decode_proof_data(&binary_encoded, ProofEncoding::Binary).unwrap();
        assert_eq!(test_data, decoded.as_slice());
    }
    
    #[test]
    fn test_authorization_contract_abi() {
        let abi = AuthorizationContractAbi {
            contract_type: AuthorizationContractType::BasicAuthorization,
            version: "1.0".to_string(),
            message_types: vec![
                ContractMessageType::SubmitZkProof,
                ContractMessageType::GrantAuthorization,
            ],
            required_permissions: vec!["admin".to_string()],
        };
        
        assert_eq!(abi.version, "1.0");
        assert_eq!(abi.message_types.len(), 2);
        assert!(matches!(abi.contract_type, AuthorizationContractType::BasicAuthorization));
    }
    
    #[test]
    fn test_authorization_contract_config() {
        let config = AuthorizationContractConfig {
            label: "test-auth-contract".to_string(),
            admin: Some("neutron1admin".to_string()),
            initial_authorized: vec!["neutron1user1".to_string(), "neutron1user2".to_string()],
            threshold: Some(2),
            time_lock_duration: None,
            custom_config: HashMap::new(),
        };
        
        // Test contract type inference
        let contract_type = ValenceAuthorizationContract::infer_contract_type(&config);
        assert!(matches!(contract_type, AuthorizationContractType::ThresholdAuthorization));
        
        // Test init message preparation
        let init_msg = ValenceAuthorizationContract::prepare_init_message(&config).unwrap();
        assert!(init_msg["threshold"].as_u64().unwrap() == 2);
        assert!(init_msg["initial_authorized"].as_array().unwrap().len() == 2);
    }
    
    #[test]
    fn test_authorization_grant_request() {
        let mut metadata = HashMap::new();
        metadata.insert("grant_reason".to_string(), serde_json::Value::String("test grant".to_string()));
        
        let grant_request = AuthorizationGrantRequest {
            grantee: "neutron1newuser".to_string(),
            level: Some(1),
            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 7200), // 2 hours
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata,
        };
        
        assert_eq!(grant_request.grantee, "neutron1newuser");
        assert_eq!(grant_request.level, Some(1));
        assert_eq!(grant_request.permissions.len(), 2);
        assert!(grant_request.metadata.contains_key("grant_reason"));
    }
    
    #[test]
    fn test_authorization_revoke_request() {
        let revoke_request = AuthorizationRevokeRequest {
            revokee: "neutron1baduser".to_string(),
            permissions: vec!["write".to_string()],
            reason: Some("violated terms".to_string()),
        };
        
        assert_eq!(revoke_request.revokee, "neutron1baduser");
        assert_eq!(revoke_request.permissions.len(), 1);
        assert_eq!(revoke_request.reason, Some("violated terms".to_string()));
    }
    
    #[test]
    fn test_batch_submission_request() {
        let zk_message1 = ZkMessage {
            id: "msg1".to_string(),
            circuit_id: "circuit1".to_string(),
            proof: ZkProofData {
                proof_bytes: "proof1".to_string(),
                encoding: ProofEncoding::Base64,
                verification_key_id: "vk1".to_string(),
                metadata: crate::types::ProofMetadata {
                    generated_at: 1000,
                    generation_time_ms: 500,
                    prover_service: None,
                    constraints_satisfied: true,
                    extra: HashMap::new(),
                },
            },
            public_inputs: Vec::new(),
            auth_context: AuthorizationContext {
                authorizer: "auth1".to_string(),
                target_contract: "contract1".to_string(),
                action: AuthorizationAction::Transfer,
                amount: None,
                expires_at: None,
                nonce: 1,
                context_data: HashMap::new(),
            },
            timestamp: 1000,
            signature: None,
        };
        
        let zk_message2 = ZkMessage {
            id: "msg2".to_string(),
            ..zk_message1.clone()
        };
        
        let batch_request = BatchZkMessageSubmissionRequest {
            messages: vec![zk_message1, zk_message2],
            contract_address: "neutron1contract".to_string(),
            gas_config: Some(GasConfiguration {
                gas_limit: 1000000,
                gas_price: 1000,
                gas_adjustment: 1.2,
            }),
            memo: Some("batch submission test".to_string()),
            wait_for_confirmations: true,
            max_parallel: Some(3),
        };
        
        assert_eq!(batch_request.messages.len(), 2);
        assert_eq!(batch_request.max_parallel, Some(3));
        assert!(batch_request.wait_for_confirmations);
    }
    
    #[test]
    fn test_zk_input_types() {
        // Test different ZK input value types
        let field_input = ZkInput {
            name: "field_test".to_string(),
            value: ZkInputValue::Field("12345".to_string()),
            input_type: ZkInputType::Custom("field".to_string()),
        };
        
        let bool_input = ZkInput {
            name: "bool_test".to_string(),
            value: ZkInputValue::Bool(true),
            input_type: ZkInputType::Custom("boolean".to_string()),
        };
        
        let hash_input = ZkInput {
            name: "hash_test".to_string(),
            value: ZkInputValue::Hash([0xff; 32]),
            input_type: ZkInputType::BlockHash,
        };
        
        assert_eq!(field_input.name, "field_test");
        assert!(matches!(field_input.value, ZkInputValue::Field(_)));
        assert!(matches!(bool_input.value, ZkInputValue::Bool(true)));
        assert!(matches!(hash_input.value, ZkInputValue::Hash(_)));
    }
} 