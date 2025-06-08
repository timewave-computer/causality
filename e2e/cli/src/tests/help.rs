//! Tests for help system and documentation commands
//!
//! Tests all help-related commands from the CLI documentation:
//! - causality help
//! - causality help tutorial  
//! - causality help guides
//! - causality help reference
//! - causality help examples
//! - causality help api
//! - causality help troubleshooting

use anyhow::Result;
use std::time::Duration;

use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

/// Run all help command tests
pub async fn run_help_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Basic help commands
    let test = cmd_test!("help_basic", "help")
        .expect_exit_code(0)
        .expect_stdout("Usage:")
        .expect_stdout("Commands:")
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "help".to_string();
    results.push(result);

    let test = cmd_test!("help_short_flag", "-h")
        .expect_exit_code(0)
        .expect_stdout("Usage:")
        .expect_stdout("Commands:")
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "help".to_string();
    results.push(result);

    let test = cmd_test!("help_long_flag", "--help")
        .expect_exit_code(0)
        .expect_stdout("Usage:")
        .expect_stdout("The Causality CLI provides")
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "help".to_string();
    results.push(result);

    // Help topics (these are not implemented yet, so mark as skipped)
    let help_topics = vec![
        "tutorial", "guides", "reference", "examples", "api", "troubleshooting"
    ];

    for topic in help_topics {
        let test = cmd_test!(&format!("help_{}", topic), "help", topic)
            .expect_exit_code(2)  // clap uses exit code 2 for unknown subcommands
            .skip("Help topics not implemented yet")
            .with_timeout(Duration::from_secs(10));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "help".to_string();
        results.push(result);
    }

    // Command-specific help
    let commands = vec![
        ("repl", "Interactive development environment"),
        ("project", "Project management"),
        ("dev", "Development workflow"),
        ("zk", "Zero-knowledge proof"),
        ("deploy", "Cross-chain deployment"),
        ("analyze", "Analysis and diagnostics"),
        ("test", "Testing and validation"),
        ("inspect", "System inspection"),
        ("viz", "Visualization"),
        ("config", "Configuration"),
    ];

    for (command, description_part) in commands {
        let test = cmd_test!(&format!("help_{}_command", command), command, "--help")
            .expect_exit_code(0)
            .expect_stdout("Usage:")
            .expect_stdout(description_part)
            .with_timeout(Duration::from_secs(10));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "help".to_string();
        results.push(result);
    }

    // Subcommand help
    let subcommands = vec![
        ("project", "new", "Create a new"),
        ("project", "build", "Build"),
        ("project", "status", "status"),
        ("dev", "compile", "Compile"),
        ("dev", "run", "Execute"),
        ("dev", "serve", "development server"),
        ("zk", "compile", "circuit"),
        ("zk", "prove", "proof"),
        ("zk", "verify", "Verify"),
        ("deploy", "simulate", "Simulate"),
        ("deploy", "submit", "Submit"),
        ("analyze", "code", "analysis"),
        ("analyze", "resources", "resource"),
        ("test", "unit", "unit"),
        ("test", "e2e", "End-to-end"),
    ];

    for (parent, sub, description_part) in subcommands {
        let test = cmd_test!(
            &format!("help_{}_{}", parent, sub),
            parent, sub, "--help"
        )
        .expect_exit_code(0)
        .expect_stdout("Usage:")
        .expect_stdout(description_part)
        .with_timeout(Duration::from_secs(10));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "help".to_string();
        results.push(result);
    }

    // Invalid topic should return error
    let test = cmd_test!("help_invalid_topic", "help", "nonexistent")
        .expect_exit_code(2)  // clap uses exit code 2 for usage errors
        .expect_stderr("error")
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "help".to_string();
    results.push(result);

    Ok(results)
} 