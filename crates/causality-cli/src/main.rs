//! Causality CLI
//!
//! Command-line interface for the Causality system, supporting compilation,
//! execution, testing, and development workflows with the unified 5-instruction
//! machine architecture.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;

use causality_cli::commands::*;
use causality_cli::error::CliErrorHandler;

/// Causality - A linear type system with unified computation and communication
#[derive(Parser)]
#[command(name = "causality")]
#[command(about = "Causality programming language CLI")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
enum Commands {
    /// Compile Causality source files
    Compile(compile::CompileCommand),
    
    /// Run simulation and cost analysis
    Simulate(simulate::SimulateCommand),
    
    /// Generate and verify zero-knowledge proofs
    Prove(zk::ProveCommand),
    
    /// Submit transactions to blockchain networks
    #[command(name = "submit-transaction")]
    SubmitTransaction(submit::SubmitCommand),
    
    /// Start interactive REPL
    Repl {
        /// Enable debug mode
        #[arg(long)]
        debug: bool,
        
        /// Show machine state after each evaluation
        #[arg(long)]
        show_state: bool,
    },
    
    /// Test effects and components
    TestEffects(test_effects::TestEffectsCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Create error handler
    let error_handler = Arc::new(CliErrorHandler::new(None, false, false));
    
    // Execute the appropriate command
    match cli.command {
        Commands::Compile(cmd) => cmd.execute().await,
        Commands::Repl { debug, show_state } => {
            let config = repl::ReplCommand {
                debug,
                max_steps: Some(10000),
                show_state,
            };
            repl::handle_repl_command(config, error_handler).await
        },
        Commands::TestEffects(cmd) => cmd.execute().await,
        Commands::Simulate(cmd) => cmd.execute().await,
        Commands::Prove(cmd) => cmd.execute().await,
        Commands::SubmitTransaction(cmd) => cmd.execute().await,
    }
}
