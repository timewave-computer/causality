//! Blockchain client for multi-chain transaction submission
//!
//! This module provides a unified client interface for interacting with multiple
//! blockchain networks, supporting transaction submission, validation, and monitoring.

use anyhow::Result;
use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

use crate::types::*;

//-----------------------------------------------------------------------------
// Local Types for Client Results
//-----------------------------------------------------------------------------

/// Result of a transaction submission or validation
#[derive(Debug, Clone)]
pub enum TransactionResult {
    Success {
        tx_hash: String,
        gas_used: u64,
        block_number: u64,
    },
    Failure {
        error: String,
        gas_estimate: Option<u64>,
    },
}

//-----------------------------------------------------------------------------
// Chain Client Implementation
//-----------------------------------------------------------------------------

/// Client for interacting with blockchain networks
pub struct ChainClient {
    /// Chain configuration
    config: ChainConfig,
    
    /// HTTP client for RPC calls
    http_client: HttpClient,
    
    /// Current nonce for transactions
    nonce: Option<u64>,
}

impl ChainClient {
    /// Create a new chain client
    pub async fn new(config: ChainConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
            
        Ok(Self {
            config,
            http_client,
            nonce: None,
        })
    }
    
    /// Submit a transaction to the blockchain
    pub async fn submit_transaction(&self, request: &TransactionRequest) -> Result<TransactionResult> {
        if request.dry_run {
            return self.validate_transaction(request).await;
        }
        
        // Get current gas price
        let gas_price = match request.gas_price {
            Some(price) => price,
            None => self.get_gas_price().await?,
        };
        
        // Estimate gas limit
        let gas_limit = match request.gas_limit {
            Some(limit) => limit,
            None => self.estimate_gas(&request.proof_data).await?,
        };
        
        // Build transaction
        let tx_data = self.build_transaction_data(&request.proof_data, gas_price, gas_limit).await?;
        
        // Submit transaction
        let tx_hash = self.send_raw_transaction(&tx_data).await?;
        
        // Wait for confirmation
        let receipt = self.wait_for_confirmation(&tx_hash).await?;
        
        Ok(TransactionResult::Success {
            tx_hash,
            gas_used: receipt.gas_used,
            block_number: receipt.block_number,
        })
    }
    
    /// Validate a transaction without submitting it
    pub async fn validate_transaction(&self, request: &TransactionRequest) -> Result<TransactionResult> {
        // Estimate gas for validation
        let gas_estimate = self.estimate_gas(&request.proof_data).await?;
        
        // Validate proof data format
        if let Err(e) = self.validate_proof_format(&request.proof_data) {
            return Ok(TransactionResult::Failure {
                error: format!("Invalid proof format: {}", e),
                gas_estimate: Some(gas_estimate),
            });
        }
        
        // Simulate transaction execution
        match self.simulate_transaction(&request.proof_data).await {
            Ok(_) => Ok(TransactionResult::Success {
                tx_hash: "dry-run".to_string(),
                gas_used: gas_estimate,
                block_number: 0,
            }),
            Err(e) => Ok(TransactionResult::Failure {
                error: format!("Simulation failed: {}", e),
                gas_estimate: Some(gas_estimate),
            }),
        }
    }
    
    /// Get current gas price from the network
    async fn get_gas_price(&self) -> Result<u64> {
        let response = self.rpc_call("eth_gasPrice", json!([])).await?;
        
        let gas_price_hex = response.as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid gas price response"))?;
            
        let gas_price = u64::from_str_radix(&gas_price_hex[2..], 16)?;
        
        // Apply multiplier for faster confirmation
        let adjusted_price = (gas_price as f64 * self.config.gas_price_multiplier) as u64;
        
        Ok(adjusted_price)
    }
    
    /// Estimate gas required for the transaction
    async fn estimate_gas(&self, proof_data: &ProofData) -> Result<u64> {
        // Build transaction for estimation
        let tx_data = json!({
            "to": self.get_contract_address(),
            "data": self.encode_proof_data(proof_data)?,
        });
        
        let response = self.rpc_call("eth_estimateGas", json!([tx_data])).await?;
        
        let gas_hex = response.as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid gas estimate response"))?;
            
        let gas_estimate = u64::from_str_radix(&gas_hex[2..], 16)?;
        
        // Add 20% buffer for safety
        Ok((gas_estimate as f64 * 1.2) as u64)
    }
    
    /// Build transaction data for submission
    async fn build_transaction_data(&self, proof_data: &ProofData, gas_price: u64, gas_limit: u64) -> Result<String> {
        let nonce = self.get_next_nonce().await?;
        
        let tx = json!({
            "nonce": format!("0x{:x}", nonce),
            "gasPrice": format!("0x{:x}", gas_price),
            "gasLimit": format!("0x{:x}", gas_limit),
            "to": self.get_contract_address(),
            "value": "0x0",
            "data": self.encode_proof_data(proof_data)?,
        });
        
        // In a real implementation, this would be signed with a private key
        // For now, we'll return a mock signed transaction
        Ok(format!("0x{}", hex::encode(serde_json::to_vec(&tx)?)))
    }
    
    /// Send raw transaction to the network
    async fn send_raw_transaction(&self, tx_data: &str) -> Result<String> {
        let response = self.rpc_call("eth_sendRawTransaction", json!([tx_data])).await?;
        
        response.as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction hash response"))
            .map(|s| s.to_string())
    }
    
    /// Wait for transaction confirmation
    async fn wait_for_confirmation(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        let start_time = SystemTime::now();
        let timeout = Duration::from_secs(300); // 5 minutes
        
        loop {
            if start_time.elapsed()? > timeout {
                return Err(anyhow::anyhow!("Transaction confirmation timeout"));
            }
            
            match self.get_transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    if receipt.block_number > 0 {
                        return Ok(receipt);
                    }
                }
                Ok(None) => {
                    // Transaction not yet mined
                }
                Err(e) => {
                    eprintln!("Error checking transaction receipt: {}", e);
                }
            }
            
            sleep(Duration::from_secs(2)).await;
        }
    }
    
    /// Get transaction receipt
    async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<Option<TransactionReceipt>> {
        let response = self.rpc_call("eth_getTransactionReceipt", json!([tx_hash])).await?;
        
        if response.is_null() {
            return Ok(None);
        }
        
        let receipt = TransactionReceipt {
            transaction_hash: tx_hash.to_string(),
            block_number: self.parse_hex_u64(response["blockNumber"].as_str().unwrap_or("0x0"))?,
            gas_used: self.parse_hex_u64(response["gasUsed"].as_str().unwrap_or("0x0"))?,
            status: response["status"].as_str().unwrap_or("0x1") == "0x1",
        };
        
        Ok(Some(receipt))
    }
    
    /// Get next nonce for transactions
    async fn get_next_nonce(&self) -> Result<u64> {
        // In a real implementation, this would get the nonce from the account
        // For now, we'll use a simple counter
        match self.nonce {
            Some(n) => Ok(n + 1),
            None => Ok(0),
        }
    }
    
    /// Validate proof data format
    fn validate_proof_format(&self, proof_data: &ProofData) -> Result<()> {
        if proof_data.proof.is_empty() {
            return Err(anyhow::anyhow!("Empty proof data"));
        }
        
        if proof_data.verification_key.is_empty() {
            return Err(anyhow::anyhow!("Missing verification key"));
        }
        
        if proof_data.circuit_id.is_empty() {
            return Err(anyhow::anyhow!("Missing circuit ID"));
        }
        
        Ok(())
    }
    
    /// Simulate transaction execution
    async fn simulate_transaction(&self, proof_data: &ProofData) -> Result<()> {
        let tx_data = json!({
            "to": self.get_contract_address(),
            "data": self.encode_proof_data(proof_data)?,
        });
        
        let response = self.rpc_call("eth_call", json!([tx_data, "latest"])).await?;
        
        if response.as_str().unwrap_or("").starts_with("0x") {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Simulation failed"))
        }
    }
    
    /// Encode proof data for contract call
    fn encode_proof_data(&self, proof_data: &ProofData) -> Result<String> {
        // This would encode the proof data according to the contract ABI
        // For now, we'll create a simple encoding
        let encoded = json!({
            "proof": proof_data.proof,
            "publicInputs": proof_data.public_inputs,
            "verificationKey": proof_data.verification_key,
        });
        
        Ok(format!("0x{}", hex::encode(serde_json::to_vec(&encoded)?)))
    }
    
    /// Get the smart contract address for proof verification
    fn get_contract_address(&self) -> String {
        // This would be configured per chain
        match self.config.chain_id {
            1 => "0x1234567890123456789012345678901234567890".to_string(), // Ethereum
            137 => "0x2345678901234567890123456789012345678901".to_string(), // Polygon
            42161 => "0x3456789012345678901234567890123456789012".to_string(), // Arbitrum
            10 => "0x4567890123456789012345678901234567890123".to_string(), // Optimism
            _ => "0x0000000000000000000000000000000000000000".to_string(),
        }
    }
    
    /// Make RPC call to the blockchain
    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request_body)
            .send()
            .await?;
            
        let response_json: Value = response.json().await?;
        
        if let Some(error) = response_json["error"].as_object() {
            return Err(anyhow::anyhow!("RPC error: {}", error["message"].as_str().unwrap_or("Unknown error")));
        }
        
        Ok(response_json["result"].clone())
    }
    
    /// Parse hexadecimal string to u64
    fn parse_hex_u64(&self, hex_str: &str) -> Result<u64> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        Ok(u64::from_str_radix(hex_str, 16)?)
    }
}

//-----------------------------------------------------------------------------
// Helper Types
//-----------------------------------------------------------------------------

/// Transaction receipt information
#[derive(Debug, Clone)]
struct TransactionReceipt {
    /// Transaction hash
    #[allow(dead_code)]
    transaction_hash: String,
    
    /// Block number where transaction was included
    block_number: u64,
    
    /// Gas used by the transaction
    gas_used: u64,
    
    /// Whether the transaction was successful
    #[allow(dead_code)]
    status: bool,
}
