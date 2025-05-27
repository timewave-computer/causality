// Purpose: Provides type-safe registries for graph nodes and edges.
// This file was migrated from causality-types/src/graph/registry.rs

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet as StdHashSet}; // Renamed to avoid conflict
use std::marker::PhantomData;

// TODO: These imports will need to be updated to point to causality_types
use causality_types::{
    core::id::{NodeId, EdgeId, DomainId},
    graph::{
        traits::{AsNodeTypesList, AsEdgeTypesList, AsContainsNodeType, AsContainsEdgeType, GraphError},
    },
    AsNode, AsEdge,
    provider::registry::{AsRegistry},
    serialization::Encode,
};

use crate::smt::{TegMultiDomainSmt, MemoryBackend};

// Define GraphResult as a type alias
type GraphResult<T> = Result<T, GraphError>;

// Define type aliases for complex factory types
type NodeFactory = Box<dyn Fn(NodeId) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>;
type EdgeFactory = Box<dyn Fn(EdgeId) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>;

//-----------------------------------------------------------------------------
// Node Registry (Moved from causality-types)
//-----------------------------------------------------------------------------

/// Storage backend for NodeRegistry
pub enum NodeStorageBackend {
    /// Traditional HashMap storage
    HashMap(HashMap<NodeId, Box<dyn Any + Send + Sync>>),
    /// SMT-backed storage with domain awareness
    Smt(TegMultiDomainSmt<MemoryBackend>),
}

impl Default for NodeStorageBackend {
    fn default() -> Self {
        Self::HashMap(HashMap::new())
    }
}

/// A type-safe registry for nodes in a graph.
pub struct NodeRegistry<L: AsNodeTypesList> {
    /// Storage backend - either HashMap or SMT
    storage: NodeStorageBackend,
    /// Domain ID for SMT operations
    domain_id: Option<DomainId>,
    /// Map from type IDs to a function that can create nodes from NodeId
    factories: HashMap<TypeId, NodeFactory>,
    /// Phantom data for the type list
    _marker: PhantomData<L>,
}

impl<L: AsNodeTypesList> NodeRegistry<L> {
    /// Creates a new empty registry with HashMap storage
    pub fn new() -> Self {
        Self {
            storage: NodeStorageBackend::default(),
            domain_id: None,
            factories: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Creates a new registry with SMT storage
    pub fn new_with_smt(domain_id: DomainId) -> Self {
        let backend = MemoryBackend::new();
        let smt = TegMultiDomainSmt::new(backend);
        Self {
            storage: NodeStorageBackend::Smt(smt),
            domain_id: Some(domain_id),
            factories: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Returns the number of nodes in the registry
    pub fn len(&self) -> usize {
        match &self.storage {
            NodeStorageBackend::HashMap(map) => map.len(),
            NodeStorageBackend::Smt(_) => {
                // For SMT, we don't have a direct count
                // TODO: Implement count tracking in SMT
                0
            }
        }
    }

    /// Returns true if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Registers a node type so it can be constructed from a NodeId.
    /// Returns an error if the type is not in the type list L.
    pub fn register_type<T>(&mut self) -> GraphResult<()>
    where
        T: AsNode + Any + Clone + Send + Sync + 'static,
        L: AsContainsNodeType<T>,
    {
        if !L::is_present() {
            return Err(GraphError {
                message: format!(
                    "Type {} is not in the registry type list",
                    std::any::type_name::<T>()
                ),
            });
        }

        let type_id = TypeId::of::<T>();
        let factory = move |id: NodeId| -> Option<Box<dyn Any + Send + Sync>> {
            T::from_node_id(id)
                .map(|node| Box::new(node) as Box<dyn Any + Send + Sync>)
        };

        self.factories.insert(type_id, Box::new(factory));

        Ok(())
    }

    /// Registers a node in the registry.
    /// Returns an error if the node type is not registered.
    pub fn register_node<T>(
        &mut self,
        node: &T,
    ) -> GraphResult<NodeId>
    where
        T: AsNode + Any + Send + Sync + Clone + 'static + Encode,
        L: AsContainsNodeType<T>,
    {
        if !L::is_present() {
            return Err(GraphError {
                message: format!(
                    "Node type {:?} is not registered in this graph's NodeTypeList.",
                    TypeId::of::<T>()
                ),
            });
        }
        
        let node_id = node.to_node_id();
        
        match &mut self.storage {
            NodeStorageBackend::HashMap(map) => {
                if map.contains_key(&node_id) {
                    return Err(GraphError {
                        message: format!("Node with ID {:?} already exists.", node_id),
                    });
                }
                let node_clone_for_factory = node.clone(); // Clone once for the factory
                map.insert(node_id, Box::new(node.clone()));
                // The factory must own its data, so use the clone.
                self.factories.insert(
                    TypeId::of::<T>(),
                    Box::new(move |_| Some(Box::new(node_clone_for_factory.clone()))),
                );
            }
            NodeStorageBackend::Smt(smt) => {
                if let Some(domain_id) = &self.domain_id {
                    let node_data = node.as_ssz_bytes();
                    let node_id_str = node_id.to_string();
                    smt.store_teg_node(domain_id, "node", node_id_str.as_bytes(), &node_data).ok();
                    
                    // Also update factory for SMT storage
                    let node_clone_for_factory = node.clone();
                    self.factories.insert(
                        TypeId::of::<T>(),
                        Box::new(move |_| Some(Box::new(node_clone_for_factory.clone()))),
                    );
                } else {
                    return Err(GraphError {
                        message: "SMT storage requires a domain ID".to_string(),
                    });
                }
            }
        }
        
        Ok(node_id)
    }

    /// Gets a node from the registry.
    /// First checks if the node is in the registry, and if not, tries to
    /// construct it using the registered factory.
    pub fn get_node<T>(&self, id: NodeId) -> Option<T>
    where
        T: AsNode + Any + Clone + 'static,
        L: AsContainsNodeType<T>,
    {
        if !L::is_present() {
            return None;
        }

        match &self.storage {
            NodeStorageBackend::HashMap(map) => {
                // First try to get the node from the registry
                if let Some(boxed_node) = map.get(&id) {
                    if let Some(node) = boxed_node.downcast_ref::<T>() {
                        return Some(node.clone());
                    }
                }
            }
            NodeStorageBackend::Smt(smt) => {
                if let Some(domain_id) = &self.domain_id {
                    let node_id_str = id.to_string();
                    if let Ok(Some(_node_data)) = smt.get_teg_node(domain_id, "node", node_id_str.as_bytes()) {
                        // TODO: Deserialize from SSZ bytes back to T
                        // For now, try factory method
                        if let Some(node) = self.get_node(id) {
                            return Some(node);
                        }
                    }
                }
            }
        }

        // If not found, try to construct it using the factory
        let type_id = TypeId::of::<T>();
        if let Some(factory) = self.factories.get(&type_id) {
            if let Some(boxed_node) = factory(id) {
                if let Some(node) = boxed_node.downcast_ref::<T>() {
                    return Some(node.clone());
                }
            }
        }

        None
    }

    pub fn get_node_mut<T: AsNode + Any + Send + Sync>(
        &mut self,
        id: NodeId,
    ) -> GraphResult<&mut T>
    where
        L: AsContainsNodeType<T>,
    {
        if !L::is_present() {
            return Err(GraphError {
                message: format!(
                    "Node type {:?} is not registered in this graph's NodeTypeList.",
                    TypeId::of::<T>()
                ),
            });
        }

        match &mut self.storage {
            NodeStorageBackend::HashMap(map) => {
                if let Some(boxed_node) = map.get_mut(&id) {
                    if let Some(node) = boxed_node.downcast_mut::<T>() {
                        return Ok(node);
                    }
                }
            }
            NodeStorageBackend::Smt(_) => {
                // SMT storage doesn't support mutable references directly
                // TODO: Implement mutable access pattern for SMT
                return Err(GraphError {
                    message: "Mutable access not yet supported for SMT storage".to_string(),
                });
            }
        }

        Err(GraphError {
            message: format!(
                "Node with ID {:?} not found in the registry for mutable access or type mismatch",
                id
            ),
        })
    }

    /// Returns an iterator over the node IDs and their boxed Any trait objects.
    /// Note: For SMT storage, this currently returns an empty iterator.
    pub fn iter_nodes(&self) -> Vec<(NodeId, &(dyn Any + Send + Sync))> {
        match &self.storage {
            NodeStorageBackend::HashMap(map) => {
                map.iter()
                    .map(|(id, boxed_node)| (*id, &**boxed_node))
                    .collect()
            }
            NodeStorageBackend::Smt(_) => {
                // TODO: Implement iteration for SMT storage
                // For now, return empty vector
                Vec::new()
            }
        }
    }
}

impl<L: AsNodeTypesList> Default for NodeRegistry<L> {
    fn default() -> Self {
        Self::new()
    }
}

// AsRegistry Implementation for NodeRegistry
#[async_trait::async_trait]
impl<L: AsNodeTypesList + Send + Sync, T> AsRegistry<NodeId, T> for NodeRegistry<L>
where
    T: AsNode + Any + Clone + Send + Sync + 'static,
    L: AsContainsNodeType<T>,
{
    async fn register(&mut self, key: NodeId, value: T) -> anyhow::Result<()> {
        self.register_sync(key, value)
    }

    async fn get(&self, key: &NodeId) -> anyhow::Result<Option<T>> {
        Ok(self.get_sync(key)?)
    }

    async fn unregister(&mut self, key: &NodeId) -> anyhow::Result<()> {
        match &mut self.storage {
            NodeStorageBackend::HashMap(map) => {
                map.remove(key);
                Ok(())
            }
            NodeStorageBackend::Smt(_smt) => {
                // TODO: Implement SMT node removal
                Err(anyhow::anyhow!("SMT node removal not yet implemented"))
            }
        }
    }

    fn get_sync(&self, key: &NodeId) -> anyhow::Result<Option<T>> {
        match &self.storage {
            NodeStorageBackend::HashMap(map) => {
                match map
                    .get(key)
                    .and_then(|node| node.downcast_ref::<T>())
                {
                    Some(value) => Ok(Some(value.clone())),
                    None => Ok(None),
                }
            }
            NodeStorageBackend::Smt(smt) => {
                if let Some(domain_id) = &self.domain_id {
                    let node_id_str = key.to_string();
                    if let Ok(Some(_node_data)) = smt.get_teg_node(domain_id, "node", node_id_str.as_bytes()) {
                        // TODO: Deserialize from SSZ bytes back to T
                        // For now, try factory method
                        if let Some(node) = self.get_node(*key) {
                            return Ok(Some(node));
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    fn register_sync(&mut self, key: NodeId, value: T) -> anyhow::Result<()> {
        match &mut self.storage {
            NodeStorageBackend::HashMap(map) => {
                if map.contains_key(&key) {
                    return Err(anyhow::anyhow!("Node with ID {:?} already exists", key));
                }
                map.insert(key, Box::new(value));
            }
            NodeStorageBackend::Smt(smt) => {
                if let Some(domain_id) = &self.domain_id {
                    let node_id_str = key.to_string();
                    // Check if already exists
                    if let Ok(Some(_)) = smt.get_teg_node(domain_id, "node", node_id_str.as_bytes()) {
                        return Err(anyhow::anyhow!("Node with ID {:?} already exists", key));
                    }
                    // For SMT, we need the value to implement Encode
                    // This is a limitation - we'll store a placeholder for now
                    let placeholder_data = vec![0u8; 32]; // TODO: Serialize value properly
                    smt.store_teg_node(domain_id, "node", node_id_str.as_bytes(), &placeholder_data).ok();
                } else {
                    return Err(anyhow::anyhow!("SMT storage requires a domain ID"));
                }
            }
        }
        Ok(())
    }

    async fn contains(&self, key: &NodeId) -> anyhow::Result<bool> {
        Ok(match &self.storage {
            NodeStorageBackend::HashMap(map) => map.contains_key(key),
            NodeStorageBackend::Smt(smt) => {
                if let Some(domain_id) = &self.domain_id {
                    let node_id_str = key.to_string();
                    smt.get_teg_node(domain_id, "node", node_id_str.as_bytes()).is_ok_and(|opt| opt.is_some())
                } else {
                    false
                }
            }
        })
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(match &self.storage {
            NodeStorageBackend::HashMap(map) => map.len(),
            NodeStorageBackend::Smt(_) => {
                // TODO: Implement count for SMT
                0
            }
        })
    }

    async fn clear(&mut self) -> anyhow::Result<()> {
        match &mut self.storage {
            NodeStorageBackend::HashMap(map) => {
                map.clear();
            }
            NodeStorageBackend::Smt(_) => {
                // TODO: Implement SMT clear
                return Err(anyhow::anyhow!("SMT clear not yet implemented"));
            }
        }
        self.factories.clear(); // Also clear factories
        Ok(())
    }
}

// Debug impl for NodeRegistry
impl<L: AsNodeTypesList> std::fmt::Debug for NodeRegistry<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRegistry")
            .field("num_nodes", &self.len())
            .field("num_factories", &self.factories.len())
            .finish()
    }
}

//-----------------------------------------------------------------------------
// Edge Registry (Moved from causality-types)
//-----------------------------------------------------------------------------

/// A type-safe registry for edges in a graph.
pub struct EdgeRegistry<L: AsEdgeTypesList> {
    /// Map from edge IDs to edge objects
    edges: HashMap<EdgeId, Box<dyn Any + Send + Sync>>,
    /// Map from type IDs to a function that can create edges from EdgeId
    factories: HashMap<TypeId, EdgeFactory>,
    /// Map from source node to target nodes and their edge IDs
    outgoing: HashMap<NodeId, HashMap<NodeId, StdHashSet<EdgeId>>>,
    /// Map from target node to source nodes and their edge IDs
    incoming: HashMap<NodeId, HashMap<NodeId, StdHashSet<EdgeId>>>,
    /// Phantom data for the type list
    _marker: PhantomData<L>,
}

impl<L: AsEdgeTypesList> EdgeRegistry<L> {
    /// Creates a new empty registry
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            factories: HashMap::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Returns the number of edges in the registry
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Returns true if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Registers an edge type so it can be constructed from an EdgeId.
    /// Returns an error if the type is not in the type list L.
    pub fn register_type<T>(&mut self) -> GraphResult<()>
    where
        T: AsEdge + Any + Clone + Send + Sync + 'static,
        L: AsContainsEdgeType<T>,
    {
        if !L::is_present() {
            return Err(GraphError {
                message: format!(
                    "Type {} is not in the registry type list",
                    std::any::type_name::<T>()
                ),
            });
        }

        let type_id = TypeId::of::<T>();
        let factory = move |id: EdgeId| -> Option<Box<dyn Any + Send + Sync>> {
            T::from_edge_id(id)
                .map(|edge| Box::new(edge) as Box<dyn Any + Send + Sync>)
        };

        self.factories.insert(type_id, Box::new(factory));

        Ok(())
    }

    /// Registers an edge in the registry.
    /// Returns an error if the edge type is not registered.
    pub fn register_edge<T: AsEdge + Any + Send + Sync + Clone + 'static>(
        &mut self,
        edge: &T,
    ) -> GraphResult<EdgeId>
    where
        L: AsContainsEdgeType<T>,
    {
        if !L::is_present() {
            return Err(GraphError {
                message: format!(
                    "Edge type {:?} is not registered in this graph's EdgeTypeList.",
                    TypeId::of::<T>()
                ),
            });
        }
        let edge_id = edge.to_edge_id();
        if self.edges.contains_key(&edge_id) {
            return Err(GraphError {
                message: format!("Edge with ID {:?} already exists.", edge_id),
            });
        }

        let source_node_id = edge.source();
        let target_node_id = edge.target();

        self.edges.insert(edge_id, Box::new(edge.clone()));
        let edge_clone_for_factory = edge.clone(); // Clone once for the factory
        self.factories.insert(
            TypeId::of::<T>(),
            Box::new(move |_| Some(Box::new(edge_clone_for_factory.clone()))),
        );

        self.outgoing
            .entry(source_node_id)
            .or_default()
            .entry(target_node_id)
            .or_default()
            .insert(edge_id);

        self.incoming
            .entry(target_node_id)
            .or_default()
            .entry(source_node_id)
            .or_default()
            .insert(edge_id);

        Ok(edge_id)
    }

    /// Gets an edge from the registry.
    /// First checks if the edge is in the registry, and if not, tries to
    /// construct it using the registered factory.
    pub fn get_edge<T>(&self, id: EdgeId) -> Option<T>
    where
        T: AsEdge + Any + Clone + 'static,
        L: AsContainsEdgeType<T>,
    {
        if !L::is_present() {
            return None;
        }

        if let Some(boxed_edge) = self.edges.get(&id) {
            if let Some(edge) = boxed_edge.downcast_ref::<T>() {
                return Some(edge.clone());
            }
        }

        let type_id = TypeId::of::<T>();
        if let Some(factory) = self.factories.get(&type_id) {
            if let Some(boxed_edge) = factory(id) {
                if let Some(edge) = boxed_edge.downcast_ref::<T>() {
                    return Some(edge.clone());
                }
            }
        }
        None
    }

    pub fn get_edge_mut<T: AsEdge + Any + Send + Sync>(
        &mut self,
        id: EdgeId,
    ) -> GraphResult<&mut T>
    where
        L: AsContainsEdgeType<T>,
    {
        if !L::is_present() {
            return Err(GraphError {
                message: format!(
                    "Edge type {:?} is not registered in this graph's EdgeTypeList.",
                    TypeId::of::<T>()
                ),
            });
        }

        if let Some(boxed_edge) = self.edges.get_mut(&id) {
            if let Some(edge) = boxed_edge.downcast_mut::<T>() {
                return Ok(edge);
            }
        }
        Err(GraphError {
            message: format!(
                "Edge with ID {:?} not found for mutable access or type mismatch",
                id
            ),
        })
    }

    /// Returns a list of IDs for edges originating from the source node.
    pub fn get_outgoing_edges(&self, source: NodeId) -> Vec<EdgeId> {
        self.outgoing.get(&source).map_or_else(Vec::new, |targets| {
            targets.values().flatten().copied().collect()
        })
    }

    /// Returns a list of IDs for edges terminating at the target node.
    pub fn get_incoming_edges(&self, target: NodeId) -> Vec<EdgeId> {
        self.incoming.get(&target).map_or_else(Vec::new, |sources| {
            sources.values().flatten().copied().collect()
        })
    }

    /// Returns a list of IDs for edges between a source and target node.
    pub fn get_edges_between(&self, source: NodeId, target: NodeId) -> Vec<EdgeId> {
        self.outgoing
            .get(&source)
            .and_then(|targets| targets.get(&target))
            .map_or_else(Vec::new, |edge_ids| edge_ids.iter().copied().collect())
    }
}

impl<L: AsEdgeTypesList> Default for EdgeRegistry<L> {
    fn default() -> Self {
        Self::new()
    }
}

// AsRegistry Implementation for EdgeRegistry
#[async_trait::async_trait]
impl<L: AsEdgeTypesList + Send + Sync, T> AsRegistry<EdgeId, T> for EdgeRegistry<L>
where
    T: AsEdge + Any + Clone + Send + Sync + 'static,
    L: AsContainsEdgeType<T>,
{
    async fn register(&mut self, key: EdgeId, value: T) -> anyhow::Result<()> {
        self.register_sync(key, value)
    }

    async fn get(&self, key: &EdgeId) -> anyhow::Result<Option<T>> {
        Ok(self.get_sync(key)?)
    }

    async fn unregister(&mut self, key: &EdgeId) -> anyhow::Result<()> {
        self.edges.remove(key);
        // Also clean up the edge from outgoing/incoming maps
        // This is a simplified version - a full implementation would need to track source/target
        // TODO: Remove from outgoing/incoming maps properly
        Ok(())
    }

    fn get_sync(&self, key: &EdgeId) -> anyhow::Result<Option<T>> {
        match self
            .edges
            .get(key)
            .and_then(|edge| edge.downcast_ref::<T>())
        {
            Some(value) => Ok(Some(value.clone())),
            None => Ok(None),
        }
    }

    fn register_sync(&mut self, id: EdgeId, value: T) -> anyhow::Result<()> {
        if self.edges.contains_key(&id) {
            return Err(anyhow::anyhow!("Edge with ID {:?} already exists", id));
        }

        // Store the edge
        self.edges.insert(id, Box::new(value.clone()));

        // Update outgoing/incoming maps
        let source_id = value.source();
        let target_id = value.target();

        self.outgoing
            .entry(source_id)
            .or_default()
            .entry(target_id)
            .or_default()
            .insert(id);

        self.incoming
            .entry(target_id)
            .or_default()
            .entry(source_id)
            .or_default()
            .insert(id);

        Ok(())
    }

    async fn contains(&self, key: &EdgeId) -> anyhow::Result<bool> {
        Ok(self.edges.contains_key(key))
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.edges.len())
    }

    async fn clear(&mut self) -> anyhow::Result<()> {
        self.edges.clear();
        self.outgoing.clear();
        self.incoming.clear();
        self.factories.clear();
        Ok(())
    }
}

// Debug impl for EdgeRegistry
impl<L: AsEdgeTypesList> std::fmt::Debug for EdgeRegistry<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeRegistry")
            .field("num_edges", &self.edges.len())
            .field("num_factories", &self.factories.len())
            .field("num_outgoing_keys", &self.outgoing.len())
            .field("num_incoming_keys", &self.incoming.len())
            .finish()
    }
}
