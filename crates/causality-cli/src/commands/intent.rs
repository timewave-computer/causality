//! Intent Submission and Query Command
//!
//! This module provides CLI commands for interacting with intents on blockchains,
//! supporting both intent submission and querying with mock capabilities.

//-----------------------------------------------------------------------------
// Intent Submission and Query Command
//-----------------------------------------------------------------------------

use causality_types::serialization::Decode;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

use causality_api::chain::intent::IntentHandler;
use causality_api::chain::mock::{MockChainClient, MockIntentHandler};
use causality_api::chain::types::ChainId;
use causality_types::primitive::ids::{IntentId, AsId};
use causality_types::core::Intent;

use crate::error::CliResult;
use crate::error::CliErrorHandler;

// Custom ID utils to replace causality_core::id_utils
mod id_utils {
    
    use anyhow::{Result, anyhow};
    use hex;
    use causality_types::primitive::ids::AsId;

    // Convert an ID to a hex string using the AsId trait
    pub fn id_to_hex<T: AsId>(id: T) -> String {
        hex::encode(id.inner())
    }

    // Create an ID from a hex string - generic version for any 32-byte ID type with AsId
    pub fn id_from_hex<T: AsId>(hex_str: &str) -> Result<T> {
        if hex_str.len() != 64 {
            return Err(anyhow!("Invalid hex string length: {}", hex_str.len()));
        }
        
        let bytes = hex::decode(hex_str)
            .map_err(|e| anyhow!("Failed to decode hex: {}", e))?;
            
        if bytes.len() != 32 {
            return Err(anyhow!("Decoded bytes length is not 32: {}", bytes.len()));
        }
        
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(T::new(arr))
    }
}

//-----------------------------------------------------------------------------
// Command Structure
//-----------------------------------------------------------------------------

/// Commands for interacting with intents on blockchains

#[derive(Parser, Debug)]
pub struct IntentCommand {
    /// Action to perform
    #[clap(subcommand)]
    pub action: IntentAction,
}

/// Subcommands for intent operations
#[derive(Debug, Subcommand)]
pub enum IntentAction {
    /// Submit an intent to a blockchain
    Submit(SubmitIntentArgs),

    /// Query an intent from a blockchain
    Query(QueryIntentArgs),
}

/// Arguments for submitting an intent
#[derive(Debug, Args)]
pub struct SubmitIntentArgs {
    /// Path to the intent file (SSZ-serialized)
    #[arg(long, short = 'f')]
    pub file: PathBuf,

    /// Blockchain to submit to (e.g., "neutron", "ethereum", or "mock")
    #[arg(long, short = 'b', default_value = "mock")]
    pub blockchain: String,
}

/// Arguments for querying an intent
#[derive(Debug, Args)]
pub struct QueryIntentArgs {
    /// Intent ID to query (64-character hex string)
    #[arg(long, short = 'i')]
    pub id: String,

    /// Blockchain to query from (e.g., "neutron", "ethereum", or "mock")
    #[arg(long, short = 'b', default_value = "mock")]
    pub blockchain: String,
}

//-----------------------------------------------------------------------------
// Command Handler
//-----------------------------------------------------------------------------

/// Handle the intent commands

pub async fn handle_intent_command(
    command: IntentAction,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    match command {
        IntentAction::Submit(args) => submit_intent(args, error_handler).await,
        IntentAction::Query(args) => query_intent(args, error_handler).await,
    }
}

/// Submit an intent to a blockchain
async fn submit_intent(
    args: SubmitIntentArgs,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    // Read intent file
    let intent_bytes = tokio::fs::read(&args.file).await.map_err(|e| {
        error_handler.create_error(
            format!("Error reading intent file: {}", e),
            "Storage",
        )
    })?;

    // Deserialize intent
    let intent = Intent::from_ssz_bytes(&intent_bytes).map_err(|e| {
        error_handler.create_error(
            format!("Error deserializing intent from file: {}", e),
            "Serialization",
        )
    })?;

    // Create a client based on the blockchain type
    let client = create_client(&args.blockchain, error_handler.clone()).await?;

    // Submit the intent
    match client.submit_intent(intent).await {
        Ok(intent_id) => {
            println!("Intent submitted successfully!");
            println!("Intent ID: {}", id_utils::id_to_hex(intent_id));
            Ok(())
        }
        Err(api_error) => Err(error_handler.create_error(
            format!("Failed to submit intent: {}", api_error),
            "Network",
        )),
    }
}

/// Query an intent from a blockchain
async fn query_intent(
    args: QueryIntentArgs,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    // Parse intent ID from hex string
    let intent_id = id_utils::id_from_hex::<IntentId>(&args.id).map_err(|e| {
        error_handler.create_error(
            format!(
                "Invalid intent ID format: '{}'. Must be a 64-character hex string. Error: {}",
                args.id, e
            ),
            "Validation",
        )
    })?;

    // Create a client based on the blockchain type
    let client = create_client(&args.blockchain, error_handler.clone()).await?;

    // Query the intent
    match client.query_intent(&intent_id).await {
        Ok(query_result) => {
            if let Some(metadata) = query_result.metadata {
                println!("Intent found!");
                println!("Status: {:?}", query_result.status);
                if let Some(tx_id) = query_result.transaction_id {
                    println!("Transaction ID: {}", tx_id.0);
                }
                println!("Block height: {}", metadata.block_height);
                println!("Timestamp: {}", metadata.timestamp);
                println!("Transaction hash from metadata: {}", metadata.tx_hash);

                if let Some(intent_details) = query_result.intent {
                    println!("\nIntent details:");
                    println!("  Input resources: {}", intent_details.inputs.len());
                    println!("  Output resources: {}", intent_details.outputs.len());
                    // Field references updated to match current Intent structure
                    if let Some(expr_id) = &intent_details.expression {
                        println!("  Expression: {}", expr_id.to_hex());
                    } else {
                        println!("  Expression: None");
                    }
                } else {
                    println!("Intent details not available in query result.");
                }
                Ok(())
            } else {
                println!("Intent not found or metadata missing!");
                println!("Status: {:?}", query_result.status);
                Ok(())
            }
        }
        Err(api_error) => Err(error_handler.create_error(
            format!("Failed to query intent: {}", api_error),
            "Network",
        )),
    }
}

//-----------------------------------------------------------------------------
// Client Factory
//-----------------------------------------------------------------------------

/// Create a blockchain intent handler for the specified blockchain

async fn create_client(
    blockchain: &str,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<Arc<dyn IntentHandler>> {
    match blockchain {
        "mock" | "local" => {
            // For mock, use MockIntentHandler which wraps MockChainClient
            let mock_chain_client =
                Arc::new(MockChainClient::new(ChainId(0), "mock_cli_address"));
            let mock_intent_handler =
                Arc::new(MockIntentHandler::new(mock_chain_client));
            Ok(mock_intent_handler)
        }
        "ethereum" | "solana" => {
            // For real implementations, this would connect to the respective chain
            // and likely wrap a ChainClient with ChainIntentHandler
            Err(error_handler.create_error(
                format!(
                    "Blockchain '{}' support not implemented yet in CLI",
                    blockchain
                ),
                "ResourceNotFound",
            ))
        }
        _ => Err(error_handler.create_error(
            format!("Unsupported blockchain: {}", blockchain),
            "Validation",
        )),
    }
}
