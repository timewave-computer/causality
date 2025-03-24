# Fact Synchronization in Causality

## Overview

This document details the fact synchronization mechanisms within the Causality architecture. Fact synchronization enables the consistent sharing of temporal facts across domains, nodes, and systems, ensuring global consistency and coherence of distributed state. This system is critical for maintaining the integrity of cross-domain operations and providing a unified view of the system's state.

## Core Concepts

### Fact Synchronization Model

The fact synchronization model defines how facts are shared across boundaries:

```rust
pub struct FactSynchronizationModel {
    /// Synchronization strategies
    strategies: HashMap<SyncContextKey, Box<dyn SyncStrategy>>,
    
    /// Synchronization policies
    policies: SyncPolicies,
    
    /// Synchronization metrics
    metrics: SyncMetrics,
}
```

### Synchronization Context

Provides context for synchronization decisions:

```rust
pub struct SyncContext {
    /// Source domain
    source_domain: DomainId,
    
    /// Target domain
    target_domain: DomainId,
    
    /// Fact type being synchronized
    fact_type: FactType,
    
    /// Synchronization mode
    mode: SyncMode,
    
    /// Authentication context
    auth_context: Option<AuthContext>,
}

/// Key for looking up synchronization strategies
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SyncContextKey {
    /// Source domain pattern
    source_domain_pattern: DomainPattern,
    
    /// Target domain pattern
    target_domain_pattern: DomainPattern,
    
    /// Fact type pattern
    fact_type_pattern: FactTypePattern,
}
```

### Synchronization Modes

Determines how facts are synchronized:

```rust
pub enum SyncMode {
    /// Push facts from source to target
    Push,
    
    /// Pull facts from target to source
    Pull,
    
    /// Bidirectional synchronization
    Bidirectional,
    
    /// Event-driven synchronization
    EventDriven,
}
```

## System Architecture

### Fact Synchronizer

The central component responsible for synchronizing facts:

```rust
pub struct FactSynchronizer {
    /// Fact registry
    fact_registry: Arc<RwLock<FactRegistry>>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
    
    /// Synchronization model
    sync_model: FactSynchronizationModel,
    
    /// Conflict resolver
    conflict_resolver: Arc<ConflictResolver>,
}

impl FactSynchronizer {
    /// Synchronize a fact with another domain
    pub fn synchronize_fact(
        &self,
        fact_id: FactId,
        target_domain: DomainId,
        mode: SyncMode,
    ) -> Result<SyncResult, SyncError> {
        // Get the fact
        let fact = {
            let registry = self.fact_registry.read().unwrap();
            registry.get_fact(&fact_id)
                .ok_or(SyncError::FactNotFound(fact_id))?
                .clone()
        };
        
        // Create sync context
        let context = SyncContext {
            source_domain: system.domain_id(),
            target_domain,
            fact_type: fact.fact_type.clone(),
            mode,
            auth_context: system.auth_context(),
        };
        
        // Get appropriate sync strategy
        let strategy = self.sync_model.get_strategy_for_context(&context)?;
        
        // Execute the synchronization
        let result = strategy.synchronize(&fact, &context)?;
        
        // Update sync metrics
        self.sync_model.metrics.record_sync_attempt(&context, &result);
        
        Ok(result)
    }
    
    /// Synchronize a batch of facts
    pub fn synchronize_facts(
        &self,
        fact_ids: &[FactId],
        target_domain: DomainId,
        mode: SyncMode,
    ) -> Result<BatchSyncResult, SyncError> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for fact_id in fact_ids {
            match self.synchronize_fact(*fact_id, target_domain, mode) {
                Ok(result) => {
                    if result.is_success() {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                    results.push((*fact_id, Ok(result)));
                }
                Err(err) => {
                    failure_count += 1;
                    results.push((*fact_id, Err(err)));
                }
            }
        }
        
        Ok(BatchSyncResult {
            results,
            success_count,
            failure_count,
        })
    }
}
```

### Synchronization Strategy

Defines how facts are synchronized in a specific context:

```rust
pub trait SyncStrategy: Send + Sync {
    /// Get a unique identifier for this strategy
    fn id(&self) -> &str;
    
    /// Check if this strategy applies to a given context
    fn applies_to(&self, context: &SyncContext) -> bool;
    
    /// Synchronize a fact according to this strategy
    fn synchronize(&self, fact: &TemporalFact, context: &SyncContext) -> Result<SyncResult, SyncError>;
}
```

### Cross-Domain Messenger

Handles communication between domains:

```rust
pub struct CrossDomainMessenger {
    /// Transport implementations
    transports: HashMap<DomainId, Box<dyn CrossDomainTransport>>,
    
    /// Message serializers
    serializers: HashMap<MessageFormat, Box<dyn MessageSerializer>>,
    
    /// Message security handlers
    security_handlers: Vec<Box<dyn MessageSecurityHandler>>,
}

impl CrossDomainMessenger {
    /// Send a fact synchronization message to another domain
    pub fn send_fact_sync_message(
        &self,
        target_domain: DomainId,
        message: FactSyncMessage,
    ) -> Result<MessageId, MessengerError> {
        // Get transport for target domain
        let transport = self.get_transport(target_domain)?;
        
        // Prepare the message
        let prepared_message = self.prepare_message(target_domain, message)?;
        
        // Send the message
        transport.send_message(prepared_message)
    }
    
    /// Handle an incoming fact synchronization message
    pub fn handle_fact_sync_message(
        &self,
        message: IncomingMessage,
    ) -> Result<(), MessengerError> {
        // Verify and deserialize the message
        let (source_domain, sync_message) = self.verify_and_deserialize(message)?;
        
        match sync_message {
            SyncMessage::FactSync { fact, .. } => {
                // Process incoming fact
                system.fact_synchronizer()
                    .process_remote_fact(fact, source_domain)?;
            }
            SyncMessage::FactSyncRequest { fact_id, .. } => {
                // Handle fact sync request
                self.handle_fact_sync_request(fact_id, source_domain)?;
            }
            SyncMessage::FactSyncResponse { fact, .. } => {
                // Process fact sync response
                system.fact_synchronizer()
                    .process_remote_fact(fact, source_domain)?;
            }
            // Other message types...
        }
        
        Ok(())
    }
}
```

## Synchronization Strategies

### Push Strategy

Proactively pushes facts to other domains:

```rust
pub struct PushSyncStrategy {
    strategy_id: String,
    context_pattern: SyncContextKey,
    batch_size: usize,
    retry_policy: RetryPolicy,
}

impl SyncStrategy for PushSyncStrategy {
    fn id(&self) -> &str {
        &self.strategy_id
    }
    
    fn applies_to(&self, context: &SyncContext) -> bool {
        self.context_pattern.matches(context)
    }
    
    fn synchronize(&self, fact: &TemporalFact, context: &SyncContext) -> Result<SyncResult, SyncError> {
        // Create sync message
        let sync_message = SyncMessage::FactSync {
            fact: fact.clone(),
            source_domain: context.source_domain,
            timestamp: system.current_time(),
        };
        
        // Send the message
        let message_id = system.cross_domain_messenger()
            .send_fact_sync_message(context.target_domain, sync_message)?;
        
        // Record the synchronization
        system.fact_sync_registry().record_sync(
            fact.id,
            context.target_domain,
            system.current_time(),
            SyncStatus::Sent,
        )?;
        
        Ok(SyncResult {
            fact_id: fact.id,
            message_id,
            status: SyncStatus::Sent,
            timestamp: system.current_time(),
        })
    }
}
```

### Pull Strategy

Requests facts from other domains:

```rust
pub struct PullSyncStrategy {
    strategy_id: String,
    context_pattern: SyncContextKey,
    polling_interval: Duration,
}

impl SyncStrategy for PullSyncStrategy {
    fn id(&self) -> &str {
        &self.strategy_id
    }
    
    fn applies_to(&self, context: &SyncContext) -> bool {
        self.context_pattern.matches(context)
    }
    
    fn synchronize(&self, fact: &TemporalFact, context: &SyncContext) -> Result<SyncResult, SyncError> {
        // Create sync request message
        let sync_message = SyncMessage::FactSyncRequest {
            fact_id: fact.id,
            source_domain: context.source_domain,
            timestamp: system.current_time(),
        };
        
        // Send the request
        let message_id = system.cross_domain_messenger()
            .send_fact_sync_message(context.target_domain, sync_message)?;
        
        // Record the synchronization request
        system.fact_sync_registry().record_sync(
            fact.id,
            context.target_domain,
            system.current_time(),
            SyncStatus::Requested,
        )?;
        
        Ok(SyncResult {
            fact_id: fact.id,
            message_id,
            status: SyncStatus::Requested,
            timestamp: system.current_time(),
        })
    }
}
```

### Event-Driven Strategy

Synchronizes facts based on events:

```rust
pub struct EventDrivenSyncStrategy {
    strategy_id: String,
    context_pattern: SyncContextKey,
    event_triggers: Vec<EventTrigger>,
}

impl SyncStrategy for EventDrivenSyncStrategy {
    fn id(&self) -> &str {
        &self.strategy_id
    }
    
    fn applies_to(&self, context: &SyncContext) -> bool {
        self.context_pattern.matches(context)
    }
    
    fn synchronize(&self, fact: &TemporalFact, context: &SyncContext) -> Result<SyncResult, SyncError> {
        // Check if any trigger applies
        let trigger = self.event_triggers.iter()
            .find(|t| t.applies_to(fact));
        
        if let Some(trigger) = trigger {
            // Create sync message with trigger information
            let sync_message = SyncMessage::FactSync {
                fact: fact.clone(),
                source_domain: context.source_domain,
                timestamp: system.current_time(),
                trigger: Some(trigger.id.clone()),
            };
            
            // Send the message
            let message_id = system.cross_domain_messenger()
                .send_fact_sync_message(context.target_domain, sync_message)?;
            
            // Record the synchronization
            system.fact_sync_registry().record_sync(
                fact.id,
                context.target_domain,
                system.current_time(),
                SyncStatus::Sent,
            )?;
            
            Ok(SyncResult {
                fact_id: fact.id,
                message_id,
                status: SyncStatus::Sent,
                timestamp: system.current_time(),
            })
        } else {
            // No trigger, so don't synchronize
            Ok(SyncResult {
                fact_id: fact.id,
                message_id: MessageId::default(),
                status: SyncStatus::Skipped,
                timestamp: system.current_time(),
            })
        }
    }
}
```

## Conflict Resolution

### Conflict Detection

Detects conflicts between facts:

```rust
pub struct ConflictDetector {
    /// Conflict detection rules
    rules: Vec<Box<dyn ConflictRule>>,
}

impl ConflictDetector {
    /// Detect conflicts between a local and remote fact
    pub fn detect_conflicts(
        &self,
        local_fact: &TemporalFact,
        remote_fact: &TemporalFact,
    ) -> Result<Vec<Conflict>, ConflictError> {
        let mut conflicts = Vec::new();
        
        // Apply each rule
        for rule in &self.rules {
            if rule.applies_to(local_fact, remote_fact) {
                if let Some(conflict) = rule.check_conflict(local_fact, remote_fact)? {
                    conflicts.push(conflict);
                }
            }
        }
        
        Ok(conflicts)
    }
}
```

### Conflict Resolution

Resolves conflicts between facts:

```rust
pub struct ConflictResolver {
    /// Resolution strategies
    strategies: Vec<Box<dyn ConflictResolutionStrategy>>,
    
    /// Resolution policies
    policies: ResolutionPolicies,
}

impl ConflictResolver {
    /// Resolve conflicts between facts
    pub fn resolve_conflicts(
        &self,
        local_fact: &TemporalFact,
        remote_fact: &TemporalFact,
        conflicts: &[Conflict],
    ) -> Result<ResolutionResult, ResolutionError> {
        // Find applicable strategy
        for strategy in &self.strategies {
            if strategy.applies_to(local_fact, remote_fact, conflicts) {
                return strategy.resolve(local_fact, remote_fact, conflicts);
            }
        }
        
        // Use default policy if no strategy applies
        self.apply_default_policy(local_fact, remote_fact, conflicts)
    }
}
```

## Synchronization Protocols

### Basic Synchronization Protocol

Simple push of facts to another domain:

```rust
/// Send a fact to another domain
pub fn send_fact_to_domain(
    fact_id: FactId,
    target_domain: DomainId,
) -> Result<SyncResult, SyncError> {
    system.fact_synchronizer().synchronize_fact(
        fact_id,
        target_domain,
        SyncMode::Push,
    )
}
```

### Request-Response Protocol

Request a fact from another domain:

```rust
/// Request a fact from another domain
pub fn request_fact_from_domain(
    fact_id: FactId,
    source_domain: DomainId,
) -> Result<Option<TemporalFact>, SyncError> {
    // Create request message
    let request = SyncMessage::FactSyncRequest {
        fact_id,
        source_domain: system.domain_id(),
        timestamp: system.current_time(),
    };
    
    // Send request and wait for response
    let response = system.cross_domain_messenger()
        .send_and_wait_response(source_domain, request, Duration::from_secs(10))?;
    
    // Process response
    if let SyncMessage::FactSyncResponse { fact, .. } = response {
        // Register the fact locally
        let mut registry = system.fact_registry().write().unwrap();
        registry.register_fact(fact.clone())?;
        
        Ok(Some(fact))
    } else {
        Ok(None)
    }
}
```

### Batch Synchronization Protocol

Synchronize multiple facts at once:

```rust
/// Synchronize a batch of facts with another domain
pub fn synchronize_fact_batch(
    query: FactQuery,
    target_domain: DomainId,
) -> Result<BatchSyncResult, SyncError> {
    // Query for facts to sync
    let facts = {
        let registry = system.fact_registry().read().unwrap();
        registry.query_facts(query, None)?
    };
    
    // Get fact IDs
    let fact_ids: Vec<FactId> = facts.iter().map(|f| f.id).collect();
    
    // Synchronize the batch
    system.fact_synchronizer().synchronize_facts(
        &fact_ids,
        target_domain,
        SyncMode::Push,
    )
}
```

### Subscription Protocol

Subscribe to facts from another domain:

```rust
/// Subscribe to facts from another domain
pub fn subscribe_to_domain_facts(
    source_domain: DomainId,
    filter: FactFilter,
) -> Result<SubscriptionId, SyncError> {
    // Create subscription message
    let subscription = SyncMessage::SubscribeFacts {
        subscriber_domain: system.domain_id(),
        filter,
        timestamp: system.current_time(),
    };
    
    // Send subscription request
    let response = system.cross_domain_messenger()
        .send_and_wait_response(source_domain, subscription, Duration::from_secs(10))?;
    
    // Process response
    if let SyncMessage::SubscribeFactsResponse { subscription_id, .. } = response {
        // Register the subscription locally
        system.fact_sync_registry().register_subscription(
            subscription_id,
            source_domain,
            filter,
        )?;
        
        Ok(subscription_id)
    } else {
        Err(SyncError::InvalidResponse)
    }
}
```

## Security and Privacy

### Secure Synchronization

Ensures the security of synchronized facts:

```rust
/// Secure a fact before synchronization
fn secure_fact_for_sync(
    fact: &TemporalFact,
    target_domain: DomainId,
) -> Result<SecuredFactData, SecurityError> {
    // Get security policy for target domain
    let policy = system.security_policies().get_policy_for_domain(target_domain)?;
    
    // Check if fact can be shared
    if !policy.can_share_fact(fact)? {
        return Err(SecurityError::SharingNotAllowed);
    }
    
    // Encrypt sensitive data if needed
    let secured_content = if policy.requires_encryption() {
        // Get target domain's public key
        let public_key = system.key_registry().get_domain_public_key(target_domain)?;
        
        // Encrypt content
        let encrypted_data = encrypt_with_public_key(&fact.content, &public_key)?;
        
        SecuredContent::Encrypted(encrypted_data)
    } else {
        SecuredContent::Plain(fact.content.clone())
    };
    
    // Create secured fact data
    let secured_data = SecuredFactData {
        fact_id: fact.id,
        fact_type: fact.fact_type.clone(),
        timestamp: fact.timestamp,
        origin_domain: fact.origin_domain,
        content: secured_content,
        proof: fact.proof.clone(),
    };
    
    // Sign the secured data
    let signature = system.crypto_provider().sign(&secured_data.to_bytes()?)?;
    
    Ok(secured_data.with_signature(signature))
}
```

### Selective Synchronization

Controls which facts are synchronized:

```rust
/// Check if a fact should be synchronized with a domain
fn should_synchronize_fact(
    fact: &TemporalFact,
    target_domain: DomainId,
) -> Result<bool, SyncError> {
    // Get synchronization policy
    let policy = system.sync_policies().get_policy_for_domain(target_domain)?;
    
    // Check if fact type is allowed
    if !policy.allows_fact_type(&fact.fact_type) {
        return Ok(false);
    }
    
    // Check if fact is within the sync window
    let sync_window = policy.get_sync_window();
    if !sync_window.contains(fact.timestamp) {
        return Ok(false);
    }
    
    // Check if related to resources that can be shared
    if let FactType::StateChange { resource_id, .. } = &fact.fact_type {
        let resource = system.resource_registry().get_resource(*resource_id)?;
        if !policy.allows_resource_type(resource.resource_type()) {
            return Ok(false);
        }
    }
    
    Ok(true)
}
```

## Implementation Status

The current implementation status of the Fact Synchronization System:

- ✅ Core synchronization framework
- ✅ Basic push synchronization
- ✅ Cross-domain messaging infrastructure
- ⚠️ Conflict detection and resolution (partially implemented)
- ⚠️ Secure fact synchronization (partially implemented)
- ❌ Subscription-based synchronization (not yet implemented)
- ❌ Advanced synchronization policies (not yet implemented)

## Future Enhancements

Planned future enhancements for the Fact Synchronization System:

1. **Causal Consistency**: Enhanced protocols for ensuring causal consistency across domains
2. **Adaptive Synchronization**: Dynamically adjust synchronization parameters based on network conditions
3. **Differential Synchronization**: Sync only the differences between facts to reduce bandwidth
4. **Privacy-Preserving Synchronization**: Advanced techniques for sharing facts while preserving privacy
5. **Federation Protocols**: Support for federated synchronization across autonomous systems
6. **Performance Optimizations**: Improved batching, compression, and transport mechanisms 