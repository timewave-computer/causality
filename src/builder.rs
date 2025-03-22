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
    pub fn deposit(self) -> Box<dyn Effect<Output = ()>> {
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
    pub fn withdrawal(self) -> Box<dyn Effect<Output = ()>> {
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
    pub fn observation(self) -> Box<dyn Effect<Output = Balance>> {
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
    effects: Vec<Box<dyn Effect<Output = ()>>>,
    continuation: Box<dyn continuation::Continuation<(), R>>,
}

impl<R> SequenceEffect<R> {
    /// Create a new sequence effect
    pub fn new(effects: Vec<Box<dyn Effect<Output = ()>>>, continuation: Box<dyn continuation::Continuation<(), R>>) -> Self {
        SequenceEffect { effects, continuation }
    }
}

impl<R> Effect for SequenceEffect<R> {
    type Output = R;
    
    fn get_type(&self) -> crate::effect::EffectType {
        crate::effect::EffectType::Custom("Sequence".to_string())
    }
    
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }
    
    fn clone_box(&self) -> Box<dyn Effect<Output = Self::Output>> {
        // This is a placeholder. A real implementation would need to clone all effects,
        // which might not be possible depending on the effect implementations.
        unimplemented!("SequenceEffect cannot be cloned")
    }
    
    fn resources(&self) -> Vec<crate::types::ResourceId> {
        self.effects.iter()
            .flat_map(|effect| effect.resources())
            .collect()
    }
    
    fn domains(&self) -> Vec<crate::types::DomainId> {
        self.effects.iter()
            .flat_map(|effect| effect.domains())
            .collect()
    }
    
    fn execute(self, handler: &dyn crate::effect::EffectHandler) -> R {
        for effect in self.effects {
            effect.execute(handler);
        }
        self.continuation.apply(())
    }
}

/// Trait for creating sequences of effects
pub trait SequenceBuilder {
    /// Create a sequence of effects
    fn sequence(self) -> Box<dyn Effect<Output = ()>>;
}

impl<I> SequenceBuilder for I
where
    I: IntoIterator<Item = Box<dyn Effect<Output = ()>>>,
{
    fn sequence(self) -> Box<dyn Effect<Output = ()>> {
        let effects: Vec<Box<dyn Effect<Output = ()>>> = self.into_iter().collect();
        
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
    pub fn deposit(account: Account, amount: Amount, timestamp: Timestamp) -> Box<dyn Effect<Output = ()>> {
        EffectBuilder::<()>::new()
            .account(account)
            .amount(amount)
            .timestamp(timestamp)
            .deposit()
    }
    
    /// Create a withdrawal effect
    pub fn withdrawal(account: Account, amount: Amount, timestamp: Timestamp) -> Box<dyn Effect<Output = ()>> {
        EffectBuilder::<()>::new()
            .account(account)
            .amount(amount)
            .timestamp(timestamp)
            .withdrawal()
    }
    
    /// Create an observation effect
    pub fn observation(account: Account, timestamp: Timestamp) -> Box<dyn Effect<Output = Balance>> {
        EffectBuilder::<Balance>::new()
            .account(account)
            .timestamp(timestamp)
            .observation()
    }
    
    /// Create a sequence of effects
    pub fn sequence(effects: Vec<Box<dyn Effect<Output = ()>>>) -> Box<dyn Effect<Output = ()>> {
        effects.sequence()
    }
} 