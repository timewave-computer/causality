// Resource validation module
//
// This module provides interfaces and implementations for validating resources
// and their operations. It includes support for state transition validation,
// schema validation, permission verification, and custom validation rules.

mod validation;
mod schema;
mod state;
mod permission;
mod custom;
mod context;
mod result;
mod rule;

#[cfg(test)]
pub mod tests;

pub use validation::{
    ResourceValidator, ResourceValidatorConfig,
    ValidationPipeline,
};

pub use schema::{
    SchemaValidator, SchemaValidationError,
    SchemaCompatibility,
    validate_schema_compatibility,
};

// Use resource types directly
pub use crate::resource::ResourceSchema;

pub use state::{
    StateTransitionValidator, StateTransitionError,
    StateTransitionRule,
    validate_state_transition,
};
// Import ResourceState directly from the interface module
pub use crate::resource::interface::ResourceState;

pub use permission::{
    PermissionValidator, PermissionValidationError,
    verify_permission, PermissionVerificationOptions,
    ResourcePermission,
};

pub use custom::{
    CustomValidator, CustomValidationRule,
    CustomValidationContext, CustomValidationError,
    register_custom_validator,
};

pub use context::{
    ValidationContext, ValidationContextBuilder,
    ValidationOptions, ValidationPhase,
};

pub use result::{
    ValidationResult, ValidationError, ValidationIssue,
    ValidationSeverity, ValidationStatus,
};

pub use rule::{
    ValidationRule, ValidationRuleEngine, ValidationRuleError,
    RuleCondition, RuleAction,
}; 