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
    session::SessionType,
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
    
    /// Required session protocols
    pub session_requirements: Vec<SessionRequirement>,
    
    /// Session endpoints this intent provides
    pub session_endpoints: Vec<SessionEndpoint>,
    
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

/// Session requirement specification for intents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRequirement {
    /// Name of the required session
    pub session_name: String,
    
    /// Role that this intent will play in the session
    pub role: String,
    
    /// Required protocol for this session role
    pub required_protocol: SessionType,
    
    /// Binding name for the session channel (optional)
    pub channel_binding: Option<String>,
}

/// Session endpoint that this intent provides
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEndpoint {
    /// Name of the session this endpoint supports
    pub session_name: String,
    
    /// Role that this endpoint plays
    pub role: String,
    
    /// Protocol that this endpoint implements
    pub protocol: SessionType,
    
    /// Binding name for the session channel
    pub channel_binding: String,
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
    
    /// Session constraints
    
    /// Require session protocol compliance
    SessionCompliant(String, String), // session_name, role
    
    /// Session ordering constraint  
    SessionBefore(String, String), // session1, session2
    
    /// Session endpoint availability
    SessionEndpointAvailable(String, String), // session_name, role
    
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
            session_requirements: Vec::new(),
            session_endpoints: Vec::new(),
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
    #[allow(clippy::only_used_in_recursion)]
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
            Constraint::SessionCompliant(session_name, _role) => {
                if !self.has_binding(session_name) && !output_names.contains(session_name) {
                    return Err(IntentError::InvalidBinding(
                        format!("SessionCompliant references unknown binding: {}", session_name)
                    ));
                }
                Ok(())
            }
            Constraint::SessionBefore(session1, session2) => {
                if !self.has_binding(session1) && !output_names.contains(session1) {
                    return Err(IntentError::InvalidBinding(
                        format!("SessionBefore constraint references unknown binding: {}", session1)
                    ));
                }
                if !self.has_binding(session2) && !output_names.contains(session2) {
                    return Err(IntentError::InvalidBinding(
                        format!("SessionBefore constraint references unknown binding: {}", session2)
                    ));
                }
                Ok(())
            }
            Constraint::SessionEndpointAvailable(session_name, _role) => {
                if !self.has_binding(session_name) && !output_names.contains(session_name) {
                    return Err(IntentError::InvalidBinding(
                        format!("SessionEndpointAvailable references unknown binding: {}", session_name)
                    ));
                }
                Ok(())
            }
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
            Constraint::Custom(_) => Ok(()),
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
    
    /// Add a session requirement to this intent
    pub fn with_session_requirement(mut self, requirement: SessionRequirement) -> Self {
        self.session_requirements.push(requirement);
        self
    }
    
    /// Add a session endpoint to this intent
    pub fn with_session_endpoint(mut self, endpoint: SessionEndpoint) -> Self {
        self.session_endpoints.push(endpoint);
        self
    }
    
    /// Check if this intent requires a specific session
    pub fn requires_session(&self, session_name: &str, role: &str) -> bool {
        self.session_requirements.iter().any(|req| 
            req.session_name == session_name && req.role == role
        )
    }
    
    /// Check if this intent provides a specific session endpoint  
    pub fn provides_session(&self, session_name: &str, role: &str) -> bool {
        self.session_endpoints.iter().any(|endpoint|
            endpoint.session_name == session_name && endpoint.role == role
        )
    }
    
    /// Add a hint to this intent
    pub fn with_hint(mut self, hint: Hint) -> Self {
        self.hint = hint;
        self
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
        _token_type: impl Into<String>
    ) -> Self {
        let from_str = from.into();
        let to_str = to.into();
        Constraint::And(vec![
            Constraint::Conservation(
                vec![from_str.clone()],
                vec![to_str]
            ),
            Constraint::Equals(
                ValueExpr::QuantityRef(from_str),
                ValueExpr::Literal(Value::Int(amount as u32))
            ),
        ])
    }
    
    /// Create a session compliance constraint
    pub fn session_compliant(session_name: impl Into<String>, role: impl Into<String>) -> Self {
        Constraint::SessionCompliant(session_name.into(), role.into())
    }
    
    /// Create a session ordering constraint
    pub fn session_before(session1: impl Into<String>, session2: impl Into<String>) -> Self {
        Constraint::SessionBefore(session1.into(), session2.into())
    }
    
    /// Create a session endpoint availability constraint
    pub fn session_endpoint_available(session_name: impl Into<String>, role: impl Into<String>) -> Self {
        Constraint::SessionEndpointAvailable(session_name.into(), role.into())
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

impl SessionRequirement {
    /// Create a new session requirement
    pub fn new(
        session_name: impl Into<String>,
        role: impl Into<String>,
        required_protocol: SessionType,
    ) -> Self {
        Self {
            session_name: session_name.into(),
            role: role.into(),
            required_protocol,
            channel_binding: None,
        }
    }
    
    /// Set the channel binding
    pub fn with_channel_binding(mut self, binding: impl Into<String>) -> Self {
        self.channel_binding = Some(binding.into());
        self
    }
}

impl SessionEndpoint {
    /// Create a new session endpoint
    pub fn new(
        session_name: impl Into<String>,
        role: impl Into<String>,
        protocol: SessionType,
        channel_binding: impl Into<String>,
    ) -> Self {
        Self {
            session_name: session_name.into(),
            role: role.into(),
            protocol,
            channel_binding: channel_binding.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::Value;
    use crate::effect::capability::Capability;

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
        let domain_bytes = vec![42u8; 32]; // 32 bytes for proper encoding
        let domain = EntityId::from_content(&domain_bytes);
        let inputs = vec![
            ResourceBinding::new("token_in", "ERC20"),
        ];
        let constraint = Constraint::produces("token_out", "ERC20");
        
        let machine_hint = MachineHint::Custom("test_hint".to_string());
        let custom_hint = Hint::custom(machine_hint.clone());
        
        let intent = Intent::new(domain, inputs, constraint)
            .with_hint(custom_hint.clone());
        
        assert_eq!(intent.hint, custom_hint);
        if let Hint::Custom(hint) = intent.hint {
            assert_eq!(hint, machine_hint);
        } else {
            panic!("Expected custom hint");
        }
    }

    #[test]
    fn test_intent_with_session_requirements() {
        use crate::lambda::base::{TypeInner, BaseType};
        
        let domain_bytes = vec![42u8; 32]; // 32 bytes for proper encoding
        let domain = EntityId::from_content(&domain_bytes);
        let inputs = vec![
            ResourceBinding::new("token_in", "ERC20"),
        ];
        let constraint = Constraint::And(vec![
            Constraint::produces("token_out", "ERC20"),
            Constraint::session_compliant("PaymentProtocol", "client"),
        ]);
        
        let session_requirement = SessionRequirement::new(
            "PaymentProtocol",
            "client", 
            SessionType::Send(
                TypeInner::Base(BaseType::Int),
                Box::new(SessionType::End)
            )
        ).with_channel_binding("payment_channel");
        
        let intent = Intent::new(domain, inputs, constraint)
            .with_session_requirement(session_requirement);
        
        assert_eq!(intent.session_requirements.len(), 1);
        assert!(intent.requires_session("PaymentProtocol", "client"));
        assert!(!intent.requires_session("PaymentProtocol", "server"));
        
        let req = &intent.session_requirements[0];
        assert_eq!(req.session_name, "PaymentProtocol");
        assert_eq!(req.role, "client");
        assert_eq!(req.channel_binding, Some("payment_channel".to_string()));
    }
    
    #[test]
    fn test_intent_with_session_endpoints() {
        use crate::lambda::base::{TypeInner, BaseType};
        
        let domain_bytes = vec![42u8; 32]; // 32 bytes for proper encoding
        let domain = EntityId::from_content(&domain_bytes);
        let inputs = vec![
            ResourceBinding::new("payment_request", "PaymentRequest"),
        ];
        let constraint = Constraint::session_endpoint_available("PaymentProtocol", "server");
        
        let session_endpoint = SessionEndpoint::new(
            "PaymentProtocol",
            "server",
            SessionType::Receive(
                TypeInner::Base(BaseType::Int),
                Box::new(SessionType::End)
            ),
            "payment_channel"
        );
        
        let intent = Intent::new(domain, inputs, constraint)
            .with_session_endpoint(session_endpoint);
        
        assert_eq!(intent.session_endpoints.len(), 1);
        assert!(intent.provides_session("PaymentProtocol", "server"));
        assert!(!intent.provides_session("PaymentProtocol", "client"));
        
        let endpoint = &intent.session_endpoints[0];
        assert_eq!(endpoint.session_name, "PaymentProtocol");
        assert_eq!(endpoint.role, "server");
        assert_eq!(endpoint.channel_binding, "payment_channel");
    }
    
    #[test]
    fn test_session_constraint_validation() {
        let domain_bytes = vec![42u8; 32]; // 32 bytes for proper encoding
        let domain = EntityId::from_content(&domain_bytes);
        let inputs = vec![
            ResourceBinding::new("session1", "SessionChannel"),
            ResourceBinding::new("session2", "SessionChannel"),
        ];
        
        // Valid session constraints
        let valid_constraint = Constraint::And(vec![
            Constraint::session_compliant("session1", "client"),
            Constraint::session_before("session1", "session2"),
            Constraint::session_endpoint_available("session2", "server"),
        ]);
        
        let intent = Intent::new(domain.clone(), inputs, valid_constraint);
        assert!(intent.validate().is_ok());
        
        // Invalid session constraint (references unknown binding)
        let invalid_constraint = Constraint::session_compliant("unknown_session", "client");
        let invalid_intent = Intent::new(domain, vec![], invalid_constraint);
        assert!(invalid_intent.validate().is_err());
    }
} 