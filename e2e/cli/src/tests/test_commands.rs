//! Tests for testing and validation commands

use anyhow::Result;
use std::time::Duration;
use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

pub async fn run_test_command_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Basic help test
    let test = cmd_test!("test_help", "test", "--help")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    // Alias test
    let test = cmd_test!("test_alias", "t", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    // Help tests for test commands
    for (name, subcmd) in [
        ("test_unit_help", "unit"),
        ("test_effects_help", "effects"),
        ("test_integration_help", "integration"),
        ("test_e2e_help", "e2e"),
    ] {
        let test = cmd_test!(name, "test", subcmd, "--help")
            .expect_exit_code(0)
            .with_timeout(Duration::from_secs(10));
        let mut result = runner.run_command_test(test).await?;
        result.category = "test".to_string();
        results.push(result);
    }

    // Test subcommand aliases
    let test = cmd_test!("test_integration_alias", "test", "int", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    // Test commands with various options - these work with mock functionality
    let test = cmd_test!("test_unit_basic", "test", "unit")
        .expect_exit_code(0)
        .expect_stdout("All tests passed")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    let test = cmd_test!("test_unit_coverage", "test", "unit", "--coverage")
        .expect_exit_code(0)
        .expect_stdout("All tests passed")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    let test = cmd_test!("test_effects_property", "test", "effects", "--property-based")
        .expect_exit_code(0)
        .expect_stdout("All tests passed")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    let test = cmd_test!("test_e2e_chains", "test", "e2e", "--chains", "ethereum,polygon")
        .expect_exit_code(0)
        .expect_stdout("All tests passed")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "test".to_string();
    results.push(result);

    Ok(results)
} 