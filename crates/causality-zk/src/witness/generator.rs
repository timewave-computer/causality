//! Witness Generation for ZK Circuits
//!
//! This module provides functionality for generating witnesses for ZK circuits.
//! It uses SSZ serialization to ensure consistent representation between
//! the runtime and ZK environments.

use causality_types::anyhow::Result;
use causality_types::{
    expr::value::ValueExpr,
    resource::Resource,
};
use sha2::{Digest, Sha256};

use super::ssz_input::SszCircuitInput;

/// A generator for ZK circuit witnesses
pub struct WitnessGenerator {
    /// The resources to include in the witness
    resources: Vec<Resource>,
    
    /// The value expressions to include in the witness
    values: Vec<ValueExpr>,
    
    /// Additional raw inputs to include in the witness
    raw_inputs: Vec<Vec<u8>>,
}

impl WitnessGenerator {
    /// Create a new witness generator
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            values: Vec::new(),
            raw_inputs: Vec::new(),
        }
    }
    
    /// Add a resource to the witness
    pub fn add_resource(&mut self, resource: Resource) -> &mut Self {
        self.resources.push(resource);
        self
    }
    
    /// Add a value expression to the witness
    pub fn add_value_expr(&mut self, value_expr: ValueExpr) -> &mut Self {
        self.values.push(value_expr);
        self
    }
    
    /// Add raw bytes to the witness
    pub fn add_raw_input(&mut self, bytes: Vec<u8>) -> &mut Self {
        self.raw_inputs.push(bytes);
        self
    }
    
    /// Generate circuit inputs using SSZ serialization
    pub fn generate_circuit_inputs(&self) -> Result<Vec<SszCircuitInput>> {
        let mut inputs = Vec::new();
        
        // Add resources
        for resource in &self.resources {
            inputs.push(SszCircuitInput::from_resource(resource));
        }
        
        // Add value expressions
        for value in &self.values {
            inputs.push(SszCircuitInput::from_value_expr(value)?);
        }
        
        // Add raw inputs
        for bytes in &self.raw_inputs {
            inputs.push(SszCircuitInput::from_raw_bytes(bytes));
        }
        
        Ok(inputs)
    }
    
    /// Generate a Merkle root for all the inputs
    pub fn generate_merkle_root(&self) -> Result<[u8; 32]> {
        let inputs = self.generate_circuit_inputs()?;
        
        // If there are no inputs, return a zero hash
        if inputs.is_empty() {
            return Ok([0u8; 32]);
        }
        
        // If there's only one input, return its hash
        if inputs.len() == 1 {
            return Ok(inputs[0].hash);
        }
        
        // Otherwise, build a Merkle tree from the input hashes
        let mut hasher = Sha256::new();
        
        // Collect all the hashes
        let hashes: Vec<[u8; 32]> = inputs.iter().map(|input| input.hash).collect();
        
        // Build a simple Merkle tree
        // (This is a simplified implementation for demonstration purposes)
        let mut level_hashes = hashes;
        
        while level_hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            // Process pairs of hashes
            for i in 0..(level_hashes.len() / 2) {
                let left = level_hashes[i * 2];
                let right = level_hashes[i * 2 + 1];
                
                hasher.update(left);
                hasher.update(right);
                
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&hasher.finalize_reset());
                
                next_level.push(hash);
            }
            
            // If there's an odd number of hashes, include the last one
            if level_hashes.len() % 2 == 1 {
                next_level.push(level_hashes[level_hashes.len() - 1]);
            }
            
            level_hashes = next_level;
        }
        
        Ok(level_hashes[0])
    }
}

impl Default for WitnessGenerator {
    fn default() -> Self {
        Self::new()
    }
} 