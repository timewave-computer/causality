<!-- Model for security authentication -->
<!-- Original file: docs/src/security_authentication_model.md -->

# Security Authentication Model in Causality

## Overview

This document describes the authentication model within the Causality architecture. The authentication model provides a unified approach to verifying the identity of principals across a distributed system with temporal consistency guarantees. It supports multiple authentication methods, cross-domain identity verification, and integration with zero-knowledge proofs to enable privacy-preserving authentication.

## Core Components

### Authentication System

The central authentication system:

```rust
pub struct AuthenticationSystem {
    /// Authentication providers
    providers: HashMap<String, Box<dyn AuthenticationProvider>>,
    
    /// Identity providers
    identity_providers: HashMap<String, Box<dyn IdentityProvider>>,
    
    /// Credential store
    credential_store: Arc<CredentialStore>,
    
    /// Authentication policy service
    policy_service: Arc<AuthenticationPolicyService>,
    
    /// Authentication cache
    cache: Arc<AuthenticationCache>,
    
    /// Cross-domain authenticator
    cross_domain: Arc<CrossDomainAuthenticator>,
    
    /// ZK authentication service
    zk_auth: Option<Arc<ZkAuthenticationService>>,
}
```

### Authentication Context

The context for authentication requests:

```rust
pub struct AuthenticationContext {
    /// Authentication request time
    timestamp: Timestamp,
    
    /// Authentication request origin
    origin: Option<RequestOrigin>,
    
    /// Domain information
    domain: Option<DomainId>,
    
    /// Client device information
    device: Option<DeviceInfo>,
    
    /// Network information
    network: Option<NetworkInfo>,
    
    /// Previous authentication context (for step-up)
    previous_context: Option<Box<AuthenticationContext>>,
    
    /// Authentication factors already used
    used_factors: Vec<AuthenticationFactor>,
    
    /// Risk score
    risk_score: Option<f64>,
    
    /// Additional context attributes
    attributes: HashMap<String, String>,
}
```

## Authentication Methods

### Identity Providers

Handling various identity sources:

```rust
pub trait IdentityProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> IdentityProviderType;
    
    /// Get the provider ID
    fn provider_id(&self) -> &str;
    
    /// Verify identity
    fn verify_identity(
        &self,
        credentials: &Credentials,
        context: &AuthenticationContext,
    ) -> Result<IdentityVerificationResult, AuthenticationError>;
    
    /// Get identity attributes
    fn get_identity_attributes(
        &self,
        identity_id: &IdentityId,
    ) -> Result<HashMap<String, String>, AuthenticationError>;
    
    /// Check if identity exists
    fn identity_exists(
        &self,
        identity_id: &IdentityId,
    ) -> Result<bool, AuthenticationError>;
}
```

### Authentication Providers

Supporting various authentication methods:

```rust
pub trait AuthenticationProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> AuthenticationProviderType;
    
    /// Get the provider ID
    fn provider_id(&self) -> &str;
    
    /// Get supported factors
    fn supported_factors(&self) -> Vec<AuthenticationFactor>;
    
    /// Authenticate a principal
    fn authenticate(
        &self,
        credentials: &Credentials,
        context: &AuthenticationContext,
    ) -> Result<AuthenticationResult, AuthenticationError>;
    
    /// Initiate multi-factor authentication
    fn initiate_mfa(
        &self,
        principal: &Principal,
        factor: AuthenticationFactor,
        context: &AuthenticationContext,
    ) -> Result<MfaChallenge, AuthenticationError>;
    
    /// Verify MFA response
    fn verify_mfa(
        &self,
        challenge_id: &str,
        response: &MfaResponse,
        context: &AuthenticationContext,
    ) -> Result<MfaVerificationResult, AuthenticationError>;
}
```

## Authentication Workflows

### Single-Factor Authentication

Basic authentication process:

```rust
impl AuthenticationSystem {
    /// Authenticate a principal with single-factor authentication
    pub fn authenticate(
        &self,
        credentials: &Credentials,
        context: Option<AuthenticationContext>,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        // Create auth context if none provided
        let context = context.unwrap_or_else(|| AuthenticationContext::new(system.current_time()));
        
        // Check cache for recent authentication
        if let Some(cached_result) = self.check_authentication_cache(credentials, &context)? {
            return Ok(cached_result);
        }
        
        // Determine provider based on credential type
        let provider = self.get_provider_for_credentials(credentials)?;
        
        // Authenticate using the provider
        let auth_result = provider.authenticate(credentials, &context)?;
        
        // Apply authentication policies
        let policy_result = self.policy_service.apply_authentication_policies(
            &auth_result,
            &context,
        )?;
        
        if let Some(policy_result) = policy_result {
            // Policy requires additional steps or denies authentication
            return Ok(policy_result);
        }
        
        // Cache successful authentication
        if auth_result.is_successful() {
            self.cache_authentication_result(&auth_result, credentials, &context)?;
        }
        
        // Return result
        Ok(auth_result)
    }
}
```

### Multi-Factor Authentication

Supporting multiple authentication factors:

```rust
impl AuthenticationSystem {
    /// Initiate multi-factor authentication
    pub fn initiate_mfa(
        &self,
        principal: &Principal,
        factor: AuthenticationFactor,
        context: Option<AuthenticationContext>,
    ) -> Result<MfaChallenge, AuthenticationError> {
        // Create auth context if none provided
        let context = context.unwrap_or_else(|| AuthenticationContext::new(system.current_time()));
        
        // Get provider that supports the requested factor
        let provider = self.get_provider_for_factor(factor)?;
        
        // Initiate MFA challenge
        let challenge = provider.initiate_mfa(principal, factor, &context)?;
        
        // Store challenge for verification
        self.store_mfa_challenge(&challenge, principal, &context)?;
        
        Ok(challenge)
    }
    
    /// Verify MFA response
    pub fn verify_mfa(
        &self,
        challenge_id: &str,
        response: &MfaResponse,
        context: Option<AuthenticationContext>,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        // Create auth context if none provided
        let context = context.unwrap_or_else(|| AuthenticationContext::new(system.current_time()));
        
        // Retrieve challenge details
        let challenge = self.get_mfa_challenge(challenge_id)?;
        
        // Check if challenge is expired
        if challenge.is_expired(system.current_time()) {
            return Ok(AuthenticationResult::Failed {
                reason: "MFA challenge expired".to_string(),
            });
        }
        
        // Get provider that issued the challenge
        let provider = self.get_provider_by_id(&challenge.provider_id)?;
        
        // Verify MFA response
        let verification_result = provider.verify_mfa(challenge_id, response, &context)?;
        
        // Convert verification result to authentication result
        let auth_result = match verification_result {
            MfaVerificationResult::Success { principal } => {
                // Update context with used factor
                let mut updated_context = context.clone();
                updated_context.add_used_factor(challenge.factor.clone());
                
                // Check if additional factors are required
                if let Some(next_required) = self.policy_service.get_next_required_factor(
                    &principal,
                    &updated_context,
                )? {
                    // Another factor is required
                    let next_challenge = self.initiate_mfa(
                        &principal,
                        next_required,
                        Some(updated_context),
                    )?;
                    
                    AuthenticationResult::AdditionalFactorRequired {
                        principal,
                        challenge: next_challenge,
                    }
                } else {
                    // All required factors satisfied
                    AuthenticationResult::Success {
                        principal,
                        session_id: Some(self.generate_session_id()?),
                        expiration: Some(system.current_time() + self.session_ttl),
                    }
                }
            },
            MfaVerificationResult::Failed { reason } => {
                AuthenticationResult::Failed { reason }
            },
        };
        
        Ok(auth_result)
    }
}
```

## Credential Management

### Credential Storage

Secure credential management:

```rust
pub struct CredentialStore {
    /// Storage backend
    storage: Arc<dyn SecureStorage>,
    
    /// Encryption provider
    encryption: Arc<dyn EncryptionProvider>,
    
    /// Hashing configuration
    hash_config: PasswordHashConfig,
}

impl CredentialStore {
    /// Store credentials for a principal
    pub fn store_credentials(
        &self,
        principal: &Principal,
        credentials: &Credentials,
    ) -> Result<(), CredentialError> {
        match credentials {
            Credentials::Password { username, password } => {
                // Hash password with appropriate algorithm and parameters
                let password_hash = self.hash_password(password)?;
                
                // Encrypt hash before storage
                let encrypted = self.encryption.encrypt(
                    password_hash.as_bytes(),
                    Some(&principal.to_string()),
                )?;
                
                // Store encrypted hash
                self.storage.store(
                    &format!("password:{}:{}", principal, username),
                    &encrypted,
                )?;
                
                Ok(())
            },
            Credentials::PrivateKey { public_key, .. } => {
                // Store public key
                self.storage.store(
                    &format!("pubkey:{}", principal),
                    public_key.as_bytes(),
                )?;
                
                Ok(())
            },
            // Handle other credential types
            _ => Err(CredentialError::UnsupportedCredentialType),
        }
    }
    
    /// Verify credentials for a principal
    pub fn verify_credentials(
        &self,
        principal: &Principal,
        credentials: &Credentials,
    ) -> Result<bool, CredentialError> {
        match credentials {
            Credentials::Password { username, password } => {
                // Retrieve stored hash
                let encrypted_hash = match self.storage.retrieve(
                    &format!("password:{}:{}", principal, username),
                )? {
                    Some(data) => data,
                    None => return Ok(false),
                };
                
                // Decrypt hash
                let hash_bytes = self.encryption.decrypt(
                    &encrypted_hash,
                    Some(&principal.to_string()),
                )?;
                
                let stored_hash = String::from_utf8(hash_bytes)
                    .map_err(|_| CredentialError::InvalidData)?;
                
                // Verify password against hash
                self.verify_password(password, &stored_hash)
            },
            Credentials::PrivateKey { public_key, signature, message } => {
                // Retrieve stored public key
                let stored_public_key = match self.storage.retrieve(
                    &format!("pubkey:{}", principal),
                )? {
                    Some(data) => data,
                    None => return Ok(false),
                };
                
                // Verify signature
                self.verify_signature(
                    &stored_public_key,
                    signature,
                    message,
                )
            },
            // Handle other credential types
            _ => Err(CredentialError::UnsupportedCredentialType),
        }
    }
}
```

## Cross-Domain Authentication

### Cross-Domain Identity

Managing authentication across domains:

```rust
pub struct CrossDomainAuthenticator {
    /// Local authentication system
    local_auth: Arc<AuthenticationSystem>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
    
    /// Trust configuration
    trust_config: Arc<TrustConfiguration>,
    
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    
    /// Identity mapping store
    identity_mapping: Arc<IdentityMappingStore>,
    
    /// Token validator
    token_validator: Arc<TokenValidator>,
}

impl CrossDomainAuthenticator {
    /// Authenticate across domains
    pub fn authenticate_cross_domain(
        &self,
        credentials: &Credentials,
        target_domain: DomainId,
        context: &AuthenticationContext,
    ) -> Result<CrossDomainAuthenticationResult, AuthenticationError> {
        // Check if domain is trusted
        if !self.trust_config.is_domain_trusted(target_domain)? {
            return Ok(CrossDomainAuthenticationResult::Failed {
                reason: format!("Domain {} is not trusted", target_domain),
            });
        }
        
        // First authenticate locally
        let local_result = self.local_auth.authenticate(credentials, Some(context.clone()))?;
        
        if !local_result.is_successful() {
            return Ok(CrossDomainAuthenticationResult::Failed {
                reason: "Local authentication failed".to_string(),
            });
        }
        
        // Get principal from successful authentication
        let principal = match &local_result {
            AuthenticationResult::Success { principal, .. } => principal,
            _ => return Err(AuthenticationError::InternalError(
                "Expected successful authentication".to_string(),
            )),
        };
        
        // Create cross-domain authentication token
        let token = self.create_cross_domain_token(principal, target_domain, context)?;
        
        // Send authentication request to target domain
        let request = CrossDomainAuthenticationRequest {
            token,
            source_domain: system.local_domain_id(),
            context: context.to_cross_domain_context()?,
            request_id: RequestId::generate(),
            timestamp: system.current_time(),
        };
        
        let response = self.messenger.send_authentication_request(target_domain, request)?;
        
        // Process response
        match response.status {
            CrossDomainResponseStatus::Success => {
                Ok(CrossDomainAuthenticationResult::Success {
                    principal: principal.clone(),
                    remote_principal: response.principal,
                    token: response.token,
                    expiration: response.expiration,
                })
            },
            CrossDomainResponseStatus::Failed => {
                Ok(CrossDomainAuthenticationResult::Failed {
                    reason: response.reason.unwrap_or_else(|| 
                        "Remote authentication failed".to_string()),
                })
            },
            CrossDomainResponseStatus::Error => {
                Ok(CrossDomainAuthenticationResult::Error {
                    error: response.error.unwrap_or_else(||
                        "Unknown remote error".to_string()),
                })
            },
        }
    }
}
```

## Zero-Knowledge Authentication

### ZK Authentication

Privacy-preserving authentication:

```rust
pub struct ZkAuthenticationService {
    /// ZK prover
    prover: Arc<dyn ZkProver>,
    
    /// ZK verifier
    verifier: Arc<dyn ZkVerifier>,
    
    /// Circuit registry
    circuit_registry: Arc<ZkCircuitRegistry>,
    
    /// Credential store
    credential_store: Arc<CredentialStore>,
}

impl ZkAuthenticationService {
    /// Generate a ZK authentication proof
    pub fn generate_auth_proof(
        &self,
        principal: &Principal,
        credentials: &Credentials,
        circuit_id: &str,
    ) -> Result<ZkProof, AuthenticationError> {
        // Get the appropriate circuit
        let circuit = self.circuit_registry.get_circuit(circuit_id)?;
        
        // Build private inputs from credentials
        let private_inputs = self.build_private_inputs(credentials)?;
        
        // Build public inputs (typically includes a commitment to the identity)
        let public_inputs = self.build_public_inputs(principal)?;
        
        // Generate witness
        let witness = self.prover.generate_witness(
            &circuit,
            &private_inputs,
        )?;
        
        // Generate proof
        let proof = self.prover.generate_proof(
            &circuit,
            &witness,
        )?;
        
        Ok(proof)
    }
    
    /// Verify a ZK authentication proof
    pub fn verify_auth_proof(
        &self,
        proof: &ZkProof,
        principal: &Principal,
        circuit_id: &str,
    ) -> Result<bool, AuthenticationError> {
        // Get the appropriate circuit
        let circuit = self.circuit_registry.get_circuit(circuit_id)?;
        
        // Build public inputs for verification
        let public_inputs = self.build_public_inputs(principal)?;
        
        // Verify the proof
        let verification_result = self.verifier.verify_proof(
            proof,
            &circuit,
            &public_inputs,
        )?;
        
        Ok(verification_result)
    }
}
```

## Implementation Status

The current implementation status of the Security Authentication Model:

- ✅ Core authentication interfaces
- ✅ Password-based authentication
- ✅ Public key authentication
- ✅ Multi-factor authentication framework
- ⚠️ Cross-domain authentication (partially implemented)
- ⚠️ Credential management (partially implemented)
- ⚠️ ZK authentication (partially implemented)
- ❌ Biometric authentication (not yet implemented)
- ❌ Device-based authentication (not yet implemented)

## Future Enhancements

Planned future enhancements for the Security Authentication Model:

1. **Biometric Authentication**: Support for biometric authentication methods
2. **Hardware Security Module Integration**: Integration with HSMs for credential management
3. **Decentralized Identifiers (DIDs)**: Support for W3C DID standard
4. **Verifiable Credentials**: Support for W3C Verifiable Credentials standard
5. **Behavioral Authentication**: Authentication based on behavioral patterns
6. **Continuous Authentication**: Real-time authentication assessment during sessions
7. **Passwordless Authentication**: Expanded support for passwordless authentication methods
8. **Self-Sovereign Identity**: Integration with SSI frameworks for user-controlled identity 