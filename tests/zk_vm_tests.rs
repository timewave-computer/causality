use std::collections::HashMap;

use causality::effect::factory;
use causality::error::Result;
use causality::riscv::{RiscVGenerator, RiscVProgram};
use causality::types::{Account, Amount, Timestamp};
use causality::vm::{ZkAdapter, ZkVirtualMachine};

/// Test that we can execute a simple effect in the ZK VM
#[test]
fn test_zk_vm_integration() -> Result<()> {
    // Create a simple deposit effect
    let account = Account::new(1);
    let amount = Amount::new(100);
    let timestamp = Timestamp::now();

    let effect = factory::deposit(account, amount, timestamp, |result| result);

    // Generate RISC-V code for the effect
    let mut generator = RiscVGenerator::new();
    let program = generator.generate_code(&effect)?;

    // Create a ZK VM adapter
    let mut vm = ZkAdapter::new(1024 * 1024); // 1MB of memory

    // Load the program into the VM
    vm.load_program(program)?;

    // Generate a witness
    let witness = vm.generate_witness()?;

    // Check that the witness has the expected structure
    assert!(!witness.transitions.isEmpty());
    assert_eq!(witness.final_state.pc, 0); // This will be updated once the VM is properly implemented

    // Generate a proof
    let proof = vm.generate_proof(&witness)?;

    // Verify the proof
    let verification_result = vm.verify_proof(&proof)?;
    assert!(verification_result);

    Ok(())
}

/// Test witness generation for a withdrawal effect
#[test]
fn test_withdrawal_witness() -> Result<()> {
    // Create a withdrawal effect
    let account = Account::new(1);
    let amount = Amount::new(50);
    let timestamp = Timestamp::now();

    let effect = factory::withdrawal(account, amount, timestamp, |result| result);

    // Generate RISC-V code for the effect
    let mut generator = RiscVGenerator::new();
    let program = generator.generate_code(&effect)?;

    // Create a ZK VM adapter
    let mut vm = ZkAdapter::new(1024 * 1024); // 1MB of memory

    // Load the program into the VM
    vm.load_program(program)?;

    // Generate a witness
    let witness = vm.generate_witness()?;

    // Check the witness structure
    assert!(!witness.transitions.isEmpty());

    Ok(())
}

/// Test generating proofs for multiple effects
#[test]
fn test_multiple_effect_proofs() -> Result<()> {
    // Create multiple effects
    let account = Account::new(1);
    let amount1 = Amount::new(100);
    let amount2 = Amount::new(50);
    let timestamp = Timestamp::now();

    let deposit_effect =
        factory::deposit(account.clone(), amount1, timestamp.clone(), |result| result);

    let withdrawal_effect = factory::withdrawal(account, amount2, timestamp, |result| result);

    // Generate RISC-V code for the effects
    let mut generator = RiscVGenerator::new();
    let deposit_program = generator.generate_code(&deposit_effect)?;
    let withdrawal_program = generator.generate_code(&withdrawal_effect)?;

    // Link the programs together
    let combined_program = generator.link(&[deposit_program, withdrawal_program])?;

    // Create a ZK VM adapter
    let mut vm = ZkAdapter::new(1024 * 1024); // 1MB of memory

    // Load the combined program into the VM
    vm.load_program(combined_program)?;

    // Generate a witness
    let witness = vm.generate_witness()?;

    // Generate a proof
    let proof = vm.generate_proof(&witness)?;

    // Verify the proof
    let verification_result = vm.verify_proof(&proof)?;
    assert!(verification_result);

    Ok(())
}
