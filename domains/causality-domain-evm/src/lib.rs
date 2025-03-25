// Domain adapters framework
// Original file: src/domain_adapters/mod.rs

/// CosmWasm ZK adapter providing zero-knowledge proofs for CosmWasm contract execution
#[cfg(feature = "cosmwasm_zk")]
pub mod cosmwasm_zk;

#[cfg(feature = "cosmwasm_zk")]
pub use cosmwasm_zk::CosmWasmZkAdapter;

/// ZK resource adapter module
#[cfg(feature = "cosmwasm_zk")]
pub mod zk_resource_adapter; 