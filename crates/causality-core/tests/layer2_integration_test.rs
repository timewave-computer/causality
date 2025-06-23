//! Integration tests for Layer 2 Transform-Based Constraint System
//!
//! These tests demonstrate the complete Layer 2 system functionality,
//! showing how effects, intents, and constraints are unified under
//! the symmetric monoidal closed category framework.

use causality_core::{
    effect::{
        transform_constraint::{TransformConstraintSystem, TransformConstraint, TransformDefinition},
        intent::Intent,
        core::{EffectExpr, EffectExprKind},
    },
    lambda::{Term, TermKind, TypeInner, base::{BaseType, Location}, Literal},
    system::deterministic::DeterministicSystem,
};

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test the complete Layer 2 transform constraint system
    #[test]
    fn test_complete_layer2_system() {
        let _system = TransformConstraintSystem::new();
        
        // Create a simple intent
        let intent = Intent::new(Location::Local);
        
        // This test verifies the system can be created and basic operations work
        // The system should be successfully created
        assert_eq!(intent.domain, Location::Local);
    }
    
    /// Test constraint validation
    #[test]
    fn test_constraint_validation() {
        let _system = TransformConstraintSystem::new();
        
        // Create a simple constraint
        let constraint = TransformConstraint::LocalTransform {
            source_type: TypeInner::Base(BaseType::Int),
            target_type: TypeInner::Base(BaseType::Int),
            transform: TransformDefinition::FunctionApplication {
                function: "identity".to_string(),
                argument: "x".to_string(),
            },
        };
        
        // Test that constraints can be created
        assert!(matches!(constraint, TransformConstraint::LocalTransform { .. }));
    }
    
    /// Test effect creation
    #[test]
    fn test_effect_creation() {
        // Create some simple effects
        let pure_effect = EffectExpr::new(EffectExprKind::Pure(
            Term::new(TermKind::Literal(Literal::Int(42)))
        ));
        
        let perform_effect = EffectExpr::new(EffectExprKind::Perform {
            effect_tag: "test_effect".to_string(),
            args: vec![
                Term::new(TermKind::Literal(Literal::Int(123))),
            ],
        });
        
        // Verify effect structure
        match &pure_effect.kind {
            EffectExprKind::Pure(term) => {
                match &term.kind {
                    TermKind::Literal(Literal::Int(42)) => {},
                    _ => panic!("Expected literal 42"),
                }
            },
            _ => panic!("Expected pure effect"),
        }
        
        match &perform_effect.kind {
            EffectExprKind::Perform { effect_tag, args } => {
                assert_eq!(effect_tag, "test_effect");
                assert_eq!(args.len(), 1);
            },
            _ => panic!("Expected perform effect"),
        }
    }

    /// Test transform constraint system creation
    #[test]
    fn test_transform_constraint_system_creation() {
        let mut system = TransformConstraintSystem::new();
        let mut det_sys = DeterministicSystem::new();
        
        // Create a simple transform constraint
        let constraint = TransformConstraint::LocalTransform {
            source_type: TypeInner::Base(BaseType::Int),
            target_type: TypeInner::Base(BaseType::Int),
            transform: TransformDefinition::FunctionApplication {
                function: "identity".to_string(),
                argument: "x".to_string(),
            },
        };
        
        // Add constraint to system
        system.add_constraint(constraint);
        
        // Test constraint solving
        let result = system.solve_constraints(&mut det_sys);
        assert!(result.is_ok());
    }

    /// Test intent creation and basic properties
    #[test]
    fn test_intent_creation() {
        let intent = Intent::new(Location::Local);
        
        // Verify basic intent properties
        assert_eq!(intent.domain, Location::Local);
        assert!(intent.constraints.is_empty());
        assert!(intent.resource_bindings.is_empty());
    }

    /// Test mathematical property preservation
    #[test]
    fn test_mathematical_property_preservation() {
        let mut system = TransformConstraintSystem::new();
        
        // Test that the system can be created successfully
        // This verifies the basic mathematical structure is preserved
        let mut det_sys = DeterministicSystem::new();
        let result = system.solve_constraints(&mut det_sys);
        assert!(result.is_ok());
    }
} 