// Purpose: Provides a command-line interface for the simulation controller.

use crate::controller::{BasicSimulationController, SimulationController, ScenarioStatus};
use crate::observer::{CallbackObserver, LogFilter};
use crate::scenario::Scenario;
use crate::agent::AgentId;
use crate::replay::LogEntry;
use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueEnum};
use tokio::sync::{mpsc, broadcast};
use std::path::PathBuf;
use std::time::Duration;
use std::fs;
use serde_json::{self, Value};
use tokio;
use chrono;
use tokio::time::sleep;
use tokio::io::{self, AsyncWriteExt};
use std::sync::Arc;
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
    
    /// Check invariants for a running scenario
    CheckInvariants(CheckInvariantsArgs),
    
    /// List running scenarios
    List,
    
    /// Inject a fact into a running scenario
    InjectFact(InjectFactArgs),
    
    /// Query the state of an agent in a running scenario
    QueryAgent(QueryAgentArgs),
    
    /// Pause a running scenario
    Pause(ScenarioNameArg),
    
    /// Resume a paused scenario
    Resume(ScenarioNameArg),
    
    /// View logs for a running scenario
    Logs(LogsArgs),
    
    /// Show the status of a scenario
    Status(ScenarioNameArg),
}

#[derive(Args, Debug)]
struct RunArgs {
    /// Scenario file path
    #[clap(required = true)]
    scenario_path: PathBuf,
    
    /// Fail on invariant violation
    #[clap(long)]
    fail_on_invariant_violation: bool,
}

#[derive(Args, Debug)]
struct ReplayArgs {
    /// Log directory path containing the scenario to replay
    #[clap(required = true)]
    log_path: PathBuf,

    /// Step through the replay one entry at a time
    #[clap(long)]
    step: bool,
    
    /// Check invariants during replay
    #[clap(long)]
    check_invariants: bool,
}

#[derive(Args, Debug)]
struct CheckInvariantsArgs {
    /// Scenario name
    #[clap(required = true)]
    scenario_name: String,
}

#[derive(Args, Debug)]
struct ScenarioNameArg {
    /// Scenario name
    #[clap(required = true)]
    scenario_name: String,
}

#[derive(Args, Debug)]
struct InjectFactArgs {
    /// Scenario name
    #[clap(required = true)]
    scenario_name: String,
    
    /// JSON file containing the fact to inject
    #[clap(required = true)]
    fact_file: PathBuf,
}

#[derive(Args, Debug)]
struct QueryAgentArgs {
    /// Scenario name
    #[clap(required = true)]
    scenario_name: String,
    
    /// Agent ID
    #[clap(required = true)]
    agent_id: String,
    
    /// Query to execute
    #[clap(required = true)]
    query: String,
    
    /// Output format
    #[clap(long, value_enum, default_value_t = OutputFormat::Pretty)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct LogsArgs {
    /// Scenario name
    #[clap(required = true)]
    scenario_name: String,
    
    /// Maximum number of log entries to show
    #[clap(long, default_value = "100")]
    limit: usize,
    
    /// Follow log output (stream new entries)
    #[clap(long, short = 'f')]
    follow: bool,
    
    /// Filter logs by type
    #[clap(long)]
    entry_type: Option<String>,
    
    /// Filter logs by agent ID
    #[clap(long)]
    agent_id: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Pretty-printed JSON
    Pretty,
    /// Compact JSON
    Compact,
    /// YAML format
    Yaml,
}

/// Run the CLI
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize the controller
    let controller = BasicSimulationController::default()?;

    match cli.command {
        Command::Run(args) => {
            run_scenario(&controller, args).await?;
        }
        Command::Replay(args) => {
            replay_scenario(&controller, args).await?;
        }
        Command::CheckInvariants(args) => {
            check_invariants(&controller, args).await?;
        }
        Command::List => {
            list_scenarios(&controller).await?;
        }
        Command::InjectFact(args) => {
            inject_fact(&controller, args).await?;
        }
        Command::QueryAgent(args) => {
            query_agent(&controller, args).await?;
        }
        Command::Pause(args) => {
            pause_scenario(&controller, args).await?;
        }
        Command::Resume(args) => {
            resume_scenario(&controller, args).await?;
        }
        Command::Logs(args) => {
            view_logs(&controller, args).await?;
        }
        Command::Status(args) => {
            show_status(&controller, args).await?;
        }
    }

    Ok(())
}

/// Run a scenario using the controller
async fn run_scenario(controller: &BasicSimulationController, args: RunArgs) -> Result<()> {
    info!("Loading and starting scenario: {}", args.scenario_path.display());

    // Read and parse the scenario file
    let scenario_content = fs::read_to_string(&args.scenario_path)
        .with_context(|| format!("Failed to read scenario file: {}", args.scenario_path.display()))?;
        
    let scenario: Scenario = toml::from_str(&scenario_content)
        .with_context(|| format!("Failed to parse scenario TOML: {}", args.scenario_path.display()))?;
    
    // Create a channel for receiving logs
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Create a callback observer that prints logs to the console
    let log_observer = Arc::new(CallbackObserver::new(
        "cli",
        None, // No filter
        Box::new(move |entry| {
            let log_message = format!("{} | {:?} | {}", 
                entry.timestamp.to_rfc3339(),
                entry.entry_type,
                serde_json::to_string_pretty(&entry.payload).unwrap_or_else(|_| "Error serializing payload".to_string()));
            
            if let Err(e) = tx.try_send(log_message) {
                error!("Failed to send log message: {}", e);
            }
        }),
    ));

    // Register the observer
    controller.observer_registry().register(log_observer);

    // Create an invariant violation observer
    let scenario_name = scenario.name.clone();
    let (violation_tx, mut violation_rx) = mpsc::channel::<String>(100);
    
    // Start the scenario
    let scenario_arc = Arc::new(scenario);
    if let Err(e) = controller.start_scenario(scenario_arc).await {
        return Err(anyhow!("Failed to start scenario: {}", e));
    }

    // Print log messages as they arrive
    info!("Scenario {} started. Press Ctrl+C to stop.", scenario_name);
    
    // Simple way to wait for Ctrl+C
    let shutdown_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_signal_clone = shutdown_signal.clone();
    let scenario_name_clone = scenario_name.clone();
    let controller_clone = controller;
    
    ctrlc::set_handler(move || {
        shutdown_signal_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut has_invariant_violations = false;
    loop {
        // Check if shutdown was requested
        if shutdown_signal.load(std::sync::atomic::Ordering::SeqCst) {
            info!("Stopping scenario...");
            if let Err(e) = controller_clone.stop_scenario(&scenario_name_clone).await {
                error!("Failed to stop scenario: {}", e);
            }
            break;
        }

        tokio::select! {
            Some(log) = rx.recv() => {
                println!("{}", log);
            }
            Some(violation) = violation_rx.recv() => {
                has_invariant_violations = true;
                eprintln!("🚨 INVARIANT VIOLATION: {}", violation);
                
                if args.fail_on_invariant_violation {
                    error!("Terminating scenario due to invariant violation");
                    if let Err(e) = controller_clone.stop_scenario(&scenario_name_clone).await {
                        error!("Failed to stop scenario: {}", e);
                    }
                    break;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Just to allow checking the shutdown signal periodically
                continue;
            }
        }
    }

    // After the scenario stops, check for any invariant violations
    let violations = controller.get_invariant_violations(&scenario_name).await?;
    if !violations.is_empty() {
        has_invariant_violations = true;
        error!("Invariant violations detected:");
        for violation in &violations {
            eprintln!("🚨 {}", violation);
        }
    }

    if has_invariant_violations && args.fail_on_invariant_violation {
        return Err(anyhow!("Scenario completed with invariant violations"));
    }

    info!("Scenario stopped.");
    Ok(())
}

/// Replay a scenario from logs
async fn replay_scenario(controller: &BasicSimulationController, args: ReplayArgs) -> Result<()> {
    info!("Replaying scenario from logs: {}", args.log_path.display());
    
    // TODO: Implement replay with invariant checking
    warn!("Replay functionality is not fully implemented yet");
    
    Ok(())
}

/// Check invariants for a running scenario
async fn check_invariants(controller: &BasicSimulationController, args: CheckInvariantsArgs) -> Result<()> {
    let violations = controller.get_invariant_violations(&args.scenario_name).await?;
    
    if violations.is_empty() {
        println!("✅ No invariant violations detected for scenario: {}", args.scenario_name);
    } else {
        println!("🚨 Invariant violations for scenario {}:", args.scenario_name);
        for (i, violation) in violations.iter().enumerate() {
            println!("  {}. {}", i+1, violation);
        }
    }
    
    Ok(())
}

/// List all running scenarios
async fn list_scenarios(controller: &impl SimulationController) -> Result<()> {
    let scenarios = controller.list_scenarios().await?;
    
    if scenarios.is_empty() {
        println!("No running scenarios");
    } else {
        println!("Running scenarios:");
        for (i, scenario) in scenarios.iter().enumerate() {
            let status = controller.get_scenario_status(scenario).await?;
            println!("  {}. {} - {:?}", i+1, scenario, status);
        }
    }
    
    Ok(())
}

/// Inject a fact into a running scenario
async fn inject_fact(controller: &impl SimulationController, args: InjectFactArgs) -> Result<()> {
    // Read the fact file
    let fact_content = fs::read_to_string(&args.fact_file)
        .with_context(|| format!("Failed to read fact file: {}", args.fact_file.display()))?;
    
    // Parse the JSON
    let fact_data: Value = serde_json::from_str(&fact_content)
        .with_context(|| format!("Failed to parse fact JSON: {}", args.fact_file.display()))?;
    
    // Inject the fact
    controller.inject_fact(&args.scenario_name, fact_data.clone()).await?;
    
    println!("✅ Successfully injected fact into scenario: {}", args.scenario_name);
    println!("Fact content:");
    println!("{}", serde_json::to_string_pretty(&fact_data)?);
    
    Ok(())
}

/// Query the state of an agent in a running scenario
async fn query_agent(controller: &impl SimulationController, args: QueryAgentArgs) -> Result<()> {
    // Parse the agent ID
    let agent_id: AgentId = args.agent_id.parse()
        .map_err(|_| anyhow!("Invalid agent ID: {}", args.agent_id))?;
    
    // Execute the query
    let result = controller.query_agent_state(&args.scenario_name, &agent_id, &args.query).await?;
    
    // Format the output
    match args.format {
        OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Compact => {
            println!("{}", serde_json::to_string(&result)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&result)?);
        }
    }
    
    Ok(())
}

/// Pause a running scenario
async fn pause_scenario(controller: &impl SimulationController, args: ScenarioNameArg) -> Result<()> {
    controller.pause_scenario(&args.scenario_name).await?;
    println!("✅ Scenario paused: {}", args.scenario_name);
    Ok(())
}

/// Resume a paused scenario
async fn resume_scenario(controller: &impl SimulationController, args: ScenarioNameArg) -> Result<()> {
    controller.resume_scenario(&args.scenario_name).await?;
    println!("✅ Scenario resumed: {}", args.scenario_name);
    Ok(())
}

/// View logs for a running scenario
async fn view_logs(controller: &impl SimulationController, args: LogsArgs) -> Result<()> {
    // Retrieve logs
    let logs = controller.get_scenario_logs(&args.scenario_name, Some(args.limit)).await?;
    
    if logs.is_empty() {
        println!("No logs found for scenario: {}", args.scenario_name);
        return Ok(());
    }
    
    // Apply filters if specified
    let filtered_logs = logs.into_iter().filter(|entry| {
        // Filter by entry type if specified
        if let Some(ref entry_type) = args.entry_type {
            if entry.entry_type.to_string() != *entry_type {
                return false;
            }
        }
        
        // Filter by agent ID if specified
        if let Some(ref agent_id_str) = args.agent_id {
            if let Some(ref entry_agent_id) = entry.agent_id {
                if entry_agent_id.to_string() != *agent_id_str {
                    return false;
                }
            } else {
                // Entry has no agent ID but filter requires one
                return false;
            }
        }
        
        true
    }).collect::<Vec<_>>();
    
    if filtered_logs.is_empty() {
        println!("No logs match the specified filters for scenario: {}", args.scenario_name);
        return Ok(());
    }
    
    // Display logs
    for entry in filtered_logs {
        println!("{} | {:?} | {}", 
            entry.timestamp.to_rfc3339(),
            entry.entry_type,
            serde_json::to_string_pretty(&entry.payload).unwrap_or_else(|_| "Error serializing payload".to_string()));
    }
    
    if args.follow {
        println!("Following logs... Press Ctrl+C to stop.");
        
        // Set up callback observer for new logs
        let (tx, mut rx) = mpsc::channel::<String>(100);
        
        // Create a callback observer that prints logs to the console
        let log_observer = Arc::new(CallbackObserver::new(
            "cli-follow",
            Some(LogFilter::new().with_domain(args.scenario_name.clone())),
            Box::new(move |entry| {
                // Apply filters
                let mut filter_out = false;
                
                // Filter by entry type if specified
                if let Some(ref entry_type) = args.entry_type {
                    if entry.entry_type.to_string() != *entry_type {
                        filter_out = true;
                    }
                }
                
                // Filter by agent ID if specified
                if let Some(ref agent_id_str) = args.agent_id {
                    if let Some(ref entry_agent_id) = entry.agent_id {
                        if entry_agent_id.to_string() != *agent_id_str {
                            filter_out = true;
                        }
                    } else {
                        // Entry has no agent ID but filter requires one
                        filter_out = true;
                    }
                }
                
                if !filter_out {
                    let log_message = format!("{} | {:?} | {}", 
                        entry.timestamp.to_rfc3339(),
                        entry.entry_type,
                        serde_json::to_string_pretty(&entry.payload).unwrap_or_else(|_| "Error serializing payload".to_string()));
                    
                    if let Err(e) = tx.try_send(log_message) {
                        error!("Failed to send log message: {}", e);
                    }
                }
            }),
        ));
        
        // Register the observer
        controller.observer_registry().register(log_observer);
        
        // Wait for Ctrl+C using atomic bool for signaling
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let running_clone = running.clone();
        
        ctrlc::set_handler(move || {
            running_clone.store(false, std::sync::atomic::Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
        
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            tokio::select! {
                Some(log) = rx.recv() => {
                    println!("{}", log);
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Just polling to check the running flag periodically
                }
            }
        }
    }
    
    Ok(())
}

/// Show the status of a scenario
async fn show_status(controller: &impl SimulationController, args: ScenarioNameArg) -> Result<()> {
    let status = controller.get_scenario_status(&args.scenario_name).await?;
    
    match status {
        ScenarioStatus::Running => {
            println!("Scenario '{}' is currently running", args.scenario_name);
        }
        ScenarioStatus::Paused => {
            println!("Scenario '{}' is currently paused", args.scenario_name);
        }
        ScenarioStatus::Stopped => {
            println!("Scenario '{}' is stopped", args.scenario_name);
        }
        ScenarioStatus::NotFound => {
            println!("Scenario '{}' was not found", args.scenario_name);
        }
    }
    
    // Check for invariant violations
    match controller.get_invariant_violations(&args.scenario_name).await {
        Ok(violations) => {
            if violations.is_empty() {
                println!("No invariant violations detected");
            } else {
                println!("Invariant violations detected: {}", violations.len());
                for (i, violation) in violations.iter().enumerate() {
                    println!("  {}. {}", i+1, violation);
                }
            }
        }
        Err(_) => {
            // Ignore error since the scenario might not exist
        }
    }
    
    Ok(())
} 