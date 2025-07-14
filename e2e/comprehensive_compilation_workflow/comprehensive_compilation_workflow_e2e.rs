//! Comprehensive Compilation Workflow E2E Test
//!
//! This test demonstrates the new file-based compilation pipeline:
//! 1. Write S-expression to .sx file
//! 2. Run causality-cli as subprocess to compile to .bc bytecode
//! 3. Load bytecode and run simulation via minimal FFI
//! 4. Validate the complete workflow

use anyhow::{Context, Result};
use causality_ffi::{
    causality_free_simulation_state, causality_load_bytecode,
    causality_run_simulation_step, CResult,
};
use scopeguard;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[tokio::test]
async fn test_new_compilation_workflow() -> Result<()> {
    println!("Starting New Compilation Workflow E2E Test");
    println!("==========================================");

    // Test data
    let test_program = "(pure 42)";
    let sx_file = "test_workflow.sx";
    let bc_file = "test_workflow.bc";

    // Cleanup function
    let cleanup = || {
        let _ = fs::remove_file(sx_file);
        let _ = fs::remove_file(bc_file);
    };

    // Ensure cleanup on any exit
    let _guard = scopeguard::guard((), |_| cleanup());

    // Step 1: Write S-expression to file
    println!("1. Writing S-expression to {}", sx_file);
    fs::write(sx_file, test_program)
        .with_context(|| format!("Failed to write S-expression to {}", sx_file))?;

    // Step 2: Run causality-cli as subprocess
    println!("2. Compiling with causality-cli...");

    // Get the project root directory and build path to causality binary
    // The test runs from the e2e directory, so we need to go up one level
    let current_dir =
        env::current_dir().with_context(|| "Failed to get current directory")?;
    let project_root = current_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get project root directory"))?;
    let causality_path = project_root.join("target/debug/causality");

    println!("   Using causality binary at: {}", causality_path.display());

    let output = Command::new(&causality_path)
        .args(&["compile", "-i", sx_file, "-o", bc_file])
        .output()
        .with_context(|| "Failed to execute causality-cli")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("causality-cli failed: {}", stderr);
    }

    println!("   Compilation successful");

    // Step 3: Read bytecode
    println!("3. Reading bytecode from {}", bc_file);
    let bytecode = fs::read(bc_file)
        .with_context(|| format!("Failed to read bytecode from {}", bc_file))?;

    if bytecode.is_empty() {
        anyhow::bail!("Bytecode file is empty");
    }

    println!("   Read {} bytes of bytecode", bytecode.len());

    // Step 4: Load simulation via FFI
    println!("4. Loading simulation via FFI...");
    let sim_state =
        unsafe { causality_load_bytecode(bytecode.as_ptr(), bytecode.len()) };

    if sim_state.is_null() {
        anyhow::bail!("Failed to load simulation: FFI returned null pointer");
    }

    println!("   Simulation loaded successfully");

    // Step 5: Run simulation steps
    println!("5. Running simulation steps...");

    let step_result = unsafe { causality_run_simulation_step(sim_state) };
    match step_result {
        CResult::Ok => println!("   Step 1 completed successfully"),
        CResult::Err(_) => anyhow::bail!("Step 1 failed"),
    }

    let step_result = unsafe { causality_run_simulation_step(sim_state) };
    match step_result {
        CResult::Ok => println!("   Step 2 completed successfully"),
        CResult::Err(_) => anyhow::bail!("Step 2 failed"),
    }

    // Step 6: Free simulation state
    println!("6. Freeing simulation state...");
    unsafe {
        causality_free_simulation_state(sim_state);
    }
    println!("   State freed successfully");

    println!("\n✅ New Compilation Workflow E2E Test Passed!");
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    println!("Testing Error Handling in New Workflow");
    println!("======================================");

    // Test with invalid S-expression
    let invalid_program = "(invalid syntax";
    let sx_file = "test_error.sx";
    let bc_file = "test_error.bc";

    // Cleanup function
    let cleanup = || {
        let _ = fs::remove_file(sx_file);
        let _ = fs::remove_file(bc_file);
    };

    let _guard = scopeguard::guard((), |_| cleanup());

    // Write invalid S-expression
    fs::write(sx_file, invalid_program)?;

    // Try to compile - should fail
    let output = Command::new("./target/debug/causality")
        .args(&["compile", "-i", sx_file, "-o", bc_file])
        .output()
        .with_context(|| "Failed to execute causality-cli")?;

    if output.status.success() {
        anyhow::bail!(
            "Expected compilation to fail with invalid syntax, but it succeeded"
        );
    }

    println!("✅ Error handling test passed - invalid syntax properly rejected");

    // Test with missing input file
    let output = Command::new("./target/debug/causality")
        .args(&["compile", "-i", "nonexistent.sx", "-o", bc_file])
        .output()
        .with_context(|| "Failed to execute causality-cli")?;

    if output.status.success() {
        anyhow::bail!(
            "Expected compilation to fail with missing file, but it succeeded"
        );
    }

    println!("✅ Error handling test passed - missing file properly handled");
    Ok(())
}

#[tokio::test]
async fn test_multiple_programs() -> Result<()> {
    println!("Testing Multiple Program Compilation");
    println!("===================================");

    let test_cases = vec![
        ("simple", "(pure 42)"),
        ("string", "(pure \"hello\")"),
        ("boolean", "(pure true)"),
    ];

    for (name, program) in test_cases {
        println!("Testing program: {}", name);

        let sx_file = format!("test_{}.sx", name);
        let bc_file = format!("test_{}.bc", name);

        // Cleanup for this test case
        let cleanup = || {
            let _ = fs::remove_file(&sx_file);
            let _ = fs::remove_file(&bc_file);
        };

        let _guard = scopeguard::guard((), |_| cleanup());

        // Write and compile
        fs::write(&sx_file, program)?;

        let output = Command::new("./target/debug/causality")
            .args(&["compile", "-i", &sx_file, "-o", &bc_file])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Compilation failed for {}: {}", name, stderr);
        }

        // Verify bytecode was created
        if !Path::new(&bc_file).exists() {
            anyhow::bail!("Bytecode file not created for {}", name);
        }

        let bytecode = fs::read(&bc_file)?;
        if bytecode.is_empty() {
            anyhow::bail!("Empty bytecode for {}", name);
        }

        println!(
            "  ✅ {} compiled successfully ({} bytes)",
            name,
            bytecode.len()
        );
    }

    println!("✅ Multiple program compilation test passed!");
    Ok(())
}
