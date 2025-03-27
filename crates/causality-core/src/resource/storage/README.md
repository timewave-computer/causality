# Resource Storage

The Resource Storage system in Causality provides content-addressed storage capabilities for resources, including versioning, indexing, and retrieval features. This system ensures that resources are stored efficiently and can be accessed using their unique content-based identifiers.

## Core Components

### ResourceStorage Trait

The `ResourceStorage` trait defines the interface for storing and retrieving resources:

- **Store Operations**: Store resources with their type information and metadata
- **Retrieval Operations**: Get resources by ID or specific versions
- **Versioning Support**: Track and access resource version history
- **Indexing Features**: Index resources by type and tags for efficient lookup
- **Metadata Management**: Associate and retrieve metadata for resources

### ContentAddressedResourceStorage

The main implementation of the `ResourceStorage` trait using content addressing principles:

- Uses an underlying `ContentAddressedStorage` for the actual data storage
- Maintains indexes for efficient lookups by resource ID, type, and tags
- Tracks version history for each resource
- Optimizes resource retrieval using content hashes

### ResourceVersion

Represents a specific version of a resource:

- Tracks version numbers (monotonically increasing)
- References content hashes for easy content retrieval
- Maintains metadata specific to each version
- Links to previous versions for history tracking

### ResourceIndexEntry

Provides indexing information for quick resource lookup:

- References current version and content hash
- Tracks resource type information
- Includes tags for categorization
- Records timestamps for creation and updates

## Usage Examples

### Storing Resources

```rust
// Create a resource
let resource = MyResource::new("example", data);

// Store with type information
let resource_id = storage.store_resource(
    resource, 
    ResourceTypeId::new("example_type"),
    Some(metadata)
).await?;
```

### Retrieving Resources

```rust
// Get latest version
let resource: MyResource = storage.get_resource(&resource_id).await?;

// Get specific version
let v1: MyResource = storage.get_resource_version(&resource_id, 1).await?;
```

### Updating Resources

```rust
// Update a resource
let updated = MyResource::new("updated", new_data);
let new_version = storage.update_resource(&resource_id, updated, None).await?;
```

### Using Tags

```rust
// Add tags for categorization
storage.add_tag(&resource_id, "important").await?;
storage.add_tag(&resource_id, "archived").await?;

// Find resources by tag
let important_resources = storage.find_resources_by_tag("important").await?;
```

## Content Addressing Benefits

The storage system leverages content addressing for several key benefits:

1. **Deduplication**: Identical resources are stored only once
2. **Integrity**: Content hashes verify resource integrity
3. **Immutability**: Resources are stored immutably, with changes creating new versions
4. **Versioning**: Resource history is preserved with links between versions

## Implementation Details

### In-Memory Implementation

The `InMemoryResourceStorage` provides a fully functional implementation for testing and development:

- Uses in-memory storage for resources
- Implements all ResourceStorage features
- Suitable for testing and prototyping

### Configurability

The system can be configured using `ResourceStorageConfig`:

- Enable/disable versioning
- Configure caching behavior
- Set limits on version history
- Performance tuning options

## Integration

The resource storage system integrates cleanly with other components of the Causality system:

- Works with the resource type registry for type validation
- Supports the cross-domain resource protocol for secure resource sharing
- Integrates with the effect system for resource modifications
- Provides content-addressed storage for domain-specific resources 