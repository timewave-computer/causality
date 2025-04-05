// Effect Adapter Generator CLI
// This is the main entry point for the causality tools

use std::path::PathBuf;
use std::env;
use colored::Colorize;
use anyhow::{Result, Context};

mod schemas;
mod codegen;

// CLI usage message
const USAGE: &str = "
Usage: adapter-gen <schema_file> <output_dir> [language] [--verbose]

Arguments:
  <schema_file>    Path to the adapter schema file
  <output_dir>     Directory to output generated code
  [language]       Target language (rust, typescript) [default: rust]
  [--verbose]      Enable verbose output

Example:
  adapter-gen schemas/ethereum.json generated/ rust --verbose
";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check if help is requested
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("{}", USAGE);
        return Ok(());
    }

    // Parse arguments
    if args.len() < 3 {
        eprintln!("{} Invalid arguments", "Error:".red().bold());
        eprintln!("{}", USAGE);
        std::process::exit(1);
    }
    
    let schema_path = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    let language = if args.len() > 3 && !args[3].starts_with("--") { &args[3] } else { "rust" };
    let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
    
    // Validate input files exist
    if !schema_path.exists() {
        eprintln!("{} Schema file '{}' does not exist", 
            "Error:".red().bold(), 
            schema_path.display());
        std::process::exit(1);
    }
    
    println!("{} effect adapter from schema: {}",
        "Generating".green().bold(),
        schema_path.display()
    );
    
    println!("{} {}",
        "Output directory:".green(),
        output_dir.display()
    );
    
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir)
        .context(format!("Failed to create output directory: {}", output_dir.display()))?;
    
    // Compile the schema
    if let Err(e) = codegen::compile_schema(&schema_path, &output_dir, language, verbose) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    } else {
        println!("{}", "Adapter generation completed successfully!".green().bold());
    }
    
    Ok(())
}

// Create template directories for development
#[allow(dead_code)]
fn create_template_dirs(base_dir: &PathBuf) -> Result<()> {
    // Create the base directories
    let template_dir = base_dir.join("templates");
    let schemas_dir = base_dir.join("schemas");
    let generated_dir = base_dir.join("generated");
    
    std::fs::create_dir_all(&template_dir)?;
    std::fs::create_dir_all(&schemas_dir)?;
    std::fs::create_dir_all(&generated_dir)?;
    
    // Create a sample template file
    let rust_template_dir = template_dir.join("rust");
    std::fs::create_dir_all(&rust_template_dir)?;
    
    let adapter_template = rust_template_dir.join("adapter.rs.tmpl");
    std::fs::write(adapter_template, include_str!("../templates/rust/adapter.rs.tmpl"))?;
    
    // Create a sample schema file
    let sample_schema = schemas_dir.join("sample.json");
    std::fs::write(sample_schema, include_str!("../schemas/sample.json"))?;
    
    println!("{}", "Template directories created successfully!".green().bold());
    
    Ok(())
} 