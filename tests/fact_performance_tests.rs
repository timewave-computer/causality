// Performance tests for fact replay and simulation
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use causality::domain::fact::replay::{FactReplayEngine, ReplayOptions};
use causality::domain::fact::simulation::{FactSimulator, SimulationOptions};
use causality::error::Result;
use causality::log::fact_types::{FactType, RegisterFact};
use causality::types::{BlockHash, BlockHeight, DomainId, Timestamp};

// Helper function to create a vector of test balance facts
fn create_balance_facts(count: usize, domain_id: &DomainId) -> Vec<FactType> {
    let mut facts = Vec::with_capacity(count);

    for i in 0..count {
        let address = format!("address{}", i);
        let amount = format!("{}", 1000 + i * 10);
        let timestamp = 1600000000 + (i as u64 * 10);

        facts.push(FactType::BalanceFact {
            domain_id: domain_id.clone(),
            address,
            amount,
            token: None,
            block_height: Some(1000 + i as u64),
            block_hash: Some(vec![1, 2, 3, 4]),
            timestamp: Some(timestamp),
            proof_data: Some(vec![5, 6, 7, 8]),
            metadata: HashMap::new(),
        });
    }

    facts
}

// Helper function to create a vector of test register facts
fn create_register_facts(count: usize, domain_id: &DomainId) -> Vec<FactType> {
    let mut facts = Vec::with_capacity(count);

    // Create register facts (creation, updates, transfers)
    for i in 0..count {
        let register_id = format!("register{}", i);
        let owner = format!("owner{}", i);
        let timestamp = 1600000000 + (i as u64 * 10);

        // Register creation
        facts.push(FactType::RegisterFact(RegisterFact::RegisterCreation {
            domain_id: domain_id.clone(),
            register_id: register_id.clone(),
            owner: owner.clone(),
            register_type: Some("token".to_string()),
            initial_value: Some("100".to_string()),
            block_height: Some(1000 + i as u64),
            block_hash: Some(vec![1, 2, 3, 4]),
            timestamp: Some(timestamp),
            proof_data: Some(vec![5, 6, 7, 8]),
            metadata: HashMap::new(),
        }));

        // Register update
        facts.push(FactType::RegisterFact(RegisterFact::RegisterUpdate {
            domain_id: domain_id.clone(),
            register_id: register_id.clone(),
            new_value: format!("{}", 200 + i * 10),
            previous_value: Some("100".to_string()),
            updater: Some(owner.clone()),
            block_height: Some(1001 + i as u64),
            block_hash: Some(vec![2, 3, 4, 5]),
            timestamp: Some(timestamp + 1),
            proof_data: Some(vec![5, 6, 7, 8]),
            metadata: HashMap::new(),
        }));

        if i % 2 == 0 && i > 0 {
            // For every other register, add a transfer
            let new_owner = format!("owner{}", i - 1);

            facts.push(FactType::RegisterFact(RegisterFact::RegisterTransfer {
                domain_id: domain_id.clone(),
                register_id,
                from: owner,
                to: new_owner,
                block_height: Some(1002 + i as u64),
                block_hash: Some(vec![3, 4, 5, 6]),
                timestamp: Some(timestamp + 2),
                proof_data: Some(vec![5, 6, 7, 8]),
                metadata: HashMap::new(),
            }));
        }
    }

    facts
}

// Helper function to measure execution time
fn measure_time<F, T>(func: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = func();
    let duration = start.elapsed();

    (result, duration)
}

// Fact replay engine performance test
#[tokio::test]
async fn test_fact_replay_performance() -> Result<()> {
    // Parameters
    let small_count = 100;
    let medium_count = 500;
    let large_count = 1000;

    let domain_id = DomainId::new("test-domain");

    // Create fact sets of different sizes
    let small_facts = create_balance_facts(small_count, &domain_id);
    let medium_facts = create_balance_facts(medium_count, &domain_id);
    let large_facts = create_balance_facts(large_count, &domain_id);

    // Initialize replay engine
    let replay_engine = FactReplayEngine::new();

    // Measure replay time for small fact set
    println!("Testing replay performance for {} facts...", small_count);
    let (_, small_duration) = measure_time(|| {
        let result = replay_engine.replay(&small_facts, ReplayOptions::default());
        assert!(result.is_ok());
        result
    });
    println!("Time to replay {} facts: {:?}", small_count, small_duration);

    // Measure replay time for medium fact set
    println!("Testing replay performance for {} facts...", medium_count);
    let (_, medium_duration) = measure_time(|| {
        let result = replay_engine.replay(&medium_facts, ReplayOptions::default());
        assert!(result.is_ok());
        result
    });
    println!(
        "Time to replay {} facts: {:?}",
        medium_count, medium_duration
    );

    // Measure replay time for large fact set
    println!("Testing replay performance for {} facts...", large_count);
    let (_, large_duration) = measure_time(|| {
        let result = replay_engine.replay(&large_facts, ReplayOptions::default());
        assert!(result.is_ok());
        result
    });
    println!("Time to replay {} facts: {:?}", large_count, large_duration);

    // Check that performance scales reasonably
    // The replay time shouldn't increase exponentially with the number of facts
    // We expect it to be roughly linear or O(n log n)
    let small_time_per_fact = small_duration.as_micros() as f64 / small_count as f64;
    let medium_time_per_fact = medium_duration.as_micros() as f64 / medium_count as f64;
    let large_time_per_fact = large_duration.as_micros() as f64 / large_count as f64;

    println!(
        "Time per fact (small set): {:.2} microseconds",
        small_time_per_fact
    );
    println!(
        "Time per fact (medium set): {:.2} microseconds",
        medium_time_per_fact
    );
    println!(
        "Time per fact (large set): {:.2} microseconds",
        large_time_per_fact
    );

    // The time per fact should be relatively stable or increase very slowly
    // We allow for some overhead in larger datasets
    // A reasonable threshold is that time per fact for large sets shouldn't be more than 3x the small set
    assert!(large_time_per_fact < small_time_per_fact * 3.0, 
        "Performance degradation: large set time per fact ({:.2}) > 3x small set time per fact ({:.2})",
        large_time_per_fact, small_time_per_fact);

    Ok(())
}

// Fact simulation performance test
#[tokio::test]
async fn test_fact_simulation_performance() -> Result<()> {
    // Parameters
    let small_count = 50; // Fewer facts because simulation is more complex
    let medium_count = 200;
    let large_count = 500;

    let domain_id = DomainId::new("test-domain");

    // Create fact sets of different sizes
    // Use register facts since they're more complex and better test simulation
    let small_facts = create_register_facts(small_count, &domain_id);
    let medium_facts = create_register_facts(medium_count, &domain_id);
    let large_facts = create_register_facts(large_count, &domain_id);

    // Initialize simulator
    let simulator = FactSimulator::new();

    // Measure simulation time for small fact set
    println!(
        "Testing simulation performance for {} register facts...",
        small_facts.len()
    );
    let (_, small_duration) = measure_time(|| {
        let result = simulator.simulate(&small_facts, SimulationOptions::default());
        assert!(result.is_ok());
        result
    });
    println!(
        "Time to simulate {} register facts: {:?}",
        small_facts.len(),
        small_duration
    );

    // Measure simulation time for medium fact set
    println!(
        "Testing simulation performance for {} register facts...",
        medium_facts.len()
    );
    let (_, medium_duration) = measure_time(|| {
        let result = simulator.simulate(&medium_facts, SimulationOptions::default());
        assert!(result.is_ok());
        result
    });
    println!(
        "Time to simulate {} register facts: {:?}",
        medium_facts.len(),
        medium_duration
    );

    // Measure simulation time for large fact set
    println!(
        "Testing simulation performance for {} register facts...",
        large_facts.len()
    );
    let (_, large_duration) = measure_time(|| {
        let result = simulator.simulate(&large_facts, SimulationOptions::default());
        assert!(result.is_ok());
        result
    });
    println!(
        "Time to simulate {} register facts: {:?}",
        large_facts.len(),
        large_duration
    );

    // Check that performance scales reasonably
    let small_time_per_fact = small_duration.as_micros() as f64 / small_facts.len() as f64;
    let medium_time_per_fact = medium_duration.as_micros() as f64 / medium_facts.len() as f64;
    let large_time_per_fact = large_duration.as_micros() as f64 / large_facts.len() as f64;

    println!(
        "Time per fact (small set): {:.2} microseconds",
        small_time_per_fact
    );
    println!(
        "Time per fact (medium set): {:.2} microseconds",
        medium_time_per_fact
    );
    println!(
        "Time per fact (large set): {:.2} microseconds",
        large_time_per_fact
    );

    // The simulation is more complex, so we allow a bit more degradation
    // A reasonable threshold is that time per fact for large sets shouldn't be more than 5x the small set
    assert!(large_time_per_fact < small_time_per_fact * 5.0, 
        "Performance degradation: large set time per fact ({:.2}) > 5x small set time per fact ({:.2})",
        large_time_per_fact, small_time_per_fact);

    Ok(())
}

// Test for simulating register state changes with time-based simulation
#[tokio::test]
async fn test_time_based_simulation() -> Result<()> {
    let domain_id = DomainId::new("test-domain");
    let register_count = 10;
    let operations_per_register = 5;

    // Create a sequence of register facts with different timestamps
    let mut facts = Vec::new();

    for i in 0..register_count {
        let register_id = format!("register{}", i);
        let owner = format!("owner{}", i);
        let base_timestamp = 1600000000 + (i as u64 * 1000); // Spread out in time

        // Register creation
        facts.push(FactType::RegisterFact(RegisterFact::RegisterCreation {
            domain_id: domain_id.clone(),
            register_id: register_id.clone(),
            owner: owner.clone(),
            register_type: Some("token".to_string()),
            initial_value: Some("100".to_string()),
            block_height: Some(1000 + i as u64),
            block_hash: Some(vec![1, 2, 3, 4]),
            timestamp: Some(base_timestamp),
            proof_data: Some(vec![5, 6, 7, 8]),
            metadata: HashMap::new(),
        }));

        // Register updates at different timestamps
        for j in 1..=operations_per_register {
            let operation_timestamp = base_timestamp + (j as u64 * 10);
            let new_value = format!("{}", 100 + j * 10);

            facts.push(FactType::RegisterFact(RegisterFact::RegisterUpdate {
                domain_id: domain_id.clone(),
                register_id: register_id.clone(),
                new_value,
                previous_value: Some(format!("{}", 100 + (j - 1) * 10)),
                updater: Some(owner.clone()),
                block_height: Some(1000 + i as u64 + j as u64),
                block_hash: Some(vec![2, 3, 4, 5]),
                timestamp: Some(operation_timestamp),
                proof_data: Some(vec![5, 6, 7, 8]),
                metadata: HashMap::new(),
            }));
        }
    }

    // Initialize simulator with time-based options
    let simulator = FactSimulator::new();
    let simulation_options = SimulationOptions {
        time_based: true,
        max_time_delta: Some(Duration::from_secs(30)),
        ..SimulationOptions::default()
    };

    // Measure simulation time
    println!(
        "Testing time-based simulation for {} register facts...",
        facts.len()
    );
    let (simulation_result, duration) = measure_time(|| {
        let result = simulator.simulate(&facts, simulation_options);
        assert!(result.is_ok());
        result.unwrap()
    });
    println!(
        "Time to simulate {} register facts with time-based simulation: {:?}",
        facts.len(),
        duration
    );

    // Verify simulation results
    let states = simulation_result.states();
    println!(
        "Generated {} state snapshots during simulation",
        states.len()
    );

    // There should be multiple states created based on time
    assert!(
        states.len() > 1,
        "Time-based simulation should create multiple state snapshots"
    );

    // Check that the states are ordered by timestamp
    let mut prev_timestamp = 0;
    for state in states {
        let timestamp = state.timestamp();
        assert!(
            timestamp >= prev_timestamp,
            "States should be ordered by timestamp"
        );
        prev_timestamp = timestamp;
    }

    Ok(())
}

// Test for measuring throughput in facts per second
#[tokio::test]
async fn test_replay_throughput() -> Result<()> {
    // Parameters
    let fact_count = 10000; // Large number for accurate throughput measurement
    let domain_id = DomainId::new("test-domain");

    // Create a large set of simple facts
    println!("Generating {} facts for throughput test...", fact_count);
    let facts = create_balance_facts(fact_count, &domain_id);

    // Initialize replay engine
    let replay_engine = FactReplayEngine::new();

    // Measure replay time
    println!("Testing replay throughput...");
    let (_, duration) = measure_time(|| {
        let result = replay_engine.replay(&facts, ReplayOptions::default());
        assert!(result.is_ok());
        result
    });

    // Calculate throughput
    let throughput = fact_count as f64 / duration.as_secs_f64();
    println!("Replay throughput: {:.2} facts per second", throughput);

    // The minimum acceptable throughput depends on the system, but we can set a basic threshold
    let min_throughput = 1000.0; // At least 1000 facts per second
    assert!(
        throughput >= min_throughput,
        "Throughput too low: {:.2} facts per second (minimum: {:.2})",
        throughput,
        min_throughput
    );

    Ok(())
}

// Test for measuring latency of fact processing
#[tokio::test]
async fn test_fact_processing_latency() -> Result<()> {
    let domain_id = DomainId::new("test-domain");

    // Create a complex fact for latency testing
    let register_fact = FactType::RegisterFact(RegisterFact::RegisterCreation {
        domain_id: domain_id.clone(),
        register_id: "test-register".to_string(),
        owner: "test-owner".to_string(),
        register_type: Some("token".to_string()),
        initial_value: Some("100".to_string()),
        block_height: Some(1000),
        block_hash: Some(vec![1, 2, 3, 4]),
        timestamp: Some(1600000000),
        proof_data: Some(vec![5, 6, 7, 8]),
        metadata: HashMap::new(),
    });

    // Initialize replay engine and simulator
    let replay_engine = FactReplayEngine::new();
    let simulator = FactSimulator::new();

    // Measure replay latency (for a single fact)
    let (_, replay_latency) = measure_time(|| {
        let result = replay_engine.replay(&[register_fact.clone()], ReplayOptions::default());
        assert!(result.is_ok());
        result
    });

    // Measure simulation latency (for a single fact)
    let (_, simulation_latency) = measure_time(|| {
        let result = simulator.simulate(&[register_fact.clone()], SimulationOptions::default());
        assert!(result.is_ok());
        result
    });

    println!(
        "Replay latency for a single register fact: {:?}",
        replay_latency
    );
    println!(
        "Simulation latency for a single register fact: {:?}",
        simulation_latency
    );

    // Define maximum acceptable latencies
    let max_replay_latency = Duration::from_millis(10);
    let max_simulation_latency = Duration::from_millis(20);

    assert!(
        replay_latency <= max_replay_latency,
        "Replay latency too high: {:?} (maximum: {:?})",
        replay_latency,
        max_replay_latency
    );

    assert!(
        simulation_latency <= max_simulation_latency,
        "Simulation latency too high: {:?} (maximum: {:?})",
        simulation_latency,
        max_simulation_latency
    );

    Ok(())
}
