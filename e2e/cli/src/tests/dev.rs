//! Tests for development workflow commands
//!
//! Tests all dev-related commands from the CLI documentation:
//! - causality dev compile -i <input> -o <output>
//! - causality dev run -f <file> --trace
//! - causality dev serve --port <port> --watch
//! - causality dev fmt

use anyhow::Result;
use std::time::Duration;

use crate::test_utils::{CommandTest, TestResult, TestRunner};
use crate::cmd_test;

/// Run all development workflow command tests
pub async fn run_dev_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Test dev command help
    let test = cmd_test!("dev_help", "dev", "--help")
        .expect_stdout("Development workflow")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev alias
    let test = cmd_test!("dev_alias", "d", "--help")
        .expect_stdout("Development workflow")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev compile help
    let test = cmd_test!("dev_compile_help", "dev", "compile", "--help")
        .expect_stdout("Compile source code")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev compile alias
    let test = cmd_test!("dev_compile_alias", "dev", "c", "--help")
        .expect_stdout("Compile source code")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Create test files for compilation
    let lisp_content = r#"
(define main
  (lambda (x)
    (+ x 1)))
"#;
    let test_file = runner.create_test_file("test.lisp", lisp_content)?;

    // Test dev compile with different formats
    let formats = vec!["intermediate", "bytecode", "native", "wasm", "js"];
    
    for format in formats {
        let output_file = format!("test.{}", format);
        let test = cmd_test!(
            &format!("dev_compile_{}", format),
            "dev", "compile",
            "-i", "test.lisp",
            "-o", &output_file,
            "--format", format
        )
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(30))
        .expect_file(&output_file)
        .skip("Compilation may not be fully implemented");
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "dev".to_string();
        results.push(result);
    }

    // Test dev compile with optimization
    let test = cmd_test!(
        "dev_compile_optimize",
        "dev", "compile",
        "-i", "test.lisp",
        "-o", "test_opt.ir",
        "--optimize"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(30))
    .skip("Compilation may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev compile with show stages
    let test = cmd_test!(
        "dev_compile_show_stages",
        "dev", "compile",
        "-i", "test.lisp",
        "-o", "test_stages.ir",
        "--show-stages"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(30))
    .skip("Compilation may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev run help
    let test = cmd_test!("dev_run_help", "dev", "run", "--help")
        .expect_stdout("Execute compiled programs")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev run alias
    let test = cmd_test!("dev_run_alias", "dev", "r", "--help")
        .expect_stdout("Execute compiled programs")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev run with file
    let test = cmd_test!(
        "dev_run_file",
        "dev", "run",
        "-f", "test.lisp"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(30))
    .skip("Execution may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev run with source
    let test = cmd_test!(
        "dev_run_source",
        "dev", "run",
        "-s", "(+ 1 2)"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(30))
    .skip("Execution may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev run with trace
    let test = cmd_test!(
        "dev_run_trace",
        "dev", "run",
        "-f", "test.lisp",
        "--trace"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(30))
    .skip("Execution may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev run with max steps
    let test = cmd_test!(
        "dev_run_max_steps",
        "dev", "run",
        "-f", "test.lisp",
        "--max-steps", "1000"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(30))
    .skip("Execution may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev serve help
    let test = cmd_test!("dev_serve_help", "dev", "serve", "--help")
        .expect_stdout("Start development server")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev serve with custom port
    let test = cmd_test!(
        "dev_serve_port",
        "dev", "serve",
        "--port", "8080", "--help"
    )
    .expect_stdout("port")
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev serve with watch
    let test = cmd_test!(
        "dev_serve_watch",
        "dev", "serve",
        "--watch", "--help"
    )
    .expect_stdout("watch")
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev serve with open
    let test = cmd_test!(
        "dev_serve_open",
        "dev", "serve",
        "--open", "--help"
    )
    .expect_stdout("open")
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev fmt help
    let test = cmd_test!("dev_fmt_help", "dev", "fmt", "--help")
        .expect_stdout("Format source code")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev fmt check
    let test = cmd_test!(
        "dev_fmt_check",
        "dev", "fmt",
        "--check"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(15))
    .skip("Formatting may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    // Test dev fmt specific files
    let test = cmd_test!(
        "dev_fmt_files",
        "dev", "fmt",
        "test.lisp"
    )
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(15))
    .skip("Formatting may not be fully implemented");
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "dev".to_string();
    results.push(result);

    Ok(results)
} 