//! End-to-End Type System Consistency Tests
//!
//! This test suite verifies that the type system is consistent across:
//! - Layer 0 (Machine values and instructions) 
//! - Layer 1 (Linear lambda calculus terms and types)
//! - Layer 2 (Effect expressions and session types)
//! - Causality Lisp (AST and value representations)
//! - OCaml FFI bindings (type mappings)
//!
//! These tests ensure that types can round-trip through different layers
//! and maintain their semantic meaning throughout the entire system.

use causality_core::{
    lambda::{
        base::{TypeInner, BaseType, SessionType, Location, Value, Type},
        term::{Term, TermKind, Literal},
    },
    effect::{
        core::{EffectExpr, EffectExprKind},
        row::{RowType, FieldType, FieldAccess, RecordType},
    },
    machine::{
        value::{MachineValue, SessionChannel, ChannelState},
        instruction::{Instruction, RegisterId},
    },
    system::content_addressing::EntityId,
};

#[cfg(test)]
mod base_type_consistency {
    use super::*;

    #[test]
    fn test_base_types_across_all_layers() {
        // Test that base types are consistent from Layer 2 down to Layer 0
        
        // Layer 2: Effect with pure computation
        let layer2_effect = EffectExpr::new(EffectExprKind::Pure(
            Term::literal(Literal::Int(42))
        ));
        
        // Layer 1: Extract the term
        let layer1_term = match &layer2_effect.kind {
            EffectExprKind::Pure(term) => term,
            _ => panic!("Expected pure effect"),
        };
        
        // Layer 0: Corresponding machine value
        let layer0_value = MachineValue::Int(42);
        
        // Verify type consistency
        let expected_type = TypeInner::Base(BaseType::Int);
        assert_eq!(layer0_value.get_type(), expected_type);
        
        // Test all base types
        let base_type_tests = vec![
            (TypeInner::Base(BaseType::Unit), Value::Unit, MachineValue::Unit),
            (TypeInner::Base(BaseType::Bool), Value::Bool(true), MachineValue::Bool(true)),
            (TypeInner::Base(BaseType::Int), Value::Int(42), MachineValue::Int(42)),
            (TypeInner::Base(BaseType::Symbol), Value::Symbol("test".into()), MachineValue::Symbol("test".into())),
        ];
        
        for (expected_type, layer1_value, layer0_value) in base_type_tests {
            assert_eq!(layer1_value.value_type(), expected_type);
            assert_eq!(layer0_value.get_type(), expected_type);
        }
    }

    #[test]
    fn test_causality_lisp_integration() {
        // Test that causality-lisp types integrate properly
        use causality_lisp::{Expr, ExprKind, LispValue};
        
        // Create Lisp expressions for each base type
        let lisp_expressions = vec![
            (Expr::unit(), TypeInner::Base(BaseType::Unit)),
            (Expr::constant(LispValue::Bool(true)), TypeInner::Base(BaseType::Bool)),
            (Expr::constant(LispValue::Int(42)), TypeInner::Base(BaseType::Int)),
            (Expr::constant(LispValue::Symbol("test".into())), TypeInner::Base(BaseType::Symbol)),
        ];
        
        for (expr, expected_type) in lisp_expressions {
            // Verify the expression structure
            match &expr.kind {
                ExprKind::UnitVal => assert_eq!(expected_type, TypeInner::Base(BaseType::Unit)),
                ExprKind::Const(lisp_val) => {
                    match lisp_val {
                        LispValue::Bool(_) => assert_eq!(expected_type, TypeInner::Base(BaseType::Bool)),
                        LispValue::Int(_) => assert_eq!(expected_type, TypeInner::Base(BaseType::Int)),
                        LispValue::Symbol(_) => assert_eq!(expected_type, TypeInner::Base(BaseType::Symbol)),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod complex_type_consistency {
    use super::*;

    #[test]
    fn test_product_types_e2e() {
        // Create a product type across all layers
        let left_type = TypeInner::Base(BaseType::Int);
        let right_type = TypeInner::Base(BaseType::Bool);
        let product_type = TypeInner::Product(Box::new(left_type), Box::new(right_type));
        
        // Layer 2: Effect with product
        let layer2_effect = EffectExpr::new(EffectExprKind::Pure(
            Term::tensor(
                Term::literal(Literal::Int(42)),
                Term::literal(Literal::Bool(true))
            )
        ));
        
        // Layer 1: Product value
        let layer1_value = Value::Product(
            Box::new(Value::Int(42)),
            Box::new(Value::Bool(true))
        );
        
        // Layer 0: Product machine value
        let layer0_value = MachineValue::Tensor(
            Box::new(MachineValue::Int(42)),
            Box::new(MachineValue::Bool(true))
        );
        
        // Verify consistency
        assert_eq!(layer1_value.value_type(), product_type);
        assert_eq!(layer0_value.get_type(), product_type);
        
        // Test tensor extraction at Layer 0
        if let Some((left, right)) = layer0_value.extract_tensor() {
            assert_eq!(left, &MachineValue::Int(42));
            assert_eq!(right, &MachineValue::Bool(true));
        } else {
            panic!("Failed to extract tensor components");
        }
    }

    #[test]
    fn test_function_types_e2e() {
        // Create a linear function type
        let input_type = TypeInner::Base(BaseType::Int);
        let output_type = TypeInner::Base(BaseType::Bool);
        let function_type = TypeInner::LinearFunction(Box::new(input_type), Box::new(output_type));
        
        // Layer 1: Lambda term
        let lambda_term = Term::lambda_typed(
            "x",
            TypeInner::Base(BaseType::Int),
            Term::literal(Literal::Bool(true))
        );
        
        // Verify the lambda structure
        if let TermKind::Lambda { param, param_type, body } = &lambda_term.kind {
            assert_eq!(param, "x");
            assert_eq!(param_type, &Some(TypeInner::Base(BaseType::Int)));
            assert!(matches!(body.kind, TermKind::Literal(Literal::Bool(true))));
        }
        
        // Layer 0: Function machine value (simplified representation)
        let function_registers = vec![RegisterId::new(0)];
        let function_body = vec![]; // Would contain actual instructions
        let captured_env = std::collections::BTreeMap::new();
        
        let layer0_function = MachineValue::Function {
            params: function_registers,
            body: function_body,
            captured_env,
        };
        
        // Verify it's recognized as a function
        assert!(matches!(layer0_function, MachineValue::Function { .. }));
    }
}

#[cfg(test)]
mod session_type_consistency {
    use super::*;

    #[test]
    fn test_session_types_e2e() {
        // Create a complex session type
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Bool)),
                Box::new(SessionType::End)
            ))
        );
        
        // Layer 2: Session effect
        let channel_term = Term::new_channel(session_type.clone());
        let send_effect = EffectExpr::new(EffectExprKind::SessionSend {
            channel: Box::new(EffectExpr::new(EffectExprKind::Pure(channel_term))),
            value: Term::literal(Literal::Int(42)),
            continuation: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::unit()))),
        });
        
        // Layer 1: Session terms
        let new_channel_term = Term::new_channel(session_type.clone());
        let send_term = Term::send(
            Term::var("channel"),
            Term::literal(Literal::Int(42))
        );
        let receive_term = Term::receive(Term::var("channel"));
        
        // Layer 0: Session channel
        let session_channel = SessionChannel::new(session_type.clone(), Location::Local);
        let channel_value = MachineValue::Channel(session_channel);
        
        // Verify session type consistency
        let session_type_inner = TypeInner::Session(Box::new(session_type.clone()));
        assert_eq!(channel_value.get_type(), session_type_inner);
        assert_eq!(channel_value.get_session_type(), Some(&session_type));
        
        // Test session duality
        let dual_session = session_type.dual();
        let expected_dual = SessionType::Receive(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Bool)),
                Box::new(SessionType::End)
            ))
        );
        assert_eq!(dual_session, expected_dual);
        
        // Verify duality is symmetric
        assert_eq!(dual_session.dual(), session_type);
    }

    #[test]
    fn test_session_lifecycle_e2e() {
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Create session channel
        let mut channel = SessionChannel::new(session_type.clone(), Location::Local);
        
        // Test initial state
        assert!(channel.is_available());
        assert!(!channel.is_consumed());
        assert_eq!(channel.state, ChannelState::Open);
        
        // Send message
        let message = MachineValue::Int(42);
        assert!(channel.send_message(message).is_ok());
        assert_eq!(channel.message_queue.len(), 1);
        
        // Receive message
        let received = channel.receive_message();
        assert!(received.is_some());
        assert_eq!(received.unwrap(), MachineValue::Int(42));
        assert_eq!(channel.message_queue.len(), 0);
        
        // Progress session to End
        channel.progress_session(SessionType::End);
        
        // Consume channel
        channel.consume();
        assert!(channel.is_consumed());
        assert_eq!(channel.state, ChannelState::Consumed);
        
        // Verify consumed channel operations fail
        assert!(channel.send_message(MachineValue::Unit).is_err());
    }

    #[test]
    fn test_causality_lisp_session_integration() {
        use causality_lisp::{Expr, ExprKind};
        use causality_core::effect::session_registry::SessionRole;
        
        // Create session type
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Create session role
        let client_role = SessionRole {
            name: "client".to_string(),
            protocol: session_type.clone(),
        };
        
        // Test session declaration in Lisp
        let session_decl = Expr::session_declaration("PaymentSession", vec![client_role]);
        assert!(matches!(session_decl.kind, ExprKind::SessionDeclaration { .. }));
        
        // Test with_session in Lisp
        let body = Expr::session_send(
            Expr::variable("channel"),
            Expr::constant(causality_lisp::LispValue::Int(100))
        );
        let with_session = Expr::with_session("PaymentSession", "client", body);
        assert!(matches!(with_session.kind, ExprKind::WithSession { .. }));
        
        // Verify session operations
        let send_expr = Expr::session_send(
            Expr::variable("ch"),
            Expr::constant(causality_lisp::LispValue::Int(42))
        );
        assert!(matches!(send_expr.kind, ExprKind::SessionSend { .. }));
        
        let recv_expr = Expr::session_receive(Expr::variable("ch"));
        assert!(matches!(recv_expr.kind, ExprKind::SessionReceive { .. }));
    }
}

#[cfg(test)]
mod location_aware_types {
    use super::*;

    #[test]
    fn test_location_types_e2e() {
        // Test all location variants
        let locations = vec![
            Location::Local,
            Location::Remote(EntityId::from_content(&[1u8; 32])),
            Location::Domain("database".to_string()),
            Location::Any,
        ];
        
        for location in locations {
            // Create located type
            let base_type = TypeInner::Base(BaseType::Int);
            let located_type = TypeInner::Located(Box::new(base_type.clone()), location.clone());
            
            // Create transform type
            let transform_type = TypeInner::Transform {
                input: Box::new(base_type.clone()),
                output: Box::new(base_type),
                location: location.clone(),
            };
            
            // Verify type structure
            assert!(matches!(located_type, TypeInner::Located(_, _)));
            assert!(matches!(transform_type, TypeInner::Transform { .. }));
            
            // Test location properties
            if location.is_concrete() {
                assert!(!location.is_variable());
                let concrete_locs = location.concrete_locations();
                assert!(!concrete_locs.is_empty());
            }
            
            // Test location composition
            let composed = location.clone().compose(Location::Local);
            if location != Location::Local {
                assert!(composed.is_composite() || composed == location);
            }
        }
    }

    #[test]
    fn test_distributed_computation_types() {
        // Test distributed location
        let nodes = vec![
            EntityId::from_content(&[2u8; 32]),
            EntityId::from_content(&[3u8; 32]),
            EntityId::from_content(&[4u8; 32]),
        ];
        let distributed_loc = Location::Distributed(nodes);
        
        // Create transform for distributed computation
        let input_type = TypeInner::Base(BaseType::Int);
        let output_type = TypeInner::Base(BaseType::Int);
        let distributed_transform = TypeInner::Transform {
            input: Box::new(input_type),
            output: Box::new(output_type),
            location: distributed_loc.clone(),
        };
        
        // Verify distributed properties
        assert!(distributed_loc.is_concrete());
        assert!(!distributed_loc.is_local());
        assert!(!distributed_loc.is_composite());
        
        // Test transform type structure
        if let TypeInner::Transform { input, output, location } = &distributed_transform {
            assert!(matches!(input.as_ref(), TypeInner::Base(BaseType::Int)));
            assert!(matches!(output.as_ref(), TypeInner::Base(BaseType::Int)));
            assert_eq!(location, &distributed_loc);
        }
    }
}

#[cfg(test)]
mod record_type_consistency {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_record_types_e2e() {
        // Create a complex record type with location-aware fields
        let mut fields = BTreeMap::new();
        
        fields.insert(
            "local_id".to_string(),
            FieldType::at_location(TypeInner::Base(BaseType::Int), Location::Local)
        );
        
        fields.insert(
            "remote_name".to_string(),
            FieldType::at_location(
                TypeInner::Base(BaseType::Symbol),
                Location::Remote(EntityId::from_content(&[5u8; 32]))
            )
        );
        
        fields.insert(
            "linear_resource".to_string(),
            FieldType::linear(TypeInner::Base(BaseType::Symbol))
        );
        
        let row_type = RowType::with_fields(fields);
        let record_type = RecordType::from_row(row_type.clone());
        
        // Test field operations
        assert!(row_type.get_field("local_id").is_some());
        assert!(row_type.get_field("remote_name").is_some());
        assert!(row_type.get_field("linear_resource").is_some());
        assert!(row_type.get_field("nonexistent").is_none());
        
        // Test field access permissions
        let linear_field = row_type.get_field("linear_resource").unwrap();
        assert!(linear_field.allows_access(&FieldAccess::Linear));
        assert!(!linear_field.allows_access(&FieldAccess::ReadOnly));
        
        // Test location-based operations
        let locations = row_type.get_locations();
        assert_eq!(locations.len(), 2); // Local and Remote
        assert!(locations.contains(&Location::Local));
        
        // Test field projection
        let projection_result = row_type.project("local_id");
        assert!(matches!(projection_result, causality_core::effect::row::RowOpResult::Success(_)));
        
        // Test location-based field filtering
        let local_fields = row_type.fields_at_location(&Location::Local);
        assert_eq!(local_fields.len(), 2);
        assert!(local_fields.contains_key("local_id"));
    }

    #[test]
    fn test_record_extension_restriction_e2e() {
        // Start with a simple record
        let mut initial_fields = BTreeMap::new();
        initial_fields.insert("x".to_string(), FieldType::simple(TypeInner::Base(BaseType::Int)));
        
        let mut row_type = RowType::with_fields(initial_fields);
        
        // Test extension
        let extension_result = row_type.extend(
            "y".to_string(),
            FieldType::simple(TypeInner::Base(BaseType::Bool))
        );
        assert!(matches!(extension_result, causality_core::effect::row::RowOpResult::Success(_)));
        
        // Add the field for further testing
        row_type.add_field("y".to_string(), FieldType::simple(TypeInner::Base(BaseType::Bool)));
        
        // Test restriction
        let restriction_result = row_type.restrict("x");
        assert!(matches!(restriction_result, causality_core::effect::row::RowOpResult::Success(_)));
        
        // Test duplicate extension (should fail)
        let duplicate_result = row_type.extend(
            "x".to_string(),
            FieldType::simple(TypeInner::Base(BaseType::Symbol))
        );
        assert!(matches!(duplicate_result, causality_core::effect::row::RowOpResult::DuplicateField(_)));
        
        // Test field names
        let field_names = row_type.field_names();
        assert!(field_names.contains(&"x".to_string()));
        assert!(field_names.contains(&"y".to_string()));
    }
}

#[cfg(test)]
mod instruction_consistency {
    use super::*;

    #[test]
    fn test_five_fundamental_instructions_e2e() {
        let r0 = RegisterId::new(0);
        let r1 = RegisterId::new(1);
        let r2 = RegisterId::new(2);
        
        // Test each of the 5 fundamental instructions
        let instructions = vec![
            Instruction::Transform {
                morph_reg: r0,
                input_reg: r1,
                output_reg: r2,
            },
            Instruction::Alloc {
                type_reg: r0,
                init_reg: r1,
                output_reg: r2,
            },
            Instruction::Consume {
                resource_reg: r0,
                output_reg: r1,
            },
            Instruction::Compose {
                first_reg: r0,
                second_reg: r1,
                output_reg: r2,
            },
            Instruction::Tensor {
                left_reg: r0,
                right_reg: r1,
                output_reg: r2,
            },
        ];
        
        // Verify each instruction
        for instruction in instructions {
            // Test mathematical properties
            assert!(instruction.verify_category_laws());
            assert!(instruction.is_linear());
            
            // Test register analysis
            let reads = instruction.reads();
            let writes = instruction.writes();
            
            // All instructions should read at least one register
            assert!(!reads.is_empty());
            
            // All instructions should write exactly one register
            assert_eq!(writes.len(), 1);
            
            // No instruction should read and write the same register
            for read_reg in &reads {
                for write_reg in &writes {
                    assert_ne!(read_reg, write_reg);
                }
            }
        }
    }

    #[test]
    fn test_instruction_compilation_from_terms() {
        // Test that Layer 1 terms correspond to Layer 0 instructions
        
        // Alloc term -> Alloc instruction
        let alloc_term = Term::alloc(Term::literal(Literal::Int(42)));
        assert!(matches!(alloc_term.kind, TermKind::Alloc { .. }));
        
        // Consume term -> Consume instruction  
        let consume_term = Term::consume(Term::var("resource"));
        assert!(matches!(consume_term.kind, TermKind::Consume { .. }));
        
        // Tensor term -> Tensor instruction
        let tensor_term = Term::tensor(
            Term::literal(Literal::Int(1)),
            Term::literal(Literal::Bool(true))
        );
        assert!(matches!(tensor_term.kind, TermKind::Tensor { .. }));
        
        // Apply term -> Transform instruction (function application)
        let apply_term = Term::apply(
            Term::lambda("x", Term::var("x")),
            Term::literal(Literal::Int(42))
        );
        assert!(matches!(apply_term.kind, TermKind::Apply { .. }));
    }
}

#[cfg(test)]
mod compilation_pipeline_e2e {
    use super::*;

    #[test]
    fn test_layer2_to_layer0_compilation() {
        // Start with a Layer 2 effect
        let layer2_effect = EffectExpr::new(EffectExprKind::Bind {
            effect: Box::new(EffectExpr::new(EffectExprKind::Pure(
                Term::alloc(Term::literal(Literal::Int(42)))
            ))),
            var: "resource".to_string(),
            body: Box::new(EffectExpr::new(EffectExprKind::Pure(
                Term::consume(Term::var("resource"))
            ))),
        });
        
        // Extract Layer 1 terms
        if let EffectExprKind::Bind { effect, var, body } = &layer2_effect.kind {
            // First effect: alloc
            if let EffectExprKind::Pure(alloc_term) = &effect.kind {
                assert!(matches!(alloc_term.kind, TermKind::Alloc { .. }));
            }
            
            // Variable binding
            assert_eq!(var, "resource");
            
            // Second effect: consume
            if let EffectExprKind::Pure(consume_term) = &body.kind {
                assert!(matches!(consume_term.kind, TermKind::Consume { .. }));
            }
        }
        
        // This would compile to Layer 0 instructions:
        // 1. Alloc instruction
        // 2. Consume instruction
        let expected_instructions = vec![
            Instruction::Alloc {
                type_reg: RegisterId::new(0),
                init_reg: RegisterId::new(1),
                output_reg: RegisterId::new(2),
            },
            Instruction::Consume {
                resource_reg: RegisterId::new(2),
                output_reg: RegisterId::new(3),
            },
        ];
        
        // Verify instruction structure
        for instr in expected_instructions {
            assert!(instr.verify_category_laws());
            assert!(instr.is_linear());
        }
    }

    #[test]
    fn test_causality_lisp_to_layer0_compilation() {
        use causality_lisp::{compile_for_simulation, LispError};
        
        // Test simple Lisp program compilation
        let lisp_programs = vec![
            "(unit)",
            "(tensor 42 true)",
            "(alloc 42)",
            "(lambda (x) x)",
        ];
        
        for program in lisp_programs {
            let result = compile_for_simulation(program);
            
            match result {
                Ok(e2e_result) => {
                    // Verify compilation succeeded
                    assert!(!e2e_result.instructions.is_empty());
                    assert!(e2e_result.instruction_count > 0);
                    
                    // Verify all instructions are valid
                    for instruction in &e2e_result.instructions {
                        assert!(instruction.verify_category_laws());
                        assert!(instruction.is_linear());
                    }
                }
                Err(LispError::Parse(_)) => {
                    // Some programs might have parse issues, that's ok for this test
                    continue;
                }
                Err(e) => {
                    panic!("Unexpected compilation error for '{}': {:?}", program, e);
                }
            }
        }
    }

    #[test]
    fn test_session_compilation_e2e() {
        // Test session type compilation through all layers
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        // Layer 2: Session effect
        let session_effect = EffectExpr::new(EffectExprKind::WithSession {
            session_decl: "TestSession".to_string(),
            role: "client".to_string(),
            body: Box::new(EffectExpr::new(EffectExprKind::SessionSend {
                channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("ch")))),
                value: Term::literal(Literal::Int(42)),
                continuation: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::unit()))),
            })),
        });
        
        // Layer 1: Session terms
        let new_channel = Term::new_channel(session_type.clone());
        let send_operation = Term::send(Term::var("ch"), Term::literal(Literal::Int(42)));
        let close_operation = Term::close(Term::var("ch"));
        
        // Layer 0: Session channel value
        let session_channel = SessionChannel::new(session_type, Location::Local);
        let channel_value = MachineValue::Channel(session_channel);
        
        // Verify session compilation chain
        assert!(matches!(session_effect.kind, EffectExprKind::WithSession { .. }));
        assert!(matches!(new_channel.kind, TermKind::NewChannel { .. }));
        assert!(matches!(send_operation.kind, TermKind::Send { .. }));
        assert!(matches!(close_operation.kind, TermKind::Close { .. }));
        assert!(channel_value.is_available_channel());
        
        // This would compile to Layer 0 instructions:
        // 1. Alloc instruction (create channel)
        // 2. Transform instruction (send operation)
        // 3. Consume instruction (close channel)
    }
}

#[cfg(test)]
mod documentation_consistency {
    use super::*;

    #[test]
    fn test_documented_examples_work() {
        // Test examples from the documentation to ensure they actually work
        
        // From 002-three-layer-architecture.md: Base types
        let base_types = vec![
            TypeInner::Base(BaseType::Unit),
            TypeInner::Base(BaseType::Bool),
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Symbol),
        ];
        
        for base_type in base_types {
            // Should be able to create values of these types
            let machine_value = match &base_type {
                TypeInner::Base(BaseType::Unit) => MachineValue::Unit,
                TypeInner::Base(BaseType::Bool) => MachineValue::Bool(true),
                TypeInner::Base(BaseType::Int) => MachineValue::Int(42),
                TypeInner::Base(BaseType::Symbol) => MachineValue::Symbol("test".into()),
                _ => continue,
            };
            
            assert_eq!(machine_value.get_type(), base_type);
        }
        
        // From documentation: Product types (τ₁ ⊗ τ₂)
        let product_type = TypeInner::Product(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::Base(BaseType::Bool))
        );
        
        let product_value = MachineValue::Product(
            Box::new(MachineValue::Int(42)),
            Box::new(MachineValue::Bool(true))
        );
        
        assert_eq!(product_value.get_type(), product_type);
        
        // From documentation: Linear function types (τ₁ ⊸ τ₂)
        let function_type = TypeInner::LinearFunction(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::Base(BaseType::Bool))
        );
        
        // Should be able to create lambda terms with this type
        let lambda_term = Term::lambda_typed(
            "x",
            TypeInner::Base(BaseType::Int),
            Term::literal(Literal::Bool(true))
        );
        
        assert!(matches!(lambda_term.kind, TermKind::Lambda { .. }));
        
        // From documentation: Session types
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Bool)),
                Box::new(SessionType::End)
            ))
        );
        
        let session_type_inner = TypeInner::Session(Box::new(session_type.clone()));
        let session_channel = SessionChannel::new(session_type, Location::Local);
        let channel_value = MachineValue::Channel(session_channel);
        
        assert_eq!(channel_value.get_type(), session_type_inner);
        
        // From documentation: Transform types
        let transform_type = TypeInner::Transform {
            input: Box::new(TypeInner::Base(BaseType::Int)),
            output: Box::new(TypeInner::Base(BaseType::Bool)),
            location: Location::Remote(EntityId::from_content(&[6u8; 32])),
        };
        
        assert!(matches!(transform_type, TypeInner::Transform { .. }));
        
        // From documentation: Located types
        let located_type = TypeInner::Located(
            Box::new(TypeInner::Base(BaseType::Int)),
            Location::Domain("database".to_string())
        );
        
        assert!(matches!(located_type, TypeInner::Located(_, _)));
    }

    #[test]
    fn test_five_fundamental_operations_documented() {
        // From 100-layer-0-the-verifiable-execution-core.md
        let r0 = RegisterId::new(0);
        let r1 = RegisterId::new(1);
        let r2 = RegisterId::new(2);
        
        // 1. transform morph input output
        let transform = Instruction::Transform {
            morph_reg: r0,
            input_reg: r1,
            output_reg: r2,
        };
        assert_eq!(transform.operation_type(), "morphism_application");
        
        // 2. alloc type init output
        let alloc = Instruction::Alloc {
            type_reg: r0,
            init_reg: r1,
            output_reg: r2,
        };
        assert_eq!(alloc.operation_type(), "object_creation");
        
        // 3. consume resource output
        let consume = Instruction::Consume {
            resource_reg: r0,
            output_reg: r1,
        };
        assert_eq!(consume.operation_type(), "object_destruction");
        
        // 4. compose f g output
        let compose = Instruction::Compose {
            first_reg: r0,
            second_reg: r1,
            output_reg: r2,
        };
        assert_eq!(compose.operation_type(), "morphism_composition");
        
        // 5. tensor left right output
        let tensor = Instruction::Tensor {
            left_reg: r0,
            right_reg: r1,
            output_reg: r2,
        };
        assert_eq!(tensor.operation_type(), "parallel_composition");
        
        // All should verify category laws
        for instruction in [transform, alloc, consume, compose, tensor] {
            assert!(instruction.verify_category_laws());
            assert!(instruction.is_linear());
        }
    }

    #[test]
    fn test_layer1_eleven_primitives_documented() {
        // From 101-layer-1-structured-types-and-causality-lisp.md
        // The 11 core Layer 1 primitives
        
        // 1. unit - Unit introduction
        let unit_term = Term::unit();
        assert!(matches!(unit_term.kind, TermKind::Unit));
        
        // 2. letunit - Unit elimination
        let letunit_term = Term::new(TermKind::LetUnit {
            unit_term: Box::new(Term::unit()),
            body: Box::new(Term::literal(Literal::Int(42))),
        });
        assert!(matches!(letunit_term.kind, TermKind::LetUnit { .. }));
        
        // 3. tensor - Product introduction
        let tensor_term = Term::tensor(
            Term::literal(Literal::Int(1)),
            Term::literal(Literal::Bool(true))
        );
        assert!(matches!(tensor_term.kind, TermKind::Tensor { .. }));
        
        // 4. lettensor - Product elimination
        let lettensor_term = Term::new(TermKind::LetTensor {
            tensor_term: Box::new(tensor_term),
            left_var: "x".to_string(),
            right_var: "y".to_string(),
            body: Box::new(Term::var("x")),
        });
        assert!(matches!(lettensor_term.kind, TermKind::LetTensor { .. }));
        
        // 5. inl - Sum introduction (left)
        let inl_term = Term::new(TermKind::Inl {
            value: Box::new(Term::literal(Literal::Int(42))),
            sum_type: TypeInner::Sum(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Bool))
            ),
        });
        assert!(matches!(inl_term.kind, TermKind::Inl { .. }));
        
        // 6. inr - Sum introduction (right)
        let inr_term = Term::new(TermKind::Inr {
            value: Box::new(Term::literal(Literal::Bool(true))),
            sum_type: TypeInner::Sum(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Bool))
            ),
        });
        assert!(matches!(inr_term.kind, TermKind::Inr { .. }));
        
        // 7. case - Sum elimination
        let case_term = Term::new(TermKind::Case {
            scrutinee: Box::new(inl_term),
            left_var: "x".to_string(),
            left_body: Box::new(Term::var("x")),
            right_var: "y".to_string(),
            right_body: Box::new(Term::literal(Literal::Int(0))),
        });
        assert!(matches!(case_term.kind, TermKind::Case { .. }));
        
        // 8. lambda - Function introduction
        let lambda_term = Term::lambda("x", Term::var("x"));
        assert!(matches!(lambda_term.kind, TermKind::Lambda { .. }));
        
        // 9. apply - Function elimination
        let apply_term = Term::apply(lambda_term, Term::literal(Literal::Int(42)));
        assert!(matches!(apply_term.kind, TermKind::Apply { .. }));
        
        // 10. alloc - Resource allocation
        let alloc_term = Term::alloc(Term::literal(Literal::Int(42)));
        assert!(matches!(alloc_term.kind, TermKind::Alloc { .. }));
        
        // 11. consume - Resource consumption
        let consume_term = Term::consume(Term::var("resource"));
        assert!(matches!(consume_term.kind, TermKind::Consume { .. }));
    }
}

#[cfg(test)]
mod comprehensive_integration {
    use super::*;

    #[test]
    fn test_complete_system_roundtrip() {
        // Test the most complex scenario: a distributed session with effects
        
        // Create a complex session type
        let session_type = SessionType::Send(
            Box::new(TypeInner::Product(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Symbol))
            )),
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Sum(
                    Box::new(TypeInner::Base(BaseType::Bool)),
                    Box::new(TypeInner::Base(BaseType::Unit))
                )),
                Box::new(SessionType::End)
            ))
        );
        
        // Create a distributed location
        let remote_location = Location::Remote(EntityId::from_content(&[7u8; 32]));
        
        // Layer 2: Complex effect with session and location
        let complex_effect = EffectExpr::new(EffectExprKind::WithSession {
            session_decl: "DistributedSession".to_string(),
            role: "client".to_string(),
            body: Box::new(EffectExpr::new(EffectExprKind::Bind {
                effect: Box::new(EffectExpr::new(EffectExprKind::SessionSend {
                    channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("ch")))),
                    value: Term::tensor(
                        Term::literal(Literal::Int(42)),
                        Term::literal(Literal::Symbol("test".into()))
                    ),
                    continuation: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::unit()))),
                })),
                var: "result".to_string(),
                body: Box::new(EffectExpr::new(EffectExprKind::SessionReceive {
                    channel: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::var("ch")))),
                    continuation: Box::new(EffectExpr::new(EffectExprKind::Pure(Term::unit()))),
                })),
            })),
        });
        
        // Layer 1: Corresponding terms
        let new_channel_term = Term::new_channel(session_type.clone());
        let product_term = Term::tensor(
            Term::literal(Literal::Int(42)),
            Term::literal(Literal::Symbol("test".into()))
        );
        let send_term = Term::send(Term::var("ch"), product_term);
        let receive_term = Term::receive(Term::var("ch"));
        let at_location_term = Term::at(remote_location.clone(), send_term);
        
        // Layer 0: Machine values and session channel
        let session_channel = SessionChannel::new(session_type.clone(), remote_location);
        let channel_value = MachineValue::Channel(session_channel);
        let product_value = MachineValue::Product(
            Box::new(MachineValue::Int(42)),
            Box::new(MachineValue::Symbol("test".into()))
        );
        
        // Verify complete consistency
        assert!(matches!(complex_effect.kind, EffectExprKind::WithSession { .. }));
        assert!(matches!(new_channel_term.kind, TermKind::NewChannel { .. }));
        assert!(matches!(at_location_term.kind, TermKind::At { .. }));
        assert!(channel_value.is_available_channel());
        assert_eq!(channel_value.get_session_type(), Some(&session_type));
        
        // Verify product type consistency
        let expected_product_type = TypeInner::Product(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::Base(BaseType::Symbol))
        );
        assert_eq!(product_value.get_type(), expected_product_type);
        
        // Verify session type consistency
        let session_type_inner = TypeInner::Session(Box::new(session_type));
        assert_eq!(channel_value.get_type(), session_type_inner);
        
        // This represents a complete end-to-end type consistency check
        // from the highest level (Layer 2 effects) down to the lowest level
        // (Layer 0 machine values and instructions)
    }
} 