//! Effect Adapter Generator CLI
//!
//! This utility allows generating effect adapters from schema files.

use std::path::PathBuf;
use std::process;
use structopt::StructOpt;
use colored::Colorize;

mod schemas;
mod codegen;

/// CLI arguments
#[derive(Debug, StructOpt)]
#[structopt(name = "adapter-gen", about = "Generate effect adapter code from schemas")]
struct Cli {
    /// Path to the adapter schema file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    
    /// Directory to output generated code
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
    
    /// Target language for code generation (rust, typescript)
    #[structopt(short, long, default_value = "rust")]
    language: String,
    
    /// Verbose output
    #[structopt(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::from_args();
    
    if cli.verbose {
        println!("{} {} adapter from schema: {}", 
            "Generating".green().bold(),
            cli.language.cyan().bold(),
            cli.input.display().to_string().yellow());
        println!("{} {}", 
            "Output directory:".green(),
            cli.output.display().to_string().yellow());
    }
    
    match codegen::compile_schema(&cli.input, &cli.output, &cli.language) {
        Ok(_) => {
            if cli.verbose {
                println!("{}", "Adapter generation completed successfully!".green().bold());
            }
        },
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

/// Create template directories for development
fn create_template_dirs() -> anyhow::Result<()> {
    // Create Rust template directories
    let rust_template_dir = "src/codegen/templates/rust";
    std::fs::create_dir_all(format!("{}/examples", rust_template_dir))?;
    std::fs::create_dir_all(format!("{}/docs", rust_template_dir))?;
    
    // Create TypeScript template directories
    let ts_template_dir = "src/codegen/templates/typescript";
    std::fs::create_dir_all(format!("{}/examples", ts_template_dir))?;
    std::fs::create_dir_all(format!("{}/docs", ts_template_dir))?;
    
    Ok(())
} 