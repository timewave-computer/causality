//! Comprehensive CLI End-to-End Test Runner
//!
//! This test runner validates every CLI command documented in the README files
//! and documentation, ensuring the CLI functionality works correctly end-to-end.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn, debug};

mod tests;
mod test_utils;

use test_utils::{TestEnvironment, TestResult, TestRunner, CommandTest};

/// Configuration for the test runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Maximum time allowed for any single test
    pub test_timeout_seconds: u64,
    /// Whether to run tests in parallel
    pub parallel_execution: bool,
    /// Maximum number of parallel tests
    pub max_parallel_tests: usize,
    /// Whether to continue on failures
    pub continue_on_failure: bool,
    /// Test environment settings
    pub environment: EnvironmentConfig,
    /// CLI binary path (if not in PATH)
    pub cli_binary_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Temporary directory for test artifacts
    pub temp_dir: Option<PathBuf>,
    /// Whether to clean up test artifacts
    pub cleanup_artifacts: bool,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

/// Comprehensive test suite results
#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteResults {
    /// Overall test run metadata
    pub metadata: TestRunMetadata,
    /// Results by test category
    pub category_results: HashMap<String, CategoryResults>,
    /// Individual test results
    pub test_results: Vec<TestResult>,
    /// Summary statistics
    pub summary: TestSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestRunMetadata {
    /// When the test run started
    pub start_time: DateTime<Utc>,
    /// When the test run completed
    pub end_time: Option<DateTime<Utc>>,
    /// Total runtime
    pub duration: Option<Duration>,
    /// Environment information
    pub environment: EnvironmentInfo,
    /// Test configuration used
    pub config: TestConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Causality CLI version
    pub cli_version: Option<String>,
    /// Available tools and versions
    pub tools: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryResults {
    /// Category name
    pub name: String,
    /// Number of tests passed
    pub passed: usize,
    /// Number of tests failed
    pub failed: usize,
    /// Number of tests skipped
    pub skipped: usize,
    /// Category runtime
    pub duration: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total tests run
    pub total_tests: usize,
    /// Tests passed
    pub passed: usize,
    /// Tests failed
    pub failed: usize,
    /// Tests skipped
    pub skipped: usize,
    /// Success rate as percentage
    pub success_rate: f64,
    /// Total runtime
    pub total_duration: Duration,
    /// Average test time
    pub avg_test_time: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_timeout_seconds: 120, // 2 minutes per test
            parallel_execution: true,
            max_parallel_tests: 4,
            continue_on_failure: true,
            environment: EnvironmentConfig {
                temp_dir: None,
                cleanup_artifacts: true,
                env_vars: HashMap::new(),
            },
            cli_binary_path: None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Starting Causality CLI End-to-End Test Suite");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args)?;

    // Set up test environment
    let env = TestEnvironment::new(config.clone()).await?;
    
    // Initialize test runner
    let mut runner = TestRunner::new(env);

    // Run comprehensive test suite
    let results = run_comprehensive_test_suite(&mut runner).await?;

    // Generate reports
    generate_reports(&results, &config).await?;

    // Exit with appropriate code
    let exit_code = if results.summary.failed > 0 { 1 } else { 0 };
    std::process::exit(exit_code);
}

/// Parse command line arguments and create test configuration
fn parse_args(args: &[String]) -> Result<TestConfig> {
    let mut config = TestConfig::default();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--timeout" => {
                i += 1;
                if i < args.len() {
                    config.test_timeout_seconds = args[i].parse()
                        .context("Invalid timeout value")?;
                }
            }
            "--no-parallel" => {
                config.parallel_execution = false;
            }
            "--max-parallel" => {
                i += 1;
                if i < args.len() {
                    config.max_parallel_tests = args[i].parse()
                        .context("Invalid max-parallel value")?;
                }
            }
            "--fail-fast" => {
                config.continue_on_failure = false;
            }
            "--cli-binary" => {
                i += 1;
                if i < args.len() {
                    config.cli_binary_path = Some(PathBuf::from(&args[i]));
                }
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                warn!("Unknown argument: {}", args[i]);
            }
        }
        i += 1;
    }

    Ok(config)
}

/// Print help message
fn print_help() {
    println!(r#"
Causality CLI End-to-End Test Runner

USAGE:
    causality-cli-e2e [OPTIONS]

OPTIONS:
    --timeout <SECONDS>         Maximum time per test (default: 120)
    --no-parallel              Disable parallel test execution
    --max-parallel <COUNT>     Maximum parallel tests (default: 4)
    --fail-fast                Stop on first failure
    --cli-binary <PATH>        Path to causality CLI binary
    --help                     Show this help message

EXAMPLES:
    # Run with default settings
    causality-cli-e2e

    # Run with custom timeout and binary path
    causality-cli-e2e --timeout 60 --cli-binary ./target/debug/causality

    # Run sequentially with fail-fast
    causality-cli-e2e --no-parallel --fail-fast
"#);
}

/// Run the comprehensive test suite covering all CLI commands
async fn run_comprehensive_test_suite(runner: &mut TestRunner) -> Result<TestSuiteResults> {
    let start_time = Utc::now();
    info!("📋 Executing comprehensive CLI test suite");

    let mut results = TestSuiteResults {
        metadata: TestRunMetadata {
            start_time,
            end_time: None,
            duration: None,
            environment: gather_environment_info().await?,
            config: runner.config().clone(),
        },
        category_results: HashMap::new(),
        test_results: Vec::new(),
        summary: TestSummary {
            total_tests: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            success_rate: 0.0,
            total_duration: Duration::from_secs(0),
            avg_test_time: Duration::from_secs(0),
        },
    };

    // Define test categories and their corresponding test modules
    let test_categories = vec![
        ("repl", "REPL and Interactive Development"),
        ("help", "Help System and Documentation"),
        ("project", "Project Management"),
        ("dev", "Development Workflow"),
        ("zk", "Zero-Knowledge Operations"),
        ("deploy", "Cross-Chain Deployment"),
        ("analyze", "Analysis and Diagnostics"),
        ("test", "Testing and Validation"),
        ("inspect", "System Inspection"),
        ("viz", "Visualization"),
        ("config", "Configuration Management"),
    ];

    for (category_name, category_desc) in test_categories {
        info!("🔍 Testing category: {}", category_desc);
        
        let category_start = Instant::now();
        let category_results = run_category_tests(runner, category_name).await?;
        let category_duration = category_start.elapsed();

        // Aggregate category results
        let passed = category_results.iter().filter(|r| r.passed).count();
        let failed = category_results.iter().filter(|r| !r.passed && !r.skipped).count();
        let skipped = category_results.iter().filter(|r| r.skipped).count();

        results.category_results.insert(category_name.to_string(), CategoryResults {
            name: category_name.to_string(),
            passed,
            failed,
            skipped,
            duration: category_duration,
        });

        results.test_results.extend(category_results);

        if !runner.config().continue_on_failure && failed > 0 {
            warn!("🛑 Stopping test suite due to failures in category: {}", category_name);
            break;
        }
    }

    // Calculate final summary
    let end_time = Utc::now();
    let total_duration = end_time.signed_duration_since(start_time)
        .to_std()
        .unwrap_or(Duration::from_secs(0));

    results.metadata.end_time = Some(end_time);
    results.metadata.duration = Some(total_duration);

    results.summary.total_tests = results.test_results.len();
    results.summary.passed = results.test_results.iter().filter(|r| r.passed).count();
    results.summary.failed = results.test_results.iter().filter(|r| !r.passed && !r.skipped).count();
    results.summary.skipped = results.test_results.iter().filter(|r| r.skipped).count();
    results.summary.success_rate = if results.summary.total_tests > 0 {
        (results.summary.passed as f64 / results.summary.total_tests as f64) * 100.0
    } else {
        0.0
    };
    results.summary.total_duration = total_duration;
    results.summary.avg_test_time = if results.summary.total_tests > 0 {
        total_duration / results.summary.total_tests as u32
    } else {
        Duration::from_secs(0)
    };

    info!("✅ Test suite completed: {}/{} tests passed ({:.1}%)", 
          results.summary.passed, 
          results.summary.total_tests, 
          results.summary.success_rate);

    Ok(results)
}

/// Run tests for a specific category
async fn run_category_tests(runner: &mut TestRunner, category: &str) -> Result<Vec<TestResult>> {
    match category {
        "repl" => tests::repl::run_repl_tests(runner).await,
        "help" => tests::help::run_help_tests(runner).await,
        "project" => tests::project::run_project_tests(runner).await,
        "dev" => tests::dev::run_dev_tests(runner).await,
        "zk" => tests::zk::run_zk_tests(runner).await,
        "deploy" => tests::deploy::run_deploy_tests(runner).await,
        "analyze" => tests::analyze::run_analyze_tests(runner).await,
        "test" => tests::test_commands::run_test_command_tests(runner).await,
        "inspect" => tests::inspect::run_inspect_tests(runner).await,
        "viz" => tests::viz::run_viz_tests(runner).await,
        "config" => tests::config::run_config_tests(runner).await,
        _ => {
            warn!("Unknown test category: {}", category);
            Ok(vec![])
        }
    }
}

/// Gather information about the test environment
async fn gather_environment_info() -> Result<EnvironmentInfo> {
    let mut info = EnvironmentInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cli_version: None,
        tools: HashMap::new(),
    };

    // Try to get CLI version
    if let Ok(output) = Command::new("causality").args(["--version"]).output() {
        if output.status.success() {
            info.cli_version = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    // Check for required tools
    let tools_to_check = vec!["cargo", "rustc", "git", "dune", "ocaml"];
    for tool in tools_to_check {
        if let Ok(output) = Command::new(tool).args(["--version"]).output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .trim()
                    .to_string();
                info.tools.insert(tool.to_string(), version);
            }
        }
    }

    Ok(info)
}

/// Generate comprehensive test reports
async fn generate_reports(results: &TestSuiteResults, config: &TestConfig) -> Result<()> {
    info!("📊 Generating test reports");

    // Console report
    print_console_report(results);

    // JSON report
    let json_report = serde_json::to_string_pretty(results)
        .context("Failed to serialize JSON report")?;
    
    std::fs::write("test-results.json", json_report)
        .context("Failed to write JSON report")?;

    // Generate Markdown report
    generate_markdown_report(results).await?;

    info!("📄 Reports generated:");
    info!("  • Console output (above)");
    info!("  • test-results.json");
    info!("  • test-results.md");

    Ok(())
}

/// Print a formatted console report
fn print_console_report(results: &TestSuiteResults) {
    println!("\n{}", "=".repeat(80));
    println!("🧪 CAUSALITY CLI E2E TEST RESULTS");
    println!("{}", "=".repeat(80));

    // Summary
    println!("\n📊 SUMMARY:");
    println!("  Total Tests:  {}", results.summary.total_tests);
    println!("  Passed:       {} ({:.1}%)", 
             results.summary.passed, 
             if results.summary.total_tests > 0 { 
                 (results.summary.passed as f64 / results.summary.total_tests as f64) * 100.0 
             } else { 0.0 });
    println!("  Failed:       {}", results.summary.failed);
    println!("  Skipped:      {}", results.summary.skipped);
    println!("  Duration:     {:.2?}", results.summary.total_duration);
    println!("  Avg/Test:     {:.2?}", results.summary.avg_test_time);

    // Category breakdown
    println!("\n📋 BY CATEGORY:");
    for (name, category) in &results.category_results {
        let total = category.passed + category.failed + category.skipped;
        let rate = if total > 0 { 
            (category.passed as f64 / total as f64) * 100.0 
        } else { 0.0 };
        
        println!("  {:12} {:3}/{} ({:5.1}%) [{:.2?}]", 
                 name, category.passed, total, rate, category.duration);
    }

    // Failed tests
    let failed_tests: Vec<_> = results.test_results.iter()
        .filter(|r| !r.passed && !r.skipped)
        .collect();
    
    if !failed_tests.is_empty() {
        println!("\n❌ FAILED TESTS:");
        for test in failed_tests {
            println!("  • {} - {}", test.name, test.error.as_deref().unwrap_or("Unknown error"));
        }
    }

    println!("\n{}", "=".repeat(80));
}

/// Generate a Markdown report
async fn generate_markdown_report(results: &TestSuiteResults) -> Result<()> {
    let mut content = String::new();
    
    // Header
    content.push_str("# Causality CLI E2E Test Results\n\n");
    content.push_str(&format!("**Generated:** {}\n\n", 
        results.metadata.start_time.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Summary
    content.push_str("## 📊 Summary\n\n");
    content.push_str("| Metric | Value |\n");
    content.push_str("|--------|-------|\n");
    content.push_str(&format!("| Total Tests | {} |\n", results.summary.total_tests));
    content.push_str(&format!("| Passed | {} |\n", results.summary.passed));
    content.push_str(&format!("| Failed | {} |\n", results.summary.failed));
    content.push_str(&format!("| Skipped | {} |\n", results.summary.skipped));
    content.push_str(&format!("| Success Rate | {:.1}% |\n", results.summary.success_rate));
    content.push_str(&format!("| Total Duration | {:.2?} |\n", results.summary.total_duration));
    content.push_str(&format!("| Average Test Time | {:.2?} |\n\n", results.summary.avg_test_time));
    
    // Category Results
    content.push_str("## 📋 Results by Category\n\n");
    for (name, category) in &results.category_results {
        let total = category.passed + category.failed + category.skipped;
        let rate = if total > 0 { 
            (category.passed as f64 / total as f64) * 100.0 
        } else { 0.0 };
        
        content.push_str(&format!("### {} ({})\n\n", name.to_uppercase(), name));
        content.push_str(&format!("- **Passed:** {}\n", category.passed));
        content.push_str(&format!("- **Failed:** {}\n", category.failed));
        content.push_str(&format!("- **Skipped:** {}\n", category.skipped));
        content.push_str(&format!("- **Success Rate:** {:.1}%\n", rate));
        content.push_str(&format!("- **Duration:** {:.2?}\n\n", category.duration));
    }
    
    // Failed Tests
    let failed_tests: Vec<_> = results.test_results.iter()
        .filter(|r| !r.passed && !r.skipped)
        .collect();
    
    if !failed_tests.is_empty() {
        content.push_str("## ❌ Failed Tests\n\n");
        for test in failed_tests {
            content.push_str(&format!("- **{}** - {}\n", 
                test.name, 
                test.error.as_deref().unwrap_or("Unknown error")));
        }
        content.push_str("\n");
    }
    
    // Environment Information
    content.push_str("## 🔧 Test Environment\n\n");
    let env = &results.metadata.environment;
    content.push_str(&format!("- **OS:** {}\n", env.os));
    content.push_str(&format!("- **Architecture:** {}\n", env.arch));
    content.push_str(&format!("- **CLI Version:** {}\n", 
        env.cli_version.as_deref().unwrap_or("Not available")));
    
    if !env.tools.is_empty() {
        content.push_str("\n### Available Tools\n\n");
        for (tool, version) in &env.tools {
            content.push_str(&format!("- **{}:** {}\n", tool, version));
        }
    }
    
    // All Test Results
    content.push_str("\n## 📝 Detailed Test Results\n\n");
    for (category_name, _) in &results.category_results {
        let category_tests: Vec<_> = results.test_results.iter()
            .filter(|t| t.category == *category_name)
            .collect();
        
        if !category_tests.is_empty() {
            content.push_str(&format!("### {} Tests\n\n", category_name.to_uppercase()));
            content.push_str("| Test Name | Status | Duration | Command |\n");
            content.push_str("|-----------|--------|----------|----------|\n");
            
            for test in category_tests {
                let status = if test.passed {
                    "✅ PASSED"
                } else if test.skipped {
                    "⏭️ SKIPPED" 
                } else {
                    "❌ FAILED"
                };
                
                content.push_str(&format!("| {} | {} | {:.2?} | `{}` |\n",
                    test.name, status, test.duration, test.command));
            }
            content.push_str("\n");
        }
    }

    std::fs::write("test-results.md", content)
        .context("Failed to write Markdown report")?;

    Ok(())
} 