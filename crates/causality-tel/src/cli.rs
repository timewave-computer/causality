// TEL CLI implementation
use std::fs;
use std::path::Path;
use colored::*;
use clap::{Parser, Subcommand};

use crate::parser;

/// CLI for the Temporal Effect Language
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Validate a TEL file
    Validate {
        /// Path to the TEL file
        path: std::path::PathBuf,
        
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Run a TEL file
    Run {
        /// Path to the TEL file
        path: std::path::PathBuf,
        
        /// Arguments in key=value format
        #[arg(short, long)]
        args: Vec<String>,
    },
}

/// Run the TEL CLI
pub fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Validate { path, verbose } => {
            validate_tel_file(&path, verbose)?;
        },
        Command::Run { path, args } => {
            run_tel_file(&path, &args)?;
        },
    }
    
    Ok(())
}

/// Validate a TEL file
fn validate_tel_file(file_path: &Path, verbose: bool) -> anyhow::Result<()> {
    // Read the file
    let content = fs::read_to_string(file_path)?;
    
    // Parse the program
    match parser::parse_program(&content) {
        Ok(program) => {
            if verbose {
                println!("{} {}", "Success:".green().bold(), "TEL program is valid");
                println!("Program contains {} statements", program.statements.len());
            } else {
                println!("{} {}", "Success:".green().bold(), "Valid TEL syntax");
            }
            Ok(())
        },
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            std::process::exit(1);
        }
    }
}

/// Run a TEL file
fn run_tel_file(file_path: &Path, args: &[String]) -> anyhow::Result<()> {
    // Parse arguments
    let mut arg_map = Vec::new();
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            arg_map.push((key, value));
        } else {
            eprintln!("{} Invalid argument format: {}", "Warning:".yellow().bold(), arg);
            eprintln!("Arguments should be in the format key=value");
        }
    }
    
    // Read the file
    let content = fs::read_to_string(file_path)?;
    let file_name = file_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed");
    
    // Parse the program
    match parser::parse_program(&content) {
        Ok(_program) => {
            // In a full implementation, we would execute the program here
            println!("{} {}", "Info:".blue().bold(), "Program parsed successfully but execution is not yet implemented");
            println!("File: {}", file_name);
            println!("Arguments: {:?}", arg_map);
            Ok(())
        },
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            std::process::exit(1);
        }
    }
} 