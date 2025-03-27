# Domain Integration Layer

The Domain Integration Layer provides a comprehensive framework for integrating various domains within the Causality system, enabling seamless communication, resource sharing, and effect handling across domain boundaries.

## Core Components

### Domain Adapter Factory

The `DomainAdapterFactory` is a central component that creates domain-specific adapters for both effect handling and resource operations:

```rust
#[async_trait]
pub trait DomainAdapterFactory: Send + Sync + Debug {
    /// Create a domain effect handler for the specified domain
    async fn create_effect_handler(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainEffectHandler>>;
    
    /// Create a domain resource adapter for the specified domain
    async fn create_resource_adapter(
        &self,
        domain_id: &DomainId,
    ) -> DomainIntegrationResult<Arc<dyn DomainResourceAdapter>>;
    
    /// Get supported domains
    fn supported_domains(&self) -> Vec<DomainId>;
}
```

### Domain Effect Router

The `DomainEffectRouter` routes domain-specific effects to their appropriate handlers:

```rust
pub struct DomainEffectRouter {
    /// Domain handlers by domain ID
    handlers: HashMap<DomainId, Arc<dyn DomainEffectHandler>>,
    
    /// Domain adapter factory
    adapter_factory: Arc<dyn DomainAdapterFactory>,
}
```

### Domain Resource Router

The `DomainResourceRouter` routes resource operations to appropriate domain-specific adapters:

```rust
pub struct DomainResourceRouter {
    /// Domain resource adapters by domain ID
    adapters: HashMap<DomainId, Arc<dyn DomainResourceAdapter>>,
    
    /// Domain adapter factory
    adapter_factory: Arc<dyn DomainAdapterFactory>,
    
    /// Cross-domain resource protocol
    cross_domain_protocol: Arc<dyn CrossDomainResourceProtocol>,
}
```

### Generic Domain Adapter

The `GenericDomainAdapter` provides a base implementation that can be extended for specific domains:

```rust
pub struct GenericDomainAdapter {
    /// Domain ID
    domain_id: DomainId,
    
    /// Supported resource types
    supported_resource_types: Vec<ResourceTypeId>,
    
    /// Required capabilities for operations
    required_capabilities: HashMap<String, Vec<Capability>>,
    
    /// Parameter validators
    parameter_validators: Vec<Arc<dyn DomainParameterValidator>>,
    
    /// Domain capability mappings
    capability_mappings: HashMap<DomainId, DomainCapabilityMapping>,
}
```

## Key Features

### Domain-Specific Validation

The integration layer includes robust validation for:

- Resource type validation against domain-supported types
- Capability validation for domain operations
- Parameter validation for domain-specific effects

### Capability Mapping

Capabilities can be mapped between domains, enabling secure cross-domain operations:

```rust
pub struct DomainCapabilityMapping {
    /// Source domain
    pub source_domain: DomainId,
    
    /// Target domain
    pub target_domain: DomainId,
    
    /// Resource mappings from source to target domain
    pub resource_mappings: HashMap<ResourceId, ResourceId>,
    
    /// Capability mappings from source to target domain
    pub capability_mappings: HashMap<String, String>,
}
```

### Domain Effect Routing

Effects are automatically routed to the appropriate domain handler:

```rust
// Create a domain effect
let effect = DomainEffectFactory::create_domain_effect(
    domain_id,
    "effect.type",
)
.with_parameter("action", "create")
.with_parameter("resource_id", "resource-1");

// Route the effect to its handler
let outcome = effect_router.route_effect(&effect, &context).await?;
```

### Resource Operation Routing

Resource operations are routed through the appropriate domain adapters:

```rust
// Create a resource transfer operation
let operation = ResourceTransferOperation::new(
    resource_id,
    source_domain,
    target_domain,
    ResourceProjectionType::Transferred,
    VerificationLevel::Hash,
    capability,
);

// Route the operation
let reference = resource_router.route_resource_operation(&operation, &context).await?;
```

## Implementation

The integration layer is implemented with several concrete components:

- `BasicDomainAdapterFactory`: A foundational implementation of the adapter factory
- Test implementations for effect handlers and resource adapters
- Generic domain adapter base implementation

## Usage

To create and use the integration layer:

```rust
// Create the integration layer
let (effect_router, resource_router, adapter_factory) = 
    create_domain_integration_layer(cross_domain_protocol);

// Get a domain effect handler
let handler = effect_router.get_handler(&domain_id).await?;

// Get a domain resource adapter
let adapter = resource_router.get_adapter(&domain_id).await?;

// Route effects and operations
let effect_outcome = effect_router.route_effect(&effect, &context).await?;
let resource_ref = resource_router.route_resource_operation(&operation, &context).await?;
``` 