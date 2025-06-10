//! Tests for system inspection commands

use anyhow::Result;
use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

pub async fn run_inspect_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    let test = cmd_test!("inspect_help", "inspect", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "inspect".to_string();
    results.push(result);

    let test = cmd_test!("inspect_alias", "i", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "inspect".to_string();
    results.push(result);

    Ok(results)
} 