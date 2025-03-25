// Effect constraint validation
// Original file: src/effect/constraints/validation.rs

//! Validation and orchestration for the effect constraint system
//!
//! This module provides validation and orchestration services for effect
//! constraints, enabling comprehensive validation across different constraint
//! types and orchestration of complex effect sequences.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use causality_types::Address;
use crate::domain::{DomainId, DomainRegistry};
use crate::resource::{ResourceId, Quantity, ResourceRegister};
use causality_resource::CapabilityRepository;
use causality_resource::ResourceAPI;
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    TransferEffect, QueryEffect, StorageEffect
};

/// Validates and orchestrates effects based on their constraints
pub struct EffectValidator {
    // Core services used for validation
    domain_registry: Arc<DomainRegistry>,
    capability_repo: Arc<dyn CapabilityRepository>,
    resource_api: Arc<dyn ResourceAPI>,
}

impl EffectValidator {
    /// Create a new effect validator
    pub fn new(
        domain_registry: Arc<DomainRegistry>,
        capability_repo: Arc<dyn CapabilityRepository>,
        resource_api: Arc<dyn ResourceAPI>,
    ) -> Self {
        Self {
            domain_registry,
            capability_repo,
            resource_api,
        }
    }
    
    /// Validate an effect based on its constraints
    pub async fn validate_effect(&self, effect: &dyn Effect, context: &EffectContext) -> Result<(), EffectError> {
        // First, validate capabilities
        self.validate_capabilities(effect, context).await?;
        
        // Then, try to validate based on specific constraint types
        if let Some(transfer) = self.as_transfer_effect(effect) {
            self.validate_transfer_effect(transfer, context).await?;
        }
        
        if let Some(storage) = self.as_storage_effect(effect) {
            self.validate_storage_effect(storage, context).await?;
        }
        
        if let Some(query) = self.as_query_effect(effect) {
            self.validate_query_effect(query, context).await?;
        }
        
        Ok(())
    }
    
    /// Validate capabilities for an effect
    async fn validate_capabilities(&self, effect: &dyn Effect, context: &EffectContext) -> Result<(), EffectError> {
        let required_capabilities = effect.required_capabilities();
        
        // Skip validation if no capabilities required
        if required_capabilities.is_empty() {
            return Ok(());
        }
        
        // Check if the context has the provided capabilities
        for (resource_id, required_right) in required_capabilities {
            let mut has_capability = false;
            
            for capability_ref in &context.capabilities {
                // Get the capability from the repository
                let capability = self.capability_repo.get_capability(&capability_ref.id)
                    .await
                    .map_err(|e| EffectError::CapabilityError(format!("Failed to retrieve capability: {}", e)))?;
                
                // Check if the capability is for the right resource and has the right
                if capability.resource_id == resource_id && capability.rights.contains(&required_right) {
                    has_capability = true;
                    break;
                }
            }
            
            if !has_capability {
                return Err(EffectError::AuthorizationFailed(
                    format!("Missing capability for resource {} with right {:?}", resource_id, required_right)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Try to cast an effect to a TransferEffect
    fn as_transfer_effect<'a>(&self, effect: &'a dyn Effect) -> Option<&'a dyn TransferEffect> {
        // This is a limitation of the current system - we can't easily downcast
        // We'll need a more robust solution in a production system
        if effect.name().contains("transfer") {
            // This is unsafe and just for demonstration
            #[allow(unused_unsafe)]
            unsafe {
                let ptr = effect as *const dyn Effect as *const dyn TransferEffect;
                Some(&*ptr)
            }
        } else {
            None
        }
    }
    
    /// Try to cast an effect to a StorageEffect
    fn as_storage_effect<'a>(&self, effect: &'a dyn Effect) -> Option<&'a dyn StorageEffect> {
        // This is a limitation of the current system - we can't easily downcast
        if effect.name().contains("store") || effect.name().contains("storage") {
            // This is unsafe and just for demonstration
            #[allow(unused_unsafe)]
            unsafe {
                let ptr = effect as *const dyn Effect as *const dyn StorageEffect;
                Some(&*ptr)
            }
        } else {
            None
        }
    }
    
    /// Try to cast an effect to a QueryEffect
    fn as_query_effect<'a>(&self, effect: &'a dyn Effect) -> Option<&'a dyn QueryEffect> {
        // This is a limitation of the current system - we can't easily downcast
        if effect.name().contains("query") || effect.name().contains("read") {
            // This is unsafe and just for demonstration
            #[allow(unused_unsafe)]
            unsafe {
                let ptr = effect as *const dyn Effect as *const dyn QueryEffect;
                Some(&*ptr)
            }
        } else {
            None
        }
    }
    
    /// Validate a transfer effect
    async fn validate_transfer_effect(&self, effect: &dyn TransferEffect, context: &EffectContext) -> Result<(), EffectError> {
        // Validate basic parameters
        if effect.amount().is_zero() {
            return Err(EffectError::ValidationError("Transfer amount cannot be zero".to_string()));
        }
        
        // Check that source and destination are valid addresses
        if effect.source().is_empty() {
            return Err(EffectError::ValidationError("Source address cannot be empty".to_string()));
        }
        
        if effect.destination().is_empty() {
            return Err(EffectError::ValidationError("Destination address cannot be empty".to_string()));
        }
        
        // Check that domain exists
        let domain_id = effect.domain_id();
        if !self.domain_registry.has_domain(domain_id) {
            return Err(EffectError::ValidationError(format!("Domain {} does not exist", domain_id)));
        }
        
        // Validate token existence
        let token_id = effect.token();
        if let Err(e) = self.resource_api.get_resource(token_id).await {
            return Err(EffectError::ValidationError(format!("Token {} does not exist: {}", token_id, e)));
        }
        
        Ok(())
    }
    
    /// Validate a storage effect
    async fn validate_storage_effect(&self, effect: &dyn StorageEffect, context: &EffectContext) -> Result<(), EffectError> {
        // Validate basic parameters
        if effect.register_id().is_empty() {
            return Err(EffectError::ValidationError("Register ID cannot be empty".to_string()));
        }
        
        if effect.fields().is_empty() {
            return Err(EffectError::ValidationError("Fields cannot be empty".to_string()));
        }
        
        // Check that domain exists
        let domain_id = effect.domain_id();
        if !self.domain_registry.has_domain(domain_id) {
            return Err(EffectError::ValidationError(format!("Domain {} does not exist", domain_id)));
        }
        
        // If this is an update, check that register exists
        if effect.is_update() {
            let register_id = effect.register_id();
            if let Err(e) = self.resource_api.get_resource(register_id).await {
                return Err(EffectError::ValidationError(format!("Register {} does not exist: {}", register_id, e)));
            }
        }
        
        Ok(())
    }
    
    /// Validate a query effect
    async fn validate_query_effect(&self, effect: &dyn QueryEffect, context: &EffectContext) -> Result<(), EffectError> {
        // Validate basic parameters
        if effect.query_type().is_empty() {
            return Err(EffectError::ValidationError("Query type cannot be empty".to_string()));
        }
        
        // Other validations depend on the specific query type
        match effect.query_type() {
            "balance" => {
                // Validate balance query parameters
                let params = effect.parameters();
                if !params.get("address").is_some() {
                    return Err(EffectError::ValidationError("Balance query requires an address parameter".to_string()));
                }
                
                if !params.get("token").is_some() {
                    return Err(EffectError::ValidationError("Balance query requires a token parameter".to_string()));
                }
            },
            "register" => {
                // Validate register query parameters
                let params = effect.parameters();
                if !params.get("register_id").is_some() {
                    return Err(EffectError::ValidationError("Register query requires a register_id parameter".to_string()));
                }
            },
            // Add more query types as needed
            _ => {
                // Unknown query type - let it pass for extensibility
            }
        }
        
        Ok(())
    }
}

/// Orchestrates execution of effects
pub struct EffectOrchestrator {
    validator: EffectValidator,
}

impl EffectOrchestrator {
    /// Create a new effect orchestrator
    pub fn new(validator: EffectValidator) -> Self {
        Self {
            validator,
        }
    }
    
    /// Execute an effect with validation
    pub async fn execute_effect<E: Effect + ?Sized>(&self, effect: &E, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Validate the effect before execution
        self.validator.validate_effect(effect, &context).await?;
        
        // Execute the effect
        effect.execute(context).await
    }
    
    /// Execute a sequence of effects
    pub async fn execute_sequence(&self, effects: Vec<Arc<dyn Effect>>, mut context: EffectContext) -> EffectResult<Vec<EffectOutcome>> {
        let mut outcomes = Vec::with_capacity(effects.len());
        
        for effect in effects {
            // Validate and execute each effect in sequence
            // If any effect fails, stop execution
            let outcome = self.execute_effect(effect.as_ref(), context.clone()).await?;
            
            // Store the outcome
            outcomes.push(outcome);
        }
        
        Ok(outcomes)
    }
    
    /// Execute effects in parallel
    pub async fn execute_parallel(&self, effects: Vec<Arc<dyn Effect>>, context: EffectContext) -> EffectResult<Vec<EffectOutcome>> {
        use futures::future::join_all;
        
        // Create a future for each effect
        let futures = effects.iter().map(|effect| {
            let effect_ref = effect.clone();
            let context_clone = context.clone();
            let self_clone = self.clone();
            
            async move {
                self_clone.execute_effect(effect_ref.as_ref(), context_clone).await
            }
        }).collect::<Vec<_>>();
        
        // Execute all futures in parallel
        let results = join_all(futures).await;
        
        // Collect results
        let mut outcomes = Vec::with_capacity(results.len());
        let mut errors = Vec::new();
        
        for result in results {
            match result {
                Ok(outcome) => outcomes.push(outcome),
                Err(e) => errors.push(e),
            }
        }
        
        // If there were any errors, return the first one
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        
        Ok(outcomes)
    }
    
    /// Execute an effect conditionally
    pub async fn execute_conditional(
        &self,
        condition: Arc<dyn Effect>,
        then_effect: Arc<dyn Effect>,
        else_effect: Option<Arc<dyn Effect>>,
        mut context: EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Execute the condition
        let condition_outcome = self.execute_effect(condition.as_ref(), context.clone()).await?;
        
        // Determine which branch to execute based on the condition outcome
        if condition_outcome.success {
            self.execute_effect(then_effect.as_ref(), context).await
        } else if let Some(else_effect) = else_effect {
            self.execute_effect(else_effect.as_ref(), context).await
        } else {
            // No else effect, return the condition outcome
            Ok(condition_outcome)
        }
    }
}

// Make EffectOrchestrator cloneable for use in parallel execution
impl Clone for EffectOrchestrator {
    fn clone(&self) -> Self {
        // This is a simplification - in a real implementation, we would need to properly
        // clone or use Arc for the validator
        Self {
            validator: EffectValidator::new(
                self.validator.domain_registry.clone(),
                self.validator.capability_repo.clone(),
                self.validator.resource_api.clone(),
            ),
        }
    }
}

//! Constraint validation infrastructure for effects
//!
//! This module provides components to validate effects against constraints
//! and coordinate constraint verification across multiple effects.

/// Result of constraint verification
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintVerificationResult {
    /// The constraint was satisfied
    Satisfied,
    /// The constraint was not satisfied, with reason
    NotSatisfied(String),
    /// Verification was deferred
    Deferred,
}

/// Condition for applying a constraint
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintCondition {
    /// Always apply this constraint
    Always,
    /// Apply only to effects with specified name
    EffectName(String),
    /// Apply based on a custom predicate
    Custom(Arc<dyn Fn(&dyn Effect) -> bool + Send + Sync>),
}

/// Effect constraint definition
pub struct EffectConstraint {
    /// Name of this constraint
    name: String,
    /// Description of this constraint
    description: String,
    /// Condition for applying this constraint
    condition: ConstraintCondition,
    /// Verification function
    verifier: Arc<dyn Fn(&dyn Effect) -> ConstraintVerificationResult + Send + Sync>,
}

impl EffectConstraint {
    /// Create a new constraint
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        condition: ConstraintCondition,
        verifier: impl Fn(&dyn Effect) -> ConstraintVerificationResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            condition,
            verifier: Arc::new(verifier),
        }
    }
    
    /// Get the constraint name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the constraint description
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// Check if this constraint applies to the given effect
    pub fn applies_to(&self, effect: &dyn Effect) -> bool {
        match &self.condition {
            ConstraintCondition::Always => true,
            ConstraintCondition::EffectName(name) => effect.name() == name,
            ConstraintCondition::Custom(predicate) => predicate(effect),
        }
    }
    
    /// Verify if an effect satisfies this constraint
    pub fn verify(&self, effect: &dyn Effect) -> ConstraintVerificationResult {
        if !self.applies_to(effect) {
            return ConstraintVerificationResult::Satisfied;
        }
        (self.verifier)(effect)
    }
}

/// Helper function to create a new constraint
pub fn create_constraint(
    name: impl Into<String>,
    description: impl Into<String>,
    condition: ConstraintCondition,
    verifier: impl Fn(&dyn Effect) -> ConstraintVerificationResult + Send + Sync + 'static,
) -> EffectConstraint {
    EffectConstraint::new(name, description, condition, verifier)
}

/// Constraint verifier for validating effects against constraints
pub struct ConstraintVerifier {
    /// Registered constraints
    constraints: Vec<EffectConstraint>,
}

impl ConstraintVerifier {
    /// Create a new constraint verifier
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }
    
    /// Register a constraint with this verifier
    pub fn register_constraint(&mut self, constraint: EffectConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Register multiple constraints
    pub fn register_constraints(&mut self, constraints: Vec<EffectConstraint>) {
        self.constraints.extend(constraints);
    }
    
    /// Verify if an effect satisfies all applicable constraints
    pub fn verify(&self, effect: &dyn Effect) -> Vec<(String, ConstraintVerificationResult)> {
        self.constraints
            .iter()
            .filter(|constraint| constraint.applies_to(effect))
            .map(|constraint| (constraint.name().to_string(), constraint.verify(effect)))
            .collect()
    }
    
    /// Verify an effect and return whether all constraints are satisfied
    pub fn verify_all_satisfied(&self, effect: &dyn Effect) -> Result<(), String> {
        let results = self.verify(effect);
        
        let failures: Vec<(String, String)> = results
            .into_iter()
            .filter_map(|(name, result)| {
                if let ConstraintVerificationResult::NotSatisfied(reason) = result {
                    Some((name, reason))
                } else {
                    None
                }
            })
            .collect();
        
        if failures.is_empty() {
            Ok(())
        } else {
            let error_message = failures
                .into_iter()
                .map(|(name, reason)| format!("{}: {}", name, reason))
                .collect::<Vec<_>>()
                .join("; ");
            
            Err(format!("Constraint verification failed: {}", error_message))
        }
    }
    
    /// Execute an effect after verifying its constraints
    pub async fn execute_effect<E: Effect + ?Sized>(&self, effect: &E, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Verify constraints first
        if let Err(reason) = self.verify_all_satisfied(effect) {
            return Err(EffectError::ValidationError(reason));
        }
        
        // If all constraints are satisfied, execute the effect
        effect.execute_async(context).await
    }
    
    /// Execute a sequence of effects, verifying constraints for each
    pub async fn execute_sequence(&self, effects: Vec<Arc<dyn Effect>>, context: &EffectContext) -> EffectResult<Vec<EffectOutcome>> {
        let mut results = Vec::with_capacity(effects.len());
        
        for effect in effects {
            // Execute with constraints
            let outcome = self.execute_effect(effect.as_ref(), context).await?;
            results.push(outcome);
        }
        
        Ok(results)
    }
    
    /// Execute effects in parallel, verifying constraints for each
    pub async fn execute_parallel(&self, effects: Vec<Arc<dyn Effect>>, context: &EffectContext) -> EffectResult<Vec<EffectOutcome>> {
        use futures::future::join_all;
        
        // Create futures for all effects
        let futures = effects
            .iter()
            .map(|effect| {
                let cloned_context = context.clone();
                let effect_ref = effect.clone();
                let verifier = self.clone();
                
                async move {
                    verifier.execute_effect(effect_ref.as_ref(), &cloned_context).await
                }
            })
            .collect::<Vec<_>>();
        
        // Execute all futures in parallel
        let results = join_all(futures).await;
        
        // Collect results, propagating any errors
        let mut outcomes = Vec::with_capacity(results.len());
        for result in results {
            outcomes.push(result?);
        }
        
        Ok(outcomes)
    }
}

impl Clone for ConstraintVerifier {
    fn clone(&self) -> Self {
        Self {
            constraints: self.constraints.clone(),
        }
    }
}

impl Default for ConstraintVerifier {
    fn default() -> Self {
        Self::new()
    }
} 
