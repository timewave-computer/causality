//! Purpose: Transforms a ParsedTegProgram into a fully linked and compiled TEG representation.

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use sha2::Digest; // Import the Digest trait for SHA256

use causality_types::{
    core::{Handler as TypesHandler},
    core::id::{NodeId, ExprId, EntityId, HandlerId, AsId},
    expr::{value::ValueExpr, ast::Expr},
    graph::elements::{Node, Edge as TypesEdge},
    serialization::Encode, // Added for as_ssz_bytes() method
};

use crate::teg_parser::ParsedTegProgram;
use crate::ids::id_from_sha256;

// Type aliases for compatibility
type ProgramId = EntityId;
type SubgraphId = EntityId;

#[derive(Debug, Clone)]
pub struct CompiledTeg {
    pub id: ProgramId,
    pub name: String, // The original name from (define-teg <name> ...)
    pub base_dir: PathBuf,

    pub expressions: HashMap<ExprId, Expr>,
    pub handlers: HashMap<HandlerId, TypesHandler>,
    pub subgraphs: HashMap<SubgraphId, CompiledSubgraph>,
    // Consider adding main_subgraph_id or similar if TEGs have a single entry point concept
}

impl Default for CompiledTeg {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledTeg {
    /// Creates a new, empty CompiledTeg.
    pub fn new() -> Self {
        CompiledTeg {
            id: ProgramId::default(),
            name: String::new(),
            base_dir: PathBuf::new(),
            expressions: HashMap::new(),
            handlers: HashMap::new(),
            subgraphs: HashMap::new(),
        }
    }
}

// Manual implementation of SSZ for CompiledTeg to handle PathBuf
impl causality_types::serialization::Encode for CompiledTeg {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.id.as_ssz_bytes());
        result.extend(self.name.as_ssz_bytes());
        
        // Convert PathBuf to string for serialization
        let base_dir_str = self.base_dir.to_string_lossy().to_string();
        result.extend(base_dir_str.as_ssz_bytes());
        
        result.extend(self.expressions.as_ssz_bytes());
        result.extend(self.handlers.as_ssz_bytes());
        result.extend(self.subgraphs.as_ssz_bytes());
        
        result
    }
}

#[derive(Debug, Clone)]
pub struct CompiledSubgraph {
    pub id: SubgraphId,
    pub name: String,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<TypesEdge>,
    pub entry_nodes: Vec<NodeId>,
    pub exit_nodes: Vec<NodeId>,
    pub metadata: HashMap<String, ValueExpr>,
    pub static_checks: Vec<ExprId>, // Subgraph-level static checks
}

impl causality_types::serialization::Encode for CompiledSubgraph {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.id.as_ssz_bytes());
        result.extend(self.name.as_ssz_bytes());
        result.extend(self.nodes.as_ssz_bytes());
        result.extend(self.edges.as_ssz_bytes());
        result.extend(self.entry_nodes.as_ssz_bytes());
        result.extend(self.exit_nodes.as_ssz_bytes());
        result.extend(self.metadata.as_ssz_bytes());
        result.extend(self.static_checks.as_ssz_bytes());
        result
    }
}

pub fn ingest_parsed_teg(
    parsed_program: ParsedTegProgram,
    program_name_from_cli: Option<String>, // Optional name if not in file or to override
) -> Result<CompiledTeg> {
    let program_name_str = parsed_program
        .program_name
        .clone()
        .or(program_name_from_cli)
        .unwrap_or_else(|| "unnamed-teg-program".to_string());

    let program_id = parsed_program
        .id
        .unwrap_or_else(|| id_from_sha256(program_name_str.as_bytes()));

    let mut compiled_expressions = parsed_program.global_expressions;
    let mut compiled_handlers: HashMap<HandlerId, TypesHandler> = HashMap::new();
    let mut compiled_subgraphs = HashMap::new();

    // 1. Ingest Handlers
    // The parser stores handlers in `defined_handlers: HashMap<String, TypesHandler>`
    // where TypesHandler is `causality_types::tel::handler::Handler`.
    // `tel::handler::Handler` has an `id()` method to get its `HandlerId`.
    for (_name, handler_data) in parsed_program.defined_handlers {
        // Convert EntityId to HandlerId
        let handler_id = HandlerId::new(handler_data.id.inner());
        if compiled_handlers
            .insert(handler_id, handler_data.clone())
            .is_some()
        {
            // This case should ideally not happen if HandlerIds are content-addressed and unique.
            // If names can map to identical handlers, this is fine.
            // If names map to different handlers that hash to the same ID, that's a collision problem (unlikely with Sha256).
            log::warn!("Handler ID {:?} (from name \"{}\") was already present. Overwriting.", handler_id, _name);
        }
    }

    // 2. Ingest Subgraphs
    for subgraph_data in parsed_program.subgraphs {
        // Create deterministic SubgraphId based on program_id and subgraph name
        let mut hasher = sha2::Sha256::new();
        hasher.update(program_id.as_ssz_bytes());
        hasher.update(subgraph_data.name.as_bytes());
        let result = hasher.finalize();
        let subgraph_id = SubgraphId::new(result.into());

        let mut current_subgraph_nodes = HashMap::new();
        for node in &subgraph_data.nodes {
            // Convert Resource to Node
            let node_id = NodeId::new(node.id.inner());
            let graph_node = Node::new(
                node_id,
                causality_types::graph::elements::TypeId::from_string(&node.resource_type.to_string())
            );
            current_subgraph_nodes.insert(node_id, graph_node);
        }

        for (expr_id, expr) in subgraph_data.subgraph_specific_expressions {
            compiled_expressions.insert(expr_id, expr);
        }
        let static_checks_expr_ids: Vec<ExprId> =
            subgraph_data.subgraph_specific_expressions_ids;

        // Resolve entry/exit node names to NodeIds using the map from the parser.
        let mut entry_node_ids = Vec::new();
        for name in &subgraph_data.entry_nodes {
            if let Some(node_id) = subgraph_data.effect_name_to_node_id_map.get(name)
            {
                entry_node_ids.push(*node_id);
            } else {
                // This case should ideally be caught by the parser if an entry/exit node name doesn't correspond to a defined effect.
                // However, a check here provides an additional layer of safety during ingestion.
                return Err(anyhow!("Entry node name '{}' in subgraph '{}' not found in effect_name_to_node_id_map.", name, subgraph_data.name));
            }
        }

        let mut exit_node_ids = Vec::new();
        for name in &subgraph_data.exit_nodes {
            if let Some(node_id) = subgraph_data.effect_name_to_node_id_map.get(name)
            {
                exit_node_ids.push(*node_id);
            } else {
                return Err(anyhow!("Exit node name '{}' in subgraph '{}' not found in effect_name_to_node_id_map.", name, subgraph_data.name));
            }
        }

        //-----------------------------------------------------------------------------
        // Process metadata with clean SSZ-compatible approach
        //-----------------------------------------------------------------------------
        let mut metadata_map = HashMap::new();
        
        if let lexpr::Value::Cons(cons) = &subgraph_data.metadata {
            // Process metadata entries by iterating through cons cells
            let (items, _) = cons.to_vec();
            
            for item in items {
                if let lexpr::Value::Cons(pair_cons) = item {
                    let (key, value) = pair_cons.as_pair();
                    
                    if let Some(key_str) = key.as_str()
                        .or_else(|| key.as_symbol())
                        .or_else(|| key.as_keyword())
                    {
                        match crate::teg_parser::lexpr_value_to_value_expr(value) {
                            Ok(val_expr) => { 
                                metadata_map.insert(key_str.to_string(), val_expr); 
                            },
                            Err(e) => return Err(anyhow!(
                                "Failed to convert metadata value for key '{}' in subgraph '{}': {}", 
                                key_str, subgraph_data.name, e
                            )),
                        }
                    } else {
                        log::warn!(
                            "Skipping metadata with non-string/symbol/keyword key in subgraph '{}': {:?}", 
                            subgraph_data.name, key
                        );
                    }
                }
            }
        } else if !subgraph_data.metadata.is_nil() && !subgraph_data.metadata.is_null() {
            return Err(anyhow!(
                "Subgraph '{}' metadata must be an object (map) or nil. Found: {:?}",
                subgraph_data.name,
                subgraph_data.metadata
            ));
        }

        // Convert tel::Edge to graph::elements::Edge
        let converted_edges: Vec<TypesEdge> = subgraph_data.edges.iter().map(|tel_edge| {
            TypesEdge::new(
                tel_edge.id,
                causality_types::graph::elements::TypeId::from_string("default_edge_type"),
                NodeId::new(tel_edge.source.inner()),
                NodeId::new(tel_edge.target.inner())
            )
        }).collect();

        compiled_subgraphs.insert(
            subgraph_id,
            CompiledSubgraph {
                id: subgraph_id,
                name: subgraph_data.name,
                nodes: current_subgraph_nodes,
                edges: converted_edges,
                entry_nodes: entry_node_ids,
                exit_nodes: exit_node_ids,
                metadata: metadata_map,
                static_checks: static_checks_expr_ids,
            },
        );
    }

    Ok(CompiledTeg {
        id: program_id,
        name: program_name_str,
        base_dir: parsed_program.base_dir,
        expressions: compiled_expressions,
        handlers: compiled_handlers,
        subgraphs: compiled_subgraphs,
    })
}

// TODO: Add unit tests for ingest_parsed_teg.
// This will require creating mock ParsedTegProgram instances.
