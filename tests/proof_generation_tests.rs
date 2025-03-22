#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use causality::effect::{CoreEffect, Effect};
    use causality::error::Result;
    use causality::factory::EffectFactory;
    use causality::riscv::{RiscVGenerator, RiscVProgram};
    use causality::types::{Account, Amount, Timestamp};
    use causality::vm::{
        generate_proof, verify_proof, MemoryAccess, Prover, ProverBackend, ProverConfig, Verifier,
        VmState, Witness, WitnessGenerator,
    };

    fn create_sample_witness() -> Witness {
        // Create a simple witness with a few transitions
        let mut registers_before = [0; 32];
        let mut registers_after = [0; 32];
        registers_after[10] = 100; // A0 register

        let transition = causality::vm::StateTransition {
            pc: 0,
            instruction: 0x06400513, // ADDI x10, x0, 100
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
    fn test_proof_generation() -> Result<()> {
        // Create a witness
        let witness = create_sample_witness();

        // Generate a proof using Groth16
        let proof = generate_proof(&witness, ProverBackend::Groth16)?;

        // Verify the proof
        let public_inputs = [0, 0, 0, 0]; // Placeholder public inputs
        let result = verify_proof(&proof, &public_inputs, ProverBackend::Groth16)?;

        assert!(result, "Proof verification failed");

        Ok(())
    }

    #[test]
    fn test_proof_verification_with_wrong_backend() -> Result<()> {
        // Create a witness
        let witness = create_sample_witness();

        // Generate a proof using Groth16
        let proof = generate_proof(&witness, ProverBackend::Groth16)?;

        // Verify with the wrong backend (PLONK)
        let public_inputs = [0, 0, 0, 0]; // Placeholder public inputs
        let result = verify_proof(&proof, &public_inputs, ProverBackend::Plonk)?;

        assert!(
            !result,
            "Proof verification should fail with mismatched backend"
        );

        Ok(())
    }

    #[test]
    fn test_multiple_proving_backends() -> Result<()> {
        // Create a witness
        let witness = create_sample_witness();

        // Test all available backends
        let backends = [
            ProverBackend::Groth16,
            ProverBackend::Plonk,
            ProverBackend::Marlin,
            ProverBackend::Halo2,
        ];

        for backend in &backends {
            // Generate a proof
            let proof = generate_proof(&witness, backend.clone())?;

            // Verify the proof with the correct backend
            let public_inputs = [0, 0, 0, 0]; // Placeholder public inputs
            let result = verify_proof(&proof, &public_inputs, backend.clone())?;

            assert!(
                result,
                "Proof verification failed for backend: {:?}",
                backend
            );

            // Try to verify with each other backend (should fail)
            for wrong_backend in &backends {
                if wrong_backend != backend {
                    let result = verify_proof(&proof, &public_inputs, wrong_backend.clone())?;
                    assert!(
                        !result,
                        "Proof verification should fail when using {:?} to verify a {:?} proof",
                        wrong_backend, backend
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_prover_with_custom_config() -> Result<()> {
        // Create a witness
        let witness = create_sample_witness();

        // Create a custom prover configuration
        let config = ProverConfig {
            proving_key_path: std::path::PathBuf::from("custom/proving_key.bin"),
            verification_key_path: std::path::PathBuf::from("custom/verification_key.bin"),
            num_threads: 2,
            memory_limit: 4096,
            verbose: true,
        };

        // Create a prover with the custom configuration
        let prover = Prover::new(config, ProverBackend::Groth16);

        // Generate a proof
        let proof = prover.generate_proof(&witness)?;

        // Create a verifier and verify the proof
        let verifier = Verifier::new(ProverBackend::Groth16);
        let public_inputs = [0, 0, 0, 0]; // Placeholder public inputs
        let result = verifier.verify_proof(&proof, &public_inputs)?;

        assert!(
            result,
            "Proof verification failed with custom prover config"
        );

        Ok(())
    }
}
