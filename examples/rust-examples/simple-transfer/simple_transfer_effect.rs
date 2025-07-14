//! Simple Transfer Effect Example
//!
//! This example demonstrates how to define, test, and mock a basic token transfer effect
//! using the Causality Toolkit's algebraic testing framework.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use causality_toolkit::{
    effects::{AlgebraicEffect, EffectCategory, FailureMode},
    schema::{EffectSchema, ParameterDef, TypeDef},
    mocks::{MockGenerator, MockStrategy, MockHandler},
    testing::{TestGenerator, PropertyTest, TestCase, CompositionTest},
    core::{EffectResult, ContentHash},
};

/// A simple token transfer effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTransfer {
    /// Recipient address
    pub to: String,
    /// Amount to transfer (in wei/smallest unit)
    pub amount: u64,
    /// Optional gas limit
    pub gas_limit: Option<u64>,
}

/// Transfer receipt returned on success
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferReceipt {
    /// Transaction hash
    pub tx_hash: String,
    /// Block number
    pub block_number: u64,
    /// Gas used
    pub gas_used: u64,
    /// Effective gas price
    pub gas_price: u64,
}

/// Transfer error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferError {
    /// Insufficient balance to complete transfer
    InsufficientBalance { required: u64, available: u64 },
    /// Invalid recipient address
    InvalidAddress(String),
    /// Network error
    NetworkError(String),
    /// Gas limit too low
    GasLimitTooLow { provided: u64, required: u64 },
}

impl AlgebraicEffect for SimpleTransfer {
    type Success = TransferReceipt;
    type Error = TransferError;
    
    fn category(&self) -> EffectCategory {
        EffectCategory::Asset
    }
    
    fn description(&self) -> &str {
        "Transfer tokens to a specified address"
    }
    
    fn failure_modes(&self) -> &[FailureMode] {
        &[
            FailureMode::InvalidInput,
            FailureMode::InsufficientBalance,
            FailureMode::NetworkError,
            FailureMode::GasLimitExceeded,
        ]
    }
    
    fn schema(&self) -> EffectSchema {
        EffectSchema::new(
            vec![
                ParameterDef::new("to", TypeDef::String),
                ParameterDef::new("amount", TypeDef::U64),
                ParameterDef::new("gas_limit", TypeDef::Option(Box::new(TypeDef::U64))),
            ],
            Some(TypeDef::Custom("TransferReceipt".to_string())),
        )
    }
    
    /// Estimate gas required for this transfer
    fn estimated_gas(&self) -> u64 {
        21000 // Standard ETH transfer
    }
    
    /// Validate transfer parameters
    fn validate(&self) -> Result<(), Self::Error> {
        // Check address format (simplified)
        if self.to.is_empty() || !self.to.starts_with("0x") {
            return Err(TransferError::InvalidAddress(self.to.clone()));
        }
        
        // Check amount is non-zero
        if self.amount == 0 {
            return Err(TransferError::InvalidAddress("Amount must be greater than 0".to_string()));
        }
        
        // Check gas limit if provided
        if let Some(gas_limit) = self.gas_limit {
            let required_gas = self.estimated_gas();
            if gas_limit < required_gas {
                return Err(TransferError::GasLimitTooLow {
                    provided: gas_limit,
                    required: required_gas,
                });
            }
        }
        
        Ok(())
    }
}

/// Example: Basic effect usage
fn basic_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Transfer Effect Usage ===");
    
    // Create a simple transfer
    let transfer = SimpleTransfer {
        to: "0x742B3B13Ca4700b4E0FF8f8E54ACa6bb9DF3A8D8".to_string(),
        amount: 1_000_000_000_000_000_000, // 1 ETH in wei
        gas_limit: Some(21000),
    };
    
    // Show effect properties
    println!("Effect Category: {:?}", transfer.category());
    println!("Description: {}", transfer.description());
    println!("Schema ID: {}", transfer.schema().id());
    println!("Estimated Gas: {}", transfer.estimated_gas());
    
    // Validate the transfer
    match transfer.validate() {
        Ok(()) => println!(" Transfer validation passed"),
        Err(e) => println!(" Transfer validation failed: {:?}", e),
    }
    
    Ok(())
}

/// Example: Mock generation and testing
async fn mock_testing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Mock Generation and Testing ===");
    
    let transfer = SimpleTransfer {
        to: "0x742B3B13Ca4700b4E0FF8f8E54ACa6bb9DF3A8D8".to_string(),
        amount: 500_000_000_000_000_000, // 0.5 ETH
        gas_limit: Some(25000),
    };
    
    let mock_generator = MockGenerator::new();
    
    // Generate different mock strategies
    println!("Generating mock handlers...");
    
    // Always succeed mock
    let always_succeed = mock_generator.generate_handler(&transfer, MockStrategy::AlwaysSucceed)?;
    println!(" Always succeed mock generated");
    
    // Probabilistic mock
    let probabilistic = mock_generator.generate_handler(
        &transfer, 
        MockStrategy::Probabilistic { success_rate: 0.8 }
    )?;
    println!(" Probabilistic mock generated (80% success rate)");
    
    // Latency mock
    let latency_mock = mock_generator.generate_handler(
        &transfer,
        MockStrategy::Latency { base_delay_ms: 100, variance_ms: 50 }
    )?;
    println!(" Latency mock generated (100ms ± 50ms delay)");
    
    // Blockchain simulation mock
    let blockchain_mock = mock_generator.generate_handler(&transfer, MockStrategy::Blockchain)?;
    println!(" Blockchain simulation mock generated");
    
    Ok(())
}

/// Example: Automatic test generation
fn test_generation_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Automatic Test Generation ===");
    
    let transfer = SimpleTransfer {
        to: "0x742B3B13Ca4700b4E0FF8f8E54ACa6bb9DF3A8D8".to_string(),
        amount: 1_000_000_000_000_000_000,
        gas_limit: Some(21000),
    };
    
    let test_generator = TestGenerator::new();
    
    // Generate comprehensive test suite
    println!("Generating comprehensive test suite...");
    let test_suite = test_generator.generate_test_suite(&transfer, 50)?;
    println!(" Generated {} tests in test suite", test_suite.len());
    
    // Generate specific test types
    println!("Generating specific test types...");
    
    let boundary_tests = test_generator.generate_boundary_tests(&transfer, 10)?;
    println!(" Generated {} boundary tests", boundary_tests.len());
    
    let invalid_tests = test_generator.generate_invalid_tests(&transfer, 10)?;
    println!(" Generated {} invalid input tests", invalid_tests.len());
    
    let property_tests = test_generator.generate_property_tests(&transfer, 20)?;
    println!(" Generated {} property tests", property_tests.len());
    
    // Show examples of generated tests
    println!("\nExample boundary tests:");
    for (i, test) in boundary_tests.iter().take(3).enumerate() {
        println!("  {}: {}", i + 1, test.name());
    }
    
    println!("\nExample invalid tests:");
    for (i, test) in invalid_tests.iter().take(3).enumerate() {
        println!("  {}: {}", i + 1, test.name());
    }
    
    Ok(())
}

/// Example: Property-based testing
fn property_testing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Property-Based Testing ===");
    
    let transfer = SimpleTransfer {
        to: "0x742B3B13Ca4700b4E0FF8f8E54ACa6bb9DF3A8D8".to_string(),
        amount: 1_000_000_000_000_000_000,
        gas_limit: Some(21000),
    };
    
    // Define conservation property
    let conservation_property = PropertyTest::ConservationLaw {
        description: "Total token supply must be conserved".to_string(),
        invariant: Box::new(|before: &HashMap<String, u64>, after: &HashMap<String, u64>| {
            let before_total: u64 = before.values().sum();
            let after_total: u64 = after.values().sum();
            before_total == after_total
        }),
    };
    
    // Define non-negative balance property
    let non_negative_property = PropertyTest::NonNegativeBalance {
        description: "Balances must never be negative".to_string(),
    };
    
    // Define slippage bounds property (not applicable to simple transfers, but shown for completeness)
    let slippage_property = PropertyTest::SlippageBounds {
        description: "Transfer should complete without slippage".to_string(),
        max_slippage: 0.0, // No slippage for direct transfers
    };
    
    println!(" Defined conservation law property");
    println!(" Defined non-negative balance property"); 
    println!(" Defined slippage bounds property");
    
    println!("Properties can be verified against generated test cases");
    
    Ok(())
}

/// Example: Composition testing
fn composition_testing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Composition Testing ===");
    
    // Create a sequence of transfers
    let transfer1 = SimpleTransfer {
        to: "0xAlice...".to_string(),
        amount: 1_000_000_000_000_000_000, // 1 ETH
        gas_limit: Some(21000),
    };
    
    let transfer2 = SimpleTransfer {
        to: "0xBob...".to_string(),
        amount: 500_000_000_000_000_000, // 0.5 ETH
        gas_limit: Some(21000),
    };
    
    let transfer3 = SimpleTransfer {
        to: "0xCharlie...".to_string(),
        amount: 250_000_000_000_000_000, // 0.25 ETH
        gas_limit: Some(21000),
    };
    
    // Create sequential composition
    let sequential_composition = CompositionTest::sequential(vec![
        Box::new(transfer1) as Box<dyn AlgebraicEffect<Success=TransferReceipt, Error=TransferError>>,
        Box::new(transfer2),
        Box::new(transfer3),
    ]);
    
    // Create parallel composition
    let parallel_composition = CompositionTest::parallel(vec![
        Box::new(transfer1.clone()),
        Box::new(transfer2.clone()),
        Box::new(transfer3.clone()),
    ]);
    
    println!(" Created sequential composition (3 transfers in order)");
    println!(" Created parallel composition (3 transfers simultaneously)");
    
    // Show composition properties
    println!("Sequential composition ensures transfers happen in order");
    println!("Parallel composition tests concurrent execution");
    
    Ok(())
}

/// Example: Content addressing and schema versioning
fn content_addressing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Content Addressing and Schema Versioning ===");
    
    let transfer_v1 = SimpleTransfer {
        to: "0x742B3B13Ca4700b4E0FF8f8E54ACa6bb9DF3A8D8".to_string(),
        amount: 1_000_000_000_000_000_000,
        gas_limit: Some(21000),
    };
    
    let transfer_v2 = SimpleTransfer {
        to: "0x742B3B13Ca4700b4E0FF8f8E54ACa6bb9DF3A8D8".to_string(),
        amount: 1_000_000_000_000_000_000,
        gas_limit: Some(21000),
    };
    
    // Schema IDs are content-addressed
    let schema_id_v1 = transfer_v1.schema().id();
    let schema_id_v2 = transfer_v2.schema().id();
    
    println!("Schema ID V1: {}", schema_id_v1);
    println!("Schema ID V2: {}", schema_id_v2);
    
    // Identical effects produce identical schema IDs
    assert_eq!(schema_id_v1, schema_id_v2);
    println!(" Identical effects produce identical schema IDs");
    
    // Show deterministic generation
    let schema_id_again = transfer_v1.schema().id();
    assert_eq!(schema_id_v1, schema_id_again);
    println!(" Schema ID generation is deterministic");
    
    Ok(())
}

/// Main example runner
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Causality Toolkit - Simple Transfer Effect Example");
    println!("=====================================================");
    
    // Run all examples
    basic_usage_example()?;
    mock_testing_example().await?;
    test_generation_example()?;
    property_testing_example()?;
    composition_testing_example()?;
    content_addressing_example()?;
    
    println!("\n All examples completed successfully!");
    println!("\nNext steps:");
    println!("1. Try running: `causality test-effects discover --detailed`");
    println!("2. Generate tests: `causality test-effects generate SimpleTransfer --count 20`");
    println!("3. Execute tests: `causality test-effects execute SimpleTransfer --mock-strategy blockchain`");
    
    Ok(())
} 