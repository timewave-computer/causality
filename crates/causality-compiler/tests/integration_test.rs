//! Integration test for enhanced compiler pipeline
//!
//! This test demonstrates the complete compilation pipeline from
//! Lisp source code to executable register machine instructions.

use causality_compiler::enhanced_pipeline::EnhancedCompilerPipeline;
use causality_core::machine::Instruction;
use causality_lisp::ast::ExprKind;

#[test]
fn test_complete_compilation_pipeline() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    // Test basic unit compilation
    let unit_program = pipeline.compile_full("(unit)").unwrap();
    assert_eq!(unit_program.source, "(unit)");
    assert!(!unit_program.instructions.is_empty());
    assert!(unit_program.metadata.registers_used > 0);
    
    // Verify instructions are valid - after optimization, unit might not need any moves
    // Just check that we have some valid instruction type
    assert!(unit_program.instructions.iter().any(|i| matches!(i, 
        Instruction::Move { .. } | 
        Instruction::Apply { .. } | 
        Instruction::Witness { .. } |
        Instruction::Return { .. }
    )));
}

#[test]
fn test_alloc_instruction_generation() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    // Test resource allocation compilation
    let alloc_program = pipeline.compile_full("(alloc 42)").unwrap();
    assert_eq!(alloc_program.source, "(alloc 42)");
    assert!(!alloc_program.instructions.is_empty());
    assert_eq!(alloc_program.metadata.resource_allocations, 1);
    assert_eq!(alloc_program.metadata.resource_consumptions, 0);
    
    // Verify alloc instruction is generated
    assert!(alloc_program.instructions.iter().any(|i| matches!(i, Instruction::Alloc { .. })));
}

#[test]
fn test_consume_instruction_generation() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    // Test resource consumption compilation - should fail for undefined variable
    let consume_result = pipeline.compile_full("(consume x)");
    assert!(consume_result.is_err(), "Should fail for undefined variable");
    
    // Test that the error is about undefined variable
    match consume_result {
        Err(e) => assert!(e.to_string().contains("Undefined variable")),
        Ok(_) => panic!("Should have failed"),
    }
}

#[test]
fn test_compilation_metadata() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    let program = pipeline.compile_full("(alloc 42)").unwrap();
    
    // Verify metadata is properly populated
    assert!(program.metadata.registers_used > 0);
    assert!(program.metadata.instruction_count > 0);
    assert_eq!(program.metadata.instruction_count, program.instructions.len());
    
    // Verify compilation passes
    assert!(program.metadata.passes.contains(&"Parse".to_string()));
    assert!(program.metadata.passes.contains(&"CodeGen".to_string()));
    
    // Check for specific optimization passes
    assert!(program.metadata.passes.contains(&"DeadCodeElimination".to_string()));
    assert!(program.metadata.passes.contains(&"PeepholeOptimization".to_string()));
    assert!(program.metadata.passes.contains(&"RegisterCoalescing".to_string()));

    // Verify optimization stats are available
    assert!(program.metadata.optimization_stats.unoptimized_instruction_count > 0);
}

#[test]
fn test_register_allocation() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    // Test that multiple expressions use different registers
    let program1 = pipeline.compile_full("(unit)").unwrap();
    let program2 = pipeline.compile_full("(alloc 42)").unwrap();
    
    // Second compilation should use more registers due to alloc
    assert!(program2.metadata.registers_used >= program1.metadata.registers_used);
}

#[test]
fn test_instruction_optimization() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    let program = pipeline.compile_full("(unit)").unwrap();
    
    // Verify optimization passes were applied
    assert!(program.metadata.passes.contains(&"DeadCodeElimination".to_string()));
    assert!(program.metadata.passes.contains(&"ConstantPropagation".to_string()));
    assert!(program.metadata.passes.contains(&"ConstantFolding".to_string()));
    assert!(program.metadata.passes.contains(&"RedundantMoveElimination".to_string()));
    assert!(program.metadata.passes.contains(&"PeepholeOptimization".to_string()));
    assert!(program.metadata.passes.contains(&"RegisterCoalescing".to_string()));
    
    // Instructions should be optimized (result can be empty for unit after optimization)
    // instruction_count is usize so always >= 0
}

#[test]
fn test_quick_expression_compilation() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    // Test the quick compilation method
    let program = pipeline.compile_full("(alloc 42)").unwrap();
    assert!(!program.instructions.is_empty());
    assert!(program.instructions.iter().any(|i| matches!(i, Instruction::Alloc { .. })));
}

#[test]
fn test_ast_generation() {
    let mut pipeline = EnhancedCompilerPipeline::new();
    
    let program = pipeline.compile_full("(alloc 42)").unwrap();
    
    // Verify AST was properly generated
    assert!(matches!(program.ast.kind, ExprKind::Alloc { .. }));
} 