//! Fluent Intent Builder DSL
//!
//! This module provides ergonomic builders for constructing Intent and related types
//! with method chaining and type safety.

use causality_core::{
    effect::{Intent, ResourceBinding, Constraint, ValueExpr},
    effect::intent::Hint,
    system::content_addressing::DomainId,
    lambda::base::Value,
    effect::capability::Capability,
};

/// Fluent builder for constructing Intents
#[derive(Debug, Clone)]
pub struct IntentBuilder {
    domain: Option<DomainId>,
    inputs: Vec<ResourceBinding>,
    constraint: Option<Constraint>,
    hint: Option<Hint>,
}

/// Fluent builder for constructing ResourceBindings
#[derive(Debug, Clone)]
pub struct ResourceBindingBuilder {
    name: Option<String>,
    resource_type: Option<String>,
    quantity: Option<u64>,
    constraints: Vec<Constraint>,
    capabilities: Vec<Capability>,
    metadata: Option<Value>,
}

/// Fluent builder for constructing Constraints
#[derive(Debug, Clone)]
pub struct ConstraintBuilder {
    constraints: Vec<Constraint>,
}

impl IntentBuilder {
    /// Create a new Intent builder
    pub fn new() -> Self {
        Self {
            domain: None,
            inputs: Vec::new(),
            constraint: None,
            hint: None,
        }
    }
    
    /// Set the domain for this intent
    pub fn domain(mut self, domain: DomainId) -> Self {
        self.domain = Some(domain);
        self
    }
    
    /// Add an input resource binding
    pub fn with_input(mut self, binding: ResourceBinding) -> Self {
        self.inputs.push(binding);
        self
    }
    
    /// Add an input resource with name and type (convenience method)
    pub fn input(mut self, name: &str, resource_type: &str) -> Self {
        self.inputs.push(ResourceBinding::new(name, resource_type));
        self
    }
    
    /// Add an input resource with quantity
    pub fn input_quantity(mut self, name: &str, resource_type: &str, quantity: u64) -> Self {
        self.inputs.push(ResourceBinding::new(name, resource_type).with_quantity(quantity));
        self
    }
    
    /// Set the constraint for this intent
    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = Some(constraint);
        self
    }
    
    /// Set a simple transfer constraint (convenience method)
    pub fn transfer(mut self, from: &str, to: &str, amount: u64, token_type: &str) -> Self {
        self.constraint = Some(Constraint::transfer(from, to, amount, token_type));
        self
    }
    
    /// Set an exchange constraint (input A for output B)
    pub fn exchange(mut self, input_name: &str, output_name: &str, output_type: &str, output_amount: u64) -> Self {
        self.constraint = Some(Constraint::and(vec![
            Constraint::conservation(
                vec![input_name.to_string()],
                vec![output_name.to_string()],
            ),
            Constraint::produces_quantity(output_name, output_type, output_amount),
        ]));
        self
    }
    
    /// Set the optimization hint
    pub fn hint(mut self, hint: Hint) -> Self {
        self.hint = Some(hint);
        self
    }
    
    /// Add minimize latency hint (convenience method)
    pub fn minimize_latency(self) -> Self {
        self.hint(Hint::minimize("latency"))
    }
    
    /// Add minimize cost hint (convenience method)  
    pub fn minimize_cost(self) -> Self {
        self.hint(Hint::minimize("cost"))
    }
    
    /// Add cost budget hint (convenience method)
    pub fn budget(self, max_cost: u64) -> Self {
        self.hint(Hint::cost_budget(max_cost))
    }
    
    /// Build the Intent
    pub fn build(self) -> Result<Intent, String> {
        let domain = self.domain.ok_or("Domain is required")?;
        let constraint = self.constraint.unwrap_or(Constraint::True);
        
        let mut intent = Intent::new(domain, self.inputs, constraint);
        
        if let Some(hint) = self.hint {
            intent.hint = hint;
        }
        
        Ok(intent)
    }
}

impl ResourceBindingBuilder {
    /// Create a new ResourceBinding builder
    pub fn new() -> Self {
        Self {
            name: None,
            resource_type: None,
            quantity: None,
            constraints: Vec::new(),
            capabilities: Vec::new(),
            metadata: None,
        }
    }
    
    /// Set the binding name
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set the resource type
    pub fn resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }
    
    /// Set the required quantity
    pub fn quantity(mut self, quantity: u64) -> Self {
        self.quantity = Some(quantity);
        self
    }
    
    /// Add a constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Add a capability requirement
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add read capability requirement (convenience method)
    pub fn readable(mut self) -> Self {
        self.capabilities.push(Capability::read("read"));
        self
    }
    
    /// Add write capability requirement (convenience method)
    pub fn writable(mut self) -> Self {
        self.capabilities.push(Capability::write("write"));
        self
    }
    
    /// Set metadata
    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Build the ResourceBinding
    pub fn build(self) -> Result<ResourceBinding, String> {
        let name = self.name.ok_or("Name is required")?;
        let resource_type = self.resource_type.ok_or("Resource type is required")?;
        
        let mut binding = ResourceBinding::new(name, resource_type);
        
        if let Some(quantity) = self.quantity {
            binding = binding.with_quantity(quantity);
        }
        
        for constraint in self.constraints {
            binding = binding.with_constraint(constraint);
        }
        
        for capability in self.capabilities {
            binding = binding.with_capability(capability);
        }
        
        if let Some(metadata) = self.metadata {
            binding = binding.with_metadata(metadata);
        }
        
        Ok(binding)
    }
}

impl ConstraintBuilder {
    /// Create a new Constraint builder
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }
    
    /// Add a constraint
    pub fn and(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Add a transfer constraint (convenience method)
    pub fn transfer(self, from: &str, to: &str, amount: u64, token_type: &str) -> Self {
        self.and(Constraint::transfer(from, to, amount, token_type))
    }
    
    /// Add a conservation constraint
    pub fn conservation(self, inputs: Vec<&str>, outputs: Vec<&str>) -> Self {
        self.and(Constraint::conservation(
            inputs.into_iter().map(|s| s.to_string()).collect(),
            outputs.into_iter().map(|s| s.to_string()).collect(),
        ))
    }
    
    /// Add an equality constraint
    pub fn equals(self, left: ValueExpr, right: ValueExpr) -> Self {
        self.and(Constraint::equals(left, right))
    }
    
    /// Add a production constraint
    pub fn produces(self, name: &str, resource_type: &str) -> Self {
        self.and(Constraint::produces(name, resource_type))
    }
    
    /// Add a production constraint with quantity
    pub fn produces_quantity(self, name: &str, resource_type: &str, quantity: u64) -> Self {
        self.and(Constraint::produces_quantity(name, resource_type, quantity))
    }
    
    /// Build the final Constraint
    pub fn build(self) -> Constraint {
        match self.constraints.len() {
            0 => Constraint::True,
            1 => self.constraints.into_iter().next().unwrap(),
            _ => Constraint::And(self.constraints),
        }
    }
}

/// Convenience macro for building intents
#[macro_export]
macro_rules! intent {
    ($domain:expr) => {
        $crate::dsl::intent_builder::IntentBuilder::new().domain($domain)
    };
}

/// Convenience macro for building resource bindings
#[macro_export]
macro_rules! resource {
    ($name:expr, $type:expr) => {
        $crate::dsl::intent_builder::ResourceBindingBuilder::new()
            .name($name)
            .resource_type($type)
    };
    ($name:expr, $type:expr, $quantity:expr) => {
        $crate::dsl::intent_builder::ResourceBindingBuilder::new()
            .name($name)
            .resource_type($type)
            .quantity($quantity)
    };
}

/// Convenience macro for building constraints
#[macro_export]
macro_rules! constraints {
    () => {
        $crate::dsl::intent_builder::ConstraintBuilder::new()
    };
}

// Re-export for easy access
pub use intent;
pub use resource;
pub use constraints;

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::system::content_addressing::{EntityId, Str};

    fn test_domain() -> DomainId {
        EntityId::from_content(&Str::new("test_domain"))
    }

    #[test]
    fn test_intent_builder_basic() {
        let domain = test_domain();
        
        let intent = IntentBuilder::new()
            .domain(domain)
            .input("token_a", "TokenA")
            .input_quantity("token_b", "TokenB", 100)
            .transfer("token_b", "token_c", 100, "TokenB")
            .minimize_cost()
            .build()
            .unwrap();
        
        assert_eq!(intent.domain, domain);
        assert_eq!(intent.inputs.len(), 2);
        assert_eq!(intent.inputs[0].name, "token_a");
        assert_eq!(intent.inputs[1].quantity, Some(100));
        assert!(matches!(intent.hint, Hint::Minimize(_)));
    }

    #[test]
    fn test_resource_builder() {
        let binding = ResourceBindingBuilder::new()
            .name("account")
            .resource_type("Account")
            .quantity(1000)
            .readable()
            .writable()
            .metadata(Value::Bool(true))
            .build()
            .unwrap();
        
        assert_eq!(binding.name, "account");
        assert_eq!(binding.resource_type, "Account");
        assert_eq!(binding.quantity, Some(1000));
        assert_eq!(binding.capabilities.len(), 2);
        assert!(matches!(binding.metadata, Value::Bool(true)));
    }

    #[test]
    fn test_constraint_builder() {
        let constraint = ConstraintBuilder::new()
            .transfer("input", "output", 50, "Token")
            .conservation(vec!["input"], vec!["output"])
            .produces_quantity("result", "Result", 1)
            .build();
        
        // Should be an And constraint with multiple sub-constraints
        assert!(matches!(constraint, Constraint::And(_)));
    }

    #[test]
    fn test_macros() {
        let domain = test_domain();
        
        // Test intent macro
        let intent = intent!(domain)
            .input("token", "Token")
            .build()
            .unwrap();
        
        assert_eq!(intent.domain, domain);
        
        // Test resource macro
        let resource_basic = resource!("test", "Test").build().unwrap();
        assert_eq!(resource_basic.name, "test");
        
        let resource_with_quantity = resource!("test2", "Test2", 100).build().unwrap();
        assert_eq!(resource_with_quantity.quantity, Some(100));
        
        // Test constraint macro
        let constraint = constraints!()
            .produces("output", "Token")
            .build();
        
        assert!(matches!(constraint, Constraint::Exists(_)));
    }

    #[test]
    fn test_exchange_builder() {
        let domain = test_domain();
        
        let intent = IntentBuilder::new()
            .domain(domain)
            .input_quantity("token_a", "TokenA", 100)
            .exchange("token_a", "token_b", "TokenB", 90)
            .budget(1000)
            .build()
            .unwrap();
        
        assert_eq!(intent.inputs.len(), 1);
        assert!(matches!(intent.constraint, Constraint::And(_)));
        assert!(matches!(intent.hint, Hint::CostBudget(1000)));
    }

    #[test]
    fn test_builder_validation() {
        // Test missing domain
        let result = IntentBuilder::new()
            .input("token", "Token")
            .build();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Domain is required"));
        
        // Test missing name in resource
        let result = ResourceBindingBuilder::new()
            .resource_type("Token")
            .build();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Name is required"));
    }
} 