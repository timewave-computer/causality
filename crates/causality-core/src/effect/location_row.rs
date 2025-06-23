//! Location-aware row types for unified computation and communication
//!
//! This module extends Causality's powerful row type constraint system to handle
//! location-aware data structures, enabling the same constraint language to work
//! for both local field access and distributed communication.
//!
//! **Design Principles**:
//! - Row operations preserve location information
//! - Same constraint language for local and distributed operations
//! - Zero runtime overhead maintained for local operations
//! - Location transparency achieved where appropriate
//! - Automatic protocol generation from data access patterns

use crate::{
    lambda::base::{Location, TypeInner},
    effect::row::{RowType, RowConstraint, FieldType},
    machine::instruction::RegisterId,
};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// Location-aware row type that tracks where data resides
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationAwareRowType {
    /// Data stored locally with standard row operations
    Local {
        row: RowType,
        register: Option<RegisterId>,
    },
    
    /// Data stored remotely requiring communication protocols
    Remote {
        row: RowType,
        location: Location,
        access_protocol: Option<AccessProtocol>,
    },
    
    /// Data distributed across multiple locations
    Distributed {
        row: RowType,
        locations: BTreeMap<String, Location>, // field -> location mapping
        sync_protocol: Option<SyncProtocol>,
    },
    
    /// Data that can move between locations
    Portable {
        row: RowType,
        current_location: Location,
        migration_history: Vec<Location>,
    },
}

/// Protocol specification for remote data access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessProtocol {
    /// Session type for the access protocol
    pub session_type: TypeInner,
    
    /// Required capabilities for access
    pub required_capabilities: Vec<String>,
    
    /// Estimated latency for access operations
    pub estimated_latency: u64,
    
    /// Whether the protocol supports atomic operations
    pub supports_atomicity: bool,
}

/// Protocol specification for distributed data synchronization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncProtocol {
    /// Consistency model (eventual, strong, etc.)
    pub consistency_model: ConsistencyModel,
    
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    
    /// Locations that must participate in sync
    pub participants: Vec<Location>,
    
    /// Session type for the sync protocol
    pub session_type: TypeInner,
}

/// Data consistency models for distributed operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyModel {
    /// All reads receive the most recent write
    Strong,
    
    /// Reads may return stale data, eventual consistency
    Eventual,
    
    /// Reads are consistent within a session
    Session,
    
    /// Monotonic read consistency
    MonotonicRead,
    
    /// Causal consistency (respects causality)
    Causal,
}

/// Conflict resolution strategies for distributed updates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last writer wins
    LastWriterWins,
    
    /// First writer wins
    FirstWriterWins,
    
    /// Application-specific merge function
    CustomMerge(String),
    
    /// Require explicit conflict resolution
    Manual,
    
    /// Vector clock based resolution
    VectorClock,
}

/// Result of a row operation that may involve location changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowOpResult {
    /// The resulting row type after the operation
    pub result_row: LocationAwareRowType,
    
    /// Any generated protocols for distributed operations
    pub generated_protocols: Vec<GeneratedProtocol>,
    
    /// Migration specifications if data needs to move
    pub migrations: Vec<MigrationSpec>,
    
    /// Constraints that must be satisfied
    pub constraints: Vec<LocationConstraint>,
}

/// Protocol automatically generated from row operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedProtocol {
    /// Type of protocol generated
    pub protocol_type: ProtocolType,
    
    /// Session type for the protocol
    pub session_type: TypeInner,
    
    /// Locations involved in the protocol
    pub participants: Vec<Location>,
    
    /// Estimated cost/latency
    pub cost_estimate: u64,
}

/// Types of protocols that can be generated
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolType {
    /// Simple field access protocol
    FieldAccess,
    
    /// Atomic update protocol
    AtomicUpdate,
    
    /// Multi-field transaction protocol
    Transaction,
    
    /// Data migration protocol
    Migration,
    
    /// Synchronization protocol
    Synchronization,
}

/// Specification for data migration between locations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationSpec {
    /// Source location
    pub from: Location,
    
    /// Destination location
    pub to: Location,
    
    /// Fields to migrate
    pub fields: Vec<String>,
    
    /// Migration strategy
    pub strategy: MigrationStrategy,
    
    /// Session type for the migration protocol
    pub protocol: TypeInner,
}

/// Strategies for data migration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    /// Copy data and leave original
    Copy,
    
    /// Move data (linear transfer)
    Move,
    
    /// Replicate data across locations
    Replicate,
    
    /// Partition data across locations
    Partition,
}

/// Location-aware constraints for row operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationConstraint {
    /// The constraint on the row operation
    pub row_constraint: RowConstraint,
    
    /// Location requirements
    pub location_requirements: LocationRequirement,
    
    /// Performance requirements
    pub performance_requirements: PerformanceRequirement,
}

/// Requirements for where operations can be performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationRequirement {
    /// Must be performed at specific location
    MustBeAt(Location),
    
    /// Can be performed at any of these locations
    AnyOf(Vec<Location>),
    
    /// Must be performed locally
    LocalOnly,
    
    /// Can be performed anywhere
    LocationAgnostic,
    
    /// Must be co-located with other data
    CoLocatedWith(Vec<String>), // field names
}

/// Performance requirements for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerformanceRequirement {
    /// Maximum acceptable latency (in milliseconds)
    pub max_latency: Option<u64>,
    
    /// Minimum required bandwidth
    pub min_bandwidth: Option<u64>,
    
    /// Consistency requirements
    pub consistency: ConsistencyModel,
    
    /// Whether operation must be atomic
    pub atomic: bool,
}

impl LocationAwareRowType {
    /// Create a new local row type
    pub fn local(row: RowType) -> Self {
        LocationAwareRowType::Local {
            row,
            register: None,
        }
    }
    
    /// Create a new remote row type
    pub fn remote(row: RowType, location: Location) -> Self {
        LocationAwareRowType::Remote {
            row,
            location,
            access_protocol: None,
        }
    }
    
    /// Create a new distributed row type
    pub fn distributed(row: RowType, locations: BTreeMap<String, Location>) -> Self {
        LocationAwareRowType::Distributed {
            row,
            locations,
            sync_protocol: None,
        }
    }
    
    /// Create a new portable row type
    pub fn portable(row: RowType, current_location: Location) -> Self {
        LocationAwareRowType::Portable {
            row,
            current_location,
            migration_history: Vec::new(),
        }
    }
    
    /// Get the underlying row type
    pub fn row_type(&self) -> &RowType {
        match self {
            LocationAwareRowType::Local { row, .. } => row,
            LocationAwareRowType::Remote { row, .. } => row,
            LocationAwareRowType::Distributed { row, .. } => row,
            LocationAwareRowType::Portable { row, .. } => row,
        }
    }
    
    /// Get the primary location for this row
    pub fn primary_location(&self) -> Location {
        match self {
            LocationAwareRowType::Local { .. } => Location::Local,
            LocationAwareRowType::Remote { location, .. } => location.clone(),
            LocationAwareRowType::Distributed { locations, .. } => {
                // Return the location with the most fields, or first one
                locations.values().next().cloned().unwrap_or(Location::Local)
            }
            LocationAwareRowType::Portable { current_location, .. } => current_location.clone(),
        }
    }
    
    /// Check if this row type is location-agnostic
    pub fn is_location_agnostic(&self) -> bool {
        matches!(self, LocationAwareRowType::Portable { .. })
    }
    
    /// Check if this row type is local
    pub fn is_local(&self) -> bool {
        matches!(self, LocationAwareRowType::Local { .. })
    }
    
    /// Check if this row type is distributed
    pub fn is_distributed(&self) -> bool {
        matches!(self, LocationAwareRowType::Distributed { .. })
    }
    
    /// Project a field locally (zero runtime overhead for local data)
    pub fn project_local(&self, field: &str) -> Result<RowOpResult, LocationRowError> {
        match self {
            LocationAwareRowType::Local { row, register } => {
                // Standard local row projection - zero runtime overhead
                let field_type = row.get_field(field)
                    .ok_or_else(|| LocationRowError::FieldNotFound(field.to_string()))?;
                
                let result_row = LocationAwareRowType::Local {
                    row: RowType::singleton(field.to_string(), field_type.clone()),
                    register: *register,
                };
                
                Ok(RowOpResult {
                    result_row,
                    generated_protocols: Vec::new(),
                    migrations: Vec::new(),
                    constraints: Vec::new(),
                })
            }
            _ => Err(LocationRowError::NotLocal),
        }
    }
    
    /// Project a field remotely (generates communication protocol)
    pub fn project_remote(&self, field: &str, target: Location) -> Result<RowOpResult, LocationRowError> {
        match self {
            LocationAwareRowType::Remote { row, location, .. } => {
                let field_type = row.get_field(field)
                    .ok_or_else(|| LocationRowError::FieldNotFound(field.to_string()))?;
                
                // Generate protocol for remote field access
                let protocol = GeneratedProtocol {
                    protocol_type: ProtocolType::FieldAccess,
                    session_type: self.create_field_access_protocol(field, field_type)?,
                    participants: vec![location.clone(), target],
                    cost_estimate: 100, // Simplified cost estimate
                };
                
                let result_row = LocationAwareRowType::Remote {
                    row: RowType::singleton(field.to_string(), field_type.clone()),
                    location: location.clone(),
                    access_protocol: None,
                };
                
                Ok(RowOpResult {
                    result_row,
                    generated_protocols: vec![protocol],
                    migrations: Vec::new(),
                    constraints: Vec::new(),
                })
            }
            _ => Err(LocationRowError::NotRemote),
        }
    }
    
    /// Migrate data between locations
    pub fn migrate(&self, from: Location, to: Location) -> Result<RowOpResult, LocationRowError> {
        let migration = MigrationSpec {
            from: from.clone(),
            to: to.clone(),
            fields: self.row_type().field_names(),
            strategy: MigrationStrategy::Move, // Default to move for linear semantics
            protocol: self.create_communication_protocol(&from, &to)?,
        };
        
        let result_row = match self {
            LocationAwareRowType::Local { row, .. } => {
                LocationAwareRowType::Remote {
                    row: row.clone(),
                    location: to,
                    access_protocol: None,
                }
            }
            LocationAwareRowType::Remote { row, .. } => {
                LocationAwareRowType::Remote {
                    row: row.clone(),
                    location: to,
                    access_protocol: None,
                }
            }
            LocationAwareRowType::Portable { row, migration_history, .. } => {
                let mut new_history = migration_history.clone();
                new_history.push(from);
                LocationAwareRowType::Portable {
                    row: row.clone(),
                    current_location: to,
                    migration_history: new_history,
                }
            }
            LocationAwareRowType::Distributed { .. } => {
                return Err(LocationRowError::CannotMigrateDistributed);
            }
        };
        
        Ok(RowOpResult {
            result_row,
            generated_protocols: Vec::new(),
            migrations: vec![migration],
            constraints: Vec::new(),
        })
    }
    
    /// Perform distributed update across multiple locations
    pub fn distributed_update(&self, field: &str, locations: Vec<Location>) -> Result<RowOpResult, LocationRowError> {
        let field_type = self.row_type().get_field(field)
            .ok_or_else(|| LocationRowError::FieldNotFound(field.to_string()))?;
        
        // Generate synchronization protocol
        let sync_protocol = SyncProtocol {
            consistency_model: ConsistencyModel::Strong, // Default to strong consistency
            conflict_resolution: ConflictResolution::LastWriterWins,
            participants: locations.clone(),
            session_type: self.create_distributed_field_access_protocol(field, field_type, &locations)?,
        };
        
        let protocol = GeneratedProtocol {
            protocol_type: ProtocolType::Synchronization,
            session_type: sync_protocol.session_type.clone(),
            participants: locations.clone(),
            cost_estimate: locations.len() as u64 * 50, // Cost scales with participants
        };
        
        let mut location_map = BTreeMap::new();
        for (i, location) in locations.iter().enumerate() {
            location_map.insert(format!("replica_{}", i), location.clone());
        }
        
        let result_row = LocationAwareRowType::Distributed {
            row: self.row_type().clone(),
            locations: location_map,
            sync_protocol: Some(sync_protocol),
        };
        
        Ok(RowOpResult {
            result_row,
            generated_protocols: vec![protocol],
            migrations: Vec::new(),
            constraints: Vec::new(),
        })
    }
    
    /// Create a field access protocol
    pub fn create_field_access_protocol(&self, _field: &str, _field_type: &FieldType) -> Result<TypeInner, LocationRowError> {
        // For now, return a simple unit type
        Ok(TypeInner::Base(crate::lambda::BaseType::Unit))
    }
    
    /// Create a communication protocol between locations
    pub fn create_communication_protocol(&self, _from: &Location, _to: &Location) -> Result<TypeInner, LocationRowError> {
        // For now, return a simple unit type
        Ok(TypeInner::Base(crate::lambda::BaseType::Unit))
    }
    
    /// Create a distributed field access protocol
    pub fn create_distributed_field_access_protocol(&self, _field: &str, _field_type: &FieldType, _locations: &[Location]) -> Result<TypeInner, LocationRowError> {
        // Implementation needed
        Err(LocationRowError::ProtocolGenerationFailed("Distributed field access protocol generation not implemented".to_string()))
    }
}

/// Errors that can occur in location-aware row operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationRowError {
    /// Field not found in row type
    FieldNotFound(String),
    
    /// Operation requires local data but data is remote
    NotLocal,
    
    /// Operation requires remote data but data is local
    NotRemote,
    
    /// Cannot migrate distributed data
    CannotMigrateDistributed,
    
    /// Location not available
    LocationUnavailable(Location),
    
    /// Protocol generation failed
    ProtocolGenerationFailed(String),
    
    /// Consistency violation
    ConsistencyViolation(String),
    
    /// Permission denied for location access
    PermissionDenied(Location),
}

impl std::fmt::Display for LocationRowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationRowError::FieldNotFound(field) => write!(f, "Field '{}' not found", field),
            LocationRowError::NotLocal => write!(f, "Operation requires local data"),
            LocationRowError::NotRemote => write!(f, "Operation requires remote data"),
            LocationRowError::CannotMigrateDistributed => write!(f, "Cannot migrate distributed data"),
            LocationRowError::LocationUnavailable(loc) => write!(f, "Location {:?} unavailable", loc),
            LocationRowError::ProtocolGenerationFailed(msg) => write!(f, "Protocol generation failed: {}", msg),
            LocationRowError::ConsistencyViolation(msg) => write!(f, "Consistency violation: {}", msg),
            LocationRowError::PermissionDenied(loc) => write!(f, "Permission denied for location {:?}", loc),
        }
    }
}

impl std::error::Error for LocationRowError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::row::{RowType, FieldType};
    
    #[test]
    fn test_local_row_projection() {
        let mut row = RowType::empty();
        row.add_field("name".to_string(), FieldType::simple(TypeInner::Base(crate::lambda::BaseType::Symbol)));
        row.add_field("age".to_string(), FieldType::simple(TypeInner::Base(crate::lambda::BaseType::Int)));
        
        let location_row = LocationAwareRowType::local(row);
        
        let result = location_row.project_local("name").unwrap();
        assert!(result.generated_protocols.is_empty());
        assert!(result.migrations.is_empty());
        assert_eq!(result.result_row.row_type().field_names(), vec!["name"]);
    }
    
    #[test]
    fn test_remote_row_projection() {
        let mut row = RowType::empty();
        row.add_field("data".to_string(), FieldType::simple(TypeInner::Base(crate::lambda::BaseType::Symbol)));
        
        let remote_location = Location::Remote("server1".to_string());
        let location_row = LocationAwareRowType::remote(row, remote_location.clone());
        
        let target = Location::Local;
        let result = location_row.project_remote("data", target).unwrap();
        
        assert_eq!(result.generated_protocols.len(), 1);
        assert_eq!(result.generated_protocols[0].protocol_type, ProtocolType::FieldAccess);
        assert_eq!(result.generated_protocols[0].participants.len(), 2);
    }
    
    #[test]
    fn test_data_migration() {
        let mut row = RowType::empty();
        row.add_field("value".to_string(), FieldType::simple(TypeInner::Base(crate::lambda::BaseType::Int)));
        
        let location_row = LocationAwareRowType::local(row);
        let target = Location::Remote("backup".to_string());
        
        let result = location_row.migrate(Location::Local, target.clone()).unwrap();
        
        assert_eq!(result.migrations.len(), 1);
        assert_eq!(result.migrations[0].to, target);
        assert_eq!(result.migrations[0].strategy, MigrationStrategy::Move);
    }
} 