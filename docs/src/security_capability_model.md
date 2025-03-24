# Security Capability Model in Causality

## Overview

This document describes the security capability model within the Causality architecture. The security capability model provides a principled approach to managing access control and authorization throughout the system. It is based on the concept of capabilities as unforgeable tokens of authority that grant specific rights to perform operations on resources. This model ensures fine-grained, flexible, and secure access control while supporting the distributed and temporal nature of the Causality system.

## Core Concepts

### Capability Model

The fundamental capability structures:

```rust
pub struct Capability {
    /// Unique identifier for this capability
    id: CapabilityId,
    
    /// The principal to whom this capability is granted
    principal: Principal,
    
    /// The right this capability grants
    right: CapabilityRight,
    
    /// The target of this capability (e.g., resource, operation)
    target: CapabilityTarget,
    
    /// Constraints on the capability
    constraints: Vec<CapabilityConstraint>,
    
    /// Capability metadata
    metadata: CapabilityMetadata,
    
    /// Cryptographic proof of the capability's validity
    proof: CapabilityProof,
    
    /// Capability status
    status: CapabilityStatus,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Expiration timestamp (if applicable)
    expires_at: Option<Timestamp>,
}

pub enum CapabilityRight {
    /// Read access
    Read,
    
    /// Write access
    Write,
    
    /// Execute access
    Execute,
    
    /// Administrative access
    Admin,
    
    /// Delegate capability
    Delegate,
    
    /// Custom right
    Custom(String),
}

pub enum CapabilityTarget {
    /// Target is a resource
    Resource(ResourceId),
    
    /// Target is a resource type
    ResourceType(ResourceType),
    
    /// Target is a domain
    Domain(DomainId),
    
    /// Target is an operation type
    OperationType(OperationType),
    
    /// Target is a registry
    Registry(RegistryId),
    
    /// Target is a capability (for delegation)
    Capability(CapabilityId),
    
    /// Custom target
    Custom(String, String),
}
```

## Capability Management

### Capability Registry

Managing capabilities across the system:

```rust
pub struct CapabilityRegistry {
    /// Storage for capabilities
    storage: Arc<dyn StorageProvider>,
    
    /// Capability validator
    validator: Arc<CapabilityValidator>,
    
    /// Capability observers
    observers: Vec<Box<dyn CapabilityObserver>>,
    
    /// Cryptographic provider for capability proofs
    crypto_provider: Arc<dyn CryptoProvider>,
}

impl CapabilityRegistry {
    /// Create a new capability
    pub fn create_capability(
        &self,
        principal: Principal,
        right: CapabilityRight,
        target: CapabilityTarget,
        constraints: Vec<CapabilityConstraint>,
        metadata: Option<CapabilityMetadata>,
        ttl: Option<Duration>,
    ) -> Result<Capability, CapabilityError> {
        // Generate a new capability ID
        let capability_id = CapabilityId::generate();
        
        // Create base capability
        let mut capability = Capability {
            id: capability_id,
            principal,
            right,
            target,
            constraints,
            metadata: metadata.unwrap_or_default(),
            proof: CapabilityProof::empty(),
            status: CapabilityStatus::Inactive,
            created_at: system.current_time(),
            expires_at: ttl.map(|d| system.current_time() + d),
        };
        
        // Validate capability
        self.validator.validate_capability(&capability)?;
        
        // Generate capability proof
        let proof = self.generate_capability_proof(&capability)?;
        capability.proof = proof;
        
        // Activate capability
        capability.status = CapabilityStatus::Active;
        
        // Store capability
        self.store_capability(&capability)?;
        
        // Notify observers
        for observer in &self.observers {
            observer.on_capability_created(&capability)?;
        }
        
        Ok(capability)
    }
    
    /// Verify a capability
    pub fn verify_capability(
        &self,
        capability: &Capability,
    ) -> Result<VerificationResult, CapabilityError> {
        // Check if capability is active
        if capability.status != CapabilityStatus::Active {
            return Ok(VerificationResult::invalid(
                format!("Capability is not active: {:?}", capability.status)
            ));
        }
        
        // Check expiration
        if let Some(expires_at) = capability.expires_at {
            if expires_at <= system.current_time() {
                return Ok(VerificationResult::invalid(
                    format!("Capability expired at {}", expires_at)
                ));
            }
        }
        
        // Verify proof
        let proof_result = self.verify_capability_proof(capability)?;
        if !proof_result.is_valid() {
            return Ok(proof_result);
        }
        
        // Verify constraints
        let constraint_result = self.verify_capability_constraints(capability)?;
        if !constraint_result.is_valid() {
            return Ok(constraint_result);
        }
        
        // All checks passed
        Ok(VerificationResult::valid())
    }
    
    /// Revoke a capability
    pub fn revoke_capability(
        &self,
        capability_id: CapabilityId,
        reason: String,
    ) -> Result<(), CapabilityError> {
        // Get the capability
        let mut capability = self.get_capability(capability_id)?;
        
        // Update status
        capability.status = CapabilityStatus::Revoked(reason);
        
        // Store updated capability
        self.store_capability(&capability)?;
        
        // Notify observers
        for observer in &self.observers {
            observer.on_capability_revoked(&capability)?;
        }
        
        Ok(())
    }
}
```

## Capability-Based Authorization

### Authorization Service

Using capabilities for authorization:

```rust
pub struct CapabilityAuthorizationService {
    /// Capability registry
    registry: Arc<CapabilityRegistry>,
    
    /// Authorization policy engine
    policy_engine: Arc<AuthorizationPolicyEngine>,
    
    /// Authorization context provider
    context_provider: Arc<AuthorizationContextProvider>,
}

impl CapabilityAuthorizationService {
    /// Authorize an operation using capabilities
    pub fn authorize_operation(
        &self,
        principal: &Principal,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<AuthorizationResult, AuthorizationError> {
        // Map operation to required capability right and target
        let (right, target) = self.map_operation_to_capability(operation)?;
        
        // Check if principal has the required capability
        let capabilities = self.registry.get_capabilities_for_principal(
            principal.clone(),
            Some(right.clone()),
            Some(target.clone()),
        )?;
        
        if capabilities.is_empty() {
            // No direct capabilities found, check policy engine
            return self.policy_engine.authorize_operation(
                principal,
                operation,
                context,
            );
        }
        
        // Verify capabilities against context
        for capability in &capabilities {
            let verification_result = self.verify_capability_in_context(
                capability,
                operation,
                context,
            )?;
            
            if verification_result.is_valid() {
                // Capability is valid for this operation
                return Ok(AuthorizationResult::Authorized {
                    capability_id: capability.id,
                });
            }
        }
        
        // No valid capabilities found
        Ok(AuthorizationResult::Denied {
            reason: "No valid capabilities found for this operation".to_string(),
        })
    }
    
    /// Map operation to capability right and target
    fn map_operation_to_capability(
        &self,
        operation: &Operation,
    ) -> Result<(CapabilityRight, CapabilityTarget), AuthorizationError> {
        // Map operation type to capability right
        let right = match operation.operation_type {
            OperationType::Read => CapabilityRight::Read,
            OperationType::Create | OperationType::Update | OperationType::Delete => CapabilityRight::Write,
            OperationType::Execute { .. } => CapabilityRight::Execute,
            OperationType::Custom { .. } => {
                // Custom operations need special mapping
                self.map_custom_operation_to_right(operation)?
            }
        };
        
        // Map operation target to capability target
        let target = CapabilityTarget::Resource(operation.resource_id.clone());
        
        Ok((right, target))
    }
    
    /// Verify a capability in an operation context
    fn verify_capability_in_context(
        &self,
        capability: &Capability,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<VerificationResult, AuthorizationError> {
        // Basic capability verification
        let base_result = self.registry.verify_capability(capability)?;
        if !base_result.is_valid() {
            return Ok(base_result);
        }
        
        // Check capability constraints against context
        for constraint in &capability.constraints {
            let constraint_result = self.verify_constraint(
                constraint,
                operation,
                context,
            )?;
            
            if !constraint_result.is_valid() {
                return Ok(constraint_result);
            }
        }
        
        // All checks passed
        Ok(VerificationResult::valid())
    }
}
```

## Capability Constraints

### Constraint Model

Defining and enforcing capability constraints:

```rust
pub enum CapabilityConstraint {
    /// Time constraint
    TimeConstraint {
        /// Valid time window
        window: TimeWindow,
    },
    
    /// Location constraint
    LocationConstraint {
        /// Allowed locations
        allowed_locations: Vec<String>,
    },
    
    /// Parameter constraint
    ParameterConstraint {
        /// Parameter matcher
        parameter_matcher: ParameterMatcher,
    },
    
    /// Domain constraint
    DomainConstraint {
        /// Allowed domains
        allowed_domains: Vec<DomainId>,
    },
    
    /// Rate limit constraint
    RateLimitConstraint {
        /// Maximum uses
        max_uses: u64,
        /// Time period
        period: Duration,
    },
    
    /// Custom constraint
    CustomConstraint {
        /// Constraint type
        constraint_type: String,
        /// Constraint data
        data: Vec<u8>,
    },
}

pub struct ConstraintVerifier {
    /// Constraint evaluators for different constraint types
    evaluators: HashMap<String, Box<dyn ConstraintEvaluator>>,
    
    /// Context provider
    context_provider: Arc<ContextProvider>,
}

impl ConstraintVerifier {
    /// Verify a constraint in a given context
    pub fn verify_constraint(
        &self,
        constraint: &CapabilityConstraint,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<VerificationResult, ConstraintError> {
        match constraint {
            CapabilityConstraint::TimeConstraint { window } => {
                let current_time = context.time.unwrap_or_else(|| system.current_time());
                
                if window.contains(current_time) {
                    Ok(VerificationResult::valid())
                } else {
                    Ok(VerificationResult::invalid(
                        format!("Current time {} outside of allowed window {}", 
                            current_time, window)
                    ))
                }
            }
            
            CapabilityConstraint::LocationConstraint { allowed_locations } => {
                if let Some(location) = &context.location {
                    if allowed_locations.contains(location) {
                        Ok(VerificationResult::valid())
                    } else {
                        Ok(VerificationResult::invalid(
                            format!("Location {} not in allowed locations", location)
                        ))
                    }
                } else {
                    Ok(VerificationResult::invalid("No location in context"))
                }
            }
            
            CapabilityConstraint::ParameterConstraint { parameter_matcher } => {
                // Verify operation parameters match the constraint
                if parameter_matcher.matches(&operation.parameters) {
                    Ok(VerificationResult::valid())
                } else {
                    Ok(VerificationResult::invalid("Parameters do not match constraint"))
                }
            }
            
            CapabilityConstraint::DomainConstraint { allowed_domains } => {
                if let Some(domain) = &context.domain {
                    if allowed_domains.contains(domain) {
                        Ok(VerificationResult::valid())
                    } else {
                        Ok(VerificationResult::invalid(
                            format!("Domain {} not in allowed domains", domain)
                        ))
                    }
                } else {
                    Ok(VerificationResult::invalid("No domain in context"))
                }
            }
            
            CapabilityConstraint::RateLimitConstraint { max_uses, period } => {
                // Check usage count in the defined period
                let usage_count = self.context_provider.get_capability_usage_count(
                    context.capability_id.as_ref().unwrap(),
                    *period,
                )?;
                
                if usage_count < *max_uses {
                    Ok(VerificationResult::valid())
                } else {
                    Ok(VerificationResult::invalid(
                        format!("Rate limit exceeded: {} uses in {:?}", usage_count, period)
                    ))
                }
            }
            
            CapabilityConstraint::CustomConstraint { constraint_type, data } => {
                // Get the appropriate evaluator
                if let Some(evaluator) = self.evaluators.get(constraint_type) {
                    evaluator.evaluate_constraint(data, operation, context)
                } else {
                    Ok(VerificationResult::invalid(
                        format!("No evaluator for custom constraint type: {}", constraint_type)
                    ))
                }
            }
        }
    }
}
```

## Capability Delegation

### Delegation Model

Delegating capabilities to other principals:

```rust
pub struct CapabilityDelegationService {
    /// Capability registry
    registry: Arc<CapabilityRegistry>,
    
    /// Delegation policy engine
    policy_engine: Arc<DelegationPolicyEngine>,
}

impl CapabilityDelegationService {
    /// Delegate a capability to another principal
    pub fn delegate_capability(
        &self,
        capability_id: CapabilityId,
        from_principal: &Principal,
        to_principal: &Principal,
        attenuations: Vec<CapabilityAttenuation>,
        ttl: Option<Duration>,
    ) -> Result<Capability, DelegationError> {
        // Get the source capability
        let source_capability = self.registry.get_capability(capability_id)?;
        
        // Verify the source principal has the capability and can delegate it
        if source_capability.principal != *from_principal {
            return Err(DelegationError::Unauthorized(
                "Principal does not own the capability".to_string(),
            ));
        }
        
        // Check if the capability includes delegation right
        if !self.has_delegation_right(&source_capability)? {
            return Err(DelegationError::Unauthorized(
                "Principal does not have delegation rights for this capability".to_string(),
            ));
        }
        
        // Apply delegation policy
        let policy_result = self.policy_engine.check_delegation(
            &source_capability,
            from_principal,
            to_principal,
            &attenuations,
        )?;
        
        if !policy_result.is_allowed() {
            return Err(DelegationError::PolicyViolation(
                policy_result.reason().to_string(),
            ));
        }
        
        // Create attenuated capability
        let delegated_capability = self.create_delegated_capability(
            &source_capability,
            to_principal.clone(),
            attenuations,
            ttl,
        )?;
        
        Ok(delegated_capability)
    }
    
    /// Create a delegated capability with attenuations
    fn create_delegated_capability(
        &self,
        source: &Capability,
        to_principal: Principal,
        attenuations: Vec<CapabilityAttenuation>,
        ttl: Option<Duration>,
    ) -> Result<Capability, DelegationError> {
        // Start with source capability attributes
        let mut right = source.right.clone();
        let mut target = source.target.clone();
        let mut constraints = source.constraints.clone();
        
        // Apply attenuations
        for attenuation in &attenuations {
            match attenuation {
                CapabilityAttenuation::NarrowRight(narrowed_right) => {
                    // Ensure narrowed right is not expanding privileges
                    if !self.is_right_narrowing(&right, narrowed_right)? {
                        return Err(DelegationError::InvalidAttenuation(
                            "Narrowed right expands privileges".to_string(),
                        ));
                    }
                    right = narrowed_right.clone();
                }
                
                CapabilityAttenuation::NarrowTarget(narrowed_target) => {
                    // Ensure narrowed target is not expanding scope
                    if !self.is_target_narrowing(&target, narrowed_target)? {
                        return Err(DelegationError::InvalidAttenuation(
                            "Narrowed target expands scope".to_string(),
                        ));
                    }
                    target = narrowed_target.clone();
                }
                
                CapabilityAttenuation::AddConstraint(constraint) => {
                    constraints.push(constraint.clone());
                }
            }
        }
        
        // Set expiration if provided, or inherit from source
        let expires_at = match ttl {
            Some(duration) => Some(system.current_time() + duration),
            None => source.expires_at,
        };
        
        // Ensure delegated capability doesn't outlive source
        if let Some(source_expiry) = source.expires_at {
            if let Some(delegated_expiry) = expires_at {
                if delegated_expiry > source_expiry {
                    return Err(DelegationError::InvalidAttenuation(
                        "Delegated capability cannot outlive source capability".to_string(),
                    ));
                }
            }
        }
        
        // Create delegation metadata
        let delegation_metadata = CapabilityMetadata::new()
            .with_field("delegated_from", source.id.to_string())
            .with_field("delegation_time", system.current_time().to_string());
        
        // Combine with source metadata
        let metadata = source.metadata.clone().merge(delegation_metadata);
        
        // Create the delegated capability
        let delegated_capability = self.registry.create_capability(
            to_principal,
            right,
            target,
            constraints,
            Some(metadata),
            ttl,
        )?;
        
        Ok(delegated_capability)
    }
}
```

## Cross-Domain Capabilities

### Cross-Domain Model

Managing capabilities across domain boundaries:

```rust
pub struct CrossDomainCapabilityService {
    /// Local capability registry
    local_registry: Arc<CapabilityRegistry>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
    
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    
    /// Capability translation service
    translator: Arc<CapabilityTranslator>,
}

impl CrossDomainCapabilityService {
    /// Create a cross-domain capability
    pub fn create_cross_domain_capability(
        &self,
        principal: Principal,
        right: CapabilityRight,
        target: CapabilityTarget,
        constraints: Vec<CapabilityConstraint>,
        target_domains: Vec<DomainId>,
        ttl: Option<Duration>,
    ) -> Result<Vec<CrossDomainCapabilityResult>, CrossDomainError> {
        // Create local capability first
        let local_capability = self.local_registry.create_capability(
            principal.clone(),
            right.clone(),
            target.clone(),
            constraints.clone(),
            None,
            ttl,
        )?;
        
        // Translate for each target domain
        let mut results = Vec::new();
        
        for domain_id in &target_domains {
            // Get domain info
            let domain = self.domain_registry.get_domain(*domain_id)?;
            
            // Translate capability for target domain
            let translated = self.translator.translate_capability(
                &local_capability,
                *domain_id,
            )?;
            
            // Create capability request
            let request = CrossDomainCapabilityRequest {
                capability: translated,
                source_domain: system.local_domain_id(),
                source_capability_id: local_capability.id,
                request_id: RequestId::generate(),
                timestamp: system.current_time(),
            };
            
            // Send request to target domain
            let response = self.messenger.send_capability_request(*domain_id, request)?;
            
            // Store result
            results.push(CrossDomainCapabilityResult {
                domain_id: *domain_id,
                status: response.status,
                capability_id: response.capability_id,
                error: response.error,
            });
        }
        
        Ok(results)
    }
    
    /// Verify a cross-domain capability
    pub fn verify_cross_domain_capability(
        &self,
        capability: &Capability,
        source_domain: DomainId,
    ) -> Result<VerificationResult, CrossDomainError> {
        // Check if domain is trusted
        let domain = self.domain_registry.get_domain(source_domain)?;
        
        if !domain.is_trusted() {
            return Ok(VerificationResult::invalid(
                format!("Domain {} is not trusted", source_domain)
            ));
        }
        
        // Verify capability signature against domain's verification key
        let domain_key = self.domain_registry.get_domain_verification_key(source_domain)?;
        
        let signature_result = self.verify_capability_signature(
            capability,
            &domain_key,
        )?;
        
        if !signature_result.is_valid() {
            return Ok(signature_result);
        }
        
        // Verify capability against local policies
        let policy_result = self.verify_against_local_policies(
            capability,
            source_domain,
        )?;
        
        if !policy_result.is_valid() {
            return Ok(policy_result);
        }
        
        // All checks passed
        Ok(VerificationResult::valid())
    }
}
```

## Usage Examples

### Basic Capability Authorization

```rust
// Create a capability for a user to read a document
let capability = capability_registry.create_capability(
    Principal::User(user_id),
    CapabilityRight::Read,
    CapabilityTarget::Resource(document_id),
    vec![
        // Allow only during business hours
        CapabilityConstraint::TimeConstraint {
            window: TimeWindow::recurring(
                TimeOfDay::new(9, 0, 0),
                TimeOfDay::new(17, 0, 0),
                vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, 
                     Weekday::Thursday, Weekday::Friday],
            ),
        },
    ],
    None,
    Some(Duration::from_days(30)),
)?;

println!("Created capability: {}", capability.id);

// Authorize an operation using the capability
let operation = Operation::new(
    OperationType::Read,
    document_id,
    OperationParameters::default(),
    AuthInfo::new().with_principal(Principal::User(user_id)),
);

let auth_context = AuthorizationContext::new()
    .with_time(system.current_time())
    .with_location("office")
    .with_source_ip("192.168.1.100");

let auth_result = authorization_service.authorize_operation(
    &Principal::User(user_id),
    &operation,
    &auth_context,
)?;

match auth_result {
    AuthorizationResult::Authorized { capability_id } => {
        println!("Operation authorized using capability: {}", capability_id);
        // Proceed with operation
    }
    AuthorizationResult::Denied { reason } => {
        println!("Operation denied: {}", reason);
        // Handle unauthorized access
    }
}
```

### Capability Delegation

```rust
// User delegates read capability to a service
let delegated_capability = delegation_service.delegate_capability(
    user_capability_id,
    &Principal::User(user_id),
    &Principal::Service(service_id),
    vec![
        // Add additional constraint for service
        CapabilityAttenuation::AddConstraint(
            CapabilityConstraint::RateLimitConstraint {
                max_uses: 10,
                period: Duration::from_hours(1),
            },
        ),
    ],
    Some(Duration::from_hours(4)),
)?;

println!("Delegated capability to service: {}", delegated_capability.id);

// Service uses the delegated capability
let service_operation = Operation::new(
    OperationType::Read,
    document_id,
    OperationParameters::default(),
    AuthInfo::new().with_principal(Principal::Service(service_id)),
);

let service_auth_context = AuthorizationContext::new()
    .with_time(system.current_time())
    .with_capability_id(delegated_capability.id);

let service_auth_result = authorization_service.authorize_operation(
    &Principal::Service(service_id),
    &service_operation,
    &service_auth_context,
)?;

println!("Service authorization result: {:?}", service_auth_result);
```

### Cross-Domain Capabilities

```rust
// Create a capability valid across domains
let cross_domain_results = cross_domain_capability_service.create_cross_domain_capability(
    Principal::User(user_id),
    CapabilityRight::Read,
    CapabilityTarget::ResourceType(ResourceType::new("document")),
    vec![
        CapabilityConstraint::DomainConstraint {
            allowed_domains: vec![local_domain_id, partner_domain_id],
        },
    ],
    vec![partner_domain_id],
    Some(Duration::from_days(7)),
)?;

// Print cross-domain capability results
for result in &cross_domain_results {
    println!("Domain {}: Status: {:?}, Capability: {:?}",
        result.domain_id, result.status, result.capability_id);
}

// Use capability in remote domain
let remote_capability_id = cross_domain_results[0].capability_id
    .ok_or_else(|| anyhow!("No capability created in remote domain"))?;

let verification_result = cross_domain_capability_service.verify_cross_domain_capability(
    &remote_capability,
    local_domain_id,
)?;

if verification_result.is_valid() {
    println!("Cross-domain capability verified successfully");
} else {
    println!("Cross-domain capability verification failed: {}", 
        verification_result.error_message().unwrap_or_default());
}
```

## Implementation Status

The current implementation status of the Security Capability Model:

- ✅ Core capability model
- ✅ Capability registry
- ✅ Basic capability authorization
- ⚠️ Capability constraints (partially implemented)
- ⚠️ Capability delegation (partially implemented)
- ⚠️ Cross-domain capabilities (partially implemented)
- ❌ Capability revocation chains (not yet implemented)
- ❌ Advanced capability restrictions (not yet implemented)

## Future Enhancements

Planned future enhancements for the Security Capability Model:

1. **Capability Chains**: More sophisticated capability chains and dependency tracking
2. **Attenuation Language**: Domain-specific language for capability attenuation
3. **Recursive Delegation Control**: Better management of recursive delegation paths
4. **Intention-Based Capabilities**: Capabilities based on operation intention rather than type
5. **Zero-Knowledge Capabilities**: Proving capability possession without revealing identity
6. **Capability Attestation**: External attestation of capability grants
7. **Automated Capability Management**: AI-based capability lifecycle management
8. **Distributed Capability Consensus**: Consensus protocols for cross-domain capabilities 