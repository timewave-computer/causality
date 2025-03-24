# Operation Routing in the Unified Architecture

## Overview

In the unified Causality architecture, operation routing is the process of directing operations to the appropriate execution context, domain, and handler based on their type, target resources, and capabilities. This document explains how the routing system works within the content-addressed, capability-based framework.

## Operation Routing Architecture

The operation routing system consists of several interconnected components:

```
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│ Operation       │─────>│ Route Resolver  │─────>│ Domain Registry │
│ Submission      │      │                 │      │                 │
└─────────────────┘      └─────────────────┘      └─────────────────┘
                                 │                         │
                                 │                         │
                                 ▼                         ▼
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│ Domain-Specific │<─────│ Domain Router   │<─────│ Domain Adapter  │
│ Handler         │      │                 │      │ Registry        │
└─────────────────┘      └─────────────────┘      └─────────────────┘
        │                         │
        │                         │
        ▼                         ▼
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│ Effect          │<─────│ Operation       │─────>│ Content-        │
│ Execution       │      │ Execution       │      │ Addressed Log   │
└─────────────────┘      └─────────────────┘      └─────────────────┘
```

## Unified Operation Model and Routing

The unified operation model uses `Operation<C>` with different execution context types:

```rust
pub struct Operation<C: ExecutionContext> {
    pub id: ContentRef<OperationId>,
    pub context: C,
    pub operation_type: OperationType,
    pub resources: Vec<ContentRef<ResourceRegister<C>>>,
    pub capabilities: Vec<ContentRef<Capability>>,
}
```

Routing decisions are based on:

1. **Execution Context Type**: Determines the general category of execution (on-chain, off-chain, simulation)
2. **Operation Type**: The specific kind of operation being performed
3. **Resource Domains**: The domains of the resources involved
4. **Capabilities**: The capabilities required and provided

## Route Resolution

The `RouteResolver` determines the appropriate route for an operation:

```rust
pub struct RouteResolver<C: ExecutionContext> {
    domain_registry: Arc<DomainRegistry>,
    adapter_registry: Arc<AdapterRegistry>,
}

impl<C: ExecutionContext> RouteResolver<C> {
    pub fn resolve_route(&self, operation: &Operation<C>) -> Result<Route<C>, RoutingError> {
        // 1. Extract domains from resources
        let domains = self.extract_domains(operation);
        
        // 2. Check for cross-domain operations
        let is_cross_domain = domains.len() > 1;
        
        // 3. Determine primary domain
        let primary_domain = self.determine_primary_domain(&domains, operation)?;
        
        // 4. Find adapter for primary domain
        let adapter = self.adapter_registry.get_adapter_for_domain::<C>(primary_domain)?;
        
        // 5. Determine handler within adapter
        let handler = adapter.get_handler_for_operation(operation)?;
        
        // 6. Create route
        let route = Route {
            operation_id: operation.id.clone(),
            primary_domain,
            involved_domains: domains,
            is_cross_domain,
            adapter_id: adapter.id(),
            handler_id: handler.id(),
        };
        
        Ok(route)
    }
}
```

## Content-Addressed Route References

Routes are content-addressed to ensure immutability and verifiability:

```rust
pub struct Route<C: ExecutionContext> {
    pub operation_id: ContentRef<OperationId>,
    pub primary_domain: ContentRef<Domain>,
    pub involved_domains: Vec<ContentRef<Domain>>,
    pub is_cross_domain: bool,
    pub adapter_id: ContentRef<AdapterId>,
    pub handler_id: ContentRef<HandlerId>,
}

// ContentRef implementation
impl<C: ExecutionContext> ContentAddressed for Route<C> {
    fn content_hash(&self) -> Hash {
        let mut hasher = Hasher::new();
        hasher.update(self.operation_id.content_hash().as_bytes());
        hasher.update(self.primary_domain.content_hash().as_bytes());
        // Hash other fields...
        hasher.finalize()
    }
}
```

## Domain-Based Routing

Domains are a central concept for routing operations:

```rust
pub struct Domain {
    pub id: ContentRef<DomainId>,
    pub name: String,
    pub domain_type: DomainType,
    pub properties: HashMap<String, Value>,
}

pub enum DomainType {
    EVM,
    CosmWasm,
    Succinct,
    Simulation,
    OffChain,
    CrossDomain,
}
```

The domain registry maintains mappings between domains and adapters:

```rust
pub struct DomainRegistry {
    domains: HashMap<ContentRef<DomainId>, Domain>,
    domain_adapter_mappings: HashMap<ContentRef<DomainId>, ContentRef<AdapterId>>,
}

impl DomainRegistry {
    pub fn get_domain(&self, domain_id: &ContentRef<DomainId>) -> Option<&Domain> {
        self.domains.get(domain_id)
    }
    
    pub fn get_adapter_id_for_domain(&self, domain_id: &ContentRef<DomainId>) -> Option<&ContentRef<AdapterId>> {
        self.domain_adapter_mappings.get(domain_id)
    }
}
```

## Domain Adapter Registry

The adapter registry manages domain adapters that implement domain-specific behavior:

```rust
pub struct AdapterRegistry {
    adapters: HashMap<ContentRef<AdapterId>, Box<dyn DomainAdapter>>,
}

impl AdapterRegistry {
    pub fn get_adapter_for_domain<C: ExecutionContext>(&self, domain_id: &ContentRef<DomainId>) -> Result<&dyn DomainAdapter<C>, RoutingError> {
        // Find the appropriate adapter for the domain
        let adapter_id = self.domain_registry.get_adapter_id_for_domain(domain_id)
            .ok_or(RoutingError::NoDomainAdapter(domain_id.clone()))?;
            
        let adapter = self.adapters.get(adapter_id)
            .ok_or(RoutingError::AdapterNotFound(adapter_id.clone()))?;
            
        // Cast to the specific context type
        adapter.as_context::<C>()
            .ok_or(RoutingError::IncompatibleContextType)
    }
}
```

## Domain Routers

Domain routers control the flow of operations within a domain:

```rust
pub trait DomainRouter<C: ExecutionContext> {
    fn route_operation(&self, operation: &Operation<C>, route: &Route<C>) -> Result<OperationResult<C>, RoutingError>;
}

// EVM-specific implementation
pub struct EvmDomainRouter {
    // Implementation details
}

impl<C: ExecutionContext> DomainRouter<C> for EvmDomainRouter {
    fn route_operation(&self, operation: &Operation<C>, route: &Route<C>) -> Result<OperationResult<C>, RoutingError> {
        // 1. Check that this is an EVM domain
        if route.primary_domain.domain_type != DomainType::EVM {
            return Err(RoutingError::DomainTypeMismatch);
        }
        
        // 2. Get the handler
        let handler = self.get_handler(route.handler_id)?;
        
        // 3. Execute the operation
        let result = handler.handle_operation(operation)?;
        
        // 4. Log the execution
        self.log_execution(operation, &result)?;
        
        Ok(result)
    }
}
```

## Cross-Domain Routing

Cross-domain operations require special routing logic:

```rust
pub struct CrossDomainRouter {
    domain_registry: Arc<DomainRegistry>,
    adapter_registry: Arc<AdapterRegistry>,
}

impl<C: ExecutionContext> DomainRouter<C> for CrossDomainRouter {
    fn route_operation(&self, operation: &Operation<C>, route: &Route<C>) -> Result<OperationResult<C>, RoutingError> {
        if !route.is_cross_domain {
            return Err(RoutingError::NotCrossDomainOperation);
        }
        
        // 1. Split operation into domain-specific sub-operations
        let sub_operations = self.split_operation(operation, &route.involved_domains)?;
        
        // 2. Create routes for each sub-operation
        let sub_routes = self.create_sub_routes(&sub_operations)?;
        
        // 3. Execute sub-operations in the correct order
        let results = self.execute_sub_operations(sub_operations, sub_routes)?;
        
        // 4. Combine results
        let combined_result = self.combine_results(results)?;
        
        // 5. Verify cross-domain consistency
        self.verify_cross_domain_consistency(operation, &combined_result)?;
        
        Ok(combined_result)
    }
}
```

## Capability-Based Routing

Capabilities influence routing decisions by restricting which operations can be performed:

```rust
pub struct CapabilityRouter<C: ExecutionContext> {
    inner_router: Box<dyn DomainRouter<C>>,
    capability_verifier: Arc<CapabilityVerifier>,
}

impl<C: ExecutionContext> DomainRouter<C> for CapabilityRouter<C> {
    fn route_operation(&self, operation: &Operation<C>, route: &Route<C>) -> Result<OperationResult<C>, RoutingError> {
        // 1. Verify all capabilities
        for capability in &operation.capabilities {
            self.capability_verifier.verify_capability(capability, operation)
                .map_err(|e| RoutingError::CapabilityError(e))?;
        }
        
        // 2. If all capabilities are valid, proceed with routing
        self.inner_router.route_operation(operation, route)
    }
}
```

## Content-Addressed Operation Logging

All routing decisions and operation executions are logged to a content-addressed log:

```rust
pub struct OperationLog<C: ExecutionContext> {
    storage: Arc<dyn ContentAddressedStorage>,
}

impl<C: ExecutionContext> OperationLog<C> {
    pub fn log_operation(&self, operation: &Operation<C>, route: &Route<C>, result: &OperationResult<C>) -> Result<(), LogError> {
        // 1. Create log entry
        let entry = OperationLogEntry {
            operation_id: operation.id.clone(),
            route: route.clone(),
            result: result.clone(),
            timestamp: Timestamp::now(),
        };
        
        // 2. Generate content hash
        let hash = entry.content_hash();
        
        // 3. Store in content-addressed storage
        self.storage.put(hash, entry)?;
        
        Ok(())
    }
}
```

## Dynamic Routing Policies

Routing policies can be defined to customize routing behavior:

```rust
pub trait RoutingPolicy<C: ExecutionContext> {
    fn modify_route(&self, operation: &Operation<C>, route: &mut Route<C>) -> Result<(), RoutingError>;
}

// Example: Latency-optimized routing policy
pub struct LatencyOptimizedRoutingPolicy {
    domain_latency_stats: HashMap<ContentRef<DomainId>, Duration>,
}

impl<C: ExecutionContext> RoutingPolicy<C> for LatencyOptimizedRoutingPolicy {
    fn modify_route(&self, operation: &Operation<C>, route: &mut Route<C>) -> Result<(), RoutingError> {
        if !route.is_cross_domain {
            return Ok(());
        }
        
        // For cross-domain operations, select the primary domain with lowest latency
        let mut lowest_latency_domain = route.primary_domain.clone();
        let mut lowest_latency = self.domain_latency_stats.get(&route.primary_domain)
            .cloned()
            .unwrap_or(Duration::MAX);
            
        for domain in &route.involved_domains {
            if let Some(latency) = self.domain_latency_stats.get(domain) {
                if *latency < lowest_latency {
                    lowest_latency = *latency;
                    lowest_latency_domain = domain.clone();
                }
            }
        }
        
        route.primary_domain = lowest_latency_domain;
        
        Ok(())
    }
}
```

## Routing Security

Security mechanisms in the routing system:

```rust
pub struct SecureRouter<C: ExecutionContext> {
    inner_router: Box<dyn DomainRouter<C>>,
    authorization_service: Arc<AuthorizationService>,
}

impl<C: ExecutionContext> DomainRouter<C> for SecureRouter<C> {
    fn route_operation(&self, operation: &Operation<C>, route: &Route<C>) -> Result<OperationResult<C>, RoutingError> {
        // 1. Verify operation authenticity
        self.verify_operation_authenticity(operation)?;
        
        // 2. Check authorization
        self.authorization_service.authorize_operation(operation, route)?;
        
        // 3. If authorized, proceed with routing
        self.inner_router.route_operation(operation, route)
    }
    
    fn verify_operation_authenticity(&self, operation: &Operation<C>) -> Result<(), RoutingError> {
        // Verify that the operation's content hash matches its claimed ID
        let calculated_hash = operation.content_hash();
        if calculated_hash != operation.id.content_hash() {
            return Err(RoutingError::ContentHashMismatch);
        }
        
        Ok(())
    }
}
```

## Temporal Aspects of Routing

Temporal constraints affect routing decisions:

```rust
pub struct TemporalRouter<C: ExecutionContext> {
    inner_router: Box<dyn DomainRouter<C>>,
    temporal_validator: Arc<TemporalValidator>,
}

impl<C: ExecutionContext> DomainRouter<C> for TemporalRouter<C> {
    fn route_operation(&self, operation: &Operation<C>, route: &Route<C>) -> Result<OperationResult<C>, RoutingError> {
        // 1. Validate temporal constraints
        self.temporal_validator.validate_temporal_constraints(operation)?;
        
        // 2. If temporally valid, proceed with routing
        self.inner_router.route_operation(operation, route)
    }
}
```

## Best Practices for Operation Routing

1. **Define Clear Domain Boundaries**: Explicitly specify domains for all resources and operations.

2. **Use ContentRef for Routing Components**: Ensure all routing-related structures use content addressing.

3. **Implement Domain-Specific Error Handling**: Each domain should have appropriate error handling.

4. **Log All Routing Decisions**: Maintain a complete log of routing decisions for auditing.

5. **Verify Capabilities Early**: Check capabilities before routing to fail fast.

6. **Optimize Cross-Domain Routing**: Carefully design cross-domain operations to minimize latency.

7. **Use Routing Policies**: Implement custom routing policies for specialized needs.

8. **Maintain Domain Registry**: Keep the domain registry up to date with all active domains.

## Conclusion

The operation routing system in the unified Causality architecture provides a flexible, secure framework for directing operations to the appropriate execution context, domain, and handler. By leveraging content addressing and capability-based authorization, the routing system ensures operations are executed correctly while maintaining verifiability and security across domain boundaries.