# ADR 016: Network Adapter

## Status

Accepted, updates required in light of recent ADRs

## Implementation Status

Not implemented. The NetworkAdapter as described in this ADR has not been implemented in the codebase. While there are some networking concepts in the codebase, such as:

1. `PeerInfo` and sync-related structures in `/src/log/sync.rs` - Related to log synchronization between peers
2. Network status enums in `/src/actor/operator.rs` - Basic status indicators for operators

These implementations are very limited and don't match the comprehensive NetworkAdapter design described in this ADR. Specifically, the codebase lacks:

1. The core `NetworkAdapter` structure and its API
2. The standardized message format with types and priorities
3. Peer management with different peer types
4. The security features (authentication, encryption)
5. The network state management system

It appears that some basic peer-to-peer synchronization code exists (for log synchronization), but it's focused on a specific use case rather than providing the general network communication layer described in this ADR.

## Context

Causality requires a robust peer-to-peer communication system to enable distributed consensus and data sharing between nodes. Unlike chain adapters (Ethereum, Celestia) that interact with external chains, this adapter focuses on direct node-to-node communication within the Causality network.

Currently, our system lacks a standardized approach for handling peer-to-peer communication, which makes it difficult to implement distributed features like consensus algorithms, shared state, and coordinated effect execution. As we move towards a more distributed architecture, we need a well-defined network layer that handles the complexities of network communication while providing a simple interface for the rest of the system.

The Causality network consists of two main types of nodes:
1. **Operator Nodes**: Full nodes that participate in consensus, execution, and data storage
2. **Party User Clients**: Lightweight clients that interact with the network to access data and services

These different node types require different communication patterns and priorities, which our network adapter needs to support.

Key challenges that must be addressed include:
1. Reliable message delivery in potentially unreliable network conditions
2. Secure communication with authentication and encryption
3. Peer discovery and management with support for different peer types
4. Handling network partitions and reconnections
5. Supporting various network topologies
6. Providing a consistent interface for both local and wide-area networks
7. Allowing nodes to control their peer connections through whitelists/blacklists
8. Prioritizing certain peer connections based on node type and requirements

## Decision

We will implement a dedicated NetworkAdapter module that provides a standardized interface for all peer-to-peer communication in the Causality system. The adapter will abstract away the complexities of network communication and provide a simple, consistent API for the rest of the system.

### Core Components

#### Configuration

The NetworkAdapter will be configurable to support different deployment scenarios:

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use bytes::Bytes;

type PeerAddress = String;
type PeerId = String;
type Connection = Arc<dyn ConnectionTrait + Send + Sync>;

struct NetworkConfig {
    listen_address: String,         // Address to listen on (e.g., "0.0.0.0")
    listen_port: i32,               // Port to listen on
    bootstrap_peers: Vec<PeerAddress>, // Initial peers to connect to
    max_connections: i32,           // Maximum number of concurrent connections 
    connection_timeout: i32,        // Connection timeout in milliseconds
    use_encryption: bool,           // Whether to use TLS/encryption
    tls_config: Option<TLSConfig>,  // TLS configuration if encryption is enabled
    node_name: String,              // Human-readable node identifier
    message_buffer_size: i32,       // Size of message buffer for async processing
    peer_list_mode: PeerListMode,   // Whitelist or blacklist mode
    peer_list: Vec<PeerId>,         // List of peers to whitelist or blacklist
    peer_type_weights: HashMap<PeerType, i32>, // Prioritization weights for peer types
}

enum PeerListMode {
    Whitelist,
    Blacklist,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PeerType {
    OperatorNode,
    PartyUserClient,
    UnknownPeer,
}
```

#### State Management

The adapter will maintain state about connected peers and network conditions:

```rust
struct NetworkState {
    active_peers: HashMap<PeerId, PeerInfo>,   // Connected and active peers
    pending_peers: HashMap<PeerId, PeerInfo>,  // Peers attempting to connect
    message_handlers: Vec<MessageHandler>,     // Registered message handlers
    network_stats: NetworkStats,               // Statistics about network activity
    async_jobs: Vec<JoinHandle<()>>,          // Background jobs for network operations
    priority_queue: PriorityQueue<Message>,    // Priority queue for outgoing messages
}

struct PeerInfo {
    peer_id: PeerId,                          // Unique peer identifier
    peer_type: PeerType,                      // Type of peer (Operator, Client, etc.)
    connection_status: ConnectionStatus,      // Current connection status
    address: PeerAddress,                     // Network address of peer
    latency: i32,                             // Last measured latency in ms
    last_seen: DateTime<Utc>,                 // Last time peer was seen
    messages_sent: i32,                       // Count of messages sent to peer
    messages_received: i32,                   // Count of messages received from peer
    connection: Connection,                   // Active connection if connected
    priority: i32,                            // Connection priority (higher = more important)
}
```

#### Message Format

A standardized message format will ensure consistency across the system:

```rust
struct Message {
    message_id: Uuid,
    message_type: MessageType,
    sender: PeerId,
    recipient: Option<PeerId>,  // None means broadcast
    timestamp: DateTime<Utc>,
    payload: Bytes,
    signature: Option<Bytes>,
    priority: MessagePriority, // Priority level for processing
}

enum MessageType {
    PeerDiscovery,
    PeerStatus,
    EffectPropagation,
    TimeMapUpdate,
    ResourceUpdate,
    ProgramSync,
    ControlMessage,
}

enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}
```

### Core API

The NetworkAdapter will expose the following core functions:

1. **Adapter Creation and Management**
   ```rust
   async fn create_network_adapter(config: NetworkConfig) -> Result<NetworkAdapter, NetworkError>;
   ```

2. **Message Operations**
   ```rust
   /// Send a message to all connected peers
   async fn broadcast_message(
       state: &mut NetworkState, 
       message_type: MessageType, 
       payload: Bytes, 
       priority: MessagePriority
   ) -> Result<Vec<PeerId>, NetworkError>;
   
   /// Send a message to a specific peer
   async fn send_direct_message(
       state: &mut NetworkState, 
       peer_id: &PeerId, 
       message_type: MessageType, 
       payload: Bytes, 
       priority: MessagePriority
   ) -> Result<(), NetworkError>;
   
   /// Subscribe to a specific message type
   async fn subscribe_to_messages(
       state: &mut NetworkState, 
       message_type: MessageType, 
       handler: impl Fn(Message) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static
   ) -> NetworkState;
   ```

3. **Peer Management**
   ```rust
   /// Get the status of a peer
   async fn get_peer_status(
       state: &NetworkState, 
       peer_id: &PeerId
   ) -> Result<PeerInfo, NetworkError>;
   
   /// Discover new peers from the network
   async fn discover_peers(
       state: &mut NetworkState
   ) -> Result<Vec<PeerInfo>, NetworkError>;
   
   /// Add a new peer
   async fn add_peer(
       state: &mut NetworkState, 
       peer_address: PeerAddress, 
       peer_type: PeerType, 
       priority: i32
   ) -> NetworkState;
   
   /// Remove a peer
   async fn remove_peer(
       state: &mut NetworkState, 
       peer_id: &PeerId
   ) -> NetworkState;
   
   /// Update peer whitelist/blacklist
   async fn update_peer_list(
       state: &mut NetworkState, 
       mode: PeerListMode, 
       peers: Vec<PeerId>
   ) -> NetworkState;
   
   /// Set peer priority
   async fn set_peer_priority(
       state: &mut NetworkState, 
       peer_id: &PeerId, 
       priority: i32
   ) -> NetworkState;
   
   /// Set peer type weights
   async fn set_peer_type_weights(
       state: &mut NetworkState, 
       weights: HashMap<PeerType, i32>
   ) -> NetworkState;
   ```

### Peer Types and Prioritization

The NetworkAdapter will support different peer types with configurable prioritization:

1. **Operator Nodes**: Full nodes that participate in consensus and execution. These are typically prioritized for program execution and consensus.

2. **Party User Clients**: Lightweight clients that access data and services. These are prioritized for serving data requests.

3. **Unknown Peers**: Newly discovered peers that haven't been classified yet.

Peer connections will be managed based on:

1. **Whitelist/Blacklist**: Nodes can be configured to only connect to specific peers (whitelist) or to connect to any peer except specific ones (blacklist).

2. **Priority Levels**: Each peer connection will have a priority level that determines resource allocation during high load.

3. **Peer Type Weights**: Global weights can be assigned to different peer types to influence connection prioritization.

### Security Measures

The NetworkAdapter will implement several security features:

1. **Authentication**: Verifying the identity of peers through public-key cryptography
2. **Encryption**: Using TLS for secure communication
3. **Message Integrity**: Including signatures to verify message authenticity
4. **Peer Validation**: Implementing measures to prevent Sybil attacks
5. **Rate Limiting**: Protecting against DoS attacks
6. **Access Control**: Using whitelist/blacklist to control which peers can connect

## Consequences

### Positive

1. **Simplified Communication**: The rest of the system can use a consistent, high-level API for network operations without worrying about the underlying details.
   
2. **Enhanced Security**: Built-in security features ensure that all communication is authenticated and encrypted.
   
3. **Better Reliability**: Automatic handling of connection issues, retries, and reconnections improves the overall reliability of the system.
   
4. **Modularity**: The adapter pattern allows for different network implementations (e.g., WebRTC, libp2p, custom TCP) without changing the rest of the codebase.
   
5. **Observability**: Centralized logging and metrics collection for all network operations.

6. **Flexibility**: Support for different peer types and prioritization allows nodes to optimize their connections for their specific needs.

### Negative

1. **Additional Complexity**: Introducing another layer adds some complexity to the system.
   
2. **Performance Overhead**: The abstraction may introduce some overhead compared to direct network calls.
   
3. **Deployment Complexity**: Proper configuration for different network environments (local, cloud, etc.) will require careful documentation and potentially additional tooling.

4. **Participation Limitations**: Nodes that significantly restrict their peer connections through strict whitelisting may limit their ability to participate in certain programs or access the full network state.

### Neutral

1. **Dependencies**: The implementation will require several external libraries for networking, cryptography, and concurrency management.

2. **Configuration Tradeoffs**: Different configurations will offer trade-offs between security, performance, and network participation.

## Implementation Plan

1. Set up the basic module structure
2. Implement core communication primitives (connect, send, receive)
3. Add peer discovery and management with support for peer types
4. Implement message serialization and handling with priority
5. Add security features (authentication, encryption)
6. Implement advanced features (reconnection, error handling, whitelist/blacklist)
7. Add message prioritization and resource allocation
8. Add comprehensive testing
9. Document usage patterns and examples