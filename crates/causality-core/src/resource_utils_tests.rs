#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        core::id::{DomainId, ExprId, ResourceId, TypeExprId, ValueExprId},
        core::numeric::Number,
        core::str::Str,
        expr::value::{ValueExpr, ValueExprMap},
        resource::{AsResource, Resource, Nullifier, Effect, Handler, Intent},
    };
    use std::collections::BTreeMap;

    /// Helper function to create a test resource
    fn create_test_resource() -> Resource {
        Resource {
            id: ResourceId::random(),
            domain: DomainId::random(),
            ephemeral: false,
            value: ValueExprId::random(),
            type_expr: TypeExprId::random(),
            static_expr: Some(ExprId::random()),
        }
    }

    /// Helper function to create a test value expression map
    fn create_test_value_map() -> BTreeMap<Str, ValueExpr> {
        let mut map = BTreeMap::new();
        map.insert(Str::from("name"), ValueExpr::String(Str::from("Test Resource")));
        map.insert(Str::from("count"), ValueExpr::Number(Number::Integer(42)));
        map.insert(Str::from("active"), ValueExpr::Bool(true));
        map
    }

    #[test]
    fn test_resource_creation_utilities() {
        // Test resource creation utilities
        
        // Create a resource with the utility function
        let domain_id = DomainId::random();
        let value_id = ValueExprId::random();
        let type_expr_id = TypeExprId::random();
        let static_expr_id = ExprId::random();
        
        let resource = create_resource(
            domain_id.clone(),
            value_id.clone(),
            type_expr_id.clone(),
            Some(static_expr_id.clone()),
            false, // non-ephemeral
        );
        
        // Verify the resource has the expected properties
        assert_eq!(resource.domain, domain_id);
        assert_eq!(resource.value, value_id);
        assert_eq!(resource.type_expr, type_expr_id);
        assert_eq!(resource.static_expr, Some(static_expr_id));
        assert_eq!(resource.ephemeral, false);
        
        // Create an ephemeral resource
        let ephemeral_resource = create_resource(
            domain_id,
            value_id,
            type_expr_id,
            None, // no static expression
            true, // ephemeral
        );
        
        // Verify the ephemeral resource
        assert!(ephemeral_resource.ephemeral);
        assert_eq!(ephemeral_resource.static_expr, None);
    }

    #[test]
    fn test_resource_to_value_expr_conversion() {
        // Test conversion of a Resource to a ValueExpr
        
        // Create a test resource
        let resource = create_test_resource();
        
        // Convert to ValueExpr
        let value_expr = resource_to_value_expr(&resource);
        
        // Verify the value expression is a record with the expected fields
        if let ValueExpr::Record(ValueExprMap(fields)) = value_expr {
            // Check required fields are present
            assert!(fields.contains_key(&Str::from("id")));
            assert!(fields.contains_key(&Str::from("domain")));
            assert!(fields.contains_key(&Str::from("value")));
            assert!(fields.contains_key(&Str::from("type_expr")));
            assert!(fields.contains_key(&Str::from("static_expr")));
            assert!(fields.contains_key(&Str::from("ephemeral")));
            
            // Check field values (as strings, since IDs are converted to strings)
            if let ValueExpr::String(id_str) = &fields[&Str::from("id")] {
                assert_eq!(id_str.as_str(), resource.id.to_string());
            } else {
                panic!("Expected id field to be String");
            }
            
            if let ValueExpr::Bool(ephemeral) = fields[&Str::from("ephemeral")] {
                assert_eq!(ephemeral, resource.ephemeral);
            } else {
                panic!("Expected ephemeral field to be Bool");
            }
        } else {
            panic!("Expected Record type for resource conversion");
        }
    }

    #[test]
    fn test_value_expr_to_resource_conversion() {
        // Test conversion from ValueExpr to Resource
        
        // Create a ValueExpr representing a resource
        let resource_id = ResourceId::random();
        let domain_id = DomainId::random();
        let value_id = ValueExprId::random();
        let type_expr_id = TypeExprId::random();
        let static_expr_id = ExprId::random();
        
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("id"), ValueExpr::String(Str::from(resource_id.to_string())));
        fields.insert(Str::from("domain"), ValueExpr::String(Str::from(domain_id.to_string())));
        fields.insert(Str::from("value"), ValueExpr::String(Str::from(value_id.to_string())));
        fields.insert(Str::from("type_expr"), ValueExpr::String(Str::from(type_expr_id.to_string())));
        fields.insert(Str::from("static_expr"), ValueExpr::String(Str::from(static_expr_id.to_string())));
        fields.insert(Str::from("ephemeral"), ValueExpr::Bool(false));
        
        let value_expr = ValueExpr::Record(ValueExprMap(fields));
        
        // Convert to Resource
        let resource = value_expr_to_resource(&value_expr)
            .expect("Failed to convert ValueExpr to Resource");
        
        // Verify the resource has the expected properties
        assert_eq!(resource.id.to_string(), resource_id.to_string());
        assert_eq!(resource.domain.to_string(), domain_id.to_string());
        assert_eq!(resource.value.to_string(), value_id.to_string());
        assert_eq!(resource.type_expr.to_string(), type_expr_id.to_string());
        assert_eq!(resource.static_expr.unwrap().to_string(), static_expr_id.to_string());
        assert_eq!(resource.ephemeral, false);
    }

    #[test]
    fn test_resource_type_conversions() {
        // Test conversion between Resource types (Effect, Handler, Intent)
        
        // Create base resources for different types
        let effect_resource = {
            let mut effect = Effect {
                id: ResourceId::random(),
                domain: DomainId::random(),
                ephemeral: false,
                value: ValueExprId::random(),
                type_expr: TypeExprId::random(),
                static_expr: None,
            };
            effect
        };
        
        let handler_resource = {
            let mut handler = Handler {
                id: ResourceId::random(),
                domain: DomainId::random(),
                ephemeral: false,
                value: ValueExprId::random(),
                type_expr: TypeExprId::random(),
                static_expr: Some(ExprId::random()),
            };
            handler
        };
        
        let intent_resource = {
            let mut intent = Intent {
                id: ResourceId::random(),
                domain: DomainId::random(),
                ephemeral: false,
                value: ValueExprId::random(),
                type_expr: TypeExprId::random(),
                static_expr: Some(ExprId::random()),
            };
            intent
        };
        
        // Test conversion from Effect to Resource and back
        let resource_from_effect = effect_resource.to_resource();
        let effect_from_resource = resource_to_effect(&resource_from_effect)
            .expect("Failed to convert Resource to Effect");
        
        assert_eq!(effect_resource.id, effect_from_resource.id);
        assert_eq!(effect_resource.domain, effect_from_resource.domain);
        assert_eq!(effect_resource.value, effect_from_resource.value);
        
        // Test conversion from Handler to Resource and back
        let resource_from_handler = handler_resource.to_resource();
        let handler_from_resource = resource_to_handler(&resource_from_handler)
            .expect("Failed to convert Resource to Handler");
        
        assert_eq!(handler_resource.id, handler_from_resource.id);
        assert_eq!(handler_resource.domain, handler_from_resource.domain);
        assert_eq!(handler_resource.value, handler_from_resource.value);
        
        // Test conversion from Intent to Resource and back
        let resource_from_intent = intent_resource.to_resource();
        let intent_from_resource = resource_to_intent(&resource_from_intent)
            .expect("Failed to convert Resource to Intent");
        
        assert_eq!(intent_resource.id, intent_from_resource.id);
        assert_eq!(intent_resource.domain, intent_from_resource.domain);
        assert_eq!(intent_resource.value, intent_from_resource.value);
    }

    #[test]
    fn test_resource_hash_computation() {
        // Test computation of resource hash for content addressing
        
        // Create two identical resources (with same fields but different IDs)
        let domain_id = DomainId::random();
        let value_id = ValueExprId::random();
        let type_expr_id = TypeExprId::random();
        let static_expr_id = ExprId::random();
        
        let resource1 = create_resource(
            domain_id.clone(),
            value_id.clone(),
            type_expr_id.clone(),
            Some(static_expr_id.clone()),
            false,
        );
        
        let resource2 = create_resource(
            domain_id.clone(),
            value_id.clone(),
            type_expr_id.clone(),
            Some(static_expr_id.clone()),
            false,
        );
        
        // Resources with the same content should have the same hash
        let hash1 = compute_resource_hash(&resource1);
        let hash2 = compute_resource_hash(&resource2);
        
        // The resources have different IDs but should have the same content hash
        assert_ne!(resource1.id, resource2.id);
        assert_eq!(hash1, hash2);
        
        // Create a resource with different content
        let resource3 = create_resource(
            domain_id.clone(),
            value_id.clone(),
            type_expr_id.clone(),
            None, // Different static expression
            true, // Different ephemeral flag
        );
        
        // Resources with different content should have different hashes
        let hash3 = compute_resource_hash(&resource3);
        assert_ne!(hash1, hash3);
    }
} 