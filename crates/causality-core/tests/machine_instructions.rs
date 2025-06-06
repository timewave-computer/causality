//! Integration tests for register machine instructions

use causality_core::machine::{
    ReductionEngine, Instruction, RegisterId, ConstraintExpr, MachineValue,
    reduction::WitnessProvider,
    instruction::Effect,
};
use causality_core::lambda::Symbol;

#[test]
fn test_match_instruction() {
    let mut engine = ReductionEngine::new(vec![
        // Create a sum value (left variant)
        Instruction::Witness { out_reg: RegisterId::new(1) },
        // Match on the sum
        Instruction::Match {
            sum_reg: RegisterId::new(1),
            left_reg: RegisterId::new(2),
            right_reg: RegisterId::new(3),
            left_label: "left".to_string(),
            right_label: "right".to_string(),
        },
        // Define labels for the branches
        Instruction::LabelMarker("left".to_string()),
        // Intentionally no instruction here for the left branch for this test,
        // execution will fall through or halt if it's the end of the program.
        // We could add a Nop or another instruction if needed.
        Instruction::LabelMarker("right".to_string())
        // Similarly, no specific instruction for the right branch target.
    ], 10);
    
    struct MatchWitness;
    impl WitnessProvider for MatchWitness {
        fn get_witness(&mut self, _reg: RegisterId) -> MachineValue {
            MachineValue::Sum {
                tag: Symbol::new("left"),
                value: Box::new(MachineValue::Int(42)),
            }
        }
    }
    
    engine.set_witness_provider(Box::new(MatchWitness));
    
    let result = engine.run();
    assert!(result.is_ok());
    
    let state = result.unwrap();
    // Left register should have the value
    let left_reg = state.load_register(RegisterId::new(2)).unwrap();
    assert_eq!(left_reg.value, MachineValue::Int(42));
}

#[test]
fn test_check_constraint() {
    let mut engine = ReductionEngine::new(vec![
        // Create two equal values
        Instruction::Witness { out_reg: RegisterId::new(1) },
        Instruction::Witness { out_reg: RegisterId::new(2) },
        // Check equality constraint
        Instruction::Check {
            constraint: ConstraintExpr::Equal(RegisterId::new(1), RegisterId::new(2)),
        },
    ], 10);
    
    struct EqualWitness;
    impl WitnessProvider for EqualWitness {
        fn get_witness(&mut self, _reg: RegisterId) -> MachineValue {
            MachineValue::Int(42)
        }
    }
    
    engine.set_witness_provider(Box::new(EqualWitness));
    
    let result = engine.run();
    assert!(result.is_ok());
}

#[test]
fn test_check_constraint_failure() {
    let mut engine = ReductionEngine::new(vec![
        // Create two different values
        Instruction::Witness { out_reg: RegisterId::new(1) },
        Instruction::Witness { out_reg: RegisterId::new(2) },
        // Check equality constraint (should fail)
        Instruction::Check {
            constraint: ConstraintExpr::Equal(RegisterId::new(1), RegisterId::new(2)),
        },
    ], 10);
    
    struct UnequalWitness;
    impl WitnessProvider for UnequalWitness {
        fn get_witness(&mut self, reg: RegisterId) -> MachineValue {
            match reg.id() {
                1 => MachineValue::Int(42),
                2 => MachineValue::Int(99),
                _ => MachineValue::Unit,
            }
        }
    }
    
    engine.set_witness_provider(Box::new(UnequalWitness));
    
    let result = engine.run();
    assert!(result.is_err());
}

#[test]
fn test_perform_effect() {
    let mut engine = ReductionEngine::new(vec![
        // Create effect parameters
        Instruction::Witness { out_reg: RegisterId::new(1) },
        // Perform effect
        Instruction::Perform {
            effect: Effect {
                tag: "transfer".to_string(),
                pre: ConstraintExpr::True,
                post: ConstraintExpr::True,
                hints: vec![],
            },
            out_reg: RegisterId::new(2),
        },
    ], 10);
    
    struct EffectWitness;
    impl WitnessProvider for EffectWitness {
        fn get_witness(&mut self, _reg: RegisterId) -> MachineValue {
            MachineValue::Int(100)
        }
    }
    
    engine.set_witness_provider(Box::new(EffectWitness));
    
    let result = engine.run();
    assert!(result.is_ok());
    
    let state = result.unwrap();
    // Check that effect was recorded
    assert_eq!(state.effects.len(), 1);
    
    // Check result register
    let result_reg = state.load_register(RegisterId::new(2)).unwrap();
    match &result_reg.value {
        MachineValue::EffectResult(tag) => assert_eq!(tag.as_str(), "transfer"),
        _ => panic!("Expected effect result"),
    }
}

#[test]
fn test_linear_consumption_violation() {
    let mut engine = ReductionEngine::new(vec![
        // Create a value
        Instruction::Witness { out_reg: RegisterId::new(1) },
        // Move it to another register
        Instruction::Move {
            src: RegisterId::new(1),
            dst: RegisterId::new(2),
        },
        // Try to use the consumed register again (should fail)
        Instruction::Move {
            src: RegisterId::new(1),
            dst: RegisterId::new(3),
        },
    ], 10);
    
    struct LinearWitness;
    impl WitnessProvider for LinearWitness {
        fn get_witness(&mut self, _reg: RegisterId) -> MachineValue {
            MachineValue::Int(42)
        }
    }
    
    engine.set_witness_provider(Box::new(LinearWitness));
    
    let result = engine.run();
    assert!(result.is_err());
} 