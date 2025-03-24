# Schema Evolution System

The Causality Schema Evolution System provides a robust framework for managing schema changes over time. It ensures that data can safely evolve alongside code, preventing compatibility issues and data loss during updates.

## Core Components

### 1. Schema Definition

The foundation of the system is the `Schema` structure, which defines the shape of data:

```rust
let mut schema = Schema::new("UserProfile", "1.0.0").unwrap();
schema.add_field(SchemaField::new("name", SchemaType::String, true));
schema.add_field(SchemaField::new("age", SchemaType::Integer, false));
```

Key components:
- `Schema`: Defines a data structure with name, version, and fields
- `SchemaField`: Defines individual fields with type, required status, and default values
- `SchemaType`: Supported data types (String, Integer, Float, Boolean, etc.)
- `SchemaVersion`: Semantic versioning (Major.Minor.Patch) for schemas

### 2. Evolution Rules

Evolution rules define what types of changes are allowed:

```rust
let mut rules = EvolutionRules::new();
rules.add_rule(EvolutionRule::new(ChangeType::AddOptionalField))
     .add_rule(EvolutionRule::new(ChangeType::RemoveUnusedField));
```

Key components:
- `EvolutionRules`: A set of allowed evolution operations
- `EvolutionRule`: A single rule for schema evolution
- `ChangeType`: Types of changes (AddOptionalField, RemoveField, etc.)
- `SchemaChange`: A concrete change to apply to a schema

### 3. Migration Engine

The migration engine handles transforming data between schema versions:

```rust
let engine = MigrationEngine::new(MigrationStrategy::Automatic);
let migrated_data = engine.migrate(old_data, &old_schema, &new_schema)?;
```

Key components:
- `MigrationEngine`: Manages migration between schema versions
- `MigrationStrategy`: Strategy for migration (Automatic, UserDefined, etc.)
- `MigrationHandler`: Custom migration logic for complex changes

### 4. User-Defined Migrations

For complex migrations, you can register custom handlers:

```rust
let registry = MigrationRegistry::new();
registry.register_fn(
    "UserProfile", "1.0.0",
    "UserProfile", "2.0.0",
    my_migration_function
)?;
```

Key components:
- `MigrationRegistry`: Registry for user-defined migrations
- `SharedMigrationRegistry`: Thread-safe shared registry
- `MigrationFn`: Function type for migration handlers

### 5. Safe State Management

The safe state system ensures schema changes happen only in safe states:

```rust
let manager = SafeStateManager::new(SafeStateOptions::default());
let transaction = SchemaTransaction::new(&mut schema, &manager)?;
// Make changes to schema
transaction.commit()?;
```

Key components:
- `SafeStateManager`: Manages safe state for domains
- `SafeStateStrategy`: Strategy for determining safe state
- `SchemaTransaction`: Transactional schema updates with rollback

## Usage Examples

### Basic Schema Creation

```rust
// Create a schema
let mut schema = Schema::new("UserProfile", "1.0.0")?;

// Add fields
schema.add_field(SchemaField::new("name", SchemaType::String, true));
schema.add_field(SchemaField::new("email", SchemaType::String, true));
schema.add_field(SchemaField::new("age", SchemaType::Integer, false));

// Serialize to JSON
let json = schema.to_json()?;
```

### Adding a Field with Evolution Rules

```rust
// Define allowed evolution rules
let mut rules = EvolutionRules::new();
rules.add_rule(EvolutionRule::new(ChangeType::AddOptionalField));

// Define a change
let change = SchemaChange::new(ChangeType::AddOptionalField, "bio")
    .with_new_type(SchemaType::String);

// Create changes list
let changes = vec![change];

// Apply the changes
apply_changes(&mut schema, &changes, &rules)?;

// Schema is now version 1.1.0 with a new "bio" field
```

### Using the Migration Engine

```rust
// Create migration engine with automatic strategy
let engine = MigrationEngine::new(MigrationStrategy::Automatic);

// Migrate data from old schema to new schema
let new_data = engine.migrate(old_data, &old_schema, &new_schema)?;
```

### Custom Migration Handler

```rust
// Define a migration function
fn migrate_user_v1_to_v2(value: Value, _source: &Schema, _target: &Schema) -> Result<Value> {
    let mut obj = value.as_object().unwrap().clone();
    
    // Split name into first_name and last_name
    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        let parts: Vec<&str> = name.split_whitespace().collect();
        let (first, last) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1..].join(" "))
        } else {
            (name.to_string(), String::new())
        };
        
        obj.remove("name");
        obj.insert("first_name".to_string(), json!(first));
        obj.insert("last_name".to_string(), json!(last));
    }
    
    Ok(Value::Object(obj))
}

// Register the migration handler
let registry = SharedMigrationRegistry::new();
registry.register_fn(
    "UserProfile", "1.0.0",
    "UserProfile", "2.0.0",
    migrate_user_v1_to_v2
)?;

// Create migration engine with user-defined strategy
let engine = MigrationEngine::new(MigrationStrategy::UserDefined);

// Migrate data using the registered handler
let new_data = engine.migrate(old_data, &old_schema, &new_schema)?;
```

### Safe State Management

```rust
// Create a safe state manager
let options = SafeStateOptions {
    strategy: SafeStateStrategy::NoInFlightOperations,
    ..Default::default()
};
let manager = SafeStateManager::new(options);

// Create a schema transaction
let mut transaction = SchemaTransaction::new(&mut schema, &manager)?;

// Make changes to the schema
transaction.schema_mut().add_field(SchemaField::new(
    "address",
    SchemaType::String,
    false
));

// Validate and commit the transaction
transaction.validate()?;
transaction.commit()?;
```

## Best Practices

1. **Use Semantic Versioning**: Follow semantic versioning rules for schema changes:
   - Major version for breaking changes
   - Minor version for backward-compatible additions
   - Patch version for backward-compatible fixes

2. **Prefer Safe Changes**: When possible, use safe changes that don't break backward compatibility:
   - Adding optional fields
   - Making required fields optional
   - Adding enum variants

3. **Document Migrations**: Document all schema changes and migrations, especially custom migrations.

4. **Test Migrations**: Always test migrations with realistic data before applying in production.

5. **Use Transactions**: Use schema transactions to ensure atomicity of schema updates.

6. **Monitor Safe State**: Ensure system is in a safe state before applying schema changes.

## Conclusion

The Schema Evolution System provides a robust framework for managing schema changes over time. By following these patterns and best practices, you can safely evolve your data schemas alongside your code, preventing compatibility issues and data loss during updates. 