//! Tests for zero-knowledge proof commands
//!
//! Tests all ZK-related commands from the CLI documentation:
//! - causality zk compile -i <input> -o <output>
//! - causality zk prove -c <circuit> -w <witness> -o <output>
//! - causality zk verify -c <circuit> -p <proof>
//! - causality zk setup -c <circuit> -o <output_dir>

use anyhow::Result;
use std::time::Duration;

use crate::test_utils::{CommandTest, TestResult, TestRunner};
use crate::cmd_test;

/// Run all zero-knowledge proof command tests
pub async fn run_zk_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Test zk command help
    let test = cmd_test!("zk_help", "zk", "--help")
        .expect_stdout("Zero-knowledge proof")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk compile help
    let test = cmd_test!("zk_compile_help", "zk", "compile", "--help")
        .expect_stdout("Compile code to ZK circuit")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk compile alias
    let test = cmd_test!("zk_compile_alias", "zk", "c", "--help")
        .expect_stdout("Compile code to ZK circuit")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Create test IR file
    runner.create_test_file("test.ir", "mock intermediate representation")?;

    // Test zk compile with different privacy levels - these work with mock functionality
    let privacy_levels = vec!["low", "medium", "high", "maximum"];
    
    for level in privacy_levels {
        let test = cmd_test!(
            &format!("zk_compile_privacy_{}", level),
            "zk", "compile",
            "--input", "test.ir",
            "--output", &format!("test_{}.zk", level),
            "--privacy-level", level
        )
        .expect_exit_code(0)
        .expect_stdout("ZK circuit compiled successfully")
        .with_timeout(Duration::from_secs(60));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "zk".to_string();
        results.push(result);
    }

    // Test zk compile with different proof systems - these work with mock functionality
    let proof_systems = vec!["groth16", "plonk", "stark", "marlin"];
    
    for system in proof_systems {
        let test = cmd_test!(
            &format!("zk_compile_proof_system_{}", system),
            "zk", "compile",
            "--input", "test.ir",
            "--output", &format!("test_{}.zk", system),
            "--proof-system", system
        )
        .expect_exit_code(0)
        .expect_stdout("ZK circuit compiled successfully")
        .with_timeout(Duration::from_secs(60));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "zk".to_string();
        results.push(result);
    }

    // Test zk compile with stats - this works with mock functionality
    let test = cmd_test!(
        "zk_compile_stats",
        "zk", "compile",
        "--input", "test.ir",
        "--output", "test_stats.zk",
        "--stats"
    )
    .expect_exit_code(0)
    .expect_stdout("ZK circuit compiled successfully")
    .with_timeout(Duration::from_secs(60));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk prove help
    let test = cmd_test!("zk_prove_help", "zk", "prove", "--help")
        .expect_stdout("Generate ZK proof")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Create test circuit and witness files
    runner.create_test_file("test.zk", "mock circuit")?;
    runner.create_test_file("witness.json", r#"{"input": 42}"#)?;
    runner.create_test_file("public_inputs.json", r#"{"output": 43}"#)?;

    // Test zk prove - this works with mock functionality
    let test = cmd_test!(
        "zk_prove_basic",
        "zk", "prove",
        "--circuit", "test.zk",
        "--witness", "witness.json",
        "--output", "proof.zk"
    )
    .expect_exit_code(0)
    .expect_stdout("ZK proof generated successfully")
    .with_timeout(Duration::from_secs(120));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk verify help
    let test = cmd_test!("zk_verify_help", "zk", "verify", "--help")
        .expect_stdout("Verify ZK proof")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk verify - this works with mock functionality
    let test = cmd_test!(
        "zk_verify_basic",
        "zk", "verify",
        "--circuit", "test.zk",
        "--proof", "proof.zk"
    )
    .expect_exit_code(0)
    .expect_stdout("ZK proof verification successful")
    .with_timeout(Duration::from_secs(60));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk verify with public inputs - this works with mock functionality
    let test = cmd_test!(
        "zk_verify_with_inputs",
        "zk", "verify",
        "--circuit", "test.zk",
        "--proof", "proof.zk",
        "--public-inputs", "public_inputs.json"
    )
    .expect_exit_code(0)
    .expect_stdout("ZK proof verification successful")
    .with_timeout(Duration::from_secs(60));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk verify with mock runtime - this works with mock functionality
    let test = cmd_test!(
        "zk_verify_mock",
        "zk", "verify",
        "--circuit", "test.zk",
        "--proof", "proof.zk",
        "--mock"
    )
    .expect_exit_code(0)
    .expect_stdout("ZK proof verification successful")
    .with_timeout(Duration::from_secs(30));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk setup help
    let test = cmd_test!("zk_setup_help", "zk", "setup", "--help")
        .expect_stdout("Setup trusted setup ceremony")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk setup - this works with mock functionality
    let test = cmd_test!(
        "zk_setup_basic",
        "zk", "setup",
        "--circuit", "test.zk",
        "--output-dir", "setup_output"
    )
    .expect_exit_code(0)
    .expect_stdout("Trusted setup completed")
    .with_timeout(Duration::from_secs(120));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    // Test zk setup with multiple participants - this works with mock functionality
    let test = cmd_test!(
        "zk_setup_multi_participants",
        "zk", "setup",
        "--circuit", "test.zk",
        "--output-dir", "setup_multi",
        "--participants", "3"
    )
    .expect_exit_code(0)
    .expect_stdout("Trusted setup completed")
    .with_timeout(Duration::from_secs(180));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "zk".to_string();
    results.push(result);

    Ok(results)
} 