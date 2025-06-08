//! Tests for analysis and diagnostics commands

use anyhow::Result;
use std::time::Duration;
use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

pub async fn run_analyze_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Help tests for all analyze subcommands
    for (name, subcmd) in [
        ("analyze_help", "--help"),
        ("analyze_code_help", "code"),
        ("analyze_resources_help", "resources"),
        ("analyze_effects_help", "effects"),
        ("analyze_security_help", "security"),
    ] {
        let test = cmd_test!(name, "analyze", subcmd, "--help")
            .expect_exit_code(0)
            .with_timeout(Duration::from_secs(10));
        let mut result = runner.run_command_test(test).await?;
        result.category = "analyze".to_string();
        results.push(result);
    }

    // Test alias
    let test = cmd_test!("analyze_alias", "a", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "analyze".to_string();
    results.push(result);

    // Create test files
    runner.create_test_file("src/main.lisp", "(define main (lambda (x) x))")?;

    // Test analysis commands - these work with mock functionality
    let test = cmd_test!("analyze_code_basic", "analyze", "code", ".")
        .expect_exit_code(0)
        .expect_stdout("Analysis complete")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "analyze".to_string();
    results.push(result);

    let test = cmd_test!("analyze_resources_basic", "analyze", "resources", "-f", "test.lisp")
        .expect_exit_code(0)
        .expect_stdout("Analysis complete")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "analyze".to_string();
    results.push(result);

    let test = cmd_test!("analyze_effects_basic", "analyze", "effects", "-f", "test.lisp")
        .expect_exit_code(0)
        .expect_stdout("Analysis complete")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "analyze".to_string();
    results.push(result);

    let test = cmd_test!("analyze_security_basic", "analyze", "security", ".")
        .expect_exit_code(0)
        .expect_stdout("Analysis complete")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "analyze".to_string();
    results.push(result);

    Ok(results)
} 