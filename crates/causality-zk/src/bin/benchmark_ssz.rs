//! SSZ Serialization Benchmark Tool
//!
//! This binary provides benchmarks for SSZ serialization in ZK proofs,
//! comparing it with the previous ssz-based implementation.

use std::time::Instant;
use causality_types::anyhow::Result;
use causality_types::{
    primitive::{ids::{DomainId, EntityId}, string::Str, time::Timestamp, number::Number},
    expression::value::ValueExpr,
    resource::types::Resource,
};
use causality_zk::witness::WitnessGenerator;

fn main() -> Result<()> {
    println!("SSZ Serialization Benchmark for ZK Proof Generation");
    println!("===================================================");
    
    // Create a simple resource for benchmarking using current Resource structure
    let resource = Resource {
        id: EntityId::new([1u8; 32]),
        name: Str::from("benchmark_resource"),
        domain_id: DomainId::new([2u8; 32]),
        resource_type: Str::from("token"),
        quantity: 100,
        timestamp: Timestamp::now(),
    };
    
    // Test SSZ serialization
    let iterations = 10_000;
    println!("\nRunning SSZ serialization benchmark with {} iterations...", iterations);
    
    let start = Instant::now();
    let mut size = 0;
    
    for _ in 0..iterations {
        let mut generator = WitnessGenerator::new();
        generator.add_resource(resource.clone());
        
        // Add some test values using current ValueExpr API
        for i in 0..10 {
            let value = ValueExpr::Number(Number::Integer(i));
            generator.add_value_expr(value);
        }
        
        // Generate circuit inputs
        let inputs = generator.generate_circuit_inputs()?;
        size += inputs.iter().map(|input| input.serialized_bytes.len()).sum::<usize>();
    }
    
    let duration = start.elapsed();
    let throughput = (size * 1000) / (duration.as_millis() as usize * 1024 * 1024);
    
    println!("SSZ Serialization Results:");
    println!("  Time: {:?}", duration);
    println!("  Total data: {} bytes", size);
    println!("  Throughput: {:.2} MB/s", throughput);
    
    // Add comparison with ssz if needed in the future
    
    Ok(())
} 