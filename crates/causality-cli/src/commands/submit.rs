//! Submit transaction command for multi-chain deployment
//!
//! This module implements transaction submission to multiple blockchain networks,
//! supporting both dry-run validation and actual deployment with ZK proof verification.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs;
use causality_api::client::{ChainClient, TransactionResult};
use causality_api::types::{TransactionRequest, ChainConfig, ProofData};

#[derive(Parser, Debug, Clone)]
pub struct SubmitCommand {
    /// ZK circuit proof file
    #[arg(long)]
    pub proof: PathBuf,
    
    /// Target blockchain networks (comma-separated)
    #[arg(long)]
    pub target_chains: String,
    
    /// Run in dry-run mode (validation only)
    #[arg(long)]
    pub dry_run: bool,
    
    /// Gas price in gwei
    #[arg(long)]
    pub gas_price_gwei: Option<u32>,
    
    /// Maximum gas limit
    #[arg(long)]
    pub gas_limit: Option<u64>,
    
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl SubmitCommand {
    pub async fn execute(&self) -> Result<()> {
        if self.verbose {
            println!("🚀 Starting multi-chain transaction submission...");
            println!("   Proof file: {}", self.proof.display());
            println!("   Target chains: {}", self.target_chains);
            println!("   Dry run: {}", self.dry_run);
        }

        // Parse target chains
        let chains: Vec<&str> = self.target_chains.split(',').map(|s| s.trim()).collect();
        
        // Read proof file
        let proof_data = fs::read_to_string(&self.proof)
            .map_err(|e| anyhow::anyhow!("Failed to read proof file {}: {}", self.proof.display(), e))?;

        if self.verbose {
            println!("   Proof loaded ({} bytes)", proof_data.len());
        }

        // Parse proof data
        let proof: ProofData = serde_json::from_str(&proof_data)
            .map_err(|e| anyhow::anyhow!("Failed to parse proof data: {}", e))?;

        // Submit to each chain
        let mut results = Vec::new();
        for chain_name in chains {
            let result = self.submit_to_chain(chain_name, &proof).await?;
            results.push((chain_name, result));
        }

        // Print results
        println!("✅ Multi-chain submission completed");
        for (chain, result) in results {
            match result {
                TransactionResult::Success { tx_hash, gas_used, block_number } => {
                    println!("   {} ✅ Success", chain);
                    if !self.dry_run {
                        println!("      Transaction: {}", tx_hash);
                        println!("      Block: {}", block_number);
                        println!("      Gas used: {}", gas_used);
                    } else {
                        println!("      Validation: PASSED");
                        println!("      Estimated gas: {}", gas_used);
                    }
                }
                TransactionResult::Failure { error, gas_estimate } => {
                    println!("   {} ❌ Failed: {}", chain, error);
                    if let Some(gas) = gas_estimate {
                        println!("      Gas estimate: {}", gas);
                    }
                }
            }
        }

        Ok(())
    }
    
    async fn submit_to_chain(&self, chain_name: &str, proof: &ProofData) -> Result<TransactionResult> {
        if self.verbose {
            println!("📡 Submitting to {} chain...", chain_name);
        }

        // Create chain configuration
        let chain_config = self.get_chain_config(chain_name)?;
        
        // Create chain client
        let client = ChainClient::new(chain_config).await?;
        
        // Create transaction request
        let tx_request = TransactionRequest {
            proof_data: proof.clone(),
            gas_price: self.gas_price_gwei.map(|g| g as u64 * 1_000_000_000), // Convert gwei to wei
            gas_limit: self.gas_limit,
            dry_run: self.dry_run,
        };
        
        // Submit transaction
        let result = if self.dry_run {
            client.validate_transaction(&tx_request).await?
        } else {
            client.submit_transaction(&tx_request).await?
        };
        
        Ok(result)
    }
    
    pub fn get_chain_config(&self, chain_name: &str) -> Result<ChainConfig> {
        let config = match chain_name.to_lowercase().as_str() {
            "ethereum" => ChainConfig {
                name: "ethereum".to_string(),
                chain_id: 1,
                rpc_url: "https://eth-mainnet.g.alchemy.com/v2/demo".to_string(),
                explorer_url: "https://etherscan.io".to_string(),
                gas_price_multiplier: 1.1,
                confirmation_blocks: 12,
            },
            "polygon" => ChainConfig {
                name: "polygon".to_string(),
                chain_id: 137,
                rpc_url: "https://polygon-rpc.com".to_string(),
                explorer_url: "https://polygonscan.com".to_string(),
                gas_price_multiplier: 1.2,
                confirmation_blocks: 20,
            },
            "arbitrum" => ChainConfig {
                name: "arbitrum".to_string(),
                chain_id: 42161,
                rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
                explorer_url: "https://arbiscan.io".to_string(),
                gas_price_multiplier: 1.0,
                confirmation_blocks: 1,
            },
            "optimism" => ChainConfig {
                name: "optimism".to_string(),
                chain_id: 10,
                rpc_url: "https://mainnet.optimism.io".to_string(),
                explorer_url: "https://optimistic.etherscan.io".to_string(),
                gas_price_multiplier: 1.0,
                confirmation_blocks: 1,
            },
            _ => return Err(anyhow::anyhow!("Unsupported chain: {}", chain_name)),
        };
        
        Ok(config)
    }
}
