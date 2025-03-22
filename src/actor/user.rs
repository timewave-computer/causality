//! User actor implementation
//!
//! This module provides the User actor type, which represents end users
//! interacting with the system. It handles user authentication, permissions,
//! and state management.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

use crate::error::{Error, Result};
use crate::types::{Timestamp};
use super::{
    Actor, ActorId, ActorType, ActorState, ActorInfo, BaseActor,
    Message, MessageCategory, MessagePayload,
};

/// User authentication method
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Public key authentication
    PublicKey,
    /// Username and password
    Password,
    /// Multi-factor authentication
    MFA,
    /// OAuth provider
    OAuth(String),
}

/// User permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPermissions {
    /// Roles assigned to this user
    pub roles: HashSet<String>,
    /// Explicit permissions granted to this user
    pub permissions: HashSet<String>,
    /// Resources this user has access to
    pub resources: HashMap<String, HashSet<String>>,
}

impl UserPermissions {
    /// Create new empty permissions
    pub fn new() -> Self {
        UserPermissions {
            roles: HashSet::new(),
            permissions: HashSet::new(),
            resources: HashMap::new(),
        }
    }
    
    /// Add a role to this user
    pub fn add_role(&mut self, role: &str) {
        self.roles.insert(role.to_string());
    }
    
    /// Remove a role from this user
    pub fn remove_role(&mut self, role: &str) {
        self.roles.remove(role);
    }
    
    /// Add a permission to this user
    pub fn add_permission(&mut self, permission: &str) {
        self.permissions.insert(permission.to_string());
    }
    
    /// Remove a permission from this user
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.remove(permission);
    }
    
    /// Check if the user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }
    
    /// Add a resource permission
    pub fn add_resource_permission(&mut self, resource: &str, permission: &str) {
        self.resources
            .entry(resource.to_string())
            .or_insert_with(HashSet::new)
            .insert(permission.to_string());
    }
    
    /// Remove a resource permission
    pub fn remove_resource_permission(&mut self, resource: &str, permission: &str) {
        if let Some(perms) = self.resources.get_mut(resource) {
            perms.remove(permission);
            
            // Remove the resource entry if no permissions left
            if perms.is_empty() {
                self.resources.remove(resource);
            }
        }
    }
    
    /// Check if the user has permission for a resource
    pub fn has_resource_permission(&self, resource: &str, permission: &str) -> bool {
        self.resources
            .get(resource)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User's display name
    pub display_name: String,
    /// User's email address
    pub email: Option<String>,
    /// User's profile picture URL
    pub profile_picture: Option<String>,
    /// User's preferred language
    pub language: Option<String>,
    /// User's timezone
    pub timezone: Option<String>,
    /// Additional user metadata
    pub metadata: HashMap<String, String>,
}

impl UserProfile {
    /// Create a new user profile
    pub fn new(display_name: &str) -> Self {
        UserProfile {
            display_name: display_name.to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the profile
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// User actor implementation
pub struct User {
    /// Base actor implementation
    base: BaseActor,
    /// User's authentication methods
    auth_methods: Arc<RwLock<HashMap<AuthMethod, bool>>>, // Method -> Verified
    /// User's permissions
    permissions: Arc<RwLock<UserPermissions>>,
    /// User's profile
    profile: Arc<RwLock<UserProfile>>,
    /// User's access tokens
    access_tokens: Arc<RwLock<HashMap<String, Timestamp>>>, // Token -> Expiry
}

impl User {
    /// Create a new user actor
    pub fn new(
        id: ActorId,
        name: &str,
    ) -> Self {
        let base = BaseActor::new(
            id,
            ActorType::User,
            name,
        );
        
        let user = User {
            base,
            auth_methods: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(UserPermissions::new())),
            profile: Arc::new(RwLock::new(UserProfile::new(name))),
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
        };
        
        user
    }
    
    /// Add an authentication method
    pub fn add_auth_method(&self, method: AuthMethod, verified: bool) -> Result<()> {
        let mut auth_methods = self.auth_methods.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        auth_methods.insert(method, verified);
        Ok(())
    }
    
    /// Verify an authentication method
    pub fn verify_auth_method(&self, method: &AuthMethod) -> Result<()> {
        let mut auth_methods = self.auth_methods.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        if let Some(verified) = auth_methods.get_mut(method) {
            *verified = true;
            Ok(())
        } else {
            Err(Error::NotFound("Authentication method not found".to_string()))
        }
    }
    
    /// Check if an authentication method is verified
    pub fn is_auth_method_verified(&self, method: &AuthMethod) -> Result<bool> {
        let auth_methods = self.auth_methods.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        Ok(auth_methods.get(method).copied().unwrap_or(false))
    }
    
    /// Add a role to this user
    pub fn add_role(&self, role: &str) -> Result<()> {
        let mut permissions = self.permissions.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        permissions.add_role(role);
        Ok(())
    }
    
    /// Add a permission to this user
    pub fn add_permission(&self, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        permissions.add_permission(permission);
        Ok(())
    }
    
    /// Add a resource permission
    pub fn add_resource_permission(&self, resource: &str, permission: &str) -> Result<()> {
        let mut permissions = self.permissions.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        permissions.add_resource_permission(resource, permission);
        Ok(())
    }
    
    /// Update the user's profile
    pub fn update_profile(&self, profile: UserProfile) -> Result<()> {
        let mut current_profile = self.profile.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        *current_profile = profile;
        Ok(())
    }
    
    /// Get the user's profile
    pub fn get_profile(&self) -> Result<UserProfile> {
        let profile = self.profile.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        Ok(profile.clone())
    }
    
    /// Create an access token for the user
    pub fn create_access_token(&self, ttl_seconds: u64) -> Result<String> {
        use uuid::Uuid;
        
        let token = Uuid::new_v4().to_string();
        let expires_at = Timestamp::now() + ttl_seconds;
        
        let mut tokens = self.access_tokens.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        tokens.insert(token.clone(), expires_at);
        
        Ok(token)
    }
    
    /// Validate an access token
    pub fn validate_token(&self, token: &str) -> Result<bool> {
        let tokens = self.access_tokens.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        match tokens.get(token) {
            Some(expires_at) => {
                let now = Timestamp::now();
                Ok(now.value() < expires_at.value())
            },
            None => Ok(false),
        }
    }
    
    /// Revoke an access token
    pub fn revoke_token(&self, token: &str) -> Result<bool> {
        let mut tokens = self.access_tokens.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        Ok(tokens.remove(token).is_some())
    }
    
    /// Clean up expired tokens
    pub fn cleanup_expired_tokens(&self) -> Result<usize> {
        let mut tokens = self.access_tokens.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        let now = Timestamp::now();
        let before_count = tokens.len();
        
        tokens.retain(|_, expires_at| now.value() < expires_at.value());
        
        Ok(before_count - tokens.len())
    }
}

#[async_trait]
impl Actor for User {
    fn id(&self) -> &ActorId {
        self.base.id()
    }
    
    fn actor_type(&self) -> ActorType {
        self.base.actor_type()
    }
    
    fn state(&self) -> ActorState {
        self.base.state()
    }
    
    fn info(&self) -> ActorInfo {
        self.base.info()
    }
    
    async fn initialize(&self) -> Result<()> {
        // Perform any necessary initialization
        self.base.update_state(ActorState::Pending)
    }
    
    async fn start(&self) -> Result<()> {
        // Start the actor
        self.base.update_state(ActorState::Active)
    }
    
    async fn stop(&self) -> Result<()> {
        // Stop the actor
        self.base.update_state(ActorState::Inactive)
    }
    
    async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
        // Handle different message types
        match message.category {
            MessageCategory::Command => self.handle_command(message).await,
            MessageCategory::Query => self.handle_query(message).await,
            MessageCategory::Event => {
                // Just acknowledge events
                Ok(Some(Message::reply_to(
                    &message,
                    self.base.id().clone(),
                    MessagePayload::Text("Event acknowledged".to_string()),
                )))
            },
            MessageCategory::System => self.handle_system_message(message).await,
            MessageCategory::Custom(_) => {
                // Unhandled custom message
                Err(Error::UnsupportedOperation("Unsupported message category".to_string()))
            }
        }
    }
    
    async fn has_permission(&self, permission: &str) -> Result<bool> {
        let perms = self.permissions.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        Ok(perms.has_permission(permission))
    }
}

impl User {
    /// Handle command messages
    async fn handle_command(&self, message: Message) -> Result<Option<Message>> {
        if let MessagePayload::Command { name, args } = &message.payload {
            match name.as_str() {
                "update_profile" => {
                    // Parse the profile JSON
                    match serde_json::from_str::<UserProfile>(&args) {
                        Ok(profile) => {
                            self.update_profile(profile)?;
                            
                            Ok(Some(Message::reply_to(
                                &message,
                                self.base.id().clone(),
                                MessagePayload::Text("Profile updated".to_string()),
                            )))
                        },
                        Err(e) => {
                            Err(Error::ParseError(format!("Failed to parse profile: {}", e)))
                        }
                    }
                },
                "create_token" => {
                    // Parse TTL from args
                    match serde_json::from_str::<HashMap<String, u64>>(&args) {
                        Ok(params) => {
                            let ttl = params.get("ttl").cloned().unwrap_or(3600); // Default 1 hour
                            let token = self.create_access_token(ttl)?;
                            
                            Ok(Some(Message::reply_to(
                                &message,
                                self.base.id().clone(),
                                MessagePayload::Json(serde_json::to_string(&HashMap::from([
                                    ("token".to_string(), token),
                                    ("expires_in".to_string(), ttl.to_string()),
                                ])).unwrap()),
                            )))
                        },
                        Err(e) => {
                            Err(Error::ParseError(format!("Failed to parse token parameters: {}", e)))
                        }
                    }
                },
                "revoke_token" => {
                    // Parse token from args
                    match serde_json::from_str::<HashMap<String, String>>(&args) {
                        Ok(params) => {
                            if let Some(token) = params.get("token") {
                                let revoked = self.revoke_token(token)?;
                                
                                Ok(Some(Message::reply_to(
                                    &message,
                                    self.base.id().clone(),
                                    MessagePayload::Json(serde_json::to_string(&HashMap::from([
                                        ("revoked".to_string(), revoked.to_string()),
                                    ])).unwrap()),
                                )))
                            } else {
                                Err(Error::InvalidInput("Missing token parameter".to_string()))
                            }
                        },
                        Err(e) => {
                            Err(Error::ParseError(format!("Failed to parse revoke parameters: {}", e)))
                        }
                    }
                },
                _ => {
                    Err(Error::UnsupportedOperation(format!("Unsupported command: {}", name)))
                }
            }
        } else {
            Err(Error::InvalidInput("Expected Command payload".to_string()))
        }
    }
    
    /// Handle query messages
    async fn handle_query(&self, message: Message) -> Result<Option<Message>> {
        if let MessagePayload::Query { name, params } = &message.payload {
            match name.as_str() {
                "get_profile" => {
                    let profile = self.get_profile()?;
                    
                    Ok(Some(Message::reply_to(
                        &message,
                        self.base.id().clone(),
                        MessagePayload::Json(serde_json::to_string(&profile).unwrap()),
                    )))
                },
                "check_permission" => {
                    // Parse permission from params
                    match serde_json::from_str::<HashMap<String, String>>(&params) {
                        Ok(p) => {
                            if let Some(permission) = p.get("permission") {
                                let has_perm = self.has_permission(permission).await?;
                                
                                Ok(Some(Message::reply_to(
                                    &message,
                                    self.base.id().clone(),
                                    MessagePayload::Json(serde_json::to_string(&HashMap::from([
                                        ("has_permission".to_string(), has_perm.to_string()),
                                    ])).unwrap()),
                                )))
                            } else {
                                Err(Error::InvalidInput("Missing permission parameter".to_string()))
                            }
                        },
                        Err(e) => {
                            Err(Error::ParseError(format!("Failed to parse permission parameters: {}", e)))
                        }
                    }
                },
                "validate_token" => {
                    // Parse token from params
                    match serde_json::from_str::<HashMap<String, String>>(&params) {
                        Ok(p) => {
                            if let Some(token) = p.get("token") {
                                let valid = self.validate_token(token)?;
                                
                                Ok(Some(Message::reply_to(
                                    &message,
                                    self.base.id().clone(),
                                    MessagePayload::Json(serde_json::to_string(&HashMap::from([
                                        ("valid".to_string(), valid.to_string()),
                                    ])).unwrap()),
                                )))
                            } else {
                                Err(Error::InvalidInput("Missing token parameter".to_string()))
                            }
                        },
                        Err(e) => {
                            Err(Error::ParseError(format!("Failed to parse token parameters: {}", e)))
                        }
                    }
                },
                _ => {
                    Err(Error::UnsupportedOperation(format!("Unsupported query: {}", name)))
                }
            }
        } else {
            Err(Error::InvalidInput("Expected Query payload".to_string()))
        }
    }
    
    /// Handle system messages
    async fn handle_system_message(&self, message: Message) -> Result<Option<Message>> {
        if let MessagePayload::System { message_type, data } = &message.payload {
            match message_type.as_str() {
                "activate" => {
                    self.base.update_state(ActorState::Active)?;
                    
                    Ok(Some(Message::reply_to(
                        &message,
                        self.base.id().clone(),
                        MessagePayload::System {
                            message_type: "activated".to_string(),
                            data: "{}".to_string(),
                        },
                    )))
                },
                "deactivate" => {
                    self.base.update_state(ActorState::Inactive)?;
                    
                    Ok(Some(Message::reply_to(
                        &message,
                        self.base.id().clone(),
                        MessagePayload::System {
                            message_type: "deactivated".to_string(),
                            data: "{}".to_string(),
                        },
                    )))
                },
                "cleanup" => {
                    let removed = self.cleanup_expired_tokens()?;
                    
                    Ok(Some(Message::reply_to(
                        &message,
                        self.base.id().clone(),
                        MessagePayload::System {
                            message_type: "cleanup_result".to_string(),
                            data: serde_json::to_string(&HashMap::from([
                                ("removed_tokens".to_string(), removed.to_string()),
                            ])).unwrap(),
                        },
                    )))
                },
                _ => {
                    Err(Error::UnsupportedOperation(format!("Unsupported system message: {}", message_type)))
                }
            }
        } else {
            Err(Error::InvalidInput("Expected System payload".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_permissions() {
        let user = User::new(
            ActorId::new("test-user"),
            "Test User",
        );
        
        // Add permissions and roles
        user.add_role("user").unwrap();
        user.add_permission("read").unwrap();
        user.add_permission("write").unwrap();
        user.add_resource_permission("document:1", "read").unwrap();
        
        // Check permissions
        assert!(user.has_permission("read").await.unwrap());
        assert!(user.has_permission("write").await.unwrap());
        assert!(!user.has_permission("delete").await.unwrap());
        
        // Check permissions through the User actor
        let query_message = Message::new(
            Some(ActorId::new("test-sender")),
            user.id().clone(),
            MessageCategory::Query,
            MessagePayload::Query {
                name: "check_permission".to_string(),
                params: r#"{"permission":"read"}"#.to_string(),
            },
        );
        
        let response = user.handle_message(query_message).await.unwrap().unwrap();
        
        if let MessagePayload::Json(json) = response.payload {
            let result: HashMap<String, String> = serde_json::from_str(&json).unwrap();
            assert_eq!(result.get("has_permission").unwrap(), "true");
        } else {
            panic!("Expected JSON payload");
        }
    }
    
    #[tokio::test]
    async fn test_user_tokens() {
        let user = User::new(
            ActorId::new("test-user"),
            "Test User",
        );
        
        // Create token
        let token = user.create_access_token(3600).unwrap();
        
        // Validate token
        assert!(user.validate_token(&token).unwrap());
        
        // Revoke token
        assert!(user.revoke_token(&token).unwrap());
        
        // Token should no longer be valid
        assert!(!user.validate_token(&token).unwrap());
    }
    
    #[tokio::test]
    async fn test_user_profile() {
        let user = User::new(
            ActorId::new("test-user"),
            "Test User",
        );
        
        // Update profile
        let profile = UserProfile::new("Updated Name")
            .with_email("user@example.com")
            .with_language("en-US");
        
        user.update_profile(profile.clone()).unwrap();
        
        // Get profile
        let retrieved_profile = user.get_profile().unwrap();
        assert_eq!(retrieved_profile.display_name, "Updated Name");
        assert_eq!(retrieved_profile.email, Some("user@example.com".to_string()));
        assert_eq!(retrieved_profile.language, Some("en-US".to_string()));
    }
    
    #[tokio::test]
    async fn test_user_message_handling() {
        let user = User::new(
            ActorId::new("test-user"),
            "Test User",
        );
        
        // Test system message
        let system_message = Message::new(
            Some(ActorId::new("test-sender")),
            user.id().clone(),
            MessageCategory::System,
            MessagePayload::System {
                message_type: "activate".to_string(),
                data: "{}".to_string(),
            },
        );
        
        let response = user.handle_message(system_message).await.unwrap().unwrap();
        
        if let MessagePayload::System { message_type, .. } = response.payload {
            assert_eq!(message_type, "activated");
        } else {
            panic!("Expected System payload");
        }
        
        // User should now be active
        assert_eq!(user.state(), ActorState::Active);
    }
} 