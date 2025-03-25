<!-- Model for resource relationships -->
<!-- Original file: docs/src/resource_relationship_model.md -->

# Content-Addressed Resource Relationship Model

This document outlines the relationship model for resources in the Causality system, reflecting the unified ResourceRegister model with content addressing, three-layer effect architecture, and capability-based authorization.

## Overview

Resource relationships define how different ResourceRegisters interact, connect, and relate to each other within and across domains. In the unified content-addressed model, these relationships themselves are content-addressed objects, providing verifiability, immutability, and cross-domain consistency.

## Content-Addressed Relationship Structure

Resource relationships are implemented as content-addressed immutable objects:

```rust
/// A content-addressed relationship between ResourceRegisters
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceRelationship<C: ExecutionContext> {
    /// Content hash that uniquely identifies this relationship
    pub content_hash: ContentHash,
    
    /// Relationship type
    pub relationship_type: RelationshipType,
    
    /// Source resource reference
    pub source: ContentRef<ResourceRegister<C>>,
    
    /// Target resource reference
    pub target: ContentRef<ResourceRegister<C>>,
    
    /// Relationship direction
    pub direction: RelationshipDirection,
    
    /// Additional attributes
    pub attributes: HashMap<String, Value>,
    
    /// Temporal context
    pub observed_at: ContentRef<TimeMapSnapshot>,
    
    /// Capability information for this relationship
    pub capabilities: ContentRef<CapabilitySet>,
    
    /// Verification information
    pub verification: VerificationInfo,
    
    /// Execution context for this relationship
    pub context: PhantomData<C>,
}

/// Implementation of ContentAddressed trait
impl<C: ExecutionContext> ContentAddressed for ResourceRelationship<C> {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        // Calculate hash from contents and verify it matches the stored hash
        calculate_content_hash(self) == self.content_hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // Serialize to canonical binary format
        serialize_canonical(self).expect("Failed to serialize ResourceRelationship")
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // Deserialize from binary format
        deserialize_canonical(bytes)
    }
}
```

## Relationship Types

The relationship model defines various types of connections between resources:

```rust
/// Types of relationships between resources
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Ownership relationship (A owns B)
    Ownership,
    
    /// Containment relationship (A contains B)
    Containment,
    
    /// Dependency relationship (A depends on B)
    Dependency,
    
    /// Reference relationship (A references B)
    Reference,
    
    /// Transformation relationship (A transforms into B)
    Transformation,
    
    /// Delegation relationship (A delegates to B)
    Delegation,
    
    /// Cross-domain relationship (A in domain X is related to B in domain Y)
    CrossDomain,
    
    /// Custom relationship with a specified name
    Custom(String),
}
```

## Relationship Directionality

Relationships can have different directionality:

```rust
/// Directionality of relationships
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipDirection {
    /// One-way relationship from source to target
    Directed,
    
    /// Bidirectional relationship between source and target
    Bidirectional,
}
```

## Three-Layer Effect Architecture for Relationship Operations

Relationship operations are implemented using the three-layer effect architecture:

### 1. Algebraic Effect Layer

```rust
/// Relationship effect for relationship operations
pub enum RelationshipEffect<C: ExecutionContext, R> {
    /// Create a relationship between resources
    CreateRelationship {
        source_id: RegisterId,
        target_id: RegisterId,
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
        attributes: HashMap<String, Value>,
        continuation: Box<dyn Continuation<CreateRelationshipResult, R>>,
    },
    
    /// Query relationships for a resource
    QueryRelationships {
        resource_id: RegisterId,
        relationship_type: Option<RelationshipType>,
        direction: Option<RelationshipDirection>,
        continuation: Box<dyn Continuation<QueryRelationshipResult, R>>,
    },
    
    /// Update a relationship
    UpdateRelationship {
        relationship_id: ContentHash,
        attributes: HashMap<String, Value>,
        continuation: Box<dyn Continuation<UpdateRelationshipResult, R>>,
    },
    
    /// Delete a relationship
    DeleteRelationship {
        relationship_id: ContentHash,
        continuation: Box<dyn Continuation<DeleteRelationshipResult, R>>,
    },
}
```

### 2. Effect Constraints Layer

```rust
/// Type constraints for relationship effects
pub trait RelationshipEffectHandler<C: ExecutionContext>: Send + Sync {
    /// Process a relationship effect
    fn handle_relationship_effect<R>(
        &self,
        effect: RelationshipEffect<C, R>,
        context: &C,
    ) -> Result<R, RelationshipError>;
    
    /// Validate a relationship effect
    fn validate_relationship_effect<R>(
        &self,
        effect: &RelationshipEffect<C, R>,
        context: &C,
    ) -> Result<ValidationResult, ValidationError>;
}
```

### 3. Domain Implementation Layer (TEL)

```rust
effect EVMCreateRelationship implements CreateRelationshipEffect {
    // State fields
    source_id: RegisterId
    target_id: RegisterId
    relationship_type: RelationshipType
    direction: RelationshipDirection
    attributes: HashMap<String, Value>
    domain_id: DomainId
    
    // Implementation of required methods
    fn source() -> RegisterId { return this.source_id; }
    fn target() -> RegisterId { return this.target_id; }
    fn type() -> RelationshipType { return this.relationship_type; }
    
    // Domain-specific validation
    fn validate_evm_relationship(context) -> Result<(), ValidationError> {
        // Validate that both resources exist in this domain
        require(
            context.resource_exists(this.source_id) && 
            context.resource_exists(this.target_id),
            "Both resources must exist in the domain"
        );
        
        // Check resource authorization
        require(
            context.has_capability(context.caller, this.source_id, Capability::CreateRelationship),
            "Caller lacks capability to create relationships for source resource"
        );
        
        return Ok(());
    }
    
    // Execution logic
    fn execute(context) -> Result<ContentHash, EffectError> {
        // Create the relationship in EVM storage
        let relationship = ResourceRelationship {
            content_hash: ContentHash::default(),  // Will be calculated later
            relationship_type: this.relationship_type,
            source: ContentRef::new(&context.get_resource(this.source_id)),
            target: ContentRef::new(&context.get_resource(this.target_id)),
            direction: this.direction,
            attributes: this.attributes,
            observed_at: ContentRef::new(&context.time_snapshot),
            capabilities: create_default_capabilities(context.caller),
            verification: create_default_verification(),
            context: PhantomData,
        };
        
        // Calculate content hash
        let relationship_with_hash = relationship.with_calculated_hash();
        
        // Store the relationship
        context.store_object(relationship_with_hash.clone());
        
        // Store EVM-specific relationship mapping
        context.evm_storage.set_relationship_mapping(
            this.source_id,
            this.target_id,
            relationship_with_hash.content_hash.clone()
        );
        
        // Return the content hash of the created relationship
        return Ok(relationship_with_hash.content_hash);
    }
}
```

## Unified Operation Model for Relationship Management

Relationship operations are implemented using the unified operation model:

```rust
/// Creates a relationship between resources
pub fn create_resource_relationship<C: ExecutionContext>(
    source: ContentRef<ResourceRegister<C>>,
    target: ContentRef<ResourceRegister<C>>,
    relationship_type: RelationshipType,
    direction: RelationshipDirection,
    attributes: HashMap<String, Value>,
    context: &C,
) -> Result<Operation<C, ResourceRelationship<C>>> {
    // Create an operation to create a relationship
    let operation = Operation::new(
        OperationType::CreateRelationship,
        context.clone(),
        move |ctx| {
            // Resolve the source and target resources
            let source_resource = source.resolve(&ctx.storage)?;
            let target_resource = target.resolve(&ctx.storage)?;
            
            // Check capabilities
            require_capability(
                &ctx.initiator,
                &source_resource,
                &Capability::CreateRelationship,
                ctx,
            )?;
            
            // Create capability set for the relationship
            let capabilities = create_default_relationship_capabilities(&ctx.initiator);
            
            // Create the relationship
            let relationship = ResourceRelationship {
                content_hash: ContentHash::default(),  // Will be calculated later
                relationship_type,
                source: source.clone(),
                target: target.clone(),
                direction,
                attributes,
                observed_at: ContentRef::new(&ctx.time_snapshot),
                capabilities: ContentRef::new(&capabilities),
                verification: VerificationInfo {
                    content_hash: ContentHash::default(),  // Will be calculated later
                    status: VerificationStatus::Unverified,
                    method: VerificationMethod::None,
                    proof: None,
                    last_verified: None,
                },
                context: PhantomData,
            };
            
            // Calculate content hash
            let relationship_with_hash = relationship.with_calculated_hash();
            
            // Store capabilities
            ctx.effect_handler.handle_effect(StorageEffect::StoreObject {
                object: capabilities,
                continuation: Box::new(|_| Ok(())),
            })?;
            
            // Store the relationship
            ctx.effect_handler.handle_effect(StorageEffect::StoreObject {
                object: relationship_with_hash.clone(),
                continuation: Box::new(|_| Ok(())),
            })?;
            
            // Return the relationship
            Ok(relationship_with_hash)
        }
    );
    
    Ok(operation)
}
```

## Cross-Domain Relationships

The content-addressed relationship model enables verifiable cross-domain relationships:

```rust
/// Creates a cross-domain relationship
pub fn create_cross_domain_relationship<C: ExecutionContext>(
    source: ContentRef<ResourceRegister<C>>,
    target: ContentRef<ResourceRegister<C>>,
    source_domain: DomainId,
    target_domain: DomainId,
    attributes: HashMap<String, Value>,
    context: &C,
) -> Result<Operation<C, ResourceRelationship<C>>> {
    // Create an operation to create a cross-domain relationship
    let operation = Operation::new(
        OperationType::CreateCrossDomainRelationship,
        context.clone(),
        move |ctx| {
            // Resolve the source and target resources
            let source_resource = source.resolve(&ctx.storage)?;
            let target_resource = target.resolve(&ctx.storage)?;
            
            // Check cross-domain capabilities
            require_capability(
                &ctx.initiator,
                &source_resource,
                &Capability::CrossDomainRelationship,
                ctx,
            )?;
            
            // Create the cross-domain operation
            let cross_domain_op = CrossDomainOperation::new(
                OperationType::CreateRelationship,
                source_resource.clone(),
                target_resource.clone(),
                source_domain.clone(),
                target_domain.clone()
            );
            
            // Generate unified proof with dual validation
            let verification_context = VerificationContext {
                domain_context: ctx.domain_context.clone(),
                time_map: ctx.time_map.clone(),
                controller_registry: ctx.controller_registry.clone(),
                effect_history: ctx.effect_history.clone(),
                capabilities: ctx.verification_capabilities.clone(),
                prover: ctx.prover.clone(),
                options: VerificationOptions::default(),
            };
            
            let proof = generate_unified_proof(&cross_domain_op, &verification_context)?;
            
            // Create capability set for the relationship
            let capabilities = create_cross_domain_capabilities(&ctx.initiator, &source_domain, &target_domain);
            
            // Create the relationship
            let relationship = ResourceRelationship {
                content_hash: ContentHash::default(),  // Will be calculated later
                relationship_type: RelationshipType::CrossDomain,
                source: source.clone(),
                target: target.clone(),
                direction: RelationshipDirection::Directed,
                attributes: attributes.clone(),
                observed_at: ContentRef::new(&ctx.time_snapshot),
                capabilities: ContentRef::new(&capabilities),
                verification: VerificationInfo {
                    content_hash: ContentHash::default(),  // Will be calculated later
                    status: VerificationStatus::Verified,
                    method: VerificationMethod::UnifiedProof,
                    proof: Some(ContentRef::new(&proof)),
                    last_verified: Some(ctx.time_snapshot.clone()),
                },
                context: PhantomData,
            };
            
            // Calculate content hash
            let relationship_with_hash = relationship.with_calculated_hash();
            
            // Store capabilities, proof, and relationship
            ctx.effect_handler.handle_effect(StorageEffect::StoreObject {
                object: capabilities,
                continuation: Box::new(|_| Ok(())),
            })?;
            
            ctx.effect_handler.handle_effect(StorageEffect::StoreObject {
                object: proof,
                continuation: Box::new(|_| Ok(())),
            })?;
            
            ctx.effect_handler.handle_effect(StorageEffect::StoreObject {
                object: relationship_with_hash.clone(),
                continuation: Box::new(|_| Ok(())),
            })?;
            
            // Register in both domains
            register_cross_domain_relationship(
                &relationship_with_hash, 
                &source_domain, 
                &target_domain, 
                ctx
            ).await?;
            
            // Return the relationship
            Ok(relationship_with_hash)
        }
    );
    
    Ok(operation)
}
```

## Relationship Queries and Navigation

The relationship model provides efficient ways to query and navigate resource relationships:

```rust
/// Query relationships for a resource
pub fn query_resource_relationships<C: ExecutionContext>(
    resource: ContentRef<ResourceRegister<C>>,
    query_params: RelationshipQueryParams,
    context: &C,
) -> Result<Operation<C, Vec<ResourceRelationship<C>>>> {
    // Create an operation to query relationships
    let operation = Operation::new(
        OperationType::QueryRelationships,
        context.clone(),
        move |ctx| {
            // Resolve the resource
            let resource_obj = resource.resolve(&ctx.storage)?;
            
            // Check capability
            require_capability(
                &ctx.initiator,
                &resource_obj,
                &Capability::ReadRelationships,
                ctx,
            )?;
            
            // Create content pattern for the query
            let pattern = ContentPattern::new()
                .with_tag("type", "ResourceRelationship")
                .with_tag("source_resource_id", resource_obj.id.to_string());
                
            if let Some(rel_type) = &query_params.relationship_type {
                pattern.with_tag("relationship_type", rel_type.to_string());
            }
            
            if let Some(direction) = &query_params.direction {
                pattern.with_tag("direction", direction.to_string());
            }
            
            // Query the content-addressed storage
            let relationship_hashes = ctx.storage.list(&pattern)?;
            
            // Resolve relationships
            let mut relationships = Vec::new();
            for hash in relationship_hashes {
                if let Ok(relationship) = ctx.storage.get::<ResourceRelationship<C>>(&hash) {
                    // Check if relationship matches all query criteria
                    if relationship_matches_query(&relationship, &query_params) {
                        relationships.push(relationship);
                    }
                }
            }
            
            // Sort by timestamp (most recent first)
            relationships.sort_by(|a, b| {
                let a_time = a.observed_at.resolve(&ctx.storage)
                    .map(|t| t.timestamp)
                    .unwrap_or(0);
                let b_time = b.observed_at.resolve(&ctx.storage)
                    .map(|t| t.timestamp)
                    .unwrap_or(0);
                b_time.cmp(&a_time)
            });
            
            Ok(relationships)
        }
    );
    
    Ok(operation)
}
```

## Relationship Verification

Relationships are verified using the unified verification framework:

```rust
impl<C: ExecutionContext> Verifiable for ResourceRelationship<C> {
    type Proof = UnifiedProof;
    type Subject = RelationshipValidity;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate temporal proof for relationship timestamp
        let temporal_proof = generate_temporal_proof(
            &self.observed_at.resolve(&context.storage)?,
            &context.time_map
        )?;
        
        // Generate ancestral proof for relationship
        let ancestral_proof = if self.relationship_type == RelationshipType::CrossDomain {
            // For cross-domain relationships, generate dual validation proof
            generate_cross_domain_proof(self, &context.domain_context)?
        } else {
            // For regular relationships, generate standard ancestral proof
            generate_relationship_ancestral_proof(self, &context.controller_registry)?
        };
        
        // Generate logical proof for relationship constraints
        let logical_proof = generate_relationship_logical_proof(self, &context.effect_history)?;
        
        // Create unified proof
        let proof = UnifiedProof {
            content_hash: ContentHash::default(), // Will be calculated
            temporal_components: Some(temporal_proof),
            ancestral_components: Some(ancestral_proof),
            logical_components: Some(logical_proof),
            zk_components: None, // Optional for privacy
            cross_domain_components: if self.relationship_type == RelationshipType::CrossDomain {
                Some(generate_cross_domain_components(self, context)?)
            } else {
                None
            },
            metadata: HashMap::new(),
            created_at: DateTime::<Utc>::from(SystemTime::now()),
            signature: None,
        };
        
        // Calculate content hash for proof
        let proof_with_hash = proof.with_calculated_hash();
        
        Ok(proof_with_hash)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Verify temporal component
        let temporal_valid = if let Some(temporal_proof) = &proof.temporal_components {
            verify_temporal_proof(
                &self.observed_at.resolve(&context.storage)?,
                temporal_proof,
                &context.time_map
            )?
        } else {
            return Err(VerificationError::MissingProofComponent("temporal_components"));
        };
        
        // Verify ancestral component
        let ancestral_valid = if let Some(ancestral_proof) = &proof.ancestral_components {
            if self.relationship_type == RelationshipType::CrossDomain {
                verify_cross_domain_proof(self, ancestral_proof, &context.domain_context)?
            } else {
                verify_relationship_ancestral_proof(self, ancestral_proof, &context.controller_registry)?
            }
        } else {
            return Err(VerificationError::MissingProofComponent("ancestral_components"));
        };
        
        // Verify logical component
        let logical_valid = if let Some(logical_proof) = &proof.logical_components {
            verify_relationship_logical_proof(self, logical_proof, &context.effect_history)?
        } else {
            return Err(VerificationError::MissingProofComponent("logical_components"));
        };
        
        // Verify cross-domain component if applicable
        let cross_domain_valid = if self.relationship_type == RelationshipType::CrossDomain {
            if let Some(cross_domain_proof) = &proof.cross_domain_components {
                verify_cross_domain_components(self, cross_domain_proof, context)?
            } else {
                return Err(VerificationError::MissingProofComponent("cross_domain_components"));
            }
        } else {
            true
        };
        
        // All verification aspects must pass
        Ok(temporal_valid && ancestral_valid && logical_valid && cross_domain_valid)
    }
}
```

## Relationship-Based Capability Control

Relationships can be used to control capability delegation and authorization:

```rust
/// Creates capabilities based on a relationship
pub fn create_relationship_based_capability<C: ExecutionContext>(
    relationship: &ResourceRelationship<C>,
    rights: Vec<Right>,
    context: &C,
) -> Result<Capability, CapabilityError> {
    // Check if initiator has authority to create capabilities from this relationship
    if !context.relationship_authorization.can_create_capability(
        &context.initiator,
        relationship,
        &rights
    ) {
        return Err(CapabilityError::Unauthorized);
    }
    
    // Create a capability that references the relationship
    let capability = Capability {
        content_hash: ContentHash::default(), // Will be calculated
        operation: OperationType::from_rights(&rights),
        resource: if relationship.direction == RelationshipDirection::Directed {
            relationship.target.resolve(&context.storage)?.id
        } else {
            // For bidirectional relationships, the capability can apply to either resource
            match context.capability_target {
                CapabilityTarget::Source => relationship.source.resolve(&context.storage)?.id,
                CapabilityTarget::Target => relationship.target.resolve(&context.storage)?.id,
                _ => return Err(CapabilityError::InvalidTarget),
            }
        },
        conditions: vec![
            CapabilityCondition::RequiresRelationship(
                relationship.content_hash.clone(),
                relationship.relationship_type.clone()
            )
        ],
        issuer: context.initiator.clone(),
        expires_at: context.capability_expiration.clone(),
        signature: context.sign_capability(&relationship.content_hash)?,
    };
    
    // Calculate content hash
    let capability_with_hash = capability.with_calculated_hash();
    
    Ok(capability_with_hash)
}
```

## Conclusion

The content-addressed resource relationship model provides a robust framework for defining, managing, and verifying relationships between resources in the Causality system. By leveraging content addressing, the three-layer effect architecture, and capability-based authorization, the relationship model ensures that connections between resources are immutable, verifiable, and secure across domain boundaries.

Key benefits of this approach include:

1. **Content-Addressed Immutability**: Relationships are immutable content-addressed objects
2. **Cross-Domain Verification**: Relationships can span domains with cryptographic verification
3. **Capability-Based Access**: Access to relationships is controlled through unforgeable capabilities
4. **Unified Verification**: Relationships are verified using the unified verification framework
5. **Three-Layer Effect Architecture**: Relationship operations follow the three-layer effect model

This unified approach simplifies the mental model for developers, ensures consistent relationship management across domains, and provides strong security guarantees through cryptographic verification.