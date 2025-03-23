# Domain Fact Observer System

This document explains the Domain Fact Observer system and its integration with the effect system in Causality.

## Overview

The Domain Fact Observer system provides a standardized way to observe facts from different blockchain domains and integrate them with the effect system. This allows effects to depend on domain facts and validate their dependencies.

## Key Components

### Domain Fact

A `DomainFact` represents a fact observed from a specific domain:

```rust
pub struct DomainFact {
    /// ID of the domain this fact was observed from
    pub domain_id: DomainId,
    
    /// Type of the fact
    pub fact_type: FactType,
    
    /// Metadata about the fact observation
    pub meta: FactObservationMeta,
    
    /// Original query that was used to observe this fact
    pub query: String,
    
    /// Parameters used in the query
    pub parameters: HashMap<String, String>,
    
    /// Hash of the fact (for verification)
    pub hash: Option<String>,
}
```

Domain facts can be converted to fact dependencies and snapshots for the effect system.

### Domain Fact Observer

The `DomainFactObserver` trait defines the interface for observing facts from domains:

```rust
#[async_trait]
pub trait DomainFactObserver: Send + Sync {
    /// Observe a fact from a domain
    async fn observe_fact(&self, query: FactQuery) -> Result<DomainFact>;
    
    /// Check if this observer supports observing a specific fact type
    fn supports_fact_type(&self, fact_type: &str) -> bool;
    
    /// Get the domain ID this observer is associated with
    fn domain_id(&self) -> &DomainId;
    
    /// Get the supported fact types
    fn supported_fact_types(&self) -> HashSet<String>;
}
```

### Adapter Fact Observer

The `AdapterFactObserver` implements the `DomainFactObserver` trait and delegates to a domain adapter:

```rust
pub struct AdapterFactObserver {
    /// The domain adapter to use for fact observation
    adapter: Arc<dyn DomainAdapter>,
    
    /// Supported fact types
    fact_types: HashSet<String>,
}
```

### Domain Fact Observer Registry

The `DomainFactObserverRegistry` manages domain fact observers and provides methods for observing facts from different domains:

```rust
pub struct DomainFactObserverRegistry {
    /// Map of domain ID to fact observer
    observers: RwLock<HashMap<DomainId, Arc<dyn DomainFactObserver>>>,
    
    /// Map of fact type to observers that support it
    fact_type_map: RwLock<HashMap<String, Vec<DomainId>>>,
}
```

### Domain Fact Effect

The `DomainFactEffect` trait is implemented by effects that depend on domain facts:

```rust
pub trait DomainFactEffect: Effect {
    /// Get the domain fact dependencies for this effect
    fn domain_fact_dependencies(&self) -> Vec<FactDependency>;
    
    /// Validate domain fact dependencies for this effect
    async fn validate_domain_fact_dependencies(
        &self, 
        registry: &DomainFactObserverRegistry
    ) -> Result<Vec<DomainFact>>;
}
```

### Observe Domain Fact Effect

The `ObserveDomainFactEffect` is an effect for observing domain facts:

```rust
pub struct ObserveDomainFactEffect {
    /// Effect ID
    id: uuid::Uuid,
    
    /// Fact query to observe
    query: FactQuery,
    
    /// Whether to cache the result
    cache_result: bool,
}
```

## Integration with Effect System

The domain fact observer system integrates with the effect system in several ways:

1. **Fact Dependencies**: Effects can declare dependencies on domain facts using the `DomainFactEffect` trait.
2. **Fact Validation**: Effects can validate their domain fact dependencies before execution.
3. **Fact Observation**: The `ObserveDomainFactEffect` allows observing domain facts as a regular effect.
4. **Fact Caching**: Observed facts can be cached as fact snapshots in the effect system.

## Usage Examples

### Creating a Domain Fact Observer

```rust
// Create a domain adapter
let cosm_adapter = CosmWasmAdapter::new(...);

// Create a fact observer from the adapter
let observer = cosm_adapter.create_fact_observer();

// Add support for specific fact types
let mut observer = AdapterFactObserver::new(Arc::new(cosm_adapter));
observer.add_fact_type("balance");
observer.add_fact_type("transaction");
```

### Registering Observers in the Registry

```rust
// Create a fact observer registry
let registry = DomainFactObserverRegistry::new();

// Register a domain fact observer
registry.register_observer(Arc::new(observer)).unwrap();
```

### Observing Facts

```rust
// Create a fact query
let query = FactQuery {
    domain_id: DomainId::new("cosmos-hub"),
    fact_type: "balance".to_string(),
    parameters: {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "cosmos1...".to_string());
        params
    },
};

// Observe a fact from a specific domain
let fact = registry.observe_fact(query).await.unwrap();

// Access the fact data
match &fact.fact_type {
    FactType::Numeric(balance) => println!("Balance: {}", balance),
    _ => println!("Unexpected fact type"),
}
```

### Using the Observe Domain Fact Effect

```rust
// Create an observe domain fact effect
let effect = ObserveDomainFactEffect::new(query);

// Add the registry to the effect context
let mut context = EffectContext::new();
context.add_service(Arc::new(registry));

// Execute the effect
let outcome = effect.execute_async(&context).await.unwrap();

// Get the result from the outcome
let value = outcome.get_data("value").unwrap();
println!("Observed value: {}", value);
```

### Implementing a Domain Fact Effect

```rust
struct MyEffect {
    id: uuid::Uuid,
    domain_id: DomainId,
    address: String,
}

impl Effect for MyEffect {
    // Standard Effect implementation...
}

impl DomainFactEffect for MyEffect {
    fn domain_fact_dependencies(&self) -> Vec<FactDependency> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), self.address.clone());
        
        vec![FactDependency {
            fact_type: "balance".to_string(),
            parameters: params,
            domain_id: Some(self.domain_id.to_string()),
        }]
    }
}
```

## Benefits

The Domain Fact Observer system provides several benefits:

1. **Standardized Interface**: A common way to observe facts from different blockchain domains.
2. **Integration with Effects**: Effects can declare dependencies on domain facts and validate them.
3. **Fact Caching**: Observed facts can be cached to improve performance.
4. **Cross-Domain Verification**: Facts from different domains can be verified against each other.
5. **Extensibility**: New domain fact observers can be added easily.

## Future Work

1. **Cross-Domain Fact Verification**: Implement verification of facts across multiple domains.
2. **Fact Streaming**: Support for streaming facts from domains in real-time.
3. **Fact Aggregation**: Combine facts from multiple domains into aggregated facts.
4. **Historical Fact Queries**: Query facts from historical blockchain state.
5. **Fact Indexing**: Index observed facts for faster retrieval. 