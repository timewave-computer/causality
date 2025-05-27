// Purpose: SMT-based collection types to replace HashMap/BTreeMap in content-addressable storage

use causality_types::{
    primitive::ids::{DomainId, EdgeId, EntityId, ExprId, NodeId, AsId},
    system::serialization::{Encode, Decode},
    effect::{Effect, Intent, Handler},
    graph::tel::{ResourceRef, Edge},
    graph::r#trait::AsEdge,
    expression::value::ValueExpr,
};

use crate::smt::{TegMultiDomainSmt, MemoryBackend};
use crate::utils::serialization::SmtCollection;
use std::{
    sync::Arc,
};
use parking_lot::Mutex;
use anyhow::Result;
use hex;

//-----------------------------------------------------------------------------
// TEL Node Enum for SMT Storage
//-----------------------------------------------------------------------------

/// Unified node type for SMT storage in project compilation
#[derive(Debug, Clone, PartialEq)]
pub enum TelNode {
    Effect(Effect),
    Resource(ResourceRef),
    Intent(Intent),
    Handler(Handler),
}

impl Encode for TelNode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            TelNode::Effect(effect) => {
                bytes.push(0u8); // discriminant
                bytes.extend(effect.as_ssz_bytes());
            }
            TelNode::Resource(resource) => {
                bytes.push(1u8); // discriminant
                bytes.extend(resource.as_ssz_bytes());
            }
            TelNode::Intent(intent) => {
                bytes.push(2u8); // discriminant
                bytes.extend(intent.as_ssz_bytes());
            }
            TelNode::Handler(handler) => {
                bytes.push(3u8); // discriminant
                bytes.extend(handler.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for TelNode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.is_empty() {
            return Err(causality_types::serialization::DecodeError {
                message: "Empty bytes for TelNode".to_string(),
            });
        }
        
        let discriminant = bytes[0];
        let data = &bytes[1..];
        
        match discriminant {
            0 => {
                let effect = Effect::from_ssz_bytes(data)?;
                Ok(TelNode::Effect(effect))
            }
            1 => {
                let resource = ResourceRef::from_ssz_bytes(data)?;
                Ok(TelNode::Resource(resource))
            }
            2 => {
                let intent = Intent::from_ssz_bytes(data)?;
                Ok(TelNode::Intent(intent))
            }
            3 => {
                let handler = Handler::from_ssz_bytes(data)?;
                Ok(TelNode::Handler(handler))
            }
            _ => Err(causality_types::serialization::DecodeError {
                message: format!("Invalid discriminant for TelNode: {}", discriminant),
            }),
        }
    }
}

//-----------------------------------------------------------------------------
// SMT-Based Collection Types
//-----------------------------------------------------------------------------

/// SMT-based expression collection
pub type SmtExpressionCollection = SmtCollection<ExprId, ValueExpr>;

/// SMT-based node collection for TEL nodes
pub type SmtNodeCollection = SmtCollection<NodeId, TelNode>;

/// SMT-based edge collection
pub type SmtEdgeCollection = SmtCollection<EdgeId, Edge>;

/// SMT-based effect collection (using EntityId as unified identifier)
pub type SmtEffectCollection = SmtCollection<EntityId, Effect>;

/// SMT-based resource collection (using EntityId as unified identifier)
pub type SmtResourceCollection = SmtCollection<EntityId, ResourceRef>;

/// SMT-based intent collection (using EntityId as unified identifier)
pub type SmtIntentCollection = SmtCollection<EntityId, Intent>;

/// SMT-based handler collection (using EntityId as unified identifier)
pub type SmtHandlerCollection = SmtCollection<EntityId, Handler>;

// TODO: Add SmtSubgraphCollection when Subgraph implements Encode/Decode traits
// pub type SmtSubgraphCollection = SmtCollection<SubgraphId, Subgraph>;

//-----------------------------------------------------------------------------
// Project SMT Storage
//-----------------------------------------------------------------------------

/// SMT-based storage for project compilation data
#[derive(Debug)]
pub struct ProjectSmtStorage {
    smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
    domain_id: DomainId,
    
    // SMT collections for different data types
    pub expressions: SmtExpressionCollection,
    pub nodes: SmtNodeCollection,
    pub edges: SmtEdgeCollection,
    pub effects: SmtEffectCollection,
    pub resources: SmtResourceCollection,
    pub intents: SmtIntentCollection,
    pub handlers: SmtHandlerCollection,
    // TODO: Add subgraphs when Subgraph implements Encode/Decode
    // pub subgraphs: SmtSubgraphCollection,
}

impl ProjectSmtStorage {
    /// Create new project SMT storage
    pub fn new(domain_id: DomainId) -> Self {
        let backend = MemoryBackend::new();
        let smt = Arc::new(Mutex::new(TegMultiDomainSmt::new(backend)));
        
        Self {
            expressions: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "expressions"),
            nodes: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "nodes"),
            edges: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "edges"),
            effects: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "effects"),
            resources: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "resources"),
            intents: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "intents"),
            handlers: SmtCollection::with_smt(Arc::clone(&smt), domain_id, "handlers"),
            smt,
            domain_id,
        }
    }
    
    /// Get the shared SMT instance
    pub fn get_smt(&self) -> Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>> {
        Arc::clone(&self.smt)
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }
    
    /// Add a TEL node to the appropriate collection
    pub fn add_tel_node(&self, node: TelNode) -> Result<NodeId> {
        let node_id = match &node {
            TelNode::Effect(effect) => {
                self.effects.insert(effect.id, effect.clone())?;
                NodeId::new(effect.id.inner()) // Convert EntityId to NodeId
            }
            TelNode::Resource(resource) => {
                let entity_id = EntityId::new(resource.resource_id.inner()); // Convert ResourceId to EntityId
                self.resources.insert(entity_id, resource.clone())?;
                NodeId::new(entity_id.inner()) // Convert EntityId to NodeId
            }
            TelNode::Intent(intent) => {
                self.intents.insert(intent.id, intent.clone())?;
                NodeId::new(intent.id.inner()) // Convert EntityId to NodeId
            }
            TelNode::Handler(handler) => {
                self.handlers.insert(handler.id, handler.clone())?;
                NodeId::new(handler.id.inner()) // Convert EntityId to NodeId
            }
        };
        
        // Also store in the unified nodes collection
        self.nodes.insert(node_id, node)?;
        
        Ok(node_id)
    }
    
    /// Add an edge to the collection
    pub fn add_edge(&self, edge: Edge) -> Result<EdgeId> {
        let edge_id = edge.to_edge_id();
        self.edges.insert(edge_id, edge)?;
        Ok(edge_id)
    }
    
    /// Get a node by ID
    pub fn get_node(&self, node_id: &NodeId) -> Result<Option<TelNode>> {
        if let Some(node_data) = self.nodes.get(node_id)? {
            Ok(Some(node_data.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// Get an effect by ID
    pub fn get_effect(&self, effect_id: &EntityId) -> Result<Option<Effect>> {
        Ok(self.effects.get(effect_id)?)
    }
    
    /// Get a resource by ID
    pub fn get_resource(&self, resource_id: &EntityId) -> Result<Option<ResourceRef>> {
        if let Some(resource_data) = self.resources.get(resource_id)? {
            if resource_data.resource_id == causality_types::ResourceId(resource_id.inner()) {
                return Ok(Some(resource_data.clone()));
            }
        }
        Ok(None)
    }
    
    /// Get an intent by ID
    pub fn get_intent(&self, intent_id: &EntityId) -> Result<Option<Intent>> {
        Ok(self.intents.get(intent_id)?)
    }
    
    /// Get a handler by ID
    pub fn get_handler(&self, handler_id: &EntityId) -> Result<Option<Handler>> {
        Ok(self.handlers.get(handler_id)?)
    }
    
    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: &EdgeId) -> Result<Option<Edge>> {
        Ok(self.edges.get(edge_id)?)
    }
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Convert HashMap collections to SMT collections
pub fn migrate_hashmap_to_smt(
    domain_id: DomainId,
    expressions: std::collections::HashMap<ExprId, ValueExpr>,
    effects: std::collections::HashMap<EntityId, Effect>,
    resources: std::collections::HashMap<EntityId, ResourceRef>,
    intents: std::collections::HashMap<EntityId, Intent>,
    handlers: std::collections::HashMap<EntityId, Handler>,
    edges: std::collections::HashMap<EdgeId, Edge>,
    // TODO: Add subgraphs parameter when Subgraph implements Encode/Decode
    // subgraphs: std::collections::HashMap<SubgraphId, Subgraph>,
) -> Result<ProjectSmtStorage> {
    let storage = ProjectSmtStorage::new(domain_id);
    
    // Migrate expressions
    for (id, expr) in expressions {
        storage.expressions.insert(id, expr)?;
    }
    
    // Migrate effects
    for (id, effect) in effects {
        let entity_id = EntityId::new(id.inner()); // Convert EffectId to EntityId
        storage.effects.insert(entity_id, effect.clone())?;
        storage.nodes.insert(NodeId::new(effect.id.inner()), TelNode::Effect(effect))?;
    }
    
    // Migrate resources
    for (id, resource) in resources {
        storage.resources.insert(id, resource.clone())?;
        let node_id = NodeId::new(id.inner());
        storage.nodes.insert(node_id, TelNode::Resource(resource))?;
    }
    
    // Migrate intents
    for (id, intent) in intents {
        storage.intents.insert(id, intent.clone())?;
        storage.nodes.insert(NodeId::new(intent.id.inner()), TelNode::Intent(intent))?;
    }
    
    // Migrate handlers
    for (id, handler) in handlers {
        storage.handlers.insert(id, handler.clone())?;
        storage.nodes.insert(NodeId::new(handler.id.inner()), TelNode::Handler(handler))?;
    }
    
    // Migrate edges
    for (id, edge) in edges {
        storage.edges.insert(id, edge)?;
    }
    
    // TODO: Migrate subgraphs when Subgraph implements Encode/Decode
    // for (id, subgraph) in subgraphs {
    //     storage.subgraphs.insert(id, subgraph)?;
    // }
    
    Ok(storage)
}

impl SmtResourceCollection {
    /// Get all resources in a specific domain
    pub fn get_resources_in_domain(&self, domain_id: &[u8]) -> Result<Vec<ResourceRef>> {
        // For now, return empty vector since we can't iterate over SMT keys efficiently
        // In a full implementation, this would use SMT range queries or indexing
        let _domain_prefix = format!("domain:{}", hex::encode(domain_id));
        Ok(Vec::new())
    }

    /// Count resources in a specific domain
    pub fn count_resources_in_domain(&self, _domain_id: &[u8]) -> usize {
        // For now, return 0 since we can't iterate over SMT keys efficiently
        // In a full implementation, this would use SMT range queries or indexing
        0
    }
}

impl SmtEffectCollection {
    /// Get all effects in a specific domain
    pub fn get_effects_in_domain(&self, domain_id: &[u8]) -> Result<Vec<Effect>> {
        // For now, return empty vector since we can't iterate over SMT keys efficiently
        // In a full implementation, this would use SMT range queries or indexing
        let _domain_prefix = format!("domain:{}", hex::encode(domain_id));
        Ok(Vec::new())
    }
}

impl SmtIntentCollection {
    /// Get all intents in a specific domain
    pub fn get_intents_in_domain(&self, domain_id: &[u8]) -> Result<Vec<Intent>> {
        // For now, return empty vector since we can't iterate over SMT keys efficiently
        // In a full implementation, this would use SMT range queries or indexing
        let _domain_prefix = format!("domain:{}", hex::encode(domain_id));
        Ok(Vec::new())
    }
}

impl SmtHandlerCollection {
    /// Get all handlers in a specific domain
    pub fn get_handlers_in_domain(&self, domain_id: &[u8]) -> Result<Vec<Handler>> {
        // For now, return empty vector since we can't iterate over SMT keys efficiently
        // In a full implementation, this would use SMT range queries or indexing
        let _domain_prefix = format!("domain:{}", hex::encode(domain_id));
        Ok(Vec::new())
    }
}