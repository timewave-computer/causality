//! Tests for REPL and interactive development commands
//!
//! Tests all REPL-related commands from the CLI documentation:
//! - causality repl
//! - causality repl --debug
//! - causality repl --show-state
//! - causality repl --load-tutorial <tutorial>
//! - causality repl --auto-save

use anyhow::Result;
use std::time::Duration;

use crate::test_utils::{CommandTest, TestResult, TestRunner};
use crate::cmd_test;

/// Run all REPL command tests
pub async fn run_repl_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Basic REPL startup (with timeout, non-interactive)
    let test = cmd_test!("repl_basic_help", "repl", "--help")
        .expect_stdout("Interactive development environment")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // REPL with debug mode
    let test = cmd_test!("repl_debug_help", "repl", "--debug", "--help")
        .expect_stdout("debug")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // REPL with show state
    let test = cmd_test!("repl_show_state_help", "repl", "--show-state", "--help")
        .expect_stdout("state")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // REPL with max steps
    let test = cmd_test!("repl_max_steps_help", "repl", "--max-steps", "100", "--help")
        .expect_stdout("max-steps")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // Test loading different tutorials (these should work with --help)
    let tutorials = vec!["basic", "effects", "zk", "defi"];
    
    for tutorial in tutorials {
        let test = cmd_test!(
            &format!("repl_load_tutorial_{}", tutorial),
            "repl", "--load-tutorial", tutorial, "--help"
        )
        .expect_exit_code(0)
        .expect_stdout("load-tutorial")
        .with_timeout(Duration::from_secs(15));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "repl".to_string();
        results.push(result);
    }

    // REPL with auto-save
    let test = cmd_test!("repl_auto_save_help", "repl", "--auto-save", "--help")
        .expect_stdout("auto-save")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // Test REPL alias
    let test = cmd_test!("repl_alias", "r", "--help")
        .expect_stdout("Interactive")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // Test invalid tutorial
    let test = cmd_test!("repl_invalid_tutorial", "repl", "--load-tutorial", "nonexistent", "--help")
        .expect_exit_code(0) // Help should still work even with invalid tutorial
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // Test combined options
    let test = cmd_test!(
        "repl_combined_options", 
        "repl", "--debug", "--show-state", "--max-steps", "50", "--help"
    )
    .expect_stdout("debug")
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "repl".to_string();
    results.push(result);

    // Note: We don't test actual REPL interaction since that would require
    // sending input to stdin and handling interactive sessions, which is
    // complex for automated testing. The above tests focus on validating
    // command line argument parsing and help output.

    Ok(results)
} 