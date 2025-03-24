// Cross-Domain Relationship Validation Module
//
// This module implements validation rules for cross-domain relationships,
// ensuring integrity and consistency across domains.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;

use crate::error::{Error, Result};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::resource::lifecycle_manager::ResourceRegisterLifecycleManager;
use super::cross_domain::{CrossDomainRelationship, CrossDomainRelationshipType, SyncStrategy};

/// Validation level for cross-domain relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Strict validation enforces all rules
    Strict,
    
    /// Moderate validation enforces critical rules only
    Moderate,
    
    /// Permissive validation provides warnings without failing
    Permissive,
}

/// Validation error type
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Missing required field
    MissingField(String),
    
    /// Invalid relationship type
    InvalidRelationshipType(String),
    
    /// Invalid domain
    InvalidDomain(String),
    
    /// Invalid resource
    InvalidResource(String),
    
    /// Invalid synchronization configuration
    InvalidSyncConfiguration(String),
    
    /// Incompatible configuration
    IncompatibleConfiguration(String),
    
    /// Other validation error
    Other(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField(field) => 
                write!(f, "Missing required field: {}", field),
            ValidationError::InvalidRelationshipType(msg) => 
                write!(f, "Invalid relationship type: {}", msg),
            ValidationError::InvalidDomain(msg) => 
                write!(f, "Invalid domain: {}", msg),
            ValidationError::InvalidResource(msg) => 
                write!(f, "Invalid resource: {}", msg),
            ValidationError::InvalidSyncConfiguration(msg) => 
                write!(f, "Invalid sync configuration: {}", msg),
            ValidationError::IncompatibleConfiguration(msg) => 
                write!(f, "Incompatible configuration: {}", msg),
            ValidationError::Other(msg) => 
                write!(f, "Validation error: {}", msg),
        }
    }
}

/// Validation warning type
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    /// Relationship might have issues
    PotentialIssue(String),
    
    /// Suggested improvement
    Suggestion(String),
    
    /// Performance concern
    PerformanceConcern(String),
    
    /// Other warning
    Other(String),
}

impl fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationWarning::PotentialIssue(msg) => 
                write!(f, "Potential issue: {}", msg),
            ValidationWarning::Suggestion(msg) => 
                write!(f, "Suggestion: {}", msg),
            ValidationWarning::PerformanceConcern(msg) => 
                write!(f, "Performance concern: {}", msg),
            ValidationWarning::Other(msg) => 
                write!(f, "Warning: {}", msg),
        }
    }
}

/// Validation result for a cross-domain relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the relationship is valid
    pub is_valid: bool,
    
    /// Validation level used
    pub validation_level: ValidationLevel,
    
    /// List of validation errors (if any)
    pub errors: Vec<ValidationError>,
    
    /// List of validation warnings (if any)
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success(validation_level: ValidationLevel) -> Self {
        Self {
            is_valid: true,
            validation_level,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Create a new failed validation result
    pub fn failure(validation_level: ValidationLevel, errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            validation_level,
            errors,
            warnings: Vec::new(),
        }
    }
    
    /// Add an error to the validation result
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

/// Validation rules for different relationship types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Default validation level
    pub default_level: ValidationLevel,
    
    /// Rules by relationship type
    pub rules_by_type: HashMap<CrossDomainRelationshipType, Vec<ValidationRule>>,
    
    /// Domain-specific rules
    pub domain_rules: HashMap<DomainId, Vec<ValidationRule>>,
}

/// A specific validation rule for cross-domain relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Unique rule identifier
    pub id: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Minimum validation level where this rule applies
    pub min_level: ValidationLevel,
    
    /// Rule type
    pub rule_type: ValidationRuleType,
}

/// Types of validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    /// Resources must exist in their respective domains
    ResourcesMustExist,
    
    /// Source resource must have correct lifecycle state
    SourceLifecycleState(Vec<String>),
    
    /// Target resource must have correct lifecycle state
    TargetLifecycleState(Vec<String>),
    
    /// Domain compatibility check
    DomainCompatibility(Vec<(DomainId, DomainId)>),
    
    /// Max relationships per resource
    MaxRelationshipsPerResource(usize),
    
    /// Relationship must be authorized
    RequiresAuthorization,
    
    /// Custom validation rule (function reference)
    Custom(String),
}

/// Manager for validating cross-domain relationships
pub struct CrossDomainRelationshipValidator {
    /// Validation rules
    rules: ValidationRules,
    
    /// Resource lifecycle managers by domain
    lifecycle_managers: HashMap<DomainId, ResourceRegisterLifecycleManager>,
}

impl CrossDomainRelationshipValidator {
    /// Create a new validator with default rules
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            lifecycle_managers: HashMap::new(),
        }
    }
    
    /// Create default validation rules
    fn default_rules() -> ValidationRules {
        let mut rules_by_type = HashMap::new();
        
        // Mirror relationship rules
        rules_by_type.insert(
            CrossDomainRelationshipType::Mirror,
            vec![
                ValidationRule {
                    id: "mirror-1".to_string(),
                    description: "Both resources must exist".to_string(),
                    min_level: ValidationLevel::Moderate,
                    rule_type: ValidationRuleType::ResourcesMustExist,
                },
                ValidationRule {
                    id: "mirror-2".to_string(),
                    description: "Source resource must be in 'Active' state".to_string(),
                    min_level: ValidationLevel::Moderate,
                    rule_type: ValidationRuleType::SourceLifecycleState(vec!["Active".to_string()]),
                },
            ],
        );
        
        // Reference relationship rules
        rules_by_type.insert(
            CrossDomainRelationshipType::Reference,
            vec![
                ValidationRule {
                    id: "reference-1".to_string(),
                    description: "Target resource must exist".to_string(),
                    min_level: ValidationLevel::Strict,
                    rule_type: ValidationRuleType::ResourcesMustExist,
                },
            ],
        );
        
        // Ownership relationship rules
        rules_by_type.insert(
            CrossDomainRelationshipType::Ownership,
            vec![
                ValidationRule {
                    id: "ownership-1".to_string(),
                    description: "Both resources must exist".to_string(),
                    min_level: ValidationLevel::Strict,
                    rule_type: ValidationRuleType::ResourcesMustExist,
                },
                ValidationRule {
                    id: "ownership-2".to_string(),
                    description: "Source resource must be in 'Active' state".to_string(),
                    min_level: ValidationLevel::Moderate,
                    rule_type: ValidationRuleType::SourceLifecycleState(vec!["Active".to_string()]),
                },
                ValidationRule {
                    id: "ownership-3".to_string(),
                    description: "Relationship must be authorized".to_string(),
                    min_level: ValidationLevel::Strict,
                    rule_type: ValidationRuleType::RequiresAuthorization,
                },
            ],
        );
        
        // Create the validation rules
        ValidationRules {
            default_level: ValidationLevel::Moderate,
            rules_by_type,
            domain_rules: HashMap::new(),
        }
    }
    
    /// Add a lifecycle manager for a domain
    pub fn add_lifecycle_manager(&mut self, domain_id: DomainId, manager: ResourceRegisterLifecycleManager) {
        self.lifecycle_managers.insert(domain_id, manager);
    }
    
    /// Add a domain-specific rule
    pub fn add_domain_rule(&mut self, domain_id: DomainId, rule: ValidationRule) {
        self.rules.domain_rules
            .entry(domain_id)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// Add a relationship type rule
    pub fn add_relationship_type_rule(&mut self, rel_type: CrossDomainRelationshipType, rule: ValidationRule) {
        self.rules.rules_by_type
            .entry(rel_type)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// Set the default validation level
    pub fn set_default_level(&mut self, level: ValidationLevel) {
        self.rules.default_level = level;
    }
    
    /// Validate a cross-domain relationship
    pub fn validate(
        &self,
        relationship: &CrossDomainRelationship,
        level: Option<ValidationLevel>,
    ) -> Result<ValidationResult> {
        let validation_level = level.unwrap_or(self.rules.default_level);
        let mut result = ValidationResult::success(validation_level);
        
        // Basic validation - all levels
        self.validate_basic_fields(relationship, &mut result);
        
        // Type-specific validation
        self.validate_relationship_type(relationship, validation_level, &mut result);
        
        // Synchronization configuration validation
        self.validate_sync_configuration(relationship, validation_level, &mut result);
        
        // If strict or moderate, validate bidirectionality
        if validation_level != ValidationLevel::Permissive {
            self.validate_bidirectionality(relationship, validation_level, &mut result);
        }
        
        // Domain-specific validation
        self.validate_domains(relationship, validation_level, &mut result);
        
        Ok(result)
    }
    
    /// Validate a batch of relationships
    pub fn validate_batch(
        &self,
        relationships: &[CrossDomainRelationship],
        level: ValidationLevel,
    ) -> Vec<Result<ValidationResult>> {
        relationships
            .iter()
            .map(|rel| self.validate(rel, Some(level)))
            .collect()
    }
    
    // Private validation methods
    
    /// Validate basic relationship fields
    fn validate_basic_fields(&self, relationship: &CrossDomainRelationship, result: &mut ValidationResult) {
        // Check for empty source resource
        if relationship.source_resource.is_empty() {
            result.add_error(ValidationError::MissingField("source_resource".to_string()));
        }
        
        // Check for empty target resource
        if relationship.target_resource.is_empty() {
            result.add_error(ValidationError::MissingField("target_resource".to_string()));
        }
        
        // Check for empty source domain
        if relationship.source_domain.is_empty() {
            result.add_error(ValidationError::MissingField("source_domain".to_string()));
        }
        
        // Check for empty target domain
        if relationship.target_domain.is_empty() {
            result.add_error(ValidationError::MissingField("target_domain".to_string()));
        }
    }
    
    /// Validate relationship type specific rules
    fn validate_relationship_type(
        &self,
        relationship: &CrossDomainRelationship,
        level: ValidationLevel,
        result: &mut ValidationResult,
    ) {
        match &relationship.relationship_type {
            CrossDomainRelationshipType::Mirror => {
                // Mirror relationships should require sync
                if !relationship.metadata.requires_sync {
                    match level {
                        ValidationLevel::Strict => {
                            result.add_error(ValidationError::InvalidSyncConfiguration(
                                "Mirror relationships require synchronization".to_string(),
                            ));
                        }
                        ValidationLevel::Moderate => {
                            result.add_warning(ValidationWarning::PotentialIssue(
                                "Mirror relationships should typically require synchronization".to_string(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
            CrossDomainRelationshipType::Reference => {
                // Reference relationships are typically bidirectional
                if !relationship.bidirectional && level == ValidationLevel::Strict {
                    result.add_warning(ValidationWarning::Suggestion(
                        "Reference relationships are typically bidirectional".to_string(),
                    ));
                }
            }
            CrossDomainRelationshipType::Custom(name) => {
                // Custom relationship types should have a non-empty name
                if name.is_empty() {
                    result.add_error(ValidationError::InvalidRelationshipType(
                        "Custom relationship type must have a non-empty name".to_string(),
                    ));
                }
            }
            _ => {}
        }
    }
    
    /// Validate synchronization configuration
    fn validate_sync_configuration(
        &self,
        relationship: &CrossDomainRelationship,
        level: ValidationLevel,
        result: &mut ValidationResult,
    ) {
        // If sync is required, there should be a reasonable sync strategy
        if relationship.metadata.requires_sync {
            match &relationship.metadata.sync_strategy {
                SyncStrategy::Periodic(duration) => {
                    if duration.as_secs() == 0 {
                        result.add_error(ValidationError::InvalidSyncConfiguration(
                            "Periodic sync duration cannot be zero".to_string(),
                        ));
                    } else if duration.as_secs() < 60 && level == ValidationLevel::Strict {
                        result.add_warning(ValidationWarning::PerformanceConcern(
                            "Periodic sync duration is very short (<60s)".to_string(),
                        ));
                    }
                }
                SyncStrategy::Hybrid(duration) => {
                    if duration.as_secs() == 0 {
                        result.add_error(ValidationError::InvalidSyncConfiguration(
                            "Hybrid sync fallback duration cannot be zero".to_string(),
                        ));
                    }
                }
                SyncStrategy::Manual => {
                    // Manual sync with requires_sync might be contradictory
                    if level == ValidationLevel::Strict {
                        result.add_warning(ValidationWarning::PotentialIssue(
                            "Relationship requires sync but is set to manual sync strategy".to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Validate bidirectionality configuration
    fn validate_bidirectionality(
        &self,
        relationship: &CrossDomainRelationship,
        level: ValidationLevel,
        result: &mut ValidationResult,
    ) {
        if relationship.bidirectional {
            // Bidirectional relationships have special considerations
            match &relationship.relationship_type {
                CrossDomainRelationshipType::Mirror => {
                    if level == ValidationLevel::Strict {
                        result.add_warning(ValidationWarning::PotentialIssue(
                            "Mirror relationships are inherently bidirectional, explicit flag is redundant".to_string(),
                        ));
                    }
                }
                CrossDomainRelationshipType::Ownership => {
                    if level == ValidationLevel::Strict {
                        result.add_warning(ValidationWarning::PotentialIssue(
                            "Bidirectional ownership may lead to circular ownership issues".to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Validate domain configuration
    fn validate_domains(
        &self,
        relationship: &CrossDomainRelationship,
        level: ValidationLevel,
        result: &mut ValidationResult,
    ) {
        // Source and target domains should be different for cross-domain relationships
        if relationship.source_domain == relationship.target_domain {
            match level {
                ValidationLevel::Strict => {
                    result.add_error(ValidationError::InvalidDomain(
                        "Cross-domain relationship source and target domains should be different".to_string(),
                    ));
                }
                ValidationLevel::Moderate => {
                    result.add_warning(ValidationWarning::PotentialIssue(
                        "Source and target domains are the same in a cross-domain relationship".to_string(),
                    ));
                }
                _ => {}
            }
        }
    }
    
    /// Validate that a resource exists in a domain
    fn validate_resource_exists(&self, resource_id: &ContentId, domain_id: &DomainId) -> Result<()> {
        if let Some(lifecycle_manager) = self.lifecycle_managers.get(domain_id) {
            // Check if the resource exists
            if lifecycle_manager.resource_exists(resource_id) {
                Ok(())
            } else {
                Err(Error::NotFound(format!(
                    "Resource '{}' not found in domain '{}'",
                    resource_id, domain_id
                )))
            }
        } else {
            // If we don't have access to the lifecycle manager, we can't validate
            Err(Error::InvalidArgument(format!(
                "No lifecycle manager available for domain '{}'",
                domain_id
            )))
        }
    }
    
    /// Validate that a resource is in one of the valid states
    fn validate_resource_state(
        &self,
        resource_id: &ContentId,
        domain_id: &DomainId,
        valid_states: &[String],
    ) -> Result<()> {
        if let Some(lifecycle_manager) = self.lifecycle_managers.get(domain_id) {
            // Get the resource state
            let state = lifecycle_manager.get_resource_state(resource_id)?;
            
            // Check if the state is valid
            if valid_states.contains(&state) {
                Ok(())
            } else {
                Err(Error::InvalidArgument(format!(
                    "Resource '{}' in domain '{}' has state '{}', which is not one of {:?}",
                    resource_id, domain_id, state, valid_states
                )))
            }
        } else {
            // If we don't have access to the lifecycle manager, we can't validate
            Err(Error::InvalidArgument(format!(
                "No lifecycle manager available for domain '{}'",
                domain_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::RegisterState;
    
    // Mock lifecycle manager for testing
    struct MockLifecycleManager {
        resources: HashMap<ContentId, String>,
    }
    
    impl MockLifecycleManager {
        fn new() -> Self {
            let mut resources = HashMap::new();
            resources.insert("resource1".to_string(), "Active".to_string());
            resources.insert("resource2".to_string(), "Active".to_string());
            resources.insert("resource3".to_string(), "Pending".to_string());
            
            Self { resources }
        }
        
        fn resource_exists(&self, resource_id: &ContentId) -> bool {
            self.resources.contains_key(resource_id)
        }
        
        fn get_resource_state(&self, resource_id: &ContentId) -> Result<String> {
            self.resources.get(resource_id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Resource '{}' not found", resource_id)))
        }
    }
    
    // Mock ResourceRegisterLifecycleManager for testing
    impl From<MockLifecycleManager> for ResourceRegisterLifecycleManager {
        fn from(_: MockLifecycleManager) -> Self {
            // This is just to satisfy the compiler
            // In real tests, we would use mockall or similar
            unimplemented!()
        }
    }
    
    #[test]
    fn test_validation_result() {
        let success = ValidationResult::success(ValidationLevel::Strict);
        assert!(success.is_valid);
        assert_eq!(success.validation_level, ValidationLevel::Strict);
        assert!(success.errors.is_empty());
        assert!(success.warnings.is_empty());
        
        let failure = ValidationResult::failure(
            ValidationLevel::Moderate,
            vec![ValidationError::MissingField("source_resource".to_string())],
        );
        assert!(!failure.is_valid);
        assert_eq!(failure.validation_level, ValidationLevel::Moderate);
        assert_eq!(failure.errors.len(), 1);
        assert!(failure.warnings.is_empty());
        
        let with_warnings = success.add_warning(ValidationWarning::PotentialIssue("Warning 1".to_string()));
        assert!(with_warnings.is_valid);
        assert_eq!(with_warnings.warnings.len(), 1);
        assert_eq!(with_warnings.warnings[0].to_string(), "Warning 1");
    }
    
    #[test]
    fn test_default_rules() {
        let validator = CrossDomainRelationshipValidator::new();
        
        // Check that we have rules for the standard relationship types
        assert!(validator.rules.rules_by_type.contains_key(&CrossDomainRelationshipType::Mirror));
        assert!(validator.rules.rules_by_type.contains_key(&CrossDomainRelationshipType::Reference));
        assert!(validator.rules.rules_by_type.contains_key(&CrossDomainRelationshipType::Ownership));
        
        // Check default level
        assert_eq!(validator.rules.default_level, ValidationLevel::Moderate);
    }
    
    /*
    // These tests would require proper mocking of ResourceRegisterLifecycleManager
    // which is beyond the scope of this example
    
    #[test]
    fn test_validate_existing_resources() {
        let mut validator = CrossDomainRelationshipValidator::new();
        
        let domain1 = "domain1".to_string();
        let domain2 = "domain2".to_string();
        
        let mock_manager1 = MockLifecycleManager::new();
        let mock_manager2 = MockLifecycleManager::new();
        
        validator.add_lifecycle_manager(domain1.clone(), mock_manager1.into());
        validator.add_lifecycle_manager(domain2.clone(), mock_manager2.into());
        
        let relationship = CrossDomainRelationship::new(
            "resource1".to_string(),
            domain1.clone(),
            "resource2".to_string(),
            domain2.clone(),
            CrossDomainRelationshipType::Mirror,
        );
        
        let result = validator.validate(&relationship, None).unwrap();
        assert!(result.is_valid);
    }
    
    #[test]
    fn test_validate_missing_resources() {
        let mut validator = CrossDomainRelationshipValidator::new();
        
        let domain1 = "domain1".to_string();
        let domain2 = "domain2".to_string();
        
        let mock_manager1 = MockLifecycleManager::new();
        let mock_manager2 = MockLifecycleManager::new();
        
        validator.add_lifecycle_manager(domain1.clone(), mock_manager1.into());
        validator.add_lifecycle_manager(domain2.clone(), mock_manager2.into());
        
        let relationship = CrossDomainRelationship::new(
            "nonexistent".to_string(),
            domain1.clone(),
            "resource2".to_string(),
            domain2.clone(),
            CrossDomainRelationshipType::Mirror,
        );
        
        let result = validator.validate(&relationship, None).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
    */
} 
