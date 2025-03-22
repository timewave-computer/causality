// Factory functions for creating effects
//
// This module provides factory functions for creating common effect types.

use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;

use crate::effect::{Effect, EffectId, EffectContext, EffectOutcome, EffectResult};
use crate::effect::boundary::ExecutionBoundary;
use crate::error::Result;
use crate::types::{ResourceId, DomainId};
use crate::log::fact_snapshot::{FactSnapshot, FactId, FactDependency, FactDependencyType};

/// An effect for depositing tokens
#[derive(Debug, Clone)]
pub struct DepositEffect {
    /// The unique ID of this effect
    pub id: crate::effect::EffectId,
    /// The resource to deposit
    pub resource_id: ResourceId,
    /// The domain to deposit to
    pub domain_id: DomainId,
    /// The amount to deposit
    pub amount: String, // Using string for now since TokenAmount is not available
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl DepositEffect {
    /// Create a new deposit effect
    pub fn new(resource_id: ResourceId, domain_id: DomainId, amount: String) -> Self {
        Self {
            id: crate::effect::EffectId::new_unique(),
            resource_id,
            domain_id,
            amount,
            dependencies: Vec::new(),
            snapshot: None,
        }
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.dependencies.push(dependency);
    }
    
    /// Add multiple fact dependencies to this effect
    pub fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>) {
        self.dependencies.extend(dependencies);
    }
    
    /// Add a fact snapshot to this effect
    pub fn with_fact_snapshot(&mut self, snapshot: FactSnapshot) {
        self.snapshot = Some(snapshot);
    }
}

impl crate::effect::Effect for DepositEffect {
    fn id(&self) -> &crate::effect::EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "deposit"
    }
    
    fn display_name(&self) -> String {
        format!("Deposit {}", self.amount)
    }
    
    fn description(&self) -> String {
        format!("Deposit {} to resource {}", self.amount, self.resource_id)
    }
    
    fn execute(&self, _context: &crate::effect::EffectContext) -> crate::error::Result<crate::effect::EffectOutcome> {
        // For now just return a successful outcome
        Ok(crate::effect::EffectOutcome::success(self.id.clone())
            .with_data("amount", self.amount.to_string())
            .with_data("resource_id", self.resource_id.to_string()))
    }
    
    async fn execute_async(&self, context: &crate::effect::EffectContext) -> crate::effect::EffectResult<crate::effect::EffectOutcome> {
        self.execute(context)
    }
    
    fn can_execute_in(&self, _boundary: crate::effect::boundary::ExecutionBoundary) -> bool {
        // For simplicity, allow execution in any boundary
        true
    }
    
    fn preferred_boundary(&self) -> crate::effect::boundary::ExecutionBoundary {
        // Prefer local execution
        crate::effect::boundary::ExecutionBoundary::Local
    }
    
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();
        params.insert("resource_id".to_string(), self.resource_id.to_string());
        params.insert("amount".to_string(), self.amount.to_string());
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params
    }
    
    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        self.snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // Simple validation - just check if we have at least one fact for deposits
        if self.dependencies.is_empty() && self.snapshot.is_none() {
            return Err(crate::error::Error::ValidationError(
                "Deposit effect requires at least one fact dependency".to_string()
            ));
        }
        Ok(())
    }
}

/// Factory function to create a new deposit effect
pub fn deposit(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: String,
) -> Box<dyn crate::effect::Effect> {
    Box::new(DepositEffect::new(resource_id, domain_id, amount))
}

/// Factory function to create a new deposit effect with fact information
pub fn deposit_with_facts(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: String,
    fact_snapshot: FactSnapshot,
) -> Box<DepositEffect> {
    let mut effect = DepositEffect::new(resource_id, domain_id, amount);
    effect.snapshot = Some(fact_snapshot);
    Box::new(effect)
}

/// Represents a withdrawal effect
#[derive(Debug, Clone)]
pub struct WithdrawalEffect {
    /// The unique ID of this effect
    pub id: crate::effect::EffectId,
    /// The resource to withdraw
    pub resource_id: ResourceId,
    /// The domain to withdraw from
    pub domain_id: DomainId,
    /// The amount to withdraw
    pub amount: String, // Using string for now
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl WithdrawalEffect {
    /// Create a new withdrawal effect
    pub fn new(resource_id: ResourceId, domain_id: DomainId, amount: String) -> Self {
        Self {
            id: crate::effect::EffectId::new_unique(),
            resource_id,
            domain_id,
            amount,
            dependencies: Vec::new(),
            snapshot: None,
        }
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.dependencies.push(dependency);
    }
    
    /// Add multiple fact dependencies to this effect
    pub fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>) {
        self.dependencies.extend(dependencies);
    }
    
    /// Add a fact snapshot to this effect
    pub fn with_fact_snapshot(&mut self, snapshot: FactSnapshot) {
        self.snapshot = Some(snapshot);
    }
}

impl crate::effect::Effect for WithdrawalEffect {
    fn id(&self) -> &crate::effect::EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "withdrawal"
    }
    
    fn display_name(&self) -> String {
        format!("Withdraw {}", self.amount)
    }
    
    fn description(&self) -> String {
        format!("Withdraw {} from resource {}", self.amount, self.resource_id)
    }
    
    fn execute(&self, _context: &crate::effect::EffectContext) -> crate::error::Result<crate::effect::EffectOutcome> {
        // For now just return a successful outcome
        Ok(crate::effect::EffectOutcome::success(self.id.clone())
            .with_data("amount", self.amount.to_string())
            .with_data("resource_id", self.resource_id.to_string()))
    }
    
    async fn execute_async(&self, context: &crate::effect::EffectContext) -> crate::effect::EffectResult<crate::effect::EffectOutcome> {
        self.execute(context)
    }
    
    fn can_execute_in(&self, _boundary: crate::effect::boundary::ExecutionBoundary) -> bool {
        // For simplicity, allow execution in any boundary
        true
    }
    
    fn preferred_boundary(&self) -> crate::effect::boundary::ExecutionBoundary {
        // Prefer local execution
        crate::effect::boundary::ExecutionBoundary::Local
    }
    
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();
        params.insert("resource_id".to_string(), self.resource_id.to_string());
        params.insert("amount".to_string(), self.amount.to_string());
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params
    }
    
    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        self.snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // Withdrawals should require a balance fact
        if self.dependencies.is_empty() && self.snapshot.is_none() {
            return Err(crate::error::Error::ValidationError(
                "Withdrawal effect requires at least one balance fact".to_string()
            ));
        }
        Ok(())
    }
}

/// Factory function to create a new withdrawal effect
pub fn withdrawal(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: String, // Change from TokenAmount to String
) -> Box<dyn crate::effect::Effect> {
    Box::new(WithdrawalEffect::new(resource_id, domain_id, amount))
}

/// Factory function to create a new withdrawal effect with fact information
pub fn withdrawal_with_facts(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: String, // Change from TokenAmount to String
    fact_snapshot: FactSnapshot,
) -> Box<WithdrawalEffect> {
    let mut effect = WithdrawalEffect::new(resource_id, domain_id, amount);
    effect.snapshot = Some(fact_snapshot);
    Box::new(effect)
}

/// Represents an observation effect
#[derive(Debug, Clone)]
pub struct ObservationEffect {
    /// The unique ID of this effect
    pub id: crate::effect::EffectId,
    /// The resource to observe
    pub resource_id: ResourceId,
    /// The domain to observe
    pub domain_id: DomainId,
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl ObservationEffect {
    /// Create a new observation effect
    pub fn new(resource_id: ResourceId, domain_id: DomainId) -> Self {
        Self {
            id: crate::effect::EffectId::new_unique(),
            resource_id,
            domain_id,
            dependencies: Vec::new(),
            snapshot: None,
        }
    }
    
    /// Add a fact dependency to this effect
    pub fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.dependencies.push(dependency);
    }
    
    /// Add multiple fact dependencies to this effect
    pub fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>) {
        self.dependencies.extend(dependencies);
    }
    
    /// Add a fact snapshot to this effect
    pub fn with_fact_snapshot(&mut self, snapshot: FactSnapshot) {
        self.snapshot = Some(snapshot);
    }
}

impl crate::effect::Effect for ObservationEffect {
    fn id(&self) -> &crate::effect::EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "observation"
    }
    
    fn display_name(&self) -> String {
        format!("Observe resource {}", self.resource_id)
    }
    
    fn description(&self) -> String {
        format!("Observe resource {} in domain {}", self.resource_id, self.domain_id)
    }
    
    fn execute(&self, _context: &crate::effect::EffectContext) -> crate::error::Result<crate::effect::EffectOutcome> {
        // For now just return a successful outcome
        Ok(crate::effect::EffectOutcome::success(self.id.clone())
            .with_data("resource_id", self.resource_id.to_string())
            .with_data("domain_id", self.domain_id.to_string()))
    }
    
    async fn execute_async(&self, context: &crate::effect::EffectContext) -> crate::effect::EffectResult<crate::effect::EffectOutcome> {
        self.execute(context)
    }
    
    fn can_execute_in(&self, _boundary: crate::effect::boundary::ExecutionBoundary) -> bool {
        // For simplicity, allow execution in any boundary
        true
    }
    
    fn preferred_boundary(&self) -> crate::effect::boundary::ExecutionBoundary {
        // Prefer local execution
        crate::effect::boundary::ExecutionBoundary::Local
    }
    
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();
        params.insert("resource_id".to_string(), self.resource_id.to_string());
        params.insert("domain_id".to_string(), self.domain_id.to_string());
        params
    }
    
    fn fact_dependencies(&self) -> Vec<crate::log::fact_snapshot::FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<crate::log::fact_snapshot::FactSnapshot> {
        self.snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // Observation effects don't strictly need fact dependencies
        Ok(())
    }
}

/// Factory function to create a new observation effect
pub fn observation(
    resource_id: ResourceId,
    domain_id: DomainId,
) -> Box<dyn crate::effect::Effect> {
    Box::new(ObservationEffect::new(resource_id, domain_id))
}

/// Factory function to create a new observation effect with fact information
pub fn observation_with_facts(
    resource_id: ResourceId,
    domain_id: DomainId,
    fact_snapshot: FactSnapshot,
) -> Box<ObservationEffect> {
    let mut effect = ObservationEffect::new(resource_id, domain_id);
    effect.snapshot = Some(fact_snapshot);
    Box::new(effect)
} 