// Cross-Domain Relationship Validation Tests
//
// This file tests the functionality of the cross-domain relationship validator,
// which ensures the integrity and consistency of relationships across domains.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    
    use causality::error::Result;
    use causality::resource::relationship::{
        CrossDomainRelationship,
        CrossDomainRelationshipType,
        CrossDomainMetadata,
        CrossDomainRelationshipManager,
        SyncStrategy,
        CrossDomainRelationshipValidator,
        ValidationLevel,
        ValidationResult,
        ValidationRule,
        ValidationRuleType,
    };
    
    // Helper to create a test relationship
    fn create_test_relationship(
        source_resource: &str,
        source_domain: &str,
        target_resource: &str,
        target_domain: &str,
        rel_type: CrossDomainRelationshipType,
        requires_sync: bool,
        sync_strategy: SyncStrategy,
        bidirectional: bool,
    ) -> CrossDomainRelationship {
        let metadata = CrossDomainMetadata {
            origin_domain: source_domain.to_string(),
            target_domain: target_domain.to_string(),
            requires_sync,
            sync_strategy,
        };
        
        CrossDomainRelationship::new(
            source_resource.to_string(),
            source_domain.to_string(),
            target_resource.to_string(),
            target_domain.to_string(),
            rel_type,
            metadata,
            bidirectional,
        )
    }
    
    #[test]
    fn test_validation_levels() -> Result<()> {
        let validator = CrossDomainRelationshipValidator::new();
        
        // Create relationships to validate
        let valid_rel = create_test_relationship(
            "resource1", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Mirror, true,
            SyncStrategy::Periodic(Duration::from_secs(3600)), true,
        );
        
        // Test with different validation levels
        let strict_result = validator.validate(&valid_rel, ValidationLevel::Strict)?;
        let moderate_result = validator.validate(&valid_rel, ValidationLevel::Moderate)?;
        let permissive_result = validator.validate(&valid_rel, ValidationLevel::Permissive)?;
        
        // Each level should apply different rules, but our test relationship should pass all
        assert!(strict_result.is_valid);
        assert!(moderate_result.is_valid);
        assert!(permissive_result.is_valid);
        
        // Strict validation should have more warnings than permissive
        assert!(strict_result.warnings.len() >= permissive_result.warnings.len());
        
        Ok(())
    }
    
    #[test]
    fn test_invalid_relationships() -> Result<()> {
        let validator = CrossDomainRelationshipValidator::new();
        
        // Create an invalid relationship (missing required fields)
        let invalid_rel = create_test_relationship(
            "", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Mirror, true,
            SyncStrategy::Periodic(Duration::from_secs(3600)), true,
        );
        
        // Validate at strict level
        let result = validator.validate(&invalid_rel, ValidationLevel::Strict)?;
        
        // Should fail validation
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        
        // Should find at least one error about empty source resource
        let has_source_error = result.errors.iter().any(|err| {
            match err {
                ValidationError::MissingField(field) => field.contains("source"),
                ValidationError::InvalidResource(msg) => msg.contains("source"),
                _ => false,
            }
        });
        
        assert!(has_source_error, "Expected validation error for empty source resource");
        
        Ok(())
    }
    
    #[test]
    fn test_custom_validation_rules() -> Result<()> {
        // Create a validator with custom rules
        let mut validator = CrossDomainRelationshipValidator::new();
        
        // Add a custom rule that requires bidirectional relationships for References
        let custom_rule = ValidationRule {
            id: "custom-rule-1".to_string(),
            description: "Reference relationships must be bidirectional".to_string(),
            min_level: ValidationLevel::Moderate,
            rule_type: ValidationRuleType::Custom("reference-bidir".to_string()),
        };
        
        validator.add_rule_for_type(CrossDomainRelationshipType::Reference, custom_rule)?;
        
        // Register the custom rule handler
        validator.register_custom_rule_handler("reference-bidir", |relationship, _| {
            if let CrossDomainRelationshipType::Reference = relationship.relationship_type {
                if !relationship.bidirectional {
                    return Err(ValidationError::Other(
                        "Reference relationships must be bidirectional".to_string()
                    ));
                }
            }
            Ok(())
        })?;
        
        // Create test relationships
        let bidir_ref = create_test_relationship(
            "resource1", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Reference, false,
            SyncStrategy::EventDriven, true, // bidirectional = true
        );
        
        let non_bidir_ref = create_test_relationship(
            "resource3", "domain1", "resource4", "domain2",
            CrossDomainRelationshipType::Reference, false,
            SyncStrategy::EventDriven, false, // bidirectional = false
        );
        
        // Validate relationships
        let bidir_result = validator.validate(&bidir_ref, ValidationLevel::Moderate)?;
        let non_bidir_result = validator.validate(&non_bidir_ref, ValidationLevel::Moderate)?;
        
        // Bidirectional reference should pass
        assert!(bidir_result.is_valid);
        
        // Non-bidirectional reference should fail
        assert!(!non_bidir_result.is_valid);
        
        Ok(())
    }
    
    #[test]
    fn test_validation_compatibility() -> Result<()> {
        let validator = CrossDomainRelationshipValidator::new();
        
        // Test different relationship types
        let mirror_rel = create_test_relationship(
            "resource1", "domain1", "resource1-mirror", "domain2",
            CrossDomainRelationshipType::Mirror, true,
            SyncStrategy::Periodic(Duration::from_secs(3600)), true,
        );
        
        let ref_rel = create_test_relationship(
            "resource2", "domain1", "resource3", "domain2",
            CrossDomainRelationshipType::Reference, false,
            SyncStrategy::EventDriven, true,
        );
        
        let ownership_rel = create_test_relationship(
            "resource4", "domain1", "resource5", "domain2",
            CrossDomainRelationshipType::Ownership, true,
            SyncStrategy::Hybrid(Duration::from_secs(1800)), false,
        );
        
        // Validate each type
        let mirror_result = validator.validate(&mirror_rel, ValidationLevel::Moderate)?;
        let ref_result = validator.validate(&ref_rel, ValidationLevel::Moderate)?;
        let ownership_result = validator.validate(&ownership_rel, ValidationLevel::Moderate)?;
        
        // All should be valid
        assert!(mirror_result.is_valid);
        assert!(ref_result.is_valid);
        assert!(ownership_result.is_valid);
        
        // Expect each type to have different validation checks applied
        assert_ne!(
            mirror_result.warnings.len(),
            ref_result.warnings.len(),
            "Expected different validation logic for Mirror vs Reference relationships"
        );
        
        Ok(())
    }
} 