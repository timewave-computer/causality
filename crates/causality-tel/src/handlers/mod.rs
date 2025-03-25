// TEL handlers module
// Original file: src/tel/handlers/mod.rs

//! TEL handlers module
//!
//! This module organizes domain-specific TEL handlers for various effect types.

// Export domain-specific handler modules
pub mod evm;
pub mod cosmwasm;

// Re-export the main handler interfaces
pub use super::handlers::{
    TelHandler, ConstraintTelHandler, TransferTelHandler, 
    StorageTelHandler, QueryTelHandler, TelHandlerRegistry,
    TransferParams, StorageParams, QueryParams,
    TelCompiler, StandardTelCompiler
};

// Re-export domain-specific handlers
pub use evm::EvmTransferHandler;
pub use cosmwasm::CosmWasmTransferHandler;

/// Factory function to create a standard TEL handler registry
pub fn create_standard_handler_registry(
    domain_registry: std::sync::Arc<crate::domain::DomainRegistry>
) -> TelHandlerRegistry {
    use std::sync::Arc;
    
    // Create registry
    let mut registry = TelHandlerRegistry::new(domain_registry.clone());
    
    // Register EVM handlers
    let evm_transfer_handler = Arc::new(EvmTransferHandler::new(domain_registry.clone()));
    registry.register_handler(evm_transfer_handler);
    
    // Register CosmWasm handlers
    let cosmwasm_transfer_handler = Arc::new(CosmWasmTransferHandler::new(domain_registry.clone()));
    registry.register_handler(cosmwasm_transfer_handler);
    
    // Return the populated registry
    registry
}

/// Factory function to create a standard TEL compiler
pub fn create_standard_compiler(
    domain_registry: std::sync::Arc<crate::domain::DomainRegistry>
) -> std::sync::Arc<dyn TelCompiler> {
    // Create handler registry
    let registry = create_standard_handler_registry(domain_registry);
    
    // Create compiler
    std::sync::Arc::new(StandardTelCompiler::new(std::sync::Arc::new(registry)))
} 