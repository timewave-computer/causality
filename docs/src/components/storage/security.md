<!-- Security for storage -->
<!-- Original file: docs/src/storage_security.md -->

# Storage Security in Causality

## Overview

This document describes the comprehensive security mechanisms implemented in the Causality storage system. Storage security is a critical component that ensures the confidentiality, integrity, and availability of all system data, including resources, facts, operations, and transactions. The Causality storage security framework encompasses encryption, access control, audit logging, and policy enforcement.

## Core Security Concepts

### Security Framework

The storage security framework is built around these key components:

```rust
pub struct StorageSecurityManager {
    /// Encryption manager
    encryption_manager: Arc<StorageEncryptionManager>,
    
    /// Access control manager
    access_control_manager: Arc<StorageAccessControlManager>,
    
    /// Audit manager
    audit_manager: Arc<StorageAuditManager>,
    
    /// Policy engine
    policy_engine: Arc<StoragePolicyEngine>,
}
```

## Encryption System

### Encryption Manager

Managing encryption of stored data:

```rust
pub struct StorageEncryptionManager {
    /// Key management service
    key_management: Arc<KeyManagementService>,
    
    /// Crypto providers for different algorithms
    crypto_providers: HashMap<EncryptionAlgorithm, Box<dyn CryptoProvider>>,
    
    /// Default encryption algorithm
    default_algorithm: EncryptionAlgorithm,
    
    /// Configuration
    config: EncryptionConfig,
}

impl StorageEncryptionManager {
    /// Encrypt data for storage
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptedData, EncryptionError> {
        // Get the algorithm to use
        let algorithm = context.algorithm.unwrap_or(self.default_algorithm);
        
        // Get the crypto provider
        let provider = self.crypto_providers.get(&algorithm)
            .ok_or_else(|| EncryptionError::UnsupportedAlgorithm(algorithm))?;
        
        // Get or generate key
        let key_id = match &context.key_id {
            Some(id) => id.clone(),
            None => self.key_management.generate_data_encryption_key(algorithm)?,
        };
        
        // Get the key
        let key = self.key_management.get_encryption_key(key_id.clone())?;
        
        // Encrypt the data
        let (ciphertext, metadata) = provider.encrypt(plaintext, &key, context)?;
        
        // Create encrypted data
        let encrypted_data = EncryptedData {
            ciphertext,
            algorithm,
            key_id,
            metadata,
            created_at: system.current_time(),
        };
        
        // Update audit log
        if self.config.audit_encryption_operations {
            self.record_encryption_operation(&encrypted_data, context)?;
        }
        
        Ok(encrypted_data)
    }
    
    /// Decrypt data from storage
    pub fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        context: &DecryptionContext,
    ) -> Result<Vec<u8>, EncryptionError> {
        // Get the crypto provider
        let provider = self.crypto_providers.get(&encrypted_data.algorithm)
            .ok_or_else(|| EncryptionError::UnsupportedAlgorithm(encrypted_data.algorithm))?;
        
        // Get the key
        let key = self.key_management.get_encryption_key(encrypted_data.key_id.clone())?;
        
        // Decrypt the data
        let plaintext = provider.decrypt(&encrypted_data.ciphertext, &key, &encrypted_data.metadata, context)?;
        
        // Update audit log
        if self.config.audit_decryption_operations {
            self.record_decryption_operation(encrypted_data, context)?;
        }
        
        Ok(plaintext)
    }
}
```

### Key Management

Managing encryption keys:

```rust
pub struct KeyManagementService {
    /// Key vault
    key_vault: Arc<dyn KeyVault>,
    
    /// Key rotation service
    key_rotation: Arc<KeyRotationService>,
    
    /// Key cache
    key_cache: Option<Arc<KeyCache>>,
    
    /// Key usage tracker
    key_usage_tracker: Arc<KeyUsageTracker>,
}

impl KeyManagementService {
    /// Generate a new data encryption key
    pub fn generate_data_encryption_key(
        &self,
        algorithm: EncryptionAlgorithm,
    ) -> Result<KeyId, KeyManagementError> {
        // Generate a new key
        let key = self.key_vault.generate_key(algorithm)?;
        
        // Register key with rotation service
        self.key_rotation.register_key(key.id.clone(), algorithm)?;
        
        // Return key ID
        Ok(key.id)
    }
    
    /// Get an encryption key by ID
    pub fn get_encryption_key(
        &self,
        key_id: KeyId,
    ) -> Result<EncryptionKey, KeyManagementError> {
        // Check cache first if enabled
        if let Some(cache) = &self.key_cache {
            if let Some(key) = cache.get_key(&key_id)? {
                // Update key usage
                self.key_usage_tracker.record_key_usage(key_id.clone())?;
                return Ok(key);
            }
        }
        
        // Get from vault
        let key = self.key_vault.get_key(key_id.clone())?;
        
        // Update cache if enabled
        if let Some(cache) = &self.key_cache {
            cache.cache_key(key_id.clone(), key.clone())?;
        }
        
        // Update key usage
        self.key_usage_tracker.record_key_usage(key_id)?;
        
        Ok(key)
    }
    
    /// Revoke a key
    pub fn revoke_key(
        &self,
        key_id: KeyId,
        reason: RevocationReason,
    ) -> Result<(), KeyManagementError> {
        // Revoke the key
        self.key_vault.revoke_key(key_id.clone(), reason)?;
        
        // Remove from cache if enabled
        if let Some(cache) = &self.key_cache {
            cache.remove_key(&key_id)?;
        }
        
        Ok(())
    }
}
```

## Access Control

### Access Control Manager

Controlling access to storage:

```rust
pub struct StorageAccessControlManager {
    /// Authorization service
    authorization_service: Arc<AuthorizationService>,
    
    /// Identity resolver
    identity_resolver: Arc<IdentityResolver>,
    
    /// Permission evaluator
    permission_evaluator: Arc<PermissionEvaluator>,
    
    /// Access policy provider
    policy_provider: Arc<AccessPolicyProvider>,
    
    /// Access decision cache
    decision_cache: Option<Arc<AccessDecisionCache>>,
}

impl StorageAccessControlManager {
    /// Check access to a storage key
    pub fn check_access(
        &self,
        principal: &Principal,
        storage_key: &StorageKey,
        access_type: AccessType,
        context: &AccessControlContext,
    ) -> Result<AccessDecision, AccessControlError> {
        // Check cache first if enabled
        if let Some(cache) = &self.decision_cache {
            if let Some(decision) = cache.get_decision(principal, storage_key, access_type, context)? {
                return Ok(decision);
            }
        }
        
        // Resolve principal identity
        let identity = self.identity_resolver.resolve_identity(principal)?;
        
        // Get applicable policies
        let policies = self.policy_provider.get_policies_for_key(storage_key)?;
        
        // Evaluate access
        let decision = self.permission_evaluator.evaluate_access(
            &identity,
            storage_key,
            access_type,
            &policies,
            context,
        )?;
        
        // Update cache if enabled
        if let Some(cache) = &self.decision_cache {
            cache.cache_decision(
                principal.clone(),
                storage_key.clone(),
                access_type,
                context.clone(),
                decision.clone(),
            )?;
        }
        
        Ok(decision)
    }
    
    /// Check authorization for a storage operation
    pub fn authorize_operation(
        &self,
        principal: &Principal,
        operation: &StorageOperation,
        context: &AccessControlContext,
    ) -> Result<AuthorizationResult, AccessControlError> {
        // Map operation to access type
        let access_type = match operation.operation_type {
            StorageOperationType::Read => AccessType::Read,
            StorageOperationType::Write => AccessType::Write,
            StorageOperationType::Delete => AccessType::Delete,
            StorageOperationType::List => AccessType::List,
        };
        
        // Check access
        let decision = self.check_access(
            principal,
            &operation.storage_key,
            access_type,
            context,
        )?;
        
        // Convert to authorization result
        let auth_result = match decision {
            AccessDecision::Allow => AuthorizationResult::Authorized,
            AccessDecision::Deny { reason } => AuthorizationResult::Denied(reason),
            AccessDecision::DeferToCapability { capability } => {
                // Check capability
                self.authorization_service.check_capability(
                    principal,
                    &capability,
                    context,
                )?
            }
        };
        
        Ok(auth_result)
    }
}
```

### Access Policies

Defining storage access policies:

```rust
pub struct StorageAccessPolicy {
    /// Policy ID
    id: PolicyId,
    
    /// Policy name
    name: String,
    
    /// Policy effect (allow or deny)
    effect: PolicyEffect,
    
    /// Principal matchers
    principals: Vec<PrincipalMatcher>,
    
    /// Resource matchers
    resources: Vec<ResourceMatcher>,
    
    /// Action matchers
    actions: Vec<ActionMatcher>,
    
    /// Condition expression
    condition: Option<ConditionExpression>,
    
    /// Policy metadata
    metadata: PolicyMetadata,
}

pub struct AccessPolicyProvider {
    /// Indexed policies
    indexed_policies: RwLock<PolicyIndex>,
    
    /// Policy store
    policy_store: Arc<dyn StorageProvider>,
    
    /// Policy engine
    policy_engine: Arc<PolicyEngine>,
}

impl AccessPolicyProvider {
    /// Get policies applicable to a storage key
    pub fn get_policies_for_key(
        &self,
        storage_key: &StorageKey,
    ) -> Result<Vec<StorageAccessPolicy>, PolicyError> {
        // Get read lock on indexed policies
        let index = self.indexed_policies.read().unwrap();
        
        // Match policies for this key
        let matched_policies = index.match_key(storage_key)?;
        
        // Fetch full policies
        let mut policies = Vec::with_capacity(matched_policies.len());
        
        for policy_id in matched_policies {
            let policy = self.get_policy(policy_id)?;
            policies.push(policy);
        }
        
        Ok(policies)
    }
    
    /// Add a policy
    pub fn add_policy(
        &self,
        policy: StorageAccessPolicy,
    ) -> Result<PolicyId, PolicyError> {
        // Validate the policy
        self.policy_engine.validate_policy(&policy)?;
        
        // Store the policy
        let policy_key = StorageKey::new(format!("policies/{}", policy.id));
        let policy_data = serde_json::to_vec(&policy)?;
        
        self.policy_store.store(
            &policy_key,
            &policy_data,
            &StorageOptions::new().with_encryption(true),
        )?;
        
        // Update index
        {
            let mut index = self.indexed_policies.write().unwrap();
            index.add_policy(&policy)?;
        }
        
        Ok(policy.id)
    }
}
```

## Audit System

### Audit Logging

Tracking storage security events:

```rust
pub struct StorageAuditManager {
    /// Audit log store
    audit_store: Arc<dyn StorageProvider>,
    
    /// Audit configuration
    config: AuditConfig,
    
    /// Audit event processor
    event_processor: Arc<AuditEventProcessor>,
}

impl StorageAuditManager {
    /// Log a storage access event
    pub fn log_access_event(
        &self,
        principal: &Principal,
        storage_key: &StorageKey,
        operation: StorageOperationType,
        result: AccessResult,
        context: &AccessControlContext,
    ) -> Result<(), AuditError> {
        // Create audit event
        let event = StorageAuditEvent::Access {
            timestamp: system.current_time(),
            principal: principal.clone(),
            storage_key: storage_key.clone(),
            operation,
            result,
            context: context.clone(),
        };
        
        // Process and store the event
        self.store_audit_event(event)
    }
    
    /// Log an encryption event
    pub fn log_encryption_event(
        &self,
        principal: &Principal,
        key_id: &KeyId,
        context: &EncryptionContext,
    ) -> Result<(), AuditError> {
        // Create audit event
        let event = StorageAuditEvent::Encryption {
            timestamp: system.current_time(),
            principal: principal.clone(),
            key_id: key_id.clone(),
            algorithm: context.algorithm.unwrap_or_default(),
            context: context.clone(),
        };
        
        // Process and store the event
        self.store_audit_event(event)
    }
    
    /// Store an audit event
    fn store_audit_event(
        &self,
        event: StorageAuditEvent,
    ) -> Result<(), AuditError> {
        // Process the event
        let processed_event = self.event_processor.process_event(event)?;
        
        // Serialize the event
        let event_data = serde_json::to_vec(&processed_event)?;
        
        // Create event key
        let event_key = StorageKey::new(format!("audit/{}/{}", 
            processed_event.timestamp().format("%Y/%m/%d/%H/%M/%S"),
            Uuid::new_v4(),
        ));
        
        // Store the event
        self.audit_store.store(
            &event_key,
            &event_data,
            &StorageOptions::new()
                .with_encryption(self.config.encrypt_audit_logs)
                .with_ttl(self.config.audit_log_retention),
        )?;
        
        Ok(())
    }
}
```

## Policy Enforcement

### Policy Engine

Enforcing storage security policies:

```rust
pub struct StoragePolicyEngine {
    /// Policy evaluator
    evaluator: Arc<PolicyEvaluator>,
    
    /// Policy repository
    repository: Arc<PolicyRepository>,
    
    /// Policy validator
    validator: Arc<PolicyValidator>,
    
    /// Policy cache
    cache: Option<Arc<PolicyCache>>,
}

impl StoragePolicyEngine {
    /// Evaluate a policy for a storage operation
    pub fn evaluate_policy(
        &self,
        principal: &Principal,
        storage_key: &StorageKey,
        operation: StorageOperationType,
        context: &PolicyEvaluationContext,
    ) -> Result<PolicyDecision, PolicyError> {
        // Get applicable policies
        let policies = self.repository.get_applicable_policies(
            principal,
            storage_key,
            operation,
        )?;
        
        // Evaluate policies
        self.evaluator.evaluate_policies(
            principal,
            storage_key,
            operation,
            &policies,
            context,
        )
    }
    
    /// Validate a policy
    pub fn validate_policy(
        &self,
        policy: &StorageAccessPolicy,
    ) -> Result<ValidationResult, PolicyError> {
        self.validator.validate_policy(policy)
    }
}
```

## Secure Configuration

### Security Configuration

Configuring storage security:

```rust
pub struct StorageSecurityConfig {
    /// Encryption configuration
    encryption: EncryptionConfig,
    
    /// Access control configuration
    access_control: AccessControlConfig,
    
    /// Audit configuration
    audit: AuditConfig,
    
    /// Policy configuration
    policy: PolicyConfig,
}

pub struct EncryptionConfig {
    /// Default encryption algorithm
    default_algorithm: EncryptionAlgorithm,
    
    /// Key rotation period
    key_rotation_period: Duration,
    
    /// Always encrypt sensitive data
    encrypt_sensitive_data: bool,
    
    /// Encrypt data at rest
    encrypt_data_at_rest: bool,
    
    /// Encrypt data in transit
    encrypt_data_in_transit: bool,
    
    /// Audit encryption operations
    audit_encryption_operations: bool,
    
    /// Audit decryption operations
    audit_decryption_operations: bool,
}

pub struct AccessControlConfig {
    /// Default access model (mandatory/discretionary)
    default_access_model: AccessControlModel,
    
    /// Cache access decisions
    cache_access_decisions: bool,
    
    /// Access decision cache TTL
    access_decision_cache_ttl: Duration,
    
    /// Default deny if no policy matches
    default_deny: bool,
    
    /// Enforce strong authentication
    enforce_strong_authentication: bool,
}
```

## Security Monitoring

### Security Monitoring

Monitoring security events:

```rust
pub struct SecurityMonitoringService {
    /// Security event dispatcher
    event_dispatcher: Arc<SecurityEventDispatcher>,
    
    /// Security alerts manager
    alerts_manager: Arc<SecurityAlertsManager>,
    
    /// Security metrics collector
    metrics_collector: Arc<SecurityMetricsCollector>,
}

impl SecurityMonitoringService {
    /// Monitor security events
    pub fn monitor_security_events(
        &self,
    ) -> Result<(), MonitoringError> {
        // Set up monitoring for different event types
        self.event_dispatcher.register_handler(
            SecurityEventType::UnauthorizedAccess,
            Box::new(|event| {
                self.alerts_manager.create_alert(
                    AlertLevel::High,
                    "Unauthorized access attempt",
                    event,
                )
            }),
        )?;
        
        self.event_dispatcher.register_handler(
            SecurityEventType::KeyCompromise,
            Box::new(|event| {
                self.alerts_manager.create_alert(
                    AlertLevel::Critical,
                    "Potential key compromise",
                    event,
                )
            }),
        )?;
        
        self.event_dispatcher.register_handler(
            SecurityEventType::PolicyViolation,
            Box::new(|event| {
                self.alerts_manager.create_alert(
                    AlertLevel::Medium,
                    "Security policy violation",
                    event,
                )
            }),
        )?;
        
        Ok(())
    }
    
    /// Collect security metrics
    pub fn collect_security_metrics(
        &self,
    ) -> Result<SecurityMetrics, MonitoringError> {
        self.metrics_collector.collect_metrics()
    }
}
```

## Integrations

### Integration with External Systems

Integration with external security systems:

```rust
pub struct ExternalSecurityIntegration {
    /// External key management service
    external_kms: Option<Arc<dyn ExternalKmsProvider>>,
    
    /// External identity provider
    external_identity: Option<Arc<dyn ExternalIdentityProvider>>,
    
    /// External audit system
    external_audit: Option<Arc<dyn ExternalAuditProvider>>,
    
    /// Integration configuration
    config: ExternalIntegrationConfig,
}

impl ExternalSecurityIntegration {
    /// Get encryption key from external KMS
    pub fn get_external_encryption_key(
        &self,
        key_id: &str,
    ) -> Result<EncryptionKey, IntegrationError> {
        if let Some(kms) = &self.external_kms {
            let key = kms.get_key(key_id)?;
            
            // Convert to internal format
            Ok(EncryptionKey {
                id: KeyId::from(key_id),
                material: key.material,
                algorithm: key.algorithm.into(),
                created_at: key.created_at.into(),
                expires_at: key.expires_at.map(|t| t.into()),
                metadata: key.metadata.into(),
            })
        } else {
            Err(IntegrationError::NotConfigured("External KMS not configured"))
        }
    }
    
    /// Verify identity with external provider
    pub fn verify_external_identity(
        &self,
        token: &str,
    ) -> Result<Identity, IntegrationError> {
        if let Some(provider) = &self.external_identity {
            let external_identity = provider.verify_token(token)?;
            
            // Convert to internal format
            Ok(Identity {
                id: external_identity.id,
                attributes: external_identity.attributes,
                provider: IdentityProvider::External(provider.provider_name().to_string()),
                authentication_time: external_identity.authentication_time.into(),
                expiration_time: external_identity.expiration_time.map(|t| t.into()),
            })
        } else {
            Err(IntegrationError::NotConfigured("External identity provider not configured"))
        }
    }
    
    /// Forward audit event to external system
    pub fn forward_audit_event(
        &self,
        event: &StorageAuditEvent,
    ) -> Result<(), IntegrationError> {
        if let Some(audit) = &self.external_audit {
            // Convert to external format
            let external_event = audit.format_event(event)?;
            
            // Send to external system
            audit.send_event(&external_event)?;
            
            Ok(())
        } else {
            Err(IntegrationError::NotConfigured("External audit system not configured"))
        }
    }
}
```

## Usage Examples

### Encrypting Sensitive Data

```rust
// Get the storage security manager
let security = system.storage_security_manager();

// Get encryption manager
let encryption = security.encryption_manager();

// Data to encrypt
let sensitive_data = serde_json::to_vec(&user_credentials)?;

// Create encryption context
let context = EncryptionContext::new()
    .with_algorithm(EncryptionAlgorithm::Aes256Gcm)
    .with_additional_data(resource_id.as_bytes())
    .with_labels(HashMap::from([
        ("data_type".to_string(), "credentials".to_string()),
        ("sensitivity".to_string(), "high".to_string()),
    ]));

// Encrypt the data
let encrypted = encryption.encrypt(&sensitive_data, &context)?;

// Store encrypted data
let key = StorageKey::new(format!("users/{}/credentials", user_id));

storage.store(
    &key,
    &serde_json::to_vec(&encrypted)?,
    &StorageOptions::new(),
)?;

println!("Encrypted with key: {}", encrypted.key_id);
```

### Access Control

```rust
// Get access control manager
let access_control = security.access_control_manager();

// Check access to a storage key
let principal = Principal::from_user_id(user_id);
let storage_key = StorageKey::new(format!("resources/{}", resource_id));

let context = AccessControlContext::new()
    .with_source_ip("192.168.1.100")
    .with_location("us-west")
    .with_time(system.current_time());

let decision = access_control.check_access(
    &principal,
    &storage_key,
    AccessType::Read,
    &context,
)?;

if matches!(decision, AccessDecision::Allow) {
    // Access allowed, retrieve the resource
    let resource_data = storage.retrieve(
        &storage_key,
        &RetrieveOptions::new(),
    )?;
    
    println!("Resource retrieved successfully");
} else {
    println!("Access denied: {:?}", decision);
}
```

### Audit Logging

```rust
// Get audit manager
let audit = security.audit_manager();

// Log a storage access event
audit.log_access_event(
    &Principal::from_user_id(user_id),
    &StorageKey::new(format!("documents/{}", document_id)),
    StorageOperationType::Read,
    AccessResult::Success,
    &AccessControlContext::new()
        .with_session_id(session_id)
        .with_request_id(request_id),
)?;

// Query audit logs
let audit_query = AuditQuery::new()
    .with_time_range(
        system.current_time() - Duration::from_hours(24),
        system.current_time(),
    )
    .with_principal(Principal::from_user_id(user_id))
    .with_operation_type(StorageOperationType::Write)
    .with_limit(100);

let audit_events = audit.query_events(audit_query)?;

println!("Found {} audit events", audit_events.len());
```

### Policy Management

```rust
// Get policy engine
let policy_engine = security.policy_engine();

// Define a new policy
let policy = StorageAccessPolicy::new(
    "resource-read-policy",
    PolicyEffect::Allow,
)
.with_principal(PrincipalMatcher::Group("document-readers"))
.with_resource(ResourceMatcher::Prefix("documents/"))
.with_action(ActionMatcher::Exact(StorageOperationType::Read))
.with_condition(ConditionExpression::from_str(
    "request.time > '08:00:00' && request.time < '18:00:00'"
)?);

// Add the policy
let policy_id = policy_engine.add_policy(policy)?;

println!("Added policy with ID: {}", policy_id);

// Evaluate policy for a request
let decision = policy_engine.evaluate_policy(
    &Principal::from_user_id(user_id),
    &StorageKey::new("documents/report.pdf"),
    StorageOperationType::Read,
    &PolicyEvaluationContext::new()
        .with_time(system.current_time())
        .with_source_ip("192.168.1.100"),
)?;

println!("Policy decision: {:?}", decision);
```

## Implementation Status

The current implementation status of Storage Security:

- ✅ Core security interfaces
- ✅ Encryption framework
- ⚠️ Key management (partially implemented)
- ⚠️ Access control system (partially implemented)
- ⚠️ Audit logging (partially implemented)
- ❌ Policy engine (not yet implemented)
- ❌ External integrations (not yet implemented)

## Future Enhancements

Planned future enhancements for Storage Security:

1. **Hardware Security Module Integration**: Support for hardware security modules for key protection
2. **Cryptographic Agility**: Framework for easily updating cryptographic algorithms
3. **Attribute-Based Encryption**: Support for attribute-based encryption for fine-grained access control
4. **Homomorphic Encryption**: Support for computations on encrypted data
5. **Advanced Threat Detection**: Enhanced security monitoring and threat detection
6. **Secure Multi-party Computation**: Protocols for secure computation across multiple parties
7. **Quantum-Resistant Cryptography**: Integration of post-quantum cryptographic algorithms
8. **Security Compliance Automation**: Automated compliance monitoring and reporting 