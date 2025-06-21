//! Intent-based programming system with unified transform constraints
//!
//! This module provides declarative effect specification through intents,
//! which are high-level descriptions of desired computations that can be
//! automatically compiled to efficient execution plans.
//!
//! **Phase 3 Updates**: Unified transform constraints, location requirements,
//! and migration specifications replace separate constraint types.

use crate::{
    lambda::{
        base::{TypeInner, SessionType, Location, Value},
        Term, TermKind,
    },
    system::{
        content_addressing::{ResourceId, Timestamp},
        deterministic::DeterministicSystem,
    },
    effect::{
        transform_constraint::{TransformConstraint, TransformConstraintError},
        capability::Capability,
    },
};
use ssz::{Encode, Decode};
use ssz_derive::{Encode, Decode};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

/// Unique identifier for intents
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IntentId(pub u64);

impl IntentId {
    /// Create a new intent ID
    pub fn new(id: u64) -> Self {
        IntentId(id)
    }
    
    /// Generate a deterministic intent ID
    pub fn generate(det_sys: &mut DeterministicSystem) -> Self {
        IntentId(det_sys.next_counter())
    }
}

/// Intent represents a high-level, declarative specification of desired computation
/// that can involve both local operations and distributed communication
#[derive(Debug, Clone)]
pub struct Intent {
    /// Unique identifier for this intent
    pub id: IntentId,
    
    /// Primary location where this intent should be executed
    pub domain: Location,
    
    /// Unified transform constraints that specify the desired computation
    pub constraints: Vec<TransformConstraint>,
    
    /// Resource bindings for the intent
    pub resource_bindings: BTreeMap<String, ResourceRef>,
    
    /// Location requirements and migration specifications
    pub location_requirements: LocationRequirements,
    
    /// Expected result type
    pub expected_result: Option<TypeInner>,
    
    /// Priority for execution scheduling
    pub priority: IntentPriority,
    
    /// Timeout for intent execution
    pub timeout: Option<u64>,
    
    /// Dependencies on other intents
    pub dependencies: BTreeSet<IntentId>,
}

/// Location requirements for intent execution
#[derive(Debug, Clone)]
pub struct LocationRequirements {
    /// Preferred execution location
    pub preferred_location: Option<Location>,
    
    /// Allowed execution locations
    pub allowed_locations: BTreeSet<Location>,
    
    /// Data migration specifications
    pub migration_specs: Vec<MigrationSpec>,
    
    /// Required protocols for distributed operations
    pub required_protocols: BTreeMap<String, SessionType>,
    
    /// Performance constraints
    pub performance_constraints: PerformanceConstraints,
}

/// Data migration specification
#[derive(Debug, Clone)]
pub struct MigrationSpec {
    /// Source location
    pub from: Location,
    
    /// Target location
    pub to: Location,
    
    /// Data to migrate
    pub data_refs: Vec<ResourceRef>,
    
    /// Migration strategy
    pub strategy: MigrationStrategy,
    
    /// Required protocol for migration
    pub protocol: Option<SessionType>,
}

/// Migration strategy for data movement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationStrategy {
    /// Copy data to new location, keep original
    Copy,
    
    /// Move data to new location, remove original
    Move,
    
    /// Replicate data across multiple locations
    Replicate {
        target_locations: BTreeSet<Location>,
        consistency_model: ConsistencyModel,
    },
    
    /// Partition data across locations
    Partition {
        partition_strategy: PartitionStrategy,
        target_locations: BTreeSet<Location>,
    },
}

/// Consistency model for replicated data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyModel {
    /// Strong consistency - all replicas updated before operation completes
    Strong,
    
    /// Eventual consistency - updates propagated asynchronously
    Eventual,
    
    /// Causal consistency - causally related operations are ordered
    Causal,
    
    /// Session consistency - consistency within a session
    Session,
}

/// Partition strategy for distributed data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionStrategy {
    /// Hash-based partitioning
    Hash { hash_function: String },
    
    /// Range-based partitioning
    Range { partition_key: String },
    
    /// Round-robin partitioning
    RoundRobin,
    
    /// Custom partitioning function
    Custom { function_name: String },
}

/// Performance constraints for intent execution
#[derive(Debug, Clone)]
pub struct PerformanceConstraints {
    /// Maximum execution time in milliseconds
    pub max_execution_time: Option<u64>,
    
    /// Maximum memory usage in bytes
    pub max_memory_usage: Option<u64>,
    
    /// Maximum network bandwidth usage in bytes/second
    pub max_bandwidth_usage: Option<u64>,
    
    /// Preferred parallelization level
    pub preferred_parallelization: Option<u64>,
    
    /// Cost constraints
    pub cost_constraints: CostConstraints,
}

/// Cost constraints for intent execution
#[derive(Debug, Clone)]
pub struct CostConstraints {
    /// Maximum computational cost
    pub max_compute_cost: Option<u64>,
    
    /// Maximum communication cost
    pub max_communication_cost: Option<u64>,
    
    /// Maximum storage cost
    pub max_storage_cost: Option<u64>,
    
    /// Cost optimization strategy
    pub optimization_strategy: CostOptimizationStrategy,
}

/// Cost optimization strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostOptimizationStrategy {
    /// Minimize total cost
    MinimizeTotalCost,
    
    /// Minimize execution time
    MinimizeTime,
    
    /// Minimize resource usage
    MinimizeResources,
    
    /// Balance cost and performance
    Balanced,
    
    /// Custom optimization function
    Custom { function_name: String },
}

/// Priority levels for intent execution
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntentPriority {
    Low,
    Normal,
    High,
    Critical,
    Immediate,
}

/// Reference to a resource used in an intent
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRef {
    /// Resource identifier
    pub id: ResourceId,
    
    /// Expected type of the resource
    pub resource_type: TypeInner,
    
    /// Location where the resource is currently stored
    pub current_location: Location,
    
    /// Access pattern for this resource
    pub access_pattern: AccessPattern,
}

/// Access pattern for resource usage
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessPattern {
    /// Read-only access
    ReadOnly,
    
    /// Write-only access
    WriteOnly,
    
    /// Read-write access
    ReadWrite,
    
    /// Linear access (consume exactly once)
    Linear,
    
    /// Streaming access
    Streaming {
        chunk_size: Option<u64>,
        prefetch_size: Option<u64>,
    },
    
    /// Random access
    Random {
        access_frequency: u64,
        cache_size: Option<u64>,
    },
}

/// Resource binding associates a name with a resource reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBinding {
    /// Binding name
    pub name: String,
    
    /// Resource reference
    pub resource: ResourceRef,
    
    /// Whether this binding is required or optional
    pub required: bool,
    
    /// Default value if optional and not provided
    pub default_value: Option<String>, // Simplified representation
}

/// Intent execution result
#[derive(Debug, Clone)]
pub enum IntentResult {
    /// Intent completed successfully
    Success {
        result: Option<ResourceRef>,
        execution_stats: ExecutionStats,
    },
    
    /// Intent failed with error
    Error {
        error: IntentError,
        partial_results: Vec<ResourceRef>,
    },
    
    /// Intent was cancelled
    Cancelled {
        reason: String,
    },
    
    /// Intent timed out
    Timeout {
        partial_results: Vec<ResourceRef>,
    },
}

/// Execution statistics for completed intents
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    /// Total execution time in milliseconds
    pub execution_time: u64,
    
    /// Memory usage in bytes
    pub memory_usage: u64,
    
    /// Network bandwidth used in bytes
    pub network_usage: u64,
    
    /// Computational cost
    pub compute_cost: u64,
    
    /// Communication cost
    pub communication_cost: u64,
    
    /// Storage cost
    pub storage_cost: u64,
    
    /// Number of locations involved
    pub locations_used: BTreeSet<Location>,
    
    /// Protocols used for communication
    pub protocols_used: Vec<SessionType>,
}

/// Errors that can occur during intent processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentError {
    /// Invalid intent specification
    InvalidIntent(String),
    
    /// Resource not found
    ResourceNotFound(ResourceId),
    
    /// Insufficient capabilities for operation
    InsufficientCapabilities {
        required: Vec<String>,
        available: Vec<String>,
    },
    
    /// Location not accessible
    LocationNotAccessible {
        location: Location,
        reason: String,
    },
    
    /// Protocol not supported
    ProtocolNotSupported {
        protocol: String,
        location: Location,
    },
    
    /// Constraint solving failed
    ConstraintSolvingFailed(TransformConstraintError),
    
    /// Migration failed
    MigrationFailed {
        from: Location,
        to: Location,
        reason: String,
    },
    
    /// Performance constraint violated
    PerformanceConstraintViolated {
        constraint: String,
        actual: u64,
        limit: u64,
    },
    
    /// Cost constraint violated
    CostConstraintViolated {
        constraint: String,
        actual: u64,
        limit: u64,
    },
    
    /// Dependency cycle detected
    DependencyCycle(Vec<IntentId>),
    
    /// External system error
    ExternalError(String),
}

impl Intent {
    /// Create a new intent with the given domain
    pub fn new(domain: Location) -> Self {
        Self {
            id: IntentId(0), // Will be set when registered
            domain,
            constraints: Vec::new(),
            resource_bindings: BTreeMap::new(),
            location_requirements: LocationRequirements::default(),
            expected_result: None,
            priority: IntentPriority::Normal,
            timeout: None,
            dependencies: BTreeSet::new(),
        }
    }
    
    /// Add a transform constraint to the intent
    pub fn with_constraint(mut self, constraint: TransformConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Add a resource binding to the intent
    pub fn with_resource(mut self, name: String, resource: ResourceRef) -> Self {
        self.resource_bindings.insert(name, resource);
        self
    }
    
    /// Set location requirements
    pub fn with_location_requirements(mut self, requirements: LocationRequirements) -> Self {
        self.location_requirements = requirements;
        self
    }
    
    /// Set expected result type
    pub fn with_expected_result(mut self, result_type: TypeInner) -> Self {
        self.expected_result = Some(result_type);
        self
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: IntentPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }
    
    /// Add a dependency on another intent
    pub fn with_dependency(mut self, dependency: IntentId) -> Self {
        self.dependencies.insert(dependency);
        self
    }
    
    /// Add a migration specification
    pub fn with_migration(mut self, migration: MigrationSpec) -> Self {
        self.location_requirements.migration_specs.push(migration);
        self
    }
    
    /// Add a required protocol
    pub fn with_protocol(mut self, name: String, protocol: SessionType) -> Self {
        self.location_requirements.required_protocols.insert(name, protocol);
        self
    }
    
    /// Check if this intent can be executed at a given location
    pub fn can_execute_at(&self, location: &Location) -> bool {
        // Check if location is allowed
        if !self.location_requirements.allowed_locations.is_empty() {
            if !self.location_requirements.allowed_locations.contains(location) {
                return false;
            }
        }
        
        // Check constraints for location compatibility
        self.constraints.iter().all(|constraint| {
            match constraint {
                TransformConstraint::LocalTransform { .. } => {
                    matches!(location, Location::Local)
                }
                TransformConstraint::RemoteTransform { source_location, target_location, .. } => {
                    location == source_location || location == target_location
                }
                TransformConstraint::DataMigration { from_location, to_location, .. } => {
                    location == from_location || location == to_location
                }
                _ => true, // Other constraints don't have location restrictions
            }
        })
    }
    
    /// Estimate the execution cost of this intent
    pub fn estimate_cost(&self, target_location: &Location) -> IntentCostEstimate {
        let mut compute_cost = 0u64;
        let mut communication_cost = 0u64;
        let mut storage_cost = 0u64;
        
        // Estimate cost based on constraints
        for constraint in &self.constraints {
            match constraint {
                TransformConstraint::LocalTransform { .. } => {
                    compute_cost += 10; // Base cost for local computation
                }
                TransformConstraint::RemoteTransform { .. } => {
                    communication_cost += 100; // Higher cost for remote operations
                    compute_cost += 5; // Protocol overhead
                }
                TransformConstraint::DataMigration { .. } => {
                    communication_cost += 500; // High cost for data movement
                    storage_cost += 50; // Storage cost for migration
                }
                TransformConstraint::DistributedSync { locations, .. } => {
                    communication_cost += locations.len() as u64 * 50; // Cost scales with participants
                }
                _ => {
                    compute_cost += 5; // Default cost
                }
            }
        }
        
        // Factor in resource access costs
        for resource in self.resource_bindings.values() {
            if resource.current_location != *target_location {
                communication_cost += 200; // Cost for remote resource access
            }
            
            match resource.access_pattern {
                AccessPattern::Streaming { .. } => {
                    communication_cost += 300; // Higher cost for streaming
                }
                AccessPattern::Random { access_frequency, .. } => {
                    compute_cost += access_frequency * 2; // Cost scales with access frequency
                }
                _ => {
                    compute_cost += 10; // Base access cost
                }
            }
        }
        
        IntentCostEstimate {
            compute_cost,
            communication_cost,
            storage_cost,
            total_cost: compute_cost + communication_cost + storage_cost,
            estimated_execution_time: compute_cost + communication_cost / 10, // Simplified time estimate
        }
    }
    
    /// Get all locations involved in this intent
    pub fn involved_locations(&self) -> BTreeSet<Location> {
        let mut locations = BTreeSet::new();
        
        // Add domain location
        locations.insert(self.domain.clone());
        
        // Add locations from constraints
        for constraint in &self.constraints {
            match constraint {
                TransformConstraint::RemoteTransform { source_location, target_location, .. } => {
                    locations.insert(source_location.clone());
                    locations.insert(target_location.clone());
                }
                TransformConstraint::DataMigration { from_location, to_location, .. } => {
                    locations.insert(from_location.clone());
                    locations.insert(to_location.clone());
                }
                TransformConstraint::DistributedSync { locations: sync_locs, .. } => {
                    locations.extend(sync_locs.iter().cloned());
                }
                _ => {}
            }
        }
        
        // Add locations from resource bindings
        for resource in self.resource_bindings.values() {
            locations.insert(resource.current_location.clone());
        }
        
        // Add locations from migration specs
        for migration in &self.location_requirements.migration_specs {
            locations.insert(migration.from.clone());
            locations.insert(migration.to.clone());
        }
        
        locations
    }
}

/// Cost estimate for intent execution
#[derive(Debug, Clone)]
pub struct IntentCostEstimate {
    /// Computational cost
    pub compute_cost: u64,
    
    /// Communication cost
    pub communication_cost: u64,
    
    /// Storage cost
    pub storage_cost: u64,
    
    /// Total cost
    pub total_cost: u64,
    
    /// Estimated execution time in milliseconds
    pub estimated_execution_time: u64,
}

impl Default for LocationRequirements {
    fn default() -> Self {
        Self {
            preferred_location: None,
            allowed_locations: BTreeSet::new(),
            migration_specs: Vec::new(),
            required_protocols: BTreeMap::new(),
            performance_constraints: PerformanceConstraints::default(),
        }
    }
}

impl Default for PerformanceConstraints {
    fn default() -> Self {
        Self {
            max_execution_time: None,
            max_memory_usage: None,
            max_bandwidth_usage: None,
            preferred_parallelization: None,
            cost_constraints: CostConstraints::default(),
        }
    }
}

impl Default for CostConstraints {
    fn default() -> Self {
        Self {
            max_compute_cost: None,
            max_communication_cost: None,
            max_storage_cost: None,
            optimization_strategy: CostOptimizationStrategy::Balanced,
        }
    }
}

impl ResourceRef {
    /// Create a new resource reference
    pub fn new(id: ResourceId, resource_type: TypeInner, location: Location) -> Self {
        Self {
            id,
            resource_type,
            current_location: location,
            access_pattern: AccessPattern::ReadOnly,
        }
    }
    
    /// Set the access pattern for this resource
    pub fn with_access_pattern(mut self, pattern: AccessPattern) -> Self {
        self.access_pattern = pattern;
        self
    }
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentError::InvalidIntent(msg) => write!(f, "Invalid intent: {}", msg),
            IntentError::ResourceNotFound(id) => write!(f, "Resource not found: {:?}", id),
            IntentError::InsufficientCapabilities { required, available } => {
                write!(f, "Insufficient capabilities: required {:?}, available {:?}", required, available)
            }
            IntentError::LocationNotAccessible { location, reason } => {
                write!(f, "Location not accessible: {:?} - {}", location, reason)
            }
            IntentError::ProtocolNotSupported { protocol, location } => {
                write!(f, "Protocol {} not supported at location {:?}", protocol, location)
            }
            IntentError::ConstraintSolvingFailed(err) => {
                write!(f, "Constraint solving failed: {}", err)
            }
            IntentError::MigrationFailed { from, to, reason } => {
                write!(f, "Migration failed from {:?} to {:?}: {}", from, to, reason)
            }
            IntentError::PerformanceConstraintViolated { constraint, actual, limit } => {
                write!(f, "Performance constraint violated: {} (actual: {}, limit: {})", constraint, actual, limit)
            }
            IntentError::CostConstraintViolated { constraint, actual, limit } => {
                write!(f, "Cost constraint violated: {} (actual: {}, limit: {})", constraint, actual, limit)
            }
            IntentError::DependencyCycle(cycle) => {
                write!(f, "Dependency cycle detected: {:?}", cycle)
            }
            IntentError::ExternalError(msg) => write!(f, "External error: {}", msg),
        }
    }
}

impl std::error::Error for IntentError {}

impl From<TransformConstraintError> for IntentError {
    fn from(err: TransformConstraintError) -> Self {
        IntentError::ConstraintSolvingFailed(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::BaseType;
    
    #[test]
    fn test_intent_creation() {
        let intent = Intent::new(Location::Local);
        assert_eq!(intent.domain, Location::Local);
        assert_eq!(intent.constraints.len(), 0);
        assert_eq!(intent.priority, IntentPriority::Normal);
    }
    
    #[test]
    fn test_intent_with_constraints() {
        let constraint = TransformConstraint::LocalTransform {
            source_type: TypeInner::Base(BaseType::Int),
            target_type: TypeInner::Base(BaseType::String),
            transform: crate::effect::transform_constraint::TransformDefinition::FunctionApplication {
                function: "int_to_string".to_string(),
                argument: "input".to_string(),
            },
        };
        
        let intent = Intent::new(Location::Local)
            .with_constraint(constraint)
            .with_priority(IntentPriority::High);
        
        assert_eq!(intent.constraints.len(), 1);
        assert_eq!(intent.priority, IntentPriority::High);
    }
    
    #[test]
    fn test_intent_location_checking() {
        let constraint = TransformConstraint::LocalTransform {
            source_type: TypeInner::Base(BaseType::Int),
            target_type: TypeInner::Base(BaseType::String),
            transform: crate::effect::transform_constraint::TransformDefinition::FunctionApplication {
                function: "test".to_string(),
                argument: "arg".to_string(),
            },
        };
        
        let intent = Intent::new(Location::Local)
            .with_constraint(constraint);
        
        assert!(intent.can_execute_at(&Location::Local));
        assert!(!intent.can_execute_at(&Location::Remote("server".to_string())));
    }
    
    #[test]
    fn test_cost_estimation() {
        let intent = Intent::new(Location::Local);
        let estimate = intent.estimate_cost(&Location::Local);
        
        assert!(estimate.total_cost > 0);
        assert!(estimate.estimated_execution_time > 0);
    }
} 