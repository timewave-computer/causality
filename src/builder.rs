// Effect builder for Causality
//
// This module provides a builder pattern for creating Causality effects.

use std::marker::PhantomData;

use crate::continuation;
use crate::effect::{Effect, factory};
use crate::error::Result;
use crate::types::{Account, Amount, Timestamp, Balance};

/// Builder for creating Causality effects
///
/// This struct provides a fluent interface for creating effects.
#[derive(Debug, Clone)]
pub struct EffectBuilder<T> {
    account: Option<Account>,
    amount: Option<Amount>,
    timestamp: Option<Timestamp>,
    _marker: PhantomData<T>,
}

impl<T> Default for EffectBuilder<T> {
    fn default() -> Self {
        EffectBuilder {
            account: None,
            amount: None,
            timestamp: None,
            _marker: PhantomData,
        }
    }
}

impl<T> EffectBuilder<T>
where
    T: 'static + Send + Sync,
{
    /// Create a new effect builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the account for the effect
    pub fn account(mut self, account: Account) -> Self {
        self.account = Some(account);
        self
    }
    
    /// Set the amount for the effect
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }
    
    /// Set the timestamp for the effect
    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
    
    /// Create a deposit effect
    pub fn deposit(self) -> Box<dyn Effect> {
        factory::deposit(
            self.account.expect("Account is required for deposit"),
            self.amount.expect("Amount is required for deposit"),
            self.timestamp.expect("Timestamp is required for deposit"),
            |result| match result {
                Ok(()) => (),
                Err(e) => panic!("Deposit failed: {}", e),
            },
        )
    }
    
    /// Create a withdrawal effect
    pub fn withdrawal(self) -> Box<dyn Effect> {
        factory::withdrawal(
            self.account.expect("Account is required for withdrawal"),
            self.amount.expect("Amount is required for withdrawal"),
            self.timestamp.expect("Timestamp is required for withdrawal"),
            |result| match result {
                Ok(()) => (),
                Err(e) => panic!("Withdrawal failed: {}", e),
            },
        )
    }
    
    /// Create an observation effect
    pub fn observation(self) -> Box<dyn Effect> {
        factory::observation(
            self.account.expect("Account is required for observation"),
            self.timestamp.expect("Timestamp is required for observation"),
            |result| match result {
                Ok(balance) => balance,
                Err(e) => panic!("Observation failed: {}", e),
            },
        )
    }
}

/// A sequence of effects that are executed in order
#[derive(Debug)]
pub struct SequenceEffect<R> {
    effects: Vec<Box<dyn Effect>>,
    continuation: Box<dyn continuation::Continuation<(), R>>,
}

impl<R> SequenceEffect<R> {
    /// Create a new sequence effect
    pub fn new(effects: Vec<Box<dyn Effect>>, continuation: Box<dyn continuation::Continuation<(), R>>) -> Self {
        SequenceEffect { effects, continuation }
    }
}

impl<R> Effect for SequenceEffect<R> {
    fn id(&self) -> &crate::effect::EffectId {
        // In a real implementation, this would return a proper ID
        unimplemented!("SequenceEffect id not implemented")
    }
    
    fn name(&self) -> &str {
        "sequence"
    }
    
    fn display_name(&self) -> String {
        "Sequence Effect".to_string()
    }
    
    fn description(&self) -> String {
        "A sequence of effects executed in order".to_string()
    }
    
    fn execute(&self, context: &crate::effect::EffectContext) -> crate::error::Result<crate::effect::EffectOutcome> {
        // In a real implementation, this would execute all effects in sequence
        unimplemented!("SequenceEffect execute not implemented")
    }
    
    async fn execute_async(&self, context: &crate::effect::EffectContext) -> crate::effect::EffectResult<crate::effect::EffectOutcome> {
        // In a real implementation, this would execute all effects in sequence asynchronously
        unimplemented!("SequenceEffect execute_async not implemented")
    }
    
    fn can_execute_in(&self, boundary: crate::effect::boundary::ExecutionBoundary) -> bool {
        // Default implementation - can be improved based on inner effects
        true
    }
    
    fn preferred_boundary(&self) -> crate::effect::boundary::ExecutionBoundary {
        // Default to inside system
        crate::effect::boundary::ExecutionBoundary::InsideSystem
    }
    
    fn display_parameters(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

/// Trait for creating sequences of effects
pub trait SequenceBuilder {
    /// Create a sequence of effects
    fn sequence(self) -> Box<dyn Effect>;
}

impl<I> SequenceBuilder for I
where
    I: IntoIterator<Item = Box<dyn Effect>>,
{
    fn sequence(self) -> Box<dyn Effect> {
        let effects: Vec<Box<dyn Effect>> = self.into_iter().collect();
        
        if effects.is_empty() {
            // Create a no-op effect
            factory::deposit(
                Account("default".to_string()),
                Amount(0), 
                Timestamp(0),
                |_| ()
            )
        } else {
            // Execute all effects in sequence, returning () when done
            let mut iter = effects.into_iter();
            let first = iter.next().unwrap();
            iter.fold(first, |acc, effect| {
                factory::deposit(
                    Account("sequence".to_string()),
                    Amount(0),
                    Timestamp(0),
                    move |_| {
                        acc.execute(&crate::handler::NoopHandler);
                        effect.execute(&crate::handler::NoopHandler);
                        ()
                    }
                )
            })
        }
    }
}

/// Convenience functions for creating common effects
pub mod prelude {
    use super::*;
    
    /// Create a new effect builder
    pub fn effect<T>() -> EffectBuilder<T> {
        EffectBuilder::new()
    }
    
    /// Create a deposit effect
    pub fn deposit(account: Account, amount: Amount, timestamp: Timestamp) -> Box<dyn Effect> {
        EffectBuilder::new()
            .account(account)
            .amount(amount)
            .timestamp(timestamp)
            .deposit()
    }
    
    /// Create a withdrawal effect
    pub fn withdrawal(account: Account, amount: Amount, timestamp: Timestamp) -> Box<dyn Effect> {
        EffectBuilder::new()
            .account(account)
            .amount(amount)
            .timestamp(timestamp)
            .withdrawal()
    }
    
    /// Create an observation effect
    pub fn observation(account: Account, timestamp: Timestamp) -> Box<dyn Effect> {
        EffectBuilder::new()
            .account(account)
            .timestamp(timestamp)
            .observation()
    }
    
    /// Create a sequence effect
    pub fn sequence(effects: Vec<Box<dyn Effect>>) -> Box<dyn Effect> {
        effects.into_iter().sequence()
    }
} 