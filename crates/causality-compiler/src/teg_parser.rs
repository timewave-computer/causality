//-----------------------------------------------------------------------------
// TEG Parser - Fixed minimal version that maintains core functionality
//-----------------------------------------------------------------------------

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{bail, Result, Context};
use sha2::Digest;

use causality_types::{
    core::Handler as TypesHandler,
    core::id::{NodeId, EdgeId, ExprId, EntityId, AsId},
    expr::{value::ValueExpr, value::ValueExprMap, value::ValueExprVec, ast::Expr},
    serialization::Encode,
};
// Removed lisp_compat dependency - using direct types


//-----------------------------------------------------------------------------
// Type Definitions (Replaced with .bak7 versions)
//-----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct ParsedTegProgram {
    pub id: Option<ProgramId>,
    pub program_name: Option<String>,
    pub base_dir: PathBuf,
    pub global_expressions: HashMap<ExprId, Expr>,
    pub defined_handlers: HashMap<String, TypesHandler>,
    pub subgraphs: Vec<SubgraphData>,
}

#[derive(Debug)]
pub struct SubgraphData {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub entry_nodes: Vec<String>,
    pub exit_nodes: Vec<String>,
    pub metadata: lexpr::Value,
    pub subgraph_specific_expressions: HashMap<ExprId, Expr>,
    pub subgraph_specific_expressions_ids: Vec<ExprId>,
    pub effect_name_to_node_id_map: HashMap<String, TypesNodeId>,
}

impl Default for SubgraphData {
    fn default() -> Self {
        Self {
            name: String::default(),
            nodes: Vec::default(),
            edges: Vec::default(),
            entry_nodes: Vec::default(),
            exit_nodes: Vec::default(),
            metadata: lexpr::Value::Nil,
            subgraph_specific_expressions: HashMap::default(),
            subgraph_specific_expressions_ids: Vec::default(),
            effect_name_to_node_id_map: HashMap::default(),
        }
    }
}

//-----------------------------------------------------------------------------
// Core Parser Functions
//-----------------------------------------------------------------------------

/// Parses a TEG definition file. This is the main entry point for parsing.
pub fn parse_teg_definition_file(file_path: &Path) -> Result<ParsedTegProgram> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read TEG definition file: {:?}", file_path))?;

    let s_expr_value = lexpr::from_str(&content)
        .with_context(|| format!("Failed to parse S-expression from file: {:?}", file_path))?;

    let mut program = ParsedTegProgram {
        base_dir: file_path.parent().unwrap_or_else(|| Path::new("")).to_path_buf(),
        ..Default::default()
    };

    if let lexpr::Value::Cons(top_cons) = &s_expr_value {
        let mut top_iter = top_cons.list_iter();
        match top_iter.next() {
            Some(lexpr::Value::Symbol(s)) if s.as_ref() == "define-teg" => {
                // (define-teg program-name section1 section2 ...)
                if let Some(name_val) = top_iter.next() {
                    if let Some(name_str) = name_val.as_symbol().map(|s| s.to_string()) // TODO: or_else as_str for string literal names
                                           .or_else(|| name_val.as_str().map(|s| s.to_string())) {
                        program.program_name = Some(name_str);
                    } else {
                        bail!("Program name in (define-teg <program-name> ...) must be a symbol or string.");
                    }
                } else {
                    bail!("(define-teg ...) requires a program name.");
                }

                // Process remaining sections
                for section_val in top_iter {
                    parse_section(section_val, &mut program)?;
                }
                Ok(program)
            }
            Some(lexpr::Value::Symbol(s)) if s.as_ref() == "tel" => {
                // Handle raw (tel section1 section2 ...) for testing or simpler cases
                // program_name will be None, base_dir will be from file_path
                log::warn!("Parsing raw '(tel ...)' form. Program name will be None.");
                for section_val in top_iter { // top_iter here starts after "tel"
                    parse_section(section_val, &mut program)?;
                }
                Ok(program)
            }
            _ => bail!("TEG definition must start with (define-teg program-name ...) or (tel ...). Found: {:?}", top_cons.car()),
        }
    } else {
        bail!("TEG definition file does not contain a valid S-expression list.")
    }
}

/// Parse a section in the TEG program
fn parse_section(
    section_val: &lexpr::Value,
    program: &mut ParsedTegProgram,
) -> Result<()> {
    // Parse a section in the form of (section-name ...)
    if let lexpr::Value::Cons(section_cons) = section_val {
        let mut iter = section_cons.list_iter();
        
        if let Some(lexpr::Value::Symbol(name)) = iter.next() {
            if let Some(args_val) = iter.next() {
                match name.as_ref() {
                    ":global-lisp" => parse_global_section(args_val, program),
                    ":handlers" => parse_handlers_section(args_val, program),
                    ":subgraphs" => parse_subgraphs_section(args_val, program),
                    _ => {
                        log::warn!("Skipping unknown section: {}", name);
                        Ok(())
                    }
                }
            } else {
                log::warn!("Skipping empty section: {}", name);
                Ok(())
            }
        } else {
            bail!("Expected section name to be a symbol")
        }
    } else {
        bail!("Expected section to be a list")
    }
}

/// Parse the :global-lisp section
fn parse_global_section(
    args_val: &lexpr::Value,
    program: &mut ParsedTegProgram,
) -> Result<()> {
    // Implementation simplified for clarity
    if let lexpr::Value::Cons(cons) = args_val {
        for item in cons.list_iter() {
            // Process global expressions
            if let Ok(expr) = lexpr_value_to_value_expr(item) {
                let ast_expr = Expr::Const(expr.clone());
                let expr_id = expr_to_id(&ast_expr);
                program.global_expressions.insert(expr_id, ast_expr);
            }
        }
    }
    Ok(())
}

/// Parse the :handlers section
fn parse_handlers_section(
    args_val: &lexpr::Value,
    program: &mut ParsedTegProgram,
) -> Result<()> {
    // Implementation simplified for clarity
    if let lexpr::Value::Cons(cons) = args_val {
        for handler_form in cons.list_iter() {
            if let lexpr::Value::Cons(handler_cons) = handler_form {
                if let Some(lexpr::Value::Symbol(s)) = handler_cons.list_iter().next() {
                    if s.as_ref() == "handler" {
                        // Process handler
                        if let Ok(handler_form_value_expr) = lexpr_value_to_value_expr(handler_form) { 
                            let handler_logic_ast_expr = Expr::Const(handler_form_value_expr.clone());
                            let handler_logic_expr_id = expr_to_id(&handler_logic_ast_expr);
                            
                            let handler_name = format!("handler_{}", handler_logic_expr_id.to_hex());
                            
                            let handler_data = TypesHandler {
                                id: EntityId::new(handler_logic_expr_id.inner()),
                                expression: Some(handler_logic_expr_id), 
                                // Other fields like effect_type, domain, constraints are Default
                                ..Default::default()
                            };
                            program.defined_handlers.insert(handler_name.clone(), handler_data);
                            program.global_expressions.insert(handler_logic_expr_id, handler_logic_ast_expr);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Parse the :subgraphs section
fn parse_subgraphs_section(
    args_val: &lexpr::Value,
    program: &mut ParsedTegProgram,
) -> Result<()> {
    // Implementation simplified for clarity
    if let lexpr::Value::Cons(cons) = args_val {
        for subgraph in cons.list_iter() {
            if let lexpr::Value::Cons(subgraph_cons) = subgraph {
                if let Some(lexpr::Value::Symbol(s)) = subgraph_cons.list_iter().next() {
                    if s.as_ref() == "subgraph" {
                        // Process subgraph
                        if let Ok(_expr_form_as_value_expr) = lexpr_value_to_value_expr(subgraph) {
                            let subgraph_data = SubgraphData {
                                name: format!("Subgraph {}", program.subgraphs.len()),
                                nodes: Vec::new(),
                                edges: Vec::new(),
                                entry_nodes: Vec::new(),
                                exit_nodes: Vec::new(),
                                metadata: subgraph.clone(),
                                subgraph_specific_expressions: HashMap::new(),
                                subgraph_specific_expressions_ids: Vec::new(),
                                effect_name_to_node_id_map: HashMap::new(),
                            };
                            program.subgraphs.push(subgraph_data);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Helper function to extract a value from a map
pub fn extract_value_from_map<'a>(
    map: &'a ValueExprMap,
    key: &str,
) -> Option<&'a ValueExpr> {
    map.0.get(key)
}

/// Converts a lexpr::Value to a causality_types::expr::value::ValueExpr.
/// This function handles basic types and lists, converting them as directly as possible.
/// Numbers are converted to i32, clamping if out of range, for ZK compatibility.
/// Strings, Symbols, and Keywords are converted to ValueExpr::String.
/// Cons cells are converted to ValueExpr::List.
pub fn lexpr_value_to_value_expr(val: &lexpr::Value) -> Result<ValueExpr> {
    match val {
        lexpr::Value::Nil => Ok(ValueExpr::Nil),
        lexpr::Value::String(s) => Ok(ValueExpr::String(s.to_string().into())),
        lexpr::Value::Symbol(s) => Ok(ValueExpr::String(s.to_string().into())),
        lexpr::Value::Keyword(s) => Ok(ValueExpr::String(s.to_string().into())),
        lexpr::Value::Number(n) => match n.as_i64() {
            Some(i) => {
                let i32_value = if i > i32::MAX as i64 {
                    i32::MAX
                } else if i < i32::MIN as i64 {
                    i32::MIN
                } else {
                    i as i32
                };
                Ok(ValueExpr::Number((i32_value as i64).into()))
            },
            None => {
                if let Some(f) = n.as_f64() {
                    let i32_value = if f > i32::MAX as f64 {
                        i32::MAX
                    } else if f < i32::MIN as f64 {
                        i32::MIN
                    } else {
                        f as i32
                    };
                    Ok(ValueExpr::Number((i32_value as i64).into()))
                } else {
                    bail!("Unsupported numeric format: {:?}", n)
                }
            }
        },
        lexpr::Value::Bool(b) => Ok(ValueExpr::Bool(*b)),
        lexpr::Value::Cons(cons) => {
            let mut elements = Vec::new();
            let mut current = Some(cons.clone());
            while let Some(cell) = current {
                let value = cell.car();
                elements.push(lexpr_value_to_value_expr(value)?);
                if let lexpr::Value::Cons(next_cell) = cell.cdr() {
                    current = Some(next_cell.clone());
                } else {
                    current = None;
                }
            }
            Ok(ValueExpr::List(ValueExprVec(elements)))
        }
        lexpr::Value::Bytes(b) => Ok(ValueExpr::String(String::from_utf8_lossy(b).to_string().into())),
        _ => bail!("Unsupported lexpr::Value variant: {:?}", val),
    }
}

/// Helper function to create a node ID
pub fn create_node_id_from_string(id_str: &str) -> TypesNodeId {
    let mut hasher = sha2::Sha256::new();
    hasher.update(id_str.as_bytes());
    let hash = hasher.finalize();
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(hash.as_slice());
    TypesNodeId::new(bytes)
}

/// Helper function to create an edge ID
pub fn create_edge_id(
    source: &TypesNodeId,
    target: &TypesNodeId,
    kind: &str,
) -> EdgeId {
    let mut hasher = sha2::Sha256::new();
    hasher.update(source.as_ssz_bytes());
    hasher.update(target.as_ssz_bytes());
    hasher.update(kind.as_bytes());
    let hash = hasher.finalize();
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(hash.as_slice());
    EdgeId::new(bytes)
}

/// Helper function to generate an ExprId from an Expr
fn expr_to_id(expr: &Expr) -> ExprId {
    use causality_types::serialization::Encode;
    use sha2::{Digest, Sha256};
    
    let encoded = expr.as_ssz_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&encoded);
    let hash = hasher.finalize();
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(hash.as_slice());
    ExprId::new(bytes)
}

// Type aliases for compatibility
type ProgramId = EntityId;
#[allow(dead_code)]
type SubgraphId = EntityId;
type TypesNodeId = NodeId;
type Node = causality_types::core::Resource; // Using Resource as a node type
type Edge = causality_types::tel::Edge;
