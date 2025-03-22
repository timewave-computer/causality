pub mod boundary;
pub mod transfer_effect;
pub mod private_effect;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::address::Address;
use crate::resource::{ResourceAPI, ResourceId, CapabilityRef, Right};
use crate::program_account::ProgramAccount;
use crate::effect::boundary::{
    EffectContext, ExecutionBoundary, BoundaryCrossing, BoundaryError, 
    BoundaryCrossingRegistry, CrossingDirection
};

/// Result type for effect operations
pub type EffectResult<T> = Result<T, EffectError>;

/// Errors that can occur during effect execution
#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Capability error: {0}")]
    CapabilityError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Boundary error: {0}")]
    BoundaryError(#[from] BoundaryError),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Not implemented")]
    NotImplemented,
}

/// Trait for effects that can be executed within the system
#[async_trait]
pub trait Effect: Send + Sync {
    /// Get the name of the effect
    fn name(&self) -> &str;
    
    /// Get the description of the effect
    fn description(&self) -> &str;
    
    /// Get the required capabilities for this effect
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)>;
    
    /// Execute the effect
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Check if the effect can be executed in the given boundary
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool;
    
    /// Get the boundary where this effect should be executed
    fn preferred_boundary(&self) -> ExecutionBoundary;
}

/// Represents the outcome of an effect execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectOutcome {
    /// The execution context ID
    pub execution_id: uuid::Uuid,
    
    /// Whether the effect execution was successful
    pub success: bool,
    
    /// The result data if successful
    pub result: Option<serde_json::Value>,
    
    /// Error message if unsuccessful
    pub error: Option<String>,
    
    /// Resource changes resulting from the effect
    pub resource_changes: Vec<ResourceChange>,
    
    /// Metadata about the execution
    pub metadata: HashMap<String, String>,
}

/// Represents a change to a resource resulting from an effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChange {
    /// The ID of the resource that changed
    pub resource_id: ResourceId,
    
    /// The type of change
    pub change_type: ResourceChangeType,
    
    /// Previous state hash (if available)
    pub previous_state_hash: Option<String>,
    
    /// New state hash
    pub new_state_hash: String,
}

/// Types of resource changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceChangeType {
    /// Resource was created
    Created,
    
    /// Resource was updated
    Updated,
    
    /// Resource was deleted
    Deleted,
    
    /// Resource was transferred
    Transferred,
    
    /// Resource was locked
    Locked,
    
    /// Resource was unlocked
    Unlocked,
}

/// Registry for managing available effects
pub struct EffectRegistry {
    effects: HashMap<String, Arc<dyn Effect>>,
    crossing_registry: BoundaryCrossingRegistry,
}

impl EffectRegistry {
    /// Create a new effect registry
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            crossing_registry: BoundaryCrossingRegistry::new(),
        }
    }
    
    /// Register an effect
    pub fn register(&mut self, effect: Arc<dyn Effect>) {
        self.effects.insert(effect.name().to_string(), effect);
    }
    
    /// Get an effect by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Effect>> {
        self.effects.get(name).cloned()
    }
    
    /// Get all registered effects
    pub fn get_all(&self) -> Vec<Arc<dyn Effect>> {
        self.effects.values().cloned().collect()
    }
    
    /// Get all effects that can be executed in the given boundary
    pub fn get_for_boundary(&self, boundary: ExecutionBoundary) -> Vec<Arc<dyn Effect>> {
        self.effects.values()
            .filter(|effect| effect.can_execute_in(boundary))
            .cloned()
            .collect()
    }
    
    /// Record a boundary crossing
    pub fn record_crossing<T>(&mut self, crossing: &BoundaryCrossing<T>, direction: CrossingDirection, success: bool, error: Option<String>)
    where
        T: std::any::Any,
    {
        self.crossing_registry.record(crossing, direction, success, error);
    }
    
    /// Get the boundary crossing registry
    pub fn crossing_registry(&self) -> &BoundaryCrossingRegistry {
        &self.crossing_registry
    }
}

/// Manages effect execution with boundary awareness
pub struct EffectManager {
    registry: EffectRegistry,
    resource_api: Arc<dyn ResourceAPI>,
}

impl EffectManager {
    /// Create a new effect manager
    pub fn new(resource_api: Arc<dyn ResourceAPI>) -> Self {
        Self {
            registry: EffectRegistry::new(),
            resource_api,
        }
    }
    
    /// Register an effect
    pub fn register_effect(&mut self, effect: Arc<dyn Effect>) {
        self.registry.register(effect);
    }
    
    /// Get the effect registry
    pub fn registry(&self) -> &EffectRegistry {
        &self.registry
    }
    
    /// Get a mutable reference to the effect registry
    pub fn registry_mut(&mut self) -> &mut EffectRegistry {
        &mut self.registry
    }
    
    /// Execute an effect with boundary crossing handling
    pub async fn execute_effect(&self, effect_name: &str, context: EffectContext) -> EffectResult<EffectOutcome> {
        let effect = self.registry.get(effect_name)
            .ok_or_else(|| EffectError::ExecutionError(format!("Effect not found: {}", effect_name)))?;
        
        // Check if the effect can execute in the given boundary
        if !effect.can_execute_in(context.boundary) {
            // We need to cross a boundary
            let preferred_boundary = effect.preferred_boundary();
            
            if preferred_boundary == ExecutionBoundary::InsideSystem && context.boundary == ExecutionBoundary::OutsideSystem {
                // We need to cross from outside to inside
                return self.cross_boundary_to_inside(effect, context).await;
            } else if preferred_boundary == ExecutionBoundary::OutsideSystem && context.boundary == ExecutionBoundary::InsideSystem {
                // We need to cross from inside to outside
                return self.cross_boundary_to_outside(effect, context).await;
            }
        }
        
        // We can execute in the current boundary
        self.execute_effect_in_current_boundary(effect, context).await
    }
    
    /// Execute an effect in the current boundary
    async fn execute_effect_in_current_boundary(&self, effect: Arc<dyn Effect>, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Check capabilities
        self.verify_capabilities(&effect, &context).await?;
        
        // Execute the effect
        let outcome = effect.execute(context).await?;
        
        Ok(outcome)
    }
    
    /// Cross boundary from outside to inside
    async fn cross_boundary_to_inside(&self, effect: Arc<dyn Effect>, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Create a boundary crossing
        let crossing = BoundaryCrossing::new_inbound(
            context.clone(),
            effect.name().to_string(),
        );
        
        // Record the crossing
        self.registry.record_crossing(&crossing, CrossingDirection::Inbound, true, None);
        
        // Create a new inside context
        let inside_context = EffectContext {
            boundary: ExecutionBoundary::InsideSystem,
            ..context
        };
        
        // Execute the effect in the inside boundary
        let outcome = self.execute_effect_in_current_boundary(effect, inside_context).await;
        
        // Record the outcome
        if let Err(ref e) = outcome {
            self.registry.record_crossing(&crossing, CrossingDirection::Inbound, false, Some(e.to_string()));
        }
        
        outcome
    }
    
    /// Cross boundary from inside to outside
    async fn cross_boundary_to_outside(&self, effect: Arc<dyn Effect>, context: EffectContext) -> EffectResult<EffectOutcome> {
        // Create a boundary crossing
        let crossing = BoundaryCrossing::new_outbound(
            context.clone(),
            effect.name().to_string(),
        );
        
        // Record the crossing
        self.registry.record_crossing(&crossing, CrossingDirection::Outbound, true, None);
        
        // Create a new outside context
        let outside_context = EffectContext {
            boundary: ExecutionBoundary::OutsideSystem,
            ..context
        };
        
        // Execute the effect in the outside boundary
        let outcome = self.execute_effect_in_current_boundary(effect, outside_context).await;
        
        // Record the outcome
        if let Err(ref e) = outcome {
            self.registry.record_crossing(&crossing, CrossingDirection::Outbound, false, Some(e.to_string()));
        }
        
        outcome
    }
    
    /// Verify that the context has the required capabilities for the effect
    async fn verify_capabilities(&self, effect: &Arc<dyn Effect>, context: &EffectContext) -> EffectResult<()> {
        let required_capabilities = effect.required_capabilities();
        
        for (resource_id, right) in required_capabilities {
            let has_capability = context.capabilities.iter().any(|cap| {
                // Check if any capability applies to this resource and has the required right
                let cap_obj = cap.capability();
                
                // Check if resource ID matches or capability has wildcard
                let resource_matches = cap_obj.resource_id() == "*" || cap_obj.resource_id() == resource_id.to_string();
                
                // Check if capability has the required right
                let has_right = cap_obj.has_right(&right);
                
                resource_matches && has_right
            });
            
            if !has_capability {
                return Err(EffectError::CapabilityError(format!(
                    "Missing capability for resource {} with right {:?}",
                    resource_id, right
                )));
            }
        }
        
        Ok(())
    }
}

/// A marker trait for effects that can be used with program accounts
pub trait ProgramAccountEffect: Effect {
    /// Get the program account types this effect can be applied to
    fn applicable_account_types(&self) -> Vec<&'static str>;
    
    /// Check if this effect can be applied to a specific program account
    fn can_apply_to(&self, account: &dyn ProgramAccount) -> bool;
    
    /// Get a display name for this effect that is shown to users
    fn display_name(&self) -> &str;
    
    /// Get display parameters that describe what this effect does
    fn display_parameters(&self) -> HashMap<String, String>;
} 