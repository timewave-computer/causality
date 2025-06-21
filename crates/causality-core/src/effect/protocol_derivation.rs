//! Automatic protocol derivation from row operations
//!
//! This module implements automatic session type generation from row transformations,
//! enabling the same constraint language to work for both local field access and
//! distributed communication protocols.
//!
//! **Design Principles**:
//! - Session types automatically derived from data access patterns
//! - Protocol optimization for common patterns
//! - Multi-party protocols for distributed data
//! - Zero runtime overhead for local operations
//! - Location transparency where appropriate

use crate::{
    lambda::base::{SessionType, TypeInner, Location, BaseType},
    effect::{
        row::{RowType, FieldType, FieldAccess},
        location_row::{LocationAwareRowType, ProtocolType, GeneratedProtocol, MigrationSpec, MigrationStrategy},
    },
    system::deterministic::DeterministicSystem,
};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};

/// Protocol derivation engine that generates session types from row operations
#[derive(Debug, Clone)]
pub struct ProtocolDerivationEngine {
    /// Cache of derived protocols for optimization
    protocol_cache: BTreeMap<ProtocolCacheKey, SessionType>,
    
    /// Optimization patterns for common access scenarios
    optimization_patterns: Vec<OptimizationPattern>,
    
    /// Multi-party protocol templates
    multiparty_templates: BTreeMap<String, MultiPartyTemplate>,
}

/// Cache key for protocol derivation results
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProtocolCacheKey {
    operation_type: String,
    source_location: Location,
    target_location: Location,
    field_types: Vec<String>, // Simplified field type representation
}

/// Optimization pattern for common access scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPattern {
    /// Pattern name for identification
    pub name: String,
    
    /// Pattern description
    pub description: String,
    
    /// Field access pattern this optimizes
    pub access_pattern: AccessPattern,
    
    /// Optimized protocol for this pattern
    pub optimized_protocol: SessionType,
    
    /// Performance improvement estimate
    pub improvement_factor: u64,
}

/// Field access pattern specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Fields accessed in this pattern
    pub fields: Vec<String>,
    
    /// Access types for each field
    pub access_types: BTreeMap<String, FieldAccess>,
    
    /// Frequency of this access pattern
    pub frequency: u64,
    
    /// Whether accesses are sequential or parallel
    pub is_parallel: bool,
}

/// Template for multi-party protocol generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPartyTemplate {
    /// Template name
    pub name: String,
    
    /// Number of participants this template supports
    pub participant_count: usize,
    
    /// Role definitions for each participant
    pub roles: Vec<ParticipantRole>,
    
    /// Coordination protocol
    pub coordination_protocol: CoordinationProtocol,
}

/// Role definition for a participant in a multi-party protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantRole {
    /// Role name
    pub name: String,
    
    /// Capabilities required for this role
    pub required_capabilities: Vec<String>,
    
    /// Protocol template for this role
    pub protocol_template: ProtocolTemplate,
}

/// Template for generating participant protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolTemplate {
    /// Coordinator role - orchestrates the protocol
    Coordinator {
        coordination_steps: Vec<CoordinationStep>,
    },
    
    /// Participant role - follows coordinator instructions
    Participant {
        response_patterns: Vec<ResponsePattern>,
    },
    
    /// Peer role - equal participant in peer-to-peer protocol
    Peer {
        peer_interactions: Vec<PeerInteraction>,
    },
}

/// Coordination step in a multi-party protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStep {
    /// Broadcast a message to all participants
    Broadcast {
        message_type: TypeInner,
        target_roles: Vec<String>,
    },
    
    /// Collect responses from participants
    Collect {
        expected_responses: BTreeMap<String, TypeInner>,
        timeout_ms: Option<u64>,
    },
    
    /// Synchronization barrier
    Barrier {
        participant_roles: Vec<String>,
    },
    
    /// Conditional execution based on responses
    Conditional {
        condition: String, // Simplified condition representation
        then_steps: Vec<CoordinationStep>,
        else_steps: Vec<CoordinationStep>,
    },
}

/// Response pattern for participant roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePattern {
    /// Trigger condition for this response
    pub trigger: String,
    
    /// Response message type
    pub response_type: TypeInner,
    
    /// Whether response is required or optional
    pub required: bool,
}

/// Peer interaction in peer-to-peer protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInteraction {
    /// Interaction type
    pub interaction_type: PeerInteractionType,
    
    /// Target peer roles
    pub target_peers: Vec<String>,
    
    /// Message types exchanged
    pub message_types: Vec<TypeInner>,
}

/// Types of peer interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerInteractionType {
    /// Request-response interaction
    RequestResponse,
    
    /// Bidirectional streaming
    BidirectionalStream,
    
    /// Gossip protocol
    Gossip,
    
    /// Consensus protocol
    Consensus,
}

/// Coordination protocol for multi-party sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationProtocol {
    /// Centralized coordination with a single coordinator
    Centralized {
        coordinator_role: String,
    },
    
    /// Distributed coordination using consensus
    Distributed {
        consensus_algorithm: String,
    },
    
    /// Peer-to-peer coordination
    PeerToPeer {
        topology: NetworkTopology,
    },
}

/// Network topology for peer-to-peer coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTopology {
    /// Fully connected mesh
    FullMesh,
    
    /// Ring topology
    Ring,
    
    /// Star topology
    Star { center_role: String },
    
    /// Tree topology
    Tree { root_role: String },
}

impl ProtocolDerivationEngine {
    /// Create a new protocol derivation engine
    pub fn new() -> Self {
        let mut engine = Self {
            protocol_cache: BTreeMap::new(),
            optimization_patterns: Vec::new(),
            multiparty_templates: BTreeMap::new(),
        };
        
        // Initialize with common optimization patterns
        engine.initialize_optimization_patterns();
        engine.initialize_multiparty_templates();
        
        engine
    }
    
    /// Generate session type for field access protocol
    pub fn derive_field_access_protocol(
        &mut self,
        field_name: &str,
        field_type: &FieldType,
        source_location: &Location,
        target_location: &Location,
        _det_sys: &mut DeterministicSystem,
    ) -> Result<SessionType, ProtocolDerivationError> {
        // Check cache first
        let cache_key = ProtocolCacheKey {
            operation_type: "field_access".to_string(),
            source_location: source_location.clone(),
            target_location: target_location.clone(),
            field_types: vec![format!("{:?}", field_type.ty)],
        };
        
        if let Some(cached) = self.protocol_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Generate protocol based on field access type
        let protocol = match &field_type.access {
            FieldAccess::ReadOnly => {
                // Simple request-response for read-only access
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Field name request
                    Box::new(SessionType::Receive(
                        Box::new(field_type.ty.clone()),
                        Box::new(SessionType::End)
                    ))
                )
            }
            
            FieldAccess::WriteOnly => {
                // Send value, receive acknowledgment
                SessionType::Send(
                    Box::new(field_type.ty.clone()),
                    Box::new(SessionType::Receive(
                        Box::new(TypeInner::Base(BaseType::Bool)), // Acknowledgment
                        Box::new(SessionType::End)
                    ))
                )
            }
            
            FieldAccess::ReadWrite => {
                // Choice between read and write operations
                SessionType::InternalChoice(vec![
                    ("read".to_string(), SessionType::Send(
                        Box::new(TypeInner::Base(BaseType::Symbol)), // Field name
                        Box::new(SessionType::Receive(
                            Box::new(field_type.ty.clone()),
                            Box::new(SessionType::End)
                        ))
                    )),
                    ("write".to_string(), SessionType::Send(
                        Box::new(field_type.ty.clone()),
                        Box::new(SessionType::Receive(
                            Box::new(TypeInner::Base(BaseType::Bool)), // Acknowledgment
                            Box::new(SessionType::End)
                        ))
                    )),
                ])
            }
            
            FieldAccess::Linear => {
                // Linear access - consume the field
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Field name
                    Box::new(SessionType::Receive(
                        Box::new(field_type.ty.clone()),
                        Box::new(SessionType::End) // Field is consumed, no further access
                    ))
                )
            }
            
            FieldAccess::LocationDependent(location_access) => {
                // Generate protocol based on current location
                let access = location_access.get(source_location)
                    .unwrap_or(&FieldAccess::ReadOnly);
                
                // Recursively derive protocol for the specific access type
                let temp_field = FieldType {
                    ty: field_type.ty.clone(),
                    location: field_type.location.clone(),
                    access: access.clone(),
                };
                
                return self.derive_field_access_protocol(
                    field_name,
                    &temp_field,
                    source_location,
                    target_location,
                    _det_sys,
                );
            }
        };
        
        // Cache the result
        self.protocol_cache.insert(cache_key, protocol.clone());
        
        Ok(protocol)
    }
    
    /// Generate session type for data migration protocol
    pub fn derive_migration_protocol(
        &mut self,
        migration_spec: &MigrationSpec,
        _det_sys: &mut DeterministicSystem,
    ) -> Result<SessionType, ProtocolDerivationError> {
        let cache_key = ProtocolCacheKey {
            operation_type: "migration".to_string(),
            source_location: migration_spec.from.clone(),
            target_location: migration_spec.to.clone(),
            field_types: migration_spec.fields.clone(),
        };
        
        if let Some(cached) = self.protocol_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        let protocol = match &migration_spec.strategy {
            MigrationStrategy::Copy => {
                // Copy: send data, keep original
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Migration request
                    Box::new(SessionType::Send(
                        Box::new(TypeInner::Base(BaseType::Unit)), // Data payload (simplified)
                        Box::new(SessionType::Receive(
                            Box::new(TypeInner::Base(BaseType::Bool)), // Success confirmation
                            Box::new(SessionType::End)
                        ))
                    ))
                )
            }
            
            MigrationStrategy::Move => {
                // Move: send data, invalidate original
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Migration request
                    Box::new(SessionType::Send(
                        Box::new(TypeInner::Base(BaseType::Unit)), // Data payload
                        Box::new(SessionType::Receive(
                            Box::new(TypeInner::Base(BaseType::Bool)), // Success confirmation
                            Box::new(SessionType::Send(
                                Box::new(TypeInner::Base(BaseType::Unit)), // Invalidation signal
                                Box::new(SessionType::End)
                            ))
                        ))
                    ))
                )
            }
            
            MigrationStrategy::Replicate => {
                // Replicate: create multiple copies
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Replication request
                    Box::new(SessionType::Receive(
                        Box::new(TypeInner::Base(BaseType::Int)), // Number of replicas
                        Box::new(SessionType::Send(
                            Box::new(TypeInner::Base(BaseType::Unit)), // Data payload
                            Box::new(SessionType::Receive(
                                Box::new(TypeInner::Base(BaseType::Bool)), // Success confirmation
                                Box::new(SessionType::End)
                            ))
                        ))
                    ))
                )
            }
            
            MigrationStrategy::Partition => {
                // Partition: split data across locations
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Partition request
                    Box::new(SessionType::Receive(
                        Box::new(TypeInner::Base(BaseType::Int)), // Partition count
                        Box::new(SessionType::Send(
                            Box::new(TypeInner::Base(BaseType::Unit)), // Partition data
                            Box::new(SessionType::Receive(
                                Box::new(TypeInner::Base(BaseType::Bool)), // Success confirmation
                                Box::new(SessionType::End)
                            ))
                        ))
                    ))
                )
            }
        };
        
        self.protocol_cache.insert(cache_key, protocol.clone());
        Ok(protocol)
    }
    
    /// Generate session type for multi-party synchronization protocol
    pub fn derive_multiparty_sync_protocol(
        &mut self,
        participants: &[Location],
        coordination_type: &str,
        det_sys: &mut DeterministicSystem,
    ) -> Result<SessionType, ProtocolDerivationError> {
        // Clone the template to avoid borrowing issues
        if let Some(template) = self.multiparty_templates.get(coordination_type).cloned() {
            self.instantiate_multiparty_template(&template, participants, det_sys)
        } else {
            // Generate default consensus-based protocol
            self.generate_consensus_protocol(participants, det_sys)
        }
    }
    
    /// Optimize protocol based on access patterns
    pub fn optimize_protocol(
        &mut self,
        base_protocol: SessionType,
        access_pattern: &AccessPattern,
    ) -> Result<SessionType, ProtocolDerivationError> {
        // Look for matching optimization patterns
        for pattern in &self.optimization_patterns {
            if self.pattern_matches(&pattern.access_pattern, access_pattern) {
                return Ok(pattern.optimized_protocol.clone());
            }
        }
        
        // Apply general optimizations
        self.apply_general_optimizations(base_protocol, access_pattern)
    }
    
    /// Initialize common optimization patterns
    fn initialize_optimization_patterns(&mut self) {
        // Bulk read optimization
        self.optimization_patterns.push(OptimizationPattern {
            name: "bulk_read".to_string(),
            description: "Optimize multiple sequential reads into a single batch request".to_string(),
            access_pattern: AccessPattern {
                fields: vec!["field1".to_string(), "field2".to_string(), "field3".to_string()],
                access_types: BTreeMap::from([
                    ("field1".to_string(), FieldAccess::ReadOnly),
                    ("field2".to_string(), FieldAccess::ReadOnly),
                    ("field3".to_string(), FieldAccess::ReadOnly),
                ]),
                frequency: 100,
                is_parallel: false,
            },
            optimized_protocol: SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Symbol)), // Batch request
                Box::new(SessionType::Receive(
                    Box::new(TypeInner::Base(BaseType::Unit)), // Batch response
                    Box::new(SessionType::End)
                ))
            ),
            improvement_factor: 3, // 3x improvement over individual requests
        });
        
        // Write-through cache optimization
        self.optimization_patterns.push(OptimizationPattern {
            name: "write_through".to_string(),
            description: "Optimize write operations with immediate consistency".to_string(),
            access_pattern: AccessPattern {
                fields: vec!["cached_field".to_string()],
                access_types: BTreeMap::from([
                    ("cached_field".to_string(), FieldAccess::ReadWrite),
                ]),
                frequency: 50,
                is_parallel: false,
            },
            optimized_protocol: SessionType::Send(
                Box::new(TypeInner::Base(BaseType::Unit)), // Write data
                Box::new(SessionType::Receive(
                    Box::new(TypeInner::Base(BaseType::Bool)), // Cache confirmation
                    Box::new(SessionType::Receive(
                        Box::new(TypeInner::Base(BaseType::Bool)), // Storage confirmation
                        Box::new(SessionType::End)
                    ))
                ))
            ),
            improvement_factor: 2,
        });
    }
    
    /// Initialize multi-party protocol templates
    fn initialize_multiparty_templates(&mut self) {
        // Two-phase commit template
        self.multiparty_templates.insert("two_phase_commit".to_string(), MultiPartyTemplate {
            name: "two_phase_commit".to_string(),
            participant_count: 3, // Coordinator + 2 participants minimum
            roles: vec![
                ParticipantRole {
                    name: "coordinator".to_string(),
                    required_capabilities: vec!["coordinate".to_string(), "commit".to_string()],
                    protocol_template: ProtocolTemplate::Coordinator {
                        coordination_steps: vec![
                            CoordinationStep::Broadcast {
                                message_type: TypeInner::Base(BaseType::Symbol), // Prepare message
                                target_roles: vec!["participant".to_string()],
                            },
                            CoordinationStep::Collect {
                                expected_responses: BTreeMap::from([
                                    ("participant".to_string(), TypeInner::Base(BaseType::Bool))
                                ]),
                                timeout_ms: Some(5000),
                            },
                            CoordinationStep::Conditional {
                                condition: "all_prepared".to_string(),
                                then_steps: vec![
                                    CoordinationStep::Broadcast {
                                        message_type: TypeInner::Base(BaseType::Symbol), // Commit message
                                        target_roles: vec!["participant".to_string()],
                                    }
                                ],
                                else_steps: vec![
                                    CoordinationStep::Broadcast {
                                        message_type: TypeInner::Base(BaseType::Symbol), // Abort message
                                        target_roles: vec!["participant".to_string()],
                                    }
                                ],
                            },
                        ],
                    },
                },
                ParticipantRole {
                    name: "participant".to_string(),
                    required_capabilities: vec!["prepare".to_string(), "commit".to_string()],
                    protocol_template: ProtocolTemplate::Participant {
                        response_patterns: vec![
                            ResponsePattern {
                                trigger: "prepare_request".to_string(),
                                response_type: TypeInner::Base(BaseType::Bool),
                                required: true,
                            },
                            ResponsePattern {
                                trigger: "commit_request".to_string(),
                                response_type: TypeInner::Base(BaseType::Bool),
                                required: true,
                            },
                        ],
                    },
                },
            ],
            coordination_protocol: CoordinationProtocol::Centralized {
                coordinator_role: "coordinator".to_string(),
            },
        });
        
        // Gossip protocol template
        self.multiparty_templates.insert("gossip".to_string(), MultiPartyTemplate {
            name: "gossip".to_string(),
            participant_count: 5, // Minimum for effective gossip
            roles: vec![
                ParticipantRole {
                    name: "peer".to_string(),
                    required_capabilities: vec!["gossip".to_string(), "relay".to_string()],
                    protocol_template: ProtocolTemplate::Peer {
                        peer_interactions: vec![
                            PeerInteraction {
                                interaction_type: PeerInteractionType::Gossip,
                                target_peers: vec!["peer".to_string()],
                                message_types: vec![TypeInner::Base(BaseType::Unit)],
                            },
                        ],
                    },
                },
            ],
            coordination_protocol: CoordinationProtocol::PeerToPeer {
                topology: NetworkTopology::FullMesh,
            },
        });
    }
    
    /// Instantiate a multi-party template for specific participants
    fn instantiate_multiparty_template(
        &mut self,
        template: &MultiPartyTemplate,
        participants: &[Location],
        det_sys: &mut DeterministicSystem,
    ) -> Result<SessionType, ProtocolDerivationError> {
        if participants.len() < template.participant_count {
            return Err(ProtocolDerivationError::InsufficientParticipants {
                required: template.participant_count,
                provided: participants.len(),
            });
        }
        
        // For simplicity, generate a basic coordination protocol
        // In a full implementation, this would be more sophisticated
        match &template.coordination_protocol {
            CoordinationProtocol::Centralized { coordinator_role: _ } => {
                // Generate centralized protocol
                Ok(SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Symbol)), // Coordination request
                    Box::new(SessionType::Receive(
                        Box::new(TypeInner::Base(BaseType::Bool)), // Coordination response
                        Box::new(SessionType::End)
                    ))
                ))
            }
            
            CoordinationProtocol::Distributed { consensus_algorithm: _ } => {
                // Generate distributed consensus protocol
                self.generate_consensus_protocol(participants, det_sys)
            }
            
            CoordinationProtocol::PeerToPeer { topology: _ } => {
                // Generate peer-to-peer protocol
                Ok(SessionType::InternalChoice(vec![
                    ("send".to_string(), SessionType::Send(
                        Box::new(TypeInner::Base(BaseType::Unit)),
                        Box::new(SessionType::End)
                    )),
                    ("receive".to_string(), SessionType::Receive(
                        Box::new(TypeInner::Base(BaseType::Unit)),
                        Box::new(SessionType::End)
                    )),
                ]))
            }
        }
    }
    
    /// Generate a consensus protocol for multiple participants
    fn generate_consensus_protocol(
        &mut self,
        participants: &[Location],
        _det_sys: &mut DeterministicSystem,
    ) -> Result<SessionType, ProtocolDerivationError> {
        // Simple majority consensus protocol
        let _majority_threshold = (participants.len() / 2) + 1;
        
        Ok(SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Symbol)), // Proposal
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Int)), // Vote count
                Box::new(SessionType::InternalChoice(vec![
                    ("accept".to_string(), SessionType::Send(
                        Box::new(TypeInner::Base(BaseType::Bool)), // Accept decision
                        Box::new(SessionType::End)
                    )),
                    ("reject".to_string(), SessionType::Send(
                        Box::new(TypeInner::Base(BaseType::Bool)), // Reject decision
                        Box::new(SessionType::End)
                    )),
                ]))
            ))
        ))
    }
    
    /// Check if an optimization pattern matches an access pattern
    fn pattern_matches(&self, optimization_pattern: &AccessPattern, access_pattern: &AccessPattern) -> bool {
        // Simple pattern matching - in practice this would be more sophisticated
        optimization_pattern.fields.len() == access_pattern.fields.len() &&
        optimization_pattern.is_parallel == access_pattern.is_parallel &&
        optimization_pattern.access_types.len() == access_pattern.access_types.len()
    }
    
    /// Apply general protocol optimizations
    fn apply_general_optimizations(
        &mut self,
        base_protocol: SessionType,
        access_pattern: &AccessPattern,
    ) -> Result<SessionType, ProtocolDerivationError> {
        // Apply batching optimization for multiple fields
        if access_pattern.fields.len() > 1 && !access_pattern.is_parallel {
            return Ok(self.apply_batching_optimization(base_protocol));
        }
        
        // Apply pipelining optimization for high-frequency access
        if access_pattern.frequency > 100 {
            return Ok(self.apply_pipelining_optimization(base_protocol));
        }
        
        // Return base protocol if no optimizations apply
        Ok(base_protocol)
    }
    
    /// Apply batching optimization to reduce round trips
    fn apply_batching_optimization(&self, _base_protocol: SessionType) -> SessionType {
        // Simplified batching - combine multiple operations into one
        SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Symbol)), // Batch request
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Unit)), // Batch response
                Box::new(SessionType::End)
            ))
        )
    }
    
    /// Apply pipelining optimization for high-frequency operations
    fn apply_pipelining_optimization(&self, base_protocol: SessionType) -> SessionType {
        // Simplified pipelining - allow multiple concurrent operations
        SessionType::InternalChoice(vec![
            ("pipeline".to_string(), SessionType::Recursive(
                "pipeline_loop".to_string(),
                Box::new(SessionType::InternalChoice(vec![
                    ("continue".to_string(), base_protocol.clone()),
                    ("end".to_string(), SessionType::End),
                ]))
            )),
            ("single".to_string(), base_protocol),
        ])
    }
}

/// Errors that can occur during protocol derivation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolDerivationError {
    /// Unsupported field access type
    UnsupportedAccessType(String),
    
    /// Invalid migration strategy
    InvalidMigrationStrategy(String),
    
    /// Insufficient participants for multi-party protocol
    InsufficientParticipants {
        required: usize,
        provided: usize,
    },
    
    /// Protocol optimization failed
    OptimizationFailed(String),
    
    /// Template instantiation failed
    TemplateInstantiationFailed(String),
    
    /// Invalid coordination protocol
    InvalidCoordinationProtocol(String),
}

impl std::fmt::Display for ProtocolDerivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolDerivationError::UnsupportedAccessType(access_type) => {
                write!(f, "Unsupported field access type: {}", access_type)
            }
            ProtocolDerivationError::InvalidMigrationStrategy(strategy) => {
                write!(f, "Invalid migration strategy: {}", strategy)
            }
            ProtocolDerivationError::InsufficientParticipants { required, provided } => {
                write!(f, "Insufficient participants: required {}, provided {}", required, provided)
            }
            ProtocolDerivationError::OptimizationFailed(reason) => {
                write!(f, "Protocol optimization failed: {}", reason)
            }
            ProtocolDerivationError::TemplateInstantiationFailed(reason) => {
                write!(f, "Template instantiation failed: {}", reason)
            }
            ProtocolDerivationError::InvalidCoordinationProtocol(protocol) => {
                write!(f, "Invalid coordination protocol: {}", protocol)
            }
        }
    }
}

impl std::error::Error for ProtocolDerivationError {}

impl Default for ProtocolDerivationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::BaseType;
    
    #[test]
    fn test_field_access_protocol_derivation() {
        let mut engine = ProtocolDerivationEngine::new();
        let mut det_sys = DeterministicSystem::new();
        
        let field_type = FieldType {
            ty: TypeInner::Base(BaseType::Int),
            location: Some(Location::Local),
            access: FieldAccess::ReadOnly,
        };
        
        let protocol = engine.derive_field_access_protocol(
            "test_field",
            &field_type,
            &Location::Local,
            &Location::Remote("server".to_string()),
            &mut det_sys,
        ).unwrap();
        
        // Should generate a request-response protocol for read-only access
        match protocol {
            SessionType::Send(_, continuation) => {
                match *continuation {
                    SessionType::Receive(_, end) => {
                        assert_eq!(*end, SessionType::End);
                    }
                    _ => panic!("Expected Receive continuation"),
                }
            }
            _ => panic!("Expected Send protocol"),
        }
    }
    
    #[test]
    fn test_migration_protocol_derivation() {
        let mut engine = ProtocolDerivationEngine::new();
        let mut det_sys = DeterministicSystem::new();
        
        let migration_spec = MigrationSpec {
            from: Location::Local,
            to: Location::Remote("backup".to_string()),
            fields: vec!["data".to_string()],
            strategy: MigrationStrategy::Move,
            protocol: TypeInner::Base(BaseType::Unit), // Placeholder
        };
        
        let protocol = engine.derive_migration_protocol(&migration_spec, &mut det_sys).unwrap();
        
        // Should generate a protocol appropriate for move strategy
        match protocol {
            SessionType::Send(_, _) => {
                // Move strategy should include invalidation step
                // Detailed verification would check the full protocol structure
            }
            _ => panic!("Expected Send protocol for migration"),
        }
    }
    
    #[test]
    fn test_multiparty_sync_protocol() {
        let mut engine = ProtocolDerivationEngine::new();
        let mut det_sys = DeterministicSystem::new();
        
        let participants = vec![
            Location::Local,
            Location::Remote("node1".to_string()),
            Location::Remote("node2".to_string()),
        ];
        
        let protocol = engine.derive_multiparty_sync_protocol(
            &participants,
            "two_phase_commit",
            &mut det_sys,
        ).unwrap();
        
        // Should generate a coordination protocol
        match protocol {
            SessionType::Send(_, _) => {
                // Two-phase commit should start with coordination request
            }
            _ => panic!("Expected coordination protocol"),
        }
    }
    
    #[test]
    fn test_protocol_optimization() {
        let mut engine = ProtocolDerivationEngine::new();
        
        let base_protocol = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Symbol)),
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(SessionType::End)
            ))
        );
        
        let access_pattern = AccessPattern {
            fields: vec!["field1".to_string(), "field2".to_string()],
            access_types: BTreeMap::new(),
            frequency: 150, // High frequency
            is_parallel: false,
        };
        
        let optimized = engine.optimize_protocol(base_protocol, &access_pattern).unwrap();
        
        // Should apply pipelining optimization for high-frequency access
        match optimized {
            SessionType::InternalChoice(_) => {
                // Pipelining creates choice between pipeline and single operation
            }
            _ => panic!("Expected optimized protocol with choices"),
        }
    }
} 