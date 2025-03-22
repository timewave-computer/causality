// Compatibility Layer for Legacy Fact System (DEPRECATED)
//
// This module provides compatibility types and functions for systems still
// using the old fact system interfaces. All types and functions in this module
// are considered DEPRECATED and will be removed in a future release.
//
// IMPORTANT: Do not use these types in new code. Use FactType and related
// types from crate::log::fact_types instead.

use std::collections::HashMap;
use serde_json::Value;
use chrono::{DateTime, Utc};

use crate::log::fact_types::{FactType, RegisterFact, ZKProofFact};
use crate::types::{DomainId, BlockHeight, BlockHash, Timestamp};
use crate::resource::register::RegisterId;
use crate::error::Result;

/// Legacy ObservedFact structure for compatibility
#[derive(Debug, Clone)]
pub struct ObservedFact {
    pub domain_id: DomainId,
    pub fact_type: String,
    pub block_height: BlockHeight,
    pub block_hash: BlockHash,
    pub timestamp: Timestamp,
    pub data: Vec<u8>,
    pub proof: Option<FactProof>,
    pub metadata: HashMap<String, String>,
}

/// Legacy FactProof structure for compatibility
#[derive(Debug, Clone)]
pub struct FactProof {
    pub proof_type: ProofType,
    pub proof_data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// Legacy ProofType enum for compatibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofType {
    Merkle,
    Signature,
    ZKP,
    Consensus,
    Custom(String),
}

/// Legacy VerifiedFact structure for compatibility
#[derive(Debug, Clone)]
pub struct VerifiedFact {
    pub fact: ObservedFact,
    pub verified: bool,
    pub verification_method: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// Convert a FactType to a legacy ObservedFact (for compatibility)
pub fn fact_type_to_observed_fact(
    fact_type: &FactType,
    domain_id: DomainId,
    block_height: BlockHeight,
    block_hash: BlockHash,
    timestamp: Timestamp,
    metadata: Option<HashMap<String, String>>,
) -> Result<ObservedFact> {
    let (fact_type_str, data) = match fact_type {
        FactType::BalanceFact => ("balance", serde_json::to_vec(&Value::Null)?),
        FactType::TransactionFact => ("transaction", serde_json::to_vec(&Value::Null)?),
        FactType::OracleFact => ("oracle", serde_json::to_vec(&Value::Null)?),
        FactType::BlockFact => ("block", serde_json::to_vec(&Value::Null)?),
        FactType::TimeFact => ("time", serde_json::to_vec(&Value::Null)?),
        FactType::RegisterFact(reg_fact) => {
            let (fact_type_str, data) = register_fact_to_observed(reg_fact)?;
            (fact_type_str, data)
        },
        FactType::ZKProofFact(zk_fact) => {
            let (fact_type_str, data) = zkproof_fact_to_observed(zk_fact)?;
            (fact_type_str, data)
        },
        FactType::Custom(name) => (name, serde_json::to_vec(&Value::Null)?),
    };
    
    Ok(ObservedFact {
        domain_id,
        fact_type: fact_type_str.to_string(),
        block_height,
        block_hash,
        timestamp,
        data,
        proof: None,
        metadata: metadata.unwrap_or_default(),
    })
}

/// Convert RegisterFact to ObservedFact data
fn register_fact_to_observed(fact: &RegisterFact) -> Result<(&'static str, Vec<u8>)> {
    match fact {
        RegisterFact::RegisterCreation { register_id, initial_data } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "register_id": register_id,
                "initial_data": hex::encode(initial_data),
            }))?;
            Ok(("register_creation", data))
        },
        RegisterFact::RegisterUpdate { register_id, new_data, previous_version } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "register_id": register_id,
                "new_data": hex::encode(new_data),
                "previous_version": previous_version,
            }))?;
            Ok(("register_update", data))
        },
        RegisterFact::RegisterTransfer { register_id, source_domain, target_domain } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "register_id": register_id,
                "source_domain": source_domain,
                "target_domain": target_domain,
            }))?;
            Ok(("register_transfer", data))
        },
        RegisterFact::RegisterMerge { source_registers, result_register } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "source_registers": source_registers,
                "result_register": result_register,
            }))?;
            Ok(("register_merge", data))
        },
        RegisterFact::RegisterSplit { source_register, result_registers } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "source_register": source_register,
                "result_registers": result_registers,
            }))?;
            Ok(("register_split", data))
        },
    }
}

/// Convert ZKProofFact to ObservedFact data
fn zkproof_fact_to_observed(fact: &ZKProofFact) -> Result<(&'static str, Vec<u8>)> {
    match fact {
        ZKProofFact::ProofVerification { verification_key_id, proof_hash, public_inputs, success } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "verification_key_id": verification_key_id,
                "proof_hash": proof_hash,
                "public_inputs": public_inputs,
                "success": success,
            }))?;
            Ok(("proof_verification", data))
        },
        ZKProofFact::BatchVerification { verification_key_ids, proof_hashes, public_inputs, success } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "verification_key_ids": verification_key_ids,
                "proof_hashes": proof_hashes,
                "public_inputs": public_inputs,
                "success": success,
            }))?;
            Ok(("batch_verification", data))
        },
        ZKProofFact::CircuitExecution { circuit_id, private_inputs_hash, public_inputs, generated_proof_hash } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "circuit_id": circuit_id,
                "private_inputs_hash": private_inputs_hash,
                "public_inputs": public_inputs,
                "generated_proof_hash": generated_proof_hash,
            }))?;
            Ok(("circuit_execution", data))
        },
        ZKProofFact::ProofComposition { source_proof_hashes, result_proof_hash, composition_circuit_id } => {
            let data = serde_json::to_vec(&serde_json::json!({
                "source_proof_hashes": source_proof_hashes,
                "result_proof_hash": result_proof_hash,
                "composition_circuit_id": composition_circuit_id,
            }))?;
            Ok(("proof_composition", data))
        },
    }
}

/// Attempt to convert an ObservedFact to a FactType
pub fn observed_fact_to_fact_type(fact: &ObservedFact) -> Result<FactType> {
    match fact.fact_type.as_str() {
        "balance" => Ok(FactType::BalanceFact),
        "transaction" => Ok(FactType::TransactionFact),
        "oracle" => Ok(FactType::OracleFact),
        "block" => Ok(FactType::BlockFact),
        "time" => Ok(FactType::TimeFact),
        
        "register_creation" => {
            let value: Value = serde_json::from_slice(&fact.data)?;
            let register_id: RegisterId = serde_json::from_value(value["register_id"].clone())?;
            let initial_data = hex::decode(value["initial_data"].as_str().unwrap_or(""))?;
            
            Ok(FactType::RegisterFact(RegisterFact::RegisterCreation {
                register_id,
                initial_data,
            }))
        },
        
        "register_update" => {
            let value: Value = serde_json::from_slice(&fact.data)?;
            let register_id: RegisterId = serde_json::from_value(value["register_id"].clone())?;
            let new_data = hex::decode(value["new_data"].as_str().unwrap_or(""))?;
            let previous_version = value["previous_version"].as_str().unwrap_or("").to_string();
            
            Ok(FactType::RegisterFact(RegisterFact::RegisterUpdate {
                register_id,
                new_data,
                previous_version,
            }))
        },
        
        "register_transfer" => {
            let value: Value = serde_json::from_slice(&fact.data)?;
            let register_id: RegisterId = serde_json::from_value(value["register_id"].clone())?;
            let source_domain = value["source_domain"].as_str().unwrap_or("").to_string();
            let target_domain = value["target_domain"].as_str().unwrap_or("").to_string();
            
            Ok(FactType::RegisterFact(RegisterFact::RegisterTransfer {
                register_id,
                source_domain,
                target_domain,
            }))
        },
        
        "register_merge" => {
            let value: Value = serde_json::from_slice(&fact.data)?;
            let source_registers: Vec<RegisterId> = serde_json::from_value(value["source_registers"].clone())?;
            let result_register: RegisterId = serde_json::from_value(value["result_register"].clone())?;
            
            Ok(FactType::RegisterFact(RegisterFact::RegisterMerge {
                source_registers,
                result_register,
            }))
        },
        
        "register_split" => {
            let value: Value = serde_json::from_slice(&fact.data)?;
            let source_register: RegisterId = serde_json::from_value(value["source_register"].clone())?;
            let result_registers: Vec<RegisterId> = serde_json::from_value(value["result_registers"].clone())?;
            
            Ok(FactType::RegisterFact(RegisterFact::RegisterSplit {
                source_register,
                result_registers,
            }))
        },
        
        "proof_verification" => {
            let value: Value = serde_json::from_slice(&fact.data)?;
            let verification_key_id = value["verification_key_id"].as_str().unwrap_or("").to_string();
            let proof_hash = value["proof_hash"].as_str().unwrap_or("").to_string();
            let public_inputs: Vec<String> = serde_json::from_value(value["public_inputs"].clone()).unwrap_or_default();
            let success = value["success"].as_bool().unwrap_or(false);
            
            Ok(FactType::ZKProofFact(ZKProofFact::ProofVerification {
                verification_key_id,
                proof_hash,
                public_inputs,
                success,
            }))
        },
        
        // Handle other ZKProofFact variants...
        
        _ => Ok(FactType::Custom(fact.fact_type.clone())),
    }
}

/// Convert a VerifiedFact to a pair of (FactType, metadata)
pub fn verified_fact_to_fact_type(fact: &VerifiedFact) -> Result<(FactType, HashMap<String, String>)> {
    let fact_type = observed_fact_to_fact_type(&fact.fact)?;
    
    let mut metadata = fact.metadata.clone();
    metadata.insert("verified".to_string(), fact.verified.to_string());
    
    if let Some(method) = &fact.verification_method {
        metadata.insert("verification_method".to_string(), method.clone());
    }
    
    if let Some(verified_at) = fact.verified_at {
        metadata.insert("verified_at".to_string(), verified_at.to_rfc3339());
    }
    
    metadata.insert("confidence".to_string(), fact.confidence.to_string());
    
    Ok((fact_type, metadata))
}

/// Create a VerifiedFact from a FactType (for compatibility)
pub fn fact_type_to_verified_fact(
    fact_type: &FactType,
    domain_id: DomainId,
    block_height: BlockHeight,
    block_hash: BlockHash,
    timestamp: Timestamp,
    verified: bool,
    verification_method: Option<String>,
    confidence: f64,
    metadata: Option<HashMap<String, String>>,
) -> Result<VerifiedFact> {
    let observed_fact = fact_type_to_observed_fact(
        fact_type,
        domain_id,
        block_height,
        block_hash,
        timestamp,
        metadata.clone(),
    )?;
    
    Ok(VerifiedFact {
        fact: observed_fact,
        verified,
        verification_method,
        verified_at: Some(Utc::now()),
        confidence,
        metadata: metadata.unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResourceId;
    
    #[test]
    fn test_round_trip_conversion_register_fact() -> Result<()> {
        let register_id = ResourceId::new("reg123");
        let initial_data = vec![1, 2, 3, 4];
        
        let fact = FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id: register_id.clone(),
            initial_data: initial_data.clone(),
        });
        
        let domain_id = DomainId::new("test_domain");
        let block_height = 1000.into();
        let block_hash = "0xabc123".into();
        let timestamp = 1600000000.into();
        
        let observed = fact_type_to_observed_fact(
            &fact,
            domain_id.clone(),
            block_height,
            block_hash.clone(),
            timestamp,
            None,
        )?;
        
        assert_eq!(observed.fact_type, "register_creation");
        assert_eq!(observed.domain_id, domain_id);
        assert_eq!(observed.block_hash, block_hash);
        
        let roundtrip_fact = observed_fact_to_fact_type(&observed)?;
        
        if let FactType::RegisterFact(RegisterFact::RegisterCreation { register_id: r_id, initial_data: r_data }) = roundtrip_fact {
            assert_eq!(r_id, register_id);
            assert_eq!(r_data, initial_data);
        } else {
            panic!("Expected RegisterFact::RegisterCreation, got: {:?}", roundtrip_fact);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_verified_fact_conversion() -> Result<()> {
        let register_id = ResourceId::new("reg123");
        let initial_data = vec![1, 2, 3, 4];
        
        let fact = FactType::RegisterFact(RegisterFact::RegisterCreation {
            register_id,
            initial_data,
        });
        
        let domain_id = DomainId::new("test_domain");
        let block_height = 1000.into();
        let block_hash = "0xabc123".into();
        let timestamp = 1600000000.into();
        
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        
        let verified = fact_type_to_verified_fact(
            &fact,
            domain_id,
            block_height,
            block_hash,
            timestamp,
            true,
            Some("merkle_proof".to_string()),
            0.99,
            Some(metadata),
        )?;
        
        assert_eq!(verified.verified, true);
        assert_eq!(verified.verification_method, Some("merkle_proof".to_string()));
        assert!(verified.confidence > 0.98);
        assert_eq!(verified.metadata.get("test_key"), Some(&"test_value".to_string()));
        
        let (converted_fact, converted_metadata) = verified_fact_to_fact_type(&verified)?;
        
        assert_eq!(converted_metadata.get("verified"), Some(&"true".to_string()));
        assert_eq!(converted_metadata.get("verification_method"), Some(&"merkle_proof".to_string()));
        
        match converted_fact {
            FactType::RegisterFact(_) => {}
            _ => panic!("Expected RegisterFact, got: {:?}", converted_fact),
        }
        
        Ok(())
    }
} 