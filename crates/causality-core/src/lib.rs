//! Core computational substrate for the Causality framework.
//!
//! This crate provides the fundamental types, traits, and implementations
//! for the Causality linear resource language, organized as a three-layer architecture.
//!
//! ## Architecture
//!
//! The crate is organized into three distinct layers:
//!
//! - **`machine/`** - Layer 0: Register Machine (11 instructions, minimal verifiable execution)
//! - **`lambda/`** - Layer 1: Linear Lambda Calculus (type-safe functional programming)
//! - **`effect/`** - Layer 2: Effect Algebra (domain-specific effect management)
//! - **`system/`** - Cross-cutting system utilities (content addressing, errors, serialization)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(missing_docs)]
#![recursion_limit = "256"]

//-----------------------------------------------------------------------------
// Core Modules
//-----------------------------------------------------------------------------

/// System-level utilities
pub mod system;

/// Layer 0: Register Machine - minimal verifiable execution model
pub mod machine;

/// Layer 1: Linear Lambda Calculus - type-safe functional programming
pub mod lambda;

/// Layer 2: Effect Algebra - domain-specific effect management
pub mod effect;

//-----------------------------------------------------------------------------
// Re-exports
//-----------------------------------------------------------------------------

// System utilities
pub use system::{
    // Errors (unified system)
    Error, Result, ErrorKind,
    error::{TypeError, MachineError, ReductionError, LinearityError, ResultExt},
    // Content addressing and core types
    EntityId, ResourceId, ExprId, RowTypeId, HandlerId, TransactionId, IntentId, DomainId, NullifierId,
    Timestamp, Str, ContentAddressable,
    encode_fixed_bytes, decode_fixed_bytes, DecodeWithRemainder,
    encode_with_length, decode_with_length, encode_enum_variant, decode_enum_variant,
    // Causality and domain system
    CausalProof, Domain,
};

// SMT re-exports from valence-coprocessor and our hasher
pub use valence_coprocessor::{
    Smt, Hash, HASH_LEN, 
    DataBackend, MemoryBackend, Hasher, SmtChildren, Opening,
};

// SHA256 hasher implementation
use sha2::{Sha256, Digest};

/// SHA256-based hasher implementation
#[derive(Clone)]
pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn hash(data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }

    fn key(domain: &str, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(domain.as_bytes());
        hasher.update(b":");
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }

    fn merge(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }

    fn digest<'a>(data: impl IntoIterator<Item = &'a [u8]>) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for chunk in data {
            hasher.update(chunk);
        }
        let result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        hash_bytes
    }
}

// An in-memory SMT implementation with SHA256 hashing
pub type MemorySmt = Smt<MemoryBackend, Sha256Hasher>;

// Layer 1: Linear Lambda Calculus types
pub use lambda::{
    BaseType, Type, TypeInner, Value, TypeRegistry,
    Linear, Affine, Relevant, Unrestricted,
    Linearity, LinearResource,
    SingleUse, Droppable, Copyable, MustUse, LinearityCheck,
    // Type constructors
    product, sum, linear_function,
    // Value types
    ProductValue, SumValue, UnitValue, LinearFunctionValue,
    // Introduction and elimination rules
    ProductIntro, ProductElim, SumIntro, SumElim,
    LinearFunctionIntro, LinearFunctionElim, UnitIntro, UnitElim,
    Symbol,
};

// Layer 0: Register Machine components
pub use machine::{
    Instruction, RegisterId, Pattern, MatchArm, ConstraintExpr, EffectCall, LiteralValue,
    MachineState, MachineValue,
    RegisterValue, Resource, Effect, Constraint,
    ReductionEngine,
    Nullifier, NullifierSet, NullifierError,
    ResourceHeap, ResourceManager,
    Metering, ComputeBudget, InstructionCosts,
};

// Layer 2: Effect Algebra components
pub use effect::{
    EffectExpr, EffectExprKind, EffectHandler, Span, Position,
    Pattern as AstPattern, PatternKind, FieldPattern,
};

// New primitive types for API compatibility
pub mod primitive {
    pub mod ids {
        pub use crate::system::content_addressing::EntityId;
        
        /// Domain identifier
        pub type DomainId = EntityId;
        
        /// Expression identifier  
        pub type ExprId = EntityId;
        
        /// Resource identifier
        pub type ResourceId = EntityId;
        
        /// Node identifier
        pub type NodeId = EntityId;
    }
    
    pub mod string {
        pub use crate::system::content_addressing::Str;
    }
    
    pub mod time {
        /// Timestamp type
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Timestamp {
            pub secs_since_epoch: u64,
        }
        
        impl Timestamp {
            pub fn now() -> Self {
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                Self {
                    secs_since_epoch: duration.as_secs(),
                }
            }
        }
    }
}

// Expression types for API compatibility
pub mod expression {
    pub mod r#type {
        use crate::lambda::base::TypeInner;
        use crate::system::content_addressing::Str;
        use std::collections::BTreeMap;
        
        /// Type expression for API compatibility
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum TypeExpr {
            Unit,
            Bool,
            Integer,
            String,
            Symbol,
            List(TypeExprBox),
            Map(TypeExprBox, TypeExprBox),
            Optional(TypeExprBox),
            Record(TypeExprMap),
        }
        
        /// Boxed type expression
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct TypeExprBox(pub Box<TypeExpr>);
        
        /// Map of type expressions for records
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct TypeExprMap(pub BTreeMap<Str, TypeExpr>);
        
        impl From<TypeInner> for TypeExpr {
            fn from(inner: TypeInner) -> Self {
                match inner {
                    TypeInner::Base(base) => match base {
                        crate::lambda::base::BaseType::Unit => TypeExpr::Unit,
                        crate::lambda::base::BaseType::Bool => TypeExpr::Bool,
                        crate::lambda::base::BaseType::Int => TypeExpr::Integer,
                        crate::lambda::base::BaseType::Symbol => TypeExpr::Symbol,
                    },
                    _ => TypeExpr::Unit, // Simplified conversion
                }
            }
        }
    }
}

// Resource types
pub mod resource {
    use crate::primitive::{ids::{EntityId, DomainId}, string::Str, time::Timestamp};
    
    /// Resource in the system
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Resource {
        pub id: EntityId,
        pub name: Str,
        pub domain_id: DomainId,
        pub resource_type: Str,
        pub quantity: u64,
        pub timestamp: Timestamp,
    }
}

// Graph dataflow structures
pub mod graph {
    pub mod dataflow {
        use crate::primitive::{ids::{ExprId, ResourceId, NodeId}, string::Str};
        use crate::expression::r#type::TypeExpr;
        use std::collections::BTreeMap;
        
        /// Process dataflow definition with automatic schema generation
        #[derive(Debug, Clone)]
        pub struct ProcessDataflowDefinition<I, O, S> {
            pub definition_id: ExprId,
            pub name: Str,
            pub nodes: Vec<ProcessDataflowNode>,
            pub edges: Vec<ProcessDataflowEdge>,
            _phantom: std::marker::PhantomData<(I, O, S)>,
        }
        
        impl<I, O, S> ProcessDataflowDefinition<I, O, S>
        where
            I: TypeSchema,
            O: TypeSchema,
            S: TypeSchema,
        {
            pub fn new(definition_id: ExprId, name: Str) -> Self {
                Self {
                    definition_id,
                    name,
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    _phantom: std::marker::PhantomData,
                }
            }
            
            pub fn add_node(&mut self, node: ProcessDataflowNode) {
                self.nodes.push(node);
            }
            
            pub fn add_edge(&mut self, edge: ProcessDataflowEdge) {
                self.edges.push(edge);
            }
            
            pub fn input_schema() -> TypeExpr {
                I::type_expr()
            }
            
            pub fn output_schema() -> TypeExpr {
                O::type_expr()
            }
            
            pub fn state_schema() -> TypeExpr {
                S::type_expr()
            }
        }
        
        /// Node in a process dataflow
        #[derive(Debug, Clone)]
        pub struct ProcessDataflowNode {
            pub id: NodeId,
            pub name: Str,
            pub node_type: Str,
            pub preferred_domain: Option<super::optimization::TypedDomain>,
        }
        
        impl ProcessDataflowNode {
            pub fn new(id: NodeId, name: Str, node_type: Str) -> Self {
                Self {
                    id,
                    name,
                    node_type,
                    preferred_domain: None,
                }
            }
            
            pub fn with_preferred_domain(mut self, domain: super::optimization::TypedDomain) -> Self {
                self.preferred_domain = Some(domain);
                self
            }
        }
        
        /// Edge in a process dataflow
        #[derive(Debug, Clone)]
        pub struct ProcessDataflowEdge {
            pub name: Str,
            pub from_node: NodeId,
            pub from_port: Str,
            pub to_node: NodeId,
            pub to_port: Str,
        }
        
        impl ProcessDataflowEdge {
            pub fn new(name: Str, from_node: NodeId, from_port: Str, to_node: NodeId, to_port: Str) -> Self {
                Self {
                    name,
                    from_node,
                    from_port,
                    to_node,
                    to_port,
                }
            }
        }
        
        /// Instance state of a process dataflow
        #[derive(Debug, Clone)]
        pub struct ProcessDataflowInstanceState {
            pub instance_id: ResourceId,
            pub definition_id: ExprId,
            pub execution_state: DataflowExecutionState,
            pub node_states: BTreeMap<NodeId, String>,
            pub metadata: BTreeMap<String, String>,
            pub initiation_hint: Option<String>,
        }
        
        /// Execution state of a dataflow
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum DataflowExecutionState {
            Pending,
            Running,
            Completed,
            Failed(String),
        }
        
        /// Trait for types that can provide schema information
        pub trait TypeSchema {
            fn type_expr() -> TypeExpr;
        }
    }
    
    pub mod optimization {
        use crate::primitive::{ids::DomainId, string::Str};
        
        /// Typed domain for optimization
        #[derive(Debug, Clone)]
        pub struct TypedDomain {
            pub domain_id: DomainId,
            pub domain_type: Str,
        }
        
        impl TypedDomain {
            pub fn new(domain_id: DomainId, domain_type: Str) -> Self {
                Self {
                    domain_id,
                    domain_type,
                }
            }
        }
    }
}
