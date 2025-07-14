//! Integration test for compiler pipeline
//!
//! This test demonstrates the complete compilation pipeline from
//! Lisp source code to executable register machine instructions.

use causality_compiler::pipeline::{compile, compile_expression};
use causality_core::machine::Instruction;

#[test]
fn test_complete_compilation_pipeline() {
    // Test basic unit compilation
    let unit_result = compile("nil");
    assert!(unit_result.is_ok());
    let unit_program = unit_result.unwrap();
    assert_eq!(unit_program.source, "nil");
    assert!(!unit_program.instructions.is_empty());

    // Verify instructions are valid - check for new instruction types
    assert!(unit_program.instructions.iter().any(|i| matches!(
        i,
        Instruction::Transform { .. }
            | Instruction::Alloc { .. }
            | Instruction::Consume { .. }
            | Instruction::Compose { .. }
            | Instruction::Tensor { .. }
    )));
}

#[test]
fn test_alloc_instruction_generation() {
    // Test resource allocation compilation
    let alloc_result = compile("(alloc TokenA 42)");
    assert!(alloc_result.is_ok());
    let alloc_program = alloc_result.unwrap();
    assert_eq!(alloc_program.source, "(alloc TokenA 42)");
    assert!(!alloc_program.instructions.is_empty());

    // Verify alloc instruction is generated
    assert!(alloc_program
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Alloc { .. })));
}

#[test]
fn test_consume_instruction_generation() {
    // Test resource consumption compilation
    let consume_result = compile("(consume (alloc TokenA 42))");
    assert!(consume_result.is_ok());
    let consume_program = consume_result.unwrap();
    assert!(!consume_program.instructions.is_empty());

    // Should have both alloc and consume instructions
    assert!(consume_program
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Alloc { .. })));
    assert!(consume_program
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Consume { .. })));
}

#[test]
fn test_tensor_instruction_generation() {
    // Test tensor compilation
    let tensor_result = compile("(tensor (alloc TokenA 42) (alloc TokenB 24))");
    assert!(tensor_result.is_ok());
    let tensor_program = tensor_result.unwrap();
    assert!(!tensor_program.instructions.is_empty());

    // Should generate tensor instruction
    assert!(tensor_program
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Tensor { .. })));
}

#[test]
fn test_lambda_compilation() {
    // Test lambda compilation
    let lambda_result = compile("(lambda (x) x)");
    assert!(lambda_result.is_ok());
    let lambda_program = lambda_result.unwrap();
    assert!(!lambda_program.instructions.is_empty());

    // Lambda should compile to alloc (for function creation)
    assert!(lambda_program
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Alloc { .. })));
}

#[test]
fn test_quick_expression_compilation() {
    // Test the quick compilation method
    let instructions = compile_expression("(alloc TokenA 42)").unwrap();
    assert!(!instructions.is_empty());
    assert!(instructions
        .iter()
        .any(|i| matches!(i, Instruction::Alloc { .. })));
}

#[test]
fn test_compose_instruction_generation() {
    // Test compose compilation (sequential composition)
    // Since compose is not a primitive, we simulate it with nested lambdas
    let compose_result = compile("(lambda (x) ((lambda (y) y) x))");
    assert!(compose_result.is_ok());
    let compose_program = compose_result.unwrap();
    assert!(!compose_program.instructions.is_empty());

    // Should generate alloc instructions for the lambda functions
    assert!(compose_program
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Alloc { .. })));
}
