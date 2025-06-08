//! Tests for cross-chain deployment commands

use anyhow::Result;
use std::time::Duration;
use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

pub async fn run_deploy_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Basic help tests
    for (name, cmd, subcmd) in [
        ("deploy_help", "deploy", "--help"),
        ("deploy_simulate_help", "deploy", "simulate"),
        ("deploy_submit_help", "deploy", "submit"), 
        ("deploy_report_help", "deploy", "report"),
    ] {
        let test = cmd_test!(name, "deploy", subcmd, "--help")
            .expect_exit_code(0)
            .with_timeout(Duration::from_secs(10));
        let mut result = runner.run_command_test(test).await?;
        result.category = "deploy".to_string();
        results.push(result);
    }

    // Create test files
    runner.create_test_file("test.ir", "mock IR")?;
    runner.create_test_file("circuit.zk", "mock circuit")?;
    runner.create_test_file("proof.zk", "mock proof")?;

    // Test deployment commands - these work with mock functionality
    let test = cmd_test!("deploy_simulate_chains", "deploy", "simulate", "--input", "test.ir", "--chains", "ethereum,polygon")
        .expect_exit_code(0)
        .expect_stdout("Total gas cost")
        .with_timeout(Duration::from_secs(30));
    let mut result = runner.run_command_test(test).await?;
    result.category = "deploy".to_string();
    results.push(result);

    Ok(results)
} 