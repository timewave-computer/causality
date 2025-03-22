use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::address::Address;
use crate::resource::{ResourceId, ResourceCapability, CapabilityRef};

/// Defines the execution environment of an effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionBoundary {
    /// The effect is executed inside the system (on-chain, protected environment)
    InsideSystem,
    
    /// The effect is executed outside the system (off-chain, user environment)
    OutsideSystem,
}

/// Defines the execution environment for a specific blockchain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainBoundary {
    /// Ethereum Virtual Machine environment
    EVM(String), // Chain identifier
    
    /// CosmWasm environment
    CosmWasm(String), // Chain identifier
    
    /// Local execution environment (for testing)
    Local,
    
    /// Custom execution environment
    Custom(String),
}

/// Represents the context in which an effect is executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectContext {
    /// Unique identifier for this effect execution
    pub execution_id: Uuid,
    
    /// The execution boundary (inside or outside system)
    pub boundary: ExecutionBoundary,
    
    /// The chain boundary if relevant
    pub chain: Option<ChainBoundary>,
    
    /// The invoker of the effect
    pub invoker: Address,
    
    /// Capabilities provided to the effect
    pub capabilities: Vec<CapabilityRef>,
    
    /// Custom context parameters
    pub parameters: HashMap<String, String>,
}

impl EffectContext {
    /// Create a new effect context with the given boundary
    pub fn new(boundary: ExecutionBoundary) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            boundary,
            chain: None,
            invoker: Address::default(),
            capabilities: Vec::new(),
            parameters: HashMap::new(),
        }
    }
    
    /// Create a new effect context for inside-system execution
    pub fn new_inside(invoker: Address) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            boundary: ExecutionBoundary::InsideSystem,
            chain: None,
            invoker,
            capabilities: Vec::new(),
            parameters: HashMap::new(),
        }
    }
    
    /// Create a new effect context for outside-system execution
    pub fn new_outside(invoker: Address) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            boundary: ExecutionBoundary::OutsideSystem,
            chain: None,
            invoker,
            capabilities: Vec::new(),
            parameters: HashMap::new(),
        }
    }
    
    /// Set the chain boundary
    pub fn with_chain(mut self, chain: ChainBoundary) -> Self {
        self.chain = Some(chain);
        self
    }
    
    /// Add a capability to the context
    pub fn with_capability(mut self, capability: CapabilityRef) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add multiple capabilities to the context
    pub fn with_capabilities(mut self, capabilities: Vec<CapabilityRef>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }
    
    /// Add a parameter to the context
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
}

/// Represents data crossing a system boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCrossing<T> {
    /// The context of the boundary crossing
    pub context: EffectContext,
    
    /// The payload being transferred across the boundary
    pub payload: T,
    
    /// Authentication information for the crossing
    pub auth: BoundaryAuthentication,
    
    /// Timestamp of the crossing
    pub timestamp: u64,
}

/// Types of authentication used for boundary crossings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryAuthentication {
    /// Signature-based authentication
    Signature {
        /// The address that signed the payload
        signer: Address,
        /// The signature
        signature: Vec<u8>,
    },
    
    /// Capability-based authentication
    Capability(CapabilityRef),
    
    /// ZK proof-based authentication
    ZkProof {
        /// The proof data
        proof: Vec<u8>,
        /// Public inputs for verification
        public_inputs: Vec<Vec<u8>>,
    },
    
    /// Multi-factor authentication
    MultiAuth(Vec<BoundaryAuthentication>),
    
    /// No authentication (for internal or testing use only)
    None,
}

impl<T: Serialize> BoundaryCrossing<T> {
    /// Create a new boundary crossing from inside to outside
    pub fn new_outbound(context: EffectContext, payload: T) -> Self {
        Self {
            context,
            payload,
            auth: BoundaryAuthentication::None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Create a new boundary crossing from outside to inside
    pub fn new_inbound(context: EffectContext, payload: T) -> Self {
        Self {
            context,
            payload,
            auth: BoundaryAuthentication::None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Set the authentication method for the boundary crossing
    pub fn with_auth(mut self, auth: BoundaryAuthentication) -> Self {
        self.auth = auth;
        self
    }
    
    /// Serialize the boundary crossing for transport
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

impl<T: for<'de> Deserialize<'de>> BoundaryCrossing<T> {
    /// Deserialize a boundary crossing from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// Trait for types that can be invoked across system boundaries
pub trait BoundaryCrossingHandler<T, R> {
    /// Process an incoming boundary crossing
    fn process_inbound(&self, crossing: BoundaryCrossing<T>) -> Result<R, BoundaryError>;
    
    /// Create an outgoing boundary crossing
    fn create_outbound(&self, context: EffectContext, payload: T) -> BoundaryCrossing<T>;
}

/// Errors that can occur during boundary crossings
#[derive(Debug, thiserror::Error)]
pub enum BoundaryError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid boundary: expected {0}, got {1}")]
    InvalidBoundary(ExecutionBoundary, ExecutionBoundary),
    
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// A registry for tracking boundary crossings for auditing purposes
#[derive(Default)]
pub struct BoundaryCrossingRegistry {
    crossings: HashMap<Uuid, BoundaryCrossingRecord>,
}

/// A record of a boundary crossing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCrossingRecord {
    /// The unique ID of the crossing
    pub id: Uuid,
    
    /// The direction of the crossing
    pub direction: CrossingDirection,
    
    /// The boundary that was crossed
    pub boundary: ExecutionBoundary,
    
    /// The chain involved (if any)
    pub chain: Option<ChainBoundary>,
    
    /// The invoker of the crossing
    pub invoker: Address,
    
    /// The timestamp of the crossing
    pub timestamp: u64,
    
    /// Whether the crossing was successful
    pub success: bool,
    
    /// Error message if the crossing failed
    pub error: Option<String>,
    
    /// The type of payload
    pub payload_type: String,
}

/// The direction of a boundary crossing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossingDirection {
    /// From outside the system to inside
    Inbound,
    
    /// From inside the system to outside
    Outbound,
}

impl BoundaryCrossingRegistry {
    /// Create a new boundary crossing registry
    pub fn new() -> Self {
        Self {
            crossings: HashMap::new(),
        }
    }
    
    /// Record a boundary crossing
    pub fn record<T>(&mut self, crossing: &BoundaryCrossing<T>, direction: CrossingDirection, success: bool, error: Option<String>) 
    where
        T: std::any::Any,
    {
        let record = BoundaryCrossingRecord {
            id: crossing.context.execution_id,
            direction,
            boundary: crossing.context.boundary,
            chain: crossing.context.chain.clone(),
            invoker: crossing.context.invoker.clone(),
            timestamp: crossing.timestamp,
            success,
            error,
            payload_type: std::any::type_name::<T>().to_string(),
        };
        
        self.crossings.insert(record.id, record);
    }
    
    /// Get a boundary crossing record by ID
    pub fn get(&self, id: &Uuid) -> Option<&BoundaryCrossingRecord> {
        self.crossings.get(id)
    }
    
    /// Get all boundary crossing records
    pub fn get_all(&self) -> Vec<&BoundaryCrossingRecord> {
        self.crossings.values().collect()
    }
    
    /// Get all boundary crossing records for a specific direction
    pub fn get_by_direction(&self, direction: CrossingDirection) -> Vec<&BoundaryCrossingRecord> {
        self.crossings.values()
            .filter(|record| record.direction == direction)
            .collect()
    }
    
    /// Get all boundary crossing records for a specific invoker
    pub fn get_by_invoker(&self, invoker: &Address) -> Vec<&BoundaryCrossingRecord> {
        self.crossings.values()
            .filter(|record| record.invoker == *invoker)
            .collect()
    }
} 