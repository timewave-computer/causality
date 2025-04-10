// Main entry point for the TEL CLI
use causality_tel::cli;

fn main() -> anyhow::Result<()> {
    cli::run_cli()
} 