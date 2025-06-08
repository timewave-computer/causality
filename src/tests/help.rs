//! Help system tests - Validates all help commands and topics

use anyhow::Result;
use std::time::Duration;

use crate::test_utils::{TestRunner, TestResult, CommandTest};
use crate::{cmd_test, help_test};

/// Test basic help functionality
pub async fn run_help_tests() -> Result<Vec<TestResult>> {
    let runner = TestRunner::new(Duration::from_secs(10))?;
    let mut results = Vec::new();

    // Basic help commands
    results.push(runner.run_test(&cmd_test!("help_basic", "help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Commands:")).await);

    results.push(runner.run_test(&cmd_test!("help_short_flag", "-h")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Commands:")).await);

    results.push(runner.run_test(&cmd_test!("help_long_flag", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Gateway to the Causality Framework")).await);

    // Help topics (these might not be implemented yet, so expect failure)
    results.push(runner.run_test(&cmd_test!("help_tutorial", "help", "tutorial")
        .skip_reason("Help topics not implemented yet")).await);

    results.push(runner.run_test(&cmd_test!("help_guides", "help", "guides")
        .skip_reason("Help topics not implemented yet")).await);

    results.push(runner.run_test(&cmd_test!("help_reference", "help", "reference")
        .skip_reason("Help topics not implemented yet")).await);

    results.push(runner.run_test(&cmd_test!("help_examples", "help", "examples")
        .skip_reason("Help topics not implemented yet")).await);

    results.push(runner.run_test(&cmd_test!("help_api", "help", "api")
        .skip_reason("Help topics not implemented yet")).await);

    results.push(runner.run_test(&cmd_test!("help_troubleshooting", "help", "troubleshooting")
        .skip_reason("Help topics not implemented yet")).await);

    // Command-specific help
    results.push(runner.run_test(&cmd_test!("help_repl_command", "repl", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Interactive development environment")).await);

    results.push(runner.run_test(&cmd_test!("help_project_command", "project", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Project management")).await);

    results.push(runner.run_test(&cmd_test!("help_dev_command", "dev", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Development workflow")).await);

    results.push(runner.run_test(&cmd_test!("help_zk_command", "zk", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Zero-knowledge proof")).await);

    results.push(runner.run_test(&cmd_test!("help_deploy_command", "deploy", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Cross-chain deployment")).await);

    results.push(runner.run_test(&cmd_test!("help_analyze_command", "analyze", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Analysis and diagnostics")).await);

    results.push(runner.run_test(&cmd_test!("help_test_command", "test", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Testing and validation")).await);

    results.push(runner.run_test(&cmd_test!("help_inspect_command", "inspect", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("System inspection")).await);

    results.push(runner.run_test(&cmd_test!("help_viz_command", "viz", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Visualization")).await);

    results.push(runner.run_test(&cmd_test!("help_config_command", "config", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Configuration")).await);

    // Subcommand help
    results.push(runner.run_test(&cmd_test!("help_project_new", "project", "new", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Create a new")).await);

    results.push(runner.run_test(&cmd_test!("help_project_build", "project", "build", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Build")).await);

    results.push(runner.run_test(&cmd_test!("help_project_status", "project", "status", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("status")).await);

    results.push(runner.run_test(&cmd_test!("help_dev_compile", "dev", "compile", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Compile")).await);

    results.push(runner.run_test(&cmd_test!("help_dev_run", "dev", "run", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Execute")).await);

    results.push(runner.run_test(&cmd_test!("help_dev_serve", "dev", "serve", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("development server")).await);

    results.push(runner.run_test(&cmd_test!("help_zk_compile", "zk", "compile", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("circuit")).await);

    results.push(runner.run_test(&cmd_test!("help_zk_prove", "zk", "prove", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("proof")).await);

    results.push(runner.run_test(&cmd_test!("help_zk_verify", "zk", "verify", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Verify")).await);

    results.push(runner.run_test(&cmd_test!("help_deploy_simulate", "deploy", "simulate", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Simulate")).await);

    results.push(runner.run_test(&cmd_test!("help_deploy_submit", "deploy", "submit", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Submit")).await);

    results.push(runner.run_test(&cmd_test!("help_analyze_code", "analyze", "code", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("analysis")).await);

    results.push(runner.run_test(&cmd_test!("help_analyze_resources", "analyze", "resources", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("resource")).await);

    results.push(runner.run_test(&cmd_test!("help_test_unit", "test", "unit", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("unit")).await);

    results.push(runner.run_test(&cmd_test!("help_test_e2e", "test", "e2e", "--help")
        .expect_exit_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("End-to-end")).await);

    // Invalid topic should return error
    results.push(runner.run_test(&cmd_test!("help_invalid_topic", "help", "nonexistent")
        .expect_exit_code(2)  // clap uses exit code 2 for usage errors
        .expect_stderr_contains("error")).await);

    Ok(results)
} 