#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use causality::effect::{CoreEffect, Effect};
    use causality::error::Result;
    use causality::factory::EffectFactory;
    use causality::riscv::{RiscVGenerator, RiscVProgram};
    use causality::types::{Account, Amount, Balance, Timestamp};
    use causality::vm::{
        verify, CausalityVerifier, Prover, ProverBackend, ProverConfig, PublicInputs,
        WitnessGenerator, ZkAdapter,
    };

    /// End-to-end test that covers the full pipeline:
    /// 1. Create an effect (deposit)
    /// 2. Compile it to RISC-V
    /// 3. Execute it in the VM to generate a witness
    /// 4. Generate a ZK proof from the witness
    /// 5. Verify the proof
    #[test]
    fn test_end_to_end_deposit_effect_proving() -> Result<()> {
        // Step 1: Create a deposit effect
        let factory = EffectFactory::new();
        let account = Account::new("alice".to_string());
        let amount = Amount::new(100);
        let timestamp = Timestamp::new(1234567890);

        let deposit_effect =
            factory.create_deposit_effect(account.clone(), amount.clone(), timestamp)?;

        // Step 2: Compile to RISC-V
        let mut generator = RiscVGenerator::new();
        let program = generator.generate_code(&deposit_effect)?;

        println!(
            "Generated RISC-V program with {} instructions",
            program.instructions.len()
        );

        // Step 3: Execute in ZK VM and generate witness
        let mut zk_vm = ZkAdapter::new(1024); // 1KB memory
        zk_vm.load_program(program)?;
        let witness = zk_vm.generate_witness()?;

        println!(
            "Generated witness with {} transitions",
            witness.transitions.len()
        );

        // Step 4: Generate ZK proof
        let backend = ProverBackend::Groth16; // Use Groth16 for this test
        let prover = Prover::default(backend.clone());
        let proof = prover.generate_proof(&witness)?;

        println!("Generated proof with {} bytes", proof.data.len());

        // Step 5: Verify the proof
        let inputs = PublicInputs::new()
            .with_account_deposit(account.clone(), amount)
            .with_account_balance(account, Balance(100)); // After deposit balance

        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed");
        println!("Proof verification succeeded!");

        Ok(())
    }

    /// End-to-end test that covers withdrawals
    #[test]
    fn test_end_to_end_withdrawal_effect_proving() -> Result<()> {
        // Step 1: Create a withdrawal effect
        let factory = EffectFactory::new();
        let account = Account::new("bob".to_string());
        let amount = Amount::new(50);
        let timestamp = Timestamp::new(1234567890);

        let withdrawal_effect =
            factory.create_withdrawal_effect(account.clone(), amount.clone(), timestamp)?;

        // Step 2: Compile to RISC-V
        let mut generator = RiscVGenerator::new();
        let program = generator.generate_code(&withdrawal_effect)?;

        // Step 3: Execute in ZK VM and generate witness
        let mut zk_vm = ZkAdapter::new(1024); // 1KB memory
        zk_vm.load_program(program)?;
        let witness = zk_vm.generate_witness()?;

        // Step 4: Generate ZK proof
        let backend = ProverBackend::Plonk; // Use PLONK for this test
        let prover = Prover::default(backend.clone());
        let proof = prover.generate_proof(&witness)?;

        // Step 5: Verify the proof
        let inputs = PublicInputs::new()
            .with_account_withdrawal(account.clone(), amount)
            .with_account_balance(account, Balance(950)); // After withdrawal balance

        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed");

        Ok(())
    }

    /// End-to-end test that covers both deposit and withdrawal in sequence
    #[test]
    fn test_end_to_end_multiple_effects_proving() -> Result<()> {
        // Create a test account
        let account = Account::new("charlie".to_string());
        let timestamp = Timestamp::new(1234567890);
        let factory = EffectFactory::new();

        // Step 1: Create effects
        let deposit_amount = Amount::new(200);
        let deposit_effect =
            factory.create_deposit_effect(account.clone(), deposit_amount.clone(), timestamp)?;

        let withdrawal_amount = Amount::new(75);
        let withdrawal_effect = factory.create_withdrawal_effect(
            account.clone(),
            withdrawal_amount.clone(),
            timestamp,
        )?;

        // Step 2: Compile effects to RISC-V
        let mut generator = RiscVGenerator::new();
        let deposit_program = generator.generate_code(&deposit_effect)?;
        let withdrawal_program = generator.generate_code(&withdrawal_effect)?;

        // Step 3A: Execute deposit in ZK VM and generate witness
        let mut deposit_zk_vm = ZkAdapter::new(1024);
        deposit_zk_vm.load_program(deposit_program)?;
        let deposit_witness = deposit_zk_vm.generate_witness()?;

        // Step 3B: Execute withdrawal in ZK VM and generate witness
        let mut withdrawal_zk_vm = ZkAdapter::new(1024);
        withdrawal_zk_vm.load_program(withdrawal_program)?;
        let withdrawal_witness = withdrawal_zk_vm.generate_witness()?;

        // Step 4A: Generate ZK proof for deposit
        let deposit_backend = ProverBackend::Groth16;
        let deposit_prover = Prover::default(deposit_backend.clone());
        let deposit_proof = deposit_prover.generate_proof(&deposit_witness)?;

        // Step 4B: Generate ZK proof for withdrawal
        let withdrawal_backend = ProverBackend::Halo2;
        let withdrawal_prover = Prover::default(withdrawal_backend.clone());
        let withdrawal_proof = withdrawal_prover.generate_proof(&withdrawal_witness)?;

        // Step 5A: Verify the deposit proof
        let deposit_inputs = PublicInputs::new()
            .with_account_deposit(account.clone(), deposit_amount)
            .with_account_balance(account.clone(), Balance(200)); // After deposit balance

        let deposit_result = verify(&deposit_proof, &deposit_inputs)?;
        assert!(deposit_result, "Deposit proof verification failed");

        // Step 5B: Verify the withdrawal proof
        let withdrawal_inputs = PublicInputs::new()
            .with_account_withdrawal(account.clone(), withdrawal_amount)
            .with_account_balance(account.clone(), Balance(125)); // After withdrawal balance

        let withdrawal_result = verify(&withdrawal_proof, &withdrawal_inputs)?;
        assert!(withdrawal_result, "Withdrawal proof verification failed");

        println!("Both proofs verified successfully!");

        Ok(())
    }

    /// End-to-end test for observation effects
    #[test]
    fn test_end_to_end_observation_effect_proving() -> Result<()> {
        // Step 1: Create an observation effect
        let factory = EffectFactory::new();
        let account = Account::new("dave".to_string());
        let timestamp = Timestamp::new(1234567890);

        let observation_effect = factory.create_observation_effect(account.clone(), timestamp)?;

        // Step 2: Compile to RISC-V
        let mut generator = RiscVGenerator::new();
        let program = generator.generate_code(&observation_effect)?;

        // Step 3: Execute in ZK VM and generate witness
        let mut zk_vm = ZkAdapter::new(1024);
        zk_vm.load_program(program)?;
        let witness = zk_vm.generate_witness()?;

        // Step 4: Generate ZK proof
        let backend = ProverBackend::Marlin;
        let prover = Prover::default(backend.clone());
        let proof = prover.generate_proof(&witness)?;

        // Step 5: Verify the proof
        let inputs = PublicInputs::new()
            .with_account_balance(account.clone(), Balance(300))
            .with_custom_input("timestamp".to_string(), timestamp.0 as u32);

        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed");

        Ok(())
    }
}
