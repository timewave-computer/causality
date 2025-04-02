use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use std::fmt::Debug;

use super::{
    EffectError, EffectId, EffectOutcome, EffectResult, 
    context::EffectContext,
    registry::EffectRegistry,
};

use crate::effect::{
    domain::{DomainEffect, DomainId, DomainEffectOutcome, EnhancedDomainContextAdapter},
    resource::ResourceEffect,
};

/// Orchestrated effect execution status
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrchestrationStatus {
    /// Pending execution
    Pending,
    /// In progress
    InProgress,
    /// Completed successfully
    Completed,
    /// Failed execution
    Failed,
    /// Cancelled
    Cancelled,
}

/// Orchestration reference for tracking effects
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrchestrationRef {
    /// Unique identifier for this orchestration
    pub id: String,
    /// Current status
    pub status: OrchestrationStatus,
    /// Primary domain ID
    pub primary_domain: DomainId,
    /// Any secondary domains involved
    pub secondary_domains: Vec<DomainId>,
}

impl OrchestrationRef {
    /// Create a new orchestration reference
    pub fn new(id: String, primary_domain: DomainId) -> Self {
        Self {
            id,
            status: OrchestrationStatus::Pending,
            primary_domain,
            secondary_domains: vec![],
        }
    }
    
    /// Update the status
    pub fn with_status(mut self, status: OrchestrationStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Add a secondary domain
    pub fn with_secondary_domain(mut self, domain: DomainId) -> Self {
        self.secondary_domains.push(domain);
        self
    }
}

/// Step in an orchestrated execution
#[derive(Debug, Clone)]
pub struct OrchestrationStep {
    /// Effect to execute in this step
    pub effect: EffectId,
    /// Domain for this step
    pub domain: DomainId,
    /// Whether this step depends on previous steps
    pub has_dependencies: bool,
    /// Status of this step
    pub status: OrchestrationStatus,
    /// Result of this step if completed
    pub result: Option<EffectOutcome>,
}

impl OrchestrationStep {
    /// Create a new orchestration step
    pub fn new(effect: EffectId, domain: DomainId) -> Self {
        Self {
            effect,
            domain,
            has_dependencies: false,
            status: OrchestrationStatus::Pending,
            result: None,
        }
    }
    
    /// Set dependency status
    pub fn with_dependencies(mut self, has_dependencies: bool) -> Self {
        self.has_dependencies = has_dependencies;
        self
    }
    
    /// Update the status
    pub fn with_status(mut self, status: OrchestrationStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set the result
    pub fn with_result(mut self, result: EffectOutcome) -> Self {
        self.result = Some(result);
        self.status = match &result {
            EffectOutcome::Success(_) => OrchestrationStatus::Completed,
            EffectOutcome::Error(_) => OrchestrationStatus::Failed,
        };
        self
    }
}

/// Orchestration plan containing steps and dependencies
#[derive(Debug, Clone)]
pub struct OrchestrationPlan {
    /// Reference to this orchestration
    pub reference: OrchestrationRef,
    /// Steps in this orchestration
    pub steps: Vec<OrchestrationStep>,
    /// Step dependencies (step_index -> dependent_step_indices)
    pub dependencies: HashMap<usize, Vec<usize>>,
    /// Overall status
    pub status: OrchestrationStatus,
}

impl OrchestrationPlan {
    /// Create a new orchestration plan
    pub fn new(reference: OrchestrationRef) -> Self {
        Self {
            reference,
            steps: vec![],
            dependencies: HashMap::new(),
            status: OrchestrationStatus::Pending,
        }
    }
    
    /// Add a step to the plan
    pub fn add_step(&mut self, step: OrchestrationStep) -> usize {
        let index = self.steps.len();
        self.steps.push(step);
        index
    }
    
    /// Add a dependency between steps
    pub fn add_dependency(&mut self, from_step: usize, to_step: usize) -> EffectResult<()> {
        if from_step >= self.steps.len() || to_step >= self.steps.len() {
            return Err(EffectError::ValidationError(
                format!("Invalid step indices: {} -> {}", from_step, to_step)
            ));
        }
        
        let deps = self.dependencies.entry(from_step).or_insert_with(Vec::new);
        deps.push(to_step);
        
        // Mark the dependent step as having dependencies
        self.steps[to_step].has_dependencies = true;
        
        Ok(())
    }
    
    /// Get all steps that are ready to execute (no pending dependencies)
    pub fn get_ready_steps(&self) -> Vec<usize> {
        self.steps
            .iter()
            .enumerate()
            .filter(|(i, step)| {
                step.status == OrchestrationStatus::Pending && (!step.has_dependencies || 
                    self.are_dependencies_completed(*i))
            })
            .map(|(i, _)| i)
            .collect()
    }
    
    /// Check if all dependencies for a step are completed
    pub fn are_dependencies_completed(&self, step_index: usize) -> bool {
        for (from_step, deps) in &self.dependencies {
            if deps.contains(&step_index) {
                if self.steps[*from_step].status != OrchestrationStatus::Completed {
                    return false;
                }
            }
        }
        true
    }
    
    /// Update the status of a step
    pub fn update_step_status(&mut self, step_index: usize, status: OrchestrationStatus) -> EffectResult<()> {
        if step_index >= self.steps.len() {
            return Err(EffectError::ValidationError(
                format!("Invalid step index: {}", step_index)
            ));
        }
        
        self.steps[step_index].status = status.clone();
        
        // Update orchestration status if needed
        self.update_orchestration_status();
        
        Ok(())
    }
    
    /// Update the result of a step
    pub fn update_step_result(&mut self, step_index: usize, result: EffectOutcome) -> EffectResult<()> {
        if step_index >= self.steps.len() {
            return Err(EffectError::ValidationError(
                format!("Invalid step index: {}", step_index)
            ));
        }
        
        self.steps[step_index].result = Some(result.clone());
        self.steps[step_index].status = match result {
            EffectOutcome::Success(_) => OrchestrationStatus::Completed,
            EffectOutcome::Error(_) => OrchestrationStatus::Failed,
        };
        
        // Update orchestration status if needed
        self.update_orchestration_status();
        
        Ok(())
    }
    
    /// Update the overall orchestration status based on steps
    pub fn update_orchestration_status(&mut self) {
        // If any step failed, the orchestration failed
        if self.steps.iter().any(|s| s.status == OrchestrationStatus::Failed) {
            self.status = OrchestrationStatus::Failed;
            return;
        }
        
        // If all steps completed, the orchestration completed
        if self.steps.iter().all(|s| s.status == OrchestrationStatus::Completed) {
            self.status = OrchestrationStatus::Completed;
            return;
        }
        
        // If any step is in progress, the orchestration is in progress
        if self.steps.iter().any(|s| s.status == OrchestrationStatus::InProgress) {
            self.status = OrchestrationStatus::InProgress;
            return;
        }
        
        // Default to pending if none of the above
        self.status = OrchestrationStatus::Pending;
    }
    
    /// Check if the orchestration is complete
    pub fn is_complete(&self) -> bool {
        self.status == OrchestrationStatus::Completed || self.status == OrchestrationStatus::Failed
    }
}

/// Orchestration builder for creating orchestration plans
#[derive(Debug)]
pub struct OrchestrationBuilder {
    /// Current orchestration plan
    plan: OrchestrationPlan,
    /// Context for orchestration
    context: Box<dyn EffectContext>,
    /// Domain context adapters
    domain_adapters: HashMap<DomainId, Arc<EnhancedDomainContextAdapter>>,
}

impl OrchestrationBuilder {
    /// Create a new orchestration builder
    pub fn new(
        id: String, 
        primary_domain: DomainId, 
        context: Box<dyn EffectContext>
    ) -> Self {
        let reference = OrchestrationRef::new(id, primary_domain);
        Self {
            plan: OrchestrationPlan::new(reference),
            context,
            domain_adapters: HashMap::new(),
        }
    }
    
    /// Add a domain context adapter
    pub fn with_domain_adapter(
        mut self, 
        domain: DomainId, 
        adapter: Arc<EnhancedDomainContextAdapter>
    ) -> Self {
        self.domain_adapters.insert(domain, adapter);
        self
    }
    
    /// Add an effect to the orchestration
    pub fn add_effect(
        &mut self, 
        effect_id: EffectId, 
        domain: DomainId
    ) -> EffectResult<usize> {
        let step = OrchestrationStep::new(effect_id, domain.clone());
        
        // Register secondary domain if not primary
        if domain != self.plan.reference.primary_domain 
            && !self.plan.reference.secondary_domains.contains(&domain) {
            self.plan.reference.secondary_domains.push(domain);
        }
        
        Ok(self.plan.add_step(step))
    }
    
    /// Add a dependency between effects
    pub fn add_dependency(
        &mut self, 
        from_step: usize, 
        to_step: usize
    ) -> EffectResult<()> {
        self.plan.add_dependency(from_step, to_step)
    }
    
    /// Build the orchestration plan
    pub fn build(self) -> OrchestrationPlan {
        self.plan
    }
}

/// Interface for effect orchestrator
#[async_trait]
pub trait EffectOrchestrator: Send + Sync {
    /// Execute an orchestration plan
    async fn execute_orchestration(
        &self,
        plan: &mut OrchestrationPlan,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Execute a single step in an orchestration
    async fn execute_step(
        &self,
        plan: &mut OrchestrationPlan,
        step_index: usize,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Get a list of all active orchestrations
    async fn get_active_orchestrations(&self) -> EffectResult<Vec<OrchestrationRef>>;
    
    /// Get an orchestration plan by id
    async fn get_orchestration(
        &self,
        id: &str,
    ) -> EffectResult<OrchestrationPlan>;
    
    /// Cancel an orchestration
    async fn cancel_orchestration(
        &self,
        id: &str,
    ) -> EffectResult<()>;
}

/// Basic implementation of the effect orchestrator
pub struct BasicEffectOrchestrator {
    /// Effect registry for handling effects
    registry: Arc<dyn EffectRegistry>,
    /// Domain adapters for context adaptation
    domain_adapters: HashMap<DomainId, Arc<EnhancedDomainContextAdapter>>,
    /// Active orchestrations
    orchestrations: HashMap<String, OrchestrationPlan>,
}

impl BasicEffectOrchestrator {
    /// Create a new basic effect orchestrator
    pub fn new(registry: Arc<dyn EffectRegistry>) -> Self {
        Self {
            registry,
            domain_adapters: HashMap::new(),
            orchestrations: HashMap::new(),
        }
    }
    
    /// Add a domain adapter
    pub fn with_domain_adapter(
        mut self,
        domain: DomainId,
        adapter: Arc<EnhancedDomainContextAdapter>,
    ) -> Self {
        self.domain_adapters.insert(domain, adapter);
        self
    }
    
    /// Create a domain-specific context
    fn create_domain_context(
        &self,
        domain: &DomainId,
        context: &dyn EffectContext,
    ) -> EffectResult<Box<dyn EffectContext>> {
        if let Some(adapter) = self.domain_adapters.get(domain) {
            // Clone context and add domain metadata
            let mut metadata = HashMap::new();
            metadata.insert("domain_id".to_string(), domain.to_string());
            
            let domain_context = context.with_additional_metadata(metadata);
            Ok(domain_context)
        } else {
            // If no adapter, just use the original context
            let context_box: Box<dyn EffectContext> = Box::new(context.clone_context());
            Ok(context_box)
        }
    }
    
    /// Store orchestration plan
    fn store_orchestration(&mut self, plan: OrchestrationPlan) {
        self.orchestrations.insert(plan.reference.id.clone(), plan);
    }
}

#[async_trait]
impl EffectOrchestrator for BasicEffectOrchestrator {
    async fn execute_orchestration(
        &self,
        plan: &mut OrchestrationPlan,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Update the orchestration status
        plan.status = OrchestrationStatus::InProgress;
        
        // Execute until completion or failure
        while !plan.is_complete() {
            // Get all ready steps
            let ready_steps = plan.get_ready_steps();
            
            // If no steps are ready, but orchestration is not complete, there's a deadlock
            if ready_steps.is_empty() && !plan.is_complete() {
                return Err(EffectError::ExecutionError(
                    "Orchestration deadlock: no ready steps but orchestration is not complete".to_string()
                ));
            }
            
            // Execute each ready step
            for step_index in ready_steps {
                let result = self.execute_step(plan, step_index, context).await?;
                plan.update_step_result(step_index, result)?;
                
                // If the step failed and it's not a continuation step, fail the orchestration
                if let EffectOutcome::Error(_) = plan.steps[step_index].result.as_ref().unwrap() {
                    plan.status = OrchestrationStatus::Failed;
                    
                    return Err(EffectError::ExecutionError(
                        format!("Orchestration failed at step {}", step_index)
                    ));
                }
            }
        }
        
        // If we get here, all steps completed successfully
        let mut result_data = HashMap::new();
        result_data.insert("orchestration_id".to_string(), plan.reference.id.clone());
        result_data.insert("status".to_string(), format!("{:?}", plan.status));
        
        // Collect all step results
        for (i, step) in plan.steps.iter().enumerate() {
            if let Some(EffectOutcome::Success(data)) = &step.result {
                for (key, value) in data.as_ref() {
                    result_data.insert(format!("step_{}.{}", i, key), value.clone());
                }
            }
        }
        
        Ok(EffectOutcome::Success(Box::new(result_data)))
    }
    
    async fn execute_step(
        &self,
        plan: &mut OrchestrationPlan,
        step_index: usize,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let step = &mut plan.steps[step_index];
        
        // Update the step status
        step.status = OrchestrationStatus::InProgress;
        
        // Create domain context
        let domain_context = self.create_domain_context(&step.domain, context)?;
        
        // Execute the effect
        match self.registry.get_effect(&step.effect).await {
            Ok(effect) => {
                // Execute the effect with domain context
                let result = self.registry.execute_effect(effect.as_ref(), &domain_context).await?;
                Ok(result)
            }
            Err(err) => {
                // Effect not found or couldn't be loaded
                Err(EffectError::ExecutionError(
                    format!("Failed to load effect {}: {}", step.effect, err)
                ))
            }
        }
    }
    
    async fn get_active_orchestrations(&self) -> EffectResult<Vec<OrchestrationRef>> {
        let active = self.orchestrations
            .values()
            .filter(|p| p.status == OrchestrationStatus::Pending || 
                   p.status == OrchestrationStatus::InProgress)
            .map(|p| p.reference.clone())
            .collect();
        
        Ok(active)
    }
    
    async fn get_orchestration(
        &self,
        id: &str,
    ) -> EffectResult<OrchestrationPlan> {
        match self.orchestrations.get(id) {
            Some(plan) => Ok(plan.clone()),
            None => Err(EffectError::NotFound(format!("Orchestration not found: {}", id))),
        }
    }
    
    async fn cancel_orchestration(
        &self,
        id: &str,
    ) -> EffectResult<()> {
        match self.orchestrations.get(id) {
            Some(_) => {
                // Get a mutable reference and update status
                let mut orchestrations = self.orchestrations.clone();
                let plan = orchestrations.get_mut(id).unwrap();
                plan.status = OrchestrationStatus::Cancelled;
                
                Ok(())
            }
            None => Err(EffectError::NotFound(format!("Orchestration not found: {}", id))),
        }
    }
}

/// Trait for creating orchestration builders
pub trait OrchestrationFactory: Send + Sync {
    /// Create a new orchestration builder
    fn create_builder(
        &self,
        id: String,
        primary_domain: DomainId,
        context: Box<dyn EffectContext>,
    ) -> OrchestrationBuilder;
    
    /// Create a cross-domain orchestration builder
    fn create_cross_domain_builder(
        &self,
        id: String,
        domains: Vec<DomainId>,
        context: Box<dyn EffectContext>,
    ) -> EffectResult<OrchestrationBuilder>;
}

/// Basic implementation of the orchestration factory
pub struct BasicOrchestrationFactory {
    /// Domain adapters for context adaptation
    domain_adapters: HashMap<DomainId, Arc<EnhancedDomainContextAdapter>>,
}

impl BasicOrchestrationFactory {
    /// Create a new basic orchestration factory
    pub fn new() -> Self {
        Self {
            domain_adapters: HashMap::new(),
        }
    }
    
    /// Add a domain adapter
    pub fn with_domain_adapter(
        mut self,
        domain: DomainId,
        adapter: Arc<EnhancedDomainContextAdapter>,
    ) -> Self {
        self.domain_adapters.insert(domain, adapter);
        self
    }
}

impl OrchestrationFactory for BasicOrchestrationFactory {
    fn create_builder(
        &self,
        id: String,
        primary_domain: DomainId,
        context: Box<dyn EffectContext>,
    ) -> OrchestrationBuilder {
        let mut builder = OrchestrationBuilder::new(id, primary_domain.clone(), context);
        
        // Add domain adapters
        for (domain, adapter) in &self.domain_adapters {
            builder = builder.with_domain_adapter(domain.clone(), adapter.clone());
        }
        
        builder
    }
    
    fn create_cross_domain_builder(
        &self,
        id: String,
        domains: Vec<DomainId>,
        context: Box<dyn EffectContext>,
    ) -> EffectResult<OrchestrationBuilder> {
        if domains.is_empty() {
            return Err(EffectError::ValidationError(
                "Cross-domain orchestration requires at least one domain".to_string()
            ));
        }
        
        // Use the first domain as primary
        let primary_domain = domains[0].clone();
        let mut builder = self.create_builder(id, primary_domain, context);
        
        // Register all other domains as secondary
        for domain in domains.iter().skip(1) {
            builder.plan.reference.secondary_domains.push(domain.clone());
        }
        
        Ok(builder)
    }
} 