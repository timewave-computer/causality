//! Integration tests for Layer 2 Transform-Based Constraint System
//!
//! These tests demonstrate the complete Layer 2 system functionality,
//! showing how effects, intents, and constraints are unified under
//! the symmetric monoidal closed category framework.

use super::{
    transform_constraint::{TransformConstraintSystem, RecordSchema, FieldDefinition, TransformDefinition},
    intent::{Intent, Constraint, ResourceBinding},
    capability::{Capability, CapabilityLevel, RecordCapability},
    core::{EffectExpr, EffectExprKind},
};
use crate::{
    lambda::{Term, TermKind, TypeInner, BaseType, Location, Symbol},
    system::content_addressing::EntityId,
    system::deterministic::DeterministicSystem,
};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test the complete Layer 2 transform constraint system
    #[test]
    fn test_complete_layer2_system() {
        let mut system = TransformConstraintSystem::new(Location::Local);
        
        // Register capabilities
        let read_capability = Capability {
            name: "account_read".to_string(),
            level: CapabilityLevel::Read,
            record_capability: Some(RecordCapability::ReadField("balance".to_string())),
        };
        
        system.register_capability("account_read".to_string(), read_capability);
        
        // Register schema
        let mut fields = BTreeMap::new();
        fields.insert("balance".to_string(), FieldDefinition {
            field_type: TypeInner::Base(BaseType::Int),
            required: true,
            access_requirements: vec!["account_read".to_string()],
        });
        
        let account_schema = RecordSchema {
            name: "Account".to_string(),
            fields,
            required_capabilities: BTreeMap::new(),
            constraints: Vec::new(),
        };
        
        system.register_schema(account_schema);
        
        // Create intent
        let intent = Intent::new(
            Location::Local,
            vec![],
            Constraint::True,
        );
        
        // Solve intent
        let result = system.solve_intent(&intent);
        assert!(result.is_ok());
    }
    
    /// Test capability-based field access compilation
    #[test]
    fn test_capability_field_access() {
        let mut system = TransformConstraintSystem::new(Location::Local);
        
        // Register read capability
        let read_capability = Capability {
            name: "balance_read".to_string(),
            level: CapabilityLevel::Read,
            record_capability: Some(RecordCapability::ReadField("balance".to_string())),
        };
        system.register_capability("balance_read".to_string(), read_capability);
        
        // Register simple schema
        let mut fields = BTreeMap::new();
        fields.insert("balance".to_string(), FieldDefinition {
            field_type: TypeInner::Base(BaseType::Int),
            required: true,
            access_requirements: vec!["balance_read".to_string()],
        });
        
        let schema = RecordSchema {
            name: "SimpleAccount".to_string(),
            fields,
            required_capabilities: BTreeMap::new(),
            constraints: Vec::new(),
        };
        system.register_schema(schema);
        
        // Create intent with capability requirement
        let intent = Intent::new(
            Location::Local,
            vec![],
            Constraint::HasCapability(
                crate::effect::intent::ResourceRef::new("account".to_string()),
                "balance_read".to_string()
            ),
        );
        
        // Solve and verify
        let result = system.solve_intent(&intent);
        assert!(result.is_ok());
        
        let effects = result.unwrap();
        
        // Should generate field access effects
        assert!(!effects.is_empty(), "Should generate field access effects");
        
        // Verify effect structure
        for effect in &effects {
            match &effect.kind {
                EffectExprKind::Perform { effect_tag, args } => {
                    assert!(effect_tag.contains("access_field"), "Should be field access effect");
                    assert!(!args.is_empty(), "Should have arguments");
                }
                _ => {} // Other effect types are fine too
            }
        }
    }
    
    /// Test schema resolution and transform generation
    #[test]
    fn test_schema_resolution() {
        let mut system = TransformConstraintSystem::new(Location::Local);
        
        // Register capabilities
        let read_cap = Capability {
            name: "user_read".to_string(),
            level: CapabilityLevel::Read,
            record_capability: Some(RecordCapability::ReadField("name".to_string())),
        };
        let write_cap = Capability {
            name: "user_write".to_string(),
            level: CapabilityLevel::Write,
            record_capability: Some(RecordCapability::WriteField("email".to_string())),
        };
        
        system.register_capability("user_read".to_string(), read_cap);
        system.register_capability("user_write".to_string(), write_cap);
        
        // Register complex schema with multiple fields
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), FieldDefinition {
            field_type: TypeInner::Base(BaseType::Symbol),
            required: true,
            access_requirements: vec!["user_read".to_string()],
        });
        fields.insert("email".to_string(), FieldDefinition {
            field_type: TypeInner::Base(BaseType::Symbol),
            required: true,
            access_requirements: vec!["user_read".to_string(), "user_write".to_string()],
        });
        fields.insert("age".to_string(), FieldDefinition {
            field_type: TypeInner::Base(BaseType::Int),
            required: false,
            access_requirements: vec!["user_read".to_string()],
        });
        
        let user_schema = RecordSchema {
            name: "User".to_string(),
            fields,
            required_capabilities: BTreeMap::new(),
            constraints: Vec::new(),
        };
        system.register_schema(user_schema);
        
        // Create intent requiring multiple capabilities
        let intent = Intent::new(
            Location::Local,
            vec![],
            Constraint::And(vec![
                Constraint::HasCapability(
                    crate::effect::intent::ResourceRef::new("user1".to_string()),
                    "user_read".to_string()
                ),
                Constraint::HasCapability(
                    crate::effect::intent::ResourceRef::new("user1".to_string()),
                    "user_write".to_string()
                ),
            ]),
        );
        
        // Solve and verify complex schema resolution
        let result = system.solve_intent(&intent);
        assert!(result.is_ok());
        
        let effects = result.unwrap();
        
        // Should generate effects for multiple fields
        let field_effects: Vec<_> = effects.iter()
            .filter(|e| match &e.kind {
                EffectExprKind::Perform { effect_tag, .. } => effect_tag.contains("access_field"),
                _ => false,
            })
            .collect();
        
        assert!(!field_effects.is_empty(), "Should generate field access effects");
        
        // Verify that we get effects for the schema fields
        let effect_tags: Vec<String> = field_effects.iter()
            .filter_map(|e| match &e.kind {
                EffectExprKind::Perform { effect_tag, .. } => Some(effect_tag.clone()),
                _ => None,
            })
            .collect();
        
        // Should have effects for User schema fields
        assert!(effect_tags.iter().any(|tag| tag.contains("User")), "Should reference User schema");
    }
    
    /// Test constraint validation
    #[test]
    fn test_constraint_validation() {
        let mut system = TransformConstraintSystem::new(Location::Local);
        
        // Test that False constraint fails
        let failing_intent = Intent::new(
            Location::Local,
            vec![],
            Constraint::False,
        );
        
        let result = system.solve_intent(&failing_intent);
        assert!(result.is_err(), "False constraint should fail");
        
        // Test that True constraint succeeds
        let succeeding_intent = Intent::new(
            Location::Local,
            vec![],
            Constraint::True,
        );
        
        let result = system.solve_intent(&succeeding_intent);
        assert!(result.is_ok(), "True constraint should succeed");
    }
    
    /// Test Layer 1 compilation
    #[test]
    fn test_layer1_compilation() {
        let mut system = TransformConstraintSystem::new(Location::Local);
        
        // Create some simple effects
        let effects = vec![
            EffectExpr::new(EffectExprKind::Pure { 
                value: crate::lambda::base::Value::Int(42) 
            }),
            EffectExpr::new(EffectExprKind::Perform {
                effect_tag: "test_effect".to_string(),
                args: vec![
                    Term::new(TermKind::Literal(crate::lambda::term::Literal::Int(123))),
                ],
            }),
        ];
        
        // Compile to Layer 1
        let result = system.compile_to_layer1(&effects);
        assert!(result.is_ok(), "Layer 1 compilation should succeed");
        
        let terms = result.unwrap();
        assert_eq!(terms.len(), 2, "Should compile all effects");
        
        // Verify term structure
        match &terms[0].kind {
            TermKind::Literal(crate::lambda::term::Literal::Int(42)) => {},
            _ => panic!("First term should be literal 42"),
        }
        
        match &terms[1].kind {
            TermKind::Apply { .. } => {},
            _ => panic!("Second term should be function application"),
        }
    }

    #[test]
    fn test_transform_constraint_system_creation() {
        let mut system = TransformConstraintSystem::new();
        
        // Register a simple transform
        let transform = TransformDefinition {
            id: "simple_transform".to_string(),
            input_type: TypeInner::Unknown,
            output_type: TypeInner::Unknown,
            required_capabilities: vec![],
            layer1_operations: vec![
                Layer1Operation::Apply {
                    function: Term {
                        kind: TermKind::Var("f".to_string()),
                        location: Location::Unknown,
                    },
                    argument: Term {
                        kind: TermKind::Var("x".to_string()),
                        location: Location::Unknown,
                    },
                }
            ],
            preserved_properties: vec![MathematicalProperty::Linearity],
        };
        
        system.register_transform(transform);
        
        // Register a schema
        let mut fields = BTreeMap::new();
        fields.insert("test_field".to_string(), FieldDefinition {
            name: "test_field".to_string(),
            field_type: TypeInner::Unknown,
            required: true,
            default_value: None,
        });
        
        let schema = RecordSchema {
            id: "test_schema".to_string(),
            fields,
            access_patterns: vec![],
            field_capabilities: BTreeMap::new(),
        };
        
        system.register_schema(schema);
        
        // Test constraint solving
        let mut det_sys = DeterministicSystem::new();
        let effect = EffectExpr {
            kind: EffectExprKind::Pure(Term {
                kind: TermKind::Var("pure_value".to_string()),
                location: Location::Unknown,
            }),
        };
        
        let constraints = system.solve_effect_constraints(&effect, &mut det_sys);
        assert!(constraints.is_ok());
        
        let constraint_list = constraints.unwrap();
        // Pure effects should have no constraints
        assert_eq!(constraint_list.len(), 0);
    }

    #[test] 
    fn test_bind_effect_constraints() {
        let mut system = TransformConstraintSystem::new();
        let mut det_sys = DeterministicSystem::new();
        
        // Create a bind effect: computation >>= continuation
        let computation = EffectExpr {
            kind: EffectExprKind::Pure(Term {
                kind: TermKind::Var("comp".to_string()),
                location: Location::Unknown,
            }),
        };
        
        let continuation = EffectExpr {
            kind: EffectExprKind::Pure(Term {
                kind: TermKind::Var("cont".to_string()),
                location: Location::Unknown,
            }),
        };
        
        let bind_effect = EffectExpr {
            kind: EffectExprKind::Bind {
                computation: Box::new(computation),
                continuation: Box::new(continuation),
            },
        };
        
        let constraints = system.solve_effect_constraints(&bind_effect, &mut det_sys);
        assert!(constraints.is_ok());
        
        let constraint_list = constraints.unwrap();
        // Bind should generate associativity constraint
        assert!(constraint_list.len() > 0);
    }

    #[test]
    fn test_schema_constraint_resolution() {
        let mut system = TransformConstraintSystem::new();
        let mut det_sys = DeterministicSystem::new();
        
        // Register a schema with fields
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), FieldDefinition {
            name: "name".to_string(),
            field_type: TypeInner::Unknown,
            required: true,
            default_value: None,
        });
        
        let schema = RecordSchema {
            id: "person".to_string(),
            fields,
            access_patterns: vec![],
            field_capabilities: BTreeMap::new(),
        };
        
        system.register_schema(schema);
        
        // Resolve constraints for field access
        let constraints = system.resolve_schema_constraints("person", "name", &mut det_sys);
        assert!(constraints.is_ok());
        
        let constraint_list = constraints.unwrap();
        assert!(constraint_list.len() > 0);
    }

    #[test]
    fn test_complete_constraint_pipeline() {
        let mut system = TransformConstraintSystem::new();
        let mut det_sys = DeterministicSystem::new();
        
        // Register a transform for the pipeline
        let transform = TransformDefinition {
            id: "pipeline_transform".to_string(),
            input_type: TypeInner::Unknown,
            output_type: TypeInner::Unknown,
            required_capabilities: vec![],
            layer1_operations: vec![
                Layer1Operation::Apply {
                    function: Term {
                        kind: TermKind::Var("pipeline_f".to_string()),
                        location: Location::Unknown,
                    },
                    argument: Term {
                        kind: TermKind::Var("pipeline_x".to_string()),
                        location: Location::Unknown,
                    },
                }
            ],
            preserved_properties: vec![
                MathematicalProperty::Linearity,
                MathematicalProperty::Causality,
            ],
        };
        
        system.register_transform(transform);
        
        // Create a complex effect for the pipeline
        let effect = EffectExpr {
            kind: EffectExprKind::Apply {
                function: Term {
                    kind: TermKind::Var("test_function".to_string()),
                    location: Location::Unknown,
                },
                argument: Term {
                    kind: TermKind::Var("test_argument".to_string()),
                    location: Location::Unknown,
                },
            },
        };
        
        // Run the complete pipeline
        let solution = system.solve_constraints_pipeline(&effect, &mut det_sys);
        assert!(solution.is_ok());
        
        let constraint_solution = solution.unwrap();
        assert!(constraint_solution.verification_passed);
        assert!(constraint_solution.layer1_operations.len() > 0);
        assert!(constraint_solution.applicable_transforms.len() > 0);
    }

    #[test]
    fn test_mathematical_property_preservation() {
        let mut system = TransformConstraintSystem::new();
        
        // Register transforms with different mathematical properties
        let associative_transform = TransformDefinition {
            id: "associative".to_string(),
            input_type: TypeInner::Unknown,
            output_type: TypeInner::Unknown,
            required_capabilities: vec![],
            layer1_operations: vec![],
            preserved_properties: vec![MathematicalProperty::Associativity],
        };
        
        let commutative_transform = TransformDefinition {
            id: "commutative".to_string(),
            input_type: TypeInner::Unknown,
            output_type: TypeInner::Unknown,
            required_capabilities: vec![],
            layer1_operations: vec![],
            preserved_properties: vec![MathematicalProperty::Commutativity],
        };
        
        let linear_transform = TransformDefinition {
            id: "linear".to_string(),
            input_type: TypeInner::Unknown,
            output_type: TypeInner::Unknown,
            required_capabilities: vec![],
            layer1_operations: vec![],
            preserved_properties: vec![MathematicalProperty::Linearity],
        };
        
        system.register_transform(associative_transform);
        system.register_transform(commutative_transform);
        system.register_transform(linear_transform);
        
        // Verify all transforms are registered
        assert_eq!(system.transforms.len(), 3);
        
        // Check that transforms preserve their mathematical properties
        for transform in system.transforms.values() {
            assert!(!transform.preserved_properties.is_empty());
        }
    }
}

/// Helper function to create a basic capability for testing
#[cfg(test)]
pub fn create_test_capability(name: &str, level: CapabilityLevel) -> Capability {
    Capability {
        name: name.to_string(),
        level,
        record_capability: None,
    }
}

/// Helper function to create a basic schema for testing
#[cfg(test)]
pub fn create_test_schema(name: &str, field_name: &str, field_type: TypeInner) -> RecordSchema {
    let mut fields = BTreeMap::new();
    fields.insert(field_name.to_string(), FieldDefinition {
        field_type,
        required: true,
        access_requirements: Vec::new(),
    });
    
    RecordSchema {
        name: name.to_string(),
        fields,
        required_capabilities: BTreeMap::new(),
        constraints: Vec::new(),
    }
}

/// Helper function to create a simple transform for testing
#[cfg(test)]
pub fn create_test_transform(name: &str, input_type: TypeInner, output_type: TypeInner) -> TransformDefinition {
    TransformDefinition {
        id: EntityId::from_content(name),
        input_type,
        output_type,
        location: Location::Local,
        implementation: Term::new(TermKind::Var("identity".to_string())),
        required_capabilities: Vec::new(),
        constraints: Vec::new(),
    }
} 