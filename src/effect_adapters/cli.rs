//! Effect Adapter Generator CLI
//!
//! This module provides the command-line interface for generating effect adapters
//! from schema files.

use std::path::PathBuf;
use structopt::StructOpt;

use crate::error::Result;
use crate::effect_adapters;

/// Command-line arguments for the adapter generator
#[derive(Debug, StructOpt)]
#[structopt(name = "adapter-gen", about = "Generate effect adapter code from schemas")]
pub struct AdapterGenCli {
    /// Path to the adapter schema file
    #[structopt(short, long, parse(from_os_str))]
    pub input: PathBuf,
    
    /// Directory to output generated code
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
    
    /// Target language for code generation (rust, typescript)
    #[structopt(short, long, default_value = "rust")]
    pub language: String,
    
    /// Verbose output
    #[structopt(short, long)]
    pub verbose: bool,
}

/// Run the adapter generator with the provided command-line arguments
pub fn run_adapter_gen(opt: &AdapterGenCli) -> Result<()> {
    // Print info if verbose
    if opt.verbose {
        println!("Generating {} adapter from schema: {}", 
            opt.language,
            opt.input.display());
        println!("Output directory: {}", opt.output.display());
    }
    
    // Generate the adapter
    effect_adapters::compile_schema(&opt.input, &opt.output, &opt.language)?;
    
    if opt.verbose {
        println!("Adapter generation completed successfully!");
    }
    
    Ok(())
}

/// Entry point function for binary execution
pub fn execute_cli() -> Result<()> {
    let opt = AdapterGenCli::from_args();
    run_adapter_gen(&opt)
} 