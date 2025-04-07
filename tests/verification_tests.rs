#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use causality::effect::{CoreEffect, Effect};
    use causality::error::Result;
    use causality::factory::EffectFactory;
    use causality::riscv::{RiscVGenerator, RiscVProgram};
    use causality::types::{Account, Amount, Balance, ResourceId, Timestamp, ContentId};
    use causality::vm::{
        generate_proof, verify, verify_proof, MemoryAccess, Prover, ProverBackend, ProverConfig,
        PublicInputs, Verifier, VmState, Witness, WitnessGenerator,
    };

    fn create_sample_deposit_witness() -> Witness {
        // Create a simple witness for a deposit effect
        let mut registers_before = [0; 32];
        let mut registers_after = [0; 32];
        registers_after[10] = 100; // A0 register - deposit amount

        let transition = causality::vm::StateTransition {
            pc: 0,
            instruction: 0x06400513, // ADDI x10, x0, 100 (deposit amount)
            registers_before,
            registers_after,
            memory_accesses: Vec::new(),
        };

        let memory = HashMap::new();

        Witness {
            transitions: vec![transition],
            final_state: VmState {
                pc: 4,
                registers: registers_after,
                memory,
            },
        }
    }

    fn create_sample_withdrawal_witness() -> Witness {
        // Create a simple witness for a withdrawal effect
        let mut registers_before = [0; 32];
        let mut registers_after = [0; 32];
        registers_after[10] = 50; // A0 register - withdrawal amount

        let transition = causality::vm::StateTransition {
            pc: 0,
            instruction: 0x03200513, // ADDI x10, x0, 50 (withdrawal amount)
            registers_before,
            registers_after,
            memory_accesses: Vec::new(),
        };

        let memory = HashMap::new();

        Witness {
            transitions: vec![transition],
            final_state: VmState {
                pc: 4,
                registers: registers_after,
                memory,
            },
        }
    }

    #[test]
    fn test_verification_with_public_inputs() -> Result<()> {
        // Create a witness for a deposit
        let witness = create_sample_deposit_witness();

        // Generate a proof
        let proof = generate_proof(&witness, ProverBackend::Groth16)?;

        // Create public inputs for verification
        let account = Account::new(1);
        let amount = Amount::new(100);
        let balance = Balance(500);

        let inputs = PublicInputs::new()
            .with_account_deposit(account.clone(), amount)
            .with_account_balance(account, balance);

        // Verify the proof
        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed with public inputs");

        Ok(())
    }

    #[test]
    fn test_verification_with_resource_states() -> Result<()> {
        // Create a witness
        let witness = create_sample_withdrawal_witness();

        // Generate a proof
        let proof = generate_proof(&witness, ProverBackend::Groth16)?;

        // Create public inputs with resource states
        let resource_id = ResourceId(ContentId::generate());
        let resource_state = 42; // Some state value

        let inputs = PublicInputs::new().with_resource_state(resource_id, resource_state);

        // Verify the proof
        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed with resource state");

        Ok(())
    }

    #[test]
    fn test_verification_with_multiple_accounts() -> Result<()> {
        // Create a witness
        let witness = create_sample_deposit_witness();

        // Generate a proof
        let proof = generate_proof(&witness, ProverBackend::Groth16)?;

        // Create public inputs with multiple accounts
        let account1 = Account::new(1);
        let account2 = Account::new(2);
        let amount1 = Amount::new(100);
        let amount2 = Amount::new(50);
        let balance1 = Balance(500);
        let balance2 = Balance(300);

        let inputs = PublicInputs::new()
            .with_account_deposit(account1.clone(), amount1)
            .with_account_balance(account1, balance1)
            .with_account_withdrawal(account2.clone(), amount2)
            .with_account_balance(account2, balance2);

        // Verify the proof
        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed with multiple accounts");

        Ok(())
    }

    #[test]
    fn test_verification_with_custom_inputs() -> Result<()> {
        // Create a witness
        let witness = create_sample_deposit_witness();

        // Generate a proof
        let proof = generate_proof(&witness, ProverBackend::Groth16)?;

        // Create public inputs with custom values
        let inputs = PublicInputs::new()
            .with_custom_input("timestamp".to_string(), 1234567890)
            .with_custom_input("fee".to_string(), 5)
            .with_custom_input("gas_price".to_string(), 20);

        // Verify the proof
        let result = verify(&proof, &inputs)?;

        assert!(result, "Proof verification failed with custom inputs");

        Ok(())
    }
}
