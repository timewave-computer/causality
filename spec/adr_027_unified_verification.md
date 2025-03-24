# ADR-027: Unified Verification Framework

## Status

Accepted

## Implementation Status

This ADR has been fully implemented. The unified verification framework is now the standard approach for verification in Causality. Key implementation components include:

- UnifiedProof structure with standardized format and metadata
- Consistent verification interfaces across different proof types
- Common proof generation pipeline
- Integration with the content addressing system
- Cross-domain proof verification capabilities
- Support for different zero-knowledge proof systems

The implementation provides a comprehensive verification framework that can handle various proof types while maintaining a consistent developer experience. Documentation is available in [docs/src/proof_generation.md](/docs/src/proof_generation.md) and [docs/src/zero_knowledge_workflows.md](/docs/src/zero_knowledge_workflows.md).

## Context

Our system currently relies on several separate but overlapping verification mechanisms:

1. **ZK Proofs** for register operations, ensuring state transitions are valid without revealing sensitive information
2. **Time Map Verification** for temporal consistency, ensuring operations respect causal ordering across domains
3. **Controller Label Verification** for ancestral validity, guaranteeing the provenance of resources
4. **Effect Validation** for logical consistency, confirming operations satisfy business rules

These mechanisms have evolved organically as we've added features to the system, but this separation has led to several issues:

- **Fragmented Security Model**: Security analysis requires understanding multiple verification systems with separate assumptions and guarantees
- **Redundant Implementation**: Similar verification logic is reimplemented across systems
- **Complexity for Developers**: Learning each verification system independently increases cognitive load
- **Inconsistent Error Handling**: Each system has its own error patterns and reporting mechanisms
- **Cross-domain Inconsistency**: Verification across domains requires coordinating multiple systems
- **Optimization Challenges**: Improvements to one verification system don't benefit others

As I've worked on cross-domain verification over the years, I've observed that these seemingly separate concerns eventually converge at the implementation level. What starts as clearly delineated verification domains inevitably develops overlapping responsibilities, creating a taxonomical confusion that becomes increasingly difficult to maintain.

## Decision

We will implement a **Unified Verification Framework** that brings together all forms of verification through a common abstraction. This framework will provide a consistent approach to validation, proof generation, and security reasoning while maintaining the unique characteristics of each verification domain.

```rust
/// Core verification abstraction
pub trait Verifiable {
    /// The type of proof this verification produces
    type Proof;
    
    /// Specifics about what is being verified
    type Subject;
    
    /// Generate a proof for this verifiable item
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError>;
    
    /// Verify a proof against this verifiable item
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError>;
    
    /// Get verification dependencies
    fn dependencies(&self) -> Vec<VerificationDependency>;
    
    /// Get required capabilities for verification
    fn required_capabilities(&self) -> HashSet<VerificationCapability>;
}

/// Context for verification operations
pub struct VerificationContext {
    /// Domain-specific context information
    pub domain_context: HashMap<DomainId, DomainContext>,
    
    /// Time map context for temporal verification
    pub time_map: TimeMap,
    
    /// Registry of known controller labels
    pub controller_registry: ControllerRegistry,
    
    /// Effect history for logical verification
    pub effect_history: EffectHistory,
    
    /// Verification capabilities available in this context
    pub capabilities: HashSet<VerificationCapability>,
    
    /// Prover implementation to use
    pub prover: Arc<dyn Prover>,
    
    /// Verification options
    pub options: VerificationOptions,
}

/// A unified proof that can contain multiple verification aspects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnifiedProof {
    /// ZK proof components (if applicable)
    pub zk_components: Option<ZkProofData>,
    
    /// Temporal verification data (time map snapshot)
    pub temporal_components: Option<TemporalProofData>,
    
    /// Ancestral verification data (controller paths)
    pub ancestral_components: Option<AncestralProofData>,
    
    /// Logical verification data (effect validation)
    pub logical_components: Option<LogicalProofData>,
    
    /// Cross-domain verification data
    pub cross_domain_components: Option<CrossDomainProofData>,
    
    /// Metadata about this proof
    pub metadata: HashMap<String, Value>,
    
    /// Proof generation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Signature over the proof contents (if applicable)
    pub signature: Option<Signature>,
}
```

### Implementation for Different Verification Aspects

Each verification aspect will implement the `Verifiable` trait with its specific proof needs:

#### 1. Register Operations with ZK Proofs

```rust
/// ZK verification for register operations
impl Verifiable for RegisterOperation {
    type Proof = UnifiedProof;
    type Subject = RegisterState;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate ZK proof for register operation
        let zk_proof = generate_zk_proof(self, context)?;
        
        // Create unified proof with ZK component
        let proof = UnifiedProof {
            zk_components: Some(zk_proof),
            temporal_components: None,
            ancestral_components: None,
            logical_components: None,
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Extract ZK proof component
        let zk_proof = proof.zk_components.as_ref()
            .ok_or(VerificationError::MissingProofComponent("zk_components"))?;
        
        // Verify ZK proof
        verify_zk_proof(self, zk_proof, context)
    }
    
    fn dependencies(&self) -> Vec<VerificationDependency> {
        // Register operations may depend on previous register states
        // Implementation details...
        vec![]
    }
    
    fn required_capabilities(&self) -> HashSet<VerificationCapability> {
        // Register operations require ZK proving capability
        let mut capabilities = HashSet::new();
        capabilities.insert(VerificationCapability::ZkProving);
        capabilities
    }
}
```

#### 2. Temporal Verification

```rust
/// Temporal verification for operations
impl Verifiable for Operation {
    type Proof = UnifiedProof;
    type Subject = TemporalConsistency;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate temporal proof
        let temporal_proof = generate_temporal_proof(self, &context.time_map)?;
        
        // Create unified proof with temporal component
        let proof = UnifiedProof {
            zk_components: None,
            temporal_components: Some(temporal_proof),
            ancestral_components: None,
            logical_components: None,
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Extract temporal proof component
        let temporal_proof = proof.temporal_components.as_ref()
            .ok_or(VerificationError::MissingProofComponent("temporal_components"))?;
        
        // Verify temporal consistency
        verify_temporal_consistency(self, temporal_proof, &context.time_map)
    }
    
    fn dependencies(&self) -> Vec<VerificationDependency> {
        // Temporal verification depends on time map entries
        // Implementation details...
        vec![]
    }
    
    fn required_capabilities(&self) -> HashSet<VerificationCapability> {
        // Temporal verification requires time map capability
        let mut capabilities = HashSet::new();
        capabilities.insert(VerificationCapability::TimeMapAccess);
        capabilities
    }
}
```

#### 3. Ancestral Verification (Controller Labels)

```rust
/// Ancestral verification for resources
impl Verifiable for Resource {
    type Proof = UnifiedProof;
    type Subject = ControllerLabel;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate ancestral proof
        let ancestral_proof = generate_ancestral_proof(
            &self.controller_label,
            &context.controller_registry
        )?;
        
        // Create unified proof with ancestral component
        let proof = UnifiedProof {
            zk_components: None,
            temporal_components: None,
            ancestral_components: Some(ancestral_proof),
            logical_components: None,
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Extract ancestral proof component
        let ancestral_proof = proof.ancestral_components.as_ref()
            .ok_or(VerificationError::MissingProofComponent("ancestral_components"))?;
        
        // Verify controller label ancestry
        verify_controller_ancestry(
            &self.controller_label,
            ancestral_proof,
            &context.controller_registry
        )
    }
    
    fn dependencies(&self) -> Vec<VerificationDependency> {
        // Ancestral verification depends on controller registry
        // Implementation details...
        vec![]
    }
    
    fn required_capabilities(&self) -> HashSet<VerificationCapability> {
        // Ancestral verification requires controller registry access
        let mut capabilities = HashSet::new();
        capabilities.insert(VerificationCapability::ControllerRegistryAccess);
        capabilities
    }
}
```

#### 4. Logical Verification (Effect Validation)

```rust
/// Logical verification for effects
impl Verifiable for Effect {
    type Proof = UnifiedProof;
    type Subject = EffectValidity;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate logical proof
        let logical_proof = generate_logical_proof(
            self,
            &context.effect_history
        )?;
        
        // Create unified proof with logical component
        let proof = UnifiedProof {
            zk_components: None,
            temporal_components: None,
            ancestral_components: None,
            logical_components: Some(logical_proof),
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Extract logical proof component
        let logical_proof = proof.logical_components.as_ref()
            .ok_or(VerificationError::MissingProofComponent("logical_components"))?;
        
        // Verify logical consistency
        verify_logical_consistency(
            self,
            logical_proof,
            &context.effect_history
        )
    }
    
    fn dependencies(&self) -> Vec<VerificationDependency> {
        // Logical verification depends on previous effects
        // Implementation details...
        vec![]
    }
    
    fn required_capabilities(&self) -> HashSet<VerificationCapability> {
        // Logical verification requires effect history access
        let mut capabilities = HashSet::new();
        capabilities.insert(VerificationCapability::EffectHistoryAccess);
        capabilities
    }
}
```

### 5. Dual Verification (Combined Temporal and Ancestral)

The real power of this framework emerges when we combine verification aspects:

```rust
/// Dual verification for cross-domain operations
impl Verifiable for CrossDomainOperation {
    type Proof = UnifiedProof;
    type Subject = CrossDomainValidity;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate temporal proof
        let temporal_proof = generate_temporal_proof(self, &context.time_map)?;
        
        // Generate ancestral proof
        let ancestral_proof = generate_ancestral_proof(
            &self.resource.controller_label,
            &context.controller_registry
        )?;
        
        // Generate cross-domain proof
        let cross_domain_proof = generate_cross_domain_proof(
            self,
            &context.domain_context
        )?;
        
        // Create unified proof with multiple components
        let proof = UnifiedProof {
            zk_components: None,
            temporal_components: Some(temporal_proof),
            ancestral_components: Some(ancestral_proof),
            logical_components: None,
            cross_domain_components: Some(cross_domain_proof),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Verify temporal component
        let temporal_valid = if let Some(temporal_proof) = &proof.temporal_components {
            verify_temporal_consistency(self, temporal_proof, &context.time_map)?
        } else {
            return Err(VerificationError::MissingProofComponent("temporal_components"));
        };
        
        // Verify ancestral component
        let ancestral_valid = if let Some(ancestral_proof) = &proof.ancestral_components {
            verify_controller_ancestry(
                &self.resource.controller_label,
                ancestral_proof,
                &context.controller_registry
            )?
        } else {
            return Err(VerificationError::MissingProofComponent("ancestral_components"));
        };
        
        // Verify cross-domain component
        let cross_domain_valid = if let Some(cross_domain_proof) = &proof.cross_domain_components {
            verify_cross_domain_validity(
                self,
                cross_domain_proof,
                &context.domain_context
            )?
        } else {
            return Err(VerificationError::MissingProofComponent("cross_domain_components"));
        };
        
        // All verification aspects must pass
        Ok(temporal_valid && ancestral_valid && cross_domain_valid)
    }
    
    fn dependencies(&self) -> Vec<VerificationDependency> {
        // Dual verification has multiple dependencies
        // Implementation details...
        vec![]
    }
    
    fn required_capabilities(&self) -> HashSet<VerificationCapability> {
        // Dual verification requires multiple capabilities
        let mut capabilities = HashSet::new();
        capabilities.insert(VerificationCapability::TimeMapAccess);
        capabilities.insert(VerificationCapability::ControllerRegistryAccess);
        capabilities.insert(VerificationCapability::CrossDomainAccess);
        capabilities
    }
}
```

### Integrating ZK Proofs with Other Verification Types

For operations requiring both ZK proofs and other verifications:

```rust
/// ZK register operation with temporal verification
impl Verifiable for RegisterOperationWithTemporal {
    type Proof = UnifiedProof;
    type Subject = (RegisterState, TemporalConsistency);
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate ZK proof
        let zk_proof = generate_zk_proof(&self.operation, context)?;
        
        // Generate temporal proof
        let temporal_proof = generate_temporal_proof(&self.operation, &context.time_map)?;
        
        // Create unified proof with both components
        let proof = UnifiedProof {
            zk_components: Some(zk_proof),
            temporal_components: Some(temporal_proof),
            ancestral_components: None,
            logical_components: None,
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Verify ZK component
        let zk_valid = if let Some(zk_proof) = &proof.zk_components {
            verify_zk_proof(&self.operation, zk_proof, context)?
        } else {
            return Err(VerificationError::MissingProofComponent("zk_components"));
        };
        
        // Verify temporal component
        let temporal_valid = if let Some(temporal_proof) = &proof.temporal_components {
            verify_temporal_consistency(&self.operation, temporal_proof, &context.time_map)?
        } else {
            return Err(VerificationError::MissingProofComponent("temporal_components"));
        };
        
        // Both verifications must pass
        Ok(zk_valid && temporal_valid)
    }
    
    // Other implementation details...
}
```

### Verification Service

The unified framework will be exposed through a verification service:

```rust
/// Unified verification service
pub struct VerificationService {
    /// Verification context
    context: VerificationContext,
    
    /// Proof cache
    proof_cache: Cache<ProofCacheKey, UnifiedProof>,
    
    /// Verification result cache
    verification_cache: Cache<VerificationCacheKey, bool>,
}

impl VerificationService {
    /// Create a new verification service
    pub fn new(context: VerificationContext) -> Self {
        Self {
            context,
            proof_cache: Cache::new(CacheConfig::default()),
            verification_cache: Cache::new(CacheConfig::default()),
        }
    }
    
    /// Generate a proof for a verifiable item
    pub fn prove<V: Verifiable>(&self, item: &V) -> Result<V::Proof, VerificationError> {
        // Check capabilities
        self.check_capabilities(item.required_capabilities())?;
        
        // Generate proof
        item.prove(&self.context)
    }
    
    /// Verify a proof against a verifiable item
    pub fn verify<V: Verifiable>(&self, item: &V, proof: &V::Proof) -> Result<bool, VerificationError> {
        // Check capabilities
        self.check_capabilities(item.required_capabilities())?;
        
        // Verify proof
        item.verify(proof, &self.context)
    }
    
    /// Check if the service has the required capabilities
    fn check_capabilities(&self, required: HashSet<VerificationCapability>) -> Result<(), VerificationError> {
        for capability in &required {
            if !self.context.capabilities.contains(capability) {
                return Err(VerificationError::MissingCapability(capability.clone()));
            }
        }
        
        Ok(())
    }
}
```

## Practical Usage Examples

### Example 1: Verifying a Register Operation

```rust
// Create verification service
let verification_service = VerificationService::new(context);

// Create register operation
let operation = RegisterOperation::new(
    RegisterOpType::Update,
    register_id,
    new_contents
);

// Generate unified proof
let proof = verification_service.prove(&operation)?;

// Verify the proof
let is_valid = verification_service.verify(&operation, &proof)?;

if is_valid {
    // Apply the operation
    register_system.apply_operation(operation, proof)?;
} else {
    return Err(ApplicationError::InvalidProof);
}
```

### Example 2: Cross-Domain Resource Transfer with Dual Verification

```rust
// Create cross-domain operation
let cross_domain_op = CrossDomainOperation::new(
    OperationType::Transfer,
    resource,
    source_domain,
    target_domain
);

// Generate unified proof with both temporal and ancestral components
let proof = verification_service.prove(&cross_domain_op)?;

// Verify the proof
let is_valid = verification_service.verify(&cross_domain_op, &proof)?;

if is_valid {
    // Apply the cross-domain transfer
    domain_bridge.transfer_resource(cross_domain_op, proof)?;
} else {
    return Err(ApplicationError::InvalidCrossDomainProof);
}
```

### Example 3: Combining ZK and Temporal Verification

```rust
// Create ZK register operation with temporal context
let operation = RegisterOperationWithTemporal {
    operation: register_op,
    temporal_context: temporal_context,
};

// Generate unified proof with both ZK and temporal components
let proof = verification_service.prove(&operation)?;

// Verify the proof
let is_valid = verification_service.verify(&operation, &proof)?;

if is_valid {
    // Apply the operation
    zkvm_system.apply_operation(operation, proof)?;
} else {
    return Err(ApplicationError::InvalidZkTemporalProof);
}
```

## Consequences

### Positive

1. **Simplified Conceptual Model**: Developers work with a single verification abstraction, with a unified proof concept.

2. **Improved Security Reasoning**: Security properties can be analyzed in a cohesive framework.

3. **Enhanced Composability**: Different verification aspects can be combined seamlessly.

4. **Consistent Error Handling**: Standardized error reporting across all verification types.

5. **Optimized Implementation**: Common verification logic can be shared and optimized once.

6. **Clearer Dependencies**: Explicit dependency tracking between verification aspects.

7. **More Flexible Cross-Domain Verification**: Verification aspects can be combined according to domain-specific needs.

8. **Better Developer Experience**: Learning one verification model rather than several separate ones.

9. **Future-proofing**: New verification types can be added within the existing framework.

### Negative

1. **Initial Implementation Complexity**: Significant refactoring of existing verification systems required.

2. **Potential Performance Overhead**: The unified approach may introduce some abstraction overhead.

3. **Migration Challenges**: Existing code using specific verification types will need to be updated.

4. **Learning Curve**: Understanding the unified model requires grasping multiple verification concepts.

### Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Security implications of unified verification | Thorough security review of the unified model |
| Performance degradation | Profile and optimize critical verification paths |
| Increased complexity for simple verifications | Provide simplified APIs for common verification patterns |
| Compatibility issues with existing code | Create adapter layer for legacy verification interfaces |

## Implementation Plan

I propose we implement this unified framework in phases:

### Phase 1: Core Framework (3 Weeks)

- Implement `Verifiable` trait and `UnifiedProof` structure
- Create basic verification context and infrastructure
- Develop verification service with basic capabilities

### Phase 2: ZK and Temporal Integration (4 Weeks)

- Migrate ZK verification to the unified framework
- Implement temporal verification within the new model
- Create combined ZK + temporal verification patterns

### Phase 3: Ancestral and Logical Integration (3 Weeks)

- Migrate controller label verification to the unified framework
- Implement logical effect validation in the new model
- Create dual verification (temporal + ancestral) examples

### Phase 4: Cross-Domain Integration (3 Weeks)

- Implement cross-domain verification logic
- Create comprehensive dual verification for cross-domain operations
- Optimize verification paths for common scenarios

### Phase 5: Migration and Documentation (3 Weeks)

- Create adapters for legacy verification interfaces
- Update all code to use the unified framework
- Develop comprehensive documentation with examples
- Conduct training for developers on the new model

## Alternatives Considered

### Alternative 1: Enhanced Integration Without Unification

Instead of a unified model, we could develop better integration points between the existing verification systems.

**Pros**:
- Less disruptive to existing code
- Potentially simpler to implement

**Cons**:
- Doesn't address the fundamental conceptual fragmentation
- Integration complexity would likely grow over time
- Security reasoning would remain fragmented

This approach would be easier in the short term but wouldn't solve the underlying architectural issues.

### Alternative 2: Verification Composition Without Shared Abstraction

We could create a composition framework for existing verification types without introducing a shared abstraction.

**Pros**:
- More flexible composition patterns
- Less refactoring of existing systems

**Cons**:
- No uniform verification interface
- Duplicated logic across verification systems
- More complex security analysis

While this would improve composability, it wouldn't address the conceptual fragmentation or redundant implementation issues.

### Alternative 3: Specialized Verification Framework for Each Domain

We could optimize each verification system individually and provide standardized interfaces specific to each domain.

**Pros**:
- Domain-specific optimizations
- Less abstraction overhead

**Cons**:
- Continued fragmentation of the verification model
- Redundant implementation of common patterns
- More complex developer experience

This approach might offer better performance in specific domains but would perpetuate the conceptual complexity.