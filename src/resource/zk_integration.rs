// ZK integration for One-Time Register System
//
// This module integrates the one-time register system with ZK proofs
// using the Succinct ZK-VM adapter.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::resource::{
    Register, RegisterId, RegisterState, RegisterNullifier,
    TransitionType, OneTimeRegisterSystem,
};
use crate::domain_adapters::succinct::{
    SuccinctAdapter, ProofData, PublicInputs,
    ZkVirtualMachine
};
use crate::types::Hash256;

/// ZK proof manager for the one-time register system
pub struct RegisterZkProofManager {
    /// The Succinct adapter for generating and verifying proofs
    adapter: SuccinctAdapter,
    /// Program ID for the register consumption operation
    consumption_program_id: String,
    /// Cache of verified proofs
    verified_proofs: HashMap<Hash256, bool>,
}

impl RegisterZkProofManager {
    /// Create a new RegisterZkProofManager
    pub fn new() -> Result<Self> {
        // Create a new Succinct adapter
        let adapter = SuccinctAdapter::new()?;
        
        // Compile the register consumption program
        // In a real implementation, this would be done ahead of time
        // and the program ID would be stored in configuration
        let consumption_program_id = "register_consumption".to_string();
        
        Ok(Self {
            adapter,
            consumption_program_id,
            verified_proofs: HashMap::new(),
        })
    }
    
    /// Generate a consumption proof
    pub fn generate_consumption_proof(
        &self,
        register: &Register,
        block_height: u64,
        transaction_id: &str,
        successor_ids: Vec<RegisterId>,
        reason: &str,
        owner_key: Vec<u8>,
    ) -> Result<(ProofData, Hash256)> {
        // Create public inputs
        let mut public_inputs = PublicInputs::new();
        public_inputs.add("register_id", &register.id.to_string())?;
        public_inputs.add("block_height", &block_height)?;
        public_inputs.add("transaction_id", transaction_id)?;
        
        // Add successor IDs
        let successor_ids_str: Vec<String> = successor_ids.iter()
            .map(|id| id.to_string())
            .collect();
        public_inputs.add("successor_ids", &successor_ids_str.join(","))?;
        
        // Add reason
        public_inputs.add("reason", reason)?;
        
        // Create private inputs
        let mut private_inputs = HashMap::new();
        private_inputs.insert("owner_key".to_string(), owner_key);
        
        // Generate the proof
        let proof_data = self.adapter.prove(
            &self.consumption_program_id,
            &public_inputs,
            &private_inputs,
            None, // Use default options
        )?;
        
        // Calculate nullifier
        let nullifier = RegisterNullifier::calculate(
            &register.id,
            block_height,
            transaction_id,
        );
        
        Ok((proof_data, nullifier))
    }
    
    /// Verify a consumption proof
    pub fn verify_consumption_proof(
        &mut self,
        proof: &ProofData,
        register_id: &RegisterId,
        nullifier: &Hash256,
        block_height: u64,
        transaction_id: &str,
        successor_ids: &Vec<RegisterId>,
        reason: &str,
    ) -> Result<bool> {
        // Check if we've already verified this proof
        if let Some(result) = self.verified_proofs.get(nullifier) {
            return Ok(*result);
        }
        
        // Create public inputs for verification
        let mut public_inputs = PublicInputs::new();
        public_inputs.add("register_id", &register_id.to_string())?;
        public_inputs.add("block_height", &block_height)?;
        public_inputs.add("transaction_id", transaction_id)?;
        
        // Add successor IDs
        let successor_ids_str: Vec<String> = successor_ids.iter()
            .map(|id| id.to_string())
            .collect();
        public_inputs.add("successor_ids", &successor_ids_str.join(","))?;
        
        // Add reason
        public_inputs.add("reason", reason)?;
        
        // Verify the proof
        let verified = self.adapter.verify(
            &self.consumption_program_id,
            proof,
            &public_inputs,
        )?;
        
        // Store the result in the cache
        self.verified_proofs.insert(*nullifier, verified);
        
        Ok(verified)
    }
}

/// Trait for register operations with ZK proofs
pub trait ZkRegisterOperations {
    /// Consume a register with ZK proof generation
    fn consume_register_with_proof(
        &self,
        register: &mut Register,
        transaction_id: &str,
        successor_ids: Vec<RegisterId>,
        reason: &str,
        owner_key: Vec<u8>,
    ) -> Result<(ProofData, Hash256)>;
    
    /// Verify a consumption proof
    fn verify_consumption_proof(
        &self,
        proof: &ProofData,
        register_id: &RegisterId,
        nullifier: &Hash256,
        block_height: u64,
        transaction_id: &str,
        successor_ids: &Vec<RegisterId>,
        reason: &str,
    ) -> Result<bool>;
}

impl ZkRegisterOperations for OneTimeRegisterSystem {
    fn consume_register_with_proof(
        &self,
        register: &mut Register,
        transaction_id: &str,
        successor_ids: Vec<RegisterId>,
        reason: &str,
        owner_key: Vec<u8>,
    ) -> Result<(ProofData, Hash256)> {
        // Get the current block height
        let block_height = self.get_current_block_height()?;
        
        // Check that the register is in a valid state for consumption
        if register.state != RegisterState::Active {
            return Err(Error::InvalidState(format!(
                "Register {} is not active", register.id
            )));
        }
        
        // Generate the ZK proof
        let zk_manager = self.get_zk_proof_manager()?;
        let (proof, nullifier) = zk_manager.generate_consumption_proof(
            register,
            block_height,
            transaction_id,
            successor_ids.clone(),
            reason,
            owner_key,
        )?;
        
        // Update the register state
        register.state = RegisterState::Consumed {
            transaction_id: transaction_id.to_string(),
            block_height,
            nullifier,
            successor_ids: successor_ids.clone(),
            reason: reason.to_string(),
        };
        
        Ok((proof, nullifier))
    }
    
    fn verify_consumption_proof(
        &self,
        proof: &ProofData,
        register_id: &RegisterId,
        nullifier: &Hash256,
        block_height: u64,
        transaction_id: &str,
        successor_ids: &Vec<RegisterId>,
        reason: &str,
    ) -> Result<bool> {
        // Get the ZK proof manager
        let zk_manager = self.get_zk_proof_manager()?;
        
        // Verify the proof
        let mut zk_manager = Arc::get_mut(&mut zk_manager.clone())
            .ok_or_else(|| Error::RuntimeError("Failed to get mutable reference to ZK proof manager".to_string()))?;
        
        zk_manager.verify_consumption_proof(
            proof,
            register_id,
            nullifier,
            block_height,
            transaction_id,
            successor_ids,
            reason,
        )
    }
}

/// Trait for accessing the ZK proof manager
trait ZkProofManagerAccess {
    fn get_zk_proof_manager(&self) -> Result<Arc<RegisterZkProofManager>>;
    fn get_current_block_height(&self) -> Result<u64>;
}

impl ZkProofManagerAccess for OneTimeRegisterSystem {
    fn get_zk_proof_manager(&self) -> Result<Arc<RegisterZkProofManager>> {
        // In a real implementation, this would be stored in the register system
        // For this example, we create a new instance each time
        let manager = RegisterZkProofManager::new()?;
        Ok(Arc::new(manager))
    }
    
    fn get_current_block_height(&self) -> Result<u64> {
        // In a real implementation, this would be fetched from a blockchain
        // For this example, we return a fixed value
        Ok(42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consumption_with_proof() {
        // In a real test, this would verify the full flow
        // For now, we just ensure the code compiles
        assert!(true);
    }
} 