// Domain coordination utilities
// Original file: src/domain_adapters/coordination.rs

//! Cross-VM Effect Coordination Module
//!
//! This module provides a framework for coordinating effects across multiple VM adapters,
//! supporting complex multi-chain transactions and compositional effect patterns.

use std::{collections::HashMap, sync::{Arc, Mutex}};
use causality_types::{Error, Result};
use super::{
    interfaces::{VmType, VmAdapter, CrossVmAdapter},
    utils::{CrossVmBroker, CrossVmRequest, CrossVmResponse},
    validation::{ValidationContext, ValidationResult, EffectValidator, EffectValidatorRegistry},
    DomainId,
};

/// Effect coordination status
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinationStatus {
    /// Initial state
    Pending,
    /// Validation in progress
    Validating,
    /// Execution in progress
    Executing,
    /// Committing changes
    Committing,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed(String),
}

/// Cross-VM effect coordination context
#[derive(Debug, Clone)]
pub struct CoordinationContext {
    /// Unique coordination ID
    pub id: String,
    /// Current status
    pub status: CoordinationStatus,
    /// Domains involved in coordination
    pub domains: Vec<DomainId>,
    /// VM types involved in coordination
    pub vm_types: Vec<VmType>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CoordinationContext {
    /// Create a new coordination context
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: CoordinationStatus::Pending,
            domains: Vec::new(),
            vm_types: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a domain to the coordination context
    pub fn add_domain(&mut self, domain: DomainId) -> &mut Self {
        if !self.domains.contains(&domain) {
            self.domains.push(domain);
        }
        self
    }

    /// Add a VM type to the coordination context
    pub fn add_vm_type(&mut self, vm_type: VmType) -> &mut Self {
        if !self.vm_types.contains(&vm_type) {
            self.vm_types.push(vm_type);
        }
        self
    }

    /// Add metadata to the coordination context
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Update status of the coordination context
    pub fn update_status(&mut self, status: CoordinationStatus) -> &mut Self {
        self.status = status;
        self
    }
}

/// Definition of a coordination step
#[derive(Debug, Clone)]
pub struct CoordinationStep {
    /// Step identifier
    pub id: String,
    /// Target domain
    pub domain: DomainId,
    /// Target VM type
    pub vm_type: VmType,
    /// Operation to execute
    pub operation: String,
    /// Parameters for the operation
    pub params: HashMap<String, serde_json::Value>,
    /// Dependencies (IDs of steps that must complete before this one)
    pub dependencies: Vec<String>,
    /// Execution status
    pub status: CoordinationStatus,
    /// Result data
    pub result: Option<serde_json::Value>,
}

impl CoordinationStep {
    /// Create a new coordination step
    pub fn new(
        id: impl Into<String>,
        domain: DomainId,
        vm_type: VmType,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            domain,
            vm_type,
            operation: operation.into(),
            params: HashMap::new(),
            dependencies: Vec::new(),
            status: CoordinationStatus::Pending,
            result: None,
        }
    }

    /// Add a parameter to the step
    pub fn add_param(
        &mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> &mut Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Add a dependency to the step
    pub fn add_dependency(&mut self, step_id: impl Into<String>) -> &mut Self {
        let dep = step_id.into();
        if !self.dependencies.contains(&dep) {
            self.dependencies.push(dep);
        }
        self
    }

    /// Update the status of the step
    pub fn update_status(&mut self, status: CoordinationStatus) -> &mut Self {
        self.status = status;
        self
    }

    /// Set the result of the step
    pub fn set_result(&mut self, result: serde_json::Value) -> &mut Self {
        self.result = Some(result);
        self
    }
}

/// Execution plan for a coordinated multi-VM operation
#[derive(Debug, Clone)]
pub struct CoordinationPlan {
    /// Coordination context
    pub context: CoordinationContext,
    /// Steps to execute
    pub steps: Vec<CoordinationStep>,
}

impl CoordinationPlan {
    /// Create a new coordination plan
    pub fn new(context: CoordinationContext) -> Self {
        Self {
            context,
            steps: Vec::new(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: CoordinationStep) -> &mut Self {
        // Update context with domain and VM type
        self.context.add_domain(step.domain.clone());
        self.context.add_vm_type(step.vm_type.clone());
        
        // Add the step
        self.steps.push(step);
        self
    }

    /// Get a step by ID
    pub fn get_step(&self, id: &str) -> Option<&CoordinationStep> {
        self.steps.iter().find(|step| step.id == id)
    }

    /// Get a mutable reference to a step by ID
    pub fn get_step_mut(&mut self, id: &str) -> Option<&mut CoordinationStep> {
        self.steps.iter_mut().find(|step| step.id == id)
    }

    /// Get the next steps that are ready to execute
    pub fn get_next_steps(&self) -> Vec<&CoordinationStep> {
        self.steps
            .iter()
            .filter(|step| {
                // Step must be pending
                if step.status != CoordinationStatus::Pending {
                    return false;
                }
                
                // All dependencies must be completed
                step.dependencies.iter().all(|dep_id| {
                    if let Some(dep) = self.get_step(dep_id) {
                        dep.status == CoordinationStatus::Completed
                    } else {
                        false
                    }
                })
            })
            .collect()
    }

    /// Check if the plan is complete
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|step| {
            matches!(
                step.status,
                CoordinationStatus::Completed | CoordinationStatus::Failed(_)
            )
        })
    }

    /// Check if the plan has failed
    pub fn has_failed(&self) -> bool {
        self.steps.iter().any(|step| {
            matches!(step.status, CoordinationStatus::Failed(_))
        })
    }
}

/// Handler trait for coordination operations
pub trait CoordinationHandler: Send + Sync {
    /// Get the name of the handler
    fn name(&self) -> &str;
    
    /// Check if the handler supports a given operation
    fn supports_operation(&self, operation: &str) -> bool;
    
    /// Execute an operation
    fn execute_operation(
        &self,
        step: &CoordinationStep,
        broker: &CrossVmBroker,
    ) -> Result<serde_json::Value>;
}

/// Executor for coordination plans
pub struct CoordinationExecutor {
    /// Cross-VM broker for executing operations
    broker: Arc<CrossVmBroker>,
    /// Validation registry
    validator_registry: Arc<EffectValidatorRegistry>,
    /// Handlers for operations
    handlers: HashMap<String, Box<dyn CoordinationHandler>>,
}

impl CoordinationExecutor {
    /// Create a new coordination executor
    pub fn new(
        broker: Arc<CrossVmBroker>,
        validator_registry: Arc<EffectValidatorRegistry>,
    ) -> Self {
        Self {
            broker,
            validator_registry,
            handlers: HashMap::new(),
        }
    }

    /// Register a handler
    pub fn register_handler(&mut self, handler: Box<dyn CoordinationHandler>) -> &mut Self {
        self.handlers.insert(handler.name().to_string(), handler);
        self
    }

    /// Validate a coordination plan
    pub fn validate_plan(&self, plan: &CoordinationPlan) -> ValidationResult {
        let mut result = ValidationResult::valid();
        
        // Validate each step
        for step in &plan.steps {
            // Get a validator for the VM type and operation
            if let Some(validator) = self.validator_registry.get_validator_for_effect(
                &step.vm_type,
                &step.operation,
            ) {
                // Create validation context
                let mut context = ValidationContext::new(
                    step.domain.clone(),
                    step.vm_type.clone(),
                    step.operation.clone(),
                );
                
                // Add parameters
                for (key, value) in &step.params {
                    context.add_param(key.clone(), value.clone());
                }
                
                // Validate the effect
                let step_result = validator.validate_effect(&context);
                result.merge(step_result);
            }
        }
        
        // Validate dependencies
        for step in &plan.steps {
            for dep_id in &step.dependencies {
                if plan.get_step(dep_id).is_none() {
                    result.add_error(
                        "dependencies",
                        format!("Dependency '{}' not found in plan", dep_id),
                        "INVALID_DEPENDENCY",
                    );
                }
            }
        }
        
        // Validate for circular dependencies
        if Self::has_circular_dependencies(plan) {
            result.add_error(
                "dependencies",
                "Plan contains circular dependencies",
                "CIRCULAR_DEPENDENCY",
            );
        }
        
        result
    }
    
    /// Check if a plan has circular dependencies
    fn has_circular_dependencies(plan: &CoordinationPlan) -> bool {
        // Build adjacency list
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for step in &plan.steps {
            adjacency.insert(step.id.clone(), step.dependencies.clone());
        }
        
        // Check for cycles using DFS
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();
        
        for step in &plan.steps {
            if Self::is_cyclic_util(
                &step.id,
                &adjacency,
                &mut visited,
                &mut rec_stack,
            ) {
                return true;
            }
        }
        
        false
    }
    
    /// Utility function for cycle detection
    fn is_cyclic_util(
        node: &str,
        adjacency: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> bool {
        // Mark current node as visited and add to recursion stack
        visited.insert(node.to_string(), true);
        rec_stack.insert(node.to_string(), true);
        
        // Check all neighbors
        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                // If neighbor not visited, check recursively
                if !visited.get(neighbor).unwrap_or(&false) {
                    if Self::is_cyclic_util(neighbor, adjacency, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.get(neighbor).unwrap_or(&false) {
                    // If neighbor is in recursion stack, there's a cycle
                    return true;
                }
            }
        }
        
        // Remove from recursion stack
        rec_stack.insert(node.to_string(), false);
        false
    }
    
    /// Execute a coordination plan
    pub fn execute_plan(&self, mut plan: CoordinationPlan) -> Result<CoordinationPlan> {
        // Validate the plan
        let validation = self.validate_plan(&plan);
        if !validation.is_valid() {
            return Err(Error::Validation(format!(
                "Invalid coordination plan: {:?}",
                validation.errors()
            )));
        }
        
        // Update context status
        plan.context.update_status(CoordinationStatus::Executing);
        
        // Execute steps until completion or failure
        while !plan.is_complete() {
            let next_steps = plan.get_next_steps();
            if next_steps.is_empty() {
                if !plan.is_complete() {
                    return Err(Error::Validation(
                        "Coordination plan stalled with no executable steps".to_string()
                    ));
                }
                break;
            }
            
            // Execute each ready step
            for &step in next_steps {
                let step_id = step.id.clone();
                if let Some(step_mut) = plan.get_step_mut(&step_id) {
                    // Find a handler for this operation
                    let handler = self.handlers.values().find(|h| h.supports_operation(&step.operation));
                    
                    match handler {
                        Some(handler) => {
                            // Mark step as executing
                            step_mut.update_status(CoordinationStatus::Executing);
                            
                            // Execute the operation
                            match handler.execute_operation(step, &self.broker) {
                                Ok(result) => {
                                    step_mut.set_result(result);
                                    step_mut.update_status(CoordinationStatus::Completed);
                                }
                                Err(e) => {
                                    step_mut.update_status(CoordinationStatus::Failed(e.to_string()));
                                    // Update plan context to failed
                                    plan.context.update_status(CoordinationStatus::Failed(e.to_string()));
                                    return Ok(plan);
                                }
                            }
                        }
                        None => {
                            // No handler found for this operation
                            step_mut.update_status(CoordinationStatus::Failed(format!(
                                "No handler found for operation '{}'", step.operation
                            )));
                            plan.context.update_status(CoordinationStatus::Failed(format!(
                                "No handler found for operation '{}'", step.operation
                            )));
                            return Ok(plan);
                        }
                    }
                }
            }
        }
        
        // Check if any steps failed
        if plan.has_failed() {
            plan.context.update_status(CoordinationStatus::Failed(
                "One or more steps failed".to_string()
            ));
        } else {
            plan.context.update_status(CoordinationStatus::Completed);
        }
        
        Ok(plan)
    }
}

/// Built-in coordination handler for cross-VM proof verification
pub struct ProofVerificationHandler {
    name: String,
}

impl ProofVerificationHandler {
    /// Create a new proof verification handler
    pub fn new() -> Self {
        Self {
            name: "proof_verification".to_string(),
        }
    }
}

impl CoordinationHandler for ProofVerificationHandler {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn supports_operation(&self, operation: &str) -> bool {
        operation == "verify_proof" || operation == "cross_verify_proof"
    }
    
    fn execute_operation(
        &self,
        step: &CoordinationStep,
        broker: &CrossVmBroker,
    ) -> Result<serde_json::Value> {
        match step.operation.as_str() {
            "verify_proof" => {
                // Get required parameters
                let proof = step.params.get("proof")
                    .ok_or_else(|| Error::Validation("Missing 'proof' parameter".to_string()))?;
                let program = step.params.get("program")
                    .ok_or_else(|| Error::Validation("Missing 'program' parameter".to_string()))?;
                
                // Create cross-VM request
                let request = CrossVmRequest::new(
                    "verify_proof",
                    step.domain.clone(),
                    step.vm_type.clone(),
                );
                
                // Execute the request
                let response = broker.execute(request)?;
                
                // Return the result
                Ok(response.result)
            }
            "cross_verify_proof" => {
                // Get required parameters
                let proof = step.params.get("proof")
                    .ok_or_else(|| Error::Validation("Missing 'proof' parameter".to_string()))?;
                let source_vm = step.params.get("source_vm")
                    .ok_or_else(|| Error::Validation("Missing 'source_vm' parameter".to_string()))?;
                let target_vm = step.params.get("target_vm")
                    .ok_or_else(|| Error::Validation("Missing 'target_vm' parameter".to_string()))?;
                
                // Create cross-VM request
                let request = CrossVmRequest::new(
                    "cross_verify_proof",
                    step.domain.clone(),
                    step.vm_type.clone(),
                );
                
                // Execute the request
                let response = broker.execute(request)?;
                
                // Return the result
                Ok(response.result)
            }
            _ => Err(Error::Validation(format!(
                "Unsupported operation: {}", step.operation
            ))),
        }
    }
}

/// Factory for creating coordination plans based on templates
pub struct CoordinationPlanFactory {
    /// Templates for coordination plans
    templates: HashMap<String, CoordinationPlan>,
}

impl CoordinationPlanFactory {
    /// Create a new coordination plan factory
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
    
    /// Register a template
    pub fn register_template(&mut self, name: impl Into<String>, template: CoordinationPlan) -> &mut Self {
        self.templates.insert(name.into(), template);
        self
    }
    
    /// Create a plan from a template
    pub fn create_from_template(
        &self,
        template_name: &str,
        context_id: impl Into<String>,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<CoordinationPlan> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| Error::NotFound(format!("Template not found: {}", template_name)))?;
        
        // Create a new context based on the template
        let mut context = CoordinationContext::new(context_id);
        context.domains = template.context.domains.clone();
        context.vm_types = template.context.vm_types.clone();
        
        // Create a new plan
        let mut plan = CoordinationPlan::new(context);
        
        // Clone and customize each step
        for template_step in &template.steps {
            let mut step = template_step.clone();
            
            // Apply parameters
            for (key, value) in &params {
                // Replace parameters in the step
                if let Some(step_param) = step.params.get_mut(key) {
                    *step_param = value.clone();
                }
            }
            
            plan.add_step(step);
        }
        
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordination_context() {
        let mut context = CoordinationContext::new("test");
        context.add_domain(DomainId::new("evm"));
        context.add_vm_type(VmType::Evm);
        context.add_metadata("key", "value");
        
        assert_eq!(context.id, "test");
        assert_eq!(context.domains.len(), 1);
        assert_eq!(context.vm_types.len(), 1);
        assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_coordination_step() {
        let mut step = CoordinationStep::new(
            "step1",
            DomainId::new("evm"),
            VmType::Evm,
            "verify_proof",
        );
        step.add_param("proof", serde_json::json!("proof_data"));
        step.add_dependency("step0");
        
        assert_eq!(step.id, "step1");
        assert_eq!(step.domain, DomainId::new("evm"));
        assert_eq!(step.operation, "verify_proof");
        assert!(step.params.contains_key("proof"));
        assert_eq!(step.dependencies.len(), 1);
    }
    
    #[test]
    fn test_coordination_plan() {
        let context = CoordinationContext::new("test_plan");
        let mut plan = CoordinationPlan::new(context);
        
        let step1 = CoordinationStep::new(
            "step1",
            DomainId::new("evm"),
            VmType::Evm,
            "verify_proof",
        );
        
        let mut step2 = CoordinationStep::new(
            "step2",
            DomainId::new("cosmos"),
            VmType::CosmWasm,
            "verify_proof",
        );
        step2.add_dependency("step1");
        
        plan.add_step(step1);
        plan.add_step(step2);
        
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.get_next_steps().len(), 1);
        assert_eq!(plan.get_next_steps()[0].id, "step1");
    }
} 