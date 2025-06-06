//! Record capability effects for Layer 2
//!
//! This module implements capability-based record operations as Layer 2 effects
//! that compile down to Layer 1 tensor operations. These effects provide
//! fine-grained access control for record field operations while maintaining
//! ZK circuit compatibility through static analysis and capability resolution.

use super::core::{EffectExpr, EffectHandler};
use super::operations::{pure, bind, perform, handle};
use super::capability::{RecordCapability, RecordSchema, FieldName, Capability};
use crate::lambda::{Term, Symbol};
use crate::system::content_addressing::EntityId;
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Record Effect Types
//-----------------------------------------------------------------------------

/// Record-specific effect expressions that compile to Layer 1 tensors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordEffect {
    /// Access a field from a record resource
    /// access_field : ResourceId → FieldName → Capability → Effect Value
    AccessField {
        resource_id: EntityId,
        field: FieldName,
        capability: RecordCapability,
    },
    
    /// Update a field in a record resource
    /// update_field : ResourceId → FieldName → Value → Capability → Effect Unit
    UpdateField {
        resource_id: EntityId,
        field: FieldName,
        value: Term,
        capability: RecordCapability,
    },
    
    /// Project specific fields from a record
    /// project_record : ResourceId → [FieldName] → Capability → Effect Record
    ProjectRecord {
        resource_id: EntityId,
        fields: Vec<FieldName>,
        capability: RecordCapability,
    },
    
    /// Extend a record with additional fields
    /// extend_record : ResourceId → RecordSchema → Values → Capability → Effect ResourceId
    ExtendRecord {
        resource_id: EntityId,
        extension: RecordSchema,
        values: HashMap<FieldName, Term>,
        capability: RecordCapability,
    },
    
    /// Restrict a record by removing fields
    /// restrict_record : ResourceId → [FieldName] → Capability → Effect ResourceId
    RestrictRecord {
        resource_id: EntityId,
        remove_fields: Vec<FieldName>,
        capability: RecordCapability,
    },
    
    /// Create a new record with given schema and initial values
    /// create_record : RecordSchema → Values → Capability → Effect ResourceId
    CreateRecord {
        schema: RecordSchema,
        values: HashMap<FieldName, Term>,
        capability: RecordCapability,
    },
    
    /// Delete an entire record resource
    /// delete_record : ResourceId → Capability → Effect Unit
    DeleteRecord {
        resource_id: EntityId,
        capability: RecordCapability,
    },
    
    /// Require a specific capability for record operations
    /// require_capability : CapabilityName → Effect CapabilityToken
    RequireCapability {
        capability_name: String,
    },
    
    /// Grant a capability for a resource (requires ownership)
    /// grant_capability : ResourceId → CapabilityName → Effect CapabilityToken
    GrantCapability {
        resource_id: EntityId,
        capability_name: String,
        capability: RecordCapability,
    },
}

/// Record capability token for runtime verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityToken {
    /// Capability identifier
    pub capability_id: String,
    /// Resource the capability applies to
    pub resource_id: Option<EntityId>,
    /// Specific record capability details
    pub record_capability: RecordCapability,
    /// Expiration (optional)
    pub expires_at: Option<u64>,
}

/// Record operation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordOperationResult {
    /// Successful field access
    FieldValue(Term),
    /// Successful record projection
    RecordProjection(HashMap<FieldName, Term>),
    /// Successful record creation/modification
    ResourceId(EntityId),
    /// Unit result for updates/deletions
    Unit,
    /// Capability token granted
    Token(CapabilityToken),
}

//-----------------------------------------------------------------------------
// Record Effect Constructors
//-----------------------------------------------------------------------------

/// Access a field from a record with read capability
pub fn access_field(
    resource_id: EntityId,
    field: impl Into<FieldName>,
    capability: RecordCapability,
) -> EffectExpr {
    let field = field.into();
    
    // Validate capability allows field access
    match &capability {
        RecordCapability::ReadField(cap_field) if cap_field == &field => {},
        RecordCapability::WriteField(cap_field) if cap_field == &field => {}, // Write implies read
        RecordCapability::ProjectFields(fields) if fields.contains(&field) => {},
        RecordCapability::FullRecordAccess => {},
        _ => {
            // Return an error effect if capability doesn't match
            return perform("capability_error", vec![
                Term::literal(crate::lambda::Literal::Symbol(
                    Symbol::new(&format!("Insufficient capability for field access: {}", field))
                ))
            ]);
        }
    }
    
    perform("record.access_field", vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&field))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))),
    ])
}

/// Update a field in a record with write capability
pub fn update_field(
    resource_id: EntityId,
    field: impl Into<FieldName>,
    value: Term,
    capability: RecordCapability,
) -> EffectExpr {
    let field = field.into();
    
    // Validate capability allows field write
    match &capability {
        RecordCapability::WriteField(cap_field) if cap_field == &field => {},
        RecordCapability::FullRecordAccess => {},
        _ => {
            // Return an error effect if capability doesn't allow writes
            return perform("capability_error", vec![
                Term::literal(crate::lambda::Literal::Symbol(
                    Symbol::new(&format!("Insufficient capability for field write: {}", field))
                ))
            ]);
        }
    }
    
    perform("record.update_field", vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&field))),
        value,
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))),
    ])
}

/// Project specific fields from a record
pub fn project_record(
    resource_id: EntityId,
    fields: Vec<impl Into<FieldName>>,
    capability: RecordCapability,
) -> EffectExpr {
    let field_names: Vec<FieldName> = fields.into_iter().map(|f| f.into()).collect();
    
    // Validate capability allows projection of all requested fields
    let has_access = match &capability {
        RecordCapability::ProjectFields(cap_fields) => {
            field_names.iter().all(|f| cap_fields.contains(f))
        },
        RecordCapability::FullRecordAccess => true,
        _ => false,
    };
    
    if !has_access {
        return perform("capability_error", vec![
            Term::literal(crate::lambda::Literal::Symbol(
                Symbol::new(&format!("Insufficient capability for record projection: {:?}", field_names))
            ))
        ]);
    }
    
    // Convert field names to terms for the effect call
    let field_terms: Vec<Term> = field_names.into_iter()
        .map(|f| Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&f))))
        .collect();
    
    let mut args = vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
    ];
    args.extend(field_terms);
    args.push(Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))));
    
    perform("record.project", args)
}

/// Extend a record with additional fields
pub fn extend_record(
    resource_id: EntityId,
    extension: RecordSchema,
    values: HashMap<FieldName, Term>,
    capability: RecordCapability,
) -> EffectExpr {
    // Validate capability allows record extension
    match &capability {
        RecordCapability::ExtendRecord(cap_schema) => {
            // Check if the extension is compatible with the capability schema
            if cap_schema != &extension {
                return perform("capability_error", vec![
                    Term::literal(crate::lambda::Literal::Symbol(
                        Symbol::new(&format!("Extension schema doesn't match capability"))
                    ))
                ]);
            }
        },
        RecordCapability::FullRecordAccess => {},
        _ => {
            return perform("capability_error", vec![
                Term::literal(crate::lambda::Literal::Symbol(
                    Symbol::new(&format!("Insufficient capability for record extension"))
                ))
            ]);
        }
    }
    
    // Convert values to effect arguments
    let mut args = vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", extension)))),
    ];
    
    // Add field values as arguments
    for (field, value) in values {
        args.push(Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&field))));
        args.push(value);
    }
    
    args.push(Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))));
    
    perform("record.extend", args)
}

/// Restrict a record by removing specific fields
pub fn restrict_record(
    resource_id: EntityId,
    remove_fields: Vec<impl Into<FieldName>>,
    capability: RecordCapability,
) -> EffectExpr {
    let field_names: Vec<FieldName> = remove_fields.into_iter().map(|f| f.into()).collect();
    
    // Validate capability allows record restriction
    match &capability {
        RecordCapability::RestrictRecord(cap_fields) => {
            // Check if we can remove the requested fields
            if !field_names.iter().all(|f| cap_fields.contains(f)) {
                return perform("capability_error", vec![
                    Term::literal(crate::lambda::Literal::Symbol(
                        Symbol::new(&format!("Insufficient capability for field removal"))
                    ))
                ]);
            }
        },
        RecordCapability::FullRecordAccess => {},
        _ => {
            return perform("capability_error", vec![
                Term::literal(crate::lambda::Literal::Symbol(
                    Symbol::new(&format!("Insufficient capability for record restriction"))
                ))
            ]);
        }
    }
    
    let field_terms: Vec<Term> = field_names.into_iter()
        .map(|f| Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&f))))
        .collect();
    
    let mut args = vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
    ];
    args.extend(field_terms);
    args.push(Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))));
    
    perform("record.restrict", args)
}

/// Create a new record with given schema and initial values
pub fn create_record(
    schema: RecordSchema,
    values: HashMap<FieldName, Term>,
    capability: RecordCapability,
) -> EffectExpr {
    // Validate capability allows record creation
    match &capability {
        RecordCapability::CreateRecord(cap_schema) => {
            if cap_schema != &schema {
                return perform("capability_error", vec![
                    Term::literal(crate::lambda::Literal::Symbol(
                        Symbol::new(&format!("Schema doesn't match creation capability"))
                    ))
                ]);
            }
        },
        RecordCapability::FullRecordAccess => {},
        _ => {
            return perform("capability_error", vec![
                Term::literal(crate::lambda::Literal::Symbol(
                    Symbol::new(&format!("Insufficient capability for record creation"))
                ))
            ]);
        }
    }
    
    // Convert schema and values to effect arguments
    let mut args = vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", schema)))),
    ];
    
    // Add field values
    for (field, value) in values {
        args.push(Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&field))));
        args.push(value);
    }
    
    args.push(Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))));
    
    perform("record.create", args)
}

/// Delete an entire record resource
pub fn delete_record(
    resource_id: EntityId,
    capability: RecordCapability,
) -> EffectExpr {
    // Validate capability allows record deletion
    match &capability {
        RecordCapability::DeleteRecord => {},
        RecordCapability::FullRecordAccess => {},
        _ => {
            return perform("capability_error", vec![
                Term::literal(crate::lambda::Literal::Symbol(
                    Symbol::new(&format!("Insufficient capability for record deletion"))
                ))
            ]);
        }
    }
    
    perform("record.delete", vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))),
    ])
}

/// Require a specific capability for record operations
pub fn require_capability(capability_name: impl Into<String>) -> EffectExpr {
    perform("capability.require", vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&capability_name.into()))),
    ])
}

/// Grant a capability for a resource (requires ownership)
pub fn grant_capability(
    resource_id: EntityId,
    capability_name: impl Into<String>,
    capability: RecordCapability,
) -> EffectExpr {
    perform("capability.grant", vec![
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&resource_id.to_string()))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&capability_name.into()))),
        Term::literal(crate::lambda::Literal::Symbol(Symbol::new(&format!("{:?}", capability)))),
    ])
}

//-----------------------------------------------------------------------------
// Capability-based Effect Handlers
//-----------------------------------------------------------------------------

/// Create handlers for record capability effects that compile to Layer 1 tensors
pub fn record_capability_handlers() -> Vec<EffectHandler> {
    vec![
        // Field access handler - compiles to tensor projection
        EffectHandler {
            effect_tag: "record.access_field".to_string(),
            params: vec!["resource_id".to_string(), "field".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("field_value")), // Placeholder - actual implementation compiles to tensor ops
        },
        
        // Field update handler - compiles to tensor reconstruction
        EffectHandler {
            effect_tag: "record.update_field".to_string(),
            params: vec!["resource_id".to_string(), "field".to_string(), "value".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::unit()),
        },
        
        // Record projection handler - compiles to multiple tensor projections
        EffectHandler {
            effect_tag: "record.project".to_string(),
            params: vec!["resource_id".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("projected_record")),
        },
        
        // Record extension handler - compiles to tensor construction
        EffectHandler {
            effect_tag: "record.extend".to_string(),
            params: vec!["resource_id".to_string(), "extension".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("extended_resource_id")),
        },
        
        // Record restriction handler - compiles to tensor projection
        EffectHandler {
            effect_tag: "record.restrict".to_string(),
            params: vec!["resource_id".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("restricted_resource_id")),
        },
        
        // Record creation handler - compiles to tensor construction
        EffectHandler {
            effect_tag: "record.create".to_string(),
            params: vec!["schema".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("new_resource_id")),
        },
        
        // Record deletion handler - compiles to resource consumption
        EffectHandler {
            effect_tag: "record.delete".to_string(),
            params: vec!["resource_id".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::unit()),
        },
        
        // Capability requirement handler
        EffectHandler {
            effect_tag: "capability.require".to_string(),
            params: vec!["capability_name".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("capability_token")),
        },
        
        // Capability granting handler
        EffectHandler {
            effect_tag: "capability.grant".to_string(),
            params: vec!["resource_id".to_string(), "capability_name".to_string(), "capability".to_string()],
            continuation: "k".to_string(),
            body: pure(Term::var("granted_token")),
        },
        
        // Capability error handler
        EffectHandler {
            effect_tag: "capability_error".to_string(),
            params: vec!["error_message".to_string()],
            continuation: "k".to_string(),
            body: perform("error", vec![Term::var("error_message")]),
        },
    ]
}

//-----------------------------------------------------------------------------
// Utility Functions
//-----------------------------------------------------------------------------

/// Combine multiple record effects into a transaction
pub fn record_transaction(effects: Vec<EffectExpr>) -> EffectExpr {
    if effects.is_empty() {
        return pure(Term::unit());
    }
    
    // Sequence all record effects with proper capability checking
    let mut result = effects[0].clone();
    for effect in effects.into_iter().skip(1) {
        result = bind(result, "_", effect);
    }
    
    // Wrap in transaction context
    handle(result, record_capability_handlers())
}

/// Validate that all required capabilities are available for a set of operations
pub fn validate_capabilities(
    operations: &[RecordEffect],
    available_capabilities: &[Capability],
) -> Result<(), String> {
    for op in operations {
        let required_cap = match op {
            RecordEffect::AccessField { capability, .. } => capability,
            RecordEffect::UpdateField { capability, .. } => capability,
            RecordEffect::ProjectRecord { capability, .. } => capability,
            RecordEffect::ExtendRecord { capability, .. } => capability,
            RecordEffect::RestrictRecord { capability, .. } => capability,
            RecordEffect::CreateRecord { capability, .. } => capability,
            RecordEffect::DeleteRecord { capability, .. } => capability,
            RecordEffect::GrantCapability { capability, .. } => capability,
            _ => continue, // Skip operations that don't require specific record capabilities
        };
        
        // Check if any available capability implies the required one
        let has_required = available_capabilities.iter().any(|avail_cap| {
            if let Some(avail_record_cap) = &avail_cap.record_capability {
                avail_record_cap.implies(required_cap)
            } else {
                false
            }
        });
        
        if !has_required {
            return Err(format!("Missing required capability: {:?}", required_cap));
        }
    }
    
    Ok(())
}

impl CapabilityToken {
    /// Create a new capability token
    pub fn new(
        capability_id: impl Into<String>,
        resource_id: Option<EntityId>,
        record_capability: RecordCapability,
    ) -> Self {
        Self {
            capability_id: capability_id.into(),
            resource_id,
            record_capability,
            expires_at: None,
        }
    }
    
    /// Create a token with expiration
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Check if the token is valid (not expired)
    pub fn is_valid(&self, current_time: u64) -> bool {
        self.expires_at.map_or(true, |expiry| current_time < expiry)
    }
    
    /// Check if this token grants the required capability
    pub fn grants(&self, required: &RecordCapability, for_resource: Option<EntityId>) -> bool {
        // Check resource match if specified
        if let (Some(token_resource), Some(required_resource)) = (self.resource_id, for_resource) {
            if token_resource != required_resource {
                return false;
            }
        }
        
        // Check capability implication
        self.record_capability.implies(required)
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::lambda::Literal;
//    use crate::system::content_addressing::EntityId;
//
//    fn test_resource_id() -> EntityId {
//        EntityId::from_bytes([1u8; 32])
//    }
//
//    #[test]
//    fn test_access_field_with_valid_capability() {
//        let resource_id = test_resource_id();
//        let capability = RecordCapability::ReadField("name".to_string());
//        
//        let effect = access_field(resource_id, "name", capability);
//        
//        // Should create a valid record access effect
//        match effect.kind {
//            EffectExpr::Perform { effect_tag, args } => {
//                assert_eq!(effect_tag, "record.access_field");
//                assert_eq!(args.len(), 3);
//            }
//            _ => panic!("Expected Perform effect"),
//        }
//    }
//
//    #[test]
//    fn test_access_field_with_invalid_capability() {
//        let resource_id = test_resource_id();
//        let capability = RecordCapability::ReadField("age".to_string());
//        
//        let effect = access_field(resource_id, "name", capability); // Wrong field
//        
//        // Should create a capability error effect
//        match effect.kind {
//            EffectExpr::Perform { effect_tag, .. } => {
//                assert_eq!(effect_tag, "capability_error");
//            }
//            _ => panic!("Expected capability error effect"),
//        }
//    }
//
//    #[test]
//    fn test_update_field_with_write_capability() {
//        let resource_id = test_resource_id();
//        let capability = RecordCapability::WriteField("name".to_string());
//        let value = Term::literal(Literal::Symbol(Symbol::new("John")));
//        
//        let effect = update_field(resource_id, "name", value, capability);
//        
//        match effect.kind {
//            EffectExpr::Perform { effect_tag, args } => {
//                assert_eq!(effect_tag, "record.update_field");
//                assert_eq!(args.len(), 4);
//            }
//            _ => panic!("Expected Perform effect"),
//        }
//    }
//
//    #[test]
//    fn test_project_record_with_valid_capability() {
//        let resource_id = test_resource_id();
//        let capability = RecordCapability::ProjectFields(vec!["name".to_string(), "age".to_string()]);
//        
//        let effect = project_record(resource_id, vec!["name", "age"], capability);
//        
//        match effect.kind {
//            EffectExpr::Perform { effect_tag, args } => {
//                assert_eq!(effect_tag, "record.project");
//                assert!(args.len() >= 3); // resource_id + fields + capability
//            }
//            _ => panic!("Expected Perform effect"),
//        }
//    }
//
//    #[test]
//    fn test_capability_token_validation() {
//        let resource_id = test_resource_id();
//        let capability = RecordCapability::ReadField("name".to_string());
//        
//        let token = CapabilityToken::new("test_token", Some(resource_id), capability.clone());
//        
//        // Should grant the exact capability
//        assert!(token.grants(&capability, Some(resource_id)));
//        
//        // Should not grant different capability
//        let other_capability = RecordCapability::ReadField("age".to_string());
//        assert!(!token.grants(&other_capability, Some(resource_id)));
//        
//        // Should not grant for different resource
//        let other_resource = EntityId::from_bytes([2u8; 32]);
//        assert!(!token.grants(&capability, Some(other_resource)));
//    }
//
//    #[test]
//    fn test_capability_token_expiration() {
//        let capability = RecordCapability::ReadField("name".to_string());
//        let token = CapabilityToken::new("test_token", None, capability)
//            .with_expiration(100);
//        
//        assert!(token.is_valid(50));  // Before expiration
//        assert!(!token.is_valid(150)); // After expiration
//    }
//
//    #[test]
//    fn test_record_transaction() {
//        let resource_id = test_resource_id();
//        let read_cap = RecordCapability::ReadField("name".to_string());
//        let write_cap = RecordCapability::WriteField("age".to_string());
//        
//        let effects = vec![
//            access_field(resource_id, "name", read_cap),
//            update_field(resource_id, "age", Term::literal(Literal::Int(30)), write_cap),
//        ];
//        
//        let transaction = record_transaction(effects);
//        
//        // Should wrap effects in a transaction with handlers
//        match transaction.kind {
//            EffectExpr::Handle { .. } => {
//                // Correct - transaction includes handlers
//            }
//            _ => panic!("Expected transaction to include handlers"),
//        }
//    }
//
//    #[test]
//    fn test_validate_capabilities() {
//        let operations = vec![
//            RecordEffect::AccessField {
//                resource_id: test_resource_id(),
//                field: "name".to_string(),
//                capability: RecordCapability::ReadField("name".to_string()),
//            },
//            RecordEffect::UpdateField {
//                resource_id: test_resource_id(),
//                field: "age".to_string(),
//                value: Term::unit(),
//                capability: RecordCapability::WriteField("age".to_string()),
//            },
//        ];
//        
//        let available_capabilities = vec![
//            Capability::read_field("record", "name"),
//            Capability::write_field("record", "age"),
//        ];
//        
//        // Should validate successfully
//        assert!(validate_capabilities(&operations, &available_capabilities).is_ok());
//        
//        // Should fail with insufficient capabilities
//        let insufficient_capabilities = vec![
//            Capability::read_field("record", "name"),
//            // Missing write capability for age
//        ];
//        
//        assert!(validate_capabilities(&operations, &insufficient_capabilities).is_err());
//    }
//
//    #[test]
//    fn test_full_record_access_capability() {
//        let resource_id = test_resource_id();
//        let full_access = RecordCapability::FullRecordAccess;
//        
//        // Full access should allow any field operation
//        let read_effect = access_field(resource_id, "any_field", full_access.clone());
//        let write_effect = update_field(
//            resource_id, 
//            "any_field", 
//            Term::unit(), 
//            full_access.clone()
//        );
//        
//        // Both should succeed (not produce capability errors)
//        match read_effect.kind {
//            EffectExpr::Perform { effect_tag, .. } => {
//                assert_eq!(effect_tag, "record.access_field");
//            }
//            _ => panic!("Expected successful access effect"),
//        }
//        
//        match write_effect.kind {
//            EffectExpr::Perform { effect_tag, .. } => {
//                assert_eq!(effect_tag, "record.update_field");
//            }
//            _ => panic!("Expected successful update effect"),
//        }
//    }
//} 