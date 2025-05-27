//! Causality API: External interfaces for the Causality framework.
//!
//! This crate provides standardized interfaces for external communication,
//! blockchain integration, and ZK coprocessor interaction with bounded types
//! for ZK compatibility and deterministic behavior.
//!
//! ## Module Organization
//!
//! * **API Gateway**: HTTP, gRPC, and WebSocket interfaces (`gateway`, `protocols`)
//! * **Blockchain Integration**: Chain interfaces and implementations (`chain`)
//! * **ZK Coprocessor**: Interaction with zero-knowledge proving services (`coprocessor`)
//! * **Core Models**: Data structures for API interactions (`models`)
//! * **Core Traits**: Interface definitions for system components (`traits`)
//! * **FFI Interface**: Foreign function interface for cross-language integration (`ffi`)
//! * **Error Handling**: Error types and handling logic (`error_handler`)

//-----------------------------------------------------------------------------
// Core API Gateway Module
//-----------------------------------------------------------------------------

pub mod gateway;
pub mod protocols;

//-----------------------------------------------------------------------------
// Serialization Utilities
//-----------------------------------------------------------------------------

pub mod serialization;

// Re-export gateway interfaces
pub use gateway::{ApiError, ApiGateway, ApiGatewayFactory};

// Re-export protocol implementations
pub use protocols::{
    // gRPC
    GrpcApiConfig,
    GrpcApiGateway,
    GrpcRequest,
    GrpcResponse,
    // REST
    RestApiConfig,
    RestApiGateway,
    RestRequest,
    RestResponse,
    // WebSocket
    WebSocketApiConfig,
    WebSocketApiGateway,
    WebSocketMessage,
    WebSocketMessageType,
};

//-----------------------------------------------------------------------------
// Blockchain Integration
//-----------------------------------------------------------------------------

pub mod chain;

// Re-export chain interface types
pub use chain::types::{
    ApiError as ChainError,
    BlockId,
    // Transaction types
    CausalityTransaction,
    CausalityTransactionId,
    ChainClient,
    ChainClientError,
    // Core types
    ChainId,
    TransactionStatus,
};

// Export from chain connector and intent modules
pub use chain::connector::{ChainConnector, ChainConnectorFactory};
pub use chain::intent::{
    ChainIntentHandler, IntentHandler, IntentMetadata as ChainIntentMetadata,
    IntentQueryResult, IntentStatus,
};

// Export ValenceChainClient and the wrapper types for specific blockchains
#[cfg(feature = "neutron")]
pub use chain::valence_client::cosmos::CosmosValenceChainClient;
#[cfg(feature = "ethereum")]
pub use chain::valence_client::evm::EvmValenceChainClient;
pub use chain::valence_client::ValenceChainClient;

// Export factory functions for creating chain clients
#[cfg(feature = "ethereum")]
pub use chain::factory::create_ethereum_client;
#[cfg(feature = "neutron")]
pub use chain::factory::create_neutron_client;

// Re-export core ID types from causality-types
pub use causality_types::primitive::ids::{
    DataId, DomainId, EdgeId, EffectId, ExprId, GraphId, HandlerId,
    IntentId, NodeId, ResourceId, SubgraphId, TransactionId, TypeExprId,
    ValueExprId,
};

//-----------------------------------------------------------------------------
// Models and Traits
//-----------------------------------------------------------------------------

pub mod models;
pub mod traits;

// Re-export core traits
pub use traits::{
    // Chain traits
    ChainConfig,
    ClientBuilder,

    // Client traits
    ClientConfig,
    IntentQuery,
    IntentSubmission,

    Query,
    // Transaction traits
    Transaction,
};

//-----------------------------------------------------------------------------
// ZK Coprocessor Interface
//-----------------------------------------------------------------------------

pub mod coprocessor;
pub mod debug;
pub mod error_handler;
pub mod mock;

// Re-export coprocessor interfaces
pub use coprocessor::{
    // Generator interfaces
    generator::{ProofGenerator, ProofGeneratorFactory},

    // Integration
    integration::{ProofContext, ProofStorage, ZkIntegration},

    // Monitoring
    monitor::{CoprocessorMonitor, HealthStatus, PerformanceMetrics},

    // Connection management
    pool::CoprocessorConnectionPool,
    retry::{RetryConfig, RetryableProofGenerator},

    // Core types
    types::{CoprocessorId, Proof, ProofRequest, ProofRequestId, ProofStatus},

    // Valence coprocessor client
    valence_client::{
        create_coprocessor_client, create_coprocessor_client_with_socket,
        ValenceCoprocessorClientWrapper,
    },
};

// Re-export testing utilities
pub use coprocessor::integration::InMemoryProofStorage;

//-----------------------------------------------------------------------------
// Foreign Function Interface (FFI)
//-----------------------------------------------------------------------------

#[cfg(feature = "ffi")]
pub mod ffi;

// Re-export FFI interfaces when the feature is enabled
#[cfg(feature = "ffi")]
pub use ffi::{
    // Core FFI functions
    value_expr_to_ocaml, value_expr_from_ocaml,
    resource_to_ocaml, resource_from_ocaml,
    handler_to_ocaml, handler_from_ocaml,
    hex_from_ocaml, hex_to_ocaml,
    
    // C bindings for direct FFI use
    free_string, free_bytes,
    value_expr_from_ocaml as c_value_expr_from_ocaml,
    value_expr_to_ocaml as c_value_expr_to_ocaml,
    free_value_expr,
    resource_from_ocaml as c_resource_from_ocaml,
    resource_to_ocaml as c_resource_to_ocaml,
    free_resource,
    handler_from_ocaml as c_handler_from_ocaml,
    handler_to_ocaml as c_handler_to_ocaml,
    free_handler,
    value_expr_from_hex,
    value_expr_to_hex,
};

//-----------------------------------------------------------------------------
// Error Handling
//-----------------------------------------------------------------------------

// Re-export error handling interfaces
pub use error_handler::{
    ApiErrorHandler, ErrorDetails, ErrorResponseBuilder, ErrorStats,
};

//-----------------------------------------------------------------------------
// Optimization API
//-----------------------------------------------------------------------------

pub mod optimization;

// Re-export optimization API interfaces
pub use optimization::{
    StrategyManagementApi, StrategyInfo, StrategyPerformanceMetrics,
    StrategyEvaluationRequest, StrategyEvaluationResponse, TestScenarioConfig,
    EvaluationParameters, EvaluationMetrics, ConfigurationUpdateRequest,
    ConfigurationUpdateResponse, PerformanceMonitoringRequest, PerformanceMonitoringResponse,
    MetricType, AggregationLevel, ValidationSeverity,
};
