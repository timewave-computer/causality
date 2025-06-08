//! Test utilities for the Causality CLI End-to-End Test Runner
//!
//! Provides testing framework, environment management, and helper utilities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use assert_cmd::Command as AssertCommand;
use assert_fs::TempDir;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::{TestConfig, EnvironmentConfig};

/// Test environment for isolated test execution
pub struct TestEnvironment {
    /// Test configuration
    pub config: TestConfig,
    /// Temporary directory for test artifacts
    pub temp_dir: TempDir,
    /// Working directory for tests
    pub work_dir: PathBuf,
    /// CLI binary path
    pub cli_binary: PathBuf,
}

impl TestEnvironment {
    pub async fn new(config: TestConfig) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let work_dir = temp_dir.path().to_path_buf();
        
        // Determine CLI binary path
        let cli_binary = if let Some(ref binary_path) = config.cli_binary_path {
            binary_path.clone()
        } else {
            PathBuf::from("causality")
        };

        Ok(Self {
            config,
            temp_dir,
            work_dir,
            cli_binary,
        })
    }

    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn work_path(&self) -> &Path {
        &self.work_dir
    }
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test category
    pub category: String,
    /// Whether the test passed
    pub passed: bool,
    /// Whether the test was skipped
    pub skipped: bool,
    /// Test execution time
    pub duration: Duration,
    /// Command that was executed
    pub command: String,
    /// Standard output
    pub stdout: Option<String>,
    /// Standard error
    pub stderr: Option<String>,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Individual command test definition
#[derive(Debug, Clone)]
pub struct CommandTest {
    /// Test name
    pub name: String,
    /// Command and arguments to execute
    pub args: Vec<String>,
    /// Expected exit code (0 = success, 1 = error, etc.)
    pub expected_exit_code: Option<i32>,
    /// Whether to skip this test
    pub skip: bool,
    /// Reason for skipping
    pub skip_reason: Option<String>,
    /// Test timeout override
    pub timeout: Option<Duration>,
    /// Working directory override
    pub work_dir: Option<PathBuf>,
    /// Additional environment variables
    pub env_vars: HashMap<String, String>,
}

impl CommandTest {
    pub fn new(name: &str, args: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            args,
            expected_exit_code: Some(0),
            skip: false,
            skip_reason: None,
            timeout: None,
            work_dir: None,
            env_vars: HashMap::new(),
        }
    }

    pub fn skip(mut self, reason: &str) -> Self {
        self.skip = true;
        self.skip_reason = Some(reason.to_string());
        self
    }

    pub fn expect_exit_code(mut self, code: i32) -> Self {
        self.expected_exit_code = Some(code);
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn work_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.work_dir = Some(dir.into());
        self
    }

    pub fn env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }
}

/// Test runner for executing tests
pub struct TestRunner {
    env: TestEnvironment,
}

impl TestRunner {
    pub fn new(env: TestEnvironment) -> Self {
        Self { env }
    }

    pub fn config(&self) -> &TestConfig {
        &self.env.config
    }

    /// Execute a single command test
    pub async fn run_test(&self, test: CommandTest, category: &str) -> Result<TestResult> {
        let start_time = Instant::now();
        
        if test.skip {
            return Ok(TestResult {
                name: test.name.clone(),
                category: category.to_string(),
                passed: false,
                skipped: true,
                duration: Duration::from_secs(0),
                command: test.args.join(" "),
                stdout: None,
                stderr: None,
                exit_code: None,
                error: test.skip_reason,
            });
        }

        let timeout_duration = test.timeout
            .unwrap_or_else(|| Duration::from_secs(self.env.config.test_timeout_seconds));

        let work_dir = test.work_dir
            .as_ref()
            .unwrap_or(&self.env.work_dir);

        let mut cmd = AssertCommand::new(&self.env.cli_binary);
        cmd.args(&test.args[1..]) // Skip the binary name
           .current_dir(work_dir);

        // Add environment variables
        for (key, value) in &test.env_vars {
            cmd.env(key, value);
        }

        // Add test mode environment variables
        cmd.env("CAUSALITY_TEST_MODE", "1");
        cmd.env("CAUSALITY_NO_PROMPT", "1");

        debug!("Executing test: {} with command: {:?}", test.name, test.args);

        let result = match timeout(timeout_duration, tokio::task::spawn_blocking(move || {
            cmd.output()
        })).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(TestResult {
                    name: test.name,
                    category: category.to_string(),
                    passed: false,
                    skipped: false,
                    duration: start_time.elapsed(),
                    command: test.args.join(" "),
                    stdout: None,
                    stderr: None,
                    exit_code: None,
                    error: Some(format!("Command execution failed: {}", e)),
                });
            }
            Err(_) => {
                return Ok(TestResult {
                    name: test.name,
                    category: category.to_string(),
                    passed: false,
                    skipped: false,
                    duration: start_time.elapsed(),
                    command: test.args.join(" "),
                    stdout: None,
                    stderr: None,
                    exit_code: None,
                    error: Some("Test timed out".to_string()),
                });
            }
        };

        let duration = start_time.elapsed();
        let exit_code = result.status.code();
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();

        // Check if test passed based on exit code
        let passed = if let Some(expected) = test.expected_exit_code {
            exit_code == Some(expected)
        } else {
            result.status.success()
        };

        let error = if !passed {
            Some(format!("Expected exit code {:?}, got {:?}", test.expected_exit_code, exit_code))
        } else {
            None
        };

        Ok(TestResult {
            name: test.name,
            category: category.to_string(),
            passed,
            skipped: false,
            duration,
            command: test.args.join(" "),
            stdout: Some(stdout),
            stderr: Some(stderr),
            exit_code,
            error,
        })
    }

    /// Execute multiple tests in parallel or sequence
    pub async fn run_tests(&self, tests: Vec<CommandTest>, category: &str) -> Result<Vec<TestResult>> {
        if self.env.config.parallel_execution {
            self.run_tests_parallel(tests, category).await
        } else {
            self.run_tests_sequential(tests, category).await
        }
    }

    async fn run_tests_parallel(&self, tests: Vec<CommandTest>, category: &str) -> Result<Vec<TestResult>> {
        use futures::stream::{FuturesUnordered, StreamExt};
        
        let max_parallel = self.env.config.max_parallel_tests;
        let mut results = Vec::new();
        let mut futures = FuturesUnordered::new();
        let mut test_iter = tests.into_iter();

        // Fill initial batch
        for _ in 0..max_parallel {
            if let Some(test) = test_iter.next() {
                let runner = self;
                let category = category.to_string();
                futures.push(async move {
                    runner.run_test(test, &category).await
                });
            }
        }

        // Process results and add new tests
        while let Some(result) = futures.next().await {
            results.push(result?);
            
            // Add next test if available
            if let Some(test) = test_iter.next() {
                let runner = self;
                let category = category.to_string();
                futures.push(async move {
                    runner.run_test(test, &category).await
                });
            }

            // Stop on failure if configured
            if !self.env.config.continue_on_failure && 
               results.last().map(|r| !r.passed && !r.skipped).unwrap_or(false) {
                break;
            }
        }

        Ok(results)
    }

    async fn run_tests_sequential(&self, tests: Vec<CommandTest>, category: &str) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        for test in tests {
            let result = self.run_test(test, category).await?;
            
            // Stop on failure if configured
            if !self.env.config.continue_on_failure && !result.passed && !result.skipped {
                results.push(result);
                break;
            }
            
            results.push(result);
        }

        Ok(results)
    }

    /// Create a test file with given content
    pub fn create_test_file(&self, filename: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.env.work_dir.join(filename);
        std::fs::write(&file_path, content)
            .with_context(|| format!("Failed to create test file: {}", filename))?;
        Ok(file_path)
    }

    /// Create a test directory
    pub fn create_test_dir(&self, dirname: &str) -> Result<PathBuf> {
        let dir_path = self.env.work_dir.join(dirname);
        std::fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create test directory: {}", dirname))?;
        Ok(dir_path)
    }

    /// Clean up test artifacts
    pub fn cleanup(&self) -> Result<()> {
        if self.env.config.environment.cleanup_artifacts {
            // TempDir automatically cleans up when dropped
            debug!("Test artifacts will be cleaned up automatically");
        }
        Ok(())
    }
}

/// Helper macros for creating common test patterns

/// Create a basic command test
#[macro_export]
macro_rules! cmd_test {
    ($name:expr, $($arg:expr),*) => {
        $crate::test_utils::CommandTest::new($name, vec!["causality".to_string(), $($arg.to_string()),*])
    };
}

/// Create a help test (expects help output)
#[macro_export]
macro_rules! help_test {
    ($name:expr, $($arg:expr),*) => {
        $crate::test_utils::CommandTest::new($name, vec!["causality".to_string(), $($arg.to_string()),*])
            .expect_exit_code(0)
    };
}

/// Create a version test
#[macro_export]
macro_rules! version_test {
    ($name:expr) => {
        $crate::test_utils::CommandTest::new($name, vec!["causality".to_string(), "--version".to_string()])
            .expect_exit_code(0)
    };
} 