use causality::memory::MemoryMapper;
use causality::riscv::{RiscVInstruction, RiscVProgram, RiscVSection};
use causality::vm::{registers, VirtualMachine, VmState};
use std::collections::HashMap;

// Helper function to create a simple test program
fn create_test_program() -> RiscVProgram {
    // Create a simple program that:
    // 1. Sets register 1 to 10
    // 2. Sets register 2 to 20
    // 3. Adds them and stores in register 3
    let instructions = vec![
        RiscVInstruction::Addi {
            rd: 1,
            rs1: 0,
            imm: 10,
        }, // x1 = x0 + 10
        RiscVInstruction::Addi {
            rd: 2,
            rs1: 0,
            imm: 20,
        }, // x2 = x0 + 20
        RiscVInstruction::Add {
            rd: 3,
            rs1: 1,
            rs2: 2,
        }, // x3 = x1 + x2
    ];

    // Create a text section with these instructions
    let mut text_section = RiscVSection {
        name: ".text".to_string(),
        instructions,
        labels: HashMap::new(),
    };

    // Add a label for the entry point
    text_section.labels.insert("main".to_string(), 0);

    // Create the program with this section
    RiscVProgram {
        sections: vec![text_section],
        entry_point: "main".to_string(),
        symbols: HashMap::new(),
    }
}

#[test]
fn test_vm_creation() {
    let vm = VirtualMachine::new(1024);
    assert_eq!(*vm.state(), VmState::Ready);
}

#[test]
fn test_register_operations() {
    let mut vm = VirtualMachine::new(1024);

    // Test setting a register
    vm.set_register(registers::T0, 42).unwrap();
    assert_eq!(vm.get_register(registers::T0).unwrap(), 42);

    // Test that x0 is always zero
    vm.set_register(registers::ZERO, 42).unwrap();
    assert_eq!(vm.get_register(registers::ZERO).unwrap(), 0);

    // Test invalid register
    assert!(vm.set_register(100, 42).is_err());
    assert!(vm.get_register(100).is_err());
}

#[test]
fn test_breakpoints() {
    let mut vm = VirtualMachine::new(1024);

    // Set a breakpoint
    vm.set_breakpoint(0x1000);
    assert!(vm.has_breakpoint(0x1000));

    // Clear a breakpoint
    vm.clear_breakpoint(0x1000);
    assert!(!vm.has_breakpoint(0x1000));

    // Toggle a breakpoint
    vm.toggle_breakpoint(0x2000);
    assert!(vm.has_breakpoint(0x2000));
    vm.toggle_breakpoint(0x2000);
    assert!(!vm.has_breakpoint(0x2000));
}

#[test]
fn test_load_program() {
    let mut vm = VirtualMachine::new(1024);
    let program = create_test_program();

    // Test loading a program
    vm.load_program(program).unwrap();
    assert_eq!(*vm.state(), VmState::Ready);
    assert_eq!(vm.get_pc(), 0);
}

// Note: We can't fully test execution yet because the fetch_instruction method
// is not yet implemented. We'll add execution tests after implementing that.
