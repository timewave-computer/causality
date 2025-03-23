// Operator Actor Module
//
// This module implements the Operator actor type for Causality.
// Operators are nodes that participate in the system, executing programs and verifying results.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock, Mutex};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant, SystemTime};

use crate::error::{Error, Result};
use crate::types::{ContentId, ContentHash, TraceId, Timestamp};
use crate::actor::{
    Actor, ActorType, ActorState, ActorInfo, 
    ActorRole, ActorCapability, Message, MessageCategory, MessagePayload,
    GenericActorId,
    ActorIdBox,
};

/// Operator capability level
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatorCapabilityLevel {
    /// Basic execution capabilities
    Basic,
    /// Advanced execution capabilities
    Advanced,
    /// Full system capabilities
    Full,
    /// Custom capability level
    Custom(String),
}

/// Operator network status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkStatus {
    /// Online and healthy
    Online,
    /// Online but degraded
    Degraded(String), // reason
    /// Offline
    Offline,
    /// Syncing
    Syncing(u8), // percentage
}

/// Operator resource allocation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// CPU allocation (percentage)
    pub cpu: u8,
    /// Memory allocation (MB)
    pub memory: u64,
    /// Storage allocation (MB)
    pub storage: u64,
    /// Network bandwidth (Mbps)
    pub bandwidth: u64,
}

impl Default for ResourceAllocation {
    fn default() -> Self {
        ResourceAllocation {
            cpu: 50,      // 50% by default
            memory: 1024, // 1GB by default
            storage: 10240, // 10GB by default
            bandwidth: 100, // 100Mbps by default
        }
    }
}

/// Operator performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time (ms)
    pub avg_response_time: f64,
    /// Requests processed per second
    pub requests_per_second: f64,
    /// Error rate (percentage)
    pub error_rate: f64,
    /// Uptime (percentage)
    pub uptime: f64,
    /// Last updated timestamp
    pub last_updated: Timestamp,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics {
            avg_response_time: 0.0,
            requests_per_second: 0.0,
            error_rate: 0.0,
            uptime: 100.0,
            last_updated: Timestamp::now(),
        }
    }
}

/// Peer status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    /// Peer is connected and operational
    Connected,
    /// Peer is known but currently disconnected
    Disconnected,
    /// Peer is pending connection establishment
    Pending,
    /// Peer is available but in degraded mode
    Degraded,
    /// Peer is temporarily unavailable
    Unavailable,
}

/// Peer performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Last time metrics were updated
    pub last_updated: SystemTime,
}

/// Operator metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperatorMetrics {
    /// Total operations processed
    pub total_operations: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Average operation execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Resource utilization percentage
    pub resource_utilization: f64,
    /// Last time metrics were updated
    pub last_updated: SystemTime,
}

/// Resource pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Resource name
    pub name: String,
    /// Total capacity
    pub total_capacity: u64,
    /// Available capacity
    pub available_capacity: u64,
    /// Resource utilization percentage
    pub utilization: f64,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// The ID of the peer
    pub id: ActorIdBox,
    
    /// Status information
    pub status: PeerStatus,
    
    /// Last time communication was received
    pub last_seen: SystemTime,
    
    /// Performance metrics
    pub metrics: PeerMetrics,
}

/// Operator actor implementation
#[derive(Debug)]
pub struct Operator {
    /// Actor ID
    id: ActorIdBox,
    /// Actor type
    actor_type: ActorType,
    /// Actor state
    state: RwLock<ActorState>,
    /// Actor information
    info: RwLock<ActorInfo>,
    /// Network address
    address: RwLock<String>,
    /// Network status
    network_status: RwLock<NetworkStatus>,
    /// Resource allocation
    resources: RwLock<HashMap<String, ResourcePool>>,
    /// Performance metrics
    metrics: RwLock<OperatorMetrics>,
    /// Known peers
    peers: RwLock<HashMap<ActorIdBox, PeerInfo>>,
    /// Operator capabilities
    capabilities: RwLock<OperatorCapabilityLevel>,
    /// Supported program types
    supported_programs: RwLock<Vec<String>>,
    /// Last heartbeat timestamp
    last_heartbeat: RwLock<Timestamp>,
}

impl Operator {
    /// Create a new operator actor
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        address: impl Into<String>,
        capabilities: OperatorCapabilityLevel,
    ) -> Self {
        let id_str = id.into();
        let name_str = name.into();
        let actor_id = GenericActorId::from_string(id_str);
        let now = Timestamp::now();
        
        let info = ActorInfo {
            id: actor_id.clone(),
            actor_type: ActorType::Operator,
            state: ActorState::Pending,
            name: name_str,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        Operator {
            id: actor_id,
            actor_type: ActorType::Operator,
            state: RwLock::new(ActorState::Pending),
            info: RwLock::new(info),
            address: RwLock::new(address.into()),
            network_status: RwLock::new(NetworkStatus::Offline),
            resources: RwLock::new(HashMap::new()),
            metrics: RwLock::new(OperatorMetrics::default()),
            peers: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(capabilities),
            supported_programs: RwLock::new(Vec::new()),
            last_heartbeat: RwLock::new(now),
        }
    }
    
    /// Get the operator's network address
    pub fn address(&self) -> Result<String> {
        let address = self.address.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on address".to_string())
        })?;
        
        Ok(address.clone())
    }
    
    /// Update the operator's network address
    pub fn update_address(&self, address: impl Into<String>) -> Result<()> {
        let mut addr = self.address.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on address".to_string())
        })?;
        
        *addr = address.into();
        
        Ok(())
    }
    
    /// Get the operator's network status
    pub fn network_status(&self) -> Result<NetworkStatus> {
        let status = self.network_status.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on network_status".to_string())
        })?;
        
        Ok(status.clone())
    }
    
    /// Update the operator's network status
    pub fn update_network_status(&self, status: NetworkStatus) -> Result<()> {
        let mut nstatus = self.network_status.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on network_status".to_string())
        })?;
        
        *nstatus = status;
        
        Ok(())
    }
    
    /// Get the operator's resource allocation
    pub fn resources(&self) -> Result<HashMap<String, ResourcePool>> {
        let resources = self.resources.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on resources".to_string())
        })?;
        
        Ok(resources.clone())
    }
    
    /// Update the operator's resource allocation
    pub fn update_resources(&self, resources: HashMap<String, ResourcePool>) -> Result<()> {
        let mut res = self.resources.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on resources".to_string())
        })?;
        
        *res = resources;
        
        Ok(())
    }
    
    /// Get the operator's performance metrics
    pub fn metrics(&self) -> Result<OperatorMetrics> {
        let metrics = self.metrics.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on metrics".to_string())
        })?;
        
        Ok(metrics.clone())
    }
    
    /// Update the operator's performance metrics
    pub fn update_metrics(&self, metrics: OperatorMetrics) -> Result<()> {
        let mut m = self.metrics.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on metrics".to_string())
        })?;
        
        *m = metrics;
        
        Ok(())
    }
    
    /// Add a peer to the operator's known peers
    pub fn add_peer(&self, peer: PeerInfo) -> Result<()> {
        let mut peers = self.peers.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on peers".to_string())
        })?;
        
        peers.insert(peer.id.clone(), peer);
        
        Ok(())
    }
    
    /// Remove a peer from the operator's known peers
    pub fn remove_peer(&self, peer_id: &ActorIdBox) -> Result<()> {
        let mut peers = self.peers.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on peers".to_string())
        })?;
        
        peers.remove(peer_id);
        
        Ok(())
    }
    
    /// Get a peer by ID
    pub fn get_peer(&self, peer_id: &ActorIdBox) -> Result<Option<PeerInfo>> {
        let peers = self.peers.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on peers".to_string())
        })?;
        
        Ok(peers.get(peer_id).cloned())
    }
    
    /// Get all known peers
    pub fn get_all_peers(&self) -> Result<Vec<PeerInfo>> {
        let peers = self.peers.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on peers".to_string())
        })?;
        
        Ok(peers.values().cloned().collect())
    }
    
    /// Get connected peers
    pub fn get_connected_peers(&self) -> Result<Vec<PeerInfo>> {
        let peers = self.peers.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on peers".to_string())
        })?;
        
        Ok(peers.values()
            .filter(|p| p.status == PeerStatus::Connected)
            .cloned()
            .collect()
        )
    }
    
    /// Update peer connection status
    pub fn update_peer_status(&self, peer_id: &ActorIdBox, status: PeerStatus) -> Result<()> {
        let mut peers = self.peers.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on peers".to_string())
        })?;
        
        if let Some(peer) = peers.get_mut(peer_id) {
            peer.status = status;
            peer.last_seen = SystemTime::now();
            Ok(())
        } else {
            Err(Error::NotFound(format!("Peer not found: {:?}", peer_id)))
        }
    }
    
    /// Get the operator's capabilities
    pub fn capabilities(&self) -> Result<OperatorCapabilityLevel> {
        let caps = self.capabilities.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on capabilities".to_string())
        })?;
        
        Ok(caps.clone())
    }
    
    /// Update the operator's capabilities
    pub fn update_capabilities(&self, capabilities: OperatorCapabilityLevel) -> Result<()> {
        let mut caps = self.capabilities.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on capabilities".to_string())
        })?;
        
        *caps = capabilities;
        
        Ok(())
    }
    
    /// Add a supported program type
    pub fn add_supported_program(&self, program_type: impl Into<String>) -> Result<()> {
        let mut programs = self.supported_programs.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on supported_programs".to_string())
        })?;
        
        let program_type_str = program_type.into();
        
        if !programs.contains(&program_type_str) {
            programs.push(program_type_str);
        }
        
        Ok(())
    }
    
    /// Remove a supported program type
    pub fn remove_supported_program(&self, program_type: &str) -> Result<()> {
        let mut programs = self.supported_programs.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on supported_programs".to_string())
        })?;
        
        programs.retain(|p| p != program_type);
        
        Ok(())
    }
    
    /// Get all supported program types
    pub fn get_supported_programs(&self) -> Result<Vec<String>> {
        let programs = self.supported_programs.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on supported_programs".to_string())
        })?;
        
        Ok(programs.clone())
    }
    
    /// Check if a program type is supported
    pub fn supports_program(&self, program_type: &str) -> Result<bool> {
        let programs = self.supported_programs.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on supported_programs".to_string())
        })?;
        
        Ok(programs.contains(&program_type.to_string()))
    }
    
    /// Record a heartbeat
    pub fn record_heartbeat(&self) -> Result<()> {
        let mut heartbeat = self.last_heartbeat.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on last_heartbeat".to_string())
        })?;
        
        *heartbeat = Timestamp::now();
        
        Ok(())
    }
    
    /// Get the last heartbeat timestamp
    pub fn last_heartbeat(&self) -> Result<Timestamp> {
        let heartbeat = self.last_heartbeat.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on last_heartbeat".to_string())
        })?;
        
        Ok(*heartbeat)
    }
    
    /// Check if the operator is healthy
    pub fn is_healthy(&self) -> Result<bool> {
        let status = self.network_status()?;
        let heartbeat = self.last_heartbeat()?;
        let now = Timestamp::now();
        
        // Consider healthy if online and heartbeat within last 60 seconds
        Ok(matches!(status, NetworkStatus::Online) && (now - heartbeat) < 60)
    }
}

#[async_trait]
impl Actor for Operator {
    fn id(&self) -> &ActorIdBox {
        &self.id
    }
    
    fn actor_type(&self) -> ActorType {
        self.actor_type.clone()
    }
    
    fn state(&self) -> ActorState {
        self.state.read().unwrap_or(ActorState::Pending)
    }
    
    fn info(&self) -> ActorInfo {
        let mut info = self.info.read().unwrap_or_else(|_| {
            panic!("Failed to acquire read lock on info")
        }).clone();
        
        // Update with current state
        info.state = self.state();
        info.updated_at = Timestamp::now();
        
        info
    }
    
    async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on state".to_string())
        })?;
        
        if *state != ActorState::Pending {
            return Err(Error::InvalidState(format!(
                "Cannot initialize actor in state: {:?}",
                *state
            )));
        }
        
        *state = ActorState::Active;
        
        // Update network status
        self.update_network_status(NetworkStatus::Online)?;
        
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on state".to_string())
        })?;
        
        if *state != ActorState::Pending && *state != ActorState::Suspended {
            return Err(Error::InvalidState(format!(
                "Cannot start actor in state: {:?}",
                *state
            )));
        }
        
        *state = ActorState::Active;
        
        // Update network status
        self.update_network_status(NetworkStatus::Online)?;
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on state".to_string())
        })?;
        
        if *state != ActorState::Active {
            return Err(Error::InvalidState(format!(
                "Cannot stop actor in state: {:?}",
                *state
            )));
        }
        
        *state = ActorState::Inactive;
        
        // Update network status
        self.update_network_status(NetworkStatus::Offline)?;
        
        Ok(())
    }
    
    async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
        // Record heartbeat
        self.record_heartbeat()?;
        
        match message.category {
            // Network management messages
            MessageCategory::NetworkManagement => {
                match message.payload {
                    MessagePayload::UpdateNetworkStatus { status } => {
                        self.update_network_status(status)?;
                        Ok(None)
                    },
                    MessagePayload::Heartbeat => {
                        self.record_heartbeat()?;
                        
                        // Respond with our status
                        let response = Message {
                            id: format!("resp-{}", message.id),
                            sender: self.id.clone(),
                            recipients: vec![message.sender.clone()],
                            category: MessageCategory::NetworkManagement,
                            payload: MessagePayload::HeartbeatResponse { 
                                status: self.network_status()?,
                                timestamp: Timestamp::now(),
                            },
                            timestamp: Timestamp::now(),
                            trace_id: message.trace_id.clone(),
                        };
                        
                        Ok(Some(response))
                    },
                    MessagePayload::AddPeer { peer } => {
                        self.add_peer(peer)?;
                        Ok(None)
                    },
                    MessagePayload::RemovePeer { peer_id } => {
                        self.remove_peer(&peer_id)?;
                        Ok(None)
                    },
                    MessagePayload::UpdatePeerStatus { peer_id, connected } => {
                        self.update_peer_status(&peer_id, connected)?;
                        Ok(None)
                    },
                    _ => Err(Error::UnsupportedMessage(
                        "Unsupported network management message".to_string()
                    )),
                }
            },
            
            // Resource management messages
            MessageCategory::ResourceManagement => {
                match message.payload {
                    MessagePayload::UpdateResources { resources } => {
                        self.update_resources(resources)?;
                        Ok(None)
                    },
                    MessagePayload::GetResources => {
                        let resources = self.resources()?;
                        
                        let response = Message {
                            id: format!("resp-{}", message.id),
                            sender: self.id.clone(),
                            recipients: vec![message.sender.clone()],
                            category: MessageCategory::ResourceManagement,
                            payload: MessagePayload::ResourcesResponse { 
                                resources,
                            },
                            timestamp: Timestamp::now(),
                            trace_id: message.trace_id.clone(),
                        };
                        
                        Ok(Some(response))
                    },
                    _ => Err(Error::UnsupportedMessage(
                        "Unsupported resource management message".to_string()
                    )),
                }
            },
            
            // Program management messages
            MessageCategory::ProgramManagement => {
                match message.payload {
                    MessagePayload::AddSupportedProgram { program_type } => {
                        self.add_supported_program(program_type)?;
                        Ok(None)
                    },
                    MessagePayload::RemoveSupportedProgram { program_type } => {
                        self.remove_supported_program(&program_type)?;
                        Ok(None)
                    },
                    MessagePayload::GetSupportedPrograms => {
                        let programs = self.get_supported_programs()?;
                        
                        let response = Message {
                            id: format!("resp-{}", message.id),
                            sender: self.id.clone(),
                            recipients: vec![message.sender.clone()],
                            category: MessageCategory::ProgramManagement,
                            payload: MessagePayload::SupportedProgramsResponse { 
                                program_types: programs,
                            },
                            timestamp: Timestamp::now(),
                            trace_id: message.trace_id.clone(),
                        };
                        
                        Ok(Some(response))
                    },
                    _ => Err(Error::UnsupportedMessage(
                        "Unsupported program management message".to_string()
                    )),
                }
            },
            
            // Other message categories
            _ => Err(Error::UnsupportedMessage(format!(
                "Unsupported message category: {:?}",
                message.category
            ))),
        }
    }
    
    async fn has_permission(&self, permission: &str) -> Result<bool> {
        // Operators have permissions based on their capabilities
        match permission {
            "operate_node" | "audit" => Ok(true),
            "verify_facts" => {
                let caps = self.capabilities()?;
                Ok(matches!(caps, OperatorCapabilityLevel::Advanced | OperatorCapabilityLevel::Full))
            },
            "manage_governance" => {
                let caps = self.capabilities()?;
                Ok(matches!(caps, OperatorCapabilityLevel::Full))
            },
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_operator_actor() -> Result<()> {
        // Create an operator
        let operator = Operator::new(
            "test-operator",
            "Test Operator",
            "127.0.0.1:8000",
            OperatorCapabilityLevel::Basic,
        );
        
        // Check initial state
        assert_eq!(operator.id().0, "test-operator");
        assert_eq!(operator.actor_type(), ActorType::Operator);
        assert_eq!(operator.state(), ActorState::Pending);
        assert_eq!(operator.info().name, "Test Operator");
        assert_eq!(operator.address()?, "127.0.0.1:8000");
        assert_eq!(operator.network_status()?, NetworkStatus::Offline);
        
        // Initialize the operator
        operator.initialize().await?;
        assert_eq!(operator.state(), ActorState::Active);
        assert_eq!(operator.network_status()?, NetworkStatus::Online);
        
        // Update resources
        let resources = ResourceAllocation {
            cpu: 75,
            memory: 2048,
            storage: 20480,
            bandwidth: 200,
        };
        
        operator.update_resources(resources.clone())?;
        assert_eq!(operator.resources()?, resources);
        
        // Add supported programs
        operator.add_supported_program("wasm")?;
        operator.add_supported_program("risc-v")?;
        
        assert!(operator.supports_program("wasm")?);
        assert!(operator.supports_program("risc-v")?);
        assert!(!operator.supports_program("x86")?);
        
        assert_eq!(operator.get_supported_programs()?.len(), 2);
        
        // Add peers
        let peer1 = PeerInfo {
            id: ActorIdBox::from("peer1"),
            status: PeerStatus::Connected,
            last_seen: SystemTime::now(),
            metrics: PeerMetrics::default(),
        };
        
        let peer2 = PeerInfo {
            id: ActorIdBox::from("peer2"),
            status: PeerStatus::Disconnected,
            last_seen: SystemTime::now(),
            metrics: PeerMetrics::default(),
        };
        
        operator.add_peer(peer1.clone())?;
        operator.add_peer(peer2.clone())?;
        
        assert_eq!(operator.get_all_peers()?.len(), 2);
        assert_eq!(operator.get_connected_peers()?.len(), 1);
        
        let retrieved_peer = operator.get_peer(&ActorIdBox::from("peer1"))?;
        assert!(retrieved_peer.is_some());
        assert_eq!(retrieved_peer.unwrap().id, ActorIdBox::from("peer1"));
        
        // Update peer status
        operator.update_peer_status(&ActorIdBox::from("peer2"), PeerStatus::Connected)?;
        assert_eq!(operator.get_connected_peers()?.len(), 2);
        
        // Check permissions
        assert!(operator.has_permission("operate_node").await?);
        assert!(operator.has_permission("audit").await?);
        assert!(!operator.has_permission("verify_facts").await?);
        
        // Update capabilities
        operator.update_capabilities(OperatorCapabilityLevel::Advanced)?;
        assert!(operator.has_permission("verify_facts").await?);
        assert!(!operator.has_permission("manage_governance").await?);
        
        operator.update_capabilities(OperatorCapabilityLevel::Full)?;
        assert!(operator.has_permission("manage_governance").await?);
        
        // Check health
        assert!(operator.is_healthy()?);
        
        // Stop the operator
        operator.stop().await?;
        assert_eq!(operator.state(), ActorState::Inactive);
        assert_eq!(operator.network_status()?, NetworkStatus::Offline);
        assert!(!operator.is_healthy()?);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_operator_message_handling() -> Result<()> {
        // Create an operator
        let operator = Operator::new(
            "message-operator",
            "Message Operator",
            "127.0.0.1:9000",
            OperatorCapabilityLevel::Basic,
        );
        
        // Initialize the operator
        operator.initialize().await?;
        
        // Test handling heartbeat message
        let heartbeat_msg = Message {
            id: "msg1".to_string(),
            sender: ActorIdBox::from("system"),
            recipients: vec![operator.id().clone()],
            category: MessageCategory::NetworkManagement,
            payload: MessagePayload::Heartbeat,
            timestamp: Timestamp::now(),
            trace_id: None,
        };
        
        let response = operator.handle_message(heartbeat_msg).await?;
        assert!(response.is_some());
        
        let resp = response.unwrap();
        assert_eq!(resp.sender, operator.id().clone());
        assert_eq!(resp.recipients[0], ActorIdBox::from("system"));
        
        match resp.payload {
            MessagePayload::HeartbeatResponse { status, .. } => {
                assert_eq!(status, NetworkStatus::Online);
            },
            _ => panic!("Unexpected response payload"),
        }
        
        // Test handling update network status message
        let status_msg = Message {
            id: "msg2".to_string(),
            sender: ActorIdBox::from("system"),
            recipients: vec![operator.id().clone()],
            category: MessageCategory::NetworkManagement,
            payload: MessagePayload::UpdateNetworkStatus { 
                status: NetworkStatus::Degraded("High load".to_string()) 
            },
            timestamp: Timestamp::now(),
            trace_id: None,
        };
        
        operator.handle_message(status_msg).await?;
        
        // Check that status was updated
        assert_eq!(
            operator.network_status()?, 
            NetworkStatus::Degraded("High load".to_string())
        );
        
        // Test handling add supported program message
        let program_msg = Message {
            id: "msg3".to_string(),
            sender: ActorIdBox::from("system"),
            recipients: vec![operator.id().clone()],
            category: MessageCategory::ProgramManagement,
            payload: MessagePayload::AddSupportedProgram { 
                program_type: "evm".to_string() 
            },
            timestamp: Timestamp::now(),
            trace_id: None,
        };
        
        operator.handle_message(program_msg).await?;
        
        // Check that program was added
        assert!(operator.supports_program("evm")?);
        
        Ok(())
    }
} 