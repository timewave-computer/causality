//! Tests for visualization commands

use anyhow::Result;
use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

pub async fn run_viz_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    let test = cmd_test!("viz_help", "viz", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "viz".to_string();
    results.push(result);

    let test = cmd_test!("viz_alias", "v", "--help")
        .expect_exit_code(0);
    let mut result = runner.run_command_test(test).await?;
    result.category = "viz".to_string();
    results.push(result);

    Ok(results)
} 