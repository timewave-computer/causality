// Factory functions for creating effects
//
// This module provides factory functions for creating common effect types.

use crate::effect::{Effect, SerializableEffect};
use crate::effect::EffectHandler;
use crate::effect::EffectType;
use crate::effect::Continuation;
use crate::error::Result;
use crate::effect::types::EffectType;
use crate::types::{ResourceId, DomainId, TokenAmount};
use crate::log::fact_snapshot::{FactSnapshot, FactId, FactDependency, FactDependencyType};
use std::sync::Arc;
use std::fmt;

/// Represents a deposit effect
#[derive(Debug, Clone)]
pub struct DepositEffect {
    /// The resource to deposit
    pub resource_id: ResourceId,
    /// The domain to deposit to
    pub domain_id: DomainId,
    /// The amount to deposit
    pub amount: TokenAmount,
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl crate::effect::Effect for DepositEffect {
    type Output = bool;
    
    fn get_type(&self) -> EffectType {
        EffectType::Deposit
    }
    
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
    
    fn clone_box(&self) -> Box<dyn crate::effect::Effect<Output = Self::Output>> {
        Box::new(self.clone())
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.domain_id.clone()]
    }
    
    fn execute(self, handler: &dyn crate::effect::EffectHandler) -> Self::Output {
        handler.handle_deposit(self.resource_id, self.domain_id, self.amount)
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.snapshot.clone()
    }
}

impl crate::effect::EffectWithFactDependencies for DepositEffect {
    fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.dependencies.push(dependency);
    }
    
    fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>) {
        self.dependencies.extend(dependencies);
    }
    
    fn with_fact_snapshot(&mut self, snapshot: FactSnapshot) {
        self.snapshot = Some(snapshot);
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

/// Create a deposit effect
pub fn deposit(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: TokenAmount,
) -> Box<dyn crate::effect::Effect<Output = bool>> {
    Box::new(DepositEffect {
        resource_id,
        domain_id,
        amount,
        dependencies: Vec::new(),
        snapshot: None,
    })
}

/// Create a deposit effect with fact dependencies
pub fn deposit_with_facts(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: TokenAmount,
    fact_snapshot: FactSnapshot,
) -> Box<DepositEffect> {
    Box::new(DepositEffect {
        resource_id,
        domain_id,
        amount,
        dependencies: Vec::new(),
        snapshot: Some(fact_snapshot),
    })
}

/// Represents a withdrawal effect
#[derive(Debug, Clone)]
pub struct WithdrawalEffect {
    /// The resource to withdraw
    pub resource_id: ResourceId,
    /// The domain to withdraw from
    pub domain_id: DomainId,
    /// The amount to withdraw
    pub amount: TokenAmount,
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl crate::effect::Effect for WithdrawalEffect {
    type Output = bool;
    
    fn get_type(&self) -> EffectType {
        EffectType::Withdraw
    }
    
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
    
    fn clone_box(&self) -> Box<dyn crate::effect::Effect<Output = Self::Output>> {
        Box::new(self.clone())
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.domain_id.clone()]
    }
    
    fn execute(self, handler: &dyn crate::effect::EffectHandler) -> Self::Output {
        handler.handle_withdrawal(self.resource_id, self.domain_id, self.amount)
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.snapshot.clone()
    }
}

impl crate::effect::EffectWithFactDependencies for WithdrawalEffect {
    fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.dependencies.push(dependency);
    }
    
    fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>) {
        self.dependencies.extend(dependencies);
    }
    
    fn with_fact_snapshot(&mut self, snapshot: FactSnapshot) {
        self.snapshot = Some(snapshot);
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

/// Create a withdrawal effect
pub fn withdrawal(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: TokenAmount,
) -> Box<dyn crate::effect::Effect<Output = bool>> {
    Box::new(WithdrawalEffect {
        resource_id,
        domain_id,
        amount,
        dependencies: Vec::new(),
        snapshot: None,
    })
}

/// Create a withdrawal effect with fact dependencies
pub fn withdrawal_with_facts(
    resource_id: ResourceId,
    domain_id: DomainId,
    amount: TokenAmount,
    fact_snapshot: FactSnapshot,
) -> Box<WithdrawalEffect> {
    Box::new(WithdrawalEffect {
        resource_id,
        domain_id,
        amount,
        dependencies: Vec::new(),
        snapshot: Some(fact_snapshot),
    })
}

/// Represents an observation effect
#[derive(Debug, Clone)]
pub struct ObservationEffect {
    /// The resource to observe
    pub resource_id: ResourceId,
    /// The domain to observe
    pub domain_id: DomainId,
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl crate::effect::Effect for ObservationEffect {
    type Output = TokenAmount;
    
    fn get_type(&self) -> EffectType {
        EffectType::Observe
    }
    
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
    
    fn clone_box(&self) -> Box<dyn crate::effect::Effect<Output = Self::Output>> {
        Box::new(self.clone())
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.domain_id.clone()]
    }
    
    fn execute(self, handler: &dyn crate::effect::EffectHandler) -> Self::Output {
        handler.handle_observation(self.resource_id, self.domain_id)
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.snapshot.clone()
    }
}

impl crate::effect::EffectWithFactDependencies for ObservationEffect {
    fn with_fact_dependency(
        &mut self,
        fact_id: FactId,
        domain_id: DomainId,
        dependency_type: FactDependencyType,
    ) {
        let dependency = FactDependency::new(fact_id, domain_id, dependency_type);
        self.dependencies.push(dependency);
    }
    
    fn with_fact_dependencies(&mut self, dependencies: Vec<FactDependency>) {
        self.dependencies.extend(dependencies);
    }
    
    fn with_fact_snapshot(&mut self, snapshot: FactSnapshot) {
        self.snapshot = Some(snapshot);
    }
    
    fn validate_fact_dependencies(&self) -> crate::error::Result<()> {
        // Observation effects don't strictly need fact dependencies
        Ok(())
    }
}

/// Create an observation effect
pub fn observation(
    resource_id: ResourceId,
    domain_id: DomainId,
) -> Box<dyn crate::effect::Effect<Output = TokenAmount>> {
    Box::new(ObservationEffect {
        resource_id,
        domain_id,
        dependencies: Vec::new(),
        snapshot: None,
    })
}

/// Create an observation effect with fact dependencies
pub fn observation_with_facts(
    resource_id: ResourceId,
    domain_id: DomainId,
    fact_snapshot: FactSnapshot,
) -> Box<ObservationEffect> {
    Box::new(ObservationEffect {
        resource_id,
        domain_id,
        dependencies: Vec::new(),
        snapshot: Some(fact_snapshot),
    })
} 