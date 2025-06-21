//! Transform-Based Effect System
//!
//! This module implements the unified effect system where all operations are treated
//! as transformations between locations. This eliminates the artificial distinction
//! between computation (local transformations) and communication (distributed transformations).
//!
//! **Design Principles**:
//! - All effects are transformations `Effect<From, To>`
//! - Local computation: `Effect<Local, Local>`  
//! - Remote communication: `Effect<Local, Remote>` or `Effect<Remote, Remote>`
//! - Data migration: `Effect<LocationA, LocationB>`
//! - Unified constraint solving for all transformation types
//! - Location transparency where appropriate

use crate::{
    lambda::base::{Location, SessionType, TypeInner},
    system::content_addressing::ResourceId,
    effect::{
        transform_constraint::{TransformConstraint, TransformDefinition, Layer1Operation},
        capability::Capability,
    },
};
use std::collections::{BTreeMap, BTreeSet};

/// Unified effect representation as location-indexed transformation
/// 
/// This replaces the previous effect taxonomy with a single, elegant abstraction
/// where the only distinction is the source and target locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect<From, To> {
    /// Source location for the transformation
    pub from: From,
    
    /// Target location for the transformation  
    pub to: To,
    
    /// Type of data being transformed
    pub input_type: TypeInner,
    
    /// Expected output type
    pub output_type: TypeInner,
    
    /// Transform definition that implements this effect
    pub transform: TransformDefinition,
    
    /// Required capabilities for this transformation
    pub required_capabilities: Vec<Capability>,
    
    /// Session type required for distributed transformations
    pub required_session: Option<SessionType>,
    
    /// Resources consumed by this transformation
    pub consumed_resources: Vec<ResourceId>,
    
    /// Resources produced by this transformation
    pub produced_resources: Vec<ResourceId>,
}

/// Local effect - transformation within the same location
pub type LocalEffect = Effect<Location, Location>;

/// Remote effect - transformation between different locations
pub type RemoteEffect = Effect<Location, Location>;

/// Generic effect that can be either local or remote
pub type GenericEffect = Effect<Location, Location>;

/// Effect composition represents sequential application of transformations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectComposition {
    /// Effects to be applied in sequence
    pub effects: Vec<GenericEffect>,
    
    /// Overall input type
    pub input_type: TypeInner,
    
    /// Overall output type
    pub output_type: TypeInner,
    
    /// Intermediate locations for data flow
    pub intermediate_locations: Vec<Location>,
}

/// Effect parallel composition for concurrent transformations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectParallel {
    /// Effects to be applied in parallel
    pub effects: Vec<GenericEffect>,
    
    /// Synchronization requirements
    pub sync_requirements: Vec<SyncRequirement>,
    
    /// Merge strategy for combining results
    pub merge_strategy: MergeStrategy,
}

/// Synchronization requirement for parallel effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRequirement {
    /// Effects that must synchronize
    pub effect_indices: Vec<usize>,
    
    /// Synchronization protocol
    pub protocol: SessionType,
    
    /// Synchronization point (before/after execution)
    pub sync_point: SyncPoint,
}

/// Synchronization points for parallel effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncPoint {
    /// Synchronize before effect execution
    Before,
    
    /// Synchronize after effect execution
    After,
    
    /// Synchronize at specific intermediate points
    Intermediate(Vec<String>),
}

/// Strategy for merging results from parallel effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Concatenate all results
    Concatenate,
    
    /// Merge results using a specific function
    Function { function_name: String },
    
    /// Take the first successful result
    FirstSuccess,
    
    /// Take all results as a tuple
    Tuple,
    
    /// Custom merge logic
    Custom { merge_logic: String },
}

/// Effect execution context
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Current execution location
    pub current_location: Location,
    
    /// Available capabilities at current location
    pub available_capabilities: Vec<Capability>,
    
    /// Active session types for communication
    pub active_sessions: BTreeMap<String, SessionType>,
    
    /// Resource bindings
    pub resource_bindings: BTreeMap<String, ResourceId>,
    
    /// Execution constraints
    pub constraints: Vec<TransformConstraint>,
}

/// Effect execution result
#[derive(Debug, Clone)]
pub enum EffectResult {
    /// Effect completed successfully
    Success {
        /// Produced resources
        resources: Vec<ResourceId>,
        
        /// Updated location (for migration effects)
        new_location: Option<Location>,
        
        /// Execution statistics
        stats: EffectStats,
    },
    
    /// Effect failed
    Error {
        /// Error description
        error: EffectError,
        
        /// Partial results if any
        partial_results: Vec<ResourceId>,
    },
    
    /// Effect requires additional capabilities
    CapabilityRequired {
        /// Missing capabilities
        missing_capabilities: Vec<Capability>,
        
        /// Suggested delegation protocols
        delegation_options: Vec<SessionType>,
    },
    
    /// Effect requires migration to different location
    MigrationRequired {
        /// Target location for migration
        target_location: Location,
        
        /// Required protocol for migration
        migration_protocol: SessionType,
    },
}

/// Effect execution statistics
#[derive(Debug, Clone)]
pub struct EffectStats {
    /// Execution time in milliseconds
    pub execution_time: u64,
    
    /// Memory used in bytes
    pub memory_used: u64,
    
    /// Network bandwidth used in bytes
    pub network_used: u64,
    
    /// Computational cost
    pub compute_cost: u64,
    
    /// Communication cost
    pub communication_cost: u64,
    
    /// Locations involved in execution
    pub locations_involved: BTreeSet<Location>,
}

/// Errors that can occur during effect execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    /// Invalid transformation specification
    InvalidTransform(String),
    
    /// Type mismatch between input and expected
    TypeMismatch {
        expected: TypeInner,
        actual: TypeInner,
    },
    
    /// Location not accessible
    LocationNotAccessible {
        location: Location,
        reason: String,
    },
    
    /// Required capability not available
    CapabilityNotAvailable {
        required: Capability,
        available: Vec<Capability>,
    },
    
    /// Session protocol error
    ProtocolError {
        expected: SessionType,
        actual: Option<SessionType>,
        message: String,
    },
    
    /// Resource not found
    ResourceNotFound(ResourceId),
    
    /// Resource already consumed
    ResourceAlreadyConsumed(ResourceId),
    
    /// Constraint violation
    ConstraintViolation(String),
    
    /// Network communication error
    NetworkError(String),
    
    /// Execution timeout
    Timeout,
}

impl<From, To> Effect<From, To> 
where 
    From: Clone + PartialEq + Eq,
    To: Clone + PartialEq + Eq,
{
    /// Create a new effect transformation
    pub fn new(
        from: From,
        to: To,
        input_type: TypeInner,
        output_type: TypeInner,
        transform: TransformDefinition,
    ) -> Self {
        Self {
            from,
            to,
            input_type,
            output_type,
            transform,
            required_capabilities: Vec::new(),
            required_session: None,
            consumed_resources: Vec::new(),
            produced_resources: Vec::new(),
        }
    }
    
    /// Add a required capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
    
    /// Set the required session type
    pub fn with_session(mut self, session: SessionType) -> Self {
        self.required_session = Some(session);
        self
    }
    
    /// Add consumed resources
    pub fn consumes(mut self, resources: Vec<ResourceId>) -> Self {
        self.consumed_resources.extend(resources);
        self
    }
    
    /// Add produced resources
    pub fn produces(mut self, resources: Vec<ResourceId>) -> Self {
        self.produced_resources.extend(resources);
        self
    }
    
    /// Check if this is a local transformation
    pub fn is_local(&self) -> bool
    where
        From: PartialEq<To>,
    {
        self.from == self.to
    }
    
    /// Check if this is a distributed transformation
    pub fn is_distributed(&self) -> bool
    where
        From: PartialEq<To>,
    {
        self.from != self.to
    }
    
    /// Get the transformation type
    pub fn transformation_type(&self) -> TransformationType
    where
        From: PartialEq<To>,
    {
        if self.is_local() {
            TransformationType::Local
        } else {
            TransformationType::Distributed
        }
    }
}

/// Type of transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformationType {
    /// Local transformation (computation)
    Local,
    
    /// Distributed transformation (communication)
    Distributed,
}

impl GenericEffect {
    /// Create a local computation effect
    pub fn local_computation(
        location: Location,
        input_type: TypeInner,
        output_type: TypeInner,
        transform: TransformDefinition,
    ) -> Self {
        Effect::new(location.clone(), location, input_type, output_type, transform)
    }
    
    /// Create a remote communication effect
    pub fn remote_communication(
        from: Location,
        to: Location,
        input_type: TypeInner,
        output_type: TypeInner,
        protocol: SessionType,
    ) -> Self {
        let input_type_clone = input_type.clone();
        Effect::new(
            from,
            to,
            input_type,
            output_type,
            TransformDefinition::CommunicationSend {
                message_type: input_type_clone,
            }
        ).with_session(protocol)
    }
    
    /// Create a data migration effect
    pub fn data_migration(
        from: Location,
        to: Location,
        data_type: TypeInner,
        migration_protocol: Option<SessionType>,
    ) -> Self {
        let mut effect = Effect::new(
            from,
            to,
            data_type.clone(),
            data_type,
            TransformDefinition::ResourceConsumption {
                resource_type: "migration".to_string(),
            }
        );
        
        if let Some(protocol) = migration_protocol {
            effect = effect.with_session(protocol);
        }
        
        effect
    }
    
    /// Compose this effect with another effect sequentially
    pub fn then(self, other: GenericEffect) -> EffectComposition {
        // Store values before moving self
        let input_type = self.input_type.clone();
        let output_type = other.output_type.clone();
        let intermediate_location = other.from.clone();
        
        EffectComposition {
            effects: vec![self, other],
            input_type,
            output_type,
            intermediate_locations: vec![intermediate_location],
        }
    }
    
    /// Compose this effect with another effect in parallel
    pub fn parallel_with(self, other: GenericEffect) -> EffectParallel {
        EffectParallel {
            effects: vec![self, other],
            sync_requirements: Vec::new(),
            merge_strategy: MergeStrategy::Tuple,
        }
    }
    
    /// Convert this effect to a Layer 1 operation
    pub fn to_layer1_operation(&self) -> Layer1Operation {
        match &self.transform {
            TransformDefinition::FunctionApplication { function, argument } => {
                Layer1Operation::LambdaTerm(
                    crate::lambda::Term::new(crate::lambda::TermKind::Unit)
                )
            }
            
            TransformDefinition::CommunicationSend { message_type } => {
                if let Some(session) = &self.required_session {
                    Layer1Operation::SessionProtocol(TypeInner::Session(Box::new(session.clone())))
                } else {
                    Layer1Operation::ChannelOp {
                        operation: "send".to_string(),
                        channel_type: message_type.clone(),
                    }
                }
            }
            
            TransformDefinition::CommunicationReceive { expected_type } => {
                Layer1Operation::ChannelOp {
                    operation: "receive".to_string(),
                    channel_type: expected_type.clone(),
                }
            }
            
            TransformDefinition::StateAllocation { .. } => {
                Layer1Operation::ResourceAlloc {
                    resource_type: self.input_type.clone(),
                    initial_value: "default".to_string(),
                }
            }
            
            TransformDefinition::ResourceConsumption { .. } => {
                Layer1Operation::ResourceAlloc {
                    resource_type: self.input_type.clone(),
                    initial_value: "consumed".to_string(),
                }
            }
        }
    }
    
    /// Execute this effect in the given context
    pub fn execute(&self, context: &EffectContext) -> EffectResult {
        // Check if we have required capabilities
        for required_cap in &self.required_capabilities {
            if !context.available_capabilities.iter().any(|cap| cap.implies(required_cap)) {
                return EffectResult::CapabilityRequired {
                    missing_capabilities: vec![required_cap.clone()],
                    delegation_options: Vec::new(),
                };
            }
        }
        
        // Check if we're at the right location
        if context.current_location != self.from {
            return EffectResult::MigrationRequired {
                target_location: self.from.clone(),
                migration_protocol: self.required_session.clone().unwrap_or_else(|| {
                    // Default migration protocol - simplified Send with End continuation
                    crate::lambda::base::SessionType::Send(
                        Box::new(self.input_type.clone()),
                        Box::new(crate::lambda::base::SessionType::End)
                    )
                }),
            };
        }
        
        // Execute the transformation
        match &self.transform {
            TransformDefinition::FunctionApplication { .. } => {
                // Local computation
                EffectResult::Success {
                    resources: self.produced_resources.clone(),
                    new_location: Some(self.to.clone()),
                    stats: EffectStats {
                        execution_time: 10, // Simplified
                        memory_used: 1024,
                        network_used: 0,
                        compute_cost: 10,
                        communication_cost: 0,
                        locations_involved: [self.from.clone()].into_iter().collect(),
                    },
                }
            }
            
            TransformDefinition::CommunicationSend { .. } |
            TransformDefinition::CommunicationReceive { .. } => {
                // Distributed communication
                EffectResult::Success {
                    resources: self.produced_resources.clone(),
                    new_location: Some(self.to.clone()),
                    stats: EffectStats {
                        execution_time: 100, // Higher latency for communication
                        memory_used: 512,
                        network_used: 2048,
                        compute_cost: 5,
                        communication_cost: 50,
                        locations_involved: [self.from.clone(), self.to.clone()].into_iter().collect(),
                    },
                }
            }
            
            _ => {
                // Other transformations
                EffectResult::Success {
                    resources: self.produced_resources.clone(),
                    new_location: Some(self.to.clone()),
                    stats: EffectStats {
                        execution_time: 20,
                        memory_used: 768,
                        network_used: if self.is_distributed() { 1024 } else { 0 },
                        compute_cost: 15,
                        communication_cost: if self.is_distributed() { 25 } else { 0 },
                        locations_involved: if self.is_distributed() {
                            [self.from.clone(), self.to.clone()].into_iter().collect()
                        } else {
                            [self.from.clone()].into_iter().collect()
                        },
                    },
                }
            }
        }
    }
}

impl EffectComposition {
    /// Add another effect to the composition
    pub fn then(mut self, effect: GenericEffect) -> Self {
        self.intermediate_locations.push(effect.from.clone());
        self.effects.push(effect);
        self.output_type = self.effects.last().unwrap().output_type.clone();
        self
    }
    
    /// Execute the entire composition
    pub fn execute(&self, context: &EffectContext) -> EffectResult {
        let mut current_context = context.clone();
        let mut all_resources = Vec::new();
        let mut total_stats = EffectStats {
            execution_time: 0,
            memory_used: 0,
            network_used: 0,
            compute_cost: 0,
            communication_cost: 0,
            locations_involved: BTreeSet::new(),
        };
        
        // Execute each effect in sequence
        for effect in &self.effects {
            match effect.execute(&current_context) {
                EffectResult::Success { resources, new_location, stats } => {
                    all_resources.extend(resources);
                    
                    // Update context with new location
                    if let Some(new_loc) = new_location {
                        current_context.current_location = new_loc;
                    }
                    
                    // Accumulate statistics
                    total_stats.execution_time += stats.execution_time;
                    total_stats.memory_used = total_stats.memory_used.max(stats.memory_used);
                    total_stats.network_used += stats.network_used;
                    total_stats.compute_cost += stats.compute_cost;
                    total_stats.communication_cost += stats.communication_cost;
                    total_stats.locations_involved.extend(stats.locations_involved);
                }
                
                error => return error, // Propagate any error
            }
        }
        
        EffectResult::Success {
            resources: all_resources,
            new_location: Some(current_context.current_location),
            stats: total_stats,
        }
    }
}

impl EffectParallel {
    /// Add a synchronization requirement
    pub fn with_sync(mut self, requirement: SyncRequirement) -> Self {
        self.sync_requirements.push(requirement);
        self
    }
    
    /// Set the merge strategy
    pub fn with_merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }
    
    /// Execute all effects in parallel
    pub fn execute(&self, context: &EffectContext) -> EffectResult {
        // In a real implementation, this would execute effects concurrently
        // For now, we simulate parallel execution
        
        let mut all_resources = Vec::new();
        let mut max_stats = EffectStats {
            execution_time: 0,
            memory_used: 0,
            network_used: 0,
            compute_cost: 0,
            communication_cost: 0,
            locations_involved: BTreeSet::new(),
        };
        
        // Execute each effect (simulated parallel execution)
        for effect in &self.effects {
            match effect.execute(context) {
                EffectResult::Success { resources, stats, .. } => {
                    all_resources.extend(resources);
                    
                    // For parallel execution, take maximum time and sum costs
                    max_stats.execution_time = max_stats.execution_time.max(stats.execution_time);
                    max_stats.memory_used += stats.memory_used;
                    max_stats.network_used += stats.network_used;
                    max_stats.compute_cost += stats.compute_cost;
                    max_stats.communication_cost += stats.communication_cost;
                    max_stats.locations_involved.extend(stats.locations_involved);
                }
                
                error => return error, // Propagate any error
            }
        }
        
        EffectResult::Success {
            resources: all_resources,
            new_location: None, // Parallel effects don't change location
            stats: max_stats,
        }
    }
}

impl Default for EffectContext {
    fn default() -> Self {
        Self {
            current_location: Location::Local,
            available_capabilities: Vec::new(),
            active_sessions: BTreeMap::new(),
            resource_bindings: BTreeMap::new(),
            constraints: Vec::new(),
        }
    }
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::InvalidTransform(msg) => write!(f, "Invalid transform: {}", msg),
            EffectError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {:?}, got {:?}", expected, actual)
            }
            EffectError::LocationNotAccessible { location, reason } => {
                write!(f, "Location {:?} not accessible: {}", location, reason)
            }
            EffectError::CapabilityNotAvailable { required, available } => {
                write!(f, "Capability {:?} not available, have {:?}", required, available)
            }
            EffectError::ProtocolError { expected, actual, message } => {
                write!(f, "Protocol error: expected {:?}, got {:?} - {}", expected, actual, message)
            }
            EffectError::ResourceNotFound(id) => write!(f, "Resource not found: {:?}", id),
            EffectError::ResourceAlreadyConsumed(id) => write!(f, "Resource already consumed: {:?}", id),
            EffectError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            EffectError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            EffectError::Timeout => write!(f, "Effect execution timed out"),
        }
    }
}

impl std::error::Error for EffectError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::BaseType;
    
    #[test]
    fn test_local_effect_creation() {
        let effect = GenericEffect::local_computation(
            Location::Local,
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::String),
            TransformDefinition::FunctionApplication {
                function: "int_to_string".to_string(),
                argument: "input".to_string(),
            }
        );
        
        assert!(effect.is_local());
        assert!(!effect.is_distributed());
        assert_eq!(effect.transformation_type(), TransformationType::Local);
    }
    
    #[test]
    fn test_remote_effect_creation() {
        let effect = GenericEffect::remote_communication(
            Location::Local,
            Location::Remote("server".to_string()),
            TypeInner::Base(BaseType::String),
            TypeInner::Base(BaseType::String),
            crate::lambda::base::SessionType::Send(
                Box::new(TypeInner::Base(BaseType::String)),
                Box::new(crate::lambda::base::SessionType::End)
            )
        );
        
        assert!(!effect.is_local());
        assert!(effect.is_distributed());
        assert_eq!(effect.transformation_type(), TransformationType::Distributed);
    }
    
    #[test]
    fn test_effect_composition() {
        let effect1 = GenericEffect::local_computation(
            Location::Local,
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::String),
            TransformDefinition::FunctionApplication {
                function: "int_to_string".to_string(),
                argument: "input".to_string(),
            }
        );
        
        let effect2 = GenericEffect::local_computation(
            Location::Local,
            TypeInner::Base(BaseType::String),
            TypeInner::Base(BaseType::String),
            TransformDefinition::FunctionApplication {
                function: "uppercase".to_string(),
                argument: "input".to_string(),
            }
        );
        
        let composition = effect1.then(effect2);
        assert_eq!(composition.effects.len(), 2);
        assert_eq!(composition.input_type, TypeInner::Base(BaseType::Int));
        assert_eq!(composition.output_type, TypeInner::Base(BaseType::String));
    }
    
    #[test]
    fn test_effect_parallel_composition() {
        let effect1 = GenericEffect::local_computation(
            Location::Local,
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::String),
            TransformDefinition::FunctionApplication {
                function: "process_a".to_string(),
                argument: "input".to_string(),
            }
        );
        
        let effect2 = GenericEffect::local_computation(
            Location::Local,
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::String),
            TransformDefinition::FunctionApplication {
                function: "process_b".to_string(),
                argument: "input".to_string(),
            }
        );
        
        let parallel = effect1.parallel_with(effect2);
        assert_eq!(parallel.effects.len(), 2);
        assert_eq!(parallel.merge_strategy, MergeStrategy::Tuple);
    }
    
    #[test]
    fn test_effect_execution() {
        let effect = GenericEffect::local_computation(
            Location::Local,
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::String),
            TransformDefinition::FunctionApplication {
                function: "test".to_string(),
                argument: "input".to_string(),
            }
        );
        
        let context = EffectContext::default();
        let result = effect.execute(&context);
        
        match result {
            EffectResult::Success { stats, .. } => {
                assert!(stats.execution_time > 0);
                assert!(stats.compute_cost > 0);
                assert_eq!(stats.communication_cost, 0); // Local effect
            }
            _ => panic!("Expected successful execution"),
        }
    }
} 