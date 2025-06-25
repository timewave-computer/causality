//! Intent system for Layer 2 declarative programming
//!
//! This module defines intents as declarative specifications of desired computations
//! that can be executed across different locations with automatic migration and optimization.

use std::collections::{BTreeMap, BTreeSet};
use crate::lambda::{TypeInner, Location};
use crate::SessionType;
use crate::system::{ResourceId, DeterministicSystem};
use crate::effect::transform_constraint::{TransformConstraint, TransformConstraintError, TransformDefinition};

/// Unique identifier for an intent
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntentId(pub u64);

impl IntentId {
    /// Create a new intent ID
    pub fn new(id: u64) -> Self {
        IntentId(id)
    }
    
    /// Generate a new intent ID deterministically
    pub fn generate(det_sys: &mut DeterministicSystem) -> Self {
        IntentId(det_sys.next_counter())
    }
}

/// Intent represents a declarative specification of a desired computation
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationRequirements {
    /// Preferred execution location
    pub preferred_location: Option<Location>,
    
    /// Allowed execution locations
    pub allowed_locations: BTreeSet<Location>,
    
    /// Data migration specifications
    pub migration_specs: Vec<MigrationSpec>,
    
    /// Required protocols for communication
    pub required_protocols: BTreeMap<String, SessionType>,
    
    /// Performance constraints
    pub performance_constraints: PerformanceConstraints,
}

/// Data migration specification
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Strategy for data migration
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ConsistencyModel {
    /// Strong consistency - all replicas updated before operation completes
    Strong,
    
    /// Eventual consistency - updates propagated asynchronously
    #[default]
    Eventual,
    
    /// Causal consistency - causally related operations are ordered
    Causal,
    
    /// Session consistency - consistency within a session
    Session,
}

/// Strategy for data partitioning
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PerformanceConstraints {
    /// Maximum execution time in milliseconds
    pub max_execution_time: Option<u64>,
    
    /// Maximum memory usage in bytes
    pub max_memory_usage: Option<u64>,
    
    /// Maximum gas consumption for blockchain operations
    pub max_gas_usage: Option<u64>,
}

/// Priority levels for intent execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntentPriority {
    Low,
    Normal,
    High,
    Critical,
    Immediate,
}

/// Reference to a resource
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Access pattern for resources
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub default_value: Option<String>,
}

/// Result of intent execution
#[derive(Debug, Clone)]
pub enum IntentResult {
    /// Intent completed successfully
    Success {
        result: Box<Option<ResourceRef>>,
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

/// Execution statistics
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
    
    /// Dependency cycle detected
    DependencyCycle(Vec<IntentId>),
    
    /// External system error
    ExternalError(String),
}

impl Intent {
    /// Create a new intent for the given domain
    pub fn new(domain: Location) -> Self {
        Self {
            id: IntentId::new(0), // Will be set by the system
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
    
    /// Add a transform constraint to this intent
    pub fn with_constraint(mut self, constraint: TransformConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Add a resource binding to this intent
    pub fn with_resource(mut self, name: String, resource: ResourceRef) -> Self {
        self.resource_bindings.insert(name, resource);
        self
    }
    
    /// Set location requirements for this intent
    pub fn with_location_requirements(mut self, requirements: LocationRequirements) -> Self {
        self.location_requirements = requirements;
        self
    }
    
    /// Set expected result type
    pub fn with_expected_result(mut self, result_type: TypeInner) -> Self {
        self.expected_result = Some(result_type);
        self
    }
    
    /// Set execution priority
    pub fn with_priority(mut self, priority: IntentPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set execution timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }
    
    /// Add a dependency on another intent
    pub fn with_dependency(mut self, dependency: IntentId) -> Self {
        self.dependencies.insert(dependency);
        self
    }
    
    /// Check if this intent can be executed at the given location
    pub fn can_execute_at(&self, location: &Location) -> bool {
        // Check if location is explicitly allowed
        if !self.location_requirements.allowed_locations.is_empty() {
            return self.location_requirements.allowed_locations.contains(location);
        }
        
        // If no restrictions, can execute anywhere
        true
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

impl ResourceBinding {
    /// Create a new resource binding
    pub fn new(name: &str, _resource_type: &str) -> Self {
        Self {
            name: name.to_string(),
            resource: ResourceRef::new(
                ResourceId::new([0u8; 32]), // Placeholder ID
                TypeInner::Base(crate::lambda::base::BaseType::Symbol), // Placeholder type
                Location::domain("local"),
            ),
            required: true,
            default_value: None,
        }
    }
    
    /// Set quantity for the resource binding
    pub fn with_quantity(self, _quantity: u64) -> Self {
        // Placeholder implementation - quantity would be part of resource metadata
        self
    }
}

impl TransformConstraint {
    /// Create a simple transform constraint for testing
    pub fn new(name: String) -> Self {
        TransformConstraint::LocalTransform {
            source_type: TypeInner::Base(crate::lambda::base::BaseType::Int),
            target_type: TypeInner::Base(crate::lambda::base::BaseType::Symbol),
            transform: TransformDefinition::FunctionApplication {
                function: name,
                argument: "input".to_string(),
            },
        }
    }
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentError::InvalidIntent(msg) => write!(f, "Invalid intent: {}", msg),
            IntentError::ResourceNotFound(id) => write!(f, "Resource not found: {:?}", id),
            IntentError::InsufficientCapabilities { required, available } => {
                write!(f, "Insufficient capabilities. Required: {:?}, Available: {:?}", required, available)
            }
            IntentError::LocationNotAccessible { location, reason } => {
                write!(f, "Location {:?} not accessible: {}", location, reason)
            }
            IntentError::ProtocolNotSupported { protocol, location } => {
                write!(f, "Protocol {} not supported at location {:?}", protocol, location)
            }
            IntentError::ConstraintSolvingFailed(err) => write!(f, "Constraint solving failed: {:?}", err),
            IntentError::MigrationFailed { from, to, reason } => {
                write!(f, "Migration failed from {:?} to {:?}: {}", from, to, reason)
            }
            IntentError::PerformanceConstraintViolated { constraint, actual, limit } => {
                write!(f, "Performance constraint '{}' violated: {} > {}", constraint, actual, limit)
            }
            IntentError::DependencyCycle(cycle) => write!(f, "Dependency cycle detected: {:?}", cycle),
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
    
    #[test]
    fn test_intent_creation() {
        let intent = Intent::new(Location::domain("test"));
        assert_eq!(intent.domain, Location::Domain("test".to_string()));
        assert_eq!(intent.priority, IntentPriority::Normal);
    }
    
    #[test]
    fn test_resource_binding_creation() {
        let binding = ResourceBinding::new("test_resource", "TestType");
        assert_eq!(binding.name, "test_resource");
        assert!(binding.required);
    }
    
    #[test]
    fn test_intent_location_checking() {
        let mut requirements = LocationRequirements::default();
        requirements.allowed_locations.insert(Location::domain("allowed"));
        
        let intent = Intent::new(Location::domain("test"))
            .with_location_requirements(requirements);
        
        assert!(intent.can_execute_at(&Location::domain("allowed")));
        assert!(!intent.can_execute_at(&Location::domain("forbidden")));
    }
} 