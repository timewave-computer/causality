// Basic resource actor implementations
//
// This module provides basic implementations of resource actors,
// including the core actor types and their behaviors.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::time;

use crate::crypto::{ContentId, ContentAddressed};

use super::{
    ActorError, ActorResult, ResourceActor, ResourceActorFactory, MessageBus,
    message::Message, state::StateManager, supervision::Supervisor,
    registry::ActorRegistry,
};

/// Basic resource actor implementation
pub struct BasicResourceActor {
    /// Actor ID
    id: ContentId,
    
    /// Actor type
    actor_type: String,
    
    /// Actor state
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    
    /// Actor metadata
    metadata: Arc<RwLock<HashMap<String, String>>>,
    
    /// Message bus
    message_bus: Arc<dyn MessageBus>,
    
    /// State manager
    state_manager: Arc<dyn StateManager>,
    
    /// Supervisor
    supervisor: Arc<dyn Supervisor>,
    
    /// Registry
    registry: Arc<dyn ActorRegistry>,
}

impl BasicResourceActor {
    /// Create a new basic resource actor
    pub fn new(
        id: ContentId,
        actor_type: String,
        message_bus: Arc<dyn MessageBus>,
        state_manager: Arc<dyn StateManager>,
        supervisor: Arc<dyn Supervisor>,
        registry: Arc<dyn ActorRegistry>,
    ) -> Self {
        Self {
            id,
            actor_type,
            state: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            state_manager,
            supervisor,
            registry,
        }
    }
}

#[async_trait]
impl ResourceActor for BasicResourceActor {
    fn id(&self) -> ContentId {
        self.id
    }
    
    fn actor_type(&self) -> &str {
        &self.actor_type
    }
    
    async fn get_state(&self) -> ActorResult<HashMap<String, serde_json::Value>> {
        Ok(self.state.read().await.clone())
    }
    
    async fn set_state(&self, state: HashMap<String, serde_json::Value>) -> ActorResult<()> {
        *self.state.write().await = state;
        Ok(())
    }
    
    async fn get_metadata(&self) -> ActorResult<HashMap<String, String>> {
        Ok(self.metadata.read().await.clone())
    }
    
    async fn set_metadata(&self, metadata: HashMap<String, String>) -> ActorResult<()> {
        *self.metadata.write().await = metadata;
        Ok(())
    }
    
    async fn handle_message(&self, message: Message) -> ActorResult<()> {
        // Handle message based on type
        match message.message_type.as_str() {
            "get_state" => {
                let state = self.get_state().await?;
                let response = Message::new(
                    self.id,
                    message.sender,
                    "state_response",
                    serde_json::to_value(state)?,
                    HashMap::new(),
                );
                self.message_bus.send(response).await?;
            }
            
            "set_state" => {
                let state: HashMap<String, serde_json::Value> = serde_json::from_value(message.payload)?;
                self.set_state(state).await?;
                let response = Message::new(
                    self.id,
                    message.sender,
                    "state_updated",
                    serde_json::Value::Null,
                    HashMap::new(),
                );
                self.message_bus.send(response).await?;
            }
            
            "get_metadata" => {
                let metadata = self.get_metadata().await?;
                let response = Message::new(
                    self.id,
                    message.sender,
                    "metadata_response",
                    serde_json::to_value(metadata)?,
                    HashMap::new(),
                );
                self.message_bus.send(response).await?;
            }
            
            "set_metadata" => {
                let metadata: HashMap<String, String> = serde_json::from_value(message.payload)?;
                self.set_metadata(metadata).await?;
                let response = Message::new(
                    self.id,
                    message.sender,
                    "metadata_updated",
                    serde_json::Value::Null,
                    HashMap::new(),
                );
                self.message_bus.send(response).await?;
            }
            
            _ => {
                return Err(ActorError::MessageError(format!(
                    "Unknown message type: {}",
                    message.message_type
                )));
            }
        }
        
        Ok(())
    }
    
    async fn start(&self) -> ActorResult<()> {
        // Initialize actor state
        let state = self.state_manager.get_state(&self.id).await?;
        self.set_state(state.state).await?;
        
        // Register actor
        self.registry.register_actor(Arc::new(self.clone())).await?;
        
        Ok(())
    }
    
    async fn stop(&self) -> ActorResult<()> {
        // Unregister actor
        self.registry.unregister_actor(&self.id).await?;
        
        Ok(())
    }
}

impl Clone for BasicResourceActor {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            actor_type: self.actor_type.clone(),
            state: self.state.clone(),
            metadata: self.metadata.clone(),
            message_bus: self.message_bus.clone(),
            state_manager: self.state_manager.clone(),
            supervisor: self.supervisor.clone(),
            registry: self.registry.clone(),
        }
    }
}

/// Basic resource actor factory
pub struct BasicResourceActorFactory {
    /// Message bus
    message_bus: Arc<dyn MessageBus>,
    
    /// State manager
    state_manager: Arc<dyn StateManager>,
    
    /// Supervisor
    supervisor: Arc<dyn Supervisor>,
    
    /// Registry
    registry: Arc<dyn ActorRegistry>,
}

impl BasicResourceActorFactory {
    /// Create a new basic resource actor factory
    pub fn new(
        message_bus: Arc<dyn MessageBus>,
        state_manager: Arc<dyn StateManager>,
        supervisor: Arc<dyn Supervisor>,
        registry: Arc<dyn ActorRegistry>,
    ) -> Self {
        Self {
            message_bus,
            state_manager,
            supervisor,
            registry,
        }
    }
}

#[async_trait]
impl ResourceActorFactory for BasicResourceActorFactory {
    async fn create_actor(
        &self,
        actor_type: &str,
        initial_state: super::InitialActorState,
    ) -> ActorResult<Arc<dyn ResourceActor>> {
        // Create actor ID
        let id = ContentId::new(&format!("{}_{}", actor_type, initial_state.id));
        
        // Create actor
        let actor = Arc::new(BasicResourceActor::new(
            id,
            actor_type.to_string(),
            self.message_bus.clone(),
            self.state_manager.clone(),
            self.supervisor.clone(),
            self.registry.clone(),
        ));
        
        // Set initial state
        actor.set_state(initial_state.state).await?;
        actor.set_metadata(initial_state.metadata).await?;
        
        // Start actor
        actor.start().await?;
        
        Ok(actor)
    }
    
    fn supported_types(&self) -> Vec<String> {
        vec!["basic".to_string()]
    }
} 