//! Tests for project management commands
//!
//! Tests all project-related commands from the CLI documentation:
//! - causality project new <name> --template <template>
//! - causality project init
//! - causality project build
//! - causality project clean
//! - causality project status
//! - causality project add <package>

use anyhow::Result;
use std::time::Duration;

use crate::test_utils::{TestResult, TestRunner};
use crate::cmd_test;

/// Run all project management command tests
pub async fn run_project_tests(runner: &mut TestRunner) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();

    // Test project command help
    let test = cmd_test!("project_help", "project", "--help")
        .expect_stdout("Project management")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project alias
    let test = cmd_test!("project_alias", "p", "--help")
        .expect_stdout("Project management")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project new help
    let test = cmd_test!("project_new_help", "project", "new", "--help")
        .expect_stdout("Create a new Causality project")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test creating projects with different templates
    let templates = vec![
        ("basic", "Basic project structure"),
        ("defi", "Cross-chain DeFi application"),
        ("privacy", "Privacy-focused application"),
        ("zk", "zkSNARK circuit development"),
        ("library", "Library/package development"),
        ("advanced", "Advanced multi-chain setup"),
    ];

    for (template, _description) in templates {
        // Create a unique project name for each template
        let project_name = format!("test-{}-project", template);
        
        let test = cmd_test!(
            &format!("project_new_{}", template),
            "project", "new", &project_name,
            "--template", template
        )
        .expect_exit_code(0)
        .expect_stdout(&format!("Creating new project '{}'", project_name))
        .with_timeout(Duration::from_secs(30));
        
        let mut result = runner.run_command_test(test).await?;
        result.category = "project".to_string();
        results.push(result);
    }

    // Test project new with git initialization
    let test = cmd_test!(
        "project_new_with_git",
        "project", "new", "git-test-project", 
        "--template", "basic", "--git"
    )
    .expect_exit_code(0)
    .expect_stdout("Git repository: enabled")
    .with_timeout(Duration::from_secs(30));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project new with description
    let test = cmd_test!(
        "project_new_with_description",
        "project", "new", "desc-test-project",
        "--template", "basic",
        "--description", "A test project with description"
    )
    .expect_exit_code(0)
    .expect_stdout("Description: A test project with description")
    .with_timeout(Duration::from_secs(30));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project init help
    let test = cmd_test!("project_init_help", "project", "init", "--help")
        .expect_stdout("Initialize current directory")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project init in empty directory
    let test = cmd_test!("project_init_empty", "project", "init")
        .expect_exit_code(0)
        .expect_stdout("Project initialized")
        .with_timeout(Duration::from_secs(30));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project init with force in non-empty directory
    let test = cmd_test!("project_init_force", "project", "init", "--force")
        .expect_exit_code(0)
        .expect_stdout("Force mode enabled")
        .with_timeout(Duration::from_secs(30));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project build help
    let test = cmd_test!("project_build_help", "project", "build", "--help")
        .expect_stdout("Build the current project")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project build alias
    let test = cmd_test!("project_build_alias", "project", "b", "--help")
        .expect_stdout("Build the current project")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project build with release flag
    let test = cmd_test!("project_build_release", "project", "build", "--release", "--help")
        .expect_stdout("release")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project build with timings
    let test = cmd_test!("project_build_timings", "project", "build", "--timings", "--help")
        .expect_stdout("timing")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project clean help
    let test = cmd_test!("project_clean_help", "project", "clean", "--help")
        .expect_stdout("Clean build artifacts")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project clean with deep flag
    let test = cmd_test!("project_clean_deep", "project", "clean", "--deep", "--help")
        .expect_stdout("deep")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project status help
    let test = cmd_test!("project_status_help", "project", "status", "--help")
        .expect_stdout("Show project status")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project status alias
    let test = cmd_test!("project_status_alias", "project", "s", "--help")
        .expect_stdout("Show project status")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project status with deps flag
    let test = cmd_test!("project_status_deps", "project", "status", "--deps", "--help")
        .expect_stdout("deps")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project add help
    let test = cmd_test!("project_add_help", "project", "add", "--help")
        .expect_stdout("Add dependencies")
        .expect_exit_code(0)
        .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    // Test project add with version
    let test = cmd_test!(
        "project_add_with_version", 
        "project", "add", "test-package", "--version", "1.0.0", "--help"
    )
    .expect_stdout("version")
    .expect_exit_code(0)
    .with_timeout(Duration::from_secs(10));
    
    let mut result = runner.run_command_test(test).await?;
    result.category = "project".to_string();
    results.push(result);

    Ok(results)
} 