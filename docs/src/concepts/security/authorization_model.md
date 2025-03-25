<!-- Model for security authorization -->
<!-- Original file: docs/src/security_authorization_model.md -->

# Security Authorization Model in Causality

## Overview

This document describes the authorization model within the Causality architecture. The authorization model provides a comprehensive framework for making access control decisions throughout the system. It integrates capability-based security with policy-based authorization and contextual awareness to create a flexible, fine-grained authorization system that supports the distributed and temporal nature of Causality.

## Core Concepts

### Authorization Components

The authorization model consists of several key components:

```rust
pub struct AuthorizationSystem {
    /// Capability-based authorization service
    capability_service: Arc<CapabilityAuthorizationService>,
    
    /// Policy-based authorization service
    policy_service: Arc<PolicyAuthorizationService>,
    
    /// Context-aware authorization service
    context_service: Arc<ContextAwareAuthorizationService>,
    
    /// Authorization decision cache
    decision_cache: Arc<AuthorizationDecisionCache>,
    
    /// Authorization observers
    observers: Vec<Box<dyn AuthorizationObserver>>,
}
```

### Authorization Context

The context for authorization decisions:

```rust
pub struct AuthorizationContext {
    /// Principal making the request
    principal: Option<Principal>,
    
    /// Operation being performed
    operation: Option<Operation>,
    
    /// Resource being accessed
    resource: Option<ResourceReference>,
    
    /// Domain in which the operation is performed
    domain: Option<DomainId>,
    
    /// Timestamp of the authorization request
    timestamp: Option<Timestamp>,
    
    /// Location information
    location: Option<LocationInfo>,
    
    /// Device information
    device: Option<DeviceInfo>,
    
    /// Network information
    network: Option<NetworkInfo>,
    
    /// Authentication context
    authentication: Option<AuthenticationContext>,
    
    /// Additional context attributes
    attributes: HashMap<String, String>,
}
```

## Authorization Services

### Unified Authorization Service

The main entry point for authorization decisions:

```rust
impl AuthorizationSystem {
    /// Authorize an operation request
    pub fn authorize(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<AuthorizationDecision, AuthorizationError> {
        // Start timing the authorization
        let start_time = Instant::now();
        
        // Check decision cache if enabled
        if let Some(cached_decision) = self.check_cache(request)? {
            return Ok(cached_decision);
        }
        
        // Build context for authorization
        let context = self.build_authorization_context(request)?;
        
        // First try capability-based authorization
        let cap_result = self.capability_service.authorize(
            request.principal(),
            request.operation(),
            &context,
        )?;
        
        // If capability authorization was definitive, use it
        if cap_result.is_definitive() {
            let decision = self.create_decision_from_capability_result(cap_result)?;
            self.cache_decision(request, &decision)?;
            self.notify_observers(request, &decision)?;
            return Ok(decision);
        }
        
        // Next try policy-based authorization
        let policy_result = self.policy_service.authorize(
            request.principal(),
            request.operation(),
            &context,
        )?;
        
        // If policy authorization was definitive, use it
        if policy_result.is_definitive() {
            let decision = self.create_decision_from_policy_result(policy_result)?;
            self.cache_decision(request, &decision)?;
            self.notify_observers(request, &decision)?;
            return Ok(decision);
        }
        
        // Finally try context-aware authorization
        let context_result = self.context_service.authorize(
            request.principal(),
            request.operation(),
            &context,
        )?;
        
        // Create decision from context result
        let decision = self.create_decision_from_context_result(context_result)?;
        
        // Record metrics
        let elapsed = start_time.elapsed();
        metrics::histogram!("authorization.decision_time", elapsed);
        metrics::increment_counter!("authorization.request_count");
        
        // Cache and notify
        self.cache_decision(request, &decision)?;
        self.notify_observers(request, &decision)?;
        
        Ok(decision)
    }
}
```

### Policy-Based Authorization

Authorization based on defined policies:

```rust
pub struct PolicyAuthorizationService {
    /// Policy store
    policy_store: Arc<PolicyStore>,
    
    /// Policy evaluator
    evaluator: Arc<PolicyEvaluator>,
    
    /// Policy resolver
    resolver: Arc<PolicyResolver>,
}

impl PolicyAuthorizationService {
    /// Authorize an operation using policies
    pub fn authorize(
        &self,
        principal: &Principal,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<PolicyAuthorizationResult, AuthorizationError> {
        // Resolve applicable policies
        let policies = self.resolver.resolve_policies(principal, operation, context)?;
        
        if policies.is_empty() {
            return Ok(PolicyAuthorizationResult::Indeterminate {
                reason: "No applicable policies found".to_string(),
            });
        }
        
        // Evaluate policies
        let mut result = PolicyAuthorizationResult::Indeterminate {
            reason: "No policy evaluation result".to_string(),
        };
        
        for policy in policies {
            let policy_result = self.evaluator.evaluate_policy(&policy, principal, operation, context)?;
            
            // Apply policy result according to policy effect and combining algorithm
            result = self.apply_policy_result(result, policy_result, &policy.combining_algorithm())?;
            
            // If we have a definitive deny and policy says to break on first applicable,
            // stop evaluating
            if matches!(result, PolicyAuthorizationResult::Deny { .. }) && 
               policy.rule_combining_algorithm() == RuleCombiningAlgorithm::FirstApplicable {
                break;
            }
        }
        
        Ok(result)
    }
}
```

### Context-Aware Authorization

Authorization considering dynamic context:

```rust
pub struct ContextAwareAuthorizationService {
    /// Context evaluators
    evaluators: Vec<Box<dyn ContextEvaluator>>,
    
    /// Risk assessment engine
    risk_engine: Arc<RiskAssessmentEngine>,
    
    /// Behavioral analysis engine
    behavioral_engine: Arc<BehavioralAnalysisEngine>,
    
    /// Anomaly detection engine
    anomaly_engine: Arc<AnomalyDetectionEngine>,
}

impl ContextAwareAuthorizationService {
    /// Authorize an operation using context evaluation
    pub fn authorize(
        &self,
        principal: &Principal,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<ContextAuthorizationResult, AuthorizationError> {
        // Run context evaluators
        let mut evaluations = Vec::new();
        
        for evaluator in &self.evaluators {
            let eval_result = evaluator.evaluate(principal, operation, context)?;
            evaluations.push(eval_result);
        }
        
        // Assess risk
        let risk_level = self.risk_engine.assess_risk(principal, operation, context, &evaluations)?;
        
        // Perform behavioral analysis
        let behavioral_score = self.behavioral_engine.analyze_behavior(principal, operation, context)?;
        
        // Detect anomalies
        let anomalies = self.anomaly_engine.detect_anomalies(principal, operation, context)?;
        
        // Combine results
        let result = self.combine_results(risk_level, behavioral_score, anomalies, &evaluations)?;
        
        Ok(result)
    }
}
```

## Authorization Policies

### Policy Definition

Defining authorization policies:

```rust
pub struct AuthorizationPolicy {
    /// Unique policy identifier
    id: PolicyId,
    
    /// Policy name
    name: String,
    
    /// Policy description
    description: String,
    
    /// Policy target
    target: PolicyTarget,
    
    /// Policy rules
    rules: Vec<PolicyRule>,
    
    /// Rule combining algorithm
    rule_combining_algorithm: RuleCombiningAlgorithm,
    
    /// Policy effect
    effect: PolicyEffect,
    
    /// Policy priority
    priority: u32,
    
    /// Policy status
    status: PolicyStatus,
    
    /// Policy metadata
    metadata: HashMap<String, String>,
    
    /// Policy version
    version: u32,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Last updated timestamp
    updated_at: Timestamp,
}

pub struct PolicyRule {
    /// Rule identifier
    id: String,
    
    /// Rule description
    description: String,
    
    /// Rule target
    target: Option<PolicyTarget>,
    
    /// Rule condition
    condition: RuleCondition,
    
    /// Rule effect
    effect: PolicyEffect,
}

pub enum PolicyEffect {
    /// Allow the operation
    Allow,
    
    /// Deny the operation
    Deny,
    
    /// Indeterminate
    Indeterminate,
}

pub enum RuleCombiningAlgorithm {
    /// All rules must match
    AllMatch,
    
    /// At least one rule must match
    AnyMatch,
    
    /// Deny overrides allow
    DenyOverrides,
    
    /// Allow overrides deny
    AllowOverrides,
    
    /// First applicable rule is used
    FirstApplicable,
}
```

### Policy Evaluation

Evaluating authorization policies:

```rust
pub struct PolicyEvaluator {
    /// Condition evaluators
    condition_evaluators: HashMap<String, Box<dyn ConditionEvaluator>>,
    
    /// Expression evaluator
    expression_evaluator: Arc<ExpressionEvaluator>,
    
    /// Function registry
    function_registry: Arc<FunctionRegistry>,
}

impl PolicyEvaluator {
    /// Evaluate a policy
    pub fn evaluate_policy(
        &self,
        policy: &AuthorizationPolicy,
        principal: &Principal,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<PolicyAuthorizationResult, AuthorizationError> {
        // Check if policy is active
        if policy.status != PolicyStatus::Active {
            return Ok(PolicyAuthorizationResult::Indeterminate {
                reason: format!("Policy {} is not active: {:?}", policy.id, policy.status),
            });
        }
        
        // Check if policy target matches
        if !self.matches_target(&policy.target, principal, operation, context)? {
            return Ok(PolicyAuthorizationResult::Indeterminate {
                reason: format!("Policy {} target does not match", policy.id),
            });
        }
        
        // Evaluate rules
        let mut rule_results = Vec::new();
        
        for rule in &policy.rules {
            // Skip rule if its target doesn't match
            if let Some(target) = &rule.target {
                if !self.matches_target(target, principal, operation, context)? {
                    continue;
                }
            }
            
            // Evaluate rule condition
            let condition_result = self.evaluate_condition(&rule.condition, principal, operation, context)?;
            
            if condition_result {
                rule_results.push(rule.effect.clone());
            }
        }
        
        // Apply rule combining algorithm
        let combined_effect = self.apply_combining_algorithm(
            rule_results,
            &policy.rule_combining_algorithm,
        )?;
        
        // Create result based on effect
        match combined_effect {
            PolicyEffect::Allow => Ok(PolicyAuthorizationResult::Allow {
                policy_id: policy.id.clone(),
            }),
            PolicyEffect::Deny => Ok(PolicyAuthorizationResult::Deny {
                policy_id: policy.id.clone(),
                reason: "Denied by policy rules".to_string(),
            }),
            PolicyEffect::Indeterminate => Ok(PolicyAuthorizationResult::Indeterminate {
                reason: format!("Policy {} evaluation was indeterminate", policy.id),
            }),
        }
    }
}
```

## Temporal Authorization

### Temporal Aspects

Authorization with temporal constraints:

```rust
pub struct TemporalAuthorizationService {
    /// Temporal constraint validator
    constraint_validator: Arc<TemporalConstraintValidator>,
    
    /// Historical authorization store
    history_store: Arc<AuthorizationHistoryStore>,
}

impl TemporalAuthorizationService {
    /// Check temporal constraints on an authorization
    pub fn check_temporal_constraints(
        &self,
        principal: &Principal,
        operation: &Operation,
        context: &AuthorizationContext,
        constraints: &[TemporalConstraint],
    ) -> Result<bool, AuthorizationError> {
        for constraint in constraints {
            let valid = self.constraint_validator.validate_constraint(
                constraint,
                principal,
                operation,
                context,
            )?;
            
            if !valid {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Analyze authorization history for anomalies
    pub fn analyze_history(
        &self,
        principal: &Principal,
        operation: &Operation,
        time_window: TimeWindow,
    ) -> Result<HistoryAnalysisResult, AuthorizationError> {
        // Get history for principal and operation type
        let history = self.history_store.get_authorization_history(
            principal,
            operation.operation_type(),
            time_window,
        )?;
        
        // Analyze patterns
        let patterns = self.analyze_patterns(&history)?;
        
        // Detect anomalies
        let anomalies = self.detect_anomalies(principal, operation, &history, &patterns)?;
        
        Ok(HistoryAnalysisResult {
            history_size: history.len(),
            patterns,
            anomalies,
        })
    }
}
```

## Cross-Domain Authorization

### Cross-Domain Model

Managing authorization across domains:

```rust
pub struct CrossDomainAuthorizationService {
    /// Local authorization system
    local_authorization: Arc<AuthorizationSystem>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
    
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    
    /// Trust configuration
    trust_config: Arc<TrustConfiguration>,
}

impl CrossDomainAuthorizationService {
    /// Authorize a cross-domain operation
    pub fn authorize_cross_domain(
        &self,
        principal: &Principal,
        operation: &Operation,
        target_domain: DomainId,
        context: &AuthorizationContext,
    ) -> Result<CrossDomainAuthorizationResult, AuthorizationError> {
        // Check if target domain is known and trusted
        let domain_info = self.domain_registry.get_domain(target_domain)?;
        
        if !domain_info.is_trusted() {
            return Ok(CrossDomainAuthorizationResult::Denied {
                reason: format!("Domain {} is not trusted", target_domain),
            });
        }
        
        // First try local authorization
        let local_result = self.local_authorization.authorize(&AuthorizationRequest::new()
            .with_principal(principal.clone())
            .with_operation(operation.clone())
            .with_context(context.clone())
        )?;
        
        // If locally denied, no need to check remote
        if local_result.is_denied() {
            return Ok(CrossDomainAuthorizationResult::Denied {
                reason: format!("Locally denied: {}", local_result.reason().unwrap_or_default()),
            });
        }
        
        // Create cross-domain authorization request
        let request = CrossDomainAuthorizationRequest {
            principal: principal.clone(),
            operation: operation.clone(),
            source_domain: system.local_domain_id(),
            context: context.to_cross_domain_context()?,
            request_id: RequestId::generate(),
            timestamp: system.current_time(),
        };
        
        // Send request to target domain
        let remote_result = self.messenger.send_authorization_request(target_domain, request)?;
        
        // Process remote result
        match remote_result.status {
            CrossDomainResponseStatus::Allowed => {
                Ok(CrossDomainAuthorizationResult::Allowed {
                    local_decision: local_result,
                    remote_decision: remote_result.clone(),
                })
            },
            CrossDomainResponseStatus::Denied => {
                Ok(CrossDomainAuthorizationResult::Denied {
                    reason: format!("Denied by target domain: {}", 
                                  remote_result.reason.unwrap_or_default()),
                })
            },
            CrossDomainResponseStatus::Error => {
                Ok(CrossDomainAuthorizationResult::Error {
                    error: format!("Error from target domain: {}", 
                                 remote_result.error.unwrap_or_default()),
                })
            },
        }
    }
}
```

## Zero-Knowledge Authorization

### ZK Proofs for Authorization

Using ZK proofs for privacy-preserving authorization:

```rust
pub struct ZkAuthorizationService {
    /// ZK proof verifier
    verifier: Arc<dyn ZkVerifier>,
    
    /// ZK circuit registry
    circuit_registry: Arc<ZkCircuitRegistry>,
    
    /// Public key registry
    key_registry: Arc<PublicKeyRegistry>,
}

impl ZkAuthorizationService {
    /// Verify a ZK authorization proof
    pub fn verify_authorization_proof(
        &self,
        proof: &ZkProof,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<ZkAuthorizationResult, AuthorizationError> {
        // Get the appropriate circuit for this proof type
        let circuit = self.circuit_registry.get_circuit(proof.proof_type())?;
        
        // Extract public inputs from operation and context
        let public_inputs = self.extract_public_inputs(operation, context)?;
        
        // Verify the proof
        let verification_result = self.verifier.verify_proof(
            proof,
            &circuit,
            &public_inputs,
        )?;
        
        if verification_result {
            Ok(ZkAuthorizationResult::Allowed {
                proof_type: proof.proof_type(),
            })
        } else {
            Ok(ZkAuthorizationResult::Denied {
                reason: "ZK proof verification failed".to_string(),
            })
        }
    }
    
    /// Generate public inputs for verification
    fn extract_public_inputs(
        &self,
        operation: &Operation,
        context: &AuthorizationContext,
    ) -> Result<Vec<u8>, AuthorizationError> {
        // Extract operation details
        let operation_type = operation.operation_type().to_string();
        let resource_id = operation.resource_id().to_string();
        
        // Hash the operation data
        let operation_hash = hash_operation(operation)?;
        
        // Extract context details
        let timestamp = context.timestamp()
            .unwrap_or_else(|| system.current_time())
            .to_unix_timestamp();
        
        // Serialize public inputs
        let inputs = PublicInputs {
            operation_type,
            resource_id,
            operation_hash,
            timestamp,
        };
        
        let serialized = serde_json::to_vec(&inputs)?;
        
        Ok(serialized)
    }
}
```

## Usage Examples

### Basic Authorization

```rust
// Create an authorization request
let request = AuthorizationRequest::new()
    .with_principal(Principal::User(user_id))
    .with_operation(Operation::new(
        OperationType::Read,
        document_id,
        OperationParameters::default(),
    ))
    .with_resource(ResourceReference::new(document_id))
    .with_context(context);

// Get authorization decision
let decision = authorization_system.authorize(&request)?;

match decision {
    AuthorizationDecision::Allowed { reason } => {
        println!("Operation authorized: {}", reason.unwrap_or_default());
        // Proceed with operation
    }
    AuthorizationDecision::Denied { reason } => {
        println!("Operation denied: {}", reason.unwrap_or_default());
        // Handle access denial
    }
    AuthorizationDecision::Challenge { challenges } => {
        println!("Additional authentication challenges required");
        for challenge in challenges {
            // Handle each challenge
            println!("Challenge: {:?}", challenge);
        }
    }
}
```

### Policy-Based Authorization

```rust
// Define a policy
let policy = AuthorizationPolicyBuilder::new()
    .with_name("Document access policy")
    .with_description("Controls access to document resources")
    .with_target(PolicyTarget::new()
        .with_resource_type("document")
        .with_action("read")
    )
    .with_rule(PolicyRule::new()
        .with_id("business-hours")
        .with_description("Allow access during business hours")
        .with_condition(RuleCondition::TimeOfDay(
            TimeOfDayCondition::between(
                TimeOfDay::new(9, 0, 0),
                TimeOfDay::new(17, 0, 0),
                vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, 
                     Weekday::Thursday, Weekday::Friday],
            )
        ))
        .with_effect(PolicyEffect::Allow)
    )
    .with_rule(PolicyRule::new()
        .with_id("employee-only")
        .with_description("Only employees can access")
        .with_condition(RuleCondition::AttributeEquals(
            AttributeEqualsCondition::new(
                AttributeSource::Principal,
                "role",
                "employee",
            )
        ))
        .with_effect(PolicyEffect::Allow)
    )
    .with_rule_combining_algorithm(RuleCombiningAlgorithm::AllMatch)
    .with_effect(PolicyEffect::Deny) // Default effect if rules don't match
    .build()?;

// Store the policy
policy_store.store_policy(policy)?;
```

### Cross-Domain Authorization

```rust
// Request authorization for cross-domain operation
let cross_domain_result = cross_domain_authorization_service.authorize_cross_domain(
    &Principal::User(user_id),
    &Operation::new(
        OperationType::Read,
        document_id,
        OperationParameters::default(),
    ),
    partner_domain_id,
    &context,
)?;

match cross_domain_result {
    CrossDomainAuthorizationResult::Allowed { .. } => {
        println!("Cross-domain operation authorized");
        // Proceed with operation
    }
    CrossDomainAuthorizationResult::Denied { reason } => {
        println!("Cross-domain operation denied: {}", reason);
        // Handle access denial
    }
    CrossDomainAuthorizationResult::Error { error } => {
        println!("Error during cross-domain authorization: {}", error);
        // Handle error
    }
}
```

## Implementation Status

The current implementation status of the Security Authorization Model:

- ✅ Core authorization interfaces
- ✅ Capability-based authorization
- ✅ Policy-based authorization
- ⚠️ Context-aware authorization (partially implemented)
- ⚠️ Cross-domain authorization (partially implemented)
- ⚠️ Temporal authorization (partially implemented)
- ❌ Zero-knowledge authorization (not yet implemented)
- ❌ Risk-based authorization (not yet implemented)

## Future Enhancements

Planned future enhancements for the Security Authorization Model:

1. **Risk-Adaptive Authorization**: Authorization decisions based on real-time risk assessment
2. **Intent-Based Authorization**: Authorizing based on operation intent rather than just type
3. **Machine Learning Integration**: Using ML models for anomaly detection and pattern recognition
4. **Attribute-Based Encryption**: Integration with ABE for data-centric access control
5. **Federated Authorization**: Unified authorization across multiple trust domains
6. **Just-In-Time Access**: Temporary elevated privileges with automatic expiration
7. **Continuous Authorization**: Real-time monitoring and re-authorization during long operations
8. **Adaptive Policy Enforcement**: Self-tuning policies based on usage patterns and security events 