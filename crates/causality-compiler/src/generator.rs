use anyhow::Result;
use std::collections::{HashMap, HashSet};
use sha2::Digest; // Added for Sha256::new()


//-----------------------------------------------------------------------------
// Crate-Local Imports
//-----------------------------------------------------------------------------
use crate::project::ProgramProject;

//-----------------------------------------------------------------------------
// External Crate Imports (causality_types)
//-----------------------------------------------------------------------------
use causality_types::{
    primitive::ids::{DomainId, CircuitId, NodeId, EdgeId},
    system::serialization::Encode,
};

// use causality_core::graph_registry::NodeRegistry;

//-----------------------------------------------------------------------------
// Type Aliases
//-----------------------------------------------------------------------------

pub type GenerationResult<T> = Result<T, anyhow::Error>;

//-----------------------------------------------------------------------------
// Constants
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// DomainInfo Struct (Helper for organizing by domain)
//-----------------------------------------------------------------------------

#[derive(Debug, Default)]
struct DomainInfo {
    nodes: Vec<NodeId>,
    edges: Vec<EdgeId>,
}

//-----------------------------------------------------------------------------
// Trait for project conversion (commenting out for now)
//-----------------------------------------------------------------------------
// pub trait FromProject {
//     type Output;
//     fn from_project(project: &ProgramProject) -> GenerationResult<Self::Output>;
// }

//-----------------------------------------------------------------------------
// Main Generator Struct and Impl
//-----------------------------------------------------------------------------

// Generates the final compiled TEG from a ProgramProject.
// This involves creating circuits for each domain and linking them.
pub struct ProgramGenerator;

impl Default for ProgramGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramGenerator {
    pub fn new() -> Self {
        ProgramGenerator
    }

    pub fn generate(
        &self,
        project: &ProgramProject,
        // _node_registry: &NodeRegistry<TelNodeTypes>, // node_registry seems unused in this function scope
    ) -> GenerationResult<CircuitId> { // TODO: This should likely return a CompiledTeg or ProgramId
        // Note: SMT collections don't support direct iteration like HashMap
        // For cycle detection, we'll need to implement a different approach
        // or maintain a separate index for edges
        
        // For now, let's generate a circuit ID based on the domain and project name
        let mut overall_project_hasher = sha2::Sha256::new();
        overall_project_hasher.update(project.name.as_bytes());
        overall_project_hasher.update(project.domain_id().as_ssz_bytes());
        let project_hash_bytes: [u8; 32] = overall_project_hasher.finalize().into();
        
        // Using crate::ids::generate_circuit_id which now returns CircuitId directly
        let circuit_id = crate::ids::generate_circuit_id(&project_hash_bytes, &[]); // Removed ?

        // Process each domain - placeholder for now
        // TODO: Restore when ProgramProject has domains field
        // for (_domain_id, _domain_module) in &project.domains {
        //     // In a real scenario, you'd generate a circuit per domain
        //     // and then combine them into a program.
        // }
        
        // Placeholder: Process project without domains for now
        log::debug!("Processing project: {}", project.name);

        // Example of how domain_specific_info might be built if needed:
        let mut _domain_specific_info: HashMap<DomainId, DomainInfo> = HashMap::new();
        // Populate _domain_specific_info as in the original generate_compiled_teg if that logic is moved here

        Ok(circuit_id) // Returning the single generated CircuitId for now
    }
}

// Commenting out FromProject impl for CircuitId as CircuitId is now foreign
// and from_project logic needs to be rethought (e.g., as part of generate_circuit_id or a helper).
// impl FromProject for CircuitId {
//     type Output = Self;
//     fn from_project(project: &ProgramProject) -> GenerationResult<Self::Output> {
//         // Placeholder: Generate ID based on some project properties
//         // This logic should ideally use a stable hashing mechanism on project contents
//         let mut hasher = Sha256::new();

//         // Include number of nodes and edges as a simple example
//         hasher.update(project.nodes.len().to_le_bytes());
//         hasher.update(project.edges.len().to_le_bytes());

//         // Include domain names (sorted for stability)
//         let mut domain_names: Vec<_> = project.domains.keys().collect();
//         domain_names.sort();
//         for domain_name in domain_names {
//             hasher.update(domain_name.as_bytes());
//         }
        
//         // ... include other relevant project data like expression hashes, etc. ...

//         let hash_result = hasher.finalize();
//         let mut id_bytes = [0u8; 32];
//         id_bytes.copy_from_slice(&hash_result[..]);
        
//         // The line causing E0425: Ok(CircuitId(id_bytes))
//         // This is invalid because CircuitId from causality_types is not a tuple struct we can call.
//         // We need to use its `new` method.
//         Ok(CircuitId::new(id_bytes))
//     }
// }

//-----------------------------------------------------------------------------
// Helper Functions (Cycle Detection, Domain Info)
//-----------------------------------------------------------------------------

// Retrieves the domain ID for a given node.
pub fn get_node_domain(
    node_id: &NodeId,
    project: &ProgramProject,
) -> Option<DomainId> {
    // For the SMT-backed storage, we use the project's domain_id
    // since the current structure supports a single domain per project
    // TODO: Future enhancement for multi-domain support
    let _node_exists = project.storage.get_node(node_id).ok().flatten().is_some();
    if _node_exists {
        Some(project.domain_id())
    } else {
        None
    }
}

// Detects cycles in the TEG graph representation.
fn tel_graph_has_cycles(edges: &HashMap<EdgeId, (NodeId, NodeId, String)>) -> bool {
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut all_nodes: HashSet<NodeId> = HashSet::new();

    for (source, target, _label) in edges.values() { // Corrected loop pattern: removed _edge_id from pattern
        adj.entry(*source).or_default().push(*target);
        all_nodes.insert(*source);
        all_nodes.insert(*target);
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut recursion_stack: HashSet<NodeId> = HashSet::new();

    for node_id in all_nodes {
        if !visited.contains(&node_id) && dfs_cycle_check(&node_id, &adj, &mut visited, &mut recursion_stack) {
            return true;
        }
    }
    false
}

fn dfs_cycle_check(
    node_id: &NodeId,
    adj: &HashMap<NodeId, Vec<NodeId>>,
    visited: &mut HashSet<NodeId>,
    recursion_stack: &mut HashSet<NodeId>,
) -> bool {
    visited.insert(*node_id);
    recursion_stack.insert(*node_id);

    if let Some(neighbors) = adj.get(node_id) {
        for neighbor_id in neighbors {
            if !visited.contains(neighbor_id) {
                if dfs_cycle_check(neighbor_id, adj, visited, recursion_stack) {
                    return true;
                }
            } else if recursion_stack.contains(neighbor_id) {
                return true; // Cycle detected
            }
        }
    }

    recursion_stack.remove(node_id);
    false
}

//-----------------------------------------------------------------------------
// Original generate_compiled_teg function (renamed or to be merged/refactored)
//-----------------------------------------------------------------------------

// Old function, parts might be merged into ProgramGenerator::generate
// COMMENTED OUT: Contains references to non-existent fields
/*
pub fn _old_generate_compiled_teg(project: &ProgramProject) -> Result<crate::ingest::CompiledTeg> {
    let mut compiled_teg = crate::ingest::CompiledTeg::new();

    // Note: SMT collections don't support direct iteration like HashMap
    // Cycle detection would need to be implemented differently with SMT storage
    // For now, we'll skip cycle detection and focus on basic compilation

    let mut domain_specific_info: HashMap<DomainId, DomainInfo> = HashMap::new();
    let mut _domain_nodes: HashSet<NodeId> = HashSet::new(); // Renamed from domain_nodes to avoid conflict if merging

    // This section seems to collect all nodes that belong to any domain.
    for domain_module in project.domains.values() {
        for nodes_in_subgraph in domain_module.subgraph_nodes.values() {
            for node_id in nodes_in_subgraph {
                _domain_nodes.insert(*node_id);
            }
        }
    }

    // Note: With SMT storage, we can't iterate over all nodes directly
    // We would need to maintain separate indexes or use different access patterns
    // For now, we'll work with the domain-specific node information we have

    // The rest of this function (processing each domain, creating circuits) 
    // would need to be adapted to use the ProgramGenerator struct and its methods,
    // or be part of a larger compilation pipeline.
    // For now, it's part of _old_generate_compiled_teg and not directly used.

    // Attempt to get or generate a ProgramId for the CompiledTeg
    let program_id = match project.program_id {
        Some(id) => id,
        None => {
            // If no program_id exists on the project, generate one based on circuit IDs.
            // This is a simplified approach. A more robust method might involve hashing
            // more comprehensive project details or ensuring circuits are generated first.
            // TODO: Restore when ProgramProject has circuits field
            let circuit_ids_for_program: Vec<[u8; 32]> = project.circuits.keys().map(|cid| cid.inner()).collect();
            if circuit_ids_for_program.is_empty() {
                // Fallback if there are no circuits yet: generate a default/random ProgramId
                // Or, decide if this should be an error condition.
                // For now, let's use a hash of the project name as a placeholder.
                let mut hasher = sha2::Sha256::new();
                hasher.update(project.name.as_bytes());
                let hash_result: [u8; 32] = hasher.finalize().into();
                crate::ids::generate_program_id(&[hash_result]) // Removed ?
            } else {
                crate::ids::generate_program_id(&circuit_ids_for_program) // Removed ?
            }
        }
    };
    compiled_teg.id = program_id; // Assuming CompiledTeg has an 'id: ProgramId' field

    Ok(compiled_teg)
}
*/
