//! Integration tests for register machine instructions

use causality_core::machine::{Instruction, RegisterId};

#[test]
fn test_transform_instruction() {
    // Test the Transform instruction (morphism application)
    // For now, just test that the instruction can be created
    let transform_instr = Instruction::Transform {
        morph_reg: RegisterId::new(1),
        input_reg: RegisterId::new(2),
        output_reg: RegisterId::new(3),
    };
    
    // Verify instruction properties
    assert_eq!(transform_instr.reads(), vec![RegisterId::new(1), RegisterId::new(2)]);
    assert_eq!(transform_instr.writes(), vec![RegisterId::new(3)]);
    assert!(transform_instr.is_linear());
    assert_eq!(transform_instr.operation_type(), "morphism_application");
    assert!(transform_instr.verify_category_laws());
}

#[test]
fn test_alloc_instruction() {
    // Test the Alloc instruction (resource allocation)
    let alloc_instr = Instruction::Alloc {
        type_reg: RegisterId::new(1),
        init_reg: RegisterId::new(2),
        output_reg: RegisterId::new(3),
    };
    
    // Verify instruction properties
    assert_eq!(alloc_instr.reads(), vec![RegisterId::new(1), RegisterId::new(2)]);
    assert_eq!(alloc_instr.writes(), vec![RegisterId::new(3)]);
    assert!(alloc_instr.is_linear());
    assert_eq!(alloc_instr.operation_type(), "object_creation");
    assert!(alloc_instr.verify_category_laws());
}

#[test]
fn test_consume_instruction() {
    // Test the Consume instruction (resource consumption)
    let consume_instr = Instruction::Consume {
        resource_reg: RegisterId::new(1),
        output_reg: RegisterId::new(2),
    };
    
    // Verify instruction properties
    assert_eq!(consume_instr.reads(), vec![RegisterId::new(1)]);
    assert_eq!(consume_instr.writes(), vec![RegisterId::new(2)]);
    assert!(consume_instr.is_linear());
    assert_eq!(consume_instr.operation_type(), "object_destruction");
    assert!(consume_instr.verify_category_laws());
}

#[test]
fn test_compose_instruction() {
    // Test the Compose instruction (morphism composition)
    let compose_instr = Instruction::Compose {
        first_reg: RegisterId::new(1),
        second_reg: RegisterId::new(2),
        output_reg: RegisterId::new(3),
    };
    
    // Verify instruction properties
    assert_eq!(compose_instr.reads(), vec![RegisterId::new(1), RegisterId::new(2)]);
    assert_eq!(compose_instr.writes(), vec![RegisterId::new(3)]);
    assert!(compose_instr.is_linear());
    assert_eq!(compose_instr.operation_type(), "morphism_composition");
    assert!(compose_instr.verify_category_laws());
}

#[test]
fn test_tensor_instruction() {
    // Test the Tensor instruction (parallel composition)
    let tensor_instr = Instruction::Tensor {
        left_reg: RegisterId::new(1),
        right_reg: RegisterId::new(2),
        output_reg: RegisterId::new(3),
    };
    
    // Verify instruction properties
    assert_eq!(tensor_instr.reads(), vec![RegisterId::new(1), RegisterId::new(2)]);
    assert_eq!(tensor_instr.writes(), vec![RegisterId::new(3)]);
    assert!(tensor_instr.is_linear());
    assert_eq!(tensor_instr.operation_type(), "parallel_composition");
    assert!(tensor_instr.verify_category_laws());
}

#[test]
fn test_register_id() {
    // Test RegisterId functionality
    let reg1 = RegisterId::new(42);
    let reg2 = RegisterId::new(42);
    let reg3 = RegisterId::new(99);
    
    assert_eq!(reg1.id(), 42);
    assert_eq!(reg1, reg2);
    assert_ne!(reg1, reg3);
    assert!(reg1 < reg3);
}

#[test]
fn test_instruction_linearity() {
    // Test that all instructions preserve linearity
    let instructions = vec![
        Instruction::Transform {
            morph_reg: RegisterId::new(1),
            input_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        },
        Instruction::Alloc {
            type_reg: RegisterId::new(1),
            init_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        },
        Instruction::Consume {
            resource_reg: RegisterId::new(1),
            output_reg: RegisterId::new(2),
        },
        Instruction::Compose {
            first_reg: RegisterId::new(1),
            second_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        },
        Instruction::Tensor {
            left_reg: RegisterId::new(1),
            right_reg: RegisterId::new(2),
            output_reg: RegisterId::new(3),
        },
    ];
    
    for instr in instructions {
        assert!(instr.is_linear(), "Instruction should respect linearity: {:?}", instr);
        assert!(instr.verify_category_laws(), "Instruction should satisfy category laws: {:?}", instr);
    }
}

#[test]
fn test_category_theory_properties() {
    // Test that the instruction set satisfies category theory properties
    
    // Test that all instructions can be created and verified
    let transform = Instruction::Transform {
        morph_reg: RegisterId::new(1),
        input_reg: RegisterId::new(2),
        output_reg: RegisterId::new(3),
    };
    
    let alloc = Instruction::Alloc {
        type_reg: RegisterId::new(4),
        init_reg: RegisterId::new(5),
        output_reg: RegisterId::new(6),
    };
    
    let compose = Instruction::Compose {
        first_reg: RegisterId::new(7),
        second_reg: RegisterId::new(8),
        output_reg: RegisterId::new(9),
    };
    
    // All should satisfy category laws
    assert!(transform.verify_category_laws());
    assert!(alloc.verify_category_laws());
    assert!(compose.verify_category_laws());
    
    // All should preserve linearity
    assert!(transform.is_linear());
    assert!(alloc.is_linear());
    assert!(compose.is_linear());
} 