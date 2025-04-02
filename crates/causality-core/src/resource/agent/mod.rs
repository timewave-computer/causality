// mod.rs - Agent resource module
//
// This module defines the agent resource system, which manages agents within
// the system. Agents are entities that can perform operations on resources
// and have capabilities that determine what they're allowed to do.

// Module definitions
pub mod types;
pub mod agent;
pub mod operation;
pub mod authorization;
pub mod registry;
pub mod service;
pub mod user;
pub mod committee;
pub mod operator;
pub mod obligation;
pub mod messaging;

// Re-export important types
pub use types::{
    AgentId,
    AgentType,
    AgentState,
    AgentRelationship,
    RelationshipType,
    AgentError,
    // AgentResult
};

pub use agent::{
    Agent,
    AgentImpl,
    // AgentQuery,
    AgentBuilder,
};

pub use operation::{
    Operation,
    OperationId,
    OperationType,
    // OperationStatus,
    OperationResult,
    OperationError,
    // OperationBatch,
    // OperationRequest,
    // OperationResponse
};

pub use authorization::{
    Authorization,
    // AuthorizationId,
    // AuthorizationType,
    // AuthorizationStatus,
    // AuthorizationResult,
    AuthorizationError,
    // AuthorizationManager,
    // AuthorizationScope
};

pub use registry::{
    AgentRegistry,
    AgentRegistryError,
    // AgentRegistryResult,
    // AgentRegistration,
    // AgentRegistrationBuilder,
    // RegistrationStatus,
    // RegistryConfig
};

// Re-export specialized agent types
pub use user::{
    UserAgent,
    UserAgentBuilder,
    UserProfile,
    // UserRole,
    // UserStatus,
    // UserPreferences,
    UserAgentError
};

pub use committee::{
    CommitteeAgent,
    CommitteeAgentBuilder,
    CommitteeMember,
    // CommitteePolicy,
    CommitteeDecision,
    // CommitteeVote,
    // CommitteeVoteType,
    // CommitteeStatus,
    CommitteeAgentError
};

// Re-export operator types (commenting out missing types)
pub use operator::{
    OperatorAgent,
    OperatorAgentBuilder,
    // OperatorRole,
    // SystemOperation,
    // SystemOperationType,
    // SystemOperationStatus,
    // SystemOperationResult,
    // MaintenanceWindow,
    OperatorAgentError,
};

// Re-export service types
pub use service::{
    ServiceStatus,
    ServiceStatusBuilder,
    ServiceState,
    ServiceVersion,
    ServiceStatusManager,
    ServiceStatusResult,
    ServiceStatusError,
    ServiceAdvertisement
};

// Re-export obligation types
pub use obligation::{
    Obligation,
    ObligationId,
    ObligationType,
    ObligationStatus,
    ObligationManager,
    ObligationError,
    ObligationResult,
    ObligationSummary,
    CapabilityObligation,
    ObligationEffect,
    ObligationEffectType
};

// Re-export messaging types
pub use messaging::{
    Message,
    MessageId,
    MessageType,
    MessageFormat,
    MessagePriority,
    MessageSecurityLevel,
    MessageDeliveryStatus,
    MessageFactory,
    MessageRouter,
    MessagingError,
    MessagingResult,
    Messaging,
    MessageEffect,
    MessageEffectType
};

/// Agent resource system
///
/// The agent module is responsible for managing agents within the system.
/// An agent is an entity capable of performing operations on resources and
/// having capabilities that determine what they're allowed to do.
///
/// # Key components
///
/// * **Core types**: Basic types for agent identification and state management
/// * **Agent trait**: Provides the interface for all agent implementations
/// * **Operation system**: Handles capability-checked resource operations
/// * **Authorization system**: Verifies capabilities before operations
/// * **Agent registry**: Manages agent registration and lookup
/// * **Service status**: Allows agents to advertise services to the system
/// * **Specialized agents**: Implementation of specific agent types:
///   * User agents: Represent human users of the system
///   * Committee agents: Represent multi-agent decision making bodies
///   * Operator agents: Represent system admins/operators
/// * **Obligation manager**: Tracks and enforces capability obligations
/// * **Messaging system**: Enables secure agent-to-agent communication
///
/// This module is integrated with the resource and capability systems to
/// provide a unified approach to agent management and authentication.
#[doc(hidden)]
pub struct _Documentation; 