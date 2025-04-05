// user.rs - User agent implementation
//
// This file implements the specialized UserAgent type, representing human users of the system.

use crate::resource_types::{ResourceId, ResourceType};
use crate::resource::{Resource, ResourceState, ResourceResult, ResourceError};
use crate::resource::operation::Capability;

use super::types::{AgentId, AgentType, AgentState, AgentRelationship, AgentError};
use super::agent::{Agent, AgentImpl};

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// User-specific error types
#[derive(Error, Debug)]
pub enum UserAgentError {
    /// Base agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Profile error
    #[error("Profile error: {0}")]
    ProfileError(String),
    
    /// Other error
    #[error("User error: {0}")]
    Other(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),
}

/// Authentication method used by a user
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthenticationMethod {
    /// Password authentication
    Password {
        /// Password hash
        hash: String,
        
        /// Salt for the password
        salt: String,
    },
    
    /// Public key authentication
    PublicKey {
        /// Public key for verification
        public_key: String,
    },
    
    /// OAuth authentication
    OAuth {
        /// OAuth provider
        provider: String,
        
        /// OAuth token
        token: String,
    },
    
    /// Multi-factor authentication
    MultiFactorAuth {
        /// Primary authentication method
        primary: Box<AuthenticationMethod>,
        
        /// Secondary authentication method
        secondary: Box<AuthenticationMethod>,
    },
}

/// User profile information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    /// Display name
    pub display_name: Option<String>,
    
    /// Email address
    pub email: Option<String>,
    
    /// Profile picture URL
    pub profile_picture: Option<String>,
    
    /// User preferences
    pub preferences: HashMap<String, String>,
    
    /// Additional profile fields
    pub additional_fields: HashMap<String, String>,
}

/// User agent that specializes the base agent with user-specific functionality
#[derive(Clone, Debug)]
pub struct UserAgent {
    /// Base agent implementation
    base: AgentImpl,
    
    /// Authentication method
    auth_method: AuthenticationMethod,
    
    /// User profile
    profile: UserProfile,
    
    /// Account IDs associated with this user
    account_ids: Vec<ResourceId>,
    
    /// Last login timestamp
    last_login: Option<u64>,
    
    /// Is the user verified
    verified: bool,
}

impl UserAgent {
    /// Create a new user agent
    pub fn new(
        base_agent: AgentImpl,
        auth_method: AuthenticationMethod,
        profile: Option<UserProfile>,
        account_ids: Option<Vec<ResourceId>>,
    ) -> Result<Self, UserAgentError> {
        // Verify that the base agent has the correct type
        if base_agent.agent_type() != &AgentType::User {
            return Err(UserAgentError::Other(
                "Base agent must have User agent type".to_string()
            ));
        }
        
        Ok(Self {
            base: base_agent,
            auth_method,
            profile: profile.unwrap_or_default(),
            account_ids: account_ids.unwrap_or_default(),
            last_login: None,
            verified: false,
        })
    }
    
    /// Get the authentication method
    pub fn auth_method(&self) -> &AuthenticationMethod {
        &self.auth_method
    }
    
    /// Set the authentication method
    pub async fn set_auth_method(&mut self, auth_method: AuthenticationMethod) -> Result<(), UserAgentError> {
        self.auth_method = auth_method;
        
        // Update the content hash
        self.base.set_metadata("auth_method_updated", &chrono::Utc::now().to_rfc3339())
            .map_err(|e: ResourceError| UserAgentError::ResourceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get the user profile
    pub fn profile(&self) -> &UserProfile {
        &self.profile
    }
    
    /// Update the user profile
    pub async fn update_profile(&mut self, profile: UserProfile) -> Result<(), UserAgentError> {
        self.profile = profile;
        
        // Update the content hash
        self.base.set_metadata("profile_updated", &chrono::Utc::now().to_rfc3339())
            .map_err(|e: ResourceError| UserAgentError::ResourceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get the account IDs associated with this user
    pub fn account_ids(&self) -> &[ResourceId] {
        &self.account_ids
    }
    
    /// Add an account ID
    pub async fn add_account_id(&mut self, account_id: ResourceId) -> Result<(), UserAgentError> {
        self.account_ids.push(account_id);
        
        // Update the content hash
        self.base.set_metadata("account_added", &chrono::Utc::now().to_rfc3339())
            .map_err(|e: ResourceError| UserAgentError::ResourceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Verify a user's identity
    pub async fn verify(&mut self) -> Result<(), UserAgentError> {
        self.verified = true;
        
        // Update the content hash
        self.base.set_metadata("verified", "true")
            .map_err(|e: ResourceError| UserAgentError::ResourceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Check if the user is verified
    pub fn is_verified(&self) -> bool {
        self.verified
    }
    
    /// Record a login
    pub async fn record_login(&mut self) -> Result<(), UserAgentError> {
        let now = chrono::Utc::now().timestamp() as u64;
        self.last_login = Some(now);
        
        // Update the content hash
        self.base.set_metadata("last_login", &ToString::to_string(&now))
            .map_err(|e: ResourceError| UserAgentError::ResourceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get the last login timestamp
    pub fn last_login(&self) -> Option<u64> {
        self.last_login
    }
    
    /// Authenticate the user with the provided credentials
    pub async fn authenticate(&mut self, credentials: &str) -> Result<bool, UserAgentError> {
        // Very basic authentication logic for demonstration
        let authenticated = match &self.auth_method {
            AuthenticationMethod::Password { hash, salt: _ } => {
                // In a real implementation, we would properly hash and validate the password
                credentials == hash
            },
            AuthenticationMethod::PublicKey { public_key } => {
                // In a real implementation, we would validate a signature
                credentials == public_key
            },
            AuthenticationMethod::OAuth { provider: _, token } => {
                // In a real implementation, we would validate the token with the OAuth provider
                credentials == token
            },
            AuthenticationMethod::MultiFactorAuth { .. } => {
                // In a real implementation, we would have a proper MFA flow
                false
            },
        };
        
        if authenticated {
            self.record_login().await?;
        }
        
        Ok(authenticated)
    }
}

// Implement Default for UserProfile
impl Default for UserProfile {
    fn default() -> Self {
        Self {
            display_name: None,
            email: None,
            profile_picture: None,
            preferences: HashMap::new(),
            additional_fields: HashMap::new(),
        }
    }
}

// Delegate the Agent trait methods to the base agent
#[async_trait]
impl Agent for UserAgent {
    fn agent_id(&self) -> &AgentId {
        self.base.agent_id()
    }
    
    fn agent_type(&self) -> &AgentType {
        self.base.agent_type()
    }
    
    fn state(&self) -> &AgentState {
        Agent::state(&self.base)
    }
    
    async fn set_state(&mut self, state: AgentState) -> Result<(), AgentError> {
        self.base.set_state(state).await
    }
    
    async fn add_capability(&mut self, capability: Capability<Box<dyn Resource>>) -> Result<(), AgentError> {
        self.base.add_capability(capability).await
    }
    
    async fn remove_capability(&mut self, capability_id: &str) -> Result<(), AgentError> {
        self.base.remove_capability(capability_id).await
    }
    
    fn has_capability(&self, capability_id: &str) -> bool {
        self.base.has_capability(capability_id)
    }
    
    fn capabilities(&self) -> Vec<Capability<Box<dyn Resource>>> {
        self.base.capabilities()
    }
    
    async fn add_relationship(&mut self, relationship: AgentRelationship) -> Result<(), AgentError> {
        self.base.add_relationship(relationship).await
    }
    
    async fn remove_relationship(&mut self, target_id: &ResourceId) -> Result<(), AgentError> {
        self.base.remove_relationship(target_id).await
    }
    
    fn relationships(&self) -> Vec<AgentRelationship> {
        self.base.relationships()
    }
    
    fn get_relationship(&self, target_id: &ResourceId) -> Option<&AgentRelationship> {
        self.base.get_relationship(target_id)
    }
    
    fn clone_agent(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}

// Implement the Resource trait through delegation to the base agent
#[async_trait]
impl Resource for UserAgent {
    fn id(&self) -> ResourceId {
        self.base.id().clone()
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::new("Agent", "1.0")
    }

    fn state(&self) -> ResourceState {
        match Agent::state(&self.base) {
            &AgentState::Active => ResourceState::Active,
            &AgentState::Inactive => ResourceState::Created,
            &AgentState::Suspended { .. } => ResourceState::Locked,
        }
    }

    fn get_metadata(&self, key: &str) -> Option<String> {
        self.base.get_metadata(key)
    }

    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()> {
        self.base.set_metadata(key, value)
    }
    
    fn clone_resource(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating user agents
pub struct UserAgentBuilder {
    /// Base agent builder
    base_builder: super::agent::AgentBuilder,
    
    /// Authentication method
    auth_method: Option<AuthenticationMethod>,
    
    /// User profile
    profile: UserProfile,
    
    /// Account IDs
    account_ids: Vec<ResourceId>,
}

impl UserAgentBuilder {
    /// Create a new user agent builder
    pub fn new() -> Self {
        Self {
            base_builder: super::agent::AgentBuilder::new().agent_type(AgentType::User),
            auth_method: None,
            profile: UserProfile::default(),
            account_ids: Vec::new(),
        }
    }
    
    /// Set the agent state
    pub fn state(mut self, state: AgentState) -> Self {
        self.base_builder = self.base_builder.state(state);
        self
    }
    
    /// Add a capability
    pub fn with_capability(mut self, capability: Capability<Box<dyn Resource>>) -> Self {
        self.base_builder = self.base_builder.with_capability(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<Capability<Box<dyn Resource>>>) -> Self {
        self.base_builder = self.base_builder.with_capabilities(capabilities);
        self
    }
    
    /// Add a relationship
    pub fn with_relationship(mut self, relationship: AgentRelationship) -> Self {
        self.base_builder = self.base_builder.with_relationship(relationship);
        self
    }
    
    /// Add multiple relationships
    pub fn with_relationships(mut self, relationships: Vec<AgentRelationship>) -> Self {
        self.base_builder = self.base_builder.with_relationships(relationships);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.base_builder = self.base_builder.with_metadata(key, value);
        self
    }
    
    /// Set the authentication method
    pub fn with_auth_method(mut self, auth_method: AuthenticationMethod) -> Self {
        self.auth_method = Some(auth_method);
        self
    }
    
    /// Set the display name
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.profile.display_name = Some(display_name.into());
        self
    }
    
    /// Set the email
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.profile.email = Some(email.into());
        self
    }
    
    /// Set the profile picture
    pub fn with_profile_picture(mut self, profile_picture: impl Into<String>) -> Self {
        self.profile.profile_picture = Some(profile_picture.into());
        self
    }
    
    /// Add a preference
    pub fn with_preference(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.profile.preferences.insert(key.into(), value.into());
        self
    }
    
    /// Add an additional profile field
    pub fn with_additional_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.profile.additional_fields.insert(key.into(), value.into());
        self
    }
    
    /// Add an account ID
    pub fn with_account_id(mut self, account_id: ResourceId) -> Self {
        self.account_ids.push(account_id);
        self
    }
    
    /// Build the user agent
    pub fn build(self) -> Result<UserAgent, UserAgentError> {
        // Build the base agent
        let base_agent = self.base_builder.build()
            .map_err(UserAgentError::AgentError)?;
        
        // Create the user agent
        let auth_method = self.auth_method.ok_or_else(|| {
            UserAgentError::Other("Authentication method is required".to_string())
        })?;
        
        UserAgent::new(
            base_agent,
            auth_method,
            Some(self.profile),
            Some(self.account_ids),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_agent_creation() {
        // Create a user agent
        let user_agent = UserAgentBuilder::new()
            .state(AgentState::Active)
            .with_auth_method(AuthenticationMethod::Password {
                hash: "hashed_password".to_string(),
                salt: "salt".to_string(),
            })
            .with_display_name("Alice")
            .with_email("alice@example.com")
            .build()
            .unwrap();
        
        // Check the agent type
        assert_eq!(user_agent.agent_type(), &AgentType::User);
        
        // Check the profile
        assert_eq!(user_agent.profile().display_name, Some("Alice".to_string()));
        assert_eq!(user_agent.profile().email, Some("alice@example.com".to_string()));
    }
    
    #[tokio::test]
    async fn test_user_authentication() {
        // Create a user agent with password authentication
        let mut user_agent = UserAgentBuilder::new()
            .state(AgentState::Active)
            .with_auth_method(AuthenticationMethod::Password {
                hash: "hashed_password".to_string(),
                salt: "salt".to_string(),
            })
            .build()
            .unwrap();
        
        // Authenticate with correct password
        let result = user_agent.authenticate("hashed_password").await.unwrap();
        assert!(result);
        
        // Authenticate with incorrect password
        let result = user_agent.authenticate("wrong_password").await.unwrap();
        assert!(!result);
    }
    
    #[tokio::test]
    async fn test_user_verification() {
        // Create a user agent
        let mut user_agent = UserAgentBuilder::new()
            .state(AgentState::Active)
            .with_auth_method(AuthenticationMethod::Password {
                hash: "hashed_password".to_string(),
                salt: "salt".to_string(),
            })
            .build()
            .unwrap();
        
        // Initially not verified
        assert!(!user_agent.is_verified());
        
        // Verify the user
        user_agent.verify().await.unwrap();
        
        // Now verified
        assert!(user_agent.is_verified());
    }
    
    #[tokio::test]
    async fn test_account_management() {
        // Create a user agent
        let mut user_agent = UserAgentBuilder::new()
            .state(AgentState::Active)
            .with_auth_method(AuthenticationMethod::Password {
                hash: "hashed_password".to_string(),
                salt: "salt".to_string(),
            })
            .build()
            .unwrap();
        
        // Initially no accounts
        assert_eq!(user_agent.account_ids().len(), 0);
        
        // Add an account
        let account_id = ResourceId::new(content_addressing::default_content_hash());
        user_agent.add_account_id(account_id.clone()).await.unwrap();
        
        // Check accounts
        assert_eq!(user_agent.account_ids().len(), 1);
        assert_eq!(&user_agent.account_ids()[0], &account_id);
    }
} 
