# Custom Cross-Domain Relationships

This document provides guidance on implementing custom relationship types in the cross-domain relationship system.

## Overview

While the system includes standard relationship types (Mirror, Reference, Ownership, Derived, Bridge), you may need to define custom relationship types for domain-specific use cases. Custom relationships allow you to define specialized behavior, validation rules, and synchronization strategies.

## Implementing a Custom Relationship Type

### Step 1: Define the Custom Relationship Type

First, extend the `CrossDomainRelationshipType` enum:

```rust
// In your project
pub enum MyCustomRelationshipType {
    Standard(CrossDomainRelationshipType), // Include standard types
    // Custom types
    DataFeed,          // A one-way data feed relationship
    CrossChainSwap,    // Cross-chain token swap relationship
    StateChannel,      // State channel between resources
    // Add more as needed
}
```

### Step 2: Implement Traits

Implement the necessary traits for your custom relationship type:

```rust
impl CustomRelationshipTypeBehavior for MyCustomRelationshipType {
    fn requires_synchronization(&self) -> bool {
        match self {
            Self::Standard(std_type) => std_type.requires_synchronization(),
            Self::DataFeed => true,
            Self::CrossChainSwap => true,
            Self::StateChannel => true,
        }
    }
    
    fn default_direction(&self) -> SyncDirection {
        match self {
            Self::Standard(std_type) => std_type.default_direction(),
            Self::DataFeed => SyncDirection::SourceToTarget,
            Self::CrossChainSwap => SyncDirection::Bidirectional,
            Self::StateChannel => SyncDirection::Bidirectional,
        }
    }
    
    fn suggested_validation_level(&self) -> ValidationLevel {
        match self {
            Self::Standard(std_type) => std_type.suggested_validation_level(),
            Self::DataFeed => ValidationLevel::Moderate,
            Self::CrossChainSwap => ValidationLevel::Strict,
            Self::StateChannel => ValidationLevel::Strict,
        }
    }
}
```

### Step 3: Create a Custom Validator

Implement validation logic specific to your custom relationship types:

```rust
pub struct MyCustomRelationshipValidator {
    // Configuration for validation
}

impl RelationshipValidator for MyCustomRelationshipValidator {
    fn validate(
        &self,
        relationship: &CrossDomainRelationship<MyCustomRelationshipType>,
        level: ValidationLevel,
    ) -> Result<ValidationResult> {
        match relationship.relationship_type {
            MyCustomRelationshipType::Standard(std_type) => {
                // Delegate to standard validator
                let standard_validator = StandardRelationshipValidator::new();
                standard_validator.validate(relationship, level)
            },
            MyCustomRelationshipType::DataFeed => {
                // Custom validation for data feeds
                self.validate_data_feed(relationship, level)
            },
            MyCustomRelationshipType::CrossChainSwap => {
                // Custom validation for cross-chain swaps
                self.validate_cross_chain_swap(relationship, level)
            },
            MyCustomRelationshipType::StateChannel => {
                // Custom validation for state channels
                self.validate_state_channel(relationship, level)
            },
        }
    }
}

impl MyCustomRelationshipValidator {
    fn validate_data_feed(
        &self, 
        relationship: &CrossDomainRelationship<MyCustomRelationshipType>,
        level: ValidationLevel,
    ) -> Result<ValidationResult> {
        // Implement data feed specific validation
        // ...
    }
    
    // Add other validation methods as needed
}
```

### Step 4: Create a Custom Synchronization Handler

Implement synchronization logic for your custom relationship types:

```rust
pub struct MyCustomSyncHandler {
    // Dependencies for synchronization
}

impl RelationshipSyncHandler for MyCustomSyncHandler {
    fn sync(
        &self,
        relationship: &CrossDomainRelationship<MyCustomRelationshipType>,
        direction: SyncDirection,
        options: SyncOptions,
    ) -> Result<SyncResult> {
        match relationship.relationship_type {
            MyCustomRelationshipType::Standard(std_type) => {
                // Delegate to standard handler
                let standard_handler = StandardSyncHandler::new();
                standard_handler.sync(relationship, direction, options)
            },
            MyCustomRelationshipType::DataFeed => {
                // Custom sync for data feeds
                self.sync_data_feed(relationship, direction, options)
            },
            MyCustomRelationshipType::CrossChainSwap => {
                // Custom sync for cross-chain swaps
                self.sync_cross_chain_swap(relationship, direction, options)
            },
            MyCustomRelationshipType::StateChannel => {
                // Custom sync for state channels
                self.sync_state_channel(relationship, direction, options)
            },
        }
    }
}

impl MyCustomSyncHandler {
    fn sync_data_feed(
        &self,
        relationship: &CrossDomainRelationship<MyCustomRelationshipType>,
        direction: SyncDirection,
        options: SyncOptions,
    ) -> Result<SyncResult> {
        // Implement data feed specific synchronization
        // ...
    }
    
    // Add other sync methods as needed
}
```

## Example: Implementing a Data Feed Relationship

Here's a complete example of implementing and using a Data Feed relationship type:

```rust
use causality::resource::relationship::{
    CrossDomainRelationship,
    CrossDomainMetadata,
    SyncStrategy,
    SyncDirection,
    ValidationLevel,
};
use std::time::Duration;

// 1. Define the relationship
enum MyRelationshipType {
    Standard(StandardRelationshipType),
    DataFeed,
}

// 2. Create an instance
let data_feed = CrossDomainRelationship::new(
    "price_oracle".to_string(),
    "ethereum".to_string(),
    "token_price_feed".to_string(),
    "internal_database".to_string(),
    MyRelationshipType::DataFeed,
    CrossDomainMetadata {
        origin_domain: "ethereum".to_string(),
        target_domain: "internal_database".to_string(),
        requires_sync: true,
        sync_strategy: SyncStrategy::Hybrid(Duration::from_secs(3600)),
        custom_properties: serde_json::json!({
            "update_threshold": 0.01,  // 1% price change threshold
            "data_ttl": 300,           // 5 minutes time-to-live
            "failover_sources": ["binance", "coinbase"]
        }),
    },
    false, // Not bidirectional
);

// 3. Add to relationship manager
let custom_manager = CustomRelationshipManager::new();
custom_manager.add_relationship(data_feed)?;

// 4. Use custom validator
let validator = MyCustomRelationshipValidator::new();
let validation_result = validator.validate(
    &data_feed,
    ValidationLevel::Strict,
)?;

// 5. Sync using custom handler
if validation_result.is_valid {
    let sync_handler = MyCustomSyncHandler::new();
    let sync_result = sync_handler.sync(
        &data_feed,
        SyncDirection::SourceToTarget,
        SyncOptions::default(),
    )?;
    
    println!("Sync completed: {:?}", sync_result);
}
```

## Best Practices

1. **Type Safety**: Use Rust's type system to ensure relationships are handled correctly
2. **Validation**: Implement thorough validation for custom relationship types
3. **Error Handling**: Provide detailed error information for failed operations
4. **Documentation**: Document the semantics and expected behavior of custom relationship types
5. **Testing**: Create comprehensive tests for custom validation and synchronization logic

## Advanced: Generic Relationship Types

For maximum flexibility, you can use generics to implement a system that works with any relationship type:

```rust
pub struct GenericRelationshipManager<T: RelationshipType> {
    relationships: HashMap<String, CrossDomainRelationship<T>>,
}

impl<T: RelationshipType> GenericRelationshipManager<T> {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }
    
    pub fn add_relationship(&mut self, relationship: CrossDomainRelationship<T>) -> Result<()> {
        self.relationships.insert(relationship.id.clone(), relationship);
        Ok(())
    }
    
    // Other methods...
}
```

This approach allows you to create entirely new relationship type hierarchies while reusing the core infrastructure. 