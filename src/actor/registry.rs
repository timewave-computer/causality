// Actor Registry Module
//
// This module provides registry functionality for actors in the Causality system.
// It allows actors to be registered, discovered, and managed.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::{ContentId, ContentHash, TraceId};
use crate::actor::{Actor, ActorId, ActorRole, ActorState, ActorMetadata, ActorCapability};

/// Registration status for actors
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistrationStatus {
    /// Actor is pending registration
    Pending,
    /// Actor is registered
    Registered,
    /// Actor registration has been revoked
    Revoked(String),
    /// Actor registration has failed
    Failed(String),
}

/// Actor registration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorRegistration {
    /// The actor ID
    pub actor_id: ActorId,
    /// Roles assigned to this actor
    pub roles: Vec<ActorRole>,
    /// Registration status
    pub status: RegistrationStatus,
    /// When this registration was created
    pub created_at: u64,
    /// When this registration was last updated
    pub updated_at: u64,
    /// Additional metadata for this registration
    pub metadata: ActorMetadata,
}

impl ActorRegistration {
    /// Create a new actor registration
    pub fn new(
        actor_id: ActorId, 
        roles: Vec<ActorRole>, 
        metadata: ActorMetadata
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        ActorRegistration {
            actor_id,
            roles,
            status: RegistrationStatus::Pending,
            created_at: now,
            updated_at: now,
            metadata,
        }
    }
    
    /// Check if this registration is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, RegistrationStatus::Registered)
    }
    
    /// Mark this registration as registered
    pub fn mark_registered(&mut self) {
        self.status = RegistrationStatus::Registered;
        self.update_timestamp();
    }
    
    /// Mark this registration as revoked
    pub fn mark_revoked(&mut self, reason: impl Into<String>) {
        self.status = RegistrationStatus::Revoked(reason.into());
        self.update_timestamp();
    }
    
    /// Mark this registration as failed
    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.status = RegistrationStatus::Failed(reason.into());
        self.update_timestamp();
    }
    
    /// Update the timestamp on this registration
    fn update_timestamp(&mut self) {
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Get a content ID for this registration
    pub fn content_id(&self) -> ContentId {
        let hash_str = format!(
            "{}:{}:{}",
            self.actor_id.as_str(),
            format!("{:?}", self.status),
            self.updated_at
        );
        
        let hash = ContentHash::new(&hash_str);
        ContentId::new(hash, "actor-registration")
    }
}

/// The actor registry for managing actor registrations
#[derive(Debug)]
pub struct ActorRegistry {
    /// Actor registrations by ID
    registrations: RwLock<HashMap<ActorId, ActorRegistration>>,
    /// Actor instances by ID
    actors: RwLock<HashMap<ActorId, Arc<dyn Actor>>>,
}

impl ActorRegistry {
    /// Create a new actor registry
    pub fn new() -> Self {
        ActorRegistry {
            registrations: RwLock::new(HashMap::new()),
            actors: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new actor
    pub fn register(&self, registration: ActorRegistration) -> Result<()> {
        let mut registrations = self.registrations.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on registrations".to_string())
        })?;
        
        let actor_id = registration.actor_id.clone();
        registrations.insert(actor_id, registration);
        
        Ok(())
    }
    
    /// Register an actor instance
    pub fn register_actor(&self, actor: Arc<dyn Actor>) -> Result<()> {
        let actor_id = actor.id().clone();
        
        // Check if the actor is registered
        let registrations = self.registrations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        let is_registered = registrations.contains_key(&actor_id);
        
        if !is_registered {
            return Err(Error::NotRegistered(format!(
                "Actor {} is not registered",
                actor_id.as_str()
            )));
        }
        
        // Add the actor to the actors map
        let mut actors = self.actors.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on actors".to_string())
        })?;
        
        actors.insert(actor_id, actor);
        
        Ok(())
    }
    
    /// Get a registration by actor ID
    pub fn get_registration(&self, actor_id: &ActorId) -> Result<Option<ActorRegistration>> {
        let registrations = self.registrations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        Ok(registrations.get(actor_id).cloned())
    }
    
    /// Get an actor by ID
    pub fn get_actor(&self, actor_id: &ActorId) -> Result<Option<Arc<dyn Actor>>> {
        let actors = self.actors.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on actors".to_string())
        })?;
        
        Ok(actors.get(actor_id).cloned())
    }
    
    /// Get all actor registrations
    pub fn get_all_registrations(&self) -> Result<Vec<ActorRegistration>> {
        let registrations = self.registrations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        Ok(registrations.values().cloned().collect())
    }
    
    /// Get all actors with a specific role
    pub fn get_actors_by_role(&self, role: &ActorRole) -> Result<Vec<Arc<dyn Actor>>> {
        let registrations = self.registrations.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        let actors = self.actors.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on actors".to_string())
        })?;
        
        let actor_ids: Vec<ActorId> = registrations.values()
            .filter(|reg| reg.roles.contains(role) && reg.is_active())
            .map(|reg| reg.actor_id.clone())
            .collect();
            
        let mut result = Vec::new();
        for actor_id in actor_ids {
            if let Some(actor) = actors.get(&actor_id) {
                result.push(actor.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Get all actors with a specific capability
    pub fn get_actors_by_capability(&self, capability: &ActorCapability) -> Result<Vec<Arc<dyn Actor>>> {
        let actors = self.actors.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on actors".to_string())
        })?;
        
        let mut result = Vec::new();
        for actor in actors.values() {
            if actor.has_capability(capability) {
                result.push(actor.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Update a registration status
    pub fn update_registration_status(
        &self, 
        actor_id: &ActorId, 
        status: RegistrationStatus
    ) -> Result<()> {
        let mut registrations = self.registrations.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on registrations".to_string())
        })?;
        
        if let Some(registration) = registrations.get_mut(actor_id) {
            registration.status = status;
            registration.update_timestamp();
            Ok(())
        } else {
            Err(Error::NotFound(format!(
                "Actor registration not found: {}",
                actor_id.as_str()
            )))
        }
    }
    
    /// Revoke an actor registration
    pub fn revoke_registration(&self, actor_id: &ActorId, reason: impl Into<String>) -> Result<()> {
        self.update_registration_status(
            actor_id, 
            RegistrationStatus::Revoked(reason.into())
        )
    }
    
    /// Remove an actor from the registry
    pub fn remove_actor(&self, actor_id: &ActorId) -> Result<()> {
        let mut actors = self.actors.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on actors".to_string())
        })?;
        
        actors.remove(actor_id);
        
        Ok(())
    }
}

impl Default for ActorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    
    #[derive(Debug)]
    struct TestActor {
        id: ActorId,
        roles: Vec<ActorRole>,
        state: ActorState,
        metadata: ActorMetadata,
    }
    
    impl TestActor {
        fn new(id: &str, roles: Vec<ActorRole>) -> Self {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
                
            let metadata = ActorMetadata {
                name: format!("Test Actor {}", id),
                description: Some("A test actor for unit tests".to_string()),
                created_at: now,
                updated_at: now,
                custom: HashMap::new(),
            };
            
            TestActor {
                id: ActorId::new(id),
                roles,
                state: ActorState::Created,
                metadata,
            }
        }
    }
    
    #[async_trait]
    impl Actor for TestActor {
        fn id(&self) -> &ActorId {
            &self.id
        }
        
        fn roles(&self) -> Vec<ActorRole> {
            self.roles.clone()
        }
        
        fn state(&self) -> ActorState {
            self.state.clone()
        }
        
        fn metadata(&self) -> &ActorMetadata {
            &self.metadata
        }
        
        async fn initialize(&mut self) -> Result<()> {
            self.state = ActorState::Active;
            Ok(())
        }
        
        async fn pause(&mut self) -> Result<()> {
            self.state = ActorState::Paused;
            Ok(())
        }
        
        async fn resume(&mut self) -> Result<()> {
            self.state = ActorState::Active;
            Ok(())
        }
        
        async fn stop(&mut self) -> Result<()> {
            self.state = ActorState::Stopped;
            Ok(())
        }
        
        async fn terminate(&mut self) -> Result<()> {
            self.state = ActorState::Terminated;
            Ok(())
        }
    }
    
    #[test]
    fn test_actor_registration() {
        let actor_id = ActorId::new("test-actor");
        let roles = vec![ActorRole::User];
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let metadata = ActorMetadata {
            name: "Test Actor".to_string(),
            description: Some("A test actor for unit tests".to_string()),
            created_at: now,
            updated_at: now,
            custom: HashMap::new(),
        };
        
        let mut registration = ActorRegistration::new(
            actor_id.clone(),
            roles.clone(),
            metadata,
        );
        
        // Test initial state
        assert_eq!(registration.actor_id, actor_id);
        assert_eq!(registration.roles, roles);
        assert_eq!(registration.status, RegistrationStatus::Pending);
        assert!(!registration.is_active());
        
        // Test marking as registered
        registration.mark_registered();
        assert_eq!(registration.status, RegistrationStatus::Registered);
        assert!(registration.is_active());
        
        // Test revoking registration
        registration.mark_revoked("Test revocation");
        assert_eq!(
            registration.status, 
            RegistrationStatus::Revoked("Test revocation".to_string())
        );
        assert!(!registration.is_active());
        
        // Test content ID
        let content_id = registration.content_id();
        assert_eq!(content_id.content_type, "actor-registration");
    }
    
    #[test]
    fn test_actor_registry() -> Result<()> {
        let registry = ActorRegistry::new();
        
        // Create an actor and registration
        let actor_id = ActorId::new("test-actor");
        let roles = vec![ActorRole::User];
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let metadata = ActorMetadata {
            name: "Test Actor".to_string(),
            description: Some("A test actor for unit tests".to_string()),
            created_at: now,
            updated_at: now,
            custom: HashMap::new(),
        };
        
        let mut registration = ActorRegistration::new(
            actor_id.clone(),
            roles.clone(),
            metadata.clone(),
        );
        
        // Mark as registered
        registration.mark_registered();
        
        // Register the actor
        registry.register(registration)?;
        
        // Create a test actor instance
        let actor = Arc::new(TestActor::new("test-actor", roles.clone()));
        
        // Register the actor instance
        registry.register_actor(actor.clone())?;
        
        // Test retrieving the registration
        let retrieved_registration = registry.get_registration(&actor_id)?;
        assert!(retrieved_registration.is_some());
        
        let reg = retrieved_registration.unwrap();
        assert_eq!(reg.actor_id, actor_id);
        assert_eq!(reg.roles, roles);
        assert_eq!(reg.status, RegistrationStatus::Registered);
        
        // Test retrieving the actor
        let retrieved_actor = registry.get_actor(&actor_id)?;
        assert!(retrieved_actor.is_some());
        
        // Test getting actors by role
        let actors_by_role = registry.get_actors_by_role(&ActorRole::User)?;
        assert_eq!(actors_by_role.len(), 1);
        
        // Test getting actors by capability
        let actors_by_capability = registry.get_actors_by_capability(&ActorCapability::CreateProgram)?;
        assert_eq!(actors_by_capability.len(), 1);
        
        // Test getting all registrations
        let all_registrations = registry.get_all_registrations()?;
        assert_eq!(all_registrations.len(), 1);
        
        // Test revoking registration
        registry.revoke_registration(&actor_id, "Test revocation")?;
        
        let updated_registration = registry.get_registration(&actor_id)?.unwrap();
        assert_eq!(
            updated_registration.status,
            RegistrationStatus::Revoked("Test revocation".to_string())
        );
        
        // Test removing actor
        registry.remove_actor(&actor_id)?;
        
        let removed_actor = registry.get_actor(&actor_id)?;
        assert!(removed_actor.is_none());
        
        Ok(())
    }
} 