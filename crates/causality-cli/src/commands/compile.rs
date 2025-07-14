//! Compile command for transforming Lisp S-expression code into bytecode.

use anyhow::Result;
use causality_compiler::{compile, CompiledArtifact};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct CompileCommand {
    /// Input file containing the Lisp S-expression source code (.sx)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output file for the compiled bytecode (.bc)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Output format (only bytecode is supported)
    #[arg(
        long,
        default_value = "bytecode",
        help = "Output format (only 'bytecode' is currently supported)"
    )]
    pub format: String,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable optimization passes
    #[arg(long)]
    pub optimize: bool,
}

impl CompileCommand {
    pub async fn execute(&self) -> Result<()> {
        if self.verbose {
            println!(" Starting Lisp compilation process...");
            println!("   Input: {}", self.input.display());
            println!("   Output: {}", self.output.display());
        }

        // Validate file extensions
        if self.input.extension().and_then(|s| s.to_str()) != Some("sx") {
            println!("Warning: Input file does not have a .sx extension. Assuming S-expression format.");
        }
        if self.output.extension().and_then(|s| s.to_str()) != Some("bc") {
            println!("Warning: Output file does not have a .bc extension. It will contain raw bytecode.");
        }

        // Read the input Lisp S-expression file
        let source_code = fs::read_to_string(&self.input).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read input file {}: {}",
                self.input.display(),
                e
            )
        })?;

        if self.verbose {
            println!("Source code loaded ({} bytes)", source_code.len());
        }

        // The only supported format is bytecode
        if self.format != "bytecode" {
            return Err(anyhow::anyhow!(
                "Unsupported output format: {}. Only 'bytecode' is supported.",
                self.format
            ));
        }

        if self.verbose {
            println!("Compiling to bytecode...");
        }

        // Compile S-expression to intermediate representation
        let compiled_artifact = compile(&source_code)?;

        if self.verbose {
            println!("    Lisp → IR compilation complete");
            println!(
                "   Instructions generated: {}",
                compiled_artifact.instructions.len()
            );
        }

        // Serialize the artifact to bytecode
        let bytecode = self.serialize_bytecode(&compiled_artifact)?;

        if self.verbose {
            println!(
                "    Bytecode serialization complete ({} bytes)",
                bytecode.len()
            );
        }

        // Write the output
        fs::write(&self.output, bytecode).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write output file {}: {}",
                self.output.display(),
                e
            )
        })?;

        if self.verbose {
            println!("💾 Output written to {}", self.output.display());
            println!("Compilation completed successfully!");
        }

        Ok(())
    }

    fn serialize_bytecode(&self, artifact: &CompiledArtifact) -> Result<Vec<u8>> {
        // Use bincode for a compact binary representation
        bincode::serialize(artifact).map_err(|e| {
            anyhow::anyhow!("Failed to serialize artifact to bytecode: {}", e)
        })
    }
}
