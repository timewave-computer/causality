use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use super::annotation::{BoundaryType, CrossingType, BoundarySafe};
use super::metrics;

/// Errors that can occur during boundary crossings
#[derive(Debug, Error)]
pub enum BoundaryCrossingError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Size limit exceeded")]
    SizeLimitExceeded,
    
    #[error("Invalid boundary crossing: {0}")]
    InvalidCrossing(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for boundary crossing operations
pub type BoundaryCrossingResult<T> = Result<T, BoundaryCrossingError>;

/// Authentication methods for boundary crossings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryAuthentication {
    /// Signature-based authentication
    Signature(String),
    
    /// Capability-based authentication
    Capability(String),
    
    /// Zero-knowledge proof authentication
    ZkProof(Vec<u8>),
    
    /// Multi-factor authentication
    MultiFactor(Vec<String>),
    
    /// No authentication
    None,
}

/// Payload for a boundary crossing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCrossingPayload {
    /// Unique identifier for this crossing
    pub crossing_id: String,
    
    /// The source boundary
    pub source_boundary: String,
    
    /// The destination boundary
    pub destination_boundary: String,
    
    /// The crossing type
    pub crossing_type: String,
    
    /// The payload data
    pub data: Vec<u8>,
    
    /// Authentication information
    pub authentication: BoundaryAuthentication,
    
    /// Additional context information
    pub context: HashMap<String, String>,
    
    /// Timestamp of the crossing
    pub timestamp: u64,
    
    /// Size of the payload in bytes
    pub size: usize,
}

impl BoundaryCrossingPayload {
    /// Create a new payload
    pub fn new(
        source: BoundaryType,
        destination: BoundaryType,
        crossing_type: CrossingType,
        data: Vec<u8>,
        auth: BoundaryAuthentication,
    ) -> Self {
        let crossing_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            crossing_id,
            source_boundary: source.to_string(),
            destination_boundary: destination.to_string(),
            crossing_type: crossing_type.to_string(),
            data,
            authentication: auth,
            context: HashMap::new(),
            timestamp: now,
            size: data.len(),
        }
    }
    
    /// Add context information to the payload
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

/// Protocol for crossing boundaries
pub trait BoundaryCrossingProtocol: Send + Sync + 'static {
    /// Get the name of the protocol
    fn name(&self) -> &str;
    
    /// Get the source boundary type
    fn source_boundary(&self) -> BoundaryType;
    
    /// Get the destination boundary type
    fn destination_boundary(&self) -> BoundaryType;
    
    /// Verify the authentication for a boundary crossing
    fn verify_authentication(
        &self,
        payload: &BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<bool>;
    
    /// Process an incoming boundary crossing
    fn process_incoming(
        &self,
        payload: BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<Vec<u8>>;
    
    /// Prepare an outgoing boundary crossing
    fn prepare_outgoing<T: BoundarySafe>(
        &self,
        data: &T,
        auth: BoundaryAuthentication,
    ) -> BoundaryCrossingResult<BoundaryCrossingPayload>;
}

/// Default implementation of a boundary crossing protocol
pub struct DefaultBoundaryCrossingProtocol {
    name: String,
    source: BoundaryType,
    destination: BoundaryType,
    crossing_type: CrossingType,
    rate_limiter: Arc<RwLock<HashMap<String, u64>>>,
    size_limit: usize,
}

impl DefaultBoundaryCrossingProtocol {
    /// Create a new protocol instance
    pub fn new(
        name: &str,
        source: BoundaryType,
        destination: BoundaryType,
        size_limit: usize,
    ) -> Self {
        let crossing_type = if source == BoundaryType::InsideSystem && destination == BoundaryType::OutsideSystem {
            CrossingType::InsideToOutside
        } else if source == BoundaryType::OutsideSystem && destination == BoundaryType::InsideSystem {
            CrossingType::OutsideToInside
        } else if source == BoundaryType::OffChain && destination == BoundaryType::OnChain {
            CrossingType::OffChainToOnChain
        } else if source == BoundaryType::OnChain && destination == BoundaryType::OffChain {
            CrossingType::OnChainToOffChain
        } else {
            CrossingType::Custom(format!("{}_{}", source, destination))
        };
        
        Self {
            name: name.to_string(),
            source,
            destination,
            crossing_type,
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            size_limit,
        }
    }
    
    /// Check if the payload size is within limits
    fn check_size_limit(&self, payload: &BoundaryCrossingPayload) -> BoundaryCrossingResult<()> {
        if payload.size > self.size_limit {
            return Err(BoundaryCrossingError::SizeLimitExceeded);
        }
        Ok(())
    }
    
    /// Apply rate limiting
    fn apply_rate_limit(&self, key: &str) -> BoundaryCrossingResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let mut limiter = self.rate_limiter.write().unwrap();
        let last_time = limiter.entry(key.to_string()).or_insert(0);
        
        // Allow 1 crossing per second for the same key
        if *last_time > 0 && now - *last_time < 1 {
            return Err(BoundaryCrossingError::RateLimitExceeded);
        }
        
        *last_time = now;
        Ok(())
    }
}

impl BoundaryCrossingProtocol for DefaultBoundaryCrossingProtocol {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn source_boundary(&self) -> BoundaryType {
        self.source
    }
    
    fn destination_boundary(&self) -> BoundaryType {
        self.destination
    }
    
    fn verify_authentication(
        &self,
        payload: &BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<bool> {
        match &payload.authentication {
            BoundaryAuthentication::None => {
                // Only allow None for InsideSystem -> OutsideSystem
                if self.source_boundary() == BoundaryType::InsideSystem && 
                   self.destination_boundary() == BoundaryType::OutsideSystem {
                    Ok(true)
                } else {
                    Err(BoundaryCrossingError::AuthenticationFailed(
                        "Authentication required for this boundary crossing".to_string()
                    ))
                }
            }
            BoundaryAuthentication::Signature(sig) => {
                // In a real implementation, verify the signature
                if sig.len() > 10 {
                    Ok(true)
                } else {
                    Err(BoundaryCrossingError::AuthenticationFailed(
                        "Invalid signature".to_string()
                    ))
                }
            }
            BoundaryAuthentication::Capability(cap) => {
                // In a real implementation, verify the capability
                if !cap.is_empty() {
                    Ok(true)
                } else {
                    Err(BoundaryCrossingError::AuthenticationFailed(
                        "Invalid capability".to_string()
                    ))
                }
            }
            BoundaryAuthentication::ZkProof(proof) => {
                // In a real implementation, verify the ZK proof
                if !proof.is_empty() {
                    Ok(true)
                } else {
                    Err(BoundaryCrossingError::AuthenticationFailed(
                        "Invalid ZK proof".to_string()
                    ))
                }
            }
            BoundaryAuthentication::MultiFactor(factors) => {
                // In a real implementation, verify all factors
                if !factors.is_empty() {
                    Ok(true)
                } else {
                    Err(BoundaryCrossingError::AuthenticationFailed(
                        "Invalid multi-factor authentication".to_string()
                    ))
                }
            }
        }
    }
    
    fn process_incoming(
        &self,
        payload: BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<Vec<u8>> {
        // Check size limit
        self.check_size_limit(&payload)?;
        
        // Apply rate limiting
        self.apply_rate_limit(&payload.crossing_id)?;
        
        // Verify authentication
        self.verify_authentication(&payload)?;
        
        // Start timing the crossing
        let timer = metrics::start_boundary_crossing_timer(&payload.crossing_type);
        
        // In a real implementation, we would process the payload based on the crossing type
        let result = Ok(payload.data);
        
        // Record the crossing completion
        metrics::complete_boundary_crossing(
            &payload.crossing_type,
            timer,
            result.is_ok()
        );
        
        result
    }
    
    fn prepare_outgoing<T: BoundarySafe>(
        &self,
        data: &T,
        auth: BoundaryAuthentication,
    ) -> BoundaryCrossingResult<BoundaryCrossingPayload> {
        // Check if the data is compatible with the destination boundary
        if !data.validate_for_boundary(self.destination_boundary()) {
            return Err(BoundaryCrossingError::InvalidCrossing(
                format!("Data is not compatible with destination boundary: {}", self.destination_boundary())
            ));
        }
        
        // Prepare the data for crossing
        let serialized_data = data.prepare_for_crossing();
        
        // Create the payload
        let payload = BoundaryCrossingPayload::new(
            self.source_boundary(),
            self.destination_boundary(),
            self.crossing_type,
            serialized_data,
            auth,
        );
        
        // Check size limit
        self.check_size_limit(&payload)?;
        
        Ok(payload)
    }
}

/// Registry for boundary crossing protocols
#[derive(Default)]
pub struct BoundaryCrossingRegistry {
    protocols: RwLock<HashMap<String, Arc<dyn BoundaryCrossingProtocol>>>,
}

impl BoundaryCrossingRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            protocols: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a protocol
    pub fn register_protocol(&self, protocol: Arc<dyn BoundaryCrossingProtocol>) {
        let mut protocols = self.protocols.write().unwrap();
        protocols.insert(protocol.name().to_string(), protocol);
    }
    
    /// Get a protocol by name
    pub fn get_protocol(&self, name: &str) -> Option<Arc<dyn BoundaryCrossingProtocol>> {
        let protocols = self.protocols.read().unwrap();
        protocols.get(name).cloned()
    }
    
    /// Find a protocol for specific boundary types
    pub fn find_protocol_for_boundaries(
        &self,
        source: BoundaryType,
        destination: BoundaryType,
    ) -> Option<Arc<dyn BoundaryCrossingProtocol>> {
        let protocols = self.protocols.read().unwrap();
        
        for protocol in protocols.values() {
            if protocol.source_boundary() == source && protocol.destination_boundary() == destination {
                return Some(protocol.clone());
            }
        }
        
        None
    }
    
    /// Process a crossing using a specific protocol
    pub fn process_crossing(
        &self,
        protocol_name: &str,
        payload: BoundaryCrossingPayload,
    ) -> BoundaryCrossingResult<Vec<u8>> {
        if let Some(protocol) = self.get_protocol(protocol_name) {
            protocol.process_incoming(payload)
        } else {
            Err(BoundaryCrossingError::ProtocolError(
                format!("Protocol not found: {}", protocol_name)
            ))
        }
    }
} 