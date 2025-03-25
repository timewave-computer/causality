<!-- Cross-domain capability management -->
<!-- Original file: docs/src/crossdomain_capability_management.md -->

# Cross-Domain Capability Management

## Overview

The Cross-Domain Capability Management system in Causality enables secure and verifiable sharing of capabilities across domain boundaries. This system ensures that authorization constraints are properly maintained when resources and operations span multiple domains, providing a unified security model with appropriate permissions projection.

```
┌───────────────────────────────────────────────────────────────────┐
│           Cross-Domain Capability Management System                │
├───────────────────┬───────────────────┬───────────────────────────┤
│  Source Domain    │   Projection      │    Target Domain          │
│                   │   System          │                           │
│  ┌─────────────┐  │  ┌─────────────┐  │  ┌─────────────┐          │
│  │ Capability  │  │  │ Capability  │  │  │ Capability  │          │
│  │ Manager     ├──┼─►│ Projection  ├──┼─►│ Manager     │          │
│  └─────────────┘  │  └─────────────┘  │  └─────────────┘          │
│        │          │        │          │        │                  │
│        ▼          │        ▼          │        ▼                  │
│  ┌─────────────┐  │  ┌─────────────┐  │  ┌─────────────┐          │
│  │ Resource    │  │  │ Translation │  │  │ Resource    │          │
│  │ Capabilities├──┼─►│ Layer       ├──┼─►│ Capabilities│          │
│  └─────────────┘  │  └─────────────┘  │  └─────────────┘          │
│        │          │        │          │        │                  │
│        ▼          │        ▼          │        ▼                  │
│  ┌─────────────┐  │  ┌─────────────┐  │  ┌─────────────┐          │
│  │ Security    │  │  │ Verification│  │  │ Security    │          │
│  │ Policies    ├──┼─►│ System      ├──┼─►│ Policies    │          │
│  └─────────────┘  │  └─────────────┘  │  └─────────────┘          │
└───────────────────┴───────────────────┴───────────────────────────┘
```

## Core Concepts

### Cross-Domain Capabilities

Cross-Domain Capabilities extend the basic capability model to work across domain boundaries:

```rust
pub struct CrossDomainCapability {
    id: CapabilityId,
    source_domain: DomainId,
    target_domain: DomainId,
    source_resource_id: ResourceId,
    target_resource_id: ResourceId,
    permissions: Vec<Permission>,
    constraints: Vec<Constraint>,
    projection_proof: Option<ProjectionProof>,
    expiration: Option<Timestamp>,
    delegation_path: Option<DelegationPath>,
}

pub struct ProjectionProof {
    proof_type: ProofType,
    proof_data: Vec<u8>,
    verification_key: VerificationKey,
    metadata: HashMap<String, String>,
}
```

### Capability Projection

Capability Projection is the process of transforming a capability from one domain to another:

```rust
pub trait CapabilityProjector {
    /// Project a capability from source domain to target domain
    fn project_capability(
        &self,
        capability: &Capability,
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &ProjectionContext
    ) -> Result<CrossDomainCapability, CapabilityError>;
    
    /// Verify a projected capability
    fn verify_projection(
        &self,
        cross_domain_capability: &CrossDomainCapability,
        context: &ProjectionContext
    ) -> Result<bool, CapabilityError>;
}
```

### Permission Mapping

Permission Mapping defines how permissions are translated between domains:

```rust
pub struct PermissionMap {
    source_domain: DomainId,
    target_domain: DomainId,
    mappings: HashMap<Permission, Vec<Permission>>,
    default_policy: DefaultMappingPolicy,
}

pub enum DefaultMappingPolicy {
    Deny,
    AllowExplicit,
    AllowWithMinimalScope,
}
```

## System Components

### Cross-Domain Capability Manager

The Cross-Domain Capability Manager orchestrates capability projection across domains:

```rust
pub struct CrossDomainCapabilityManager {
    domain_registry: DomainRegistry,
    projectors: HashMap<(DomainId, DomainId), Box<dyn CapabilityProjector>>,
    verifiers: HashMap<DomainId, Box<dyn CapabilityVerifier>>,
    permission_maps: HashMap<(DomainId, DomainId), PermissionMap>,
}

impl CrossDomainCapabilityManager {
    /// Create a new cross-domain capability
    pub async fn create_cross_domain_capability(
        &self,
        source_capability: &Capability,
        source_domain: &DomainId,
        target_domain: &DomainId,
        target_resource_id: &ResourceId,
        context: &ProjectionContext
    ) -> Result<CrossDomainCapability, CapabilityError> {
        // Verify source capability is valid
        let source_verifier = self.verifiers.get(source_domain)
            .ok_or(CapabilityError::VerifierNotFound)?;
        
        if !source_verifier.verify_capability(source_capability)? {
            return Err(CapabilityError::InvalidSourceCapability);
        }
        
        // Get appropriate projector
        let key = (source_domain.clone(), target_domain.clone());
        let projector = self.projectors.get(&key)
            .ok_or(CapabilityError::ProjectorNotFound)?;
        
        // Project the capability
        let cross_domain_cap = projector.project_capability(
            source_capability,
            source_domain,
            target_domain,
            context
        )?;
        
        // Verify the projected capability in target domain
        let target_verifier = self.verifiers.get(target_domain)
            .ok_or(CapabilityError::VerifierNotFound)?;
        
        if !target_verifier.verify_capability(&cross_domain_cap.into())? {
            return Err(CapabilityError::InvalidProjection);
        }
        
        Ok(cross_domain_cap)
    }
    
    /// Verify a cross-domain capability
    pub async fn verify_cross_domain_capability(
        &self,
        capability: &CrossDomainCapability,
        context: &ProjectionContext
    ) -> Result<bool, CapabilityError> {
        // Get the projector
        let key = (capability.source_domain.clone(), capability.target_domain.clone());
        let projector = self.projectors.get(&key)
            .ok_or(CapabilityError::ProjectorNotFound)?;
        
        // Verify the projection
        projector.verify_projection(capability, context)
    }
    
    /// Register a capability projector
    pub fn register_projector(
        &mut self,
        source_domain: DomainId,
        target_domain: DomainId,
        projector: Box<dyn CapabilityProjector>
    ) {
        let key = (source_domain, target_domain);
        self.projectors.insert(key, projector);
    }
}
```

### Projection Context

The Projection Context provides information needed for capability projection:

```rust
pub struct ProjectionContext {
    resource_resolver: Box<dyn ResourceResolver>,
    schema_registry: Box<dyn SchemaRegistry>,
    permission_registry: Box<dyn PermissionRegistry>,
    projection_parameters: HashMap<String, Value>,
}

impl ProjectionContext {
    /// Resolve a resource across domains
    pub fn resolve_resource(
        &self,
        source_resource_id: &ResourceId,
        target_domain: &DomainId
    ) -> Result<Option<ResourceId>, CapabilityError> {
        self.resource_resolver.resolve_cross_domain(source_resource_id, target_domain)
    }
    
    /// Map a permission across domains
    pub fn map_permission(
        &self,
        permission: &Permission,
        source_domain: &DomainId,
        target_domain: &DomainId
    ) -> Result<Vec<Permission>, CapabilityError> {
        self.permission_registry.map_permission(permission, source_domain, target_domain)
    }
}
```

### Resource Resolver

The Resource Resolver maps resources between domains:

```rust
pub trait ResourceResolver: Send + Sync {
    /// Resolve a resource ID from one domain to another
    fn resolve_cross_domain(
        &self,
        source_resource_id: &ResourceId,
        target_domain: &DomainId
    ) -> Result<Option<ResourceId>, CapabilityError>;
    
    /// Check if two resources are equivalent across domains
    fn check_resource_equivalence(
        &self,
        resource_id_1: &ResourceId,
        domain_1: &DomainId,
        resource_id_2: &ResourceId,
        domain_2: &DomainId
    ) -> Result<bool, CapabilityError>;
}
```

### Constraint Translator

The Constraint Translator converts constraints between domains:

```rust
pub trait ConstraintTranslator: Send + Sync {
    /// Translate constraints from source domain to target domain
    fn translate_constraints(
        &self,
        constraints: &[Constraint],
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &ProjectionContext
    ) -> Result<Vec<Constraint>, CapabilityError>;
    
    /// Check if translated constraints maintain security properties
    fn verify_constraint_security(
        &self,
        source_constraints: &[Constraint],
        target_constraints: &[Constraint],
        context: &ProjectionContext
    ) -> Result<bool, CapabilityError>;
}
```

## Projection Strategies

### Direct Projection

Direct projection maintains the same permissions and constraints:

```rust
pub struct DirectProjector {
    resource_resolver: Box<dyn ResourceResolver>,
}

impl CapabilityProjector for DirectProjector {
    fn project_capability(
        &self,
        capability: &Capability,
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &ProjectionContext
    ) -> Result<CrossDomainCapability, CapabilityError> {
        // Resolve the target resource
        let target_resource_id = self.resource_resolver.resolve_cross_domain(
            &capability.resource_id(),
            target_domain
        )?.ok_or(CapabilityError::ResourceNotFound)?;
        
        // Create cross-domain capability with same permissions
        let cross_domain_cap = CrossDomainCapability {
            id: CapabilityId::new(),
            source_domain: source_domain.clone(),
            target_domain: target_domain.clone(),
            source_resource_id: capability.resource_id().clone(),
            target_resource_id,
            permissions: capability.permissions().clone(),
            constraints: capability.constraints().clone(),
            projection_proof: None, // Will be filled in later
            expiration: capability.expiration(),
            delegation_path: capability.delegation_path().cloned(),
        };
        
        Ok(cross_domain_cap)
    }
}
```

### Restricted Projection

Restricted projection narrows permissions based on domain security policies:

```rust
pub struct RestrictedProjector {
    resource_resolver: Box<dyn ResourceResolver>,
    permission_mapper: Box<dyn PermissionMapper>,
    constraint_translator: Box<dyn ConstraintTranslator>,
}

impl CapabilityProjector for RestrictedProjector {
    fn project_capability(
        &self,
        capability: &Capability,
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &ProjectionContext
    ) -> Result<CrossDomainCapability, CapabilityError> {
        // Resolve the target resource
        let target_resource_id = self.resource_resolver.resolve_cross_domain(
            &capability.resource_id(),
            target_domain
        )?.ok_or(CapabilityError::ResourceNotFound)?;
        
        // Map permissions to target domain with potential restrictions
        let mut mapped_permissions = Vec::new();
        for permission in capability.permissions() {
            let domain_permissions = self.permission_mapper.map_permission(
                permission,
                source_domain,
                target_domain,
                context
            )?;
            
            mapped_permissions.extend(domain_permissions);
        }
        
        // Translate and potentially strengthen constraints
        let translated_constraints = self.constraint_translator.translate_constraints(
            capability.constraints(),
            source_domain,
            target_domain,
            context
        )?;
        
        // Create cross-domain capability with mapped permissions and translated constraints
        let cross_domain_cap = CrossDomainCapability {
            id: CapabilityId::new(),
            source_domain: source_domain.clone(),
            target_domain: target_domain.clone(),
            source_resource_id: capability.resource_id().clone(),
            target_resource_id,
            permissions: mapped_permissions,
            constraints: translated_constraints,
            projection_proof: None, // Will be filled in later
            expiration: capability.expiration(),
            delegation_path: capability.delegation_path().cloned(),
        };
        
        Ok(cross_domain_cap)
    }
}
```

### Zero-Knowledge Projection

Zero-Knowledge Projection uses ZK proofs to verify capability properties without revealing details:

```rust
pub struct ZkProjector {
    resource_resolver: Box<dyn ResourceResolver>,
    zk_prover: Box<dyn ZkProver>,
}

impl CapabilityProjector for ZkProjector {
    fn project_capability(
        &self,
        capability: &Capability,
        source_domain: &DomainId,
        target_domain: &DomainId,
        context: &ProjectionContext
    ) -> Result<CrossDomainCapability, CapabilityError> {
        // Resolve the target resource
        let target_resource_id = self.resource_resolver.resolve_cross_domain(
            &capability.resource_id(),
            target_domain
        )?.ok_or(CapabilityError::ResourceNotFound)?;
        
        // Create a ZK proof of capability validity
        let circuit = self.zk_prover.get_projection_circuit(
            source_domain,
            target_domain
        )?;
        
        let inputs = HashMap::from([
            ("capability".to_string(), serialize_capability(capability)?),
            ("source_domain".to_string(), serialize_domain_id(source_domain)?),
            ("target_domain".to_string(), serialize_domain_id(target_domain)?),
        ]);
        
        let witness = self.zk_prover.generate_witness(&circuit, &inputs)?;
        let proof = self.zk_prover.generate_proof(&circuit, &witness)?;
        
        // Create projection proof
        let projection_proof = ProjectionProof {
            proof_type: proof.proof_type(),
            proof_data: proof.data().to_vec(),
            verification_key: self.zk_prover.get_verification_key(&circuit)?,
            metadata: HashMap::new(),
        };
        
        // Create cross-domain capability with proof
        let cross_domain_cap = CrossDomainCapability {
            id: CapabilityId::new(),
            source_domain: source_domain.clone(),
            target_domain: target_domain.clone(),
            source_resource_id: capability.resource_id().clone(),
            target_resource_id,
            permissions: capability.permissions().clone(),
            constraints: capability.constraints().clone(),
            projection_proof: Some(projection_proof),
            expiration: capability.expiration(),
            delegation_path: capability.delegation_path().cloned(),
        };
        
        Ok(cross_domain_cap)
    }
}
```

## Cross-Domain Authorization

### Authorization Flow

```rust
pub async fn authorize_cross_domain_operation(
    operation: &CrossDomainOperation,
    auth_context: &AuthorizationContext,
    capability_manager: &CrossDomainCapabilityManager
) -> Result<bool, CapabilityError> {
    // Get the source and target domains
    let source_domain = operation.source_domain();
    let target_domains = operation.target_domains();
    
    // Verify authorization in source domain
    let source_result = verify_domain_authorization(
        operation.source_operations(),
        auth_context,
        &source_domain,
        capability_manager
    ).await?;
    
    if !source_result {
        return Ok(false);
    }
    
    // Verify authorization in each target domain
    for target_domain in target_domains {
        let projection_context = create_projection_context(
            operation,
            &source_domain,
            target_domain
        )?;
        
        // Project capabilities to target domain
        let projected_capabilities = project_capabilities(
            auth_context.capabilities(),
            &source_domain,
            target_domain,
            &projection_context,
            capability_manager
        ).await?;
        
        // Create target auth context with projected capabilities
        let target_auth_context = AuthorizationContext::new(
            auth_context.principal(),
            projected_capabilities
        );
        
        // Verify authorization in target domain
        let target_result = verify_domain_authorization(
            operation.target_operations(target_domain)?,
            &target_auth_context,
            target_domain,
            capability_manager
        ).await?;
        
        if !target_result {
            return Ok(false);
        }
    }
    
    Ok(true)
}

async fn project_capabilities(
    capabilities: &[Capability],
    source_domain: &DomainId,
    target_domain: &DomainId,
    context: &ProjectionContext,
    capability_manager: &CrossDomainCapabilityManager
) -> Result<Vec<Capability>, CapabilityError> {
    let mut projected = Vec::new();
    
    for capability in capabilities {
        match capability_manager.create_cross_domain_capability(
            capability,
            source_domain,
            target_domain,
            &capability.resource_id(), // This would be resolved in the manager
            context
        ).await {
            Ok(cross_domain_cap) => projected.push(cross_domain_cap.into()),
            Err(e) => log::debug!("Failed to project capability: {}", e),
        }
    }
    
    Ok(projected)
}
```

## Integration with Validation Pipeline

The Cross-Domain Capability management integrates with the validation pipeline:

```rust
pub struct CrossDomainCapabilityValidator {
    capability_manager: CrossDomainCapabilityManager,
}

impl ValidationStage for CrossDomainCapabilityValidator {
    fn validate(
        &self,
        item: &dyn Validatable,
        context: &ValidationContext
    ) -> ValidationResult {
        if let Some(cross_domain_op) = item.as_cross_domain_operation() {
            // Extract auth context
            let auth_context = cross_domain_op.auth_context();
            
            // Create projection context from validation context
            let projection_context = ProjectionContext::from_validation_context(context);
            
            // Perform cross-domain capability validation
            match self.authorize_operation(cross_domain_op, auth_context, &projection_context) {
                Ok(true) => ValidationResult::new_valid("cross_domain_capability"),
                Ok(false) => ValidationResult::new_error(
                    "cross_domain_capability",
                    ValidationErrorCode::Unauthorized,
                    "Operation not authorized across domains"
                ),
                Err(e) => ValidationResult::new_error(
                    "cross_domain_capability",
                    ValidationErrorCode::ValidationError,
                    format!("Capability validation error: {}", e)
                ),
            }
        } else {
            // Not a cross-domain operation, skip this validation
            ValidationResult::new_valid("cross_domain_capability")
        }
    }
}
```

## Usage Examples

### Example 1: Basic Cross-Domain Capability Projection

```rust
// Create a capability in source domain
let capability = Capability::new(
    resource_id,
    vec![Permission::Read, Permission::Write],
    vec![TimeConstraint::new(expiration)]
);

// Create a cross-domain capability for target domain
let cross_domain_capability = cross_domain_manager
    .create_cross_domain_capability(
        &capability,
        &source_domain,
        &target_domain,
        &target_resource_id,
        &projection_context
    )
    .await?;

// Use the cross-domain capability in the target domain
let result = target_domain_service
    .execute_operation(
        operation,
        AuthorizationContext::new(principal, vec![cross_domain_capability.into()])
    )
    .await;
```

### Example 2: Zero-Knowledge Capability Projection

```rust
// Register a ZK projector for EVM to Cosmos domains
cross_domain_manager.register_projector(
    DomainId::new("evm"),
    DomainId::new("cosmos"),
    Box::new(ZkProjector::new(
        resource_resolver,
        zk_prover
    ))
);

// Create a cross-domain capability with ZK proof
let cross_domain_capability = cross_domain_manager
    .create_cross_domain_capability(
        &evm_capability,
        &DomainId::new("evm"),
        &DomainId::new("cosmos"),
        &cosmos_resource_id,
        &projection_context
    )
    .await?;

// The capability includes a ZK proof of valid projection
assert!(cross_domain_capability.projection_proof.is_some());

// Use for authorization in target domain
let auth_result = cosmos_domain_service
    .verify_authorization(
        operation,
        &AuthorizationContext::new(principal, vec![cross_domain_capability.into()])
    )
    .await?;
```

### Example 3: Permission Mapping Configuration

```rust
// Define permission mappings between EVM and Solana domains
let mut permission_map = PermissionMap::new(
    DomainId::new("evm"),
    DomainId::new("solana"),
    DefaultMappingPolicy::Deny
);

// Add specific mappings
permission_map.add_mapping(
    Permission::Admin,
    vec![
        Permission::Write,
        Permission::Transfer,
        Permission::Update
    ]
);

permission_map.add_mapping(
    Permission::Write,
    vec![Permission::Write]
);

permission_map.add_mapping(
    Permission::Read,
    vec![Permission::Read]
);

// Register the permission map
cross_domain_manager.register_permission_map(permission_map);
```

## Best Practices

### Security Considerations

1. **Least Privilege Projection**: Always project capabilities with the minimal set of permissions needed
2. **Constraint Preservation**: Ensure constraints are maintained or strengthened during projection
3. **Expiration Management**: Use shorter expiration times for cross-domain capabilities
4. **Verification**: Always verify capabilities in both source and target domains
5. **Revocation**: Implement capability revocation mechanisms that work across domains

### Performance Optimization

1. **Caching**: Cache projection results for frequently used capabilities
2. **Batch Processing**: Process capability projections in batches
3. **Proof Reuse**: Reuse ZK proofs when applicable
4. **Domain Specialization**: Specialize projectors for specific domain pairs

### Architectural Patterns

1. **Capability Federation**: Establish trust relationships between domains
2. **Hierarchical Projection**: Organize domains into hierarchies for simpler projection paths
3. **Capability Chaining**: Support chaining capabilities across multiple domains
4. **Domain Adapters**: Use adapters to handle domain-specific capability formats

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Cross-Domain Capability Manager | In Progress | Core functionality working |
| Direct Projector | Complete | Basic projection implemented |
| Restricted Projector | In Progress | Permission mapping working |
| ZK Projector | Planned | Design completed |
| Resource Resolver | Complete | Basic resolution implemented |
| Constraint Translator | In Progress | Simple constraints working |
| Cross-Domain Authorization | In Progress | Basic validation working |
| Integration with Validation Pipeline | Planned | Framework in place |

## Future Enhancements

1. **Recursive Capability Projection**: Support for projecting capabilities across multiple domains
2. **Delegable Cross-Domain Capabilities**: Enable delegation of capabilities across domains
3. **Revocation Mechanisms**: Implement cross-domain capability revocation
4. **Dynamic Permission Mapping**: Support for dynamic permission mapping based on context
5. **Fine-Grained Constraints**: More expressive constraint language for cross-domain scenarios
6. **Federated Trust Model**: Framework for establishing trust between domains
7. **Capability Synthesis**: Generate domain-specific capabilities from capability requirements

## References

- [Architecture Overview](architecture.md)
- [Capability Model](capability_model.md)
- [Cross-Domain Operations](cross_domain_operations.md)
- [Proof Generation Framework](proof_generation.md)
- [Zero-Knowledge Workflows](zero_knowledge_workflows.md)
- [Security Architecture](security.md) 