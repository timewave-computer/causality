// Purpose: Provides a command-line interface for the simulation controller.

use crate::controller::{Controller, SimulationController};
use crate::observer::{MemoryObserver, CallbackObserver, LogFilter};
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Causality Simulation CLI
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Log directory path
    #[clap(long, env = "CAUSALITY_LOG_DIR", default_value = "logs")]
    log_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run a scenario
    Run(RunArgs),

    /// Replay a scenario from logs
    Replay(ReplayArgs),
}

#[derive(Args, Debug)]
struct RunArgs {
    /// Scenario file path
    #[clap(required = true)]
    scenario_path: PathBuf,
}

#[derive(Args, Debug)]
struct ReplayArgs {
    /// Log directory path containing the scenario to replay
    #[clap(required = true)]
    log_path: PathBuf,

    /// Step through the replay one entry at a time
    #[clap(long)]
    step: bool,
}

/// Run the CLI
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize the controller with the specified log directory
    let controller = SimulationController::new()
        .with_log_dir(cli.log_dir);

    match cli.command {
        Command::Run(args) => {
            run_scenario(&controller, args).await?;
        }
        Command::Replay(args) => {
            replay_scenario(&controller, args).await?;
        }
    }

    Ok(())
}

/// Run a scenario using the controller
async fn run_scenario(controller: &SimulationController, args: RunArgs) -> Result<()> {
    info!("Loading and starting scenario: {}", args.scenario_path.display());

    // Create a channel for receiving logs
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Create a callback observer that prints logs to the console
    let observer = Arc::new(CallbackObserver::new(
        "cli",
        None, // No filter
        Box::new(move |entry| {
            let log_message = format!("{} | {:?} | {}", 
                chrono::Utc.timestamp_millis(entry.timestamp as i64).to_rfc3339(),
                entry.entry_type,
                entry.payload);
            
            if let Err(e) = tx.try_send(log_message) {
                error!("Failed to send log message: {}", e);
            }
        }),
    ));

    // Load and start the scenario
    let scenario_id = controller.load_and_start_scenario(args.scenario_path).await
        .context("Failed to start scenario")?;

    // Register the observer
    controller.register_observer(&scenario_id, observer).await
        .context("Failed to register observer")?;

    // Print log messages as they arrive
    info!("Scenario {} started. Press Ctrl+C to stop.", scenario_id);
    
    // Simple way to wait for Ctrl+C
    let (ctrlc_tx, ctrlc_rx) = tokio::sync::oneshot::channel::<()>();
    let scenario_id_clone = scenario_id.clone();
    let controller_clone = controller.clone();
    
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(());
    }).expect("Error setting Ctrl-C handler");

    loop {
        tokio::select! {
            Some(log) = rx.recv() => {
                println!("{}", log);
            }
            _ = ctrlc_rx => {
                info!("Stopping scenario...");
                if let Err(e) = controller_clone.stop_scenario(&scenario_id_clone).await {
                    error!("Failed to stop scenario: {}", e);
                }
                break;
            }
        }
    }

    info!("Scenario stopped.");
    Ok(())
}

/// Replay a scenario from logs
async fn replay_scenario(controller: &SimulationController, args: ReplayArgs) -> Result<()> {
    info!("Replaying scenario from logs: {}", args.log_path.display());
    
    // Create a channel for receiving logs
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Create a callback observer that prints logs to the console
    let observer = Arc::new(CallbackObserver::new(
        "replay-cli",
        None, // No filter
        Box::new(move |entry| {
            let log_message = format!("{} | {:?} | {}", 
                chrono::Utc.timestamp_millis(entry.timestamp as i64).to_rfc3339(),
                entry.entry_type,
                entry.payload);
            
            if let Err(e) = tx.try_send(log_message) {
                error!("Failed to send log message: {}", e);
            }
        }),
    ));

    // Start the replay
    let replay_id = controller.replay_scenario(args.log_path).await
        .context("Failed to start replay")?;

    // Register the observer
    controller.register_replay_observer(&replay_id, observer).await
        .context("Failed to register observer")?;

    info!("Replay {} started.", replay_id);
    
    if args.step {
        // Step through the replay manually
        let (ctrlc_tx, ctrlc_rx) = tokio::sync::oneshot::channel::<()>();
        
        ctrlc::set_handler(move || {
            let _ = ctrlc_tx.send(());
        }).expect("Error setting Ctrl-C handler");
        
        println!("Press Enter to step through log entries. Press Ctrl+C to exit.");
        let stdin = tokio::io::stdin();
        let mut buffer = [0; 1];
        
        loop {
            tokio::select! {
                Some(log) = rx.recv() => {
                    println!("{}", log);
                }
                _ = ctrlc_rx => {
                    info!("Exiting replay mode...");
                    break;
                }
                _ = stdin.read(&mut buffer) => {
                    match controller.step_replay(&replay_id).await {
                        Ok(Some(_)) => {
                            // The actual log entry will be sent to us via the observer channel
                        }
                        Ok(None) => {
                            info!("Replay complete!");
                            break;
                        }
                        Err(e) => {
                            error!("Error stepping through replay: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    } else {
        // Run the replay to completion
        if let Err(e) = controller.run_replay(&replay_id).await {
            error!("Failed to run replay: {}", e);
            return Err(e);
        }
        
        // Process any remaining logs
        while let Ok(log) = rx.try_recv() {
            println!("{}", log);
        }
        
        info!("Replay complete!");
    }
    
    Ok(())
} 