use anyhow::{Context, Result};
use causality_types::serialization::Encode;
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

// Assuming CompiledTeg is in the root of the causality_compiler crate
// and compile_teg_definition is also available.
use causality_compiler::compile_teg_definition;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to the input TEG definition file (.teg)
    #[clap(value_parser)]
    input_file: PathBuf,

    /// Optional path for the compiled output file
    #[clap(short, long, value_parser)]
    output_file: Option<PathBuf>,

    /// Optional name to override the program name defined in the TEG file
    #[clap(short, long)]
    name: Option<String>,
}

fn main() -> Result<()> {
    // Initialize logging using env_logger.
    // You can control the default log level via the RUST_LOG environment variable.
    // For example: RUST_LOG=causality_compiler=debug,info (sets debug for this crate, info for others)
    // If RUST_LOG is not set, it defaults to `info` level for all modules.
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    let cli = Cli::parse();

    log::info!("Compiling TEG definition file: {:?}", cli.input_file);
    if let Some(name_override) = &cli.name {
        log::info!("Overriding program name with: {}", name_override);
    }

    let compiled_teg = compile_teg_definition(&cli.input_file, cli.name.clone())
        .with_context(|| {
            format!("Failed to compile TEG file: {:?}", cli.input_file)
        })?;

    log::info!(
        "Compilation successful for program: {} (ID: {})",
        compiled_teg.name,
        compiled_teg.id
    );

    let output_path = cli.output_file.unwrap_or_else(|| {
        let mut path = cli.input_file.clone();
        let original_filename =
            path.file_name().unwrap_or_default().to_string_lossy();
        let new_filename = format!(
            "{}.compiled.cbor",
            original_filename
                .strip_suffix(".teg")
                .unwrap_or(&original_filename)
        );
        path.set_file_name(new_filename);
        path
    });

    log::info!("Writing compiled TEG to: {:?}", output_path);

    let serialized_data = compiled_teg
        .as_ssz_bytes();
    // SSZ serialization doesn't fail, so we don't need error handling here

    let mut file = File::create(&output_path).with_context(|| {
        format!("Failed to create output file: {:?}", output_path)
    })?;

    file.write_all(&serialized_data).with_context(|| {
        format!(
            "Failed to write serialized data to output file: {:?}",
            output_path
        )
    })?;

    log::info!("Successfully wrote compiled TEG to {:?}", output_path);

    Ok(())
}
