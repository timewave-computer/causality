# TEL Integration with One-Time Register System

This document describes the integration between the TEL (Temporal Effect Language) resource system and the one-time register system.

## Overview

The integration provides a direct and clean interface for TEL resources to interact with the one-time register system. Rather than creating a separate integration module or intermediate layer, we've implemented a bidirectional adapter that can:

1. Import TEL resources into the register system
2. Export registers to TEL resources
3. Synchronize state between systems in both directions
4. Process TEL operations through the register system

## Design Approach

We followed these key design principles:

1. **Bidirectional Mapping**: Full bidirectional mapping between TEL resources and registers
2. **Stateful Adapter**: The adapter maintains mapping state but doesn't own the systems it connects
3. **Explicit Synchronization**: Explicit sync operations rather than automatic sync to maintain control

## Implementation Details

### TelResourceAdapter

The `TelResourceAdapter` class provides the main integration point:

- It takes references to both systems (TEL resource manager and register system)
- It maintains mappings between TEL resource IDs and register IDs
- It provides methods to import/export and sync resources between systems
- It can process TEL operations through the register system

### Key Features

1. **ID Mapping**: Maps TEL resource IDs to register IDs and vice versa
2. **Content Conversion**: Converts between TEL register contents and register system contents
3. **Metadata Preservation**: Preserves metadata during conversion between systems
4. **State Synchronization**: Syncs register states (Active, Locked, Consumed) with TEL states
5. **Operation Processing**: Processes TEL resource operations through the register system

### Use Cases

The adapter supports these primary use cases:

1. **TEL to Register**: Import TEL resources into the register system
2. **Register to TEL**: Export registers to the TEL resource system
3. **Sync from TEL**: Update registers with changes from TEL resources
4. **Sync to TEL**: Update TEL resources with changes from registers
5. **TEL Operation Handling**: Process TEL operations through the register system

## API Reference

### Core Methods

- `import_tel_register(tel_id)`: Import a TEL resource into the register system
- `export_register_to_tel(register_id)`: Export a register to the TEL resource system
- `sync_from_tel(tel_id)`: Sync changes from a TEL resource to its register
- `sync_to_tel(register_id)`: Sync changes from a register to its TEL resource
- `process_tel_operation(operation)`: Process a TEL operation through the register system

### Conversion Methods

- `convert_tel_register_to_register(tel_register)`: Convert a TEL register to our register format
- `convert_register_to_tel_contents(register)`: Convert our register contents to TEL register contents

## Example Usage

```rust
// Create the TEL resource manager and register system
let tel_manager = Arc::new(TelResourceManager::new());
let register_system = Arc::new(OneTimeRegisterSystem::new(config)?);

// Create the TEL resource adapter
let adapter = TelResourceAdapter::new(register_system.clone(), tel_manager.clone());

// Import a TEL resource
let register_id = adapter.import_tel_register(&tel_resource_id)?;

// Update a register
register_system.update_register_contents(&register_id, new_contents, "update")?;

// Sync changes back to TEL
adapter.sync_to_tel(&register_id)?;
```

## Integration Benefits

1. **Clean Architecture**: Avoids unnecessary abstraction layers
2. **Performance**: Direct conversion without intermediate representations
3. **Maintainability**: Clear ownership boundaries and responsibilities
4. **Flexibility**: Both systems can operate independently when needed
5. **Testability**: Easy to test with clear inputs and outputs

## Future Improvements

1. **Batch Operations**: Add support for batched imports/exports and syncs
2. **Event-Based Sync**: Add optional event-driven synchronization
3. **Enhanced Filtering**: Add filtering options for syncing specific resources
4. **Optimized Conversion**: Optimize content conversion for large data
5. **Transaction Support**: Add support for transactional operations across systems 