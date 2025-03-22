/// CosmWasm ZK adapter providing zero-knowledge proofs for CosmWasm contract execution
#[cfg(feature = "cosmwasm_zk")]
pub mod cosmwasm_zk;

#[cfg(feature = "cosmwasm_zk")]
pub use cosmwasm_zk::CosmWasmZkAdapter; 