# Implementing Agent-Based Resources

*This guide provides implementation details for the agent-based resource system as defined in [ADR-032](../../../spec/adr-032-role-based-resource-system.md) and the [Agent-Based Resources](../../architecture/core/agent-based-resources.md) architecture document.*

*Last updated: 2023-08-15*

## Introduction

This guide provides practical steps and code examples for implementing and working with the Agent-Based Resource System in Causality. You'll learn how to:

1. Create and manage User, Operator, and Committee agent resources
2. Implement agent-based resource accessors
3. Authenticate and authorize operations based on agent types
4. Send messages between agents
5. Integrate agents with the capability system

## Prerequisites

Before implementing agent-based resources, ensure you understand:
- Basic Resource System concepts
- Content addressing
- Capability-based security

## Implementing Agent Resources

### 1. Implementing the User Agent

```rust
use causality_core::agent::{Agent, AgentType};
use causality_core::resource::ResourceId;
use causality_core::content::{ContentAddressed, ContentHash};
use causality_crypto::signature::{PublicKey, Signature};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID (content-addressed)
    id: ResourceId,
    /// Public keys
    public_keys: Vec<PublicKey>,
    /// User profile
    profile: UserProfile,
    /// Metadata
    metadata: HashMap<String, serde_json::Value>,
    /// Content hash
    content_hash: ContentHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Display name
    display_name: String,
    /// Email
    email: Option<String>,
    /// Avatar URL
    avatar_url: Option<String>,
}

impl User {
    /// Create a new user
    pub fn new(
        domain: DomainId, 
        user_id: &str, 
        public_keys: Vec<PublicKey>, 
        profile: UserProfile
    ) -> Result<Self, ResourceError> {
        // Create the resource ID
        let id = ResourceId::new("user", domain, user_id)?;
        
        // Create the user
        let mut user = Self {
            id,
            public_keys,
            profile,
            metadata: HashMap::new(),
            content_hash: ContentHash::default(),
        };
        
        // Calculate the content hash
        user.content_hash = user.calculate_content_hash()?;
        
        Ok(user)
    }
    
    /// Add metadata to the user
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Result<Self, ResourceError> {
        self.metadata.insert(key.to_string(), value);
        
        // Recalculate the content hash
        self.content_hash = self.calculate_content_hash()?;
        
        Ok(self)
    }
}

impl Agent for User {
    fn agent_type(&self) -> AgentType {
        AgentType::User
    }
    
    fn resource_id(&self) -> &ResourceId {
        &self.id
    }
    
    fn public_keys(&self) -> &[PublicKey] {
        &self.public_keys
    }
    
    fn verify_signature(&self, message: &[u8], signature: &Signature) -> Result<bool, CryptoError> {
        // Try each public key
        for key in &self.public_keys {
            if key.verify(message, signature)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ContentAddressed for User {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        hasher.update("User");
        hasher.update(self.id.as_bytes());
        
        for key in &self.public_keys {
            hasher.update(key.as_bytes());
        }
        
        // Hash profile and metadata
        let profile_bytes = serde_json::to_vec(&self.profile)
            .map_err(|e| ContentHashError::SerializationError(e.to_string()))?;
        hasher.update(&profile_bytes);
        
        let metadata_bytes = serde_json::to_vec(&self.metadata)
            .map_err(|e| ContentHashError::SerializationError(e.to_string()))?;
        hasher.update(&metadata_bytes);
        
        Ok(hasher.finalize())
    }
    
    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    
    fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = hash;
        self
    }
}
```

### 2. Implementing the Committee Agent

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Committee {
    /// Committee ID (content-addressed)
    id: ResourceId,
    /// Associated domain
    domain: DomainId,
    /// Committee members
    members: Vec<ValidatorInfo>,
    /// Required signatures threshold
    threshold: u32,
    /// Epoch
    epoch: u64,
    /// Metadata
    metadata: HashMap<String, serde_json::Value>,
    /// Content hash
    content_hash: ContentHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator ID
    id: ResourceId,
    /// Public key
    public_key: PublicKey,
    /// Voting power
    voting_power: u64,
}

impl Committee {
    /// Create a new committee
    pub fn new(
        domain_id: DomainId,
        committee_id: &str,
        members: Vec<ValidatorInfo>,
        threshold: u32,
        epoch: u64
    ) -> Result<Self, ResourceError> {
        // Create the resource ID
        let id = ResourceId::new("committee", DomainId::root(), committee_id)?;
        
        // Create the committee
        let mut committee = Self {
            id,
            domain: domain_id,
            members,
            threshold,
            epoch,
            metadata: HashMap::new(),
            content_hash: ContentHash::default(),
        };
        
        // Calculate the content hash
        committee.content_hash = committee.calculate_content_hash()?;
        
        Ok(committee)
    }
}

impl Agent for Committee {
    fn agent_type(&self) -> AgentType {
        AgentType::Committee
    }
    
    // ... implement the rest of the trait methods ...
}

impl ContentAddressed for Committee {
    // ... implement ContentAddressed methods ...
}
```

## Implementing Agent-Based Resource Accessors

### 1. User Resource Accessor

```rust
use async_trait::async_trait;
use causality_core::resource::{ResourceAccessor, ResourceId, ResourceQuery, ResourceError};
use causality_core::agent::{User, Credentials, Message};
use causality_core::capability::Capability;

#[async_trait]
pub trait UserAccessor: ResourceAccessor<Resource = User> {
    /// Authenticate a user
    async fn authenticate(&self, id: &ResourceId, credentials: &Credentials) 
        -> Result<bool, ResourceError>;
    
    /// Send a message to a user
    async fn send_message(&self, from: &ResourceId, to: &ResourceId, message: Message)
        -> Result<(), ResourceError>;
    
    /// Get a user's capabilities
    async fn get_capabilities(&self, id: &ResourceId) -> Result<Vec<Capability>, ResourceError>;
}

/// In-memory user accessor implementation
pub struct InMemoryUserAccessor {
    /// Domain
    domain: DomainId,
    /// Users
    users: RwLock<HashMap<String, User>>,
    /// User messages
    messages: RwLock<HashMap<String, Vec<Message>>>,
    /// User capabilities
    capabilities: RwLock<HashMap<String, Vec<Capability>>>,
}

impl InMemoryUserAccessor {
    /// Create a new in-memory user accessor
    pub fn new(domain: DomainId) -> Self {
        Self {
            domain,
            users: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ResourceAccessor for InMemoryUserAccessor {
    type Resource = User;
    
    async fn get(&self, id: &ResourceId) -> Result<Option<Self::Resource>, ResourceError> {
        // Verify domain and type
        if id.domain() != &self.domain {
            return Err(ResourceError::DomainMismatch);
        }
        
        if id.resource_type() != "user" {
            return Err(ResourceError::TypeMismatch);
        }
        
        // Get the user
        let users = self.users.read().map_err(|_| ResourceError::LockError)?;
        Ok(users.get(id.id()).cloned())
    }
    
    // ... implement the rest of the ResourceAccessor methods ...
}

#[async_trait]
impl UserAccessor for InMemoryUserAccessor {
    async fn authenticate(&self, id: &ResourceId, credentials: &Credentials) 
        -> Result<bool, ResourceError> {
        // Get the user
        let user = match self.get(id).await? {
            Some(user) => user,
            None => return Err(ResourceError::NotFound),
        };
        
        // Verify credentials
        match credentials {
            Credentials::Signature { message, signature } => {
                user.verify_signature(message, signature)
                    .map_err(|e| ResourceError::CryptoError(e.to_string()))
            },
            // ... other credential types ...
        }
    }
    
    async fn send_message(&self, from: &ResourceId, to: &ResourceId, message: Message)
        -> Result<(), ResourceError> {
        // Verify the recipient exists
        if self.get(to).await?.is_none() {
            return Err(ResourceError::NotFound);
        }
        
        // Add the message
        let mut messages = self.messages.write().map_err(|_| ResourceError::LockError)?;
        messages.entry(to.id().to_string())
            .or_insert_with(Vec::new)
            .push(message);
        
        Ok(())
    }
    
    async fn get_capabilities(&self, id: &ResourceId) -> Result<Vec<Capability>, ResourceError> {
        // Verify the user exists
        if self.get(id).await?.is_none() {
            return Err(ResourceError::NotFound);
        }
        
        // Get the capabilities
        let capabilities = self.capabilities.read().map_err(|_| ResourceError::LockError)?;
        Ok(capabilities.get(id.id()).cloned().unwrap_or_default())
    }
}
```

## Using Agent-Based Resources

### 1. Creating and Authenticating a User Agent

```rust
use causality_core::agent::{User, UserProfile, Credentials};
use causality_core::resource::{ResourceId, ResourceManager};
use causality_crypto::signature::{KeyPair, Signature};

async fn create_and_authenticate_user(
    resource_manager: &ResourceManager,
    domain_id: DomainId
) -> Result<(), Box<dyn Error>> {
    // Generate a key pair
    let key_pair = KeyPair::generate()?;
    let public_key = key_pair.public_key();
    
    // Create a user profile
    let profile = UserProfile {
        display_name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
        avatar_url: None,
    };
    
    // Create the user
    let user = User::new(
        domain_id,
        "alice",
        vec![public_key.clone()],
        profile
    )?;
    
    // Register the user
    let user_id = resource_manager.register(user).await?;
    
    // Authenticate the user with a signature
    let message = b"authenticate me";
    let signature = key_pair.sign(message)?;
    
    // Get the user accessor
    let user_accessor = resource_manager.get_accessor::<dyn UserAccessor>(&domain_id)?;
    
    // Authenticate
    let credentials = Credentials::Signature {
        message: message.to_vec(),
        signature,
    };
    
    let is_authenticated = user_accessor.authenticate(&user_id, &credentials).await?;
    println!("User authenticated: {}", is_authenticated);
    
    Ok(())
}
```

### 2. Sending Messages Between Agents

```rust
async fn send_message_between_agents(
    resource_manager: &ResourceManager,
    from_id: &ResourceId,
    to_id: &ResourceId,
    content: &str
) -> Result<(), Box<dyn Error>> {
    // Create the message
    let message = Message {
        id: Uuid::new_v4().to_string(),
        from: from_id.clone(),
        to: to_id.clone(),
        content: content.to_string(),
        timestamp: Utc::now(),
        content_hash: ContentHash::default(),
    };
    
    // Calculate the content hash
    let message = message.with_content_hash(message.calculate_content_hash()?);
    
    // Get the appropriate accessor based on the recipient's agent type
    match to_id.resource_type() {
        "user" => {
            let user_accessor = resource_manager.get_accessor::<dyn UserAccessor>(to_id.domain())?;
            user_accessor.send_message(from_id, to_id, message).await?;
        },
        "operator" => {
            let operator_accessor = resource_manager.get_accessor::<dyn OperatorAccessor>(to_id.domain())?;
            operator_accessor.send_message(from_id, to_id, message).await?;
        },
        "committee" => {
            let committee_accessor = resource_manager.get_accessor::<dyn CommitteeAccessor>(to_id.domain())?;
            committee_accessor.submit_observation(from_id, to_id, message.into()).await?;
        },
        _ => return Err(Box::new(ResourceError::TypeMismatch)),
    }
    
    Ok(())
}
```

### 3. Managing Agent Capabilities

```rust
use causality_core::capability::{Capability, CapabilityType, CapabilityStore};
use causality_core::resource::ResourceId;

async fn grant_capability_to_agent(
    capability_store: &dyn CapabilityStore,
    grantor: &ResourceId,
    grantee: &ResourceId,
    target: &ResourceId,
    capability_type: CapabilityType
) -> Result<(), Box<dyn Error>> {
    // Create the capability
    let capability = Capability::new(target.clone(), capability_type)?;
    
    // Grant the capability
    capability_store.grant(grantor, grantee, capability).await?;
    
    Ok(())
}

async fn verify_agent_capability(
    capability_store: &dyn CapabilityStore,
    agent_id: &ResourceId,
    target: &ResourceId,
    capability_type: CapabilityType
) -> Result<bool, Box<dyn Error>> {
    // Verify the capability
    let has_capability = capability_store.verify(agent_id, target, capability_type).await?;
    
    Ok(has_capability)
}
```

## Best Practices

1. **Security**: Always verify signatures when authenticating agents
2. **Content Addressing**: Ensure all agent resources have correct content hashes
3. **Capability Management**: Use the least privilege principle when granting capabilities
4. **Resource IDs**: Follow the convention of `resource_type:domain:id` for resource IDs
5. **Metadata**: Use metadata for extensibility rather than modifying the core resource structures

## Common Pitfalls

1. **Domain Mismatch**: Ensure you're using the correct domain when creating and accessing agents
2. **Signature Verification**: Verify signatures against all public keys associated with an agent
3. **Content Hash Recalculation**: Remember to recalculate content hashes after any modifications
4. **Resource Type Verification**: Always verify the resource type before accessing a resource

## Advanced Patterns

### 1. Agent Delegation

```rust
async fn delegate_agent_actions(
    from_agent: &ResourceId,
    to_agent: &ResourceId,
    capability_manager: &CapabilityManager
) -> Result<(), Box<dyn Error>> {
    // Create a delegated capability
    let capability = Capability::new(
        ResourceId::new("resource", from_agent.domain(), "target")?,
        CapabilityType::Write
    )?;
    
    // Add constraints to the capability
    let capability = capability
        .with_constraint(CapabilityConstraint::Time(TimeConstraint::ExpiresAt(
            Utc::now() + chrono::Duration::days(7)
        )))?
        .with_constraint(CapabilityConstraint::Field(FieldConstraint::new(
            "amount", FieldConstraintOp::LessThan, serde_json::json!(1000)
        )))?;
    
    // Delegate the capability
    capability_manager.delegate(from_agent, to_agent, capability).await?;
    
    Ok(())
}
```

### 2. Multi-Signature Agents

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigUser {
    // ... standard User fields ...
    
    /// Threshold of required signatures
    threshold: u32,
}

impl Agent for MultiSigUser {
    // ... standard Agent implementation ...
    
    fn verify_signature(&self, message: &[u8], signatures: &[Signature]) -> Result<bool, CryptoError> {
        let mut valid_signatures = 0;
        
        // Count valid signatures
        for signature in signatures {
            for key in &self.public_keys {
                if key.verify(message, signature)? {
                    valid_signatures += 1;
                    break;
                }
            }
        }
        
        // Check if we have enough valid signatures
        Ok(valid_signatures >= self.threshold)
    }
}
```

## Integration with Other Systems

### 1. Integration with the Effect System

```rust
use causality_core::effect::{Effect, EffectExecutor, EffectContext};
use causality_core::agent::{Agent, UserAccessor};

/// Effect for user operations
pub struct UserEffect {
    /// The user ID
    user_id: ResourceId,
    /// The operation to perform
    operation: UserOperation,
}

/// User operations
pub enum UserOperation {
    /// Update the user profile
    UpdateProfile(UserProfile),
    /// Send a message
    SendMessage { recipient: ResourceId, content: String },
    /// Other operations...
}

#[async_trait]
impl Effect for UserEffect {
    async fn execute(&self, context: &dyn EffectContext) -> Result<EffectResult, EffectError> {
        // Get the user accessor
        let user_accessor = context.resource_manager()
            .get_accessor::<dyn UserAccessor>(self.user_id.domain())?;
        
        // Execute the operation
        match &self.operation {
            UserOperation::UpdateProfile(profile) => {
                // Get the user
                let mut user = user_accessor.get(&self.user_id).await?
                    .ok_or(EffectError::ResourceNotFound)?;
                
                // Update the profile
                user.set_profile(profile.clone())?;
                
                // Update the user
                user_accessor.update(&self.user_id, user).await?;
                
                Ok(EffectResult::Success)
            },
            UserOperation::SendMessage { recipient, content } => {
                // Create the message
                let message = Message {
                    id: Uuid::new_v4().to_string(),
                    from: self.user_id.clone(),
                    to: recipient.clone(),
                    content: content.clone(),
                    timestamp: Utc::now(),
                    content_hash: ContentHash::default(),
                };
                
                // Calculate the content hash
                let message = message.with_content_hash(message.calculate_content_hash()?);
                
                // Send the message
                user_accessor.send_message(&self.user_id, recipient, message).await?;
                
                Ok(EffectResult::Success)
            },
            // ... other operations ...
        }
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.user_id.clone()]
    }
    
    fn capabilities(&self) -> Vec<CapabilityType> {
        match &self.operation {
            UserOperation::UpdateProfile(_) => vec![CapabilityType::Write],
            UserOperation::SendMessage { .. } => vec![CapabilityType::Execute],
            // ... other operations ...
        }
    }
}
```

## Where to Go Next

1. Explore the [Agent-Based Resources Architecture](../../architecture/core/agent-based-resources.md) for more details on the design
2. Learn about [The Capability System](../../architecture/core/capability-system.md) to understand agent-based permissions
3. See the [Resource System Implementation Guide](./resource-system.md) for more on the underlying resource system

## Reference

### Related ADRs
- [ADR-005: Invocation Model](../../../spec/adr_005_invocation.md)
- [ADR-032: Role-Based Resource System](../../../spec/adr-032-role-based-resource-system.md)
- [System Contract](../../../spec/system_contract.md)

### Related Architecture Documents
- [Agent-Based Resources](../../architecture/core/agent-based-resources.md)
- [Resource System](../../architecture/core/resource-system.md)
- [Capability System](../../architecture/core/capability-system.md) 