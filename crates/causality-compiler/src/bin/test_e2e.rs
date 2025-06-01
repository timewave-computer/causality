//! Simple test binary for E2E compilation

use causality_compiler::minimal_test;

fn main() {
    println!("Testing E2E compilation pipeline...");
    
    match minimal_test() {
        Ok(instructions) => {
            println!("✅ SUCCESS: Compiled '(pure 42)' to {} instructions:", instructions.len());
            for (i, instr) in instructions.iter().enumerate() {
                println!("  {}: {:?}", i, instr);
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            std::process::exit(1);
        }
    }
} 