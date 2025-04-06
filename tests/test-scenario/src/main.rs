// Test scenario loader
//
// This crate provides utilities to load and run test scenarios

use std::fs;
use std::path::Path;

fn main() {
    println!("Test Scenario Loader");
    
    // Load the basic scenario
    let scenario_path = Path::new("tests/test-scenario/basic_scenario.toml");
    if scenario_path.exists() {
        match fs::read_to_string(scenario_path) {
            Ok(contents) => {
                println!("Loaded scenario:");
                println!("{}", contents);
            },
            Err(e) => {
                eprintln!("Error reading scenario file: {}", e);
            }
        }
    } else {
        eprintln!("Scenario file not found");
    }
} 