// Resource actor reference system
//
// This module provides reference types for resource actors,
// allowing actors to reference and interact with each other.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};

use super::{ActorError, ActorResult};

/// Actor relationship type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Parent-child relationship
    ParentChild,
    
    /// Peer relationship
    Peer,
    
    /// Service relationship
    Service,
    
    /// Dependency relationship
    Dependency,
}

/// Actor relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorRelationship {
    /// Source actor
    pub source: ContentId,
    
    /// Target actor
    pub target: ContentId,
    
    /// Relationship type
    pub relationship_type: RelationshipType,
    
    /// Relationship metadata
    pub metadata: HashMap<String, String>,
    
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Actor reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorReference {
    /// Actor ID
    pub actor_id: ContentId,
    
    /// Actor type
    pub actor_type: String,
    
    /// Actor capabilities
    pub capabilities: Vec<ActorCapability>,
    
    /// Reference metadata
    pub metadata: HashMap<String, String>,
}

/// Actor capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorCapability {
    /// Capability name
    pub name: String,
    
    /// Allowed operations
    pub operations: Vec<String>,
    
    /// Capability metadata
    pub metadata: HashMap<String, String>,
}

/// Actor reference registry
#[async_trait]
pub trait ActorReferenceRegistry: Send + Sync + Debug {
    /// Get actor reference
    async fn get_reference(&self, actor_id: &ContentId) -> ActorResult<Option<ActorReference>>;
    
    /// Add actor reference
    async fn add_reference(&self, reference: ActorReference) -> ActorResult<()>;
    
    /// Remove actor reference
    async fn remove_reference(&self, actor_id: &ContentId) -> ActorResult<()>;
    
    /// Update actor reference
    async fn update_reference(&self, reference: ActorReference) -> ActorResult<()>;
    
    /// Get references by actor type
    async fn get_references_by_type(&self, actor_type: &str) -> ActorResult<Vec<ActorReference>>;
    
    /// Get references by relationship
    async fn get_references_by_relationship(
        &self,
        source_id: &ContentId,
        relationship_type: RelationshipType,
    ) -> ActorResult<Vec<ActorReference>>;
    
    /// Add relationship
    async fn add_relationship(&self, relationship: ActorRelationship) -> ActorResult<()>;
    
    /// Remove relationship
    async fn remove_relationship(
        &self,
        source_id: &ContentId,
        target_id: &ContentId,
        relationship_type: RelationshipType,
    ) -> ActorResult<()>;
    
    /// Get relationships by source
    async fn get_relationships_by_source(
        &self,
        source_id: &ContentId,
    ) -> ActorResult<Vec<ActorRelationship>>;
    
    /// Get relationships by target
    async fn get_relationships_by_target(
        &self,
        target_id: &ContentId,
    ) -> ActorResult<Vec<ActorRelationship>>;
} 