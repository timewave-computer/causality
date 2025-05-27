// Tests for the S-expression FFI functionality

#[cfg(test)]
mod test_resource_handlers {
    use super::super::sexpr_utils::SexprSerializable;
    use causality_types::serialization::Encode;
    use lexpr::Value as SexprValue;
    use crate::content_addressing::SimpleContentAddressable;
    
    // A test type that implements both S-expression and ssz serialization
    #[derive(Debug, Clone, PartialEq)]
    pub struct TestResource {
        pub id: String,
        pub value: u32,
    }
    
    // Manual implementation of Encode for testing
    impl Encode for TestResource {
        fn as_ssz_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(self.id.len() as u32).to_le_bytes());
            bytes.extend_from_slice(self.id.as_bytes());
            bytes.extend_from_slice(&self.value.to_le_bytes());
            bytes
        }
    }
    
    impl SexprSerializable for TestResource {
        fn to_sexpr(&self) -> SexprValue {
            SexprValue::list(vec![
                SexprValue::symbol("test-resource"),
                SexprValue::cons(
                    SexprValue::keyword("id"),
                    SexprValue::string(self.id.as_str())
                ),
                SexprValue::cons(
                    SexprValue::keyword("value"),
                    SexprValue::from(self.value)
                ),
            ])
        }
        
        fn from_sexpr(value: &SexprValue) -> anyhow::Result<Self> {
            // Validate tag
            if !value.is_list() {
                return Err(anyhow::anyhow!("S-expression is not a list"));
            }
            
            // Handle Cons structure directly
            if let Some((first, rest)) = value.as_pair() {
                if !first.is_symbol() || first.as_symbol() != Some("test-resource") {
                    return Err(anyhow::anyhow!("Expected test-resource tag"));
                }
                
                // Walk through the rest of the list to find id and value pairs
                let mut id = String::new();
                let mut value = 0u32;
                
                let mut current = rest;
                while let Some((item, next)) = current.as_pair() {
                    if let Some((key, val)) = item.as_pair() {
                        if let Some(key_str) = key.as_keyword() {
                            match key_str {
                                "id" => {
                                    if let Some(id_str) = val.as_str() {
                                        id = id_str.to_string();
                                    }
                                }
                                "value" => {
                                    if let Some(val_u64) = val.as_u64() {
                                        value = val_u64 as u32;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    current = next;
                }
                
                return Ok(TestResource { id, value });
            }
            
            Err(anyhow::anyhow!("Invalid S-expression format for TestResource"))
        }
    }
    
    // Test the TestResource S-expression round-trip
    #[test]
    fn test_resource_sexpr_round_trip() {
        let resource = TestResource {
            id: "test-123".to_string(),
            value: 42,
        };
        
        let sexpr = resource.to_sexpr();
        let sexpr_str = resource.to_canonical_sexpr_string();
        
        println!("TestResource S-expression: {}", sexpr_str);
        
        let recovered = TestResource::from_sexpr(&sexpr).unwrap();
        assert_eq!(resource, recovered);
        
        let recovered_from_str = TestResource::from_sexpr_string(&sexpr_str).unwrap();
        assert_eq!(resource, recovered_from_str);
    }
    
    // Test content addressing
    #[test]
    fn test_resource_content_addressing() {
        let resource1 = TestResource {
            id: "test-123".to_string(),
            value: 42,
        };
        
        let resource2 = TestResource {
            id: "test-123".to_string(),
            value: 42,
        };
        
        let resource3 = TestResource {
            id: "test-456".to_string(),
            value: 42,
        };
        
        // Same content should produce same ID
        let id1 = resource1.content_id();
        let id2 = resource2.content_id();
        assert_eq!(id1, id2);
        
        // Different content should produce different ID
        let id3 = resource3.content_id();
        assert_ne!(id1, id3);
        
        println!("Resource content ID: {}", resource1.content_id_hex());
    }
    
    // Test for SSZ serialization using our unified content addressing
    #[test]
    fn test_resource_ssz_content_addressing() {
        let resource = TestResource {
            id: "test-resource".to_string(),
            value: 42,
        };
        
        // Use the content addressing API instead of try_to_vec
        let ssz_bytes = resource.as_ssz_bytes();
        println!("SSZ serialized: {} bytes", ssz_bytes.len());
        
        // Get the content ID from the SSZ bytes
        let content_id = resource.content_id_hex();
        println!("Content ID: {}", content_id);
        
        // Verify that the same content produces the same ID
        let resource2 = TestResource {
            id: "test-resource".to_string(),
            value: 42,
        };
        assert_eq!(resource.content_id(), resource2.content_id());
    }
    
    // Simulate FFI call by creating an S-expression string, then converting to
    // SSZ via the FFI functions that would be called from OCaml
    #[test]
    fn test_simulated_ffi_flow() {
        // This test simulates what would happen in the OCaml-to-Rust handoff
        // without actually using the unsafe FFI functions
        
        let resource = TestResource {
            id: "test-resource".to_string(),
            value: 42,
        };
        
        // 1. Create the S-expression
        let sexpr_value = resource.to_sexpr();
        
        // Print it for debugging
        let serialized = resource.to_canonical_sexpr_string();
        println!("S-expression: {}", serialized);
        
        // 2. Simulate OCaml passing S-expression to Rust via FFI
        // By directly using the sexpr_value instead of parsing the string
        
        // 3. Rust converts S-expression to native type
        let rust_resource = match TestResource::from_sexpr(&sexpr_value) {
            Ok(res) => res,
            Err(e) => {
                println!("Error parsing S-expression: {}", e);
                panic!("Failed to parse S-expression");
            }
        };
        
        // 4. Rust generates content-addressed ID using SSZ
        let content_id = rust_resource.content_id_hex();
        let ssz_bytes = rust_resource.as_ssz_bytes();
        println!("SSZ serialized: {} bytes, Content ID: {}", ssz_bytes.len(), content_id);
        
        // 5. Verify the roundtrip worked
        assert_eq!(resource, rust_resource);
        assert_eq!(resource.content_id(), rust_resource.content_id());
    }
} 