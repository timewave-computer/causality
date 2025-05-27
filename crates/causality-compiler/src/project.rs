//! Program Project - SMT-backed unified project compilation system

use std::any::Any;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log;

use causality_types::{
    core::{Effect, Handler, Intent},
    tel::graph::Edge,
    graph::traits::AsNode,
    core::id::{DomainId, EdgeId, NodeId},
    tel::common_refs::ResourceRef,

};

use crate::ids::{ProgramId};
use crate::program::Program;
use crate::registry::ProgramRegistry;
use causality_core::smt_collections::{ProjectSmtStorage, TelNode as SmtTelNode};

//-----------------------------------------------------------------------------
// Collection Type Aliases (SMT-backed only)
//-----------------------------------------------------------------------------

// Re-export SMT collection types for convenience
pub use causality_core::smt_collections::{
    SmtExpressionCollection, SmtNodeCollection, SmtEdgeCollection,
    SmtEffectCollection, SmtResourceCollection, SmtIntentCollection, SmtHandlerCollection
};

//-----------------------------------------------------------------------------
// ProgramProject Definition (SMT-only)
//-----------------------------------------------------------------------------

/// Project for assembling a multi-domain program using SMT-backed storage
#[derive(Debug)]
pub struct ProgramProject {
    pub name: String,
    pub root_path: Option<PathBuf>,
    pub storage: ProjectSmtStorage,
    pub program_id: Option<ProgramId>,
}

impl Default for ProgramProject {
    fn default() -> Self {
        // Default domain ID
        let default_domain = DomainId::new([0u8; 32]);
        ProgramProject {
            name: String::new(),
            root_path: None,
            storage: ProjectSmtStorage::new(default_domain),
            program_id: None,
        }
    }
}

//-----------------------------------------------------------------------------
// ProgramProject Implementation (SMT-only)
//-----------------------------------------------------------------------------

impl ProgramProject {
    /// Create a new SMT-backed program project
    pub fn new(name: String, root_path: Option<PathBuf>, domain_id: DomainId) -> Self {
        ProgramProject {
            name,
            root_path,
            storage: ProjectSmtStorage::new(domain_id),
            program_id: None,
        }
    }

    /// Create a new project with just a name
    pub fn with_name(name: &str) -> Self {
        Self::new(name.to_string(), None, DomainId::new([0u8; 32]))
    }

    /// Get the domain ID for this project
    pub fn domain_id(&self) -> DomainId {
        self.storage.domain_id()
    }

    /// Add a TEL node with automatic type detection and SMT storage
    pub fn add_tel_node<N: AsNode + Any + Clone>(
        &mut self,
        node: &N,
    ) -> Result<NodeId> {
        let node_id = node.to_node_id();

        if let Some(effect_node) = (node as &dyn Any).downcast_ref::<Effect>() {
            let smt_node = SmtTelNode::Effect(effect_node.clone());
            self.storage.add_tel_node(smt_node)?;
        } else if let Some(resource_node) = (node as &dyn Any).downcast_ref::<ResourceRef>() {
            let smt_node = SmtTelNode::Resource(*resource_node);
            self.storage.add_tel_node(smt_node)?;
        } else if let Some(intent_node) = (node as &dyn Any).downcast_ref::<Intent>() {
            let smt_node = SmtTelNode::Intent(intent_node.clone());
            self.storage.add_tel_node(smt_node)?;
        } else if let Some(handler_node) = (node as &dyn Any).downcast_ref::<Handler>() {
            let smt_node = SmtTelNode::Handler(handler_node.clone());
            self.storage.add_tel_node(smt_node)?;
        } else {
            return Err(anyhow::anyhow!(
                "Failed to downcast node {:?} to a known TEL type",
                node_id
            ));
        }
        Ok(node_id)
    }

    /// Add a TEL edge with SMT storage
    pub fn add_tel_edge(&mut self, edge: &Edge) -> Result<EdgeId> {
        // Verify source and target nodes exist
        if self.storage.get_node(&edge.source)?.is_none() {
            return Err(anyhow::anyhow!(
                "Source node {} not found for edge {}",
                edge.source,
                edge.id
            ));
        }
        if self.storage.get_node(&edge.target)?.is_none() {
            return Err(anyhow::anyhow!(
                "Target node {} not found for edge {}",
                edge.target,
                edge.id
            ));
        }
        
        self.storage.add_edge(edge.clone())?;
        log::debug!("Added TEL edge: {:?}", edge);
        Ok(edge.id)
    }

    /// Get a node as an Effect
    pub fn try_as_effect(&self, node_id: &NodeId) -> Result<Option<Effect>> {
        if let Some(SmtTelNode::Effect(effect)) = self.storage.get_node(node_id)? {
            Ok(Some(effect))
        } else {
            Ok(None)
        }
    }

    /// Get a node as a Resource
    pub fn try_as_resource(&self, node_id: &NodeId) -> Result<Option<ResourceRef>> {
        if let Some(SmtTelNode::Resource(resource)) = self.storage.get_node(node_id)? {
            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    /// Get a node as an Intent
    pub fn try_as_intent(&self, node_id: &NodeId) -> Result<Option<Intent>> {
        if let Some(SmtTelNode::Intent(intent)) = self.storage.get_node(node_id)? {
            Ok(Some(intent))
        } else {
            Ok(None)
        }
    }

    /// Get a node as a Handler
    pub fn try_as_handler(&self, node_id: &NodeId) -> Result<Option<Handler>> {
        if let Some(SmtTelNode::Handler(handler)) = self.storage.get_node(node_id)? {
            Ok(Some(handler))
        } else {
            Ok(None)
        }
    }

    /// Generate a program from the current project state
    pub fn generate_program(
        &mut self,
        registry: &mut ProgramRegistry,
    ) -> Result<ProgramId> {
        let prog_id = self.program_id.unwrap_or(ProgramId([0u8; 32])); 
        self.program_id = Some(prog_id);

        // Create a minimal program with the generated ID
        let program = Program {
            id: prog_id,
            circuit_ids: std::collections::HashSet::new(), // Empty for now since we're removing circuits
        };

        registry
            .insert(program.id, program)
            .map_err(|e| anyhow::anyhow!("Failed to insert program: {}", e))
            .with_context(|| format!("Failed to register program with ID {:?}", prog_id))?;

        Ok(prog_id)
    }
}
