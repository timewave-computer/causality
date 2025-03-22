// TEL Effect Module
//
// This module defines the effect system for Temporal Effect Language,
// including resource effects, proofs, and adaptation to different domains.

pub mod proof;

// Re-export core components
pub use self::proof::{
    EffectProofGenerator,
    EffectProofVerifier,
    EffectProofFormat,
    EffectProofMetadata,
};

use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::tel::{
    error::{TelError, TelResult},
    types::{ResourceId, Domain, Address, OperationId, Proof},
    resource::{
        ResourceManager,
        ResourceOperation,
        RegisterId,
        RegisterState,
        ResourceOperationType,
        RegisterContents,
    },
};

/// Represents an effect that can be applied to resources
#[derive(Debug, Clone)]
pub struct ResourceEffect {
    /// ID of the effect
    pub id: Uuid,
    /// The operation this effect will perform
    pub operation: ResourceOperation,
    /// The proof associated with this effect (if any)
    pub proof: Option<Proof>,
    /// Whether this effect requires verification
    pub requires_verification: bool,
}

impl ResourceEffect {
    /// Create a new resource effect
    pub fn new(operation: ResourceOperation) -> Self {
        Self {
            id: Uuid::new_v4(),
            operation,
            proof: None,
            requires_verification: false,
        }
    }

    /// Set a proof for this effect
    pub fn with_proof(mut self, proof: Proof) -> Self {
        self.proof = Some(proof);
        self.requires_verification = true;
        self
    }

    /// Mark this effect as requiring verification
    pub fn requires_verification(mut self, value: bool) -> Self {
        self.requires_verification = value;
        self
    }
}

/// The result of applying a resource effect
#[derive(Debug, Clone)]
pub struct EffectResult {
    /// ID of the effect
    pub effect_id: Uuid,
    /// ID of the operation
    pub operation_id: OperationId,
    /// Whether the effect was successful
    pub success: bool,
    /// Result data if any
    pub data: Option<RegisterContents>,
    /// Error message if the effect failed
    pub error: Option<String>,
}

/// Adapter that can apply resource effects
pub struct ResourceEffectAdapter {
    /// Resource manager
    resource_manager: Arc<ResourceManager>,
}

impl ResourceEffectAdapter {
    /// Create a new resource effect adapter
    pub fn new(resource_manager: Arc<ResourceManager>) -> Self {
        Self {
            resource_manager,
        }
    }

    /// Apply a resource effect
    pub fn apply(&self, effect: ResourceEffect) -> TelResult<EffectResult> {
        // Process the effect based on the operation type
        match &effect.operation.op_type {
            ResourceOperationType::Create { owner, domain, initial_data } => {
                // Create a new resource
                let result = self.resource_manager.create_resource(
                    owner,
                    domain,
                    initial_data.clone(),
                )?;

                Ok(EffectResult {
                    effect_id: effect.id,
                    operation_id: effect.operation.id.clone(),
                    success: true,
                    data: Some(RegisterContents::ResourceId(result)),
                    error: None,
                })
            },
            ResourceOperationType::Update { resource_id, new_data } => {
                // Update an existing resource
                let register_id = self.resource_manager.get_register(resource_id)?
                    .id;

                self.resource_manager.update_register(
                    &register_id,
                    new_data.clone(),
                )?;

                Ok(EffectResult {
                    effect_id: effect.id,
                    operation_id: effect.operation.id.clone(),
                    success: true,
                    data: None,
                    error: None,
                })
            },
            ResourceOperationType::Delete { resource_id } => {
                // Delete a resource
                self.resource_manager.delete_resource(resource_id)?;

                Ok(EffectResult {
                    effect_id: effect.id,
                    operation_id: effect.operation.id.clone(),
                    success: true,
                    data: None,
                    error: None,
                })
            },
            ResourceOperationType::Transfer { resource_id, new_owner } => {
                // Transfer ownership of a resource
                self.resource_manager.transfer_resource(resource_id, new_owner)?;

                Ok(EffectResult {
                    effect_id: effect.id,
                    operation_id: effect.operation.id.clone(),
                    success: true,
                    data: None,
                    error: None,
                })
            },
            ResourceOperationType::Lock { resource_id } => {
                // Lock a resource
                let register_id = self.resource_manager.get_register(resource_id)?
                    .id;

                self.resource_manager.lock_register(&register_id)?;

                Ok(EffectResult {
                    effect_id: effect.id,
                    operation_id: effect.operation.id.clone(),
                    success: true,
                    data: None,
                    error: None,
                })
            },
            ResourceOperationType::Unlock { resource_id } => {
                // Unlock a resource
                let register_id = self.resource_manager.get_register(resource_id)?
                    .id;

                self.resource_manager.unlock_register(&register_id)?;

                Ok(EffectResult {
                    effect_id: effect.id,
                    operation_id: effect.operation.id.clone(),
                    success: true,
                    data: None,
                    error: None,
                })
            },
            ResourceOperationType::Custom { resource_id, action, params } => {
                // Handle custom operations
                // This would be extended for specific domain implementations
                Err(TelError::UnsupportedOperation(format!(
                    "Custom operation '{}' is not supported by this adapter",
                    action
                )))
            }
        }
    }

    /// Apply multiple effects in sequence
    pub fn apply_sequence(&self, effects: Vec<ResourceEffect>) -> TelResult<Vec<EffectResult>> {
        let mut results = Vec::with_capacity(effects.len());
        
        for effect in effects {
            match self.apply(effect) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    /// Apply effects in parallel when possible
    pub fn apply_parallel(&self, effects: Vec<ResourceEffect>) -> TelResult<Vec<EffectResult>> {
        // This is a simplified implementation - in a real system, this would
        // analyze dependencies between effects and execute independent effects
        // in parallel. For now, we'll just use the sequential implementation.
        self.apply_sequence(effects)
    }
    
    /// Apply a repeating effect based on the specified schedule
    pub fn apply_repeating(&self, repeater: &RepeatingEffect) -> TelResult<Vec<EffectResult>> {
        // Check if the repeater is active
        if !repeater.is_active() {
            return Ok(Vec::new());
        }
        
        // Calculate how many iterations should be executed since the last run
        let iterations = repeater.iterations_due();
        
        if iterations == 0 {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::with_capacity(iterations);
        
        // Apply the effect the required number of times
        for _ in 0..iterations {
            match self.apply(repeater.effect.clone()) {
                Ok(result) => {
                    results.push(result);
                    
                    // If the effect fails, stop the iteration
                    if !result.success {
                        break;
                    }
                },
                Err(e) => return Err(e),
            }
        }
        
        // Update the last execution time
        repeater.update_last_execution();
        
        Ok(results)
    }
}

/// Schedule type for repeating effects
#[derive(Debug, Clone, PartialEq)]
pub enum RepeatSchedule {
    /// Fixed number of repetitions
    Count(usize),
    /// Regular interval
    Interval(Duration),
    /// Repeat until a specific time
    Until(SystemTime),
    /// Indefinitely (limited by max_iterations)
    Indefinite,
}

/// Configuration for repeating effects
#[derive(Debug, Clone)]
pub struct RepeatConfig {
    /// The schedule for repetition
    pub schedule: RepeatSchedule,
    /// Maximum number of iterations (safety limit)
    pub max_iterations: usize,
    /// Whether to retry on failure
    pub retry_on_failure: bool,
    /// Maximum number of retries for failed attempts
    pub max_retries: usize,
    /// Delay between retries
    pub retry_delay: Duration,
}

impl Default for RepeatConfig {
    fn default() -> Self {
        Self {
            schedule: RepeatSchedule::Count(1),
            max_iterations: 100,
            retry_on_failure: false,
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
        }
    }
}

/// A repeating effect that can be executed multiple times
#[derive(Debug)]
pub struct RepeatingEffect {
    /// The effect to repeat
    pub effect: ResourceEffect,
    /// Configuration for repeating
    pub config: RepeatConfig,
    /// Time of first execution
    pub start_time: SystemTime,
    /// Last execution time
    last_execution: Arc<std::sync::Mutex<Option<SystemTime>>>,
    /// Current iteration count
    iteration_count: Arc<std::sync::Mutex<usize>>,
    /// Whether the repeater is active
    active: Arc<std::sync::Mutex<bool>>,
}

impl RepeatingEffect {
    /// Create a new repeating effect
    pub fn new(effect: ResourceEffect, config: RepeatConfig) -> Self {
        Self {
            effect,
            config,
            start_time: SystemTime::now(),
            last_execution: Arc::new(std::sync::Mutex::new(None)),
            iteration_count: Arc::new(std::sync::Mutex::new(0)),
            active: Arc::new(std::sync::Mutex::new(true)),
        }
    }
    
    /// Create a repeating effect with a specific number of repetitions
    pub fn repeat_count(effect: ResourceEffect, count: usize) -> Self {
        let config = RepeatConfig {
            schedule: RepeatSchedule::Count(count),
            ..Default::default()
        };
        
        Self::new(effect, config)
    }
    
    /// Create a repeating effect with a regular interval
    pub fn repeat_interval(effect: ResourceEffect, interval: Duration) -> Self {
        let config = RepeatConfig {
            schedule: RepeatSchedule::Interval(interval),
            ..Default::default()
        };
        
        Self::new(effect, config)
    }
    
    /// Create a repeating effect that runs until a specific time
    pub fn repeat_until(effect: ResourceEffect, until: SystemTime) -> Self {
        let config = RepeatConfig {
            schedule: RepeatSchedule::Until(until),
            ..Default::default()
        };
        
        Self::new(effect, config)
    }
    
    /// Create an indefinitely repeating effect
    pub fn repeat_indefinitely(effect: ResourceEffect) -> Self {
        let config = RepeatConfig {
            schedule: RepeatSchedule::Indefinite,
            ..Default::default()
        };
        
        Self::new(effect, config)
    }
    
    /// Check if the repeating effect is active
    pub fn is_active(&self) -> bool {
        let active = self.active.lock().unwrap();
        
        if !*active {
            return false;
        }
        
        // Check if we've reached the max iterations
        let count = self.iteration_count.lock().unwrap();
        if *count >= self.config.max_iterations {
            return false;
        }
        
        // Check schedule-specific conditions
        match self.config.schedule {
            RepeatSchedule::Count(limit) => *count < limit,
            RepeatSchedule::Until(end_time) => {
                match SystemTime::now().duration_since(end_time) {
                    Ok(_) => false, // End time has passed
                    Err(_) => true,  // End time is in the future
                }
            },
            _ => true, // Other schedules don't have termination conditions
        }
    }
    
    /// Calculate how many iterations are due since the last execution
    pub fn iterations_due(&self) -> usize {
        let now = SystemTime::now();
        let last_exec = self.last_execution.lock().unwrap();
        
        match self.config.schedule {
            RepeatSchedule::Count(_) => {
                // For count-based schedules, we always return 1 to execute one at a time
                1
            },
            RepeatSchedule::Interval(interval) => {
                // For interval-based schedules, calculate how many intervals have passed
                match *last_exec {
                    Some(time) => {
                        match now.duration_since(time) {
                            Ok(elapsed) => {
                                elapsed.as_secs() / interval.as_secs()
                            },
                            Err(_) => 0, // Clock went backwards, no iterations due
                        }
                    },
                    None => {
                        // First execution
                        match now.duration_since(self.start_time) {
                            Ok(elapsed) => {
                                elapsed.as_secs() / interval.as_secs()
                            },
                            Err(_) => 0, // Clock went backwards, no iterations due
                        }
                    }
                }
            },
            RepeatSchedule::Until(_) | RepeatSchedule::Indefinite => {
                // For these schedules, we execute one at a time
                1
            }
        }
    }
    
    /// Update the last execution time to now
    pub fn update_last_execution(&self) {
        let mut last_exec = self.last_execution.lock().unwrap();
        *last_exec = Some(SystemTime::now());
        
        let mut count = self.iteration_count.lock().unwrap();
        *count += 1;
    }
    
    /// Stop the repeating effect
    pub fn stop(&self) {
        let mut active = self.active.lock().unwrap();
        *active = false;
    }
    
    /// Resume the repeating effect
    pub fn resume(&self) {
        let mut active = self.active.lock().unwrap();
        *active = true;
    }
    
    /// Reset the repeating effect
    pub fn reset(&self) {
        let mut last_exec = self.last_execution.lock().unwrap();
        *last_exec = None;
        
        let mut count = self.iteration_count.lock().unwrap();
        *count = 0;
        
        let mut active = self.active.lock().unwrap();
        *active = true;
    }
}

/// Composes multiple effects into a single composite effect
pub struct EffectComposer {
    effects: Vec<ResourceEffect>,
}

impl EffectComposer {
    /// Create a new effect composer
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }
    
    /// Add an effect to the composition
    pub fn add_effect(&mut self, effect: ResourceEffect) {
        self.effects.push(effect);
    }
    
    /// Get all effects in the composition
    pub fn get_effects(&self) -> &[ResourceEffect] {
        &self.effects
    }
    
    /// Check if the composition is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
    
    /// Apply conditional logic to effect composition
    pub fn with_condition<F>(&mut self, condition: bool, f: F) -> &mut Self 
    where
        F: FnOnce(&mut EffectComposer),
    {
        if condition {
            f(self);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tel::resource::model::ResourceManager;
    
    #[test]
    fn test_resource_effect_creation() {
        let operation = ResourceOperation::new(
            ResourceOperationType::Create {
                owner: Address::random(),
                domain: Domain::new("test"),
                initial_data: RegisterContents::Text("Hello, World!".to_string()),
            },
        );
        
        let effect = ResourceEffect::new(operation);
        
        assert!(effect.id != Uuid::nil());
        assert!(!effect.requires_verification);
        assert!(effect.proof.is_none());
    }
    
    #[test]
    fn test_effect_with_proof() {
        let operation = ResourceOperation::new(
            ResourceOperationType::Create {
                owner: Address::random(),
                domain: Domain::new("test"),
                initial_data: RegisterContents::Text("Hello, World!".to_string()),
            },
        );
        
        let proof = Proof::new("test", vec![1, 2, 3, 4]);
        let effect = ResourceEffect::new(operation).with_proof(proof.clone());
        
        assert!(effect.requires_verification);
        assert_eq!(effect.proof.unwrap(), proof);
    }
    
    #[test]
    fn test_repeating_effect() {
        let operation = ResourceOperation::new(
            ResourceOperationType::Create {
                owner: Address::random(),
                domain: Domain::new("test"),
                initial_data: RegisterContents::Text("Hello, World!".to_string()),
            },
        );
        
        let effect = ResourceEffect::new(operation);
        
        // Test count-based repeating
        let repeater = RepeatingEffect::repeat_count(effect.clone(), 5);
        assert!(repeater.is_active());
        assert_eq!(repeater.iterations_due(), 1);
        
        // Test interval-based repeating
        let interval = Duration::from_secs(10);
        let repeater = RepeatingEffect::repeat_interval(effect.clone(), interval);
        assert!(repeater.is_active());
        
        // Test until-based repeating
        let until = SystemTime::now() + Duration::from_secs(60);
        let repeater = RepeatingEffect::repeat_until(effect.clone(), until);
        assert!(repeater.is_active());
        
        // Test indefinite repeating
        let repeater = RepeatingEffect::repeat_indefinitely(effect);
        assert!(repeater.is_active());
    }
} 