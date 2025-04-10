//! Integration tests for the TEL components
//! 
//! This file contains tests that verify the interaction between
//! different components of the TEL implementation.

use std::collections::{BTreeMap, HashMap};
use crate::types::{TelType, BaseType, RecordType, TypeEnvironment};
use crate::types::effect::EffectRow;
use crate::{TelEffectError};
use crate::combinators::{Combinator, Literal};
use crate::combinators::reducer::{BetaReducer, ReducerSettings, EvaluationStrategy};
use crate::combinators::parser::parse_combinator;
use crate::combinators::merkle::{MerkleNode, MerklePath};
use crate::{CoreEffectError};

/// Test that combines parsing, reduction, and content addressing
#[test]
fn test_parse_reduce_content_id() {
    // Parse a simple combinator expression
    let expr_str = "S K K x";
    let expr = parse_combinator(expr_str).unwrap();
    
    // Create a reducer with default settings
    let mut settings = ReducerSettings::default();
    settings.max_steps = Some(100);
    settings.optimize = true;
    
    let mut reducer = BetaReducer::with_settings(settings);
    
    // Reduce the expression
    let result = reducer.eval(&expr).unwrap();
    let reduced = result.expr;
    
    // Check that S K K x reduces to x (via I x)
    match &reduced {
        Combinator::Ref(name) => assert_eq!(name, "x"),
        _ => panic!("Expected reference 'x', got {:?}", reduced),
    }
    
    // Compute content IDs for the original and reduced expressions
    let original_node = MerkleNode::from_combinator(&expr).unwrap();
    let reduced_node = MerkleNode::from_combinator(&reduced).unwrap();
    
    let original_id = original_node.content_id;
    let reduced_id = reduced_node.content_id;
    
    // Content IDs should be different
    assert_ne!(original_id, reduced_id);
}

/// Test that combines row types with effect types
#[test]
fn test_row_with_effects() {
    let env = TypeEnvironment::new();
    
    // Create a record type with fields
    let mut fields = BTreeMap::new();
    fields.insert("name".to_string(), TelType::Base(BaseType::String));
    fields.insert("age".to_string(), TelType::Base(BaseType::Int));
    let record_type = RecordType { fields, extension: None };
    
    // Create an effect row with read/write operations
    let mut effects = BTreeMap::new();
    effects.insert(
        "read".to_string(), 
        TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String))
        )
    );
    effects.insert(
        "write".to_string(), 
        TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::Unit))
        )
    );
    
    let effect_row = EffectRow { 
        effects,
        extension: None,
    };
    
    // Create a function type that takes a record and returns a string
    let fn_type = TelType::Function(
        Box::new(TelType::Record(record_type.clone())),
        Box::new(TelType::Base(BaseType::String))
    );
    
    // Test extracting fields and operations
    assert!(record_type.fields.contains_key("name"));
    assert!(record_type.fields.contains_key("age"));
    assert!(effect_row.effects.contains_key("read"));
    assert!(effect_row.effects.contains_key("write"));
}

/// Test that combines combinators, content addressing, and Merkle paths
#[test]
fn test_merkle_path_verification() {
    // Parse a simple combinator expression
    let expr_str = "S K K x";
    let expr = parse_combinator(expr_str).unwrap();
    
    // Create a Merkle node from the combinator
    let node = MerkleNode::from_combinator(&expr).unwrap();
    let content_id = node.content_id.clone();
    
    // Test that a node can find itself by content ID
    let found = node.find_by_id(&content_id);
    assert!(found.is_some());
    
    // Create a path from a node to itself
    let path = node.create_path(&content_id);
    assert!(path.is_some());
    
    // For a more complete test, we could create paths between nodes in a more complex tree
    let complex_expr_str = "S (K (I x)) (B C y)";
    let complex_expr = parse_combinator(complex_expr_str).unwrap();
    let complex_node = MerkleNode::from_combinator(&complex_expr).unwrap();
    
    // Check that we can find parts of the tree
    if let Combinator::App { function: box_f, argument: _ } = &complex_expr {
        if let Combinator::App { function: box_s, argument: _ } = &**box_f {
            // Create a Merkle node for just the S part
            let s_node = MerkleNode::from_combinator(box_s).unwrap();
            let s_id = s_node.content_id.clone();
            
            // Try to find the S combinator in the full tree
            let found_s = complex_node.find_by_id(&s_id);
            assert!(found_s.is_some());
        }
    }
}

/// Test that combines record types and effect operations 
#[test]
fn test_typed_effects() {
    // Create a record type with fields
    let mut fields = BTreeMap::new();
    fields.insert("name".to_string(), TelType::Base(BaseType::String));
    fields.insert("age".to_string(), TelType::Base(BaseType::Int));
    let person_type = RecordType { fields, extension: None };
    
    // Create effect operations
    let mut effects = BTreeMap::new();
    effects.insert(
        "read".to_string(),
        TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String))
        )
    );
    effects.insert(
        "write".to_string(),
        TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::Unit))
        )
    );
    
    let io_effect = EffectRow { 
        effects,
        extension: None,
    };
    
    // Test that we can check for the presence of operations
    assert!(io_effect.effects.contains_key("read"));
    assert!(io_effect.effects.contains_key("write"));
    assert!(!io_effect.effects.contains_key("append"));
    
    // Test effect row with extension
    let mut read_only = BTreeMap::new();
    read_only.insert(
        "read".to_string(),
        TelType::Function(
            Box::new(TelType::Base(BaseType::String)),
            Box::new(TelType::Base(BaseType::String))
        )
    );
    
    let extensible_effect = EffectRow {
        effects: read_only,
        extension: Some("e".to_string()),
    };
    
    // Test extension presence
    assert_eq!(extensible_effect.extension, Some("e".to_string()));
    
    // Test subtyping relationships based on extensions
    // The io_effect should not be a subtype of extensible_effect because
    // extensible_effect can potentially have more effects through its extension
    assert!(!io_effect.is_subtype(&extensible_effect));
    
    // extensible_effect is not a subtype of io_effect because it lacks the "write" effect
    assert!(!extensible_effect.is_subtype(&io_effect));
}

/// Test error type conversion between causality-core and causality-tel
#[test]
fn test_effect_error_conversion() {
    // Create a CoreEffectError
    let core_error = CoreEffectError::NotFound("test effect".to_string());
    
    // Convert to TelEffectError
    let tel_error: TelEffectError = core_error.into();
    
    // Check that it was converted correctly
    match tel_error {
        TelEffectError::EffectNotFound(msg) => {
            assert_eq!(msg, "test effect");
        },
        _ => panic!("Expected EffectNotFound, got {:?}", tel_error),
    }
    
    // Test another error type
    let core_error = CoreEffectError::ValidationError("invalid effect format".to_string());
    let tel_error: TelEffectError = core_error.into();
    
    match tel_error {
        TelEffectError::InvalidEffectType(msg) => {
            assert_eq!(msg, "invalid effect format");
        },
        _ => panic!("Expected InvalidEffectType, got {:?}", tel_error),
    }
    
    // Test generic conversion for unmatched error types
    let core_error = CoreEffectError::SystemError("system failure".to_string());
    let tel_error: TelEffectError = core_error.into();
    
    match tel_error {
        TelEffectError::CoreError(msg) => {
            assert!(msg.contains("system failure"));
        },
        _ => panic!("Expected CoreError, got {:?}", tel_error),
    }
} 