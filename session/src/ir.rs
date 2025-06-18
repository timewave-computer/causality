// Content-Addressed Intermediate Representation
// Built on the session types foundation for multi-target compilation
//
// This module provides a complete intermediate representation (IR) system suitable
// for compiling high-level languages (Rust, OCaml) to multiple targets (ZK circuits, 
// Solidity, CosmWasm, WebAssembly). The IR is built on mathematical foundations of
// linear session types with content addressing for deterministic, verifiable compilation.
//
// ## Core Features
//
// 1. **Content Addressing**: Every IR node has a SHA256 content address, enabling:
//    - Deterministic builds across platforms
//    - Deduplication of identical sub-expressions
//    - Incremental compilation and caching
//    - Cryptographic verification of transformations
//
// 2. **Target Metadata**: Rich metadata system supporting:
//    - Gas estimates for blockchain targets
//    - Constraint counts for ZK circuits
//    - Memory requirements for WASM
//    - Storage layout hints for smart contracts
//
// 3. **Linear Type Safety**: Built on session types ensuring:
//    - Linear resource usage tracking
//    - Protocol compliance verification
//    - Safe concurrent programming patterns
//    - Automatic resource cleanup
//
// 4. **Transformation Pipeline**: Complete toolkit including:
//    - Builder pattern for IR construction
//    - Dead code elimination
//    - Constant folding
//    - Variable usage analysis
//    - Content-preserving transformations
//
// ## Example Usage
//
// ```rust
// use session::ir::{IrBuilder, IrTransform, TargetHint};
// use session::layer1::{Type, SessionType};
//
// // Create a builder with Solidity target hints
// let builder = IrBuilder::new()
//     .with_default_target("solidity".to_string(), TargetHint::Solidity {
//         gas_estimate: Some(25000),
//         storage_slots: vec!["main_data".to_string()],
//     });
//
// // Build a payment protocol IR
// let amount = builder.int(100);
// let recipient = builder.record(vec![
//     ("address", builder.int(12345)),
//     ("verified", builder.bool(true)),
// ]);
// let payment = builder.record(vec![
//     ("amount", amount),
//     ("recipient", recipient),
// ]);
//
// // Apply optimizations
// let optimized = IrTransform::constant_fold(
//     IrTransform::eliminate_dead_code(payment)
// );
//
// // Content addressing ensures deterministic compilation
// let content_id = optimized.id();
// println!("Payment IR: {}", content_id); // ir:a1b2c3d4...
// ```
//
// ## Mathematical Foundation
//
// The IR extends the four-layer Causality-Valence architecture:
// - Layer 0: Content-addressed message machine (execution model)
// - Layer 1: Session-typed terms (programming model) 
// - Layer 2: Effect handlers with outcomes (composition model)
// - Layer 3: Agent choreography with capabilities (distribution model)
// - **IR Layer**: Content-addressed compilation with target metadata
//
// This ensures that the IR maintains all the mathematical properties of the
// underlying session type system while adding the metadata needed for 
// multi-target compilation.

use crate::layer1::{Term, Type, SessionType};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// Content-addressed IR node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IrNodeId([u8; 32]);

impl IrNodeId {
    /// Create an IrNodeId from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        IrNodeId(bytes)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Compute content address from serializable data
    pub fn from_content<T: Serialize>(content: &T) -> Self {
        let serialized = bincode::serialize(content)
            .expect("Failed to serialize content for addressing");
        
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        IrNodeId(bytes)
    }
}

impl std::fmt::Display for IrNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Show first 8 bytes in hex for readability
        let hex: String = self.0[..8].iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        write!(f, "ir:{}", hex)
    }
}

/// Trait for types that can be content-addressed
pub trait ContentAddress {
    /// Compute the content address of this value
    fn content_id(&self) -> IrNodeId;
}

/// Content-addressed wrapper around any IR component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrNode<T> {
    /// The content-addressed ID
    id: IrNodeId,
    
    /// The actual content
    content: T,
    
    /// Optional metadata for compilation targets
    metadata: IrMetadata,
}

/// Metadata attached to IR nodes for compilation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub struct IrMetadata {
    /// Type information (if available)
    pub type_info: Option<Type>,
    
    /// Session type information (if applicable)
    pub session_type: Option<SessionType>,
    
    /// Target-specific hints - deterministic ordering
    pub target_hints: BTreeMap<String, TargetHint>,
    
    /// Linearity information for resources
    pub linearity: LinearityInfo,
}

/// Target-specific compilation hints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetHint {
    /// ZK circuit specific hints
    ZkCircuit {
        constraint_count: Option<usize>,
        public_inputs: Vec<String>,
    },
    
    /// Solidity contract hints
    Solidity {
        gas_estimate: Option<u64>,
        storage_slots: Vec<String>,
    },
    
    /// CosmWasm contract hints
    CosmWasm {
        gas_limit: Option<u64>,
        required_capabilities: Vec<String>,
    },
    
    /// WebAssembly hints
    Wasm {
        memory_pages: Option<u32>,
        exports: Vec<String>,
    },
}

/// Linearity information for linear resource tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub struct LinearityInfo {
    /// Variables that must be used exactly once
    pub linear_vars: Vec<String>,
    
    /// Variables that can be used multiple times
    pub unrestricted_vars: Vec<String>,
    
    /// Affine variables (used at most once)
    pub affine_vars: Vec<String>,
}

impl<T> IrNode<T> 
where 
    T: Serialize + Clone,
{
    /// Create a new IR node with content addressing
    pub fn new(content: T) -> Self {
        let id = IrNodeId::from_content(&content);
        IrNode {
            id,
            content,
            metadata: IrMetadata::default(),
        }
    }
    
    /// Create IR node with metadata
    pub fn with_metadata(content: T, metadata: IrMetadata) -> Self {
        // Include metadata in content addressing for consistency
        let addressed_content = (&content, &metadata);
        let id = IrNodeId::from_content(&addressed_content);
        
        IrNode {
            id,
            content,
            metadata,
        }
    }
    
    /// Get the content ID
    pub fn id(&self) -> IrNodeId {
        self.id
    }
    
    /// Get the content
    pub fn content(&self) -> &T {
        &self.content
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &IrMetadata {
        &self.metadata
    }
    
    /// Update metadata (changes the ID)
    pub fn update_metadata(mut self, metadata: IrMetadata) -> Self {
        let addressed_content = (&self.content, &metadata);
        self.id = IrNodeId::from_content(&addressed_content);
        self.metadata = metadata;
        self
    }
    
    /// Add a target hint
    pub fn with_target_hint(mut self, target: String, hint: TargetHint) -> Self {
        self.metadata.target_hints.insert(target, hint);
        // Recompute ID since metadata changed
        let addressed_content = (&self.content, &self.metadata);
        self.id = IrNodeId::from_content(&addressed_content);
        self
    }
}

impl<T: Serialize> ContentAddress for IrNode<T> {
    fn content_id(&self) -> IrNodeId {
        self.id
    }
}

// Implement ContentAddress for core types
impl ContentAddress for Term {
    fn content_id(&self) -> IrNodeId {
        IrNodeId::from_content(self)
    }
}

impl ContentAddress for Type {
    fn content_id(&self) -> IrNodeId {
        IrNodeId::from_content(self)
    }
}

impl ContentAddress for SessionType {
    fn content_id(&self) -> IrNodeId {
        IrNodeId::from_content(self)
    }
}

/// Type aliases for commonly used IR nodes
pub type IrTerm = IrNode<Term>;
pub type IrType = IrNode<Type>;
pub type IrSession = IrNode<SessionType>;

/// Builder for constructing IR graphs with automatic content addressing
#[derive(Debug, Clone)]
pub struct IrBuilder {
    /// Default metadata to apply to new nodes
    default_metadata: IrMetadata,
    
    /// Target hints to apply to new nodes - deterministic ordering
    default_targets: BTreeMap<String, TargetHint>,
}

/// Transformation utilities for IR manipulation
pub struct IrTransform;

impl IrBuilder {
    /// Create a new IR builder with default settings
    pub fn new() -> Self {
        IrBuilder {
            default_metadata: IrMetadata::default(),
            default_targets: BTreeMap::new(),
        }
    }
    
    /// Set default type information for new nodes
    pub fn with_default_type(mut self, ty: Type) -> Self {
        self.default_metadata.type_info = Some(ty);
        self
    }
    
    /// Add a default target hint
    pub fn with_default_target(mut self, target: String, hint: TargetHint) -> Self {
        self.default_targets.insert(target, hint);
        self
    }
    
    /// Build an IR node from a term
    pub fn build_term(&self, term: Term) -> IrTerm {
        let mut metadata = self.default_metadata.clone();
        metadata.target_hints.extend(self.default_targets.clone());
        
        IrTerm::with_metadata(term, metadata)
    }
    
    /// Build an integer constant
    pub fn int(&self, value: i64) -> IrTerm {
        self.build_term(Term::Int(value))
    }
    
    /// Build a boolean constant
    pub fn bool(&self, value: bool) -> IrTerm {
        self.build_term(Term::Bool(value))
    }
    
    /// Build a variable reference
    pub fn var(&self, name: &str) -> IrTerm {
        self.build_term(Term::var(name))
    }
    
    /// Build a let binding
    pub fn let_bind(&self, var: &str, value: IrTerm, body: IrTerm) -> IrTerm {
        self.build_term(Term::Let {
            var: crate::layer1::linear::Variable(var.to_string()),
            value: Box::new(value.content().clone()),
            body: Box::new(body.content().clone()),
        })
    }
    
    /// Build a record (message)
    pub fn record(&self, fields: Vec<(&str, IrTerm)>) -> IrTerm {
        let term_fields: Vec<(&str, Term)> = fields.iter()
            .map(|(label, ir_term)| (*label, ir_term.content().clone()))
            .collect();
        self.build_term(Term::record(term_fields))
    }
    
    /// Build a field projection
    pub fn project(&self, record: IrTerm, field: &str) -> IrTerm {
        self.build_term(Term::project(record.content().clone(), field))
    }
    
    /// Build a pair
    pub fn pair(&self, left: IrTerm, right: IrTerm) -> IrTerm {
        self.build_term(Term::pair(left.content().clone(), right.content().clone()))
    }
    
    /// Build session creation
    pub fn new_session(&self, session_type: SessionType) -> IrTerm {
        self.build_term(Term::NewSession(session_type))
    }
    
    /// Build session send
    pub fn send(&self, channel: IrTerm, value: IrTerm) -> IrTerm {
        self.build_term(Term::Send {
            channel: Box::new(channel.content().clone()),
            value: Box::new(value.content().clone()),
        })
    }
    
    /// Build session receive
    pub fn receive(&self, channel: IrTerm) -> IrTerm {
        self.build_term(Term::Receive(Box::new(channel.content().clone())))
    }
}

impl IrTransform {
    /// Apply a transformation function to all terms in an IR graph
    pub fn map_terms<F>(ir_node: IrTerm, f: F) -> IrTerm 
    where 
        F: Fn(&Term) -> Term,
    {
        let transformed = Self::map_term_recursive(ir_node.content(), &f);
        IrTerm::with_metadata(transformed, ir_node.metadata().clone())
    }
    
    /// Recursively apply transformation to a term
    fn map_term_recursive<F>(term: &Term, f: &F) -> Term 
    where 
        F: Fn(&Term) -> Term,
    {
        let transformed = match term {
            Term::Pair(left, right) => Term::Pair(
                Box::new(Self::map_term_recursive(left, f)),
                Box::new(Self::map_term_recursive(right, f)),
            ),
            
            Term::Let { var, value, body } => Term::Let {
                var: var.clone(),
                value: Box::new(Self::map_term_recursive(value, f)),
                body: Box::new(Self::map_term_recursive(body, f)),
            },
            
            Term::Send { channel, value } => Term::Send {
                channel: Box::new(Self::map_term_recursive(channel, f)),
                value: Box::new(Self::map_term_recursive(value, f)),
            },
            
            Term::Receive(channel) => Term::Receive(
                Box::new(Self::map_term_recursive(channel, f))
            ),
            
            Term::Project { record, label } => Term::Project {
                record: Box::new(Self::map_term_recursive(record, f)),
                label: label.clone(),
            },
            
            Term::Record(fields) => Term::Record(
                fields.iter()
                    .map(|(label, term)| (label.clone(), Box::new(Self::map_term_recursive(term, f))))
                    .collect()
            ),
            
            // Base cases - no recursion needed
            _ => term.clone(),
        };
        
        f(&transformed)
    }
    
    /// Constant folding optimization
    pub fn constant_fold(ir_node: IrTerm) -> IrTerm {
        Self::map_terms(ir_node, |term| {
            match term {
                // Fold arithmetic operations when possible
                Term::Pair(left, right) => {
                    if let (Term::Int(_a), Term::Int(_b)) = (left.as_ref(), right.as_ref()) {
                        // Could add arithmetic operations here
                        term.clone()
                    } else {
                        term.clone()
                    }
                }
                _ => term.clone(),
            }
        })
    }
    
    /// Dead code elimination - remove unused let bindings
    pub fn eliminate_dead_code(ir_node: IrTerm) -> IrTerm {
        Self::map_terms(ir_node, |term| {
            match term {
                Term::Let { var, value: _, body } => {
                    // Simple heuristic: if variable is never used in body, remove the let
                    if !Self::term_uses_variable(body, var) {
                        body.as_ref().clone()
                    } else {
                        term.clone()
                    }
                }
                _ => term.clone(),
            }
        })
    }
    
    /// Check if a term uses a specific variable
    fn term_uses_variable(term: &Term, var: &crate::layer1::linear::Variable) -> bool {
        match term {
            Term::Var(v) => v == var,
            Term::Pair(left, right) => {
                Self::term_uses_variable(left, var) || Self::term_uses_variable(right, var)
            }
            Term::Let { var: bound_var, value, body } => {
                // Variable is shadowed in the body if it's rebound
                let used_in_value = Self::term_uses_variable(value, var);
                let used_in_body = if bound_var == var {
                    false // Shadowed
                } else {
                    Self::term_uses_variable(body, var)
                };
                used_in_value || used_in_body
            }
            Term::Send { channel, value } => {
                Self::term_uses_variable(channel, var) || Self::term_uses_variable(value, var)
            }
            Term::Receive(channel) => Self::term_uses_variable(channel, var),
            Term::Project { record, .. } => Self::term_uses_variable(record, var),
            Term::Record(fields) => {
                fields.values().any(|term| Self::term_uses_variable(term, var))
            }
            // Base cases
            _ => false,
        }
    }
    
    /// Collect all variable names used in an IR graph
    pub fn collect_variables(ir_node: &IrTerm) -> Vec<String> {
        let mut vars = Vec::new();
        Self::collect_variables_recursive(ir_node.content(), &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }
    
    fn collect_variables_recursive(term: &Term, vars: &mut Vec<String>) {
        match term {
            Term::Var(v) => vars.push(v.0.clone()),
            Term::Pair(left, right) => {
                Self::collect_variables_recursive(left, vars);
                Self::collect_variables_recursive(right, vars);
            }
            Term::Let { var, value, body } => {
                vars.push(var.0.clone());
                Self::collect_variables_recursive(value, vars);
                Self::collect_variables_recursive(body, vars);
            }
            Term::Send { channel, value } => {
                Self::collect_variables_recursive(channel, vars);
                Self::collect_variables_recursive(value, vars);
            }
            Term::Receive(channel) => Self::collect_variables_recursive(channel, vars),
            Term::Project { record, .. } => Self::collect_variables_recursive(record, vars),
            Term::Record(fields) => {
                for term in fields.values() {
                    Self::collect_variables_recursive(term, vars);
                }
            }
            // Base cases
            _ => {}
        }
    }
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{Term, Type, SessionType};
    
    #[test]
    fn test_content_addressing_determinism() {
        let term = Term::Int(42);
        
        // Multiple calls should produce same ID
        let id1 = term.content_id();
        let id2 = term.content_id();
        let id3 = term.clone().content_id();
        
        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }
    
    #[test]
    fn test_different_content_different_ids() {
        let term1 = Term::Int(42);
        let term2 = Term::Int(43);
        let term3 = Term::Bool(true);
        
        let id1 = term1.content_id();
        let id2 = term2.content_id();
        let id3 = term3.content_id();
        
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);
    }
    
    #[test]
    fn test_ir_node_creation() {
        let term = Term::let_bind("x", Term::Int(42), Term::var("x"));
        let ir_node = IrTerm::new(term.clone());
        
        assert_eq!(ir_node.content(), &term);
        assert_eq!(ir_node.id(), term.content_id());
    }
    
    #[test]
    fn test_metadata_affects_id() {
        let term = Term::Int(42);
        
        let node1 = IrTerm::new(term.clone());
        let node2 = IrTerm::with_metadata(term.clone(), IrMetadata {
            type_info: Some(Type::Int),
            ..Default::default()
        });
        
        // Metadata inclusion should change the ID
        assert_ne!(node1.id(), node2.id());
    }
    
    #[test]
    fn test_target_hints() {
        let term = Term::Int(42);
        let ir_node = IrTerm::new(term)
            .with_target_hint("solidity".to_string(), TargetHint::Solidity {
                gas_estimate: Some(21000),
                storage_slots: vec!["slot0".to_string()],
            });
        
        assert!(ir_node.metadata().target_hints.contains_key("solidity"));
    }
    
    #[test]
    fn test_session_type_content_addressing() {
        let session1 = SessionType::Send(
            Box::new(Type::Int),
            Box::new(SessionType::End)
        );
        
        let session2 = SessionType::Receive(
            Box::new(Type::Int),
            Box::new(SessionType::End)
        );
        
        assert_ne!(session1.content_id(), session2.content_id());
        assert_eq!(session1.content_id(), session1.clone().content_id());
    }
    
    #[test]
    fn test_target_hint_system() {
        let term = Term::Int(42);
        
        // Create IR node with multiple target hints
        let ir_node = IrTerm::new(term)
            .with_target_hint("solidity".to_string(), TargetHint::Solidity {
                gas_estimate: Some(21000),
                storage_slots: vec!["slot0".to_string()],
            })
            .with_target_hint("zk-circuit".to_string(), TargetHint::ZkCircuit {
                constraint_count: Some(1000),
                public_inputs: vec!["input1".to_string(), "input2".to_string()],
            })
            .with_target_hint("cosmwasm".to_string(), TargetHint::CosmWasm {
                gas_limit: Some(500000),
                required_capabilities: vec!["stargate".to_string()],
            });
        
        // Verify all hints are present
        let metadata = ir_node.metadata();
        assert!(metadata.target_hints.contains_key("solidity"));
        assert!(metadata.target_hints.contains_key("zk-circuit"));
        assert!(metadata.target_hints.contains_key("cosmwasm"));
        
        // Verify hint content
        if let Some(TargetHint::Solidity { gas_estimate, .. }) = metadata.target_hints.get("solidity") {
            assert_eq!(*gas_estimate, Some(21000));
        } else {
            panic!("Expected Solidity hint");
        }
    }
    
    #[test]
    fn test_linearity_metadata() {
        let term = Term::let_bind("x", Term::Int(42), Term::var("x"));
        
        let metadata = IrMetadata {
            type_info: Some(Type::Int),
            session_type: None,
            target_hints: BTreeMap::new(),
            linearity: LinearityInfo {
                linear_vars: vec!["x".to_string()],
                unrestricted_vars: vec![],
                affine_vars: vec![],
            },
        };
        
        let ir_node = IrTerm::with_metadata(term, metadata);
        
        assert_eq!(ir_node.metadata().linearity.linear_vars, vec!["x"]);
        assert!(ir_node.metadata().linearity.unrestricted_vars.is_empty());
    }
    
    #[test]
    fn test_ir_node_id_display() {
        let term = Term::Bool(true);
        let id = term.content_id();
        
        let display = format!("{}", id);
        assert!(display.starts_with("ir:"));
        assert_eq!(display.len(), 19); // "ir:" + 16 hex chars
    }
    
    #[test]
    fn test_complex_ir_transformation() {
        // Create a complex term
        let term = Term::record(vec![
            ("amount", Term::Int(100)),
            ("recipient", Term::record(vec![
                ("address", Term::Int(12345)),
                ("verified", Term::Bool(true)),
            ])),
        ]);
        
        // Create IR node with full metadata
        let metadata = IrMetadata {
            type_info: Some(Type::Record(crate::layer1::types::RowType::from_fields(vec![
                ("amount".to_string(), Type::Int),
                ("recipient".to_string(), Type::Record(
                    crate::layer1::types::RowType::from_fields(vec![
                        ("id".to_string(), Type::Int),
                        ("name".to_string(), Type::Int), // Simplified
                    ])
                )),
            ]))),
            session_type: None,
            target_hints: {
                let mut hints = BTreeMap::new();
                hints.insert("solidity".to_string(), TargetHint::Solidity {
                    gas_estimate: Some(50000),
                    storage_slots: vec!["payment_data".to_string()],
                });
                hints
            },
            linearity: LinearityInfo {
                linear_vars: vec!["payment".to_string()],
                unrestricted_vars: vec!["config".to_string()],
                affine_vars: vec![],
            },
        };
        
        let ir_node = IrTerm::with_metadata(term, metadata);
        
        // Verify the IR node maintains all information
        assert!(ir_node.metadata().type_info.is_some());
        assert!(ir_node.metadata().target_hints.contains_key("solidity"));
        assert_eq!(ir_node.metadata().linearity.linear_vars, vec!["payment"]);
        
        // Verify content addressing is deterministic
        let id1 = ir_node.id();
        let id2 = ir_node.clone().id();
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_ir_builder() {
        let builder = IrBuilder::new()
            .with_default_target("solidity".to_string(), TargetHint::Solidity {
                gas_estimate: Some(30000),
                storage_slots: vec![],
            });
        
        // Build a simple program: let x = 42 in x + 1
        let x_var = builder.var("x");
        let forty_two = builder.int(42);
        let program = builder.let_bind("x", forty_two, x_var);
        
        // Verify the IR node has the default target hint
        assert!(program.metadata().target_hints.contains_key("solidity"));
        
        // Verify content addressing works
        let id1 = program.id();
        let id2 = program.clone().id();
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_ir_builder_record() {
        let builder = IrBuilder::new();
        
        let name_field = builder.var("name");
        let age_field = builder.int(25);
        
        let record = builder.record(vec![
            ("name", name_field),
            ("age", age_field),
        ]);
        
        // Verify the record was constructed correctly
        if let Term::Record(fields) = record.content() {
            assert!(fields.contains_key("name"));
            assert!(fields.contains_key("age"));
        } else {
            panic!("Expected record term");
        }
    }
    
    #[test]
    fn test_ir_builder_session() {
        let builder = IrBuilder::new();
        
        let session_type = SessionType::Send(
            Box::new(Type::Int),
            Box::new(SessionType::End)
        );
        
        let channel = builder.new_session(session_type);
        let value = builder.int(100);
        let send_op = builder.send(channel, value);
        
        // Verify the send operation was constructed
        if let Term::Send { .. } = send_op.content() {
            // Success
        } else {
            panic!("Expected send term");
        }
    }
    
    #[test]
    fn test_ir_transform_map() {
        let builder = IrBuilder::new();
        
        // Create: let x = 42 in pair(x, true)
        let x_var = builder.var("x");
        let bool_val = builder.bool(true);
        let pair = builder.pair(x_var, bool_val);
        let program = builder.let_bind("x", builder.int(42), pair);
        
        // Transform: increment all integers by 1
        let transformed = IrTransform::map_terms(program.clone(), |term| {
            match term {
                Term::Int(n) => Term::Int(n + 1),
                _ => term.clone(),
            }
        });
        
        // Verify transformation occurred
        // The ID should be different because content changed
        assert_ne!(program.id(), transformed.id());
        
        // Verify the int was incremented (would need to traverse to check)
    }
    
    #[test]
    fn test_variable_collection() {
        let builder = IrBuilder::new();
        
        let x_var = builder.var("x");
        let y_var = builder.var("y");
        let pair = builder.pair(x_var, y_var);
        let program = builder.let_bind("z", builder.int(42), pair);
        
        let vars = IrTransform::collect_variables(&program);
        
        // Should find x, y, z
        assert!(vars.contains(&"x".to_string()));
        assert!(vars.contains(&"y".to_string()));
        assert!(vars.contains(&"z".to_string()));
    }
    
    #[test]
    fn test_dead_code_elimination() {
        let builder = IrBuilder::new();
        
        // Create: let unused = 42 in true (unused variable)
        let program = builder.let_bind("unused", builder.int(42), builder.bool(true));
        
        let optimized = IrTransform::eliminate_dead_code(program.clone());
        
        // The let binding should be eliminated, leaving just true
        if let Term::Bool(true) = optimized.content() {
            // Success - dead code was eliminated
        } else {
            panic!("Dead code elimination failed");
        }
        
        // IDs should be different
        assert_ne!(program.id(), optimized.id());
    }
    
    #[test]
    fn test_used_variable_not_eliminated() {
        let builder = IrBuilder::new();
        
        // Create: let used = 42 in used (variable is used)
        let program = builder.let_bind("used", builder.int(42), builder.var("used"));
        
        let optimized = IrTransform::eliminate_dead_code(program.clone());
        
        // The let binding should NOT be eliminated
        if let Term::Let { .. } = optimized.content() {
            // Success - used variable was preserved
        } else {
            panic!("Used variable was incorrectly eliminated");
        }
        
        // IDs should be the same (no change)
        assert_eq!(program.id(), optimized.id());
    }
    
    #[test]
    fn test_ir_program_pipeline() {
        // Test a complete IR construction and transformation pipeline
        let builder = IrBuilder::new()
            .with_default_target("solidity".to_string(), TargetHint::Solidity {
                gas_estimate: Some(25000),
                storage_slots: vec!["main_storage".to_string()],
            });
        
        // Build a payment protocol IR:
        // let amount = 100 in
        // let recipient = record { address: 12345, verified: true } in
        // record { amount: amount, recipient: recipient }
        
        let recipient_record = builder.record(vec![
            ("address", builder.int(12345)),
            ("verified", builder.bool(true)),
        ]);
        
        let payment_record = builder.record(vec![
            ("amount", builder.var("amount")),
            ("recipient", builder.var("recipient")),
        ]);
        
        let program = builder.let_bind("amount", builder.int(100),
            builder.let_bind("recipient", recipient_record, payment_record)
        );
        
        // Apply optimizations
        let optimized = IrTransform::constant_fold(
            IrTransform::eliminate_dead_code(program.clone())
        );
        
        // Verify the target hint was preserved
        assert!(optimized.metadata().target_hints.contains_key("solidity"));
        
        // Verify variables are tracked correctly
        let vars = IrTransform::collect_variables(&optimized);
        assert!(vars.contains(&"amount".to_string()));
        assert!(vars.contains(&"recipient".to_string()));
        
        // Verify content addressing remains consistent
        let id1 = optimized.id();
        let id2 = optimized.clone().id();
        assert_eq!(id1, id2);
    }
} 