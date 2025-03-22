// Handler module for Causality Effect System
//
// This module provides the effect handler functionality for handling
// different types of effects in the system.

use std::sync::Arc;

use crate::effect::EffectType;
use crate::error::Result;
use crate::types::{ResourceId, DomainId};
use crate::zk::{RiscVProgram, Witness, Proof};

// Import for the effect type definitions
#[cfg(feature = "zk-vm")]
use crate::effect_adapters::zk::{
    CompileZkProgramEffect, GenerateZkWitnessEffect,
    GenerateZkProofEffect, VerifyZkProofEffect
};

/// Handler trait for executing effects
///
/// Effect handlers are responsible for executing effects in the Causality system.
/// Each handler implements behavior for specific effect types.
pub trait EffectHandler: Send + Sync {
    /// Check if this handler can handle the given effect type
    fn can_handle(&self, effect_type: &EffectType) -> bool;
    
    /// Handle a deposit effect
    fn handle_deposit(&self, account: String, amount: i64, timestamp: u64) -> Result<()>;
    
    /// Handle a withdrawal effect
    fn handle_withdrawal(&self, account: String, amount: i64, timestamp: u64) -> Result<()>;
    
    /// Handle an observation effect
    fn handle_observation(&self, account: String, timestamp: u64) -> Result<i64>;
    
    /// Handle compiling a ZK program
    #[cfg(feature = "zk-vm")]
    fn handle_compile_zk_program(&self, effect: CompileZkProgramEffect) -> Result<RiscVProgram> {
        Err(crate::error::Error::OperationFailed("Handler does not implement ZK program compilation".to_string()))
    }
    
    /// Handle generating a ZK witness
    #[cfg(feature = "zk-vm")]
    fn handle_generate_zk_witness(&self, effect: GenerateZkWitnessEffect) -> Result<Witness> {
        Err(crate::error::Error::OperationFailed("Handler does not implement ZK witness generation".to_string()))
    }
    
    /// Handle generating a ZK proof
    #[cfg(feature = "zk-vm")]
    fn handle_generate_zk_proof(&self, effect: GenerateZkProofEffect) -> Result<Proof> {
        Err(crate::error::Error::OperationFailed("Handler does not implement ZK proof generation".to_string()))
    }
    
    /// Handle verifying a ZK proof
    #[cfg(feature = "zk-vm")]
    fn handle_verify_zk_proof(&self, effect: VerifyZkProofEffect) -> Result<bool> {
        Err(crate::error::Error::OperationFailed("Handler does not implement ZK proof verification".to_string()))
    }
}

/// A composite handler that tries multiple handlers in sequence
///
/// This handler tries the primary handler first, and if it returns a specific
/// error result, it falls back to the secondary handler.
#[derive(Debug)]
pub struct CompositeHandler<P, S> {
    /// The primary handler to try first
    primary: P,
    /// The secondary handler to use as a fallback
    secondary: S,
}

impl<P, S> CompositeHandler<P, S> {
    /// Create a new composite handler
    pub fn new(primary: P, secondary: S) -> Self {
        CompositeHandler { primary, secondary }
    }
}

impl<P, S> EffectHandler for CompositeHandler<P, S>
where
    P: EffectHandler,
    S: EffectHandler,
{
    fn can_handle(&self, effect_type: &EffectType) -> bool {
        self.primary.can_handle(effect_type) || self.secondary.can_handle(effect_type)
    }
    
    fn handle_deposit(&self, account: String, amount: i64, timestamp: u64) -> Result<()> {
        match self.primary.handle_deposit(account.clone(), amount, timestamp) {
            Ok(()) => Ok(()),
            Err(_) => self.secondary.handle_deposit(account, amount, timestamp),
        }
    }
    
    fn handle_withdrawal(&self, account: String, amount: i64, timestamp: u64) -> Result<()> {
        match self.primary.handle_withdrawal(account.clone(), amount, timestamp) {
            Ok(()) => Ok(()),
            Err(_) => self.secondary.handle_withdrawal(account, amount, timestamp),
        }
    }
    
    fn handle_observation(&self, account: String, timestamp: u64) -> Result<i64> {
        match self.primary.handle_observation(account.clone(), timestamp) {
            Ok(balance) => Ok(balance),
            Err(_) => self.secondary.handle_observation(account, timestamp),
        }
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_compile_zk_program(&self, effect: CompileZkProgramEffect) -> Result<RiscVProgram> {
        match self.primary.handle_compile_zk_program(effect.clone()) {
            Ok(program) => Ok(program),
            Err(_) => self.secondary.handle_compile_zk_program(effect),
        }
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_generate_zk_witness(&self, effect: GenerateZkWitnessEffect) -> Result<Witness> {
        match self.primary.handle_generate_zk_witness(effect.clone()) {
            Ok(witness) => Ok(witness),
            Err(_) => self.secondary.handle_generate_zk_witness(effect),
        }
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_generate_zk_proof(&self, effect: GenerateZkProofEffect) -> Result<Proof> {
        match self.primary.handle_generate_zk_proof(effect.clone()) {
            Ok(proof) => Ok(proof),
            Err(_) => self.secondary.handle_generate_zk_proof(effect),
        }
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_verify_zk_proof(&self, effect: VerifyZkProofEffect) -> Result<bool> {
        match self.primary.handle_verify_zk_proof(effect.clone()) {
            Ok(result) => Ok(result),
            Err(_) => self.secondary.handle_verify_zk_proof(effect),
        }
    }
}

/// Shared handler that can be cloned and shared between multiple contexts
pub type SharedHandler = Arc<dyn EffectHandler>;

/// Create a shared handler from any effect handler
pub fn shared<H: EffectHandler + 'static>(handler: H) -> SharedHandler {
    Arc::new(handler)
}

/// A no-op handler that returns failure results for all effects
///
/// This is useful as a placeholder or for testing.
#[derive(Debug, Default)]
pub struct NoopHandler;

impl EffectHandler for NoopHandler {
    fn can_handle(&self, _effect_type: &EffectType) -> bool {
        false // NoopHandler cannot handle any effects
    }
    
    fn handle_deposit(&self, _account: String, _amount: i64, _timestamp: u64) -> Result<()> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot perform deposits".to_string()))
    }
    
    fn handle_withdrawal(&self, _account: String, _amount: i64, _timestamp: u64) -> Result<()> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot perform withdrawals".to_string()))
    }
    
    fn handle_observation(&self, _account: String, _timestamp: u64) -> Result<i64> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot perform observations".to_string()))
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_compile_zk_program(&self, _effect: CompileZkProgramEffect) -> Result<RiscVProgram> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot compile ZK programs".to_string()))
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_generate_zk_witness(&self, _effect: GenerateZkWitnessEffect) -> Result<Witness> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot generate ZK witnesses".to_string()))
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_generate_zk_proof(&self, _effect: GenerateZkProofEffect) -> Result<Proof> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot generate ZK proofs".to_string()))
    }
    
    #[cfg(feature = "zk-vm")]
    fn handle_verify_zk_proof(&self, _effect: VerifyZkProofEffect) -> Result<bool> {
        Err(crate::error::Error::OperationFailed("Noop handler cannot verify ZK proofs".to_string()))
    }
}

/// Helper function to compose two effect handlers
pub fn compose<P, S>(primary: P, secondary: S) -> CompositeHandler<P, S>
where
    P: EffectHandler,
    S: EffectHandler,
{
    CompositeHandler::new(primary, secondary)
} 