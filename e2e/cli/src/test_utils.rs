//! Test utilities and framework for CLI end-to-end testing

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::TestConfig;

/// Test environment that manages temporary directories and state
pub struct TestEnvironment {
    /// Configuration for the test run
    pub config: TestConfig,
    /// Temporary directory for test artifacts
    _temp_dir: TempDir,
    /// Working directory for tests
    pub work_dir: PathBuf,
    /// Environment variables for CLI commands
    pub env_vars: HashMap<String, String>,
}

/// Test runner that executes CLI commands and validates results
pub struct TestRunner {
    /// Test environment
    env: TestEnvironment,
    /// Results from executed tests
    results: Vec<TestResult>,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name/identifier
    pub name: String,
    /// Test category
    pub category: String,
    /// Command that was tested
    pub command: String,
    /// Whether the test passed
    pub passed: bool,
    /// Whether the test was skipped
    pub skipped: bool,
    /// Test execution time
    pub duration: Duration,
    /// Standard output from command
    pub stdout: Option<String>,
    /// Standard error from command  
    pub stderr: Option<String>,
    /// Exit code from command
    pub exit_code: Option<i32>,
    /// Error message if test failed
    pub error: Option<String>,
    /// Additional test metadata
    pub metadata: HashMap<String, String>,
}

/// Represents a CLI command test case
#[derive(Debug, Clone)]
pub struct CommandTest {
    /// Test name
    pub name: String,
    /// CLI command and arguments
    pub command: Vec<String>,
    /// Expected exit code (None = any success code)
    pub expected_exit_code: Option<i32>,
    /// Expected stdout patterns
    pub expected_stdout: Vec<String>,
    /// Expected stderr patterns
    pub expected_stderr: Vec<String>,
    /// Working directory for the command
    pub working_dir: Option<PathBuf>,
    /// Environment variables for the command
    pub env_vars: HashMap<String, String>,
    /// Files that should be created by the command
    pub expected_files: Vec<PathBuf>,
    /// Files to clean up after test
    pub cleanup_files: Vec<PathBuf>,
    /// Timeout for command execution
    pub timeout: Duration,
    /// Whether to skip this test
    pub skip: bool,
    /// Reason for skipping
    pub skip_reason: Option<String>,
    /// Setup commands to run before the test
    pub setup_commands: Vec<Vec<String>>,
    /// Cleanup commands to run after the test
    pub cleanup_commands: Vec<Vec<String>>,
}

impl TestEnvironment {
    /// Create a new test environment
    pub async fn new(config: TestConfig) -> Result<Self> {
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory")?;
        
        let work_dir = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&work_dir)
            .context("Failed to create workspace directory")?;

        let mut env_vars = config.environment.env_vars.clone();
        
        // Set common environment variables for testing
        env_vars.insert("CAUSALITY_TEST_MODE".to_string(), "1".to_string());
        env_vars.insert("CAUSALITY_LOG_LEVEL".to_string(), "info".to_string());
        
        // Disable interactive prompts
        env_vars.insert("CAUSALITY_NO_PROMPT".to_string(), "1".to_string());
        
        Ok(Self {
            config,
            _temp_dir: temp_dir,
            work_dir,
            env_vars,
        })
    }

    /// Get the temporary directory path
    #[allow(dead_code)]
    pub fn temp_path(&self) -> &Path {
        self._temp_dir.path()
    }

    /// Get the working directory path
    #[allow(dead_code)]
    pub fn work_path(&self) -> &Path {
        &self.work_dir
    }

    /// Create a project directory for testing
    #[allow(dead_code)]
    pub fn create_project_dir(&self, name: &str) -> Result<PathBuf> {
        let project_dir = self.work_dir.join(name);
        std::fs::create_dir_all(&project_dir)
            .with_context(|| format!("Failed to create project directory: {}", name))?;
        Ok(project_dir)
    }

    /// Create a test file with content
    pub fn create_test_file(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create parent directories")?;
        }
        std::fs::write(path, content)
            .context("Failed to write test file")?;
        Ok(())
    }
}

impl TestRunner {
    /// Create a new test runner
    pub fn new(env: TestEnvironment) -> Self {
        Self {
            env,
            results: Vec::new(),
        }
    }

    /// Get the test configuration
    pub fn config(&self) -> &TestConfig {
        &self.env.config
    }

    /// Execute a command test
    pub async fn run_command_test(&mut self, test: CommandTest) -> Result<TestResult> {
        let start_time = Instant::now();
        
        info!("🧪 Running test: {}", test.name);

        if test.skip {
            return Ok(TestResult {
                name: test.name.clone(),
                category: "unknown".to_string(),
                command: test.command.join(" "),
                passed: false,
                skipped: true,
                duration: Duration::from_millis(0),
                stdout: None,
                stderr: None,
                exit_code: None,
                error: test.skip_reason,
                metadata: HashMap::new(),
            });
        }

        // Run setup commands
        for setup_cmd in &test.setup_commands {
            debug!("Running setup command: {:?}", setup_cmd);
            if let Err(e) = self.execute_command_simple(setup_cmd).await {
                warn!("Setup command failed: {}", e);
            }
        }

        // Prepare the main command
        let mut cmd = if let Some(cli_path) = &self.env.config.cli_binary_path {
            AssertCommand::new(cli_path)
        } else {
            AssertCommand::new("causality")
        };

        // Add arguments (skip first element which is the binary name)
        if test.command.len() > 1 {
            cmd.args(&test.command[1..]);
        }

        // Set working directory
        let work_dir = test.working_dir.as_ref()
            .unwrap_or(&self.env.work_dir);
        cmd.current_dir(work_dir);

        // Set environment variables
        for (key, value) in &self.env.env_vars {
            cmd.env(key, value);
        }
        for (key, value) in &test.env_vars {
            cmd.env(key, value);
        }

        // Set timeout
        cmd.timeout(test.timeout);

        // Execute the command
        let result = match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();

                debug!("Command output - stdout: {}", stdout);
                debug!("Command output - stderr: {}", stderr);
                debug!("Command exit code: {:?}", exit_code);

                // Validate results
                let mut passed = true;
                let mut error_messages = Vec::new();

                // Check exit code
                if let Some(expected_code) = test.expected_exit_code {
                    if exit_code != Some(expected_code) {
                        passed = false;
                        error_messages.push(format!(
                            "Expected exit code {}, got {:?}",
                            expected_code, exit_code
                        ));
                    }
                } else {
                    // Default: expect success (0)
                    if exit_code != Some(0) {
                        passed = false;
                        error_messages.push(format!(
                            "Expected success (exit code 0), got {:?}",
                            exit_code
                        ));
                    }
                }

                // Check stdout patterns
                for pattern in &test.expected_stdout {
                    if !stdout.contains(pattern) {
                        passed = false;
                        error_messages.push(format!(
                            "Expected stdout to contain: '{}'",
                            pattern
                        ));
                    }
                }

                // Check stderr patterns
                for pattern in &test.expected_stderr {
                    if !stderr.contains(pattern) {
                        passed = false;
                        error_messages.push(format!(
                            "Expected stderr to contain: '{}'",
                            pattern
                        ));
                    }
                }

                // Check expected files
                for expected_file in &test.expected_files {
                    let full_path = work_dir.join(expected_file);
                    if !full_path.exists() {
                        passed = false;
                        error_messages.push(format!(
                            "Expected file to be created: {}",
                            expected_file.display()
                        ));
                    }
                }

                TestResult {
                    name: test.name.clone(),
                    category: "unknown".to_string(),
                    command: test.command.join(" "),
                    passed,
                    skipped: false,
                    duration: start_time.elapsed(),
                    stdout: Some(stdout),
                    stderr: Some(stderr),
                    exit_code,
                    error: if error_messages.is_empty() {
                        None
                    } else {
                        Some(error_messages.join("; "))
                    },
                    metadata: HashMap::new(),
                }
            }
            Err(e) => {
                TestResult {
                    name: test.name.clone(),
                    category: "unknown".to_string(),
                    command: test.command.join(" "),
                    passed: false,
                    skipped: false,
                    duration: start_time.elapsed(),
                    stdout: None,
                    stderr: None,
                    exit_code: None,
                    error: Some(format!("Command execution failed: {}", e)),
                    metadata: HashMap::new(),
                }
            }
        };

        // Run cleanup commands
        for cleanup_cmd in &test.cleanup_commands {
            debug!("Running cleanup command: {:?}", cleanup_cmd);
            if let Err(e) = self.execute_command_simple(cleanup_cmd).await {
                warn!("Cleanup command failed: {}", e);
            }
        }

        // Clean up test files
        for cleanup_file in &test.cleanup_files {
            let full_path = work_dir.join(cleanup_file);
            if full_path.exists() {
                if let Err(e) = std::fs::remove_file(&full_path) {
                    warn!("Failed to clean up file {}: {}", full_path.display(), e);
                }
            }
        }

        self.results.push(result.clone());
        Ok(result)
    }

    /// Execute a simple command without validation
    async fn execute_command_simple(&self, command: &[String]) -> Result<()> {
        if command.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new(&command[0]);
        if command.len() > 1 {
            cmd.args(&command[1..]);
        }

        cmd.current_dir(&self.env.work_dir);
        
        for (key, value) in &self.env.env_vars {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::null())
           .stderr(Stdio::null());

        let status = cmd.status()
            .context("Failed to execute command")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Command failed with exit code: {:?}", status.code()));
        }

        Ok(())
    }

    /// Get all test results
    #[allow(dead_code)]
    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    /// Create a test file in the working directory
    pub fn create_test_file(&self, path: &str, content: &str) -> Result<PathBuf> {
        let full_path = self.env.work_dir.join(path);
        self.env.create_test_file(&full_path, content)?;
        Ok(full_path)
    }

    /// Create a project directory for testing
    #[allow(dead_code)]
    pub fn create_project_dir(&self, name: &str) -> Result<PathBuf> {
        self.env.create_project_dir(name)
    }
}

impl Default for CommandTest {
    fn default() -> Self {
        Self {
            name: "default_test".to_string(),
            command: vec!["causality".to_string(), "--help".to_string()],
            expected_exit_code: Some(0),
            expected_stdout: vec![],
            expected_stderr: vec![],
            working_dir: None,
            env_vars: HashMap::new(),
            expected_files: vec![],
            cleanup_files: vec![],
            timeout: Duration::from_secs(30),
            skip: false,
            skip_reason: None,
            setup_commands: vec![],
            cleanup_commands: vec![],
        }
    }
}

impl CommandTest {
    /// Create a new command test with basic settings
    pub fn new(name: &str, command: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            command,
            ..Default::default()
        }
    }

    /// Set expected exit code
    pub fn expect_exit_code(mut self, code: i32) -> Self {
        self.expected_exit_code = Some(code);
        self
    }

    /// Add expected stdout pattern
    pub fn expect_stdout(mut self, pattern: &str) -> Self {
        self.expected_stdout.push(pattern.to_string());
        self
    }

    /// Add expected stderr pattern
    pub fn expect_stderr(mut self, pattern: &str) -> Self {
        self.expected_stderr.push(pattern.to_string());
        self
    }

    /// Set working directory
    #[allow(dead_code)]
    pub fn in_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Add environment variable
    #[allow(dead_code)]
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Add expected file
    pub fn expect_file(mut self, path: &str) -> Self {
        self.expected_files.push(PathBuf::from(path));
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Skip this test
    pub fn skip(mut self, reason: &str) -> Self {
        self.skip = true;
        self.skip_reason = Some(reason.to_string());
        self
    }

    /// Add setup command
    #[allow(dead_code)]
    pub fn with_setup(mut self, command: Vec<String>) -> Self {
        self.setup_commands.push(command);
        self
    }

    /// Add cleanup command
    #[allow(dead_code)]
    pub fn with_cleanup(mut self, command: Vec<String>) -> Self {
        self.cleanup_commands.push(command);
        self
    }
}

/// Helper macros for creating common test patterns
#[macro_export]
macro_rules! cmd_test {
    ($name:expr, $($arg:expr),*) => {
        $crate::test_utils::CommandTest::new($name, vec!["causality".to_string(), $($arg.to_string()),*])
    };
}

#[macro_export]
macro_rules! help_test {
    ($name:expr, $topic:expr) => {
        cmd_test!($name, "help", $topic)
            .expect_stdout("Usage:")
            .expect_exit_code(0)
    };
}

#[macro_export]
macro_rules! version_test {
    () => {
        cmd_test!("version_check", "--version")
            .expect_stdout("causality")
            .expect_exit_code(0)
    };
} 