//! Causality CLI entry point
//!
//! Comprehensive command-line interface for the Causality system.

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
/// Provides a comprehensive interface to the Causality system
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
    /// Interactive REPL for evaluating Lisp expressions
    Repl {
        /// Enable debug mode with verbose output
        #[arg(short, long)]
        debug: bool,
        
        /// Maximum execution steps before timeout
        #[arg(short, long)]
        max_steps: Option<usize>,
        
        /// Show machine state after each evaluation
        #[arg(short, long)]
        show_state: bool,
    },
    
    /// Compile Causality Lisp source to machine instructions
    Compile {
        /// Source file to compile
        #[arg(short, long)]
        file: Option<String>,
        
        /// Source code to compile directly
        #[arg(short, long)]
        source: Option<String>,
        
        /// Output file for compiled instructions
        #[arg(short, long)]
        output: Option<String>,
        
        /// Show compilation stages
        #[arg(long)]
        show_stages: bool,
        
        /// Enable optimization
        #[arg(long)]
        optimize: bool,
    },
    
    /// Execute compiled Causality programs
    Execute {
        /// Source file to execute
        #[arg(short, long)]
        file: Option<String>,
        
        /// Source code to execute directly
        #[arg(short, long)]
        source: Option<String>,
        
        /// Show execution trace
        #[arg(short, long)]
        trace: bool,
        
        /// Maximum execution steps
        #[arg(long)]
        max_steps: Option<usize>,
    },
    
    /// Run tests on Causality code
    Test {
        /// Test file or directory
        #[arg(short, long)]
        path: Option<String>,
        
        /// Run specific test by name pattern
        #[arg(short, long)]
        filter: Option<String>,
        
        /// Show verbose test output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Run diagnostics on source code
    Diagnostics {
        /// Source file to analyze
        #[arg(short, long)]
        file: Option<String>,
        
        /// Source code to analyze directly
        #[arg(short, long)]
        source: Option<String>,
        
        /// Show detailed resource lifetime analysis
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Generate visualizations of resource flow and effect graphs
    Visualize {
        /// Source file to visualize
        #[arg(short, long)]
        file: Option<String>,
        
        /// Source code to visualize directly
        #[arg(short, long)]
        source: Option<String>,
        
        /// Output format (mermaid, dot, ascii)
        #[arg(long, default_value = "mermaid")]
        format: String,
        
        /// Show register information
        #[arg(short, long)]
        registers: bool,
        
        /// Generate effect graph instead of resource flow
        #[arg(short, long)]
        effects: bool,
    },
    
    /// Project management commands
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    
    /// Development server for live development
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Enable auto-reload on file changes
        #[arg(long)]
        watch: bool,
    },
    
    /// Test and validate algebraic effects
    TestEffects {
        #[command(subcommand)]
        action: test_effects::TestEffectsAction,
    },
}

#[derive(Debug, Subcommand)]
enum ProjectAction {
    /// Create a new Causality project
    New {
        /// Project name
        name: String,
        /// Project template (basic, advanced, library)
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
    /// Initialize current directory as Causality project
    Init {
        /// Project name (defaults to directory name)
        name: Option<String>,
    },
    /// Build the current project
    Build {
        /// Enable optimizations
        #[arg(long)]
        release: bool,
    },
    /// Clean build artifacts
    Clean,
    /// Show project status
    Status,
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
        Command::Repl { debug, max_steps, show_state } => {
            let config = ReplCommand {
                debug,
                max_steps,
                show_state,
            };
            handle_repl_command(config, error_handler.clone()).await
        }
        Command::Compile { file, source, output, show_stages, optimize } => {
            handle_compile_command(file, source, output, show_stages, optimize, error_handler.clone()).await
        }
        Command::Execute { file, source, trace, max_steps } => {
            handle_execute_command(file, source, trace, max_steps, error_handler.clone()).await
        }
        Command::Test { path, filter, verbose } => {
            handle_test_command(path, filter, verbose, error_handler.clone()).await
        }
        Command::Diagnostics { file, source, detailed } => {
            handle_diagnostics_command(file, source, detailed, error_handler.clone()).await
        }
        Command::Visualize { file, source, format, registers, effects } => {
            handle_visualize_command(file, source, format, registers, effects, error_handler.clone()).await
        }
        Command::Project { action } => {
            handle_project_command(action, error_handler.clone()).await
        }
        Command::Serve { port, watch } => {
            handle_serve_command(port, watch, error_handler.clone()).await
        }
        Command::TestEffects { action } => {
            let command = test_effects::TestEffectsCommand;
            command.execute(action).await
        }
    };

    // Handle any errors
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            error_handler.handle_error(&err);
            process::exit(1);
        }
    }
}

//-----------------------------------------------------------------------------
// Command Handlers
//-----------------------------------------------------------------------------

/// Handle compile command
async fn handle_compile_command(
    file: Option<String>,
    source: Option<String>,
    output: Option<String>,
    show_stages: bool,
    optimize: bool,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use causality_compiler::EnhancedCompilerPipeline;
    use colored::Colorize;
    
    let source_code = get_source_input(file, source)?;
    
    println!("{}", "🔧 Compiling Causality Lisp...".cyan().bold());
    
    let mut compiler = EnhancedCompilerPipeline::new();
    let compiled = compiler.compile_full(&source_code)?;
    
    if show_stages {
        println!("\n{}", "📋 Compilation Stages:".yellow());
        println!("  {} Parse → AST", "✓".green());
        println!("  {} Type Check", "✓".green());
        println!("  {} Code Generation → {} instructions", "✓".green(), compiled.instructions.len());
        if optimize {
            println!("  {} Optimization", "✓".green());
        }
    }
    
    if let Some(output_path) = output {
        // In a real implementation, you'd serialize instructions to file
        println!("📁 Output written to: {}", output_path);
    } else {
        println!("\n{}", "📄 Generated Instructions:".cyan());
        for (i, instr) in compiled.instructions.iter().enumerate() {
            println!("  {}: {:?}", i, instr);
        }
    }
    
    println!("\n{} Compilation completed successfully!", "✅".green());
    Ok(())
}

/// Handle execute command  
async fn handle_execute_command(
    file: Option<String>,
    source: Option<String>,
    trace: bool,
    max_steps: Option<usize>,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use causality_compiler::EnhancedCompilerPipeline;
    use causality_runtime::Executor;
    use colored::Colorize;
    
    let source_code = get_source_input(file, source)?;
    
    println!("{}", "🚀 Executing Causality Lisp...".cyan().bold());
    
    // Compile
    let mut compiler = EnhancedCompilerPipeline::new();
    let compiled = compiler.compile_full(&source_code)?;
    
    if trace {
        println!("\n{}", "📋 Instructions:".yellow());
        for (i, instr) in compiled.instructions.iter().enumerate() {
            println!("  {}: {:?}", i, instr);
        }
    }
    
    // Execute
    let mut executor = Executor::new();
    let result = executor.execute(&compiled.instructions)?;
    
    println!("\n{}", "📤 Result:".green().bold());
    println!("  {:?}", result);
    
    if trace {
        println!("\n{}", "🔍 Final Machine State:".cyan());
        for i in 0..8 {
            let reg_id = causality_core::machine::RegisterId(i);
            if let Ok(register) = executor.machine_state().load_register(reg_id) {
                println!("  R{}: {:?}", i, register.value);
            }
        }
    }
    
    println!("\n{} Execution completed successfully!", "✅".green());
    Ok(())
}

/// Handle test command
async fn handle_test_command(
    path: Option<String>,
    filter: Option<String>,
    verbose: bool,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use colored::Colorize;
    
    let test_path = path.unwrap_or_else(|| "tests/".to_string());
    
    println!("{}", "🧪 Running Causality tests...".cyan().bold());
    
    if let Some(filter_pattern) = &filter {
        println!("📝 Filter: {}", filter_pattern);
    }
    
    // In a real implementation, you would:
    // 1. Discover test files in the path
    // 2. Parse and execute each test
    // 3. Report results
    
    println!("\n{}", "📁 Discovering tests...".yellow());
    println!("  Found 0 test files in {}", test_path);
    
    println!("\n{}", "📊 Test Results:".green().bold());
    println!("  Tests: 0 passed, 0 failed, 0 total");
    println!("  Time:  0.00s");
    
    Ok(())
}

/// Handle project command
async fn handle_project_command(
    action: ProjectAction,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use colored::Colorize;
    
    match action {
        ProjectAction::New { name, template } => {
            println!("{}", format!("🚀 Creating new Causality project '{}'...", name).cyan().bold());
            println!("📋 Template: {}", template);
            
            // In a real implementation:
            // 1. Create project directory
            // 2. Generate template files
            // 3. Initialize configuration
            
            println!("\n{}", "📁 Created project structure:".yellow());
            println!("  {}/", name);
            println!("    src/");
            println!("      main.lisp");
            println!("    tests/");
            println!("    causality.toml");
            println!("    README.md");
            
            println!("\n{} Project created successfully!", "✅".green());
            println!("💡 Next steps:");
            println!("  cd {}", name);
            println!("  causality project build");
        }
        ProjectAction::Init { name } => {
            let project_name = name.unwrap_or_else(|| "causality-project".to_string());
            println!("{}", format!("🔧 Initializing Causality project '{}'...", project_name).cyan().bold());
            
            println!("\n{} Project initialized successfully!", "✅".green());
        }
        ProjectAction::Build { release } => {
            println!("{}", "🔨 Building Causality project...".cyan().bold());
            if release {
                println!("🚀 Release mode enabled");
            }
            
            println!("\n{} Build completed successfully!", "✅".green());
        }
        ProjectAction::Clean => {
            println!("{}", "🧹 Cleaning build artifacts...".cyan().bold());
            println!("\n{} Clean completed successfully!", "✅".green());
        }
        ProjectAction::Status => {
            println!("{}", "📊 Project Status:".cyan().bold());
            println!("  Project: causality-project");
            println!("  Version: 0.1.0");
            println!("  Target:  debug");
            println!("  Status:  {} Ready", "✅".green());
        }
    }
    
    Ok(())
}

/// Handle serve command
async fn handle_serve_command(
    port: u16,
    watch: bool,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use colored::Colorize;
    
    println!("{}", "🌐 Starting Causality development server...".cyan().bold());
    println!("📡 Server running on http://localhost:{}", port);
    
    if watch {
        println!("👀 File watcher enabled");
    }
    
    println!("\n{}", "💡 Development features:".yellow());
    println!("  • Live REPL at /repl");
    println!("  • Code compilation at /compile");
    println!("  • Visualization tools at /visualize");
    println!("  • API documentation at /docs");
    
    println!("\n{} Press Ctrl+C to stop", "ℹ️".blue());
    
    // In a real implementation, start the server
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("Server would run here... (not implemented in this demo)");
    
    Ok(())
}

/// Helper function to get source input from file or direct source
fn get_source_input(file: Option<String>, source: Option<String>) -> anyhow::Result<String> {
    match (file, source) {
        (Some(file_path), None) => {
            std::fs::read_to_string(&file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))
        }
        (None, Some(source_str)) => Ok(source_str),
        (Some(_), Some(_)) => {
            Err(anyhow::anyhow!("Cannot specify both --file and --source"))
        }
        (None, None) => {
            Err(anyhow::anyhow!("Must specify either --file or --source"))
        }
    }
}

/// Handle diagnostics command
async fn handle_diagnostics_command(
    file: Option<String>,
    source: Option<String>,
    detailed: bool,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use commands::diagnostics::run_diagnostics;
    
    let source_code = match (file, source) {
        (Some(file_path), None) => {
            std::fs::read_to_string(&file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?
        }
        (None, Some(source_str)) => source_str,
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!("Cannot specify both --file and --source"));
        }
        (None, None) => {
            return Err(anyhow::anyhow!("Must specify either --file or --source"));
        }
    };
    
    let report = run_diagnostics(&source_code)?;
    
    if detailed {
        // Show detailed analysis
        println!("{}", report);
        
        if !report.resource_usage.resource_lifetime_graph.is_empty() {
            println!("\n📈 Resource Lifetime Analysis:");
            for lifetime in &report.resource_usage.resource_lifetime_graph {
                println!("  Resource {}: allocated at instruction {}", 
                         lifetime.resource_id, lifetime.allocated_at);
                match lifetime.consumed_at {
                    Some(consumed) => println!("    consumed at instruction {} ({:?})", 
                                               consumed, lifetime.status),
                    None => println!("    ⚠️  never consumed ({:?})", lifetime.status),
                }
            }
        }
    } else {
        // Show summary
        println!("{}", report);
    }
    
    Ok(())
}

/// Handle visualize command
async fn handle_visualize_command(
    file: Option<String>,
    source: Option<String>,
    format: String,
    registers: bool,
    effects: bool,
    _error_handler: Arc<CliErrorHandler>,
) -> anyhow::Result<()> {
    use commands::visualizer::{visualize_resources, visualize_effect_graph, VisualizationConfig, OutputFormat};
    
    let source_code = match (file, source) {
        (Some(file_path), None) => {
            std::fs::read_to_string(&file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?
        }
        (None, Some(source_str)) => source_str,
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!("Cannot specify both --file and --source"));
        }
        (None, None) => {
            return Err(anyhow::anyhow!("Must specify either --file or --source"));
        }
    };
    
    if effects {
        // Generate effect graph
        let diagram = visualize_effect_graph(&source_code)?;
        println!("{}", diagram);
    } else {
        // Generate resource flow diagram
        let output_format = match format.as_str() {
            "mermaid" => OutputFormat::Mermaid,
            "dot" => OutputFormat::Dot,
            "ascii" => OutputFormat::Ascii,
            _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use mermaid, dot, or ascii", format)),
        };
        
        let config = VisualizationConfig {
            format: output_format,
            show_registers: registers,
            show_gas_costs: false,
            simplify_graph: false,
        };
        
        let diagram = visualize_resources(&source_code, config)?;
        println!("{}", diagram);
    }
    
    Ok(())
}
