//! Tests for configuration management commands

use anyhow::Result;
use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

pub async fn run_config_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    let test = cmd_test!("config_help", "config", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "config".to_string();
    results.push(result);

    let test = cmd_test!("config_alias", "c", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "config".to_string();
    results.push(result);

    Ok(results)
} 