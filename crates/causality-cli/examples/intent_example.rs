//-----------------------------------------------------------------------------
// Intent CLI Usage Example
//-----------------------------------------------------------------------------

//
// This example demonstrates how to use the Intent CLI commands to submit

// and query intents with a mock blockchain.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use causality_types::primitive::ids::{EntityId, DomainId, ExprId, AsId};
use causality_types::core::Intent;
use causality_types::primitive::string::Str;
use causality_types::core::time::Timestamp;
use causality_types::resource::ResourceFlow;
use causality_types::serialization::Encode;

fn main() -> anyhow::Result<()> {
    // Create a simple intent for testing
    let intent = create_test_intent();

    // Save the intent to a file
    let intent_path = save_intent_to_file(&intent)?;

    println!(
        "Created test intent and saved to: {}",
        intent_path.display()
    );
    println!("\nTo submit this intent to a mock blockchain, run:");
    println!(
        "  cargo run --bin causality -- intent submit --file {}",
        intent_path.display()
    );

    println!(
        "\nAfter submitting, you'll receive an Intent ID that you can use to query:"
    );
    println!("  cargo run --bin causality -- intent query --id <INTENT_ID>");

    Ok(())
}

/// Create a simple test intent
fn create_test_intent() -> Intent {
    // Create some test resource flows
    let domain_id = DomainId::null(); // Use null domain for example
    
    let input_flow = ResourceFlow::new(Str::from("test_input_resource"), 1, domain_id);
    let output_flow = ResourceFlow::new(Str::from("test_output_resource"), 1, domain_id);
    
    let intent_id = EntityId::null(); // Use null ID for example

    Intent {
        id: intent_id,
        name: Str::from("transfer_tokens"),
        domain_id,
        priority: 100,
        inputs: vec![input_flow],
        outputs: vec![output_flow],
        expression: Some(ExprId::from([5u8; 32])),
        timestamp: Timestamp::now(),
        hint: Some(ExprId::new([1u8; 32])),
    }
}

/// Save an intent to an SSZ-serialized file
fn save_intent_to_file(intent: &Intent) -> anyhow::Result<std::path::PathBuf> {
    // Serialize the intent using SSZ
    let serialized = intent.as_ssz_bytes();

    // Create the examples directory if it doesn't exist
    let examples_dir = Path::new("./examples");
    if !examples_dir.exists() {
        std::fs::create_dir_all(examples_dir)?;
    }

    // Write to file
    let intent_path = examples_dir.join("test_intent.ssz");
    let mut file = File::create(&intent_path)?;
    file.write_all(&serialized)?;

    Ok(intent_path)
}
