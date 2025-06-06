//! Linear resource visualizer and effect graph viewer
//! 
//! This module provides visualization tools for understanding linear resource
//! lifetimes, effect graphs, and program execution flow.

use anyhow::Result;
use causality_compiler::compile;
use causality_core::{
    lambda::{Term, TermKind},
    machine::{Instruction, RegisterId},
};
use std::collections::{HashMap, HashSet};

/// Visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub format: OutputFormat,
    pub show_registers: bool,
    pub show_gas_costs: bool,
    pub simplify_graph: bool,
}

/// Output format for visualizations
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Mermaid,
    Dot,
    Ascii,
}

/// Resource flow graph for visualization
#[derive(Debug, Clone)]
pub struct ResourceFlowGraph {
    pub nodes: Vec<ResourceNode>,
    pub edges: Vec<ResourceEdge>,
    pub metadata: GraphMetadata,
}

/// Node in the resource flow graph
#[derive(Debug, Clone)]
pub struct ResourceNode {
    pub id: String,
    pub node_type: NodeType,
    pub instruction_index: usize,
    pub register: Option<RegisterId>,
    pub label: String,
}

/// Type of node in the graph
#[derive(Debug, Clone)]
pub enum NodeType {
    Allocation,
    Consumption,
    Move,
    Operation,
    Witness,
}

/// Edge in the resource flow graph
#[derive(Debug, Clone)]
pub struct ResourceEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub label: Option<String>,
}

/// Type of edge in the graph
#[derive(Debug, Clone)]
pub enum EdgeType {
    ResourceFlow,
    DataDependency,
    Control,
}

/// Graph metadata
#[derive(Debug, Clone)]
pub struct GraphMetadata {
    pub total_instructions: usize,
    pub total_registers: usize,
    pub resource_count: usize,
    pub max_live_resources: usize,
}

/// Generate resource visualization from source code
pub fn visualize_resources(
    source: &str,
    config: VisualizationConfig,
) -> Result<String> {
    // Compile the source to get instructions
    let artifact = compile(source)?;
    
    // Build resource flow graph
    let graph = build_resource_flow_graph(&artifact.instructions)?;
    
    // Generate visualization based on format
    match config.format {
        OutputFormat::Mermaid => generate_mermaid_diagram(&graph, &config),
        OutputFormat::Dot => generate_dot_diagram(&graph, &config),
        OutputFormat::Ascii => generate_ascii_diagram(&graph, &config),
    }
}

/// Build resource flow graph from instructions
fn build_resource_flow_graph(instructions: &[Instruction]) -> Result<ResourceFlowGraph> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut register_map: HashMap<RegisterId, String> = HashMap::new();
    let mut resource_allocations: HashMap<RegisterId, String> = HashMap::new();
    
    // First pass: create nodes for each instruction
    for (i, instruction) in instructions.iter().enumerate() {
        let node_id = format!("inst_{}", i);
        
        let (node_type, register, label) = match instruction {
            Instruction::Alloc { type_reg: _, val_reg: _, out_reg } => {
                resource_allocations.insert(*out_reg, node_id.clone());
                register_map.insert(*out_reg, format!("r{}", out_reg.0));
                (
                    NodeType::Allocation,
                    Some(*out_reg),
                    format!("Alloc r{}", out_reg.0),
                )
            }
            Instruction::Consume { resource_reg, out_reg } => {
                register_map.insert(*out_reg, format!("r{}", out_reg.0));
                (
                    NodeType::Consumption,
                    Some(*resource_reg),
                    format!("Consume r{}", resource_reg.0),
                )
            }
            Instruction::Move { src, dst } => {
                register_map.insert(*dst, format!("r{}", dst.0));
                (
                    NodeType::Move,
                    Some(*dst),
                    format!("Move r{} → r{}", src.0, dst.0),
                )
            }
            Instruction::Witness { out_reg } => {
                register_map.insert(*out_reg, format!("r{}", out_reg.0));
                (
                    NodeType::Witness,
                    Some(*out_reg),
                    format!("Witness r{}", out_reg.0),
                )
            }
            Instruction::Apply { fn_reg, arg_reg, out_reg } => {
                register_map.insert(*out_reg, format!("r{}", out_reg.0));
                (
                    NodeType::Operation,
                    Some(*out_reg),
                    format!("Apply r{} r{} → r{}", fn_reg.0, arg_reg.0, out_reg.0),
                )
            }
            _ => (NodeType::Operation, None, "Other".to_string()),
        };
        
        nodes.push(ResourceNode {
            id: node_id,
            node_type,
            instruction_index: i,
            register,
            label,
        });
    }
    
    // Second pass: create edges based on data dependencies
    for (i, instruction) in instructions.iter().enumerate() {
        let current_node = format!("inst_{}", i);
        
        match instruction {
            Instruction::Move { src, .. } => {
                // Find the instruction that produced src
                if let Some(producer) = find_register_producer(instructions, *src, i) {
                    edges.push(ResourceEdge {
                        from: format!("inst_{}", producer),
                        to: current_node,
                        edge_type: EdgeType::DataDependency,
                        label: Some(format!("r{}", src.0)),
                    });
                }
            }
            Instruction::Consume { resource_reg, .. } => {
                // Find where this resource was allocated
                if let Some(producer) = find_register_producer(instructions, *resource_reg, i) {
                    edges.push(ResourceEdge {
                        from: format!("inst_{}", producer),
                        to: current_node,
                        edge_type: EdgeType::ResourceFlow,
                        label: Some(format!("r{}", resource_reg.0)),
                    });
                }
            }
            Instruction::Apply { fn_reg, arg_reg, .. } => {
                // Add dependencies for both function and argument
                if let Some(fn_producer) = find_register_producer(instructions, *fn_reg, i) {
                    edges.push(ResourceEdge {
                        from: format!("inst_{}", fn_producer),
                        to: current_node.clone(),
                        edge_type: EdgeType::DataDependency,
                        label: Some(format!("fn:r{}", fn_reg.0)),
                    });
                }
                if let Some(arg_producer) = find_register_producer(instructions, *arg_reg, i) {
                    edges.push(ResourceEdge {
                        from: format!("inst_{}", arg_producer),
                        to: current_node,
                        edge_type: EdgeType::DataDependency,
                        label: Some(format!("arg:r{}", arg_reg.0)),
                    });
                }
            }
            _ => {}
        }
    }
    
    // Calculate metadata
    let unique_registers: HashSet<_> = register_map.keys().collect();
    let resource_count = resource_allocations.len();
    
    Ok(ResourceFlowGraph {
        nodes,
        edges,
        metadata: GraphMetadata {
            total_instructions: instructions.len(),
            total_registers: unique_registers.len(),
            resource_count,
            max_live_resources: calculate_max_live_resources(instructions),
        },
    })
}

/// Find the instruction that produces a given register
fn find_register_producer(
    instructions: &[Instruction],
    register: RegisterId,
    before_index: usize,
) -> Option<usize> {
    for (i, instruction) in instructions.iter().enumerate().take(before_index) {
        let produces_register = match instruction {
            Instruction::Alloc { out_reg, .. } => *out_reg == register,
            Instruction::Consume { out_reg, .. } => *out_reg == register,
            Instruction::Move { dst, .. } => *dst == register,
            Instruction::Witness { out_reg } => *out_reg == register,
            Instruction::Apply { out_reg, .. } => *out_reg == register,
            _ => false,
        };
        
        if produces_register {
            return Some(i);
        }
    }
    None
}

/// Calculate maximum number of live resources at any point
fn calculate_max_live_resources(instructions: &[Instruction]) -> usize {
    let mut live_resources = HashSet::new();
    let mut max_live = 0;
    
    for instruction in instructions {
        match instruction {
            Instruction::Alloc { out_reg, .. } => {
                live_resources.insert(*out_reg);
                max_live = max_live.max(live_resources.len());
            }
            Instruction::Consume { resource_reg, .. } => {
                live_resources.remove(resource_reg);
            }
            _ => {}
        }
    }
    
    max_live
}

/// Generate Mermaid flowchart diagram
fn generate_mermaid_diagram(
    graph: &ResourceFlowGraph,
    config: &VisualizationConfig,
) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("```mermaid\n");
    output.push_str("flowchart TD\n");
    output.push_str("    %% Resource Flow Graph\n");
    output.push_str("    %% Generated by Causality Visualizer\n\n");
    
    // Add nodes
    for node in &graph.nodes {
        let shape = match node.node_type {
            NodeType::Allocation => "([{}])",
            NodeType::Consumption => "{{{}}}",
            NodeType::Move => "[{}]",
            NodeType::Operation => "({})",
            NodeType::Witness => "<{}>",
        };
        
        let label = if config.show_registers {
            format!("{}<br/>#{}", node.label, node.instruction_index)
        } else {
            node.label.clone()
        };
        
        output.push_str(&format!(
            "    {} {}\n",
            node.id,
            shape.replace("{}", &label)
        ));
    }
    
    output.push_str("\n");
    
    // Add edges
    for edge in &graph.edges {
        let arrow = match edge.edge_type {
            EdgeType::ResourceFlow => "==>",
            EdgeType::DataDependency => "-->",
            EdgeType::Control => "-.->",
        };
        
        if let Some(label) = &edge.label {
            output.push_str(&format!(
                "    {} {}|{}| {}\n",
                edge.from, arrow, label, edge.to
            ));
        } else {
            output.push_str(&format!(
                "    {} {} {}\n",
                edge.from, arrow, edge.to
            ));
        }
    }
    
    // Add styling
    output.push_str("\n    %% Styling\n");
    output.push_str("    classDef allocation fill:#e1f5fe\n");
    output.push_str("    classDef consumption fill:#fff3e0\n");
    output.push_str("    classDef move fill:#f3e5f5\n");
    output.push_str("    classDef operation fill:#e8f5e8\n");
    output.push_str("    classDef witness fill:#fce4ec\n");
    
    // Apply styles to nodes
    for node in &graph.nodes {
        let class = match node.node_type {
            NodeType::Allocation => "allocation",
            NodeType::Consumption => "consumption",
            NodeType::Move => "move",
            NodeType::Operation => "operation",
            NodeType::Witness => "witness",
        };
        output.push_str(&format!("    class {} {}\n", node.id, class));
    }
    
    output.push_str("```\n\n");
    
    // Add metadata
    output.push_str(&format!("**Graph Metadata:**\n"));
    output.push_str(&format!("- Instructions: {}\n", graph.metadata.total_instructions));
    output.push_str(&format!("- Registers: {}\n", graph.metadata.total_registers));
    output.push_str(&format!("- Resources: {}\n", graph.metadata.resource_count));
    output.push_str(&format!("- Max live resources: {}\n", graph.metadata.max_live_resources));
    
    Ok(output)
}

/// Generate DOT (Graphviz) diagram
fn generate_dot_diagram(
    graph: &ResourceFlowGraph,
    _config: &VisualizationConfig,
) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("digraph ResourceFlow {\n");
    output.push_str("    rankdir=TD;\n");
    output.push_str("    node [fontname=\"Arial\"];\n");
    output.push_str("    edge [fontname=\"Arial\"];\n\n");
    
    // Add nodes
    for node in &graph.nodes {
        let (shape, color) = match node.node_type {
            NodeType::Allocation => ("box", "lightblue"),
            NodeType::Consumption => ("diamond", "orange"),
            NodeType::Move => ("ellipse", "lightgreen"),
            NodeType::Operation => ("circle", "yellow"),
            NodeType::Witness => ("triangle", "pink"),
        };
        
        output.push_str(&format!(
            "    {} [label=\"{}\" shape={} fillcolor={} style=filled];\n",
            node.id, node.label, shape, color
        ));
    }
    
    output.push_str("\n");
    
    // Add edges
    for edge in &graph.edges {
        let style = match edge.edge_type {
            EdgeType::ResourceFlow => "style=bold color=red",
            EdgeType::DataDependency => "color=blue",
            EdgeType::Control => "style=dashed color=gray",
        };
        
        if let Some(label) = &edge.label {
            output.push_str(&format!(
                "    {} -> {} [label=\"{}\" {}];\n",
                edge.from, edge.to, label, style
            ));
        } else {
            output.push_str(&format!(
                "    {} -> {} [{}];\n",
                edge.from, edge.to, style
            ));
        }
    }
    
    output.push_str("}\n");
    
    Ok(output)
}

/// Generate ASCII diagram
fn generate_ascii_diagram(
    graph: &ResourceFlowGraph,
    _config: &VisualizationConfig,
) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("Resource Flow Diagram (ASCII)\n");
    output.push_str("=============================\n\n");
    
    // Simple linear representation
    for (i, node) in graph.nodes.iter().enumerate() {
        let symbol = match node.node_type {
            NodeType::Allocation => "⊕",
            NodeType::Consumption => "⊖",
            NodeType::Move => "→",
            NodeType::Operation => "◯",
            NodeType::Witness => "◊",
        };
        
        output.push_str(&format!("{:2}: {} {}\n", i, symbol, node.label));
    }
    
    output.push_str("\nLegend:\n");
    output.push_str("⊕ = Allocation\n");
    output.push_str("⊖ = Consumption\n");
    output.push_str("→ = Move\n");
    output.push_str("◯ = Operation\n");
    output.push_str("◊ = Witness\n");
    
    Ok(output)
}

/// Generate effect graph from lambda term
pub fn visualize_effect_graph(source: &str) -> Result<String> {
    let artifact = compile(source)?;
    generate_effect_graph_mermaid(&artifact.term)
}

/// Generate Mermaid diagram for effect graph
fn generate_effect_graph_mermaid(term: &Term) -> Result<String> {
    let mut output = String::new();
    let mut node_counter = 0;
    
    output.push_str("```mermaid\n");
    output.push_str("graph TD\n");
    output.push_str("    %% Effect Graph\n\n");
    
    traverse_term_for_effects(term, &mut output, &mut node_counter, None);
    
    output.push_str("```\n");
    
    Ok(output)
}

/// Traverse term to build effect graph
fn traverse_term_for_effects(
    term: &Term,
    output: &mut String,
    counter: &mut u32,
    parent: Option<String>,
) -> String {
    let current_id = format!("n{}", counter);
    *counter += 1;
    
    let (label, shape) = match &term.kind {
        TermKind::Alloc { .. } => ("Alloc", "([{}])"),
        TermKind::Consume { .. } => ("Consume", "{{{}}}"),
        TermKind::Apply { .. } => ("Apply", "({})"),
        TermKind::Lambda { .. } => ("Lambda", "[{}]"),
        TermKind::Let { .. } => ("Let", "[{}]"),
        TermKind::Var(name) => (name.as_str(), "({})"),
        TermKind::Literal(_) => ("Literal", "<{}>"),
        TermKind::Unit => ("Unit", "({})"),
        _ => ("Term", "({})"),
    };
    
    output.push_str(&format!(
        "    {} {}\n",
        current_id,
        shape.replace("{}", label)
    ));
    
    if let Some(parent_id) = parent {
        output.push_str(&format!("    {} --> {}\n", parent_id, current_id));
    }
    
    // Recursively process child terms
    match &term.kind {
        TermKind::Apply { func, arg } => {
            traverse_term_for_effects(func, output, counter, Some(current_id.clone()));
            traverse_term_for_effects(arg, output, counter, Some(current_id.clone()));
        }
        TermKind::Lambda { body, .. } => {
            traverse_term_for_effects(body, output, counter, Some(current_id.clone()));
        }
        TermKind::Let { value, body, .. } => {
            traverse_term_for_effects(value, output, counter, Some(current_id.clone()));
            traverse_term_for_effects(body, output, counter, Some(current_id.clone()));
        }
        TermKind::Alloc { value } => {
            traverse_term_for_effects(value, output, counter, Some(current_id.clone()));
        }
        TermKind::Consume { resource } => {
            traverse_term_for_effects(resource, output, counter, Some(current_id.clone()));
        }
        _ => {}
    }
    
    current_id
} 