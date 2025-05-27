//! Project Management Command
//!
//! This module implements the project management commands for the Causality CLI,
//! including initialization, building, and testing of Causality projects.

//-----------------------------------------------------------------------------
// Project Management Command
//-----------------------------------------------------------------------------

// use anyhow::anyhow; // Not used
use clap::Args;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;

use crate::error::{CliErrorHandler, CliResult};

/// Project management commands

#[derive(Debug, Args)]
pub struct ProjectCommands {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

/// Project management subcommands
#[derive(Debug, clap::Subcommand)]
pub enum ProjectCommand {
    /// Initialize a new Causality project
    Init(InitArgs),

    /// Build the current project
    Build(BuildArgs),

    /// Run tests for the current project
    Test(TestArgs),
}

/// Arguments for the 'init' command
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Name of the project to create
    #[arg(help = "Name of the project to create")]
    pub name: String,

    /// Template to use (default: basic)
    #[arg(long, short, default_value = "basic")]
    pub template: String,

    /// Output directory (default: current directory)
    #[arg(long, short)]
    pub output_dir: Option<PathBuf>,
}

/// Arguments for the 'build' command
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Build in release mode
    #[arg(long)]
    pub release: bool,

    /// Target directory for build artifacts
    #[arg(long)]
    pub target_dir: Option<PathBuf>,
}

/// Arguments for the 'test' command
#[derive(Debug, Args)]
pub struct TestArgs {
    /// Run only tests containing this string in their names
    #[arg(long)]
    pub filter: Option<String>,

    /// Run tests in release mode
    #[arg(long)]
    pub release: bool,
}

/// Handle project commands
pub async fn handle_project_command(
    command: ProjectCommand,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    match command {
        ProjectCommand::Init(args) => init_project(args, error_handler).await,
        ProjectCommand::Build(args) => build_project(args, error_handler).await,
        ProjectCommand::Test(args) => test_project(args, error_handler).await,
    }
}

/// Initialize a new Causality project
async fn init_project(
    args: InitArgs,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    let project_dir = match &args.output_dir {
        Some(dir) => dir.join(&args.name),
        None => PathBuf::from(&args.name),
    };

    if project_dir.exists() {
        return Err(error_handler.create_error(
            format!(
                "Project directory already exists: {}",
                project_dir.display()
            ),
            "Validation"
        ));
    }

    // Create project directory
    fs::create_dir_all(&project_dir).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to create project directory '{}': {}",
                project_dir.display(),
                e
            ),
            "Storage"
        )
    })?;

    // Create project config file
    create_project_config(&project_dir, &args.name, error_handler.clone()).await?;

    // Create template files based on template type
    match args.template.as_str() {
        "basic" => {
            create_basic_template(&project_dir, error_handler.clone()).await?
        }
        "zk" => create_zk_template(&project_dir, error_handler.clone()).await?,
        "full" => create_full_template(&project_dir, error_handler.clone()).await?,
        _ => {
            return Err(error_handler.create_error(
                format!("Unknown template type: {}", args.template),
                "Validation"
            ));
        }
    }

    println!("Successfully initialized project: {}", args.name);
    Ok(())
}

/// Create project configuration file
async fn create_project_config(
    project_dir: &Path,
    name: &str,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    let config = format!(
        r#"# Causality Project Configuration
name = "{}"
version = "0.1.0"

[dependencies]
causality-types = "0.1.0"
causality-lisp = "0.1.0"
causality-runtime = "0.1.0"
"#,
        name
    );

    let config_path = project_dir.join("causality.toml");

    fs::write(&config_path, config).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write configuration file '{}': {}",
                config_path.display(),
                e
            ),
            "Storage"
        )
    })?;

    Ok(())
}

/// Create basic project template files
async fn create_basic_template(
    project_dir: &Path,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    // Create main.rs
    let main_content = r#"// Basic Causality project

use causality_types::prelude::*;

fn main() {
    println!("Hello from Causality!");
}
"#;
    let main_path = project_dir.join("src/main.rs");
    fs::create_dir_all(main_path.parent().unwrap())
        .await
        .map_err(|e| {
            error_handler.create_error(
                format!(
                    "Failed to create src directory for '{}': {}",
                    main_path.display(),
                    e
                ),
                "Storage"
            )
        })?;
    fs::write(&main_path, main_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write basic template file '{}': {}",
                main_path.display(),
                e
            ),
            "Storage"
        )
    })?;
    println!("Created file: src/main.rs");

    Ok(())
}

/// Create ZK project template files
async fn create_zk_template(
    project_dir: &Path,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    // Create ZK-specific files
    let main_content = r#"// ZK-enabled Causality project

use causality_types::prelude::*;
use causality_zk::prelude::*;

fn main() {
    println!("Hello from ZK-enabled Causality!");
}
"#;
    let main_path = project_dir.join("src/main.rs");
    fs::create_dir_all(main_path.parent().unwrap())
        .await
        .map_err(|e| {
            error_handler.create_error(
                format!(
                    "Failed to create src directory for '{}': {}",
                    main_path.display(),
                    e
                ),
                "Storage"
            )
        })?;
    fs::write(&main_path, main_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write ZK template file '{}': {}",
                main_path.display(),
                e
            ),
            "Storage"
        )
    })?;
    println!("Created file: src/main.rs");

    // Update the project config to include ZK dependencies
    let config_path = project_dir.join("causality.toml");
    let mut config_content =
        fs::read_to_string(&config_path).await.map_err(|e| {
            error_handler.create_error(
                format!(
                    "Failed to read config file '{}' for ZK template: {}",
                    config_path.display(),
                    e
                ),
                "Storage"
            )
        })?;
    config_content.push_str("causality-zk = { version = \"0.1.0\" }\n");
    fs::write(&config_path, config_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write updated config file '{}' for ZK template: {}",
                config_path.display(),
                e
            ),
            "Storage"
        )
    })?;

    Ok(())
}

/// Create full project template files
async fn create_full_template(
    project_dir: &Path,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    // Create a more comprehensive project structure
    let model_dir = project_dir.join("src/model");
    fs::create_dir_all(&model_dir).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to create directory '{}': {}",
                model_dir.display(),
                e
            ),
            "Storage"
        )
    })?;

    let runtime_dir = project_dir.join("src/runtime");
    fs::create_dir_all(&runtime_dir).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to create directory '{}': {}",
                runtime_dir.display(),
                e
            ),
            "Storage"
        )
    })?;

    let zk_dir = project_dir.join("src/zk");
    fs::create_dir_all(&zk_dir).await.map_err(|e| {
        error_handler.create_error(
            format!("Failed to create directory '{}': {}", zk_dir.display(), e),
            "Storage"
        )
    })?;

    // Create main.rs
    let main_content = r#"// Full Causality project

mod model;
mod runtime;
mod zk;

use causality_types::prelude::*;
use causality_runtime::prelude::*;
use causality_zk::prelude::*;

fn main() {
    println!("Hello from full Causality project!");
}
"#;
    let main_path = project_dir.join("src/main.rs");
    fs::create_dir_all(main_path.parent().unwrap())
        .await
        .map_err(|e| {
            error_handler.create_error(
                format!(
                    "Failed to create src directory for '{}': {}",
                    main_path.display(),
                    e
                ),
                "Storage"
            )
        })?;
    fs::write(&main_path, main_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write main.rs for full template '{}': {}",
                main_path.display(),
                e
            ),
            "Storage"
        )
    })?;
    println!("Created file: src/main.rs");

    // Create model.rs
    let model_content = r#"// Data models

use causality_types::prelude::*;

pub struct Model {
    // Define your model here
}
"#;
    let model_path = project_dir.join("src/model/mod.rs");
    fs::write(&model_path, model_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write model/mod.rs for full template '{}': {}",
                model_path.display(),
                e
            ),
            "Storage"
        )
    })?;
    println!("Created file: src/model/mod.rs");

    // Create runtime.rs
    let runtime_content = r#"// Runtime components

use causality_runtime::prelude::*;

pub struct Runtime {
    // Define your runtime here
}
"#;
    let runtime_path = project_dir.join("src/runtime/mod.rs");
    fs::write(&runtime_path, runtime_content)
        .await
        .map_err(|e| {
            error_handler.create_error(
                format!(
                    "Failed to write runtime/mod.rs for full template '{}': {}",
                    runtime_path.display(),
                    e
                ),
                "Storage"
            )
        })?;
    println!("Created file: src/runtime/mod.rs");

    // Create zk.rs
    let zk_content = r#"// ZK components

use causality_zk::prelude::*;

pub struct ZkProver {
    // Define your ZK prover here
}
"#;
    let zk_path = project_dir.join("src/zk/mod.rs");
    fs::write(&zk_path, zk_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write zk/mod.rs for full template '{}': {}",
                zk_path.display(),
                e
            ),
            "Storage"
        )
    })?;
    println!("Created file: src/zk/mod.rs");

    // Update the project config to include all dependencies
    let config_path = project_dir.join("causality.toml");
    let mut config_content =
        fs::read_to_string(&config_path).await.map_err(|e| {
            error_handler.create_error(
                format!(
                    "Failed to read config file '{}' for full template: {}",
                    config_path.display(),
                    e
                ),
                "Storage"
            )
        })?;
    // Ensure dependencies are added only if not present to avoid duplicates
    if !config_content.contains("causality-zk") {
        config_content.push_str("causality-zk = { version = \"0.1.0\" }\n");
    }
    if !config_content.contains("causality-simulation") {
        config_content.push_str("causality-simulation = { version = \"0.1.0\" }\n");
    }
    if !config_content.contains("causality-compiler") {
        config_content.push_str("causality-compiler = { version = \"0.1.0\" }\n");
    }
    fs::write(&config_path, config_content).await.map_err(|e| {
        error_handler.create_error(
            format!(
                "Failed to write updated config file '{}' for full template: {}",
                config_path.display(),
                e
            ),
            "Storage"
        )
    })?;

    Ok(())
}

/// Build the current project
async fn build_project(
    args: BuildArgs,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    println!("Building project...");
    let release_arg = if args.release {
        "--release"
    } else {
        ""
    };
    let target_arg = args
        .target_dir
        .map_or(String::new(), |d| format!("--target-dir={}", d.display()));

    let status = Command::new("cargo")
        .arg("build")
        .arg(release_arg)
        .arg(&target_arg)
        .status()
        .await
        .map_err(|e| {
            error_handler.create_error(
                format!("Failed to execute cargo build: {}", e),
                "System"
            )
        })?;

    if !status.success() {
        return Err(error_handler.create_error(
            format!("Build failed with status: {}", status),
            "System"
        ));
    }

    println!("Project built successfully.");
    Ok(())
}

/// Run tests for the current project
async fn test_project(
    args: TestArgs,
    error_handler: Arc<CliErrorHandler>,
) -> CliResult<()> {
    println!("Running tests...");
    let release_arg = if args.release {
        "--release"
    } else {
        ""
    };
    let filter_arg = args.filter.as_deref().unwrap_or("");

    let mut cmd = Command::new("cargo");
    cmd.arg("test").arg(release_arg);
    if !filter_arg.is_empty() {
        cmd.arg(filter_arg);
    }

    let status = cmd.status().await.map_err(|e| {
        error_handler.create_error(
            format!("Failed to execute cargo test: {}", e),
            "System"
        )
    })?;

    if !status.success() {
        return Err(error_handler.create_error(
            format!("Tests failed with status: {}", status),
            "System"
        ));
    }

    println!("Tests passed successfully.");
    Ok(())
}
