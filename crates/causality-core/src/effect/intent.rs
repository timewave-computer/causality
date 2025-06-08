//! Intent-based programming system for declarative effect specification
//!
//! This module implements the core intent types that allow users to specify
//! what they want to achieve declaratively, rather than how to achieve it.

use std::time::{SystemTime, UNIX_EPOCH};
use crate::{
    system::content_addressing::{EntityId, Timestamp, DomainId},
    lambda::base::Value,
    machine::instruction::{ConstraintExpr, MachineHint},
};
use super::{
    capability::Capability,
};

/// Unique identifier for intents
pub type IntentId = EntityId;

/// A declarative intent specifying desired outcomes without implementation details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intent {
    /// Unique content-addressed identifier
    pub id: IntentId,
    
    /// Domain where this intent operates
    pub domain: DomainId,
    
    /// Required input resources
    pub inputs: Vec<ResourceBinding>,
    
    /// The constraint that must be satisfied (purely declarative)
    /// This expresses what should be true after execution
    pub constraint: Constraint,
    
    /// Hint expression for runtime optimization (optional)
    /// This guides the solver without affecting correctness
    pub hint: Hint,
    
    /// When this intent was created
    pub timestamp: Timestamp,
}

/// Binding specification for resources in intents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBinding {
    /// Binding name (used in constraints)
    pub name: String,
    
    /// Expected resource type/label
    pub resource_type: String,
    
    /// Required quantity (None means any amount)
    pub quantity: Option<u64>,
    
    /// Additional constraints on this resource
    pub constraints: Vec<Constraint>,
    
    /// Required capabilities for operations on this resource
    pub capabilities: Vec<Capability>,
    
    /// Optional metadata
    pub metadata: Value,
}

/// Declarative constraints that must be satisfied
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// Always true (trivial constraint)
    True,
    
    /// Always false (impossible constraint)  
    False,
    
    /// Logical conjunction (all must be true)
    And(Vec<Constraint>),
    
    /// Logical disjunction (at least one must be true)
    Or(Vec<Constraint>),
    
    /// Logical negation
    Not(Box<Constraint>),
    
    /// Equality constraint between two values
    Equals(ValueExpr, ValueExpr),
    
    /// Less than constraint
    LessThan(ValueExpr, ValueExpr),
    
    /// Greater than constraint
    GreaterThan(ValueExpr, ValueExpr),
    
    /// Require a capability to be held
    HasCapability(ResourceRef, String),
    
    /// Conservation constraint (inputs must equal outputs)
    Conservation(Vec<String>, Vec<String>),
    
    /// Temporal ordering constraint
    Before(String, String),
    
    /// Require that a resource exists (typically outputs)
    Exists(ResourceBinding),
    
    /// Require that multiple resources exist
    ExistsAll(Vec<ResourceBinding>),
    
    /// Custom constraint expression from machine layer
    Custom(ConstraintExpr),
}

/// Value expressions for constraint evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpr {
    /// Literal value
    Literal(Value),
    
    /// Reference to a resource binding
    ResourceRef(String),
    
    /// Reference to resource metadata field
    MetadataRef(String, String), // binding_name, field_name
    
    /// Reference to resource quantity
    QuantityRef(String),
    
    /// Arithmetic operations
    Add(Box<ValueExpr>, Box<ValueExpr>),
    Sub(Box<ValueExpr>, Box<ValueExpr>),
    Mul(Box<ValueExpr>, Box<ValueExpr>),
    Div(Box<ValueExpr>, Box<ValueExpr>),
    
    /// Function application
    Apply(String, Vec<ValueExpr>), // function_name, args
}

/// Reference to a resource in constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRef {
    /// Reference by binding name
    ByName(String),
    
    /// Reference by resource ID
    ById(EntityId),
    
    /// Reference to input resource by index
    Input(usize),
    
    /// Reference to output resource by index
    Output(usize),
}

/// Runtime optimization hints that guide the solver without affecting correctness
/// These mirror the constraint structure but provide optimization guidance
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hint {
    /// No optimization hint (trivial hint)
    True,
    
    /// Impossible hint (should not occur)
    False,
    
    /// All hints should be considered (conjunction)
    And(Vec<Hint>),
    
    /// Any of these hints may be considered (disjunction)
    Or(Vec<Hint>),
    
    /// Negation of a hint
    Not(Box<Hint>),
    
    /// Batch effects with matching selector
    BatchWith(String), // selector for batching strategy
    
    /// Minimize a specific metric (price, latency, etc.)
    Minimize(String), // metric name
    
    /// Maximize a specific metric
    Maximize(String), // metric name
    
    /// Prefer execution in a specific domain
    PreferDomain(DomainId),
    
    /// Deadline constraint for completion
    Deadline(Timestamp),
    
    /// Prefer parallel execution where possible
    PreferParallel,
    
    /// Prefer sequential execution
    PreferSequential,
    
    /// Resource usage limit hint
    ResourceLimit(String, u64), // resource_type, max_amount
    
    /// Cost budget hint
    CostBudget(u64), // max_cost
    
    /// Custom optimization hint from machine layer
    Custom(MachineHint), // structured machine-level hint
}

/// Intent validation and processing errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentError {
    /// Invalid constraint
    InvalidConstraint(String),
    
    /// Missing required resource
    MissingResource(String),
    
    /// Constraint evaluation failed
    ConstraintFailed(String),
    
    /// Invalid resource binding
    InvalidBinding(String),
    
    /// Unsupported operation
    UnsupportedOperation(String),
    
    /// Synthesis failed
    SynthesisFailed(String),
}

impl Intent {
    /// Create a new intent
    pub fn new(
        domain: DomainId,
        inputs: Vec<ResourceBinding>,
        constraint: Constraint,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Generate content-addressed ID using simple string hash
        let content_str = format!("intent_{}_{}", domain, timestamp);
        let content_bytes: Vec<u8> = content_str.as_bytes().to_vec();
        let id = EntityId::from_content(&content_bytes);
        
        Self {
            id,
            domain,
            inputs,
            constraint,
            hint: Hint::True,
            timestamp: Timestamp { millis: timestamp },
        }
    }
    
    /// Validate this intent for basic consistency
    pub fn validate(&self) -> Result<(), IntentError> {
        // Check that all resource bindings have unique names
        let mut names = std::collections::HashSet::new();
        for binding in &self.inputs {
            if !names.insert(&binding.name) {
                return Err(IntentError::InvalidBinding(
                    format!("Duplicate input binding name: {}", binding.name)
                ));
            }
        }
        
        // Collect all output binding names from the constraint tree
        let output_names = self.collect_output_names(&self.constraint);
        
        // Validate constraints reference valid bindings
        self.validate_constraint_with_outputs(&self.constraint, &output_names)?;
        
        Ok(())
    }
    
    /// Collect all output binding names declared in constraint expressions
    fn collect_output_names(&self, constraint: &Constraint) -> std::collections::HashSet<String> {
        let mut output_names = std::collections::HashSet::new();
        self.collect_output_names_recursive(constraint, &mut output_names);
        output_names
    }
    
    /// Recursively collect output names from constraint tree
    fn collect_output_names_recursive(&self, constraint: &Constraint, output_names: &mut std::collections::HashSet<String>) {
        match constraint {
            Constraint::And(constraints) | Constraint::Or(constraints) => {
                for constraint in constraints {
                    self.collect_output_names_recursive(constraint, output_names);
                }
            }
            Constraint::Not(constraint) => {
                self.collect_output_names_recursive(constraint, output_names);
            }
            Constraint::Exists(binding) => {
                output_names.insert(binding.name.clone());
            }
            Constraint::ExistsAll(bindings) => {
                for binding in bindings {
                    output_names.insert(binding.name.clone());
                }
            }
            _ => {} // Other constraints don't declare output names
        }
    }
    
    /// Validate a constraint references valid resource bindings (including outputs)
    fn validate_constraint_with_outputs(&self, constraint: &Constraint, output_names: &std::collections::HashSet<String>) -> Result<(), IntentError> {
        match constraint {
            Constraint::True | Constraint::False => Ok(()),
            Constraint::And(constraints) | Constraint::Or(constraints) => {
                for constraint in constraints {
                    self.validate_constraint_with_outputs(constraint, output_names)?;
                }
                Ok(())
            }
            Constraint::Not(constraint) => {
                self.validate_constraint_with_outputs(constraint, output_names)
            }
            Constraint::Equals(expr1, expr2) |
            Constraint::LessThan(expr1, expr2) |
            Constraint::GreaterThan(expr1, expr2) => {
                self.validate_value_expr_with_outputs(expr1, output_names)?;
                self.validate_value_expr_with_outputs(expr2, output_names)?;
                Ok(())
            }
            Constraint::HasCapability(resource_ref, _) => {
                self.validate_resource_ref_with_outputs(resource_ref, output_names)
            }
            Constraint::Conservation(inputs, outputs) => {
                for input_name in inputs {
                    if !self.has_binding(input_name) && !output_names.contains(input_name) {
                        return Err(IntentError::InvalidBinding(
                            format!("Conservation input references unknown binding: {}", input_name)
                        ));
                    }
                }
                for output_name in outputs {
                    if !self.has_binding(output_name) && !output_names.contains(output_name) {
                        return Err(IntentError::InvalidBinding(
                            format!("Conservation output references unknown binding: {}", output_name)
                        ));
                    }
                }
                Ok(())
            }
            Constraint::Custom(_) => Ok(()),
            Constraint::Before(r1, r2) => {
                if !self.has_binding(r1) && !output_names.contains(r1) {
                    return Err(IntentError::InvalidBinding(
                        format!("Before constraint references unknown binding: {}", r1)
                    ));
                }
                if !self.has_binding(r2) && !output_names.contains(r2) {
                    return Err(IntentError::InvalidBinding(
                        format!("Before constraint references unknown binding: {}", r2)
                    ));
                }
                Ok(())
            }
            Constraint::Exists(_binding) => {
                // For Exists, we don't require the binding to already exist
                // This is for specifying desired outputs
                Ok(())
            }
            Constraint::ExistsAll(_bindings) => {
                // Same for ExistsAll - these are output specifications
                Ok(())
            }
        }
    }
    
    /// Validate a value expression (including outputs)
    fn validate_value_expr_with_outputs(&self, expr: &ValueExpr, output_names: &std::collections::HashSet<String>) -> Result<(), IntentError> {
        match expr {
            ValueExpr::Literal(_) => Ok(()),
            ValueExpr::ResourceRef(name) => {
                if self.has_binding(name) || output_names.contains(name) {
                    Ok(())
                } else {
                    Err(IntentError::InvalidConstraint(
                        format!("Unknown resource binding: {}", name)
                    ))
                }
            }
            ValueExpr::MetadataRef(name, _) => {
                if self.has_binding(name) || output_names.contains(name) {
                    Ok(())
                } else {
                    Err(IntentError::InvalidConstraint(
                        format!("Unknown resource binding: {}", name)
                    ))
                }
            }
            ValueExpr::QuantityRef(name) => {
                if self.has_binding(name) || output_names.contains(name) {
                    Ok(())
                } else {
                    Err(IntentError::InvalidConstraint(
                        format!("Unknown resource binding: {}", name)
                    ))
                }
            }
            ValueExpr::Add(left, right) |
            ValueExpr::Sub(left, right) |
            ValueExpr::Mul(left, right) |
            ValueExpr::Div(left, right) => {
                self.validate_value_expr_with_outputs(left, output_names)?;
                self.validate_value_expr_with_outputs(right, output_names)?;
                Ok(())
            }
            ValueExpr::Apply(_, args) => {
                for arg in args {
                    self.validate_value_expr_with_outputs(arg, output_names)?;
                }
                Ok(())
            }
        }
    }
    
    /// Validate a resource reference (including outputs)
    fn validate_resource_ref_with_outputs(&self, resource_ref: &ResourceRef, output_names: &std::collections::HashSet<String>) -> Result<(), IntentError> {
        match resource_ref {
            ResourceRef::ByName(name) => {
                if self.has_binding(name) || output_names.contains(name) {
                    Ok(())
                } else {
                    Err(IntentError::InvalidConstraint(
                        format!("Unknown resource binding: {}", name)
                    ))
                }
            }
            ResourceRef::ById(_) => Ok(()), // Always valid for external references
            ResourceRef::Input(index) => {
                if *index < self.inputs.len() {
                    Ok(())
                } else {
                    Err(IntentError::InvalidConstraint(
                        format!("Input index {} out of bounds", index)
                    ))
                }
            }
            ResourceRef::Output(index) => {
                if *index < self.inputs.len() {
                    Ok(())
                } else {
                    Err(IntentError::InvalidConstraint(
                        format!("Output index {} out of bounds", index)
                    ))
                }
            }
        }
    }
    
    /// Check if a binding name exists
    fn has_binding(&self, name: &str) -> bool {
        self.inputs.iter().any(|b| b.name == name)
    }
    
    /// Get all resource binding names
    pub fn get_binding_names(&self) -> Vec<String> {
        self.inputs.iter().map(|b| b.name.clone()).collect()
    }
    
    /// Get a resource binding by name
    pub fn get_binding(&self, name: &str) -> Option<&ResourceBinding> {
        self.inputs.iter().find(|b| b.name == name)
    }
}

impl ResourceBinding {
    /// Create a new resource binding
    pub fn new(
        name: impl Into<String>,
        resource_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            resource_type: resource_type.into(),
            quantity: None,
            constraints: Vec::new(),
            capabilities: Vec::new(),
            metadata: Value::Bool(false),
        }
    }
    
    /// Set the required quantity
    pub fn with_quantity(mut self, quantity: u64) -> Self {
        self.quantity = Some(quantity);
        self
    }
    
    /// Add a constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

// Helper functions for constraint construction
impl Constraint {
    /// Create a logical AND constraint
    pub fn and(constraints: Vec<Constraint>) -> Self {
        Constraint::And(constraints)
    }
    
    /// Create a logical OR constraint
    pub fn or(constraints: Vec<Constraint>) -> Self {
        Constraint::Or(constraints)
    }
    
    /// Create a logical NOT constraint
    #[allow(clippy::should_implement_trait)]
    pub fn not(constraint: Constraint) -> Self {
        Constraint::Not(Box::new(constraint))
    }
    
    /// Create an equality constraint
    pub fn equals(left: ValueExpr, right: ValueExpr) -> Self {
        Constraint::Equals(left, right)
    }
    
    /// Create a conservation constraint
    pub fn conservation(inputs: Vec<String>, outputs: Vec<String>) -> Self {
        Constraint::Conservation(inputs, outputs)
    }
    
    /// Create a capability constraint
    pub fn has_capability(resource: impl Into<String>, capability: impl Into<String>) -> Self {
        Constraint::HasCapability(ResourceRef::ByName(resource.into()), capability.into())
    }
    
    /// Create an output existence constraint
    pub fn produces(name: impl Into<String>, resource_type: impl Into<String>) -> Self {
        Constraint::Exists(ResourceBinding::new(name.into(), resource_type.into()))
    }
    
    /// Create an output existence constraint with quantity
    pub fn produces_quantity(
        name: impl Into<String>, 
        resource_type: impl Into<String>, 
        quantity: u64
    ) -> Self {
        Constraint::Exists(
            ResourceBinding::new(name.into(), resource_type.into()).with_quantity(quantity)
        )
    }
    
    /// Create a constraint requiring multiple outputs
    pub fn produces_all(outputs: Vec<ResourceBinding>) -> Self {
        Constraint::ExistsAll(outputs)
    }
    
    /// Create a transfer constraint (common pattern)
    pub fn transfer(
        from: impl Into<String>,
        to: impl Into<String>, 
        amount: u64,
        token_type: impl Into<String>
    ) -> Self {
        let token_type = token_type.into();
        let to_string = to.into();
        Constraint::and(vec![
            // Conservation: input amount equals output amount
            Constraint::conservation(
                vec![from.into()],
                vec![to_string.clone()],
            ),
            // Output exists with correct type and amount
            Constraint::produces_quantity(to_string, token_type, amount),
        ])
    }
}

// Helper functions for value expressions
impl ValueExpr {
    /// Create a literal value expression
    pub fn literal(value: Value) -> Self {
        ValueExpr::Literal(value)
    }
    
    /// Create a resource reference
    pub fn resource(name: impl Into<String>) -> Self {
        ValueExpr::ResourceRef(name.into())
    }
    
    /// Create a quantity reference
    pub fn quantity(name: impl Into<String>) -> Self {
        ValueExpr::QuantityRef(name.into())
    }
    
    /// Create an addition expression
    #[allow(clippy::should_implement_trait)]
    pub fn add(left: ValueExpr, right: ValueExpr) -> Self {
        ValueExpr::Add(Box::new(left), Box::new(right))
    }
}

// Error implementations
impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentError::InvalidConstraint(msg) => write!(f, "Invalid constraint: {}", msg),
            IntentError::MissingResource(name) => write!(f, "Missing resource: {}", name),
            IntentError::ConstraintFailed(msg) => write!(f, "Constraint failed: {}", msg),
            IntentError::InvalidBinding(msg) => write!(f, "Invalid binding: {}", msg),
            IntentError::UnsupportedOperation(op) => write!(f, "Unsupported operation: {}", op),
            IntentError::SynthesisFailed(msg) => write!(f, "Synthesis failed: {}", msg),
        }
    }
}

impl std::error::Error for IntentError {}

impl Hint {
    /// Create a logical AND hint
    pub fn and(hints: Vec<Hint>) -> Self {
        Hint::And(hints)
    }
    
    /// Create a logical OR hint
    pub fn or(hints: Vec<Hint>) -> Self {
        Hint::Or(hints)
    }
    
    /// Create a logical NOT hint
    #[allow(clippy::should_implement_trait)]
    pub fn not(hint: Hint) -> Self {
        Hint::Not(Box::new(hint))
    }
    
    /// Create a batching hint
    pub fn batch_with(selector: impl Into<String>) -> Self {
        Hint::BatchWith(selector.into())
    }
    
    /// Create a minimize hint
    pub fn minimize(metric: impl Into<String>) -> Self {
        Hint::Minimize(metric.into())
    }
    
    /// Create a maximize hint
    pub fn maximize(metric: impl Into<String>) -> Self {
        Hint::Maximize(metric.into())
    }
    
    /// Create a domain preference hint
    pub fn prefer_domain(domain: DomainId) -> Self {
        Hint::PreferDomain(domain)
    }
    
    /// Create a deadline hint
    pub fn deadline(timestamp: Timestamp) -> Self {
        Hint::Deadline(timestamp)
    }
    
    /// Create a cost budget hint
    pub fn cost_budget(max_cost: u64) -> Self {
        Hint::CostBudget(max_cost)
    }
    
    /// Create a resource limit hint
    pub fn resource_limit(resource_type: impl Into<String>, max_amount: u64) -> Self {
        Hint::ResourceLimit(resource_type.into(), max_amount)
    }
    
    /// Create a custom machine-level hint
    pub fn custom(machine_hint: MachineHint) -> Self {
        Hint::Custom(machine_hint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::Value;
    use crate::effect::capability::Capability;
    use crate::machine::instruction::{Metric, Selector};

    #[test]
    fn test_intent_creation() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let intent = Intent::new(
            domain,
            vec![
                ResourceBinding::new("source_account", "Account").with_quantity(100),
                ResourceBinding::new("tokens", "Token").with_quantity(50),
            ],
            Constraint::True,
        );
        
        assert_eq!(intent.domain.to_string(), domain.to_string());
        assert_eq!(intent.inputs.len(), 2);
        assert_eq!(intent.inputs[0].name, "source_account");
        assert_eq!(intent.inputs[1].quantity, Some(50));
    }

    #[test]
    fn test_intent_validation() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        // Valid intent
        let valid_intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::True,
        );
        
        assert!(valid_intent.validate().is_ok());
        
        // Invalid intent - references unknown binding in conservation constraint
        let invalid_intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::conservation(
                vec!["unknown_binding".to_string()],
                vec!["output".to_string()],
            ),
        );
        
        assert!(invalid_intent.validate().is_err());
    }

    #[test]
    fn test_resource_binding_builder() {
        let binding = ResourceBinding::new("account", "Account")
            .with_quantity(1000)
            .with_capability(Capability::read("read"))
            .with_metadata(Value::Bool(true));
        
        assert_eq!(binding.name, "account");
        assert_eq!(binding.resource_type, "Account");
        assert_eq!(binding.quantity, Some(1000));
        assert_eq!(binding.capabilities.len(), 1);
        assert!(matches!(binding.metadata, Value::Bool(true)));
    }

    #[test]
    fn test_intent_with_transfer_constraint() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let intent = Intent::new(
            domain,
            vec![
                ResourceBinding::new("source_account", "Account"),
                ResourceBinding::new("tokens", "Token").with_quantity(50),
            ],
            Constraint::transfer("tokens", "destination_tokens", 50, "Token"),
        );
        
        assert_eq!(intent.domain.to_string(), domain.to_string());
        assert_eq!(intent.inputs.len(), 2);
        assert_eq!(intent.inputs[0].name, "source_account");
        assert_eq!(intent.inputs[1].quantity, Some(50));
        
        // Constraint should be compound (And)
        assert!(matches!(intent.constraint, Constraint::And(_)));
    }
    
    #[test] 
    fn test_intent_with_compound_constraints() {
        let domain_name = String::from("defi_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let intent = Intent::new(
            domain,
            vec![
                ResourceBinding::new("token_a", "TokenA").with_quantity(100),
            ],
            Constraint::and(vec![
                // Must produce token B
                Constraint::produces_quantity("token_b", "TokenB", 90),
                // Must maintain value conservation (simplified)
                Constraint::equals(
                    ValueExpr::quantity("token_a"),
                    ValueExpr::literal(Value::Int(100)),
                ),
                // Output quantity constraint
                Constraint::equals(
                    ValueExpr::QuantityRef("token_b".to_string()),
                    ValueExpr::literal(Value::Int(90)),
                ),
            ])
        );
        
        assert!(matches!(intent.constraint, Constraint::And(_)));
        if let Constraint::And(constraints) = &intent.constraint {
            assert_eq!(constraints.len(), 3);
            assert!(matches!(constraints[0], Constraint::Exists(_)));
        }
    }

    #[test]
    fn test_constraint_validation() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let valid_intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::produces("output", "Token"),
        );
        
        assert!(valid_intent.validate().is_ok());
    }

    #[test]
    fn test_constraint_validation_error() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let invalid_intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::conservation(
                vec!["unknown_binding".to_string()],
                vec!["output".to_string()],
            ),
        );
        
        assert!(invalid_intent.validate().is_err());
    }

    #[test]
    fn test_intent_with_hints() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let mut intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::produces("output", "Token"),
        );
        
        // Test hint composition
        intent.hint = Hint::and(vec![
            Hint::minimize("latency"),
            Hint::prefer_domain(domain),
            Hint::cost_budget(1000),
            Hint::batch_with("same_type"),
        ]);
        
        assert!(matches!(intent.hint, Hint::And(_)));
        if let Hint::And(hints) = &intent.hint {
            assert_eq!(hints.len(), 4);
            assert!(matches!(hints[0], Hint::Minimize(_)));
            assert!(matches!(hints[1], Hint::PreferDomain(_)));
            assert!(matches!(hints[2], Hint::CostBudget(_)));
            assert!(matches!(hints[3], Hint::BatchWith(_)));
        }
    }

    #[test]
    fn test_intent_with_custom_machine_hints() {
        let domain_name = String::from("test_domain");
        let domain = EntityId::from_content(&domain_name.as_bytes().to_vec());
        
        let mut intent = Intent::new(
            domain,
            vec![ResourceBinding::new("input", "Token")],
            Constraint::produces("output", "Token"),
        );
        
        // Test machine-level custom hints
        intent.hint = Hint::custom(MachineHint::HintAll(vec![
            MachineHint::BatchWith(Selector::SameType),
            MachineHint::Minimize(Metric::Latency),
            MachineHint::PreferDomain(domain.to_string()),
            MachineHint::Deadline(1000),
        ]));
        
        assert!(matches!(intent.hint, Hint::Custom(_)));
        if let Hint::Custom(machine_hint) = &intent.hint {
            assert!(matches!(machine_hint, MachineHint::HintAll(_)));
            if let MachineHint::HintAll(hints) = machine_hint {
                assert_eq!(hints.len(), 4);
                assert!(matches!(hints[2], MachineHint::PreferDomain(_)));
            }
        }
    }
} 