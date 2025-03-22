# ADR-015: Geo-distributed Node Implementation

## Status

Proposed

## Context

The Causality system needs to operate efficiently across multiple machines in different geographic locations while maintaining security and performance. The existing local-only implementation does not provide the features needed for geo-distributed deployment, such as secure network transport, certificate management, and peer discovery.

## Decision

We will implement a Geo-Distributed mode for Causality using QUIC (Quick UDP Internet Connections) as the transport protocol. QUIC provides a modern, secure, and efficient foundation for our distributed system with built-in TLS 1.3 encryption and UDP-based transport.

### Architecture

The implementation will consist of the following components:

1. **QUIC Server**: Handles incoming connections and requests.
2. **QUIC Client**: Manages outgoing connections to other Causality nodes.
3. **Certificate Management**: Generates and validates TLS certificates.
4. **Peer Discovery**: Automatic and manual peer discovery mechanisms.
5. **Message Routing**: Efficient routing of messages between nodes.

### QUIC Configuration Options

- **Server Port**: Configurable port for the QUIC server.
- **Client Connection Timeout**: Maximum time to wait for connection establishment.
- **TLS Certificate Path**: Location of TLS certificates for secure communication.
- **Maximum Message Size**: Configurable limit on message size.
- **Retry Token**: Enable/disable retry token for DoS protection.

### Network Modes

1. **ClientMode**: Node acts only as a client, connecting to existing server nodes.
2. **ServerMode**: Node acts as a server, accepting incoming connections from clients.
3. **HybridMode**: Node acts as both client and server, enabling peer-to-peer operation.

### Message Types

- **QuicPing/QuicPong**: Health check mechanisms.
- **QuicDiscover**: Peer discovery request.
- **QuicDiscoveryResponse**: Response containing known peers.
- **QuicMessage**: Generic message for application-level communication.
- **QuicBroadcast**: Message to be propagated to all connected peers.

## Consequences

### Security

The Geo-Distributed mode provides several security enhancements:

- **TLS 1.3 Encryption**: All communications are encrypted using the latest TLS protocol.
- **Certificate Verification**: All peers must present valid certificates.
- **Message Authentication**: All messages are authenticated to prevent spoofing.
- **Connection Security**: QUIC's connection ID ensures connection validity, even with changing IP addresses.

### Performance

- **Multiplexing**: Multiple streams over a single connection reduce latency.
- **Connection Migration**: Connections can migrate across network changes without disruption.
- **Low Latency**: Designed for minimal round-trips during connection establishment.
- **UDP-Based**: Avoids head-of-line blocking issues present in TCP.

### Scalability

- **Dynamic Peer Discovery**: Nodes can discover peers automatically.
- **Resource Efficiency**: QUIC's design minimizes resource usage for idle connections.
- **Geographic Distribution**: System can operate effectively across global regions.

## Implementation

### Integration with Existing Infrastructure

The QUIC implementation integrates with the existing P2P network infrastructure through:

- **Configuration Conversion**: Translation between `P2PConfig` and `QuicConfig`.
- **Actor Integration**: QUIC nodes register with actor system for message handling.
- **Logging**: Integration with the existing logging infrastructure.

### Usage

```rust
use quinn::{Endpoint, ServerConfig, Connection};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;
use std::error::Error;

// Placeholder types
type QuicMessage = String;
type PeerAddress = String;
type QuicClient = Endpoint;
type QuicServer = Endpoint;

/// Default QUIC configuration with server port and certificate
fn default_quic_config(cert_path: &str, server_port: u16) -> Result<ServerConfig, Box<dyn Error>> {
    let config = ServerConfig::default(); // TODO: Load TLS cert from `cert_path`
    Ok(config)
}

/// Start a QUIC server
async fn start_quic_server<F>(config: ServerConfig, handle_message: F) -> Result<QuicServer, Box<dyn Error>>
where
    F: Fn(QuicMessage) + Send + 'static,
{
    let (endpoint, _) = Endpoint::server(config, "[::]:4433".parse()?)?;
    
    // Message handling loop (spawned task)
    tokio::spawn(async move {
        while let Some(conn) = endpoint.accept().await {
            let conn = conn.await.expect("Failed to accept connection");
            let handle_message = handle_message.clone();
            tokio::spawn(async move {
                let _ = handle_quic_connection(conn, handle_message).await;
            });
        }
    });

    Ok(endpoint)
}

/// Stop QUIC server
fn stop_quic_server(server: QuicServer) {
    drop(server); // Endpoint closes when dropped
}

/// Handle incoming QUIC messages
async fn handle_quic_connection(conn: Connection, handle_message: impl Fn(QuicMessage)) -> Result<(), Box<dyn Error>> {
    let mut recv = conn.open_uni().await?;
    let mut buffer = vec![0; 1024];
    let n = recv.read(&mut buffer).await?;
    let msg = String::from_utf8_lossy(&buffer[..n]).to_string();
    handle_message(msg);
    Ok(())
}

/// Connect to a peer
async fn connect_to_peer(client: &QuicClient, address: PeerAddress) -> Result<Connection, Box<dyn Error>> {
    let conn = client.connect(address.to_socket_addrs()?.next().unwrap(), "quic-server")?.await?;
    
    // Send initial message
    let mut send = conn.open_uni().await?;
    send.write_all(b"QuicPing: Hello").await?;
    send.finish().await?;

    Ok(conn)
}

/// Broadcast a message to all peers
async fn broadcast_message(server: &QuicServer, msg: QuicMessage) {
    // This is a placeholder, as QUIC doesn’t have built-in broadcast
    println!("Broadcasting message: {}", msg);
}
```

### Testing

A testing script (`scripts/test-quic.hs`) is provided that demonstrates both server and client modes, allowing for verification of connectivity and message passing.

### Future Improvements

- **Connection Pooling**: Implement connection caching for frequently contacted peers.
- **Load Balancing**: Distribute connections across multiple server instances.
- **Congestion Control**: Add custom congestion control algorithms for Causality traffic patterns.
- **NAT Traversal**: Improve handling of NAT scenarios for better connectivity.

### Conclusion

The QUIC-based Geo-Distributed mode provides a secure, efficient, and modern foundation for operating Causality across geographic locations. It simplifies deployments in distributed environments while maintaining the security properties required by the system. 