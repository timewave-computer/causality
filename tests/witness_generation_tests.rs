#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use causality::effect::{CoreEffect, Effect};
    use causality::error::Result;
    use causality::factory::EffectFactory;
    use causality::riscv::{RiscVGenerator, RiscVProgram};
    use causality::types::{Account, Amount, Timestamp};
    use causality::vm::{MemoryAccess, WitnessGenerator};

    fn create_sample_program() -> Result<RiscVProgram> {
        // Create a deposit effect
        let factory = EffectFactory::new();
        let account = Account::new("user123".to_string());
        let amount = Amount::new(100);
        let timestamp = Timestamp::new(1234567890);

        let deposit = factory.create_deposit_effect(account, amount, timestamp)?;

        // Generate RISC-V code for the deposit effect
        let mut generator = RiscVGenerator::new();
        generator.generate_code(&deposit)
    }

    #[test]
    fn test_witness_generator_creation() {
        let generator = WitnessGenerator::new();
        assert_eq!(generator.generate_witness().unwrap().transitions.len(), 0);
    }

    #[test]
    fn test_record_register_states() {
        let mut generator = WitnessGenerator::new();
        let registers1 = [0; 32];
        let mut registers2 = [0; 32];
        registers2[10] = 100; // A0 register

        generator.record_registers(registers1);
        generator.record_registers(registers2);

        let witness = generator.generate_witness().unwrap();
        assert_eq!(witness.final_state.registers[10], 100);
    }

    #[test]
    fn test_record_pc_values() {
        let mut generator = WitnessGenerator::new();
        generator.record_pc(0);
        generator.record_pc(4);
        generator.record_pc(8);

        let witness = generator.generate_witness().unwrap();
        assert_eq!(witness.final_state.pc, 8);
    }

    #[test]
    fn test_record_instructions() {
        let mut generator = WitnessGenerator::new();
        // ADDI x10, x0, 100 (0x06400513)
        generator.record_instruction(0x06400513);

        // Record PC and register states for the transition
        generator.record_pc(0);

        let initial_registers = [0; 32];
        generator.record_registers(initial_registers);

        let mut final_registers = [0; 32];
        final_registers[10] = 100; // A0 register
        generator.record_registers(final_registers);

        // Generate the witness
        let witness = generator.generate_witness().unwrap();

        // Since we only have one instruction, we should have one transition
        assert_eq!(witness.transitions.len(), 1);
        assert_eq!(witness.transitions[0].instruction, 0x06400513);
        assert_eq!(witness.transitions[0].registers_after[10], 100);
    }

    #[test]
    fn test_record_memory_access() {
        let mut generator = WitnessGenerator::new();

        // Record a memory write at address 0x1000
        let access = MemoryAccess {
            address: 0x1000,
            value: 0x12345678,
            is_write: true,
        };
        generator.record_memory_access(access);

        // Generate the witness
        let witness = generator.generate_witness().unwrap();

        // Check that the memory access was recorded
        assert_eq!(witness.final_state.memory.get(&0x1000), Some(&0x12345678));
    }

    #[test]
    fn test_witness_for_deposit_effect() -> Result<()> {
        // Create a RISC-V program for a deposit effect
        let program = create_sample_program()?;

        // Create a witness generator
        let mut generator = WitnessGenerator::new();

        // For this test, we'll manually add some simulated execution data
        // In a real implementation, this would come from actually executing the program

        // Record initial state
        let mut initial_registers = [0; 32];
        generator.record_registers(initial_registers);
        generator.record_pc(0);

        // Record execution of a few instructions
        for i in 0..10 {
            // Record the instruction (we're using dummy values here)
            generator.record_instruction(0x06400513 + i);

            // Record PC
            generator.record_pc(i * 4);

            // Update registers for this step
            initial_registers[10] += 10; // Accumulate in A0
            generator.record_registers(initial_registers);

            // Add a memory access every other instruction
            if i % 2 == 0 {
                let access = MemoryAccess {
                    address: 0x1000 + i,
                    value: 0x10000 + i,
                    is_write: true,
                };
                generator.record_memory_access(access);
            }
        }

        // Generate the witness
        let witness = generator.generate_witness()?;

        // Check the witness
        assert_eq!(witness.transitions.len(), 9); // 10 instructions = 9 transitions
        assert_eq!(witness.final_state.registers[10], 90); // 9 * 10 = 90
        assert_eq!(witness.final_state.pc, 36); // 9 * 4 = 36
        assert!(witness.final_state.memory.contains_key(&0x1008)); // Should have written to this address

        Ok(())
    }

    #[test]
    fn test_witness_reset() {
        let mut generator = WitnessGenerator::new();

        // Add some data
        generator.record_pc(100);
        generator.record_instruction(0x12345678);
        let registers = [42; 32];
        generator.record_registers(registers);
        generator.record_registers(registers);

        // Reset the generator
        generator.reset();

        // Check that everything was cleared
        let witness = generator.generate_witness().unwrap();
        assert_eq!(witness.transitions.len(), 0);
        assert_eq!(witness.final_state.pc, 0);
        assert_eq!(witness.final_state.registers[0], 0);
        assert!(witness.final_state.memory.is_empty());
    }
}
