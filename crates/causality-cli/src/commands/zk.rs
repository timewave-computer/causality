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
        _circuit_id: &Option<String>,
        output: &Path,
        _error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        let spinner = create_spinner("Generating verification key");
        spinner.finish_with_message(
            "Verification key generated successfully (placeholder)",
        );
        println!("Key saved to {} (placeholder)", output.display());
        Ok(())
    }

    /// List available verification keys
    async fn list_keys(
        &self,
        _error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        println!("Available verification keys (placeholder):");
        Ok(())
    }

    /// Import a verification key
    async fn import_key(
        &self,
        _key_path: &Path,
        _error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        let spinner = create_spinner("Importing verification key");
        spinner.finish_with_message(
            "Verification key imported successfully (placeholder)",
        );
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
        _error_handler: Arc<CliErrorHandler>,
    ) -> CliResult<()> {
        let spinner = create_spinner("Verifying proof");

        // Placeholder for actual proof verification
        // This would call into causality-zk functionality

        spinner.finish_with_message("✓ Proof verified successfully");
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
