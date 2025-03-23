# ADR-026: Unified Program Model

## Status

Proposed

## Context

Our system currently makes a clear distinction between two types of programs:

1. **Logic Programs**: Handle business logic, implement custom workflows, and coordinate operations across domains.

2. **Account Programs**: Serve as user entry points, manage assets/resources, and mediate all external domain interactions.

While this separation has served us well in clarifying responsibilities, it has led to several challenges:

- **Duplicated Core Functionality**: Both types share significant implementation overlap in state management, effect handling, schema evolution, and upgrade mechanisms.

- **Inconsistent Developer Experience**: Developers must learn two similar but distinct APIs, leading to confusion and reduced productivity.

- **Deployment Complexity**: Separate deployment pipelines and management systems increase operational overhead.

- **Testing Overhead**: Similar but separate testing frameworks for each program type.

- **Feature Divergence**: Improvements to one program type don't automatically benefit the other.

In practice, we've observed that the distinction isn't as clear-cut as initially designed. Logic programs often need account-like capabilities for coherent user experiences, while account programs sometimes need complex logic traditionally reserved for logic programs.

## Decision

We will unify logic programs and account programs into a single **Program** concept with capability-based differentiation:

```rust
/// Unified Program model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Unique program identifier
    pub id: ProgramId,
    
    /// Program owner
    pub owner: UserId,
    
    /// Program type (more a semantic indicator than functional constraint)
    pub program_type: ProgramType,
    
    /// Program capabilities (determines what the program can do)
    pub capabilities: HashSet<ProgramCapability>,
    
    /// Program state
    pub state: ProgramState,
    
    /// Program schema (defines state structure)
    pub schema: Schema,
    
    /// Safe state policy for upgrades
    pub safe_state_policy: SafeStatePolicy,
    
    /// Effect processing pipeline
    pub effect_pipeline: EffectPipeline,
    
    /// Security settings
    pub security: SecuritySettings,
    
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Program types (used primarily for categorization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgramType {
    /// Logic program (primary purpose is business logic)
    Logic,
    
    /// Account program (primary purpose is asset management)
    Account,
    
    /// Hybrid program (combines both purposes)
    Hybrid,
}

/// Program capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgramCapability {
    /// Can handle external deposits
    ExternalDeposit,
    
    /// Can handle external withdrawals
    ExternalWithdrawal,
    
    /// Can manage external resources
    ResourceManagement,
    
    /// Can invoke cross-program calls
    CrossProgramInvocation,
    
    /// Can observe external facts
    FactObservation,
    
    /// Can update internal state
    StateUpdate,
    
    /// Can manage register operations
    RegisterOperation,
    
    /// Can delegate capabilities
    CapabilityDelegation,
    
    /// Can handle user authentication
    UserAuthentication,
    
    /// Can generate ZK proofs
    ZkProofGeneration,
    
    // Additional capabilities
    // ...
}
```

This unified model offers several key improvements:

### 1. Capability-Based Differentiation

Rather than rigid type distinctions, programs are differentiated by their capabilities:

```rust
// Create an account-like program
let account_program = Program::new(ProgramType::Account)
    .with_capabilities(vec![
        ProgramCapability::ExternalDeposit,
        ProgramCapability::ExternalWithdrawal, 
        ProgramCapability::ResourceManagement,
        ProgramCapability::UserAuthentication
    ])
    .with_schema(account_schema)
    .with_safe_state_policy(SafeStatePolicy::NoInTransitResources);

// Create a logic-like program
let logic_program = Program::new(ProgramType::Logic)
    .with_capabilities(vec![
        ProgramCapability::CrossProgramInvocation,
        ProgramCapability::FactObservation,
        ProgramCapability::StateUpdate
    ])
    .with_schema(logic_schema)
    .with_safe_state_policy(SafeStatePolicy::NoPendingCalls);

// Create a hybrid program
let hybrid_program = Program::new(ProgramType::Hybrid)
    .with_capabilities(vec![
        ProgramCapability::ExternalDeposit,
        ProgramCapability::ResourceManagement,
        ProgramCapability::CrossProgramInvocation,
        ProgramCapability::FactObservation,
        ProgramCapability::RegisterOperation
    ])
    .with_schema(hybrid_schema)
    .with_safe_state_policy(SafeStatePolicy::Custom(custom_policy));
```

This capability-based approach provides fine-grained control over program permissions while maintaining security boundaries.

### 2. Unified State Management

All programs use the same state management system:

```rust
/// Program state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramState {
    /// Current schema version
    pub schema_version: Version,
    
    /// State data conforming to schema
    pub data: Value,
    
    /// Effect history
    pub effect_dag: EffectDAG,
    
    /// Resource registry
    pub resources: ResourceRegistry,
    
    /// Register operations (if applicable)
    pub registers: Option<RegisterRegistry>,
    
    /// External domain connections
    pub domain_connections: HashMap<DomainId, DomainConnection>,
}

impl ProgramState {
    /// Access state data with type validation
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, StateError> {
        // Implementation
    }
    
    /// Update state data with schema validation
    pub fn update<T: Serialize + DeserializeOwned>(
        &mut self, 
        path: &str, 
        value: T
    ) -> Result<(), StateError> {
        // Implementation
    }
    
    /// Apply an effect to update state
    pub fn apply_effect(
        &mut self, 
        effect: &Effect,
        validator: &dyn EffectValidator
    ) -> Result<EffectResult, EffectError> {
        // Implementation
    }
}
```

### 3. Unified Effect Pipeline

All programs process effects through the same pipeline, with capability-based validation:

```rust
/// Process an effect in any program
pub async fn process_effect(
    program: &mut Program,
    effect: Effect,
) -> Result<EffectResult, EffectError> {
    // Check capability requirements for this effect
    let required_capabilities = effect.required_capabilities();
    for capability in &required_capabilities {
        if !program.capabilities.contains(capability) {
            return Err(EffectError::CapabilityMissing(capability.clone()));
        }
    }
    
    // Validate effect
    program.effect_pipeline.validate_effect(&effect)?;
    
    // Process the effect based on type
    match effect.effect_type {
        EffectType::Deposit { .. } => process_deposit(program, effect).await?,
        EffectType::Withdraw { .. } => process_withdrawal(program, effect).await?,
        EffectType::Transfer { .. } => process_transfer(program, effect).await?,
        EffectType::Invoke { .. } => process_invocation(program, effect).await?,
        EffectType::Observe { .. } => process_observation(program, effect).await?,
        // Other effect types...
        _ => process_generic_effect(program, effect).await?,
    }
}
```

### 4. Unified Deployment and Upgrade

Both program types share the same deployment and upgrade mechanisms:

```rust
/// Deploy a program (works for any program type)
pub async fn deploy_program(
    program: Program,
    deployment_config: DeploymentConfig,
) -> Result<DeploymentResult, DeploymentError> {
    // Validate program schema
    schema_validator.validate(&program.schema)?;
    
    // Validate capabilities
    capability_validator.validate_capabilities(&program.capabilities)?;
    
    // Deploy program
    let deployment_id = deployer.deploy(program, deployment_config).await?;
    
    // Register program with registry
    program_registry.register(program.id, deployment_id).await?;
    
    Ok(DeploymentResult {
        program_id: program.id,
        deployment_id,
        timestamp: Utc::now(),
    })
}

/// Upgrade a program (works for any program type)
pub async fn upgrade_program(
    program_id: ProgramId,
    new_program: Program,
    upgrade_config: UpgradeConfig,
) -> Result<UpgradeResult, UpgradeError> {
    // Get current program
    let current_program = program_registry.get(program_id).await?;
    
    // Check safe state
    if !safe_state_checker.is_in_safe_state(&current_program)? {
        return Err(UpgradeError::NotInSafeState);
    }
    
    // Validate schema compatibility
    schema_validator.validate_compatibility(
        &current_program.schema,
        &new_program.schema,
    )?;
    
    // Perform migration if needed
    let migrated_state = if upgrade_config.migration_needed {
        state_migrator.migrate(
            &current_program.state,
            &current_program.schema,
            &new_program.schema,
        ).await?
    } else {
        current_program.state.clone()
    };
    
    // Create upgraded program
    let upgraded_program = new_program.with_state(migrated_state);
    
    // Deploy upgraded program
    let deployment_id = deployer.deploy(upgraded_program.clone(), upgrade_config.deployment_config).await?;
    
    // Update registry
    program_registry.update(program_id, deployment_id, upgraded_program).await?;
    
    Ok(UpgradeResult {
        program_id,
        deployment_id,
        timestamp: Utc::now(),
    })
}
```

### 5. Capability Verification

A key aspect of the unified model is runtime capability verification:

```rust
/// Verify program capabilities for an operation
pub fn verify_capabilities(
    program: &Program,
    required_capabilities: &[ProgramCapability],
) -> Result<(), CapabilityError> {
    for capability in required_capabilities {
        if !program.capabilities.contains(capability) {
            return Err(CapabilityError::MissingCapability {
                program_id: program.id.clone(),
                capability: capability.clone(),
            });
        }
    }
    
    Ok(())
}

/// Capability delegation
pub fn delegate_capability(
    from_program: &Program,
    to_program: &mut Program,
    capability: ProgramCapability,
    constraints: CapabilityConstraints,
) -> Result<(), DelegationError> {
    // Check if source program has the capability
    if !from_program.capabilities.contains(&capability) {
        return Err(DelegationError::SourceMissingCapability(capability));
    }
    
    // Check if source can delegate capabilities
    if !from_program.capabilities.contains(&ProgramCapability::CapabilityDelegation) {
        return Err(DelegationError::CannotDelegate);
    }
    
    // Add constrained capability to target program
    to_program.capabilities.insert(capability);
    to_program.capability_constraints.insert(capability, constraints);
    
    Ok(())
}
```

## Consequences

### Positive

1. **Reduced Duplication**: Consolidates overlapping functionality into a single codebase.

2. **Simplified Developer Experience**: Developers learn one program model with consistent patterns.

3. **Feature Parity**: Improvements benefit all program types simultaneously.

4. **Flexible Capability Model**: Programs can have precisely the capabilities they need.

5. **Streamlined Deployment**: Single deployment and upgrade pipeline for all programs.

6. **Unified Testing**: Shared testing framework for all program types.

7. **Natural Evolution**: Programs can evolve their capabilities over time based on changing requirements.

8. **Better Code Organization**: Core functionality is centralized rather than scattered across program types.

9. **Security Through Capability Checking**: Clear capability requirements and validation at runtime.

### Negative

1. **Migration Complexity**: Existing programs need to be migrated to the new model.

2. **Learning Curve**: While simpler overall, developers need to learn the capability model.

3. **Potential Security Risks**: Incorrectly granted capabilities could compromise security boundaries.

4. **Performance Overhead**: Capability checking adds minor runtime overhead.

### Mitigation Strategies

1. **Migration Tools**: Build automated tools to migrate existing programs to the new model.

2. **Capability Presets**: Provide preset capability collections for common program types.

3. **Static Analysis**: Develop tools to analyze capability usage and identify potential issues.

4. **Performance Optimization**: Cache capability checks and optimize the verification path.

5. **Comprehensive Documentation**: Create clear documentation with migration guides and capability reference.

## Implementation Plan

We'll implement this unification in several phases:

### Phase 1: Core Unified Model (3 weeks)

- Define the unified `Program` structure with capabilities
- Implement basic state management
- Create validation framework for capabilities

### Phase 2: Effect Pipeline Integration (4 weeks)

- Migrate effect handling to the unified model
- Implement capability-based effect validation
- Create adapters for existing effect processors

### Phase 3: Deployment and Upgrade (3 weeks)

- Unify deployment mechanisms
- Implement capability-aware upgrade logic
- Create migration tools for existing programs

### Phase 4: Developer Tools (2 weeks)

- Update CLI tools to support the unified model
- Create capability presets for common use cases
- Build visualization tools for capability graphs

### Phase 5: Migration (4 weeks)

- Migrate existing logic programs
- Migrate existing account programs
- Update documentation and examples

## Example Workflows

### Example 1: Creating a User Account Program

```rust
// Create a standard user account program
let account_program = Program::new(ProgramType::Account)
    .with_owner(user_id)
    .with_capabilities(vec![
        ProgramCapability::ExternalDeposit,
        ProgramCapability::ExternalWithdrawal,
        ProgramCapability::ResourceManagement,
        ProgramCapability::UserAuthentication,
        ProgramCapability::RegisterOperation,
    ])
    .with_schema(account_schema)
    .with_safe_state_policy(SafeStatePolicy::NoInTransitResources)
    .with_metadata(account_metadata);

// Deploy the program
let result = deploy_program(account_program, DeploymentConfig::default()).await?;

// User can now deposit assets
let deposit_effect = Effect::new(EffectType::Deposit {
    domain: "ethereum".into(),
    asset: "ETH".into(),
    amount: 1000000000000000000, // 1 ETH
});

process_effect(&mut account_program, deposit_effect).await?;
```

### Example 2: Creating a Trading Program

```rust
// Create a trading program with hybrid capabilities
let trading_program = Program::new(ProgramType::Hybrid)
    .with_owner(developer_id)
    .with_capabilities(vec![
        // Logic capabilities
        ProgramCapability::CrossProgramInvocation,
        ProgramCapability::FactObservation,
        ProgramCapability::StateUpdate,
        
        // Account capabilities
        ProgramCapability::ResourceManagement,
        ProgramCapability::RegisterOperation,
    ])
    .with_schema(trading_schema)
    .with_safe_state_policy(SafeStatePolicy::Custom(trading_safe_state))
    .with_metadata(trading_metadata);

// Deploy the program
let result = deploy_program(trading_program, DeploymentConfig::default()).await?;

// Program can now observe facts and manage resources
let observe_effect = Effect::new(EffectType::Observe {
    fact_type: "price".into(),
    parameters: {
        let mut params = HashMap::new();
        params.insert("pair".to_string(), "ETH/USD".into());
        params
    },
});

process_effect(&mut trading_program, observe_effect).await?;
```

### Example 3: Upgrading a Program with New Capabilities

```rust
// Get existing program
let mut program = program_registry.get(program_id).await?;

// Create upgraded version with new capabilities
let upgraded_program = program.clone()
    .with_capabilities(
        program.capabilities.iter()
            .chain(vec![
                ProgramCapability::ZkProofGeneration,
                ProgramCapability::CapabilityDelegation,
            ].iter())
            .cloned()
            .collect()
    )
    .with_schema(new_schema);

// Perform upgrade
let result = upgrade_program(
    program_id,
    upgraded_program,
    UpgradeConfig {
        migration_needed: true,
        deployment_config: DeploymentConfig::default(),
    },
).await?;
```

### Example 4: Capability Delegation Between Programs

```rust
// Get two programs
let source_program = program_registry.get(source_id).await?;
let mut target_program = program_registry.get(target_id).await?;

// Delegate resource management capability with constraints
delegate_capability(
    &source_program,
    &mut target_program,
    ProgramCapability::ResourceManagement,
    CapabilityConstraints {
        resource_types: vec!["ETH".into(), "USDC".into()],
        max_amount: Some(1000000000), // 1000 USDC
        expiration: Some(Utc::now() + Duration::days(7)),
    },
).await?;

// Update target program
program_registry.update(target_id, target_program.deployment_id, target_program).await?;
```

### Example 5: Creating a Multi-Chain Token Bridge

```rust
// Create a token bridge program
let bridge_program = Program::new(ProgramType::Hybrid)
    .with_owner(operator_id)
    .with_capabilities(vec![
        // Cross-domain capabilities
        ProgramCapability::ExternalDeposit,
        ProgramCapability::ExternalWithdrawal,
        ProgramCapability::ResourceManagement,
        
        // Logic capabilities
        ProgramCapability::CrossProgramInvocation,
        ProgramCapability::FactObservation,
        
        // Security capabilities
        ProgramCapability::ZkProofGeneration,
        ProgramCapability::RegisterOperation,
    ])
    .with_schema(bridge_schema)
    .with_safe_state_policy(SafeStatePolicy::NoInTransitResources)
    .with_effect_pipeline(cross_domain_pipeline)
    .with_security(SecuritySettings {
        required_confirmations: HashMap::from([
            ("ethereum".into(), 12),
            ("solana".into(), 32),
        ]),
        authorization_rules: bridge_auth_rules,
    });

// Deploy the program
let result = deploy_program(bridge_program, DeploymentConfig::default()).await?;
```

## Alternatives Considered

### 1. Enhanced Interface Inheritance

Instead of unifying the models, we could enhance both program types to inherit from a common base interface.

**Pros**:
- Less migration effort
- Maintains clear separation of concerns

**Cons**:
- Duplicated implementations remain
- Interface bloat as capabilities grow
- Less flexibility for hybrid programs

**Why rejected**: This would only partially solve the duplication problem and would still require separate deployment pipelines and upgrade mechanisms.

### 2. Composition Instead of Unification

Programs could be built by composing capability modules rather than having a unified type with capabilities.

**Pros**:
- More flexible and extensible
- Clean separation of responsibilities

**Cons**:
- More complex architecture
- Potential performance overhead
- Harder to reason about security boundaries

**Why rejected**: While elegant in theory, the composition approach would significantly increase system complexity and make capability verification more challenging.

### 3. Maintain Separation with Shared Libraries

Keep programs separate but extract shared functionality into common libraries.

**Pros**:
- Minimal disruption to existing code
- Clear conceptual separation maintained

**Cons**:
- Does not address the underlying conceptual redundancy
- Similar but different APIs remain confusing
- Shared libraries can drift from program implementations

**Why rejected**: This would address the symptom (code duplication) but not the root cause (conceptual redundancy).

## Conclusion

The unified program model represents a natural evolution of our architecture, reducing duplication while enhancing flexibility. By moving from a type-based distinction to a capability-based model, we maintain strong security boundaries while allowing programs to precisely declare what they can do.

This change will require significant refactoring but provides a solid foundation for future growth. The capability model in particular gives us a more nuanced way to express program permissions and evolve them over time.

I've encountered similar challenges in designing service boundaries in distributed systems, and I've found that capability-based models often provide the right balance of flexibility and security. They allow you to start with clear boundaries (traditional "account" vs "logic" distinctions) but gracefully accommodate the "in-between" cases that inevitably emerge as requirements evolve.

Rather than maintain artificial boundaries that don't match real-world usage patterns, this unified model embraces the reality that programs often need varying combinations of capabilities. It's a pragmatic approach that should make our system more coherent and easier to work with in the long run.