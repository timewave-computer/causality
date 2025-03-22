/// CosmWasm ZK-VM adapter module
/// 
/// This module provides an implementation of the DomainAdapter trait
/// for CosmWasm with ZK verification support. It allows compiling,
/// deploying, executing, proving, and verifying CosmWasm contracts
/// within a zero-knowledge proof system.

pub mod adapter;
pub mod effects;
pub mod types;
pub mod vm;
pub mod bridge;

#[cfg(test)]
pub mod tests;

// Re-export primary types for easier access
pub use adapter::CosmWasmZkAdapter;
pub use effects::{CompileEffect, ExecuteContractEffect, ProveEffect, VerifyEffect};
pub use types::{CosmWasmCallData, CosmWasmPublicInputs, CosmWasmZkProgram, VerificationResult, DetailedVerificationResult};
pub use self::vm::CosmWasmZkVm;
pub use self::bridge::CosmWasmZkBridge;

/// Create a default CosmWasm ZK adapter configured for standard use
pub fn default_adapter() -> CosmWasmZkAdapter {
    CosmWasmZkAdapter::new()
}

/// Create a CosmWasm ZK-VM bridge for integration with the ZK-VM system
pub fn create_cosmwasm_zk_bridge() -> CosmWasmZkBridge {
    CosmWasmZkBridge::new(default_adapter())
} 