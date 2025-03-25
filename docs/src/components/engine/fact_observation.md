<!-- Observation of facts -->
<!-- Original file: docs/src/fact_observation.md -->

# Fact Observation System in Causality

## Overview

This document describes the Fact Observation System within the Causality architecture. The Fact Observation System is responsible for detecting, recording, and propagating temporal facts as they occur throughout the system. It plays a crucial role in maintaining the temporal consistency model by ensuring that all significant events are properly captured as immutable facts.

## System Architecture

### Core Components

The Fact Observation System consists of several key components:

```rust
pub struct FactObservationSystem {
    /// Fact registry for storing observed facts
    fact_registry: Arc<RwLock<FactRegistry>>,
    
    /// Observer instances for different domains and fact types
    observers: Vec<Arc<dyn FactObserver>>,
    
    /// Event bus for distributing fact notifications
    event_bus: FactEventBus,
    
    /// Validation pipeline for fact validation
    validation_pipeline: FactValidationPipeline,
    
    /// Configuration for the observation system
    config: FactObservationConfig,
}
```

### Fact Observer

A Fact Observer monitors a specific source or domain for facts:

```rust
/// Trait for fact observers that can detect and record facts
pub trait FactObserver: Send + Sync {
    /// Get the types of facts this observer can detect
    fn supported_fact_types(&self) -> Vec<FactType>;
    
    /// Get the domains this observer monitors
    fn observed_domains(&self) -> Vec<DomainId>;
    
    /// Start observing for facts
    fn start_observing(&self) -> Result<(), ObservationError>;
    
    /// Stop observing for facts
    fn stop_observing(&self) -> Result<(), ObservationError>;
    
    /// Manually observe a fact (in addition to automatic detection)
    fn observe_fact(&self, fact_data: FactData) -> Result<FactId, ObservationError>;
}
```

### Fact Event Bus

The event bus distributes fact notifications to interested parties:

```rust
pub struct FactEventBus {
    /// Subscribers to fact events
    subscribers: RwLock<HashMap<FactTypeKey, Vec<Box<dyn FactEventHandler>>>>,
    
    /// Metrics collection
    metrics: FactEventMetrics,
}

impl FactEventBus {
    /// Subscribe to events for a specific fact type
    pub fn subscribe(
        &self,
        fact_type_key: FactTypeKey,
        handler: Box<dyn FactEventHandler>,
    ) -> SubscriptionId {
        let mut subscribers = self.subscribers.write().unwrap();
        let handlers = subscribers
            .entry(fact_type_key)
            .or_insert_with(Vec::new);
        
        let id = SubscriptionId::generate();
        handlers.push(handler);
        
        id
    }
    
    /// Publish a fact event to all interested subscribers
    pub fn publish(&self, fact: &TemporalFact) -> Result<(), EventError> {
        let type_key = FactTypeKey::from(&fact.fact_type);
        let subscribers = self.subscribers.read().unwrap();
        
        if let Some(handlers) = subscribers.get(&type_key) {
            for handler in handlers {
                handler.handle_fact(fact)?;
            }
        }
        
        // Update metrics
        self.metrics.record_event(fact);
        
        Ok(())
    }
}
```

### Fact Validation Pipeline

Validates facts before they're officially recorded:

```rust
pub struct FactValidationPipeline {
    /// Fact validators for different validation aspects
    validators: Vec<Box<dyn FactValidator>>,
}

impl FactValidationPipeline {
    /// Validate a fact before registration
    pub fn validate_fact(&self, fact: &TemporalFact) -> Result<ValidationResult, ValidationError> {
        for validator in &self.validators {
            let result = validator.validate_fact(fact)?;
            if !result.is_valid() {
                return Ok(result);
            }
        }
        
        Ok(ValidationResult::Success)
    }
}
```

## Observer Types

Causality supports various specialized fact observers:

### Resource State Observer

Observes changes to resource states:

```rust
pub struct ResourceStateObserver {
    registry: Arc<ResourceRegistry>,
    fact_registry: Arc<RwLock<FactRegistry>>,
    config: ResourceObserverConfig,
}

impl FactObserver for ResourceStateObserver {
    fn supported_fact_types(&self) -> Vec<FactType> {
        vec![FactType::StateChange { 
            resource_id: ResourceId::wildcard(), 
            change_type: StateChangeType::wildcard() 
        }]
    }
    
    fn observed_domains(&self) -> Vec<DomainId> {
        vec![self.config.domain_id]
    }
    
    fn observe_fact(&self, fact_data: FactData) -> Result<FactId, ObservationError> {
        // Extract state change data
        let resource_id = fact_data.resource_id()?;
        let change_type = fact_data.get_change_type()?;
        let old_state = fact_data.get_old_state()?;
        let new_state = fact_data.get_new_state()?;
        
        // Create the fact content
        let content = FactContent::Json(serde_json::to_string(&StateChangeData {
            old_state,
            new_state,
            reason: fact_data.get_reason().unwrap_or_default(),
            timestamp: system.current_time(),
        })?);
        
        // Create the fact
        let fact_id = FactId::generate();
        let fact = TemporalFact {
            id: fact_id,
            fact_type: FactType::StateChange {
                resource_id,
                change_type,
            },
            timestamp: system.current_time(),
            origin_domain: self.config.domain_id,
            content,
            proof: self.generate_proof(&fact_data)?,
            dependencies: fact_data.get_dependencies()?,
            metadata: fact_data.get_metadata()?,
        };
        
        // Validate and register the fact
        let mut fact_registry = self.fact_registry.write().unwrap();
        fact_registry.register_fact(fact)?;
        
        Ok(fact_id)
    }
}
```

### Operation Observer

Monitors operation executions:

```rust
pub struct OperationObserver {
    operation_executor: Arc<OperationExecutor>,
    fact_registry: Arc<RwLock<FactRegistry>>,
    config: OperationObserverConfig,
}

impl FactObserver for OperationObserver {
    fn supported_fact_types(&self) -> Vec<FactType> {
        vec![FactType::Operation { 
            operation_id: OperationId::wildcard(), 
            operation_type: OperationType::wildcard() 
        }]
    }
    
    // Implementation of other methods...
}
```

### Transaction Observer

Monitors transaction results:

```rust
pub struct TransactionObserver {
    transaction_manager: Arc<TransactionManager>,
    fact_registry: Arc<RwLock<FactRegistry>>,
    config: TransactionObserverConfig,
}

impl FactObserver for TransactionObserver {
    fn supported_fact_types(&self) -> Vec<FactType> {
        vec![FactType::Transaction { 
            transaction_id: TransactionId::wildcard(),
            status: TransactionStatus::wildcard()
        }]
    }
    
    // Implementation of other methods...
}
```

### Cross-Domain Observer

Monitors cross-domain message exchanges:

```rust
pub struct CrossDomainObserver {
    cross_domain_messenger: Arc<CrossDomainMessenger>,
    fact_registry: Arc<RwLock<FactRegistry>>,
    config: CrossDomainObserverConfig,
}

impl FactObserver for CrossDomainObserver {
    fn supported_fact_types(&self) -> Vec<FactType> {
        vec![FactType::CrossDomain { 
            message_id: MessageId::wildcard(),
            target_domain: DomainId::wildcard()
        }]
    }
    
    // Implementation of other methods...
}
```

## Observation Process

### Fact Detection

Facts can be detected through various mechanisms:

1. **Direct Observation**: System components directly report facts
2. **Event Hooks**: Observers attach hooks to system events
3. **Polling**: Observers periodically check system state
4. **Log Analysis**: Observers analyze system logs

```rust
/// Register hooks into system components for automatic fact detection
pub fn register_observation_hooks(&self) -> Result<(), ObservationError> {
    // Register with resource registry for state changes
    self.resource_registry.register_state_change_hook(Box::new(|
        resource_id,
        old_state,
        new_state,
        change_reason
    | {
        let fact_data = FactData::new()
            .with_resource_id(resource_id)
            .with_change_type(StateChangeType::StateUpdate)
            .with_old_state(old_state)
            .with_new_state(new_state)
            .with_reason(change_reason);
        
        self.resource_observer.observe_fact(fact_data)
    }))?;
    
    // Register with operation executor for operation events
    self.operation_executor.register_operation_hook(Box::new(|
        operation_id,
        operation_type,
        operation_result
    | {
        let fact_data = FactData::new()
            .with_operation_id(operation_id)
            .with_operation_type(operation_type)
            .with_operation_result(operation_result);
        
        self.operation_observer.observe_fact(fact_data)
    }))?;
    
    // Additional hooks...
    
    Ok(())
}
```

### Fact Processing

Once a fact is detected, it undergoes the following process:

1. **Validation**: The fact is validated for correctness and consistency
2. **Enrichment**: Additional metadata and context are added
3. **Registration**: The fact is registered in the fact registry
4. **Publication**: The fact is published to interested subscribers

```rust
/// Process a new fact from initial detection to registration
pub fn process_fact(&self, fact_data: FactData) -> Result<FactId, ObservationError> {
    // Find appropriate observer for this fact type
    let observer = self.find_observer_for_fact(&fact_data)?;
    
    // Prepare the fact (validate, enrich)
    let fact_id = observer.observe_fact(fact_data)?;
    
    // Get the created fact
    let fact = {
        let registry = self.fact_registry.read().unwrap();
        registry.get_fact(&fact_id)
            .ok_or(ObservationError::FactNotFound(fact_id))?
            .clone()
    };
    
    // Publish the fact to subscribers
    self.event_bus.publish(&fact)?;
    
    // Process dependencies if needed
    self.process_dependencies(&fact)?;
    
    Ok(fact_id)
}
```

## Event Handling

### Fact Event Handlers

Components can subscribe to fact events:

```rust
/// Trait for handling fact events
pub trait FactEventHandler: Send + Sync {
    /// Handle a new fact
    fn handle_fact(&self, fact: &TemporalFact) -> Result<(), EventError>;
    
    /// Get the types of facts this handler is interested in
    fn interested_fact_types(&self) -> Vec<FactTypeKey>;
}
```

### Handler Example: State Projection

Updates projections based on state changes:

```rust
pub struct StateProjectionHandler {
    projection_store: Arc<ProjectionStore>,
}

impl FactEventHandler for StateProjectionHandler {
    fn handle_fact(&self, fact: &TemporalFact) -> Result<(), EventError> {
        if let FactType::StateChange { resource_id, .. } = &fact.fact_type {
            if let FactContent::Json(json) = &fact.content {
                let change_data: StateChangeData = serde_json::from_str(json)
                    .map_err(|e| EventError::InvalidFactContent(e.to_string()))?;
                
                // Update projections
                self.projection_store.update_projections(
                    *resource_id,
                    &change_data.new_state,
                    fact.timestamp,
                )?;
            }
        }
        
        Ok(())
    }
    
    fn interested_fact_types(&self) -> Vec<FactTypeKey> {
        vec![FactTypeKey::StateChange]
    }
}
```

### Handler Example: Notification Dispatcher

Sends notifications about important facts:

```rust
pub struct NotificationDispatcher {
    notification_service: Arc<NotificationService>,
    config: NotificationConfig,
}

impl FactEventHandler for NotificationDispatcher {
    fn handle_fact(&self, fact: &TemporalFact) -> Result<(), EventError> {
        // Check if this fact type should trigger notifications
        if !self.config.should_notify_for_fact_type(&fact.fact_type) {
            return Ok(());
        }
        
        // Create notification
        let notification = self.create_notification_from_fact(fact)?;
        
        // Send notification
        self.notification_service.send_notification(notification)?;
        
        Ok(())
    }
    
    fn interested_fact_types(&self) -> Vec<FactTypeKey> {
        self.config.notifiable_fact_types()
    }
}
```

## Fact Data Collection

### Fact Collection Context

Provides context for fact collection:

```rust
pub struct FactCollectionContext {
    /// Current domain
    domain_id: DomainId,
    
    /// Current timestamp
    timestamp: Timestamp,
    
    /// Authentication context
    auth_context: Option<AuthContext>,
    
    /// Additional collection parameters
    parameters: HashMap<String, Value>,
}
```

### Fact Data Builder

Simplifies construction of fact data:

```rust
pub struct FactData {
    /// Data elements
    elements: HashMap<String, Value>,
    
    /// Collection context
    context: FactCollectionContext,
}

impl FactData {
    /// Create a new fact data instance
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            context: FactCollectionContext::current(),
        }
    }
    
    /// Add resource ID
    pub fn with_resource_id(mut self, resource_id: ResourceId) -> Self {
        self.elements.insert("resource_id".to_string(), resource_id.into());
        self
    }
    
    /// Add operation ID
    pub fn with_operation_id(mut self, operation_id: OperationId) -> Self {
        self.elements.insert("operation_id".to_string(), operation_id.into());
        self
    }
    
    /// Add change type
    pub fn with_change_type(mut self, change_type: StateChangeType) -> Self {
        self.elements.insert("change_type".to_string(), change_type.into());
        self
    }
    
    /// Add old state
    pub fn with_old_state(mut self, old_state: ResourceState) -> Self {
        self.elements.insert("old_state".to_string(), old_state.into());
        self
    }
    
    /// Add new state
    pub fn with_new_state(mut self, new_state: ResourceState) -> Self {
        self.elements.insert("new_state".to_string(), new_state.into());
        self
    }
    
    /// Add change reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.elements.insert("reason".to_string(), reason.into().into());
        self
    }
    
    // Additional methods...
}
```

## Advanced Features

### Fact Stream Processing

Process streams of facts for analysis:

```rust
pub struct FactStreamProcessor {
    /// Processing configuration
    config: StreamProcessingConfig,
    
    /// Stream transformations
    transformations: Vec<Box<dyn FactStreamTransformation>>,
    
    /// Stream output handlers
    output_handlers: Vec<Box<dyn StreamOutputHandler>>,
}

impl FactStreamProcessor {
    /// Process a stream of facts
    pub fn process_stream(
        &self,
        fact_stream: impl Stream<Item = TemporalFact>,
    ) -> impl Stream<Item = ProcessedFactResult> {
        fact_stream
            .filter(|fact| self.config.should_process_fact(fact))
            .map(|fact| {
                // Apply transformations
                let mut transformed_fact = fact;
                for transformation in &self.transformations {
                    transformed_fact = transformation.transform(transformed_fact)?;
                }
                
                // Process the transformed fact
                let result = self.process_fact(&transformed_fact)?;
                
                // Handle output
                for handler in &self.output_handlers {
                    handler.handle_output(&result)?;
                }
                
                Ok(result)
            })
    }
}
```

### Temporal Pattern Detection

Detect patterns in sequences of facts:

```rust
pub struct TemporalPatternDetector {
    /// Pattern definitions
    patterns: Vec<TemporalPattern>,
    
    /// Notification handlers for detected patterns
    notification_handlers: Vec<Box<dyn PatternNotificationHandler>>,
}

impl TemporalPatternDetector {
    /// Check for patterns in a sequence of facts
    pub fn detect_patterns(
        &self,
        facts: &[TemporalFact],
    ) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        
        for pattern in &self.patterns {
            if let Some(pattern_match) = pattern.match_facts(facts) {
                matches.push(pattern_match);
                
                // Notify handlers
                for handler in &self.notification_handlers {
                    handler.handle_pattern_match(&pattern_match);
                }
            }
        }
        
        matches
    }
}
```

## Implementation Status

The current implementation status of the Fact Observation System:

- ✅ Core observation framework
- ✅ Basic fact observers for resources and operations
- ✅ Fact event bus
- ⚠️ Fact validation pipeline (partially implemented)
- ⚠️ Cross-domain observation (partially implemented)
- ⚠️ Stream processing (early implementation)
- ❌ Temporal pattern detection (not yet implemented)
- ❌ Advanced notification system (not yet implemented)

## Future Enhancements

Planned future enhancements for the Fact Observation System:

1. **Real-time Analytics**: Advanced real-time analytics on fact streams
2. **Machine Learning Integration**: ML-based pattern detection and anomaly detection
3. **Adaptive Observation**: Dynamic adjustment of observation parameters based on system load
4. **Distributed Fact Collection**: Improved collection of facts across distributed nodes
5. **Privacy-Preserving Observation**: Observation mechanisms that preserve data privacy
6. **Custom Fact Types**: Framework for defining and observing custom domain-specific facts 