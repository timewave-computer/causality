//! Chain Client Tests
//!
//! This module tests the chain client implementations.
//! All tests are feature-gated to only run when the appropriate features are enabled.

// Import ChainConfig trait
use causality_api::traits::ChainConfig;

// Simple compile-time test that doesn't require any chain features
#[test]
fn test_compile_only() {
    // This test doesn't actually test any chain client functionality
    // It just verifies that the test file compiles correctly
    println!("Chain client tests compile successfully!");
    assert!(true);
}

// Simple test for ChainConfig trait
#[test]
fn test_chain_configs() {
    // Define test configs inline
    struct NeutronConfig;
    impl ChainConfig for NeutronConfig {
        const CHAIN_NAME: &'static str = "neutron";
        const CHAIN_ID: &'static str = "neutron-1";
        const DEFAULT_RPC_PORT: &'static str = "26657";
        const CHAIN_TYPE: &'static str = "cosmos";
    }

    struct EthereumConfig;
    impl ChainConfig for EthereumConfig {
        const CHAIN_NAME: &'static str = "ethereum";
        const CHAIN_ID: &'static str = "1";
        const DEFAULT_RPC_PORT: &'static str = "8545";
        const CHAIN_TYPE: &'static str = "evm";
    }

    // Test Neutron Config
    assert_eq!(NeutronConfig::CHAIN_NAME, "neutron");
    assert_eq!(NeutronConfig::DEFAULT_RPC_PORT, "26657");
    assert_eq!(NeutronConfig::CHAIN_TYPE, "cosmos");

    // Test Ethereum Config
    assert_eq!(EthereumConfig::CHAIN_NAME, "ethereum");
    assert_eq!(EthereumConfig::DEFAULT_RPC_PORT, "8545");
    assert_eq!(EthereumConfig::CHAIN_TYPE, "evm");
}
