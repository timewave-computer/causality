#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        core::id::{ExprId, ResourceId, ValueExprId},
        core::numeric::Number,
        core::str::Str,
        expr::value::{ValueExpr, ValueExprMap, ValueExprVec},
    };
    use lexpr::Value as SExpr;
    use std::collections::BTreeMap;

    // Helper function to create a test value expression
    fn create_test_value_expr() -> ValueExpr {
        let mut record = BTreeMap::new();
        record.insert(Str::from("id"), ValueExpr::String(Str::from("test-id")));
        record.insert(Str::from("count"), ValueExpr::Number(Number::Integer(42)));
        record.insert(Str::from("active"), ValueExpr::Bool(true));
        ValueExpr::Record(ValueExprMap(record))
    }

    // Helper function to create a list value expression
    fn create_list_value_expr() -> ValueExpr {
        let items = vec![
            ValueExpr::Number(Number::Integer(1)),
            ValueExpr::Number(Number::Integer(2)),
            ValueExpr::Number(Number::Integer(3)),
        ];
        ValueExpr::List(ValueExprVec(items))
    }

    #[test]
    fn test_primitive_value_expr_to_sexpr_conversion() {
        // Test conversion of primitive ValueExpr types to S-expressions
        
        // Test unit/nil conversion
        let unit_val = ValueExpr::Unit;
        let sexpr = value_expr_to_sexpr(&unit_val);
        assert!(sexpr.is_null());
        
        // Test boolean conversion
        let bool_val = ValueExpr::Bool(true);
        let sexpr = value_expr_to_sexpr(&bool_val);
        assert!(sexpr.is_bool());
        assert_eq!(sexpr.as_bool().unwrap(), true);
        
        // Test integer conversion
        let int_val = ValueExpr::Number(Number::Integer(42));
        let sexpr = value_expr_to_sexpr(&int_val);
        assert!(sexpr.is_number());
        assert_eq!(sexpr.as_i64().unwrap(), 42);
        
        // Test string conversion
        let string_val = ValueExpr::String(Str::from("hello"));
        let sexpr = value_expr_to_sexpr(&string_val);
        assert!(sexpr.is_string());
        assert_eq!(sexpr.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_complex_value_expr_to_sexpr_conversion() {
        // Test conversion of complex ValueExpr types to S-expressions
        
        // Test list conversion
        let list_val = create_list_value_expr();
        let sexpr = value_expr_to_sexpr(&list_val);
        
        // Check that we got a list S-expression
        assert!(sexpr.is_list());
        let items = sexpr.as_list().unwrap();
        
        // Check length
        assert_eq!(items.len(), 3);
        
        // Check values
        assert_eq!(items[0].as_i64().unwrap(), 1);
        assert_eq!(items[1].as_i64().unwrap(), 2);
        assert_eq!(items[2].as_i64().unwrap(), 3);
        
        // Test record/map conversion
        let record_val = create_test_value_expr();
        let sexpr = value_expr_to_sexpr(&record_val);
        
        // For maps, we convert to a list of key-value pairs (an association list in Lisp terms)
        assert!(sexpr.is_list());
        let pairs = sexpr.as_list().unwrap();
        
        // Check we have the expected number of pairs
        assert_eq!(pairs.len(), 3);
        
        // Each pair should be a 2-element list of (key value)
        for pair in pairs {
            assert!(pair.is_list());
            let pair_list = pair.as_list().unwrap();
            assert_eq!(pair_list.len(), 2);
            assert!(pair_list[0].is_string()); // Key should be a string
        }
        
        // Check if we can find expected keys and values
        let mut found_id = false;
        let mut found_count = false;
        let mut found_active = false;
        
        for pair in pairs {
            let pair_list = pair.as_list().unwrap();
            let key = pair_list[0].as_str().unwrap();
            
            match key {
                "id" => {
                    found_id = true;
                    assert_eq!(pair_list[1].as_str().unwrap(), "test-id");
                },
                "count" => {
                    found_count = true;
                    assert_eq!(pair_list[1].as_i64().unwrap(), 42);
                },
                "active" => {
                    found_active = true;
                    assert_eq!(pair_list[1].as_bool().unwrap(), true);
                },
                _ => panic!("Unexpected key in record: {}", key),
            }
        }
        
        // Ensure we found all expected keys
        assert!(found_id && found_count && found_active);
    }

    #[test]
    fn test_sexpr_to_value_expr_conversion() {
        // Test conversion from S-expressions back to ValueExpr
        
        // Test simple values
        let sexpr_null = SExpr::Null;
        let val_expr = sexpr_to_value_expr(&sexpr_null);
        assert!(matches!(val_expr, ValueExpr::Nil));
        
        let sexpr_bool = SExpr::Bool(true);
        let val_expr = sexpr_to_value_expr(&sexpr_bool);
        assert!(matches!(val_expr, ValueExpr::Bool(true)));
        
        let sexpr_int = SExpr::Number(42.into());
        let val_expr = sexpr_to_value_expr(&sexpr_int);
        if let ValueExpr::Number(Number::Integer(val)) = val_expr {
            assert_eq!(val, 42);
        } else {
            panic!("Expected Integer ValueExpr");
        }
        
        let sexpr_string = SExpr::String("hello".into());
        let val_expr = sexpr_to_value_expr(&sexpr_string);
        if let ValueExpr::String(s) = val_expr {
            assert_eq!(s.to_string(), "hello");
        } else {
            panic!("Expected String ValueExpr");
        }
        
        // Test list conversion
        let sexpr_list = SExpr::List(vec![
            SExpr::Number(1.into()),
            SExpr::Number(2.into()),
            SExpr::Number(3.into()),
        ]);
        
        let val_expr = sexpr_to_value_expr(&sexpr_list);
        if let ValueExpr::List(ValueExprVec(items)) = val_expr {
            assert_eq!(items.len(), 3);
            if let ValueExpr::Number(Number::Integer(val)) = items[0] {
                assert_eq!(val, 1);
            } else {
                panic!("Expected Integer ValueExpr");
            }
        } else {
            panic!("Expected List ValueExpr");
        }
    }

    #[test]
    fn test_value_expr_sexpr_roundtrip() {
        // Test round-trip conversion: ValueExpr -> SExpr -> ValueExpr
        
        // Test with a simple value
        let original = ValueExpr::Number(Number::Integer(42));
        let sexpr = value_expr_to_sexpr(&original);
        let roundtrip = sexpr_to_value_expr(&sexpr);
        
        if let ValueExpr::Number(Number::Integer(val)) = roundtrip {
            assert_eq!(val, 42);
        } else {
            panic!("Expected Integer ValueExpr after roundtrip");
        }
        
        // Test with a complex structure
        let original = create_test_value_expr();
        let sexpr = value_expr_to_sexpr(&original);
        let roundtrip = sexpr_to_value_expr(&sexpr);
        
        // For maps/records, the roundtrip will convert to a Map instead of a Record
        if let ValueExpr::Map(ValueExprMap(items)) = roundtrip {
            assert_eq!(items.len(), 3);
            assert!(items.contains_key(&Str::from("id")));
            assert!(items.contains_key(&Str::from("count")));
            assert!(items.contains_key(&Str::from("active")));
            
            if let ValueExpr::String(s) = &items[&Str::from("id")] {
                assert_eq!(s.to_string(), "test-id");
            } else {
                panic!("Expected String value for id key");
            }
        } else {
            panic!("Expected Map ValueExpr after roundtrip");
        }
    }

    #[test]
    fn test_canonical_sexpr_serialization() {
        // Test that S-expressions are serialized in a canonical form for content addressing
        
        // Create a simple S-expression
        let sexpr1 = SExpr::List(vec![
            SExpr::String("test".into()),
            SExpr::Number(42.into()),
            SExpr::Bool(true),
        ]);
        
        // Create the same S-expression but in a different order
        let sexpr2 = SExpr::List(vec![
            SExpr::String("test".into()),
            SExpr::Bool(true),
            SExpr::Number(42.into()),
        ]);
        
        // Canonical form should be order-dependent for lists
        let canonical1 = serialize_sexpr_canonical(&sexpr1);
        let canonical2 = serialize_sexpr_canonical(&sexpr2);
        
        // Different order, different canonical form
        assert_ne!(canonical1, canonical2);
        
        // Test with maps (should normalize keys)
        let record_val1 = create_test_value_expr();
        let record_val2 = {
            let mut record = BTreeMap::new();
            // Insert in different order
            record.insert(Str::from("active"), ValueExpr::Bool(true));
            record.insert(Str::from("count"), ValueExpr::Number(Number::Integer(42)));
            record.insert(Str::from("id"), ValueExpr::String(Str::from("test-id")));
            ValueExpr::Record(ValueExprMap(record))
        };
        
        let sexpr_record1 = value_expr_to_sexpr(&record_val1);
        let sexpr_record2 = value_expr_to_sexpr(&record_val2);
        
        // The S-expressions might differ due to order, but canonical forms should match
        let canonical_record1 = serialize_sexpr_canonical(&sexpr_record1);
        let canonical_record2 = serialize_sexpr_canonical(&sexpr_record2);
        
        // Equal records should have equal canonical forms
        assert_eq!(canonical_record1, canonical_record2);
    }
} 