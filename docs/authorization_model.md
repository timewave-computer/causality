# Authorization Model

This document describes the comprehensive authorization model within the Causality architecture, focusing on capability-based security, access control mechanisms, and authorization patterns.

## Core Concepts

### Authorization Principles

The Causality authorization model is built on several key principles:

1. **Principle of Least Privilege**: Entities are granted the minimum privileges necessary to perform their functions
2. **Capability-Based Security**: Authorization is based on unforgeable capabilities rather than access control lists
3. **Separation of Privilege**: Critical operations require multiple capabilities
4. **Delegation**: Capabilities can be delegated with restrictions
5. **Contextual Authorization**: Authorization decisions consider execution context
6. **Composable Security**: Security policies can be composed for complex scenarios

### Authorization Flow

```
┌────────────┐      ┌───────────────┐      ┌──────────────┐      ┌────────────────┐
│            │      │               │      │              │      │                │
│  Request   │─────▶│  Capability   │─────▶│  Authority   │─────▶│  Operation     │
│  Initiator │      │  Verification │      │  Validation  │      │  Execution     │
│            │      │               │      │              │      │                │
└────────────┘      └───────────────┘      └──────────────┘      └────────────────┘
                            ▲                      ▲
                            │                      │
                    ┌───────────────┐      ┌──────────────┐
                    │               │      │              │
                    │  Capability   │      │  Policy      │
                    │  Store        │      │  Engine      │
                    │               │      │              │
                    └───────────────┘      └──────────────┘
```

## Capability Model

The foundation of the authorization system is a capability-based security model.

### Capability Structure

```rust
/// A capability that grants specific rights to resources
pub struct Capability {
    /// Unique identifier for the capability
    id: CapabilityId,
    
    /// The right granted by this capability
    right: Right,
    
    /// The resource this capability applies to
    resource: ResourceId,
    
    /// The holder of this capability
    holder: ActorIdBox,
    
    /// Who delegated this capability (if delegated)
    delegated_by: Option<ActorIdBox>,
    
    /// Constraints on the capability
    constraints: Vec<CapabilityConstraint>,
    
    /// When the capability expires (None = never)
    expiration: Option<Timestamp>,
    
    /// Whether the capability can be further delegated
    delegatable: bool,
    
    /// Maximum delegation depth (-1 = unlimited)
    max_delegation_depth: i32,
    
    /// Revocation information
    revocation: Option<RevocationInfo>,
}

/// A right that can be granted to a resource
pub enum Right {
    /// Right to read a resource
    Read,
    
    /// Right to update a resource
    Update,
    
    /// Right to transfer a resource
    Transfer,
    
    /// Right to mint tokens of a resource
    Mint,
    
    /// Right to burn tokens of a resource
    Burn,
    
    /// Right to freeze a resource
    Freeze,
    
    /// Right to unfreeze a resource
    Unfreeze,
    
    /// Right to withdraw from a resource
    Withdraw,
    
    /// Right to deposit to a resource
    Deposit,
    
    /// Right to create child resources
    CreateChild,
    
    /// Right to destroy a resource
    Destroy,
    
    /// Right to execute a program
    Execute,
    
    /// Right to administer a capability
    Admin,
    
    /// Custom right with specified name
    Custom(String),
}
```

### Capability Constraints

Constraints restrict when and how capabilities can be used:

```rust
/// A constraint on a capability
pub enum CapabilityConstraint {
    /// Time-based constraint
    TimeWindow {
        /// Earliest time the capability can be used
        start: Option<Timestamp>,
        /// Latest time the capability can be used
        end: Option<Timestamp>,
    },
    
    /// Amount-based constraint (for transfer, mint, etc.)
    Amount {
        /// Maximum amount allowed
        max_amount: u64,
        /// Minimum amount allowed
        min_amount: Option<u64>,
    },
    
    /// Rate-limiting constraint
    RateLimit {
        /// Maximum number of uses
        max_uses: u64,
        /// Time window for rate limiting
        window: Duration,
    },
    
    /// Domain constraint
    Domain {
        /// Allowed domains
        allowed_domains: Vec<DomainId>,
    },
    
    /// Relationship constraint
    Relationship {
        /// Required relationship type
        relationship_type: String,
        /// Required relationship with resource
        related_resource: ResourceId,
    },
    
    /// Custom constraint with specified verification logic
    Custom {
        /// Name of the constraint
        name: String,
        /// Serialized constraint data
        data: Value,
    },
}
```

## Authorization Components

### Capability Store

The capability store manages all capabilities in the system:

```rust
/// Stores and retrieves capabilities
pub struct CapabilityStore {
    /// Storage backend
    storage: Arc<dyn Storage>,
    
    /// Capability index
    index: CapabilityIndex,
    
    /// Verification provider
    verifier: Arc<dyn VerificationProvider>,
}

impl CapabilityStore {
    /// Store a capability
    pub fn store_capability(&self, capability: Capability) -> Result<()>;
    
    /// Retrieve a capability by ID
    pub fn get_capability(&self, id: &CapabilityId) -> Result<Option<Capability>>;
    
    /// Find capabilities for a given holder and resource
    pub fn find_capabilities(
        &self,
        holder: &ActorIdBox,
        resource: &ResourceId,
        right: Option<&Right>,
    ) -> Result<Vec<Capability>>;
    
    /// Revoke a capability
    pub fn revoke_capability(&self, id: &CapabilityId, revoker: &ActorIdBox) -> Result<()>;
    
    /// Check if a capability has been revoked
    pub fn is_revoked(&self, id: &CapabilityId) -> Result<bool>;
}
```

### Authorization Service

The authorization service verifies authorization for operations:

```rust
/// Service for authorization decisions
pub struct AuthorizationService {
    /// Capability store
    capability_store: Arc<CapabilityStore>,
    
    /// Policy engine
    policy_engine: Arc<PolicyEngine>,
    
    /// Validation context provider
    context_provider: Arc<dyn ContextProvider>,
}

impl AuthorizationService {
    /// Check if an actor is authorized to perform an operation
    pub fn is_authorized(
        &self,
        actor: &ActorIdBox,
        operation: &Operation,
        context: &ExecutionContext,
    ) -> Result<AuthorizationResult>;
    
    /// Get the capabilities needed for an operation
    pub fn required_capabilities(
        &self,
        operation: &Operation,
    ) -> Result<Vec<RequiredCapability>>;
    
    /// Create an authorization proof
    pub fn create_authorization_proof(
        &self,
        actor: &ActorIdBox,
        operation: &Operation,
        capabilities: &[CapabilityId],
    ) -> Result<AuthorizationProof>;
    
    /// Verify an authorization proof
    pub fn verify_authorization_proof(
        &self,
        proof: &AuthorizationProof,
        operation: &Operation,
    ) -> Result<bool>;
}
```

### Policy Engine

The policy engine enforces authorization policies:

```rust
/// Engine for evaluating authorization policies
pub struct PolicyEngine {
    /// Policy rules
    rules: Vec<Box<dyn PolicyRule>>,
    
    /// Policy evaluator
    evaluator: Arc<PolicyEvaluator>,
}

impl PolicyEngine {
    /// Evaluate policies for an authorization request
    pub fn evaluate(
        &self,
        actor: &ActorIdBox,
        operation: &Operation,
        capabilities: &[Capability],
        context: &ExecutionContext,
    ) -> Result<PolicyEvaluationResult>;
    
    /// Add a policy rule
    pub fn add_rule(&mut self, rule: Box<dyn PolicyRule>);
    
    /// Check for policy conflicts
    pub fn check_conflicts(&self) -> Result<Vec<PolicyConflict>>;
}
```

## Authorization Mechanisms

### Direct Capability Authorization

The simplest form of authorization is direct capability verification:

```rust
// Get capabilities for the user
let capabilities = capability_store.find_capabilities(
    &user_id,
    &resource_id,
    Some(&Right::Transfer)
)?;

// Create an authorization object
let authorization = Authorization::with_capabilities(
    user_id.clone(),
    capabilities
);

// Create an operation with the authorization
let operation = Operation::new(OperationType::TransferResource)
    .with_abstract_representation(Box::new(transfer_effect))
    .with_authorization(authorization);

// Verify the authorization
let auth_result = authorization_service.is_authorized(
    &user_id,
    &operation,
    &context
)?;

if auth_result.authorized {
    // Proceed with operation
} else {
    // Handle unauthorized access
    println!("Unauthorized: {}", auth_result.reason.unwrap_or_default());
}
```

### Delegated Authorization

Capabilities can be delegated from one actor to another:

```rust
// Original capability
let original_capability = capability_store.get_capability(&capability_id)?
    .ok_or(Error::capability_not_found(capability_id))?;

// Check if the capability is delegatable
if !original_capability.delegatable {
    return Err(Error::capability_not_delegatable(capability_id));
}

// Create a delegated capability
let delegated_capability = Capability::new(
    Right::Transfer,
    resource_id.clone(),
    recipient_id.clone()
)
.with_delegated_by(original_capability.holder().clone())
.with_constraints(vec![
    CapabilityConstraint::Amount {
        max_amount: 1000,
        min_amount: Some(1),
    },
    CapabilityConstraint::TimeWindow {
        start: None,
        end: Some(expiration_time),
    },
])
.with_delegatable(false);

// Store the delegated capability
capability_store.store_capability(delegated_capability)?;
```

### Compound Authorization

Multiple capabilities can be combined for compound authorization:

```rust
// Get multiple capabilities
let read_capability = capability_store.find_capabilities(
    &user_id,
    &resource_id,
    Some(&Right::Read)
)?;

let update_capability = capability_store.find_capabilities(
    &user_id,
    &resource_id,
    Some(&Right::Update)
)?;

// Create a compound authorization
let authorization = Authorization::with_capabilities(
    user_id.clone(),
    [read_capability, update_capability].concat()
);

// Create an operation with the authorization
let operation = Operation::new(OperationType::UpdateResource)
    .with_abstract_representation(Box::new(update_effect))
    .with_authorization(authorization);
```

### Capability Chains

Authorization can follow capability delegation chains:

```rust
// Verify a capability chain
let chain_result = authorization_service.verify_capability_chain(
    &capability_id,
    &user_id,
    &resource_id,
    &Right::Transfer
)?;

if chain_result.valid {
    // The capability chain is valid
    println!("Valid delegation chain: {} levels", chain_result.chain_length);
    
    // Use the capability
    let authorization = Authorization::with_capabilities(
        user_id.clone(),
        vec![capability_id]
    );
} else {
    // The chain is invalid
    println!("Invalid chain: {}", chain_result.reason.unwrap_or_default());
}
```

### Authorization Context

Authorization decisions consider execution context:

```rust
// Create an execution context
let context = ExecutionContext::new(ExecutionPhase::Execution)
    .with_environment(ExecutionEnvironment::Production)
    .with_domain(domain_id.clone())
    .with_timestamp(current_time)
    .with_invoker(user_id.clone())
    .with_metadata("transaction_id", transaction_id.to_string());

// Verify authorization with context
let auth_result = authorization_service.is_authorized_with_context(
    &user_id,
    &operation,
    &context
)?;
```

## Integration with Resource System

### Resource Lifecycle Integration

Authorization is integrated with resource lifecycle states:

```rust
// Create a resource with capability grants
let resource = Resource::new(
    ResourceType::FungibleToken,
    properties
)
.with_capability_grants(vec![
    CapabilityGrant::new(creator_id.clone(), Right::Admin),
    CapabilityGrant::new(creator_id.clone(), Right::Mint),
    CapabilityGrant::new(creator_id.clone(), Right::Burn),
    CapabilityGrant::new(creator_id.clone(), Right::Transfer),
]);

// Lifecycle state transitions require proper capabilities
let freeze_operation = Operation::new(OperationType::FreezeResource)
    .with_input(resource.clone())
    .with_authorization(Authorization::with_capabilities(
        admin_id.clone(),
        vec![admin_capability]
    ));

// Execute with lifecycle validation
let result = resource_lifecycle_manager.transition_state(
    &resource_id,
    ResourceState::Frozen,
    &freeze_operation
)?;
```

### Relationship-Based Authorization

Resources can grant capabilities based on relationships:

```rust
// Create a relationship between resources
let relationship = ResourceRelationship::new(
    parent_resource_id.clone(),
    "OWNS",
    child_resource_id.clone()
)
.with_capabilities(vec![
    RelationshipCapability::new(Right::Transfer),
    RelationshipCapability::new(Right::Update),
]);

// Store the relationship
relationship_tracker.create_relationship(relationship)?;

// Get capabilities based on relationships
let relationship_capabilities = authorization_service.get_relationship_capabilities(
    &owner_id,
    &child_resource_id
)?;

// Use relationship-derived capabilities
let authorization = Authorization::with_capabilities(
    owner_id.clone(),
    relationship_capabilities
);
```

## Cross-Domain Authorization

### Cross-Domain Capability Projection

Capabilities can be projected across domains:

```rust
// Project a capability to another domain
let projected_capability = cross_domain_manager.project_capability(
    &capability_id,
    &source_domain.id(),
    &target_domain.id(),
    ProjectionType::Shadow
)?;

// Use the projected capability in the target domain
let authorization = Authorization::with_capabilities(
    user_id.clone(),
    vec![projected_capability.id()]
);
```

### Cross-Domain Capability Verification

Capabilities from other domains require special verification:

```rust
// Create a verifier for cross-domain capabilities
let cross_domain_verifier = CrossDomainCapabilityVerifier::new(
    domain_registry.clone(),
    capability_store.clone()
);

// Verify a cross-domain capability
let verification_result = cross_domain_verifier.verify_capability(
    &capability_id,
    &user_id,
    &resource_id,
    &Right::Transfer,
    &source_domain.id()
)?;

if verification_result.valid {
    // Use the verified capability
    let authorization = Authorization::with_cross_domain_proof(
        user_id.clone(),
        capability_id.clone(),
        verification_result.proof
    );
}
```

## Security Considerations

### Capability Revocation

Capabilities can be revoked when necessary:

```rust
// Revoke a capability
let revocation = capability_store.revoke_capability(
    &capability_id,
    &admin_id,
    RevocationReason::SecurityRisk
)?;

// Record the revocation
let revocation_fact = FactSnapshot::new(
    FactId::new(),
    "capability_revoked",
    json!({
        "capability_id": capability_id.to_string(),
        "revoked_by": admin_id.to_string(),
        "reason": "Security risk",
    }),
    vec![]
);

fact_store.store_fact(revocation_fact)?;
```

### Capability Expiration

Capabilities can expire automatically:

```rust
// Create a capability with expiration
let temporary_capability = Capability::new(
    Right::Execute,
    program_id.clone(),
    user_id.clone()
)
.with_expiration(current_time + Duration::from_secs(3600));

// Check for expiration during verification
let capability = capability_store.get_capability(&capability_id)?
    .ok_or(Error::capability_not_found(capability_id))?;

if capability.is_expired(current_time) {
    return Err(Error::capability_expired(capability_id));
}
```

### Audit Trail

Authorization decisions are tracked for audit purposes:

```rust
// Record an authorization decision
let audit_entry = AuthorizationAuditEntry::new(
    user_id.clone(),
    operation.id().clone(),
    auth_result.authorized,
    auth_result.reason.clone(),
    context.clone()
);

audit_service.record_authorization(audit_entry)?;

// Query the audit trail
let audit_entries = audit_service.query_authorization_history(
    &user_id,
    &resource_id,
    TimeRange::since(one_day_ago)
)?;
```

## Usage Patterns

### Resource Creation and Initial Capabilities

```rust
// Create a resource with initial capabilities
let resource = ResourceBuilder::new(ResourceType::FungibleToken)
    .with_properties(token_properties)
    .with_initial_capability(creator_id.clone(), Right::Admin)
    .with_initial_capability(creator_id.clone(), Right::Mint)
    .with_initial_capability(creator_id.clone(), Right::Burn)
    .build();

// Create the resource
let resource_id = resource_register.create_resource(resource)?;
```

### Multi-Signature Authorization

```rust
// Create a multi-signature policy
let multi_sig_policy = MultiSignaturePolicy::new(
    2,  // Threshold (minimum required signatures)
    vec![user1_id.clone(), user2_id.clone(), user3_id.clone()]
);

// Register the policy
policy_engine.register_multi_signature_policy(
    resource_id.clone(),
    Right::Transfer,
    multi_sig_policy
)?;

// Create a multi-signature authorization
let partial_auth = MultiSignatureAuthorization::new(
    operation.id().clone(),
    user1_id.clone()
);

// Sign the authorization
let signature = crypto_service.sign(
    &private_key,
    &partial_auth.to_bytes()?
)?;

let signed_auth = partial_auth.with_signature(user1_id.clone(), signature);

// Submit the partial authorization
multi_sig_service.submit_partial_authorization(signed_auth)?;
```

### Hierarchical Resource Authorization

```rust
// Create a resource hierarchy
let organization = ResourceBuilder::new(ResourceType::Organization)
    .with_properties(org_properties)
    .with_initial_capability(admin_id.clone(), Right::Admin)
    .build();

let org_id = resource_register.create_resource(organization)?;

let department = ResourceBuilder::new(ResourceType::Department)
    .with_properties(dept_properties)
    .with_parent(org_id.clone())
    .with_relationship_type("BELONGS_TO")
    .with_inherited_capability(Right::Admin)
    .build();

let dept_id = resource_register.create_resource(department)?;

// Admin of parent resource gets capabilities on child resources
let admin_capabilities = authorization_service.get_hierarchical_capabilities(
    &admin_id,
    &dept_id
)?;
```

## Implementation Status

The authorization model implementation status:

- ✅ Core capability model
- ✅ Authorization service
- ✅ Capability store
- ✅ Direct capability authorization
- ✅ Basic policy engine
- ✅ Resource lifecycle integration
- ✅ Capability revocation
- ✅ Audit trail
- ⚠️ Cross-domain capability projection (in progress)
- ⚠️ Multi-signature authorization (in progress)
- ⚠️ Advanced policy rules (in progress)
- ❌ Hierarchical authorization
- ❌ Capability analytics

## Future Enhancements

1. **Fine-grained Delegation Control**: More detailed control over capability delegation
2. **Compound Capability Verification**: Optimized verification for multiple capabilities
3. **Attribute-Based Access Control**: Integration with attribute-based policies
4. **Risk-Based Authorization**: Adaptive authorization based on risk factors
5. **Authorization Caching**: Performance optimizations for high-volume systems
6. **Zero-Knowledge Authorization Proofs**: Privacy-preserving capability verification
7. **Homomorphic Capability Encryption**: Secure capability handling without decryption 