// EVM Domain Module
//
// This module contains implementations for interacting with EVM-compatible chains.

mod adapter;
mod types;

pub use adapter::EthereumAdapter;
pub use adapter::EthereumConfig;
 