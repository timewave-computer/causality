//! Causality CLI entry point
//!
//! Minimal command-line interface for the Causality system.

// use causality_types::core::contextual_error::DefaultErrorContext; // Unused
use clap::{Parser, Subcommand};
use std::process;
use std::sync::Arc;

mod commands;
mod error;

use commands::*;
use error::CliErrorHandler;

//-----------------------------------------------------------------------------
// Command Definition
//-----------------------------------------------------------------------------

/// Causality command-line interface
///
/// Provides a minimal, clean interface to the Causality system
#[derive(Debug, Parser)]
#[command(name = "causality", about = "Causality system command-line tools")]
struct Cli {
    /// Enable verbose error output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Project management operations
    Project(ProjectCommands),

    /// Simulation operations
    Simulate(SimulateCommands),

    /// Debugging tools
    Debug(DebugCommands),

    /// Blockchain intent operations
    Intent(IntentCommand),

    /// Zero-knowledge proof operations
    #[command(subcommand)]
    Zk(ZkCommands),
}

//-----------------------------------------------------------------------------
// Main Function
//-----------------------------------------------------------------------------

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Create error handler
    let error_handler = Arc::new(CliErrorHandler::new(None, cli.verbose, false));

    // Execute the command and handle any errors
    let result = match cli.command {
        Command::Project(cmd) => {
            handle_project_command(cmd.command, error_handler.clone()).await
        }
        Command::Simulate(cmd) => {
            handle_simulate_command(cmd.command, error_handler.clone()).await
        }
        Command::Debug(cmd) => {
            handle_debug_command(cmd.command, error_handler.clone()).await
        }
        Command::Zk(cmd) => cmd.execute(error_handler.clone()).await,
        Command::Intent(cmd) => {
            intent::handle_intent_command(cmd.action, error_handler.clone()).await
        }
    };

    // Handle any errors
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            error_handler.handle_error(&err);
            process::exit(1); // Default to exit code 1 for errors
        }
    }
}
