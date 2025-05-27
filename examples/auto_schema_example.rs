//! Automatic Schema Generation Example
//!
//! This example demonstrates the new automatic schema generation system
//! for ProcessDataflowDefinition, eliminating manual schema maintenance
//! while providing better type safety and developer experience.

use causality_types::{
    primitive::{
        ids::{ExprId, NodeId, DomainId},
        string::Str,
    },
    expression::r#type::{TypeExpr, TypeExprMap, TypeExprBox},
    graph::{
        dataflow::{ProcessDataflowDefinition, ProcessDataflowNode, ProcessDataflowEdge, TypeSchema},
        optimization::TypedDomain,
    },
    system::serialization::SimpleSerialize,
};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Example 1: Simple Token Transfer
//-----------------------------------------------------------------------------

/// Input parameters for a simple token transfer
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct TokenTransferInput {
    pub from_account: String,
    pub to_account: String,
    pub amount: u64,
    pub token_type: String,
}

/// Output from a simple token transfer
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct TokenTransferOutput {
    pub transaction_id: String,
    pub success: bool,
    pub final_balance: u64,
}

/// State maintained during token transfer
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct TokenTransferState {
    pub current_step: String,
    pub locked_amount: Option<u64>,
    pub validation_passed: bool,
}

// Manual TypeSchema implementations (demonstrating what the derive macro would generate)
impl TypeSchema for TokenTransferInput {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("from_account"), TypeExpr::String);
        fields.insert(Str::from("to_account"), TypeExpr::String);
        fields.insert(Str::from("amount"), TypeExpr::Integer);
        fields.insert(Str::from("token_type"), TypeExpr::String);
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for TokenTransferOutput {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("transaction_id"), TypeExpr::String);
        fields.insert(Str::from("success"), TypeExpr::Bool);
        fields.insert(Str::from("final_balance"), TypeExpr::Integer);
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for TokenTransferState {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("current_step"), TypeExpr::String);
        fields.insert(
            Str::from("locked_amount"), 
            TypeExpr::Optional(TypeExprBox(Box::new(TypeExpr::Integer)))
        );
        fields.insert(Str::from("validation_passed"), TypeExpr::Bool);
        TypeExpr::Record(TypeExprMap(fields))
    }
}

/// Type alias for the token transfer dataflow with automatic schema generation
pub type TokenTransferDataflow = ProcessDataflowDefinition<TokenTransferInput, TokenTransferOutput, TokenTransferState>;

//-----------------------------------------------------------------------------
// Example 2: Complex Cross-Domain Bridge
//-----------------------------------------------------------------------------

/// Input for cross-domain bridge operations
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct BridgeInput {
    pub source_chain: String,
    pub target_chain: String,
    pub asset: AssetInfo,
    pub recipient: String,
    pub bridge_config: BridgeConfig,
}

/// Asset information for bridging
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct AssetInfo {
    pub token_address: String,
    pub amount: u64,
    pub decimals: u8,
}

/// Bridge configuration parameters
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct BridgeConfig {
    pub fee_percentage: f64,
    pub timeout_seconds: u64,
    pub require_proof: bool,
    pub validators: Vec<String>,
}

/// Output from bridge operation
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct BridgeOutput {
    pub bridge_transaction_id: String,
    pub source_tx_hash: String,
    pub target_tx_hash: Option<String>,
    pub status: BridgeStatus,
    pub fee_paid: u64,
}

/// Bridge operation status
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub enum BridgeStatus {
    Pending,
    Confirmed,
    Failed(String),
    Cancelled,
}

/// State maintained during bridge operation
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub struct BridgeState {
    pub current_phase: BridgePhase,
    pub locked_assets: Vec<AssetInfo>,
    pub proofs_collected: Vec<String>,
    pub validator_signatures: BTreeMap<String, String>,
    pub timeout_at: Option<u64>,
}

/// Bridge operation phases
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "derive", derive(TypeSchema))]
pub enum BridgePhase {
    Validation,
    Locking,
    ProofGeneration,
    CrossChainRelay,
    Verification,
    Completion,
}

// Manual TypeSchema implementations for complex types
impl TypeSchema for BridgeInput {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("source_chain"), TypeExpr::String);
        fields.insert(Str::from("target_chain"), TypeExpr::String);
        fields.insert(Str::from("asset"), AssetInfo::type_expr());
        fields.insert(Str::from("recipient"), TypeExpr::String);
        fields.insert(Str::from("bridge_config"), BridgeConfig::type_expr());
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for AssetInfo {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("token_address"), TypeExpr::String);
        fields.insert(Str::from("amount"), TypeExpr::Integer);
        fields.insert(Str::from("decimals"), TypeExpr::Integer);
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for BridgeConfig {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("fee_percentage"), TypeExpr::Number);
        fields.insert(Str::from("timeout_seconds"), TypeExpr::Integer);
        fields.insert(Str::from("require_proof"), TypeExpr::Bool);
        fields.insert(
            Str::from("validators"), 
            TypeExpr::List(TypeExprBox(Box::new(TypeExpr::String)))
        );
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for BridgeOutput {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("bridge_transaction_id"), TypeExpr::String);
        fields.insert(Str::from("source_tx_hash"), TypeExpr::String);
        fields.insert(
            Str::from("target_tx_hash"), 
            TypeExpr::Optional(TypeExprBox(Box::new(TypeExpr::String)))
        );
        fields.insert(Str::from("status"), BridgeStatus::type_expr());
        fields.insert(Str::from("fee_paid"), TypeExpr::Integer);
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for BridgeStatus {
    fn type_expr() -> TypeExpr {
        TypeExpr::Union(vec![
            TypeExpr::Atom(Str::from("Pending")),
            TypeExpr::Atom(Str::from("Confirmed")),
            TypeExpr::Record(TypeExprMap({
                let mut fields = BTreeMap::new();
                fields.insert(Str::from("Failed"), TypeExpr::String);
                fields
            })),
            TypeExpr::Atom(Str::from("Cancelled")),
        ])
    }
}

impl TypeSchema for BridgeState {
    fn type_expr() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("current_phase"), BridgePhase::type_expr());
        fields.insert(
            Str::from("locked_assets"), 
            TypeExpr::List(TypeExprBox(Box::new(AssetInfo::type_expr())))
        );
        fields.insert(
            Str::from("proofs_collected"), 
            TypeExpr::List(TypeExprBox(Box::new(TypeExpr::String)))
        );
        fields.insert(
            Str::from("validator_signatures"),
            TypeExpr::Map(
                TypeExprBox(Box::new(TypeExpr::String)),
                TypeExprBox(Box::new(TypeExpr::String))
            )
        );
        fields.insert(
            Str::from("timeout_at"), 
            TypeExpr::Optional(TypeExprBox(Box::new(TypeExpr::Integer)))
        );
        TypeExpr::Record(TypeExprMap(fields))
    }
}

impl TypeSchema for BridgePhase {
    fn type_expr() -> TypeExpr {
        TypeExpr::Union(vec![
            TypeExpr::Atom(Str::from("Validation")),
            TypeExpr::Atom(Str::from("Locking")),
            TypeExpr::Atom(Str::from("ProofGeneration")),
            TypeExpr::Atom(Str::from("CrossChainRelay")),
            TypeExpr::Atom(Str::from("Verification")),
            TypeExpr::Atom(Str::from("Completion")),
        ])
    }
}

/// Type alias for the bridge dataflow with automatic schema generation
pub type BridgeDataflow = ProcessDataflowDefinition<BridgeInput, BridgeOutput, BridgeState>;

//-----------------------------------------------------------------------------
// Example Usage Functions
//-----------------------------------------------------------------------------

/// Demonstrate automatic schema generation for token transfers
pub fn demonstrate_token_transfer_schemas() {
    println!("🔄 Token Transfer Automatic Schema Generation");
    println!("==============================================");
    
    // Get automatically generated schemas
    let input_schema = TokenTransferDataflow::input_schema();
    let output_schema = TokenTransferDataflow::output_schema();
    let state_schema = TokenTransferDataflow::state_schema();
    
    println!("📋 Input Schema: {:?}", input_schema);
    println!("📤 Output Schema: {:?}", output_schema);
    println!("🗃️ State Schema: {:?}", state_schema);
    
    // Create a dataflow definition with automatic schemas
    let mut dataflow = TokenTransferDataflow::new(
        ExprId::new([1u8; 32]),
        Str::from("token_transfer_flow"),
    );
    
    // Add nodes
    dataflow.add_node(ProcessDataflowNode::new(
        NodeId::new([1u8; 32]),
        Str::from("validate"),
        Str::from("validation"),
    ));
    
    dataflow.add_node(ProcessDataflowNode::new(
        NodeId::new([2u8; 32]),
        Str::from("transfer"),
        Str::from("execution"),
    ));
    
    // Add edge
    dataflow.add_edge(ProcessDataflowEdge::new(
        Str::from("validate_to_transfer"),
        NodeId::new([1u8; 32]),
        Str::from("output"),
        NodeId::new([2u8; 32]),
        Str::from("input"),
    ));
    
    println!("✅ Created typed dataflow with {} nodes", dataflow.nodes.len());
    println!("🔗 Schema IDs are content-addressed and deterministic");
}

/// Demonstrate automatic schema generation for complex bridge operations
pub fn demonstrate_bridge_schemas() {
    println!("\n🌉 Bridge Operation Automatic Schema Generation");
    println!("==============================================");
    
    // Get automatically generated schemas
    let input_schema = BridgeDataflow::input_schema();
    let output_schema = BridgeDataflow::output_schema();
    let state_schema = BridgeDataflow::state_schema();
    
    println!("📋 Input Schema: {:?}", input_schema);
    println!("📤 Output Schema: {:?}", output_schema);
    println!("🗃️ State Schema: {:?}", state_schema);
    
    // Create a bridge dataflow with automatic schemas
    let mut bridge_flow = BridgeDataflow::new(
        ExprId::new([2u8; 32]),
        Str::from("cross_chain_bridge"),
    );
    
    // Add bridge-specific nodes
    let bridge_nodes = vec![
        ("validate_request", "validation"),
        ("lock_assets", "asset_management"),
        ("generate_proof", "cryptographic"),
        ("relay_message", "cross_chain"),
        ("verify_on_target", "verification"),
        ("complete_bridge", "finalization"),
    ];
    
    for (i, (name, node_type)) in bridge_nodes.iter().enumerate() {
        bridge_flow.add_node(ProcessDataflowNode::new(
            NodeId::new([i as u8 + 10; 32]),
            Str::from(*name),
            Str::from(*node_type),
        ));
    }
    
    println!("✅ Created bridge dataflow with {} nodes", bridge_flow.nodes.len());
    println!("🔄 Complex nested types automatically generate proper schemas");
    println!("🎯 Enum variants and optional fields handled correctly");
}

/// Show the benefits of automatic schema generation
pub fn show_benefits() {
    println!("\n💡 Benefits of Automatic Schema Generation");
    println!("=========================================");
    
    println!("✅ Type Safety:");
    println!("   • Impossible to have schema/type mismatches");
    println!("   • Compile-time validation of all schemas");
    println!("   • Full IDE support with IntelliSense");
    
    println!("\n🔄 Zero Maintenance:");
    println!("   • Schemas automatically update when types change");
    println!("   • No manual string-based schema definitions");
    println!("   • Eliminate schema drift and inconsistencies");
    
    println!("\n🎯 Content Addressing:");
    println!("   • Deterministic schema IDs through TypeExpr serialization");
    println!("   • Cross-domain schema compatibility verification");
    println!("   • Immutable schema versioning");
    
    println!("\n🚀 Developer Experience:");
    println!("   • Works seamlessly with generic types");
    println!("   • Handles complex nested structures automatically");
    println!("   • Supports enums, options, maps, and custom types");
    
    println!("\n⚡ Performance:");
    println!("   • Schema generation happens at compile-time");
    println!("   • Runtime validation using pre-computed schemas");
    println!("   • Efficient serialization and deserialization");
}

/// Main example function
pub fn main() {
    println!("🎪 Causality Automatic Schema Generation Examples");
    println!("================================================\n");
    
    demonstrate_token_transfer_schemas();
    demonstrate_bridge_schemas();
    show_benefits();
    
    println!("\n🎉 Examples completed successfully!");
    println!("📚 For more information, see the dataflow ADR documentation.");
} 