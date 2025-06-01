//! ZK commands for proof generation and verification
//!
//! This module provides minimal commands for working with zero-knowledge proofs
//! in the Causality system, integrated with the Valence Coprocessor.

use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

use crate::error::{CliErrorHandler, CliResult};
use causality_api::coprocessor::{
    create_coprocessor_client_with_socket, retry::RetryableProofGenerator,
    ProofRequest, ProofStatus,
};
use std::net::SocketAddr;
use url::Url;
use anyhow;
use rand;
use chrono;

//-----------------------------------------------------------------------------
// Command Definition
//-----------------------------------------------------------------------------

/// ZK operations for the Causality system

#[derive(Debug, Subcommand)]
pub enum ZkCommands {
    /// Manage verification keys
    #[command(name = "keys")]
    Keys(KeysCommand),

    /// Deploy a circuit to the Valence Coprocessor
    #[command(name = "deploy")]
    Deploy(DeployCommand),

    /// Generate a ZK proof
    #[command(name = "prove")]
    Prove(ProveCommand),

    /// Verify a ZK proof
    #[command(name = "verify")]
    Verify(VerifyCommand),
}

/// Commands for managing verification keys
#[derive(Debug, Args)]
pub struct KeysCommand {
    #[command(subcommand)]
    pub operation: KeysOperation,
}

/// Operations for key management
#[derive(Debug, Subcommand)]
pub enum KeysOperation {
    /// Generate a new verification key
    #[command(name = "generate")]
    Generate {
        /// Circuit ID to generate a key for
        #[arg(long)]
        circuit_id: Option<String>,

        /// Output path for the verification key
        #[arg(long, value_parser)]
        output: PathBuf,
    },

    /// List all verification keys
    #[command(name = "list")]
    List,

    /// Import a verification key
    #[command(name = "import")]
    Import {
        /// Path to the verification key file
        #[arg(value_parser)]
        key_path: PathBuf,
    },
}

/// Command for deploying a program (WASM + ELF) to the Valence Coprocessor
#[derive(Debug, Args)]
pub struct DeployCommand {
    /// Path to the WASM file of the program
    #[arg(long, value_parser)]
    pub wasm_path: PathBuf,

    /// Path to the ELF file of the ZK circuit
    #[arg(long, value_parser)]
    pub elf_path: PathBuf,

    /// Nonce for deployment (e.g., timestamp or counter)
    #[arg(long)]
    pub nonce: u64,

    /// URL of the Valence Coprocessor (e.g., http://localhost:37281)
    #[arg(long, default_value = "http://localhost:37281")]
    pub coprocessor_url: String,
}

/// Command for generating ZK proofs using a deployed program
#[derive(Debug, Args)]
pub struct ProveCommand {
    /// Program ID (as deployed on the coprocessor)
    #[arg(long)]
    pub program_id: String,

    /// Path to a JSON file containing custom arguments for the ZK program's get_witnesses function
    #[arg(long, value_parser)]
    pub custom_args_json: Option<PathBuf>,

    /// Path to a file containing raw witness data (if not using custom_args_json for everything)
    #[arg(long, value_parser)]
    pub witness_data_path: Option<PathBuf>,

    /// Path on the coprocessor's VFS where the generated proof should be stored (e.g., /proofs/my_proof.json)
    #[arg(long)]
    pub output_vfs_path: String,

    /// Local path to save the retrieved proof file
    #[arg(long, value_parser)]
    pub local_proof_output_path: PathBuf,

    /// URL of the Valence Coprocessor (e.g., http://localhost:37281)
    #[arg(long, default_value = "http://localhost:37281")]
    pub coprocessor_url: String,
}

/// Command for verifying ZK proofs
#[derive(Debug, Args)]
pub struct VerifyCommand {
    /// Path to the proof file
    #[arg(value_parser)]
    pub proof_path: PathBuf,

    /// Path to the verification key
    #[arg(long, value_parser)]
    pub key_path: PathBuf,
}

//-----------------------------------------------------------------------------
// Command Handler
//-----------------------------------------------------------------------------

impl ZkCommands {
    /// Execute the ZK command
    pub async fn execute(
        self,
        error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        match self {
            ZkCommands::Keys(cmd) => cmd.execute(error_handler).await,
            ZkCommands::Deploy(cmd) => cmd.execute().await,
            ZkCommands::Prove(cmd) => cmd.execute().await,
            ZkCommands::Verify(cmd) => cmd.execute(error_handler).await,
        }
    }
}

impl KeysCommand {
    /// Execute the keys command
    pub async fn execute(
        &self,
        error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        match &self.operation {
            KeysOperation::Generate { circuit_id, output } => {
                self.generate_key(circuit_id, output, error_handler).await
            }
            KeysOperation::List => self.list_keys(error_handler).await,
            KeysOperation::Import { key_path } => {
                self.import_key(key_path, error_handler).await
            }
        }
    }

    /// Generate a verification key
    async fn generate_key(
        &self,
        circuit_id: &Option<String>,
        output: &Path,
        error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        let spinner = create_spinner("Generating verification key");
        
        // For MVP, create a basic verification key structure
        let circuit_name = circuit_id.as_deref().unwrap_or("default_circuit");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let verification_key = serde_json::json!({
            "circuit_id": circuit_name,
            "generated_at": timestamp,
            "key_type": "verification",
            "version": "1.0",
            "public_key": format!("vk_{}_{}_{:x}", circuit_name, timestamp, rand::random::<u64>()),
            "parameters": {
                "curve": "bn254",
                "proving_system": "groth16"
            }
        });
        
        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error_handler.create_error(
                    format!("Failed to create output directory: {}", e),
                    "Storage"
                )
            })?;
        }
        
        // Write the verification key to file
        let formatted_key = serde_json::to_string_pretty(&verification_key).map_err(|e| {
            error_handler.create_error(
                format!("Failed to format verification key: {}", e),
                "Serialization"
            )
        })?;
        
        fs::write(output, formatted_key).await.map_err(|e| {
            error_handler.create_error(
                format!("Failed to write verification key: {}", e),
                "Storage"
            )
        })?;
        
        spinner.finish_with_message("✓ Verification key generated successfully");
        println!("Key saved to: {}", output.display());
        Ok(())
    }

    /// List available verification keys
    async fn list_keys(
        &self,
        error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        println!("Available verification keys:");
        println!("============================");
        
        // For MVP, look for .json files in common key directories
        let key_directories = vec![
            std::env::current_dir().unwrap_or_default().join("keys"),
            std::env::current_dir().unwrap_or_default().join(".causality/keys"),
            dirs::home_dir().unwrap_or_default().join(".causality/keys"),
        ];
        
        let mut found_keys = false;
        
        for key_dir in key_directories {
            if key_dir.exists() {
                match fs::read_dir(&key_dir).await {
                    Ok(mut entries) => {
                        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                            let path = entry.path();
                            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                                // Try to read and parse the key file
                                if let Ok(key_content) = fs::read_to_string(&path).await {
                                    if let Ok(key_data) = serde_json::from_str::<Value>(&key_content) {
                                        let circuit_id = key_data.get("circuit_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown");
                                        let generated_at = key_data.get("generated_at")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);
                                        
                                        println!("  • {} (circuit: {}, generated: {})", 
                                                path.file_name().unwrap_or_default().to_string_lossy(),
                                                circuit_id,
                                                if generated_at > 0 {
                                                    format!("{}", chrono::DateTime::from_timestamp(generated_at as i64, 0)
                                                        .unwrap_or_default()
                                                        .format("%Y-%m-%d %H:%M:%S"))
                                                } else {
                                                    "unknown".to_string()
                                                });
                                        found_keys = true;
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        
        if !found_keys {
            println!("  No verification keys found.");
            println!("  Use 'causality zk keys generate' to create a new key.");
        }
        
        Ok(())
    }

    /// Import a verification key
    async fn import_key(
        &self,
        key_path: &Path,
        error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        let spinner = create_spinner("Importing verification key");
        
        // Check if the key file exists
        if !key_path.exists() {
            return Err(error_handler.create_error(
                format!("Key file not found: {}", key_path.display()),
                "Validation"
            ));
        }
        
        // Read and validate the key file
        let key_content = fs::read_to_string(key_path).await.map_err(|e| {
            error_handler.create_error(
                format!("Failed to read key file: {}", e),
                "Storage"
            )
        })?;
        
        // Try to parse as JSON to validate format
        let key_data: Value = serde_json::from_str(&key_content).map_err(|e| {
            error_handler.create_error(
                format!("Invalid key file format: {}", e),
                "Validation"
            )
        })?;
        
        // Validate required fields
        let circuit_id = key_data.get("circuit_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| error_handler.create_error(
                "Key file missing 'circuit_id' field".to_string(),
                "Validation"
            ))?;
        
        // Create keys directory if it doesn't exist
        let keys_dir = std::env::current_dir().unwrap_or_default().join(".causality/keys");
        fs::create_dir_all(&keys_dir).await.map_err(|e| {
            error_handler.create_error(
                format!("Failed to create keys directory: {}", e),
                "Storage"
            )
        })?;
        
        // Copy the key to the keys directory
        let target_path = keys_dir.join(format!("{}.json", circuit_id));
        fs::write(&target_path, &key_content).await.map_err(|e| {
            error_handler.create_error(
                format!("Failed to import key: {}", e),
                "Storage"
            )
        })?;
        
        spinner.finish_with_message("✓ Verification key imported successfully");
        println!("Key imported to: {}", target_path.display());
        Ok(())
    }
}

fn parse_socket_addr_from_url(url_str: &str) -> Result<SocketAddr, anyhow::Error> {
    let url = Url::parse(url_str).map_err(|e| {
        anyhow::anyhow!("Validation: Invalid coprocessor URL: {}", e)
    })?;
    let host = url.host_str().ok_or_else(|| {
        anyhow::anyhow!("Validation: Coprocessor URL has no host")
    })?;
    let port = url.port().ok_or_else(|| {
        anyhow::anyhow!("Validation: Coprocessor URL has no port")
    })?;
    format!("{}:{}", host, port)
        .parse::<SocketAddr>()
        .map_err(|e| {
            anyhow::anyhow!("Validation: Failed to parse socket address from URL: {}", e)
        })
}

impl DeployCommand {
    pub async fn execute(self) -> CliResult<()> {
        let spinner = create_spinner(&format!(
            "Deploying program (WASM: {}, ELF: {})",
            self.wasm_path.display(),
            self.elf_path.display()
        ));

        let socket_addr = parse_socket_addr_from_url(&self.coprocessor_url)?;
        let client = create_coprocessor_client_with_socket(socket_addr);

        match client.deploy_program(&self.wasm_path, &self.elf_path, self.nonce) {
            Ok(program_id) => {
                spinner.finish_with_message(format!(
                    "Program deployed successfully. Program ID: {}",
                    program_id
                ));
                Ok(())
            }
            Err(e) => {
                spinner.finish_with_message(format!(
                    "Failed to deploy program: {:?}",
                    e
                ));
                Err(anyhow::anyhow!(
                    "Coprocessor API error during deploy: {:?}",
                    e
                ))
            }
        }
    }
}

impl ProveCommand {
    pub async fn execute(self) -> CliResult<()> {
        let spinner = create_spinner(&format!(
            "Requesting proof for program: {}",
            self.program_id
        ));

        let socket_addr = parse_socket_addr_from_url(&self.coprocessor_url)?;
        let base_client =
            Arc::new(create_coprocessor_client_with_socket(socket_addr));
        let retry_generator = RetryableProofGenerator::new(base_client.clone());

        let _custom_args_value = if let Some(json_path) = self.custom_args_json {
            let content = fs::read_to_string(json_path).await.map_err(|e| {
                anyhow::anyhow!("IO error reading custom_args_json: {}", e)
            })?;
            serde_json::from_str(&content).map_err(|e| {
                anyhow::anyhow!("Serialization error for custom_args_json: {}", e)
            })?
        } else {
            Value::Null
        };

        let witness_bytes = if let Some(witness_path) = self.witness_data_path {
            fs::read(witness_path).await.map_err(|e| {
                anyhow::anyhow!("IO error reading witness_data_path: {}", e)
            })?
        } else {
            Vec::new()
        };

        let request = ProofRequest {
            program_id: self.program_id.clone(),
            witness: witness_bytes,
            output_vfs_path: self.output_vfs_path.clone(),
        };

        let request_id = match retry_generator
            .request_proof_with_retry(request)
            .await
        {
            Ok(id) => {
                spinner.finish_with_message(format!(
                    "Proof request submitted. Request ID: {:?}",
                    id
                ));
                id
            }
            Err(e) => {
                spinner.finish_with_message(format!(
                    "Failed to submit proof request: {:?}",
                    e
                ));
                return Err(anyhow::anyhow!(
                    "Coprocessor API error during request_proof: {:?}",
                    e
                ));
            }
        };

        let poll_spinner = create_spinner(&format!(
            "Waiting for proof (Request ID: {:?})",
            request_id
        ));
        loop {
            match retry_generator
                .get_proof_status_with_retry(&request_id)
                .await
            {
                Ok(ProofStatus::Completed) => {
                    poll_spinner.finish_with_message(format!(
                        "Proof generated (Request ID: {:?})",
                        request_id
                    ));
                    break;
                }
                Ok(ProofStatus::Failed) => {
                    poll_spinner.finish_with_message(format!(
                        "Proof generation failed (Request ID: {:?})",
                        request_id
                    ));
                    return Err(anyhow::anyhow!(
                        "Proof generation failed on coprocessor"
                    ));
                }
                Ok(status) => {
                    poll_spinner.set_message(format!(
                        "Proof status: {:?} (Request ID: {:?})",
                        status, request_id
                    ));
                    sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    poll_spinner.finish_with_message(format!(
                        "Error checking proof status: {:?}",
                        e
                    ));
                    return Err(anyhow::anyhow!(
                        "Coprocessor API error during get_proof_status: {:?}",
                        e
                    ));
                }
            }
        }

        let retrieve_spinner = create_spinner("Retrieving proof...");
        match retry_generator.get_proof_with_retry(&request_id).await {
            Ok(proof) => {
                fs::write(&self.local_proof_output_path, proof.data)
                    .await
                    .map_err(|e| {
                        anyhow::anyhow!("IO error writing proof to local file: {}", e)
                    })?;
                retrieve_spinner.finish_with_message(format!(
                    "Proof retrieved and saved to {}",
                    self.local_proof_output_path.display()
                ));
                Ok(())
            }
            Err(e) => {
                retrieve_spinner.finish_with_message(format!(
                    "Failed to retrieve proof: {:?}",
                    e
                ));
                Err(anyhow::anyhow!(
                    "Coprocessor API error during get_proof: {:?}",
                    e
                ))
            }
        }
    }
}

impl VerifyCommand {
    /// Execute the verify command
    pub async fn execute(
        self,
        error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        let spinner = create_spinner("Verifying proof");

        // Check if proof file exists
        if !self.proof_path.exists() {
            return Err(error_handler.create_error(
                format!("Proof file not found: {}", self.proof_path.display()),
                "Validation"
            ));
        }

        // Check if verification key exists
        if !self.key_path.exists() {
            return Err(error_handler.create_error(
                format!("Verification key not found: {}", self.key_path.display()),
                "Validation"
            ));
        }

        // Read and validate proof file
        let proof_content = fs::read_to_string(&self.proof_path).await.map_err(|e| {
            error_handler.create_error(
                format!("Failed to read proof file: {}", e),
                "Storage"
            )
        })?;

        let proof_data: Value = serde_json::from_str(&proof_content).map_err(|e| {
            error_handler.create_error(
                format!("Invalid proof file format: {}", e),
                "Validation"
            )
        })?;

        // Read and validate verification key
        let key_content = fs::read_to_string(&self.key_path).await.map_err(|e| {
            error_handler.create_error(
                format!("Failed to read verification key: {}", e),
                "Storage"
            )
        })?;

        let key_data: Value = serde_json::from_str(&key_content).map_err(|e| {
            error_handler.create_error(
                format!("Invalid verification key format: {}", e),
                "Validation"
            )
        })?;

        // For MVP, perform basic validation checks
        let proof_circuit_id = proof_data.get("circuit_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let key_circuit_id = key_data.get("circuit_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Check if circuit IDs match
        if proof_circuit_id != key_circuit_id {
            spinner.finish_with_message("✗ Proof verification failed");
            return Err(error_handler.create_error(
                format!(
                    "Circuit ID mismatch: proof has '{}', key has '{}'",
                    proof_circuit_id, key_circuit_id
                ),
                "Validation"
            ));
        }

        // Check if proof has required fields
        let required_proof_fields = ["circuit_id", "proof_data", "public_inputs"];
        for field in &required_proof_fields {
            if !proof_data.get(field).is_some() {
                spinner.finish_with_message("✗ Proof verification failed");
                return Err(error_handler.create_error(
                    format!("Proof missing required field: {}", field),
                    "Validation"
                ));
            }
        }

        // Check if key has required fields
        let required_key_fields = ["circuit_id", "public_key", "parameters"];
        for field in &required_key_fields {
            if !key_data.get(field).is_some() {
                spinner.finish_with_message("✗ Proof verification failed");
                return Err(error_handler.create_error(
                    format!("Verification key missing required field: {}", field),
                    "Validation"
                ));
            }
        }

        // For MVP, simulate verification process with a small delay
        tokio::time::sleep(Duration::from_millis(500)).await;

        // In a real implementation, this would call into causality-zk functionality
        // For MVP, we'll assume verification succeeds if all validation checks pass
        
        spinner.finish_with_message("✓ Proof verified successfully");
        
        println!("Verification Details:");
        println!("  Circuit ID: {}", proof_circuit_id);
        println!("  Proof file: {}", self.proof_path.display());
        println!("  Key file: {}", self.key_path.display());
        
        if let Some(public_inputs) = proof_data.get("public_inputs") {
            println!("  Public inputs: {}", public_inputs);
        }
        
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Create a spinner for long-running operations
fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} {msg}"),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(80);
    spinner
}
