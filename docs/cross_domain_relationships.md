# Cross-domain Relationships

Causality's cross-domain relationship system provides a powerful mechanism for managing connections between resources that exist in different domains. This document outlines the core concepts, architecture, and usage of the cross-domain relationship system.

## Overview

Resources definitions will often span multiple domains, while maintaining a logical relationship. Cross-domain relationships formalize these connections, enabling:

- **Resource Mirroring**: Maintaining identical resources across domains
- **Resource References**: Allowing resources to reference each other across domain boundaries
- **Ownership Chains**: Establishing hierarchical ownership relationships spanning domains
- **Derived Resources**: Creating resources in one domain based on resources in another
- **Bridge Resources**: Connecting resources across domain boundaries
- **Custom Relationships**: Defining domain-specific relationship types

## Standard Relationship Types

| Type | Description | Requires Sync | Typically Bidirectional |
|------|-------------|---------------|-------------------------|
| Mirror | Identical copies of a resource across domains | Yes | Yes (implicit) |
| Reference | One resource references another | Optional | Often |
| Ownership | Source resource owns target resource | Optional | No |
| Derived | Target is derived from source | Yes | No |
| Bridge | Connects resources across domains | Yes | Often |
| Custom | User-defined relationship type | Configurable | Configurable |

## Architecture

The cross-domain relationship system is comprised of several modular components:

| Relationship Definition                         | Relationship Management                     | Relationship Validation                  | Synchronization Mechanism                    |
|-------------------------------------------------|---------------------------------------------|------------------------------------------|-----------------------------------------------|
| • Types<br>• Metadata<br>• Properties           | • Storage<br>• Indexing<br>• CRUD Ops       | • Validators<br>• Rules<br>• Reporting   | • Sync Manager<br>• Sync Handlers<br>• Scheduler |


### Key Components

#### CrossDomainRelationship

The core data structure representing a relationship between resources across domains.

```rust
pub struct CrossDomainRelationship {
    pub id: String,                            // Unique identifier
    pub source_resource: String,               // Source resource ID
    pub source_domain: String,                 // Source domain ID
    pub target_resource: String,               // Target resource ID
    pub target_domain: String,                 // Target domain ID
    pub relationship_type: CrossDomainRelationshipType, // Type of relationship
    pub metadata: CrossDomainMetadata,         // Additional metadata
    pub bidirectional: bool,                   // Whether the relationship is bidirectional
}
```

#### CrossDomainRelationshipManager

Manages the lifecycle of cross-domain relationships, including creation, retrieval, and removal.

```rust
pub struct CrossDomainRelationshipManager {
    // Internal storage and indexes
}

impl CrossDomainRelationshipManager {
    // Create a new manager
    pub fn new() -> Self { ... }
    
    // Add a relationship
    pub fn add_relationship(&self, relationship: CrossDomainRelationship) -> Result<()> { ... }
    
    // Get a relationship by ID
    pub fn get_relationship(&self, id: &str) -> Result<CrossDomainRelationship> { ... }
    
    // Get relationships by various criteria
    pub fn get_relationships_by_source_domain(&self, domain: String) -> Result<Vec<CrossDomainRelationship>> { ... }
    
    // Remove a relationship
    pub fn remove_relationship(&self, id: &str) -> Result<()> { ... }
}
```

#### CrossDomainRelationshipValidator

Validates relationships according to configurable rules and validation levels.

```rust
pub struct CrossDomainRelationshipValidator {
    // Validation configuration
}

impl CrossDomainRelationshipValidator {
    // Validate a relationship
    pub fn validate(
        &self, 
        relationship: &CrossDomainRelationship, 
        level: Option<ValidationLevel>
    ) -> Result<ValidationResult> { ... }
}
```

#### CrossDomainSyncManager

Manages synchronization between resources connected by cross-domain relationships.

```rust
pub struct CrossDomainSyncManager {
    // Sync handlers and state
}

impl CrossDomainSyncManager {
    // Synchronize a specific relationship
    pub fn sync_relationship(
        &self,
        relationship: &CrossDomainRelationship,
        direction: SyncDirection,
        options: SyncOptions,
    ) -> Result<SyncResult> { ... }
    
    // Synchronize all relationships meeting criteria
    pub fn sync_all_relationships(
        &self,
        options: SyncOptions,
    ) -> Result<Vec<SyncResult>> { ... }
}
```

#### CrossDomainSyncScheduler

Schedules and manages automated synchronization of relationships.

```rust
pub struct CrossDomainSyncScheduler {
    // Scheduling configuration and state
}

impl CrossDomainSyncScheduler {
    // Start the scheduler
    pub fn start(&self) -> Result<()> { ... }
    
    // Stop the scheduler
    pub fn stop(&self) -> Result<()> { ... }
    
    // Pause the scheduler
    pub fn pause(&self) -> Result<()> { ... }
    
    // Schedule a specific relationship for immediate sync
    pub fn schedule_immediate_sync(
        &self,
        relationship_id: &str,
        priority: usize,
    ) -> Result<()> { ... }
}
```

## Synchronization Strategies

The system supports several synchronization strategies:

1. **Periodic**: Resources are synchronized at regular intervals
   ```rust
   SyncStrategy::Periodic(Duration::from_secs(3600)) // Every hour
   ```

2. **Event-Driven**: Resources are synchronized when events occur
   ```rust
   SyncStrategy::EventDriven
   ```

3. **Hybrid**: Combines event-driven with periodic fallback
   ```rust
   SyncStrategy::Hybrid(Duration::from_secs(86400)) // Fallback: daily
   ```

4. **Manual**: Resources are only synchronized when explicitly requested
   ```rust
   SyncStrategy::Manual
   ```

## Validation Levels

Relationships can be validated at different levels of strictness:

1. **Strict**: All rules are enforced
2. **Moderate**: Only critical rules are enforced
3. **Permissive**: Validation produces warnings but rarely fails

## CLI Commands

The cross-domain relationship system includes a CLI for management:

```
causality relationship create [options] <source-resource> <source-domain> <target-resource> <target-domain> <type>
causality relationship list [--domain <domain>] [--resource <resource>] [--type <type>]
causality relationship get <relationship-id>
causality relationship delete <relationship-id>
causality relationship validate <relationship-id> [--level <level>]
causality relationship sync <relationship-id> [--direction <direction>] [--force]
causality relationship scheduler [start|stop|pause|resume] [--config <config-file>]
```

## Examples

### Creating and Synchronizing Relationships

```rust
// Import necessary types
use causality::resource::relationship::{
    CrossDomainRelationship,
    CrossDomainRelationshipType,
    CrossDomainMetadata,
    SyncStrategy,
};
use std::time::Duration;

// Create a mirror relationship
let metadata = CrossDomainMetadata {
    origin_domain: "ethereum".to_string(),
    target_domain: "solana".to_string(),
    requires_sync: true,
    sync_strategy: SyncStrategy::Periodic(Duration::from_secs(3600)), // Hourly
};

let relationship = CrossDomainRelationship::new(
    "token:0x1234".to_string(),
    "ethereum".to_string(),
    "token:A9B8C7".to_string(),
    "solana".to_string(),
    CrossDomainRelationshipType::Mirror,
    metadata,
    false, // Not explicitly bidirectional (mirror is implicitly bidirectional)
);

// Add to relationship manager
relationship_manager.add_relationship(relationship.clone())?;

// Synchronize relationship
let result = sync_manager.sync_relationship(
    &relationship,
    SyncDirection::SourceToTarget,
    SyncOptions::default(),
)?;

// Check result
match result {
    SyncResult::Success { .. } => println!("Synchronization successful"),
    SyncResult::Skipped { reason, .. } => println!("Synchronization skipped: {}", reason),
    SyncResult::InProgress { .. } => println!("Synchronization in progress"),
}
```

### Using the Scheduler

```rust
// Create a scheduler
let scheduler = CrossDomainSyncScheduler::new(
    Arc::new(relationship_manager),
    Arc::new(sync_manager),
);

// Configure the scheduler
{
    let mut config = scheduler.config.write().unwrap();
    config.max_concurrent_tasks = 5;
    config.periodic_check_interval = Duration::from_secs(30);
    config.retry_failed = true;
    config.max_retry_attempts = 3;
    config.retry_backoff = RetryBackoff::Exponential {
        initial: Duration::from_secs(5),
        max: Duration::from_secs(300),
        multiplier: 2.0,
    };
}

// Start the scheduler
scheduler.start()?;

// Schedule an immediate sync for a specific relationship
scheduler.schedule_immediate_sync(&relationship.id, 1)?; // priority 1 (high)

// Later, when done:
scheduler.stop()?;
```

## API Reference

### Key Structs

#### CrossDomainRelationship

```rust
pub struct CrossDomainRelationship {
    pub id: String,
    pub source_resource: String,
    pub source_domain: String,
    pub target_resource: String,
    pub target_domain: String,
    pub relationship_type: CrossDomainRelationshipType,
    pub metadata: CrossDomainMetadata,
    pub bidirectional: bool,
}
```

#### CrossDomainMetadata

```rust
pub struct CrossDomainMetadata {
    pub origin_domain: String,
    pub target_domain: String,
    pub requires_sync: bool,
    pub sync_strategy: SyncStrategy,
}
```

### Key Enums

#### CrossDomainRelationshipType

```rust
pub enum CrossDomainRelationshipType {
    Mirror,
    Reference,
    Ownership,
    Derived,
    Bridge,
    Custom(String),
}
```

#### SyncStrategy

```rust
pub enum SyncStrategy {
    OneTime,
    Periodic(Duration),
    EventDriven,
    Hybrid(Duration),
    Manual,
}
```

#### ValidationLevel

```rust
pub enum ValidationLevel {
    Strict,
    Moderate,
    Permissive,
}
```

## Best Practices

1. **Choose the Appropriate Relationship Type**: Use the relationship type that most accurately represents the connection between resources.

2. **Set Appropriate Sync Strategy**: Consider the frequency of changes and the criticality of consistency when setting the sync strategy.

3. **Handle Bidirectionality Carefully**: Bidirectional relationships require careful consideration of update conflicts and circular references.

4. **Validate Before Syncing**: Always validate relationships before synchronizing to ensure they meet all requirements.

5. **Monitor Sync Operations**: Keep track of synchronization operations and set up alerts for failed syncs.

6. **Tune Scheduler Parameters**: Adjust the scheduler parameters based on system load and performance requirements.

7. **Document Domain-Specific Relationships**: When using custom relationship types, document their meaning and intended use.

## Conclusion

Cross-domain relationships provide a powerful foundation for modeling complex multi-domain systems while maintaining consistency across domain boundaries. By using the appropriate relationship types, validation levels, and synchronization strategies, you can create robust systems that span multiple domains while ensuring data integrity and consistency. 